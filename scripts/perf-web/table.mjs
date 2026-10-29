// Tableau Markdown depuis les séries JSON de docs/taches/rapports/polissage-perf-web/ (même résumé que measure-helpers.js).
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
const dir = process.argv[2] || 'docs/taches/rapports/polissage-perf-web';
const pct = (a, p) => { const s = [...a].sort((x, y) => x - y); return s[Math.min(s.length - 1, Math.floor(p * s.length))]; };
const rows = [];
for (const file of readdirSync(dir).filter((f) => f.endsWith('.json')).sort()) {
	const r = JSON.parse(readFileSync(join(dir, file), 'utf8'));
	const all = r.samples[0].slice(2);
	const restarts = all.filter((x, j) => j > 0 && x.frame < all[j - 1].frame - 100).length;
	const s = all.filter((x, j) => j > 0 && x.frame >= all[j - 1].frame && x.entities > 0);
	const col = (k) => s.map((x) => x[k]);
	const avg = (a) => a.reduce((x, y) => x + y, 0) / a.length;
	rows.push(`| ${file.replace('.json', '')} | ${pct(col('fps_display'), 0.5).toFixed(1)} / ${pct(col('fps_display'), 0.05).toFixed(1)} | ${pct(col('fps_sim'), 0.5).toFixed(1)} / ${pct(col('fps_sim'), 0.05).toFixed(1)} | ${avg(col('advance_avg_ms')).toFixed(2)} / ${Math.max(...col('advance_max_ms')).toFixed(1)} | ${Math.max(...col('frame_max_ms')).toFixed(0)} | ${Math.min(...col('entities'))}–${Math.max(...col('entities'))} | ${restarts} | ${r.busy[0].main_busy_pct} | ${(r.firstGame[0].ms / 1000).toFixed(1)} | ${r.before.uptime.split(' ')[0]}→${r.after.uptime.split(' ')[0]} | ${r.before.power.replace(/'/g, '')}→${r.after.power.replace(/'/g, '')} |`);
}
console.log('| série | affichage méd. / p5 (fps) | sim méd. / p5 (fps) | pas sim moy. / max (ms) | frame max (ms) | entités | relances | fil principal occupé (%) | chargement (s) | charge Mac (1 min) | alimentation |');
console.log('|---|---|---|---|---|---|---|---|---|---|---|');
console.log(rows.join('\n'));
