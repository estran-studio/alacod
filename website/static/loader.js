// The loader only accepts catalog game IDs and immutable build names. Solo deliberately
// sets no signaling attributes and does not access Allumette or saved credentials.
const canvas = document.getElementById('bevy-canvas');
const send = (type, message) => window.parent.postMessage({ source: 'alacod-loader', type, message }, window.location.origin);
const params = new URLSearchParams(window.location.search);
const game = params.get('game');
const build = params.get('build');

window.download_log_file_js = (filename, content) => {
	const url = URL.createObjectURL(new Blob([content], { type: 'text/plain;charset=utf-8' }));
	const link = document.createElement('a');
	link.href = url;
	link.download = filename;
	link.click();
	setTimeout(() => URL.revokeObjectURL(url), 1000);
};

async function launch() {
	if (!['zombies', 'throne'].includes(game) || !build || !/^[a-zA-Z0-9._-]+$/.test(build)) throw new Error('Version ou jeu invalide.');
	canvas.tabIndex = 0;
	canvas.setAttribute('data-number-player', '1');
	canvas.setAttribute('data-telemetry', 'false');
	const module = await import(`/builds/${build}/${game}/wasm.js`);
	await module.default();
	canvas.focus();
	canvas.addEventListener('pointerdown', () => canvas.focus());
	send('ready');
}
window.addEventListener('error', (event) => send('error', `Le jeu a rencontré une erreur : ${event.message}`));
window.addEventListener('unhandledrejection', (event) => send('error', `Le jeu a rencontré une erreur : ${String(event.reason)}`));
launch().catch((error) => {
	send('error', `Impossible de lancer le jeu : ${error.message}`);
	console.error(error);
});
