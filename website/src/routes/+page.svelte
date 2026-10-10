<script lang="ts">
	import DevProgress from '$lib/game/DevProgress.svelte';
	import SvgIcon from '$lib/SVGIcon.svelte';
	import Title from '$lib/Title.svelte';
</script>

<div class="flex flex-col w-full min-h-full">
	<header>
		<Title />
	</header>
	<main class="flex flex-col md:flex-row gap-6 p-4">
		<article class="flex-1 bg-surface-100-800-token rounded-lg p-6">
			<h2 class="font-metal-mania text-2xl font-bold mb-4">Bienvenue !</h2>
            <p class="mb-4">Alacod est un moteur 2D modulaire pour les jeux de survie et les roguelikes. On le développe en construisant des prototypes qui partagent les mêmes systèmes.</p>
            <div class="mb-6">
                <h3 class="font-metal-mania text-xl font-semibold mb-2">Moteur : ALACOD</h3>
                <ul class="list-disc pl-6 space-y-1">
                    <li>Cartes LDtk et génération procédurale de cavernes</li>
                    <li>Armes, projectiles, effets et comportements composables</li>
                    <li>Contenu configurable en RON : personnages, ennemis et progression</li>
                    <li>Simulation déterministe et synchronisation rollback avec GGRS</li>
                    <li>Allumette, stack indépendante pour les salons et les connexions P2P avec relais</li>
                </ul>
                <a class="text-link" href="/engine">En savoir plus sur le moteur →</a>
            </div>
            <div class="mb-6">
                <h3 class="font-metal-mania text-xl font-semibold mb-2">Les jeux actuels</h3>
                <p class="mb-3"><strong>Zombies</strong> met à l’épreuve les vagues, achats et systèmes de survie. <strong>Throne</strong> explore les cavernes, étages et mutations.</p>
                <p class="mb-4">Les versions solo navigateur sont accessibles selon les builds disponibles. Le multijoueur entre amis est en préparation.</p>
                <a class="button primary" href="/games">Découvrir les jeux et jouer</a>
            </div>
            <div class="mb-6">
                <h3 class="font-metal-mania text-xl font-semibold mb-2">Une direction : 1837</h3>
                <p>Ces prototypes enrichissent le moteur pour construire, à terme, 1837 : un roguelike de survie dans un Bas-Canada apocalyptique.</p>
            </div>
            <div>
                <h3 class="font-metal-mania text-xl font-semibold mb-2">Technologies</h3>
                <ul class="list-disc pl-6"><li>Rust / Bevy</li><li>Matchbox, WebRTC et GGRS</li><li>Versions natives et navigateur; autres plateformes à valider</li></ul>
            </div>
		</article>

		<aside class="w-full md:w-1/3 lg:w-2/5 flex flex-col gap-6 items-center md:items-end">
      		<DevProgress />
		</aside>
	</main>
	<footer class="mt-auto p-8">
		<div class="rounded-container grid w-full grid-cols-1 gap-4 overflow-hidden md:grid-cols-3">
			<a
				target="_blank"
				class="card flex justify-center preset-filled-surface-100-900 p-4 py-8 text-center hover:scale-105 transition-transform"
				href="https://www.youtube.com/@BasCanada"
			>
				<SvgIcon color="var(--color-primary-500)" id="youtube" height="32px" />
			</a>
			<a
				target="_blank"
				class="card flex justify-center preset-filled-surface-100-900 p-4 py-8 text-center hover:scale-105 transition-transform"
				href="https://github.com/bascanada/alacod"
			>
				<SvgIcon color="var(--color-primary-500)" id="github" height="32px" />
			</a>
			<a
				target="_blank"
				class="card flex justify-center preset-filled-surface-100-900 p-4 py-8 text-center hover:scale-105 transition-transform"
				href="mailto:bascanada1841@gmail.com"
			>
				<SvgIcon color="var(--color-primary-500)" id="gmail" height="32px" />
			</a>
		</div>
	</footer>
</div>

