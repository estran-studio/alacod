// Catalog games and immutable builds only. Online credentials arrive from the
// same-origin host in memory; solo never accesses Allumette.
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

window.alacod_return_to_lobby = () => send('return-lobby');
let onlineSession;
async function sessionFromHost() {
	return new Promise((resolve, reject) => {
		const timeout = setTimeout(() => {
			window.removeEventListener('message', receive);
			reject(new Error('Session non reçue.'));
		}, 10000);
		function receive(event) {
			if (
				event.origin !== location.origin ||
				event.source !== window.parent ||
				event.data?.source !== 'alacod-host' ||
				event.data.type !== 'session'
			)
				return;
			clearTimeout(timeout);
			window.removeEventListener('message', receive);
			resolve(event.data.session);
		}
		window.addEventListener('message', receive);
		send('session-request');
	});
}
window.addEventListener('message', (event) => {
	if (
		onlineSession &&
		event.origin === location.origin &&
		event.source === window.parent &&
		event.data?.source === 'alacod-host' &&
		event.data.type === 'participants'
	)
		canvas.setAttribute('data-participants', JSON.stringify(event.data.participants));
});
async function launch() {
	if (!['zombies', 'throne'].includes(game) || !build || !/^[a-zA-Z0-9._-]+$/.test(build)) throw new Error('Version ou jeu invalide.');
	canvas.tabIndex = 0;
	if (params.get('mode') === 'online') {
		onlineSession = await sessionFromHost();
		if (
			onlineSession.schema_version !== 1 ||
			onlineSession.game_id !== game ||
			onlineSession.compatibility_id !== `${game}:${build}` ||
			![2, 4].includes(onlineSession.capacity) ||
			onlineSession.expires_at * 1000 <= Date.now()
		)
			throw new Error('Contrat de session invalide.');
		const signaling = new URL(onlineSession.signaling_url);
		if (!['ws:', 'wss:'].includes(signaling.protocol)) throw new Error('Signalisation invalide.');
		if (!Array.isArray(onlineSession.ice_servers) || !onlineSession.ice_servers.length) throw new Error('Serveurs ICE absents.');
		// Isolated iframe adapter: Matchbox accepts one ICE config, but the browser
		// receives the complete server-issued STUN/TURN list, including credentials.
		const NativePeerConnection = window.RTCPeerConnection;
		window.RTCPeerConnection = class extends NativePeerConnection {
			constructor(config) {
				super({
					...config,
					iceServers: onlineSession.ice_servers,
					...(onlineSession.ice_transport_policy === 'relay' ? { iceTransportPolicy: 'relay' } : {})
				});
				this.addEventListener('connectionstatechange', () => {
					if (this.connectionState === 'connected') send('connected');
					if (this.connectionState === 'failed') send('error', 'La connexion WebRTC a échoué. Réessaie depuis le salon.');
				});
			}
		};
		canvas.setAttribute('data-session', JSON.stringify({ signaling_url: onlineSession.signaling_url }));
		canvas.setAttribute('data-participants', JSON.stringify(onlineSession.participants));
		canvas.setAttribute('data-number-player', String(onlineSession.capacity));
		canvas.setAttribute('data-matchbox', signaling.origin);
		canvas.setAttribute(
			'data-players',
			JSON.stringify(
				onlineSession.participants.map((p) => ({
					pubkey: p.player_id,
					name: p.username,
					is_local: p.player_id === onlineSession.local_player_id
				}))
			)
		);
	} else {
		canvas.setAttribute('data-number-player', '1');
	}
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
