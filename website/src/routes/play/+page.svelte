<script lang="ts">
	import { onMount } from 'svelte';
	import { findGame, getReleases, type BuildManifest } from '$lib/game/catalog';
	let game = $state<ReturnType<typeof findGame>>();
	let manifest = $state<BuildManifest>();
	let error = $state('');
	let loading = $state(true);
	let started = $state(false);
	let status = $state('');
	let frame = $state<HTMLIFrameElement>();
	let frameSrc = $state('');

	onMount(() => {
		let active = true;
		const controller = new AbortController();
		async function prepare() {
			try {
				const params = new URLSearchParams(window.location.search);
				game = findGame(params.get('id'));
				if (!game) throw new Error('Ce jeu ne figure pas au catalogue.');
				if (params.get('online') === 'true')
					throw new Error('Ce lien utilise l’ancien lancement en ligne. Retournez au catalogue pour rejoindre une version compatible.');
				const releases = await getReleases();
				const release = releases.games[game.id];
				if (!release?.solo) throw new Error('La version navigateur de ce prototype est encore en préparation.');
				const response = await fetch(release.manifest, { signal: controller.signal, cache: 'no-store' });
				if (!response.ok) throw new Error('Le manifeste de cette version est indisponible.');
				const build: BuildManifest = await response.json();
				if (build.schemaVersion !== 1 || build.gameId !== game.id || build.buildId !== release.buildId)
					throw new Error('La version publiée ne correspond pas au catalogue.');
				const root = `/builds/${encodeURIComponent(build.buildId)}/${game.id}/`;
				if (!/^[a-zA-Z0-9._-]+$/.test(build.buildId) || build.module !== `${root}wasm.js` || build.wasm !== `${root}wasm_bg.wasm`)
					throw new Error('Chemins de chargement invalides.');
				if (active) manifest = build;
			} catch (e) {
				if (active) error = e instanceof Error ? e.message : 'Impossible de préparer le jeu.';
			} finally {
				if (active) loading = false;
			}
		}
		void prepare();
		const listener = (event: MessageEvent) => {
			if (event.origin !== window.location.origin || event.source !== frame?.contentWindow || event.data?.source !== 'alacod-loader')
				return;
			if (event.data.type === 'error') {
				error = event.data.message;
				status = '';
			}
			if (event.data.type === 'ready') status = 'Jeu lancé · Cliquez dans le jeu pour prendre les contrôles.';
		};
		window.addEventListener('message', listener);
		return () => {
			active = false;
			controller.abort();
			window.removeEventListener('message', listener);
		};
	});

	function start() {
		if (!manifest || !game) return;
		error = '';
		status = 'Chargement du jeu…';
		const params = new URLSearchParams({ game: game.id, build: manifest.buildId });
		frameSrc = `/loader.html?${params}`;
		started = true;
	}
	async function fullscreen() {
		try {
			await frame?.requestFullscreen();
		} catch {
			status = 'Le plein écran n’est pas disponible dans ce navigateur.';
		}
	}
</script>

<svelte:head><title>{game?.name || 'Jouer'} — Alacod</title></svelte:head>
<section class="section-wrap page-intro">
	<span class="eyebrow">JOUER / SOLO</span>
	<h1>{game?.name || 'Votre partie'}</h1>
	<p>Une partie locale, sans compte et sans serveur multijoueur.</p>
</section>
<section class="section-wrap">
	{#if error}<div class="notice" role="alert">{error} <a href="/games">Retour aux jeux</a></div>{/if}
	{#if loading}<p role="status">Vérification de la version…</p>{:else if manifest && game}
		<div class="play-controls">
			<p class="eyebrow">VERSION {manifest.buildId}</p>
			<a class="text-link" href="/games">← Quitter vers le catalogue</a>
		</div>
		{#if !started}<div class="play-panel">
				<h2>Prêt à jouer ?</h2>
				<p style="margin-top:16px">Cliquez dans le jeu pour activer les contrôles. Le premier téléchargement peut prendre un moment.</p>
				<div class="controls-list">
					<span><kbd>W A S D</kbd> Déplacement</span><span><kbd>Souris</kbd> Visée · clic pour tirer</span><span
						><kbd>R</kbd> Recharger</span
					><span><kbd>H</kbd> Interagir</span><span><kbd>Tab</kbd> Changer d’arme</span><span><kbd>C</kbd> Dash</span
					>{#if game.id === 'throne'}<span><kbd>1 2 3</kbd> Mutation</span>{/if}
				</div>
				<button class="button primary" onclick={start}>Lancer la partie ↗</button>
			</div>
		{:else}<iframe class="play-frame" title="Partie de {game.name}" src={frameSrc} bind:this={frame} allow="autoplay; fullscreen"></iframe>
			<div class="play-controls">
				<p role="status">{status}</p>
				<button class="button" onclick={fullscreen}>Plein écran ↗</button>
			</div>{/if}
	{/if}
</section>
