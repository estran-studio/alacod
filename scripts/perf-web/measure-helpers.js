// Aides de mesure S1-5 pour la cellule JS d'`eval` d'omp (global `browser`) ; chargées par
// `eval(require('node:fs').readFileSync('scripts/perf-web/measure-helpers.js', 'utf8'))`. Voir docs/taches/rapports/polissage-perf-web.md § 1.
// Prérequis : un serveur statique sur 127.0.0.1:8778 (dossier website/static, builds dans builds/<id>/<jeu>/).
globalThis.botSrc = require('node:fs').readFileSync('scripts/perf-web/gamepad-bot.js', 'utf8');
// Aucun son sur le Mac sans demande de William : le navigateur géré d'omp est un Chrome headless (`--headless=new --mute-audio`,
// pas de sortie audio) ; on refuse de mesurer si son processus principal n'a pas `--mute-audio`, et la page ne peut de toute façon
// pas démarrer un AudioContext (suspendu en permanence par le script d'init ci-dessous).
globalThis.muteStub = `(() => { for (const name of ['AudioContext', 'webkitAudioContext']) { const N = window[name]; if (!N) continue; window[name] = class extends N { constructor(o) { super(o); this.suspend(); } resume() { return Promise.resolve(); } }; } })();`;
globalThis.assertMuted = async () => {
	const out = await Bun.$`sh -c "ps -Ao command | grep 'omp.browser.headless.profile' | grep -v -- '--type=' | grep -v grep || true"`.text();
	const mains = out.split('\n').filter((l) => l.includes('--headless') && l.includes('Google Chrome for Testing'));
	if (!mains.length || !mains.every((l) => l.includes('--mute-audio'))) throw new Error('navigateur géré non muet : mesure refusée');
};
globalThis.openTab = async (name) =>
	browser.open({ name, url: 'about:blank', headed: true, viewport: { width: 1280, height: 720 }, app: { tern: false, relay: false }, init_scripts: [globalThis.botSrc, globalThis.muteStub] });
globalThis.loadInfo = async () => {
	const uptime = (await Bun.$`uptime`.text()).trim().replace(/.*load averages?: /, '');
	const busy = Number((await Bun.$`sh -c 'ps -Ao comm | grep -cE "rustc|cargo|scenarios-|wasm-opt|clang" || true'`.text()).trim());
	const power = (await Bun.$`sh -c "pmset -g batt | sed -n 1p"`.text()).trim().replace(/^Now drawing from /, '');
	return { uptime, busy_procs: busy, power };
};
// tabs = [{ name, url }], secs = durée de la partie après le premier échantillon en jeu (frame > 0), cold = cache désactivé.
globalThis.measure = async ({ tabs, secs, cold }) => {
	await globalThis.assertMuted();
	const before = await globalThis.loadInfo();
	const opened = [];
	for (const t of tabs) opened.push(await globalThis.openTab(t.name));
	if (cold) for (const tab of opened) await tab.run(async ({ page }) => { await page.setCacheEnabled(false); });
	await Promise.all(tabs.map((t, i) => opened[i].goto(t.url, { wait_until: 'domcontentloaded' })));
	const probe = `(() => { try { const c = document.getElementById('bevy-canvas'); const s = JSON.parse(c.getAttribute('data-perf-sample')); if (!s || !(s.frame > 0)) return null; window.__log = [s]; new MutationObserver(() => window.__log.push(JSON.parse(c.getAttribute('data-perf-sample')))).observe(c, { attributes: true, attributeFilter: ['data-perf-sample'] }); const w = performance.getEntriesByType('resource').find(r => r.name.endsWith('wasm_bg.wasm')); return { ms: Math.round(performance.now()), wasm_response_end_ms: w ? Math.round(w.responseEnd) : null, wasm_transfer: w ? w.transferSize : null }; } catch (e) { return null; } })()`;
	const firstGame = await Promise.all(
		opened.map(async (tab) => {
			for (let i = 0; i < 240; i++) {
				try {
					const found = await tab.evaluate(probe);
					if (found) return found;
				} catch (e) {
					// page en cours de navigation : on réessaie
				}
				await new Promise((r) => setTimeout(r, 500));
			}
			throw new Error('le jeu n\'a pas démarré en 120 s');
		})
	);
	const m0 = await Promise.all(opened.map((tab) => tab.metrics()));
	await new Promise((r) => setTimeout(r, secs * 1000));
	const m1 = await Promise.all(opened.map((tab) => tab.metrics()));
	// Part du temps où le fil principal (le jeu entier : simulation, ECS, appels WebGL) est occupé : l'indicateur qui ne dépend
	// ni de la synchro verticale ni du GPU.
	const busy = m0.map((a, i) => ({ main_busy_pct: +((100 * (m1[i].TaskDuration - a.TaskDuration)) / (m1[i].Timestamp - a.Timestamp)).toFixed(1), script_pct: +((100 * (m1[i].ScriptDuration - a.ScriptDuration)) / (m1[i].Timestamp - a.Timestamp)).toFixed(1) }));
	const after = await globalThis.loadInfo();
	const samples = [];
	for (const tab of opened) samples.push(await tab.evaluate('window.__log'));
	return { before, after, firstGame, busy, secs, samples, tabs: opened };
};
const pct = (a, p) => { const s = [...a].sort((x, y) => x - y); return s[Math.min(s.length - 1, Math.floor(p * s.length))]; };
// Résumé : on écarte les 2 premières secondes et les secondes de relance (frame qui recule, 0 entité), comptées à part.
globalThis.summarize = (label, r) =>
	r.samples.map((samples, i) => {
		const all = samples.slice(2);
		const restarts = all.filter((x, j) => j > 0 && x.frame < all[j - 1].frame - 100).length;
		const s = all.filter((x, j) => j > 0 && x.frame >= all[j - 1].frame && x.entities > 0);
		const col = (k) => s.map((x) => x[k]);
		const sum = (a) => a.reduce((x, y) => x + y, 0);
		const r2 = (x) => +x.toFixed(2);
		return {
			label: `${label}#${i}`, secs: all.length, kept: s.length, restarts,
			disp_med: r2(pct(col('fps_display'), 0.5)), disp_p5: r2(pct(col('fps_display'), 0.05)), disp_min: Math.min(...col('fps_display')),
			sim_med: r2(pct(col('fps_sim'), 0.5)), sim_p5: r2(pct(col('fps_sim'), 0.05)), sim_min: Math.min(...col('fps_sim')),
			adv_avg_ms: r2(sum(col('advance_avg_ms')) / s.length), adv_max_ms: Math.max(...col('advance_max_ms')), frame_max_ms: Math.max(...col('frame_max_ms')),
			rollbacks: sum(col('rollbacks')), ent_min: Math.min(...col('entities')), ent_max: Math.max(...col('entities')),
			...r.busy[i], load_ms: r.firstGame[i].ms, load: `${r.before.uptime} → ${r.after.uptime}`, busy: `${r.before.busy_procs}/${r.after.busy_procs}`, power: `${r.before.power} / ${r.after.power}`
		};
	});
