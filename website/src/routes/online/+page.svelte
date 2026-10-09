<script lang="ts">
	import { onMount } from 'svelte';
	import { games, getReleases, type Releases } from '$lib/game/catalog';
	let releases: Releases = $state({ schemaVersion: 1, games: {} });
	let error = $state('');
	onMount(async () => {
		try {
			releases = await getReleases();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Catalogue indisponible.';
		}
	});
</script>

<svelte:head><title>Jouer avec des amis — Alacod</title></svelte:head>
<section class="section-wrap page-intro">
	<span class="eyebrow">MULTIJOUEUR / ALLUMETTE</span>
	<h1>La prochaine partie,<br /><span>ensemble.</span></h1>
	<p>Allumette réunit les joueurs dans un salon et permet les connexions P2P, avec un relais lorsque la liaison directe ne passe pas.</p>
</section>
<section class="section-wrap">
	<div class="play-panel">
		<h2>Les salons web se préparent</h2>
		<p style="margin:20px 0">
			Nous vérifions les invitations, les versions compatibles et les connexions entre réseaux différents avant d’ouvrir les parties en
			ligne.
		</p>
		{#if error}<p role="alert">{error}</p>{/if}{#each games as game}<p>
				{game.name} · {releases.games[game.id]?.online
					? 'Version réseau en validation — salons bientôt disponibles'
					: 'Multijoueur web en préparation'}
			</p>{/each}<a class="button primary" href="/games" style="margin-top:24px">Découvrir les versions solo ↗</a>
	</div>
</section>
