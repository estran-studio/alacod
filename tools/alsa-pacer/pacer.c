/*
 * Carte son virtuelle à l'heure, pour écouter un jeu sans carte son (M-7b, docs/conventions.md § 124).
 *
 * Plugin ALSA « ioplug » chargé par processus (ALSA_CONFIG_PATH, voir scripts/audio-capture.sh), jamais par un
 * ~/.asoundrc global. Chaque flux de sortie ouvert par le processus (kira, harmonium : deux flux cpal) devient un fichier
 * WAV à lui, sur l'horloge murale depuis l'ouverture de **ce flux** :
 *
 *   - la carte consomme les échantillons au rythme réel (CLOCK_MONOTONIC × fréquence), comme une vraie carte : un flux en
 *     retard voit un « underrun » (xrun signalé à l'application, -EPIPE, et silence dans le fichier à l'endroit exact) ;
 *   - un flux arrêté ou en pause écrit du silence (sans compter d'underrun) : son fichier suit l'horloge murale sans trou ;
 *     deux flux d'un processus n'ouvrent pas au même instant : la première ligne du journal donne l'origine du fichier
 *     (`monotonic=`, s), qui ramène les deux dans un même repère (scripts/audio-gaps.py) ;
 *   - à côté du WAV, un journal texte : ouverture, format, démarrages, arrêts, pauses, underruns (instant, durée) et, à la
 *     fermeture, la marge minimale du tampon (frames d'avance de l'application sur la carte) et le plus long écart entre
 *     deux écritures.
 *
 * Sortie : <prefix>-<pid>-<n>.wav et .log (n : ordre d'ouverture des flux dans le processus). WAV flottant 32 bits.
 * Construction : cc -shared -fPIC -O2 -o libasound_module_pcm_pacer.so pacer.c -lasound
 */
#include <alsa/asoundlib.h>
#include <alsa/pcm_external.h>
#include <errno.h>
#include <poll.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/timerfd.h>
#include <time.h>
#include <unistd.h>

#define RATE 48000
#define CHANNELS 2

static atomic_int stream_counter = 0;

typedef struct {
	snd_pcm_ioplug_t io;
	FILE *wav;
	FILE *log;
	double t_open;      /* s, CLOCK_MONOTONIC */
	uint64_t written;   /* frames écrits dans le WAV (horloge murale) */
	float *fifo;        /* échantillons reçus de l'application, pas encore joués */
	snd_pcm_uframes_t fifo_cap, fifo_len, fifo_head;
	uint64_t consumed;  /* frames de données jouées depuis le dernier prepare (hw_ptr) */
	snd_pcm_uframes_t boundary;
	int running;
	int xrun;
	/* mesures */
	uint64_t underruns, underrun_frames, starts, stops, transfers;
	long min_margin;    /* frames, -1 : jamais mesuré */
	double last_transfer, max_transfer_gap;
} pacer_t;

static double now_s(void)
{
	struct timespec ts;
	clock_gettime(CLOCK_MONOTONIC, &ts);
	return (double)ts.tv_sec + (double)ts.tv_nsec / 1e9;
}

static void wav_header(FILE *f, uint32_t frames)
{
	uint32_t data = frames * CHANNELS * 4;
	uint32_t riff = 36 + data, fmt_len = 16, rate = RATE, byte_rate = RATE * CHANNELS * 4;
	uint16_t format = 3, channels = CHANNELS, align = CHANNELS * 4, bits = 32;
	fseek(f, 0, SEEK_SET);
	fwrite("RIFF", 1, 4, f);
	fwrite(&riff, 4, 1, f);
	fwrite("WAVEfmt ", 1, 8, f);
	fwrite(&fmt_len, 4, 1, f);
	fwrite(&format, 2, 1, f);
	fwrite(&channels, 2, 1, f);
	fwrite(&rate, 4, 1, f);
	fwrite(&byte_rate, 4, 1, f);
	fwrite(&align, 2, 1, f);
	fwrite(&bits, 2, 1, f);
	fwrite("data", 1, 4, f);
	fwrite(&data, 4, 1, f);
	fseek(f, 0, SEEK_END);
}

static double t_rel(pacer_t *p)
{
	return now_s() - p->t_open;
}

/* Avance le fichier jusqu'à maintenant : données de la file si la carte tourne, silence sinon. */
static void advance(pacer_t *p)
{
	uint64_t target = (uint64_t)(t_rel(p) * RATE);
	if (target <= p->written)
		return;
	uint64_t before = p->written;
	uint64_t todo = target - p->written;
	static const float zero[CHANNELS * 256];
	while (todo > 0) {
		if (p->running && p->fifo_len > 0) {
			snd_pcm_uframes_t n = p->fifo_len;
			if (n > todo)
				n = todo;
			if (n > p->fifo_cap - p->fifo_head)
				n = p->fifo_cap - p->fifo_head;
			fwrite(p->fifo + p->fifo_head * CHANNELS, sizeof(float) * CHANNELS, n, p->wav);
			p->fifo_head = (p->fifo_head + n) % p->fifo_cap;
			p->fifo_len -= n;
			p->consumed += n;
			p->written += n;
			todo -= n;
			continue;
		}
		if (p->running && !p->xrun) {
			/* La carte tourne et l'application n'a rien fourni : underrun. */
			p->xrun = 1;
			p->underruns++;
			fprintf(p->log, "t=%.4f underrun\n", (double)p->written / RATE);
		}
		if (p->running)
			p->underrun_frames += todo;
		while (todo > 0) {
			uint64_t n = todo > 256 ? 256 : todo;
			fwrite(zero, sizeof(float) * CHANNELS, n, p->wav);
			p->written += n;
			todo -= n;
		}
	}
	/* En-tête à jour chaque seconde : un processus qui sort sans fermer le flux laisse un WAV lisible. */
	if (before / RATE != p->written / RATE)
		wav_header(p->wav, (uint32_t)p->written);
}

static int pacer_start(snd_pcm_ioplug_t *io)
{
	pacer_t *p = io->private_data;
	advance(p);
	p->running = 1;
	p->xrun = 0;
	p->starts++;
	fprintf(p->log, "t=%.4f start fifo=%lu\n", t_rel(p), (unsigned long)p->fifo_len);
	return 0;
}

static int pacer_stop(snd_pcm_ioplug_t *io)
{
	pacer_t *p = io->private_data;
	advance(p);
	p->running = 0;
	p->stops++;
	fprintf(p->log, "t=%.4f stop\n", t_rel(p));
	return 0;
}

static int pacer_pause(snd_pcm_ioplug_t *io, int enable)
{
	pacer_t *p = io->private_data;
	advance(p);
	p->running = !enable;
	fprintf(p->log, "t=%.4f %s\n", t_rel(p), enable ? "pause" : "reprise");
	return 0;
}

static snd_pcm_sframes_t pacer_pointer(snd_pcm_ioplug_t *io)
{
	pacer_t *p = io->private_data;
	advance(p);
	if (p->running) {
		long margin = (long)p->fifo_len;
		if (p->min_margin < 0 || margin < p->min_margin)
			p->min_margin = margin;
	}
	if (p->xrun && p->running)
		return -EPIPE;
	return (snd_pcm_sframes_t)(p->consumed % p->boundary);
}

static snd_pcm_sframes_t pacer_transfer(snd_pcm_ioplug_t *io, const snd_pcm_channel_area_t *areas,
					snd_pcm_uframes_t offset, snd_pcm_uframes_t size)
{
	pacer_t *p = io->private_data;
	advance(p);
	double t = now_s();
	if (p->transfers > 0 && p->running && t - p->last_transfer > p->max_transfer_gap)
		p->max_transfer_gap = t - p->last_transfer;
	p->last_transfer = t;
	p->transfers++;
	if (size > p->fifo_cap - p->fifo_len)
		size = p->fifo_cap - p->fifo_len;
	for (snd_pcm_uframes_t i = 0; i < size; i++) {
		snd_pcm_uframes_t slot = (p->fifo_head + p->fifo_len + i) % p->fifo_cap;
		for (unsigned c = 0; c < io->channels && c < CHANNELS; c++) {
			const snd_pcm_channel_area_t *a = &areas[c];
			const char *base = (const char *)a->addr + (a->first + (offset + i) * a->step) / 8;
			float v = io->format == SND_PCM_FORMAT_FLOAT_LE ? *(const float *)base
								      : (float)*(const int16_t *)base / 32768.0f;
			p->fifo[slot * CHANNELS + c] = v;
		}
		if (io->channels == 1)
			p->fifo[slot * CHANNELS + 1] = p->fifo[slot * CHANNELS];
	}
	p->fifo_len += size;
	return (snd_pcm_sframes_t)size;
}

static int pacer_prepare(snd_pcm_ioplug_t *io)
{
	pacer_t *p = io->private_data;
	advance(p);
	p->running = 0;
	p->xrun = 0;
	p->fifo_len = 0;
	p->fifo_head = 0;
	p->consumed = 0;
	return 0;
}

static int pacer_hw_params(snd_pcm_ioplug_t *io, snd_pcm_hw_params_t *params)
{
	pacer_t *p = io->private_data;
	(void)params;
	free(p->fifo);
	p->fifo_cap = io->buffer_size * 2;
	p->fifo = calloc(p->fifo_cap * CHANNELS, sizeof(float));
	p->fifo_len = p->fifo_head = 0;
	fprintf(p->log, "t=%.4f hw_params format=%s channels=%u rate=%u period=%lu buffer=%lu\n", t_rel(p),
		snd_pcm_format_name(io->format), io->channels, io->rate, (unsigned long)io->period_size,
		(unsigned long)io->buffer_size);
	return p->fifo ? 0 : -ENOMEM;
}

static int pacer_sw_params(snd_pcm_ioplug_t *io, snd_pcm_sw_params_t *params)
{
	pacer_t *p = io->private_data;
	snd_pcm_uframes_t boundary = 0;
	if (snd_pcm_sw_params_get_boundary(params, &boundary) == 0 && boundary > 0)
		p->boundary = boundary;
	(void)io;
	return 0;
}

static int pacer_poll_revents(snd_pcm_ioplug_t *io, struct pollfd *pfd, unsigned int nfds, unsigned short *revents)
{
	pacer_t *p = io->private_data;
	uint64_t ticks;
	(void)nfds;
	if (read(pfd[0].fd, &ticks, sizeof ticks) < 0 && errno != EAGAIN)
		return -errno;
	advance(p);
	snd_pcm_uframes_t avail = io->buffer_size > p->fifo_len ? io->buffer_size - p->fifo_len : 0;
	*revents = (avail >= io->period_size || p->xrun || !p->running) ? POLLOUT : 0;
	return 0;
}

static int pacer_close(snd_pcm_ioplug_t *io)
{
	pacer_t *p = io->private_data;
	advance(p);
	fprintf(p->log,
		"t=%.4f close frames=%llu underruns=%llu underrun_ms=%.1f starts=%llu stops=%llu transfers=%llu "
		"min_margin_frames=%ld max_transfer_gap_ms=%.1f\n",
		t_rel(p), (unsigned long long)p->written, (unsigned long long)p->underruns,
		(double)p->underrun_frames * 1000.0 / RATE, (unsigned long long)p->starts,
		(unsigned long long)p->stops, (unsigned long long)p->transfers, p->min_margin,
		p->max_transfer_gap * 1000.0);
	wav_header(p->wav, (uint32_t)p->written);
	fclose(p->wav);
	fclose(p->log);
	close(io->poll_fd);
	free(p->fifo);
	free(p);
	return 0;
}

static const snd_pcm_ioplug_callback_t pacer_callbacks = {
	.start = pacer_start,
	.stop = pacer_stop,
	.pointer = pacer_pointer,
	.transfer = pacer_transfer,
	.close = pacer_close,
	.hw_params = pacer_hw_params,
	.sw_params = pacer_sw_params,
	.prepare = pacer_prepare,
	.pause = pacer_pause,
	.poll_revents = pacer_poll_revents,
};

SND_PCM_PLUGIN_DEFINE_FUNC(pacer)
{
	snd_config_iterator_t i, next;
	const char *prefix = "/tmp/pacer";
	(void)root;
	snd_config_for_each(i, next, conf) {
		snd_config_t *n = snd_config_iterator_entry(i);
		const char *id;
		if (snd_config_get_id(n, &id) < 0)
			continue;
		if (!strcmp(id, "comment") || !strcmp(id, "type") || !strcmp(id, "hint"))
			continue;
		if (!strcmp(id, "prefix")) {
			snd_config_get_string(n, &prefix);
			continue;
		}
		SNDERR("pacer : champ inconnu %s", id);
		return -EINVAL;
	}
	if (stream != SND_PCM_STREAM_PLAYBACK)
		return -EINVAL;
	const char *env = getenv("ALACOD_PACER_PREFIX");
	if (env && *env)
		prefix = env;

	pacer_t *p = calloc(1, sizeof *p);
	if (!p)
		return -ENOMEM;
	int index = atomic_fetch_add(&stream_counter, 1);
	char path[4096];
	snprintf(path, sizeof path, "%s-%d-%d.wav", prefix, (int)getpid(), index);
	p->wav = fopen(path, "wb");
	snprintf(path, sizeof path, "%s-%d-%d.log", prefix, (int)getpid(), index);
	p->log = fopen(path, "w");
	if (!p->wav || !p->log) {
		free(p);
		return -EIO;
	}
	setvbuf(p->log, NULL, _IOLBF, 0);
	wav_header(p->wav, 0);
	p->t_open = now_s();
	p->boundary = 1UL << 30;
	p->min_margin = -1;
	/* monotonic : instant d'ouverture (CLOCK_MONOTONIC, s), pour aligner les WAV de plusieurs flux d'un processus. */
	fprintf(p->log, "open pid=%d stream=%d name=%s monotonic=%.4f\n", (int)getpid(), index, name ? name : "?",
		p->t_open);

	int fd = timerfd_create(CLOCK_MONOTONIC, TFD_NONBLOCK | TFD_CLOEXEC);
	struct itimerspec spec = {{0, 1000000}, {0, 1000000}}; /* 1 ms */
	timerfd_settime(fd, 0, &spec, NULL);

	p->io.version = SND_PCM_IOPLUG_VERSION;
	p->io.name = "alacod pacer";
	p->io.callback = &pacer_callbacks;
	p->io.private_data = p;
	p->io.poll_fd = fd;
	p->io.poll_events = POLLIN;
	p->io.mmap_rw = 0;
	p->io.flags = SND_PCM_IOPLUG_FLAG_BOUNDARY_WA | SND_PCM_IOPLUG_FLAG_MONOTONIC;

	int err = snd_pcm_ioplug_create(&p->io, name, stream, mode);
	if (err < 0) {
		fclose(p->wav);
		fclose(p->log);
		close(fd);
		free(p);
		return err;
	}
	unsigned int access[] = {SND_PCM_ACCESS_RW_INTERLEAVED};
	unsigned int formats[] = {SND_PCM_FORMAT_FLOAT_LE, SND_PCM_FORMAT_S16_LE};
	snd_pcm_ioplug_set_param_list(&p->io, SND_PCM_IOPLUG_HW_ACCESS, 1, access);
	snd_pcm_ioplug_set_param_list(&p->io, SND_PCM_IOPLUG_HW_FORMAT, 2, formats);
	snd_pcm_ioplug_set_param_minmax(&p->io, SND_PCM_IOPLUG_HW_CHANNELS, 1, CHANNELS);
	snd_pcm_ioplug_set_param_minmax(&p->io, SND_PCM_IOPLUG_HW_RATE, RATE, RATE);
	snd_pcm_ioplug_set_param_minmax(&p->io, SND_PCM_IOPLUG_HW_PERIOD_BYTES, 64, 64 * 1024);
	snd_pcm_ioplug_set_param_minmax(&p->io, SND_PCM_IOPLUG_HW_PERIODS, 2, 64);
	snd_pcm_ioplug_set_param_minmax(&p->io, SND_PCM_IOPLUG_HW_BUFFER_BYTES, 256, 1024 * 1024);
	*pcmp = p->io.pcm;
	return 0;
}

SND_PCM_PLUGIN_SYMBOL(pacer);
