<script lang="ts">
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import { AllumetteBrowserClient, AllumetteError, type Lobby, type SessionContext } from '@bascanada/allumette-browser';
	import { games, findGame, getReleases, type Releases, type BuildManifest } from '$lib/game/catalog';
	import { settingsStore } from '../settings/settingsStore';
	let releases: Releases = $state({ schemaVersion: 1, games: {} }),
		username = $state(''),
		gameId = $state('zombies'),
		error = $state(''),
		busy = $state(false);
	let user = $state<{ player_id: string; username: string } | null>(null),
		lobby = $state<Lobby | null>(null),
		session = $state<SessionContext | null>(null);
	let invitation = $state<{ id: string; token: string; game: string; build: string } | null>(null);
	let inviteLink = $state(''),
		status = $state(''),
		frame: HTMLIFrameElement | undefined = $state();
	function releaseFor(id: string) {
		const game = findGame(id);
		return game ? releases.games[game.id] : undefined;
	}
	async function validatedRelease(id: string) {
		const r = releaseFor(id);
		if (!r?.online) throw new Error('Multijoueur en préparation pour cette version.');
		const response = await fetch(r.manifest, { cache: 'no-store' });
		if (!response.ok) throw new Error('Build indisponible.');
		const manifest: BuildManifest = await response.json();
		if (
			manifest.schemaVersion !== 1 ||
			manifest.gameId !== id ||
			manifest.buildId !== r.buildId ||
			manifest.module !== `/builds/${r.buildId}/${id}/wasm.js`
		)
			throw new Error('Build incompatible.');
		return r;
	}
	let client: AllumetteBrowserClient,
		controller: AbortController,
		timer: ReturnType<typeof setTimeout>,
		mounted = false,
		refreshing = false,
		opening = false,
		rejectedSession = '';
	const messages: Record<string, string> = {
		incompatible_build: 'Cette invitation utilise une autre version du jeu.',
		lobby_full: 'Le salon est complet.',
		players_not_ready: 'Tous les joueurs doivent être présents et prêts.',
		invitation_invalid: 'Cette invitation a expiré ou a été remplacée.',
		already_in_lobby: 'Tu es déjà dans un salon.',
		session_expired: 'La connexion a expiré. Reviens au salon pour réessayer.'
	};
	function explain(e: unknown) {
		if (e instanceof AllumetteError && e.status === 401) {
			user = null;
			lobby = null;
			session = null;
			return 'Ta session a expiré. Reconnecte-toi.';
		}
		return e instanceof AllumetteError
			? messages[e.code] || `Allumette : ${e.code}`
			: e instanceof Error
				? e.message
				: 'Connexion impossible.';
	}
	async function action(fn: () => Promise<void>) {
		busy = true;
		error = '';
		try {
			await fn();
		} catch (e) {
			error = explain(e);
		} finally {
			busy = false;
		}
	}
	async function refresh() {
		if (!mounted || !user || refreshing) return;
		refreshing = true;
		try {
			const list = await client.listLobbies(controller.signal);
			if (!mounted) return;
			lobby = list[0] || null;
			if (!lobby?.session_id) {
				session = null;
				status = '';
				rejectedSession = '';
			} else if (lobby.session_id !== rejectedSession) {
				if (!session && !opening) {
					opening = true;
					try {
						const c = await client.getSessionContext(lobby.id, controller.signal);
						if (mounted && lobby?.session_id === c.session_id && rejectedSession !== c.session_id) {
							session = c;
							status = 'Chargement du jeu…';
						}
					} finally {
						opening = false;
					}
				} else if (session) {
					const c = await client.getSessionContext(lobby.id, controller.signal);
					if (mounted && c.session_id === session.session_id)
						frame?.contentWindow?.postMessage(
							{ source: 'alacod-host', type: 'participants', participants: c.participants },
							location.origin
						);
				}
			}
		} catch (e) {
			if (!controller.signal.aborted) error = explain(e);
		} finally {
			refreshing = false;
		}
	}
	async function connect() {
		await action(async () => {
			if (!user) user = await client.loginGuest(username);
			if (invitation) {
				gameId = invitation.game;
				const r = await validatedRelease(gameId);
				if (!r?.online || r.buildId !== invitation.build) throw new Error('Cette version multijoueur n’est pas disponible sur ce site.');
				await client.joinLobby(invitation.id, invitation.token, `${gameId}:${r.buildId}`);
				invitation = null;
			}
			await refresh();
		});
	}
	async function invite() {
		if (!lobby) return;
		const i = await client.createInvitation(lobby.id),
			url = new URL('/online', location.origin);
		url.searchParams.set('game', lobby.game_id);
		url.searchParams.set('build', releaseFor(lobby.game_id)!.buildId);
		url.searchParams.set('lobby', lobby.id);
		url.hash = new URLSearchParams({ invite: i.invitation_token }).toString();
		inviteLink = url.href;
	}
	async function create() {
		await action(async () => {
			const r = await validatedRelease(gameId);
			if (!r?.online) throw new Error('Multijoueur en préparation pour cette version.');
			lobby = await client.createLobby(gameId, `${gameId}:${r.buildId}`, 2);
			await invite();
		});
	}
	async function returnToLobby() {
		const id = lobby?.id;
		if (session) rejectedSession = session.session_id;
		session = null;
		status = '';
		if (id)
			await action(async () => {
				await client.endSession(id);
				await refresh();
			});
	}
	onMount(() => {
		mounted = true;
		controller = new AbortController();
		settingsStore.load();
		client = new AllumetteBrowserClient(get(settingsStore).allumetteServerUrl);
		const url = new URL(location.href),
			token = new URLSearchParams(url.hash.slice(1)).get('invite');
		if (token && url.searchParams.get('lobby')) {
			invitation = {
				id: url.searchParams.get('lobby')!,
				token,
				game: url.searchParams.get('game') || '',
				build: url.searchParams.get('build') || ''
			};
			history.replaceState(null, '', url.pathname + url.search);
		}
		getReleases()
			.then((r) => {
				if (mounted) releases = r;
			})
			.catch((e) => {
				if (mounted) error = explain(e);
			});
		const receive = (event: MessageEvent) => {
			if (event.origin !== location.origin || event.source !== frame?.contentWindow || event.data?.source !== 'alacod-loader') return;
			if (event.data.type === 'session-request' && session)
				frame?.contentWindow?.postMessage({ source: 'alacod-host', type: 'session', session: $state.snapshot(session) }, location.origin);
			if (event.data.type === 'ready') status = 'Jeu chargé · Connexion aux autres joueurs…';
			if (event.data.type === 'connected') status = 'Connexion établie · Clique dans le jeu pour jouer.';
			if (event.data.type === 'return-lobby') void returnToLobby();
			if (event.data.type === 'error') {
				const message = event.data.message;
				void returnToLobby().then(() => {
					error = message;
				});
			}
		};
		window.addEventListener('message', receive);
		const poll = async () => {
			await refresh();
			if (mounted) timer = setTimeout(poll, 1000);
		};
		timer = setTimeout(poll, 1000);
		return () => {
			mounted = false;
			clearTimeout(timer);
			controller.abort();
			window.removeEventListener('message', receive);
			client.logout();
		};
	});
</script>

<svelte:head><title>Jouer avec des amis — Alacod</title></svelte:head>
<section class="section-wrap page-intro">
	<span class="eyebrow">MULTIJOUEUR / ALLUMETTE</span>
	<h1>Jouer avec des amis</h1>
	<p>Crée un salon privé, partage une invitation et lance une partie à deux. Chaque joueur doit utiliser la même version du jeu.</p>
</section>
<section class="section-wrap">
	{#if error}<p role="alert" class="notice">{error}</p>{/if}
	{#if !user}
		<div class="play-panel">
			<h2>Ton pseudonyme</h2>
			<p>
				Une identité temporaire suffit. Fermer ou recharger la page nécessite une nouvelle connexion et une nouvelle admission au salon.
			</p>
			<form
				onsubmit={(e) => {
					e.preventDefault();
					void connect();
				}}
			>
				<label for="username">Pseudonyme</label><input
					id="username"
					bind:value={username}
					required
					maxlength="32"
					autocomplete="nickname"
					class="input"
				/><button class="button" disabled={busy || !username.trim()}>Se connecter</button>
			</form>
			<p><a href="/settings">Configurer le serveur Allumette</a></p>
		</div>
	{:else if !lobby}
		<div class="play-panel">
			<h2>Bienvenue, {user.username}</h2>
			<label for="game">Jeu</label><select id="game" bind:value={gameId} class="select"
				>{#each games as game}<option value={game.id}
						>{game.name}{releases.games[game.id]?.online ? '' : ' · multijoueur en préparation'}</option
					>{/each}</select
			><button class="button" disabled={busy || !releaseFor(gameId)?.online} onclick={create}>Créer un salon privé</button
			>{#if invitation}<button class="button" disabled={busy} onclick={connect}>Réessayer l’invitation</button>{/if}
		</div>
	{:else if session}
		<div class="play-controls">
			<span>{games.find((g) => g.id === session?.game_id)?.name} · Partie à {session.capacity}</span><button
				class="button"
				disabled={busy}
				onclick={returnToLobby}>Retour au salon</button
			>
		</div>
		<iframe
			bind:this={frame}
			class="play-frame"
			title="Partie multijoueur"
			src="/loader.html?game={session.game_id}&build={releaseFor(session.game_id)!.buildId}&mode=online"
			allow="fullscreen; gamepad"
		></iframe>
		<p role="status">{status}</p>
		<p>WASD : déplacement · Souris : viser et tirer · R : recharger · Après la défaite, R ramène au salon pour une nouvelle partie.</p>
	{:else}
		<div class="play-panel">
			<h2>Salon {games.find((g) => g.id === lobby?.game_id)?.name}</h2>
			<p>{lobby.players.length} / {lobby.capacity} joueurs · {lobby.status === 'Waiting' ? 'En attente' : 'Connexion en cours'}</p>
			<ul class="feature-list">
				{#each lobby.players as player}<li>
						{player.username}{player.player_id === user.player_id ? ' (toi)' : ''} · {player.ready ? 'Prêt' : 'Pas encore prêt'}
					</li>{/each}
			</ul>
			{#if lobby.is_owner}<button class="button" disabled={busy} onclick={() => action(invite)}>Créer une nouvelle invitation</button
				>{#if inviteLink}<label for="invite-link">Lien à partager (15 minutes)</label><input
						id="invite-link"
						class="input"
						readonly
						value={inviteLink}
					/><button
						class="button"
						onclick={() =>
							action(async () => {
								await navigator.clipboard.writeText(inviteLink);
							})}>Copier le lien</button
					>{/if}{/if}
			<div class="game-actions">
				<button
					class="button"
					disabled={busy || lobby.status !== 'Waiting'}
					onclick={() =>
						action(async () => {
							lobby = await client.setReady(
								lobby!.id,
								!lobby!.players.find((p) => p.player_id === user!.player_id)?.ready,
								lobby!.compatibility_id
							);
						})}>{lobby.players.find((p) => p.player_id === user?.player_id)?.ready ? 'Je ne suis plus prêt' : 'Je suis prêt'}</button
				>
				{#if lobby.is_owner}<button
						class="button"
						disabled={busy || lobby.players.length !== lobby.capacity || lobby.players.some((p) => !p.ready) || lobby.status !== 'Waiting'}
						onclick={() =>
							action(async () => {
								lobby = await client.startSession(lobby!.id);
								await refresh();
							})}>Lancer la partie</button
					>{/if}
				<button
					class="button"
					disabled={busy}
					onclick={() =>
						action(async () => {
							await client.leaveLobby(lobby!.id);
							lobby = null;
							inviteLink = '';
						})}>{lobby.is_owner ? 'Fermer le salon' : 'Quitter le salon'}</button
				>
			</div>
		</div>
	{/if}
</section>
