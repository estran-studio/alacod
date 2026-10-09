<script lang="ts">
	import { onMount } from 'svelte';
	import { games, getReleases, type Releases } from '$lib/game/catalog';
	let releases: Releases = $state({ schemaVersion: 1, games: {} });
	let error = $state('');
	let loaded = $state(false);
	onMount(async () => {
		try {
			releases = await getReleases();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Catalogue indisponible.';
		} finally {
			loaded = true;
		}
	});
</script>

<svelte:head><title>Les jeux — Alacod</title></svelte:head>
<section class="section-wrap page-intro">
	<span class="eyebrow">LES JEUX / PROTOTYPES</span>
	<h1>Choisissez votre<br /><span>terrain de jeu.</span></h1>
	<p>
		Du contenu différent, un moteur commun. Le solo est accessible sans compte. Le multijoueur s’ouvre après validation des connexions entre
		amis.
	</p>
</section>
<section class="section-wrap">
	<p class="notice">Bêta en préparation · Ordinateur, clavier et souris · Disponibilité selon les versions publiées</p>
	{#if error}<p role="alert" class="notice">{error} <a href="/games">Réessayer</a></p>{/if}
	<div class="game-grid">
		{#each games as game}<article class="game-preview {game.accent}">
				<div class="preview-art">
					{#if game.image}<img src={game.image} alt="Capture du prototype Zombies" />{:else}<div class="cave-art" aria-hidden="true">
							<span>THRONE</span><i></i><i></i><i></i>
						</div>{/if}<span class="game-tag">{game.milestone} / EN DÉVELOPPEMENT</span>
				</div>
				<div class="game-preview-body">
					<span class="eyebrow">{game.genre}</span>
					<h2>{game.name}</h2>
					<p>{game.description}</p>
					<ul class="feature-list">
						{#each game.features as feature}<li>{feature}</li>{/each}
					</ul>
					<div class="game-actions">
						{#if releases.games[game.id]?.solo}<a class="button primary" href="/play?id={game.id}">Jouer solo ↗</a>{:else}<span
								class="availability">{loaded ? 'Version web en préparation' : 'Vérification des versions…'}</span
							>{/if}{#if releases.games[game.id]?.online}<a class="button" href="/online?game={game.id}">Avec des amis ↗</a>{/if}
					</div>
					{#if releases.games[game.id]}<small>Version {releases.games[game.id]?.buildId}</small>{/if}
				</div>
			</article>{/each}
	</div>
</section>
<section class="section-wrap next-game">
	<span class="eyebrow">À L’HORIZON / M2</span>
	<h2>Gungeon</h2>
	<p>
		Salles, objets, synergies et boss : le prochain prototype étendra le vocabulaire du moteur. Il n’est pas encore proposé au catalogue.
	</p>
	<a class="text-link" href="/roadmap">Voir la feuille de route ↗</a>
</section>
