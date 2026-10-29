// Joueur automatique pour les mesures de performance dans le navigateur (S1-5,
// docs/taches/rapports/polissage-perf-web.md) : une manette standard simulée (`navigator.getGamepads`) qui
// tourne, tire sans s'arrêter, recharge, esquive et appuie sur A de temps en temps (relance après une mort).
// À injecter AVANT le chargement de la page (`browser.open({ init_scripts: [...] })`). Les liaisons sont celles de
// `crates/game/src/character/control.rs::get_input_map` : stick gauche = déplacement (seuil 0,4 par axe), stick
// droit = visée, RT (bouton 7) = tir (seuil 0,5), X (2) = recharge, LB (4) = esquive, A (0) = bombe / valider.
// Déterministe pour une graine donnée (`window.__botSeed`, défaut 1) : le même parcours à chaque mesure.
(() => {
	let seed = (window.__botSeed || 1) >>> 0;
	const rand = () => ((seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0) / 4294967296);
	const buttons = Array.from({ length: 17 }, () => ({ pressed: false, touched: false, value: 0 }));
	const pad = {
		id: 'Alacod perf bot (STANDARD GAMEPAD)',
		index: 0,
		connected: true,
		mapping: 'standard',
		timestamp: 0,
		axes: [0, 0, 0, 0],
		buttons,
		vibrationActuator: null,
		hapticActuators: []
	};
	const set = (i, v) => {
		buttons[i].value = v;
		buttons[i].pressed = v > 0.5;
		buttons[i].touched = v > 0;
	};
	let moveAngle = 0;
	let aimAngle = 0;
	let nextTurn = 0;
	const tick = (now) => {
		if (now >= nextTurn) {
			moveAngle = rand() * Math.PI * 2;
			nextTurn = now + 700 + rand() * 1300;
		}
		aimAngle += 0.045;
		pad.axes[0] = Math.cos(moveAngle);
		pad.axes[1] = Math.sin(moveAngle);
		pad.axes[2] = Math.cos(aimAngle);
		pad.axes[3] = Math.sin(aimAngle);
		// RT : tir par rafales de 2,5 s toutes les 12 s (la réserve de la mitrailleuse, 240 balles, ne dure sinon que
		// 30 s) ; chaque rafale change la valeur du bouton, ce qu'attend le lecteur de manette du navigateur (gilrs ne
		// produit un événement que sur un changement : un bouton déjà tenu à la connexion ne serait jamais vu).
		set(7, now % 12000 < 2500 ? 1 : 0);
		set(2, now % 9000 < 120 ? 1 : 0); // X : recharge toutes les 9 s
		set(4, now % 5000 < 120 ? 1 : 0); // LB : esquive toutes les 5 s
		set(0, now % 3000 < 150 ? 1 : 0); // A : relance après la défaite, valide un choix
		pad.timestamp = now;
		requestAnimationFrame(tick);
	};
	navigator.getGamepads = () => [pad, null, null, null];
	requestAnimationFrame(tick);
	// Chrome n'annonce une manette qu'au premier appui : on l'annonce après le chargement du jeu, et on
	// recommence tant que le jeu n'a pas publié son premier échantillon (`data-perf-sample`, ?perf=1).
	const announce = setInterval(() => {
		const canvas = document.getElementById('bevy-canvas');
		if (canvas?.getAttribute('data-perf-sample')) return clearInterval(announce);
		window.dispatchEvent(new GamepadEvent('gamepadconnected', { gamepad: pad }));
	}, 2000);
})();
