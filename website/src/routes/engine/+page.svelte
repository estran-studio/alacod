<svelte:head><title>Le moteur — Alacod</title></svelte:head>
<section class="section-wrap page-intro">
	<span class="eyebrow">LE MOTEUR</span>
	<h1>Construire une fois.<br /><span>Composer autrement.</span></h1>
	<p>
		Alacod est un moteur 2D en Rust et Bevy pour les jeux de survie et les roguelikes. Les comportements sont codés dans le moteur; le
		contenu les assemble en fichiers RON.
	</p>
</section>
<section class="section-wrap prose">
	<h2>Ce qui existe aujourd’hui</h2>
	<div class="three-columns">
		<article>
			<span class="index">01 / COMBAT</span>
			<h3>Des armes aux comportements</h3>
			<p>Projectiles composables, statuts, effets, ennemis et navigation. Zombies et Throne partagent ces systèmes.</p>
		</article>
		<article>
			<span class="index">02 / MONDES</span>
			<h3>Des cartes aux parties</h3>
			<p>Cartes LDtk, cavernes procédurales, surfaces, vagues, étages et progression. Chaque clone choisit ses propres règles.</p>
		</article>
		<article>
			<span class="index">03 / PREUVES</span>
			<h3>Un gameplay vérifiable</h3>
			<p>Scénarios scriptés, traces de simulation, tests de rollback, bots et benchmarks accompagnent les changements.</p>
		</article>
	</div>
	<h2>Jouer ensemble</h2>
	<p>
		GGRS synchronise les parties par rollback. Allumette est une stack indépendante : elle gère les identités, salons et invitations,
		négocie les connexions WebRTC et fournit l’accès au relais TURN lorsque le P2P direct ne passe pas.
	</p>
	<p>
		Les performances et la compatibilité du navigateur sont vérifiées séparément des résultats natifs. Les modes disponibles apparaissent
		sur chaque jeu après leur validation.
	</p>
	<h2>Une direction : 1837</h2>
	<p>
		Les prototypes enrichissent les systèmes réutilisables pour, à terme, construire 1837, un roguelike de survie situé dans un Bas-Canada
		apocalyptique. Ce projet reste une direction de développement.
	</p>
	<a class="button" href="/roadmap">La suite du moteur ↗</a>
	<p class="meta">État éditorial : 9 octobre 2026 · Les versions jouables évoluent indépendamment.</p>
</section>
