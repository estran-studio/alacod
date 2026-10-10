<script lang="ts">
	// Responsive Layout
	import { Navigation, AppBar } from '@skeletonlabs/skeleton-svelte';
	import { House, Settings, Gamepad2, BookOpen, ListChecks, Info } from '@lucide/svelte/icons';
	import { Toaster } from '@skeletonlabs/skeleton-svelte';
	import { toaster } from '$lib/toaster';
	import { fly, fade } from 'svelte/transition';
	import '../app.css';
	import CurrentVersion from '$lib/game/CurrentVersion.svelte';

	let { children } = $props();

	let open = $state(false);

</script>

{#if open}
	<!-- Backdrop -->
	<div
		class="fixed inset-0 bg-black/50 z-[999] md:hidden"
		transition:fade={{ duration: 200 }}
		onclick={() => open = false}
		aria-hidden="true"
	></div>

	<!-- Drawer Content -->
	<div
		class="fixed top-0 left-0 bottom-0 w-auto bg-surface-100-800-token z-[1000] md:hidden"
		transition:fly={{ x: -200, duration: 200 }}
	>
		<div class="h-full overflow-y-auto">
			<Navigation.Rail>
				{#snippet header()}
					<Navigation.Tile label="" href="/" onclick={() => open = false}><img class="logo" src="/icons/android-chrome-192x192.png" alt="logo" /></Navigation.Tile>
					<CurrentVersion></CurrentVersion>
				{/snippet}
				{#snippet tiles()}
					<Navigation.Tile label="Accueil" href="/" onclick={() => open = false}><House /></Navigation.Tile>
					<Navigation.Tile label="En ligne" href="/online" onclick={() => open = false}><Gamepad2 /></Navigation.Tile>

					<Navigation.Tile label="Jeux" href="/games" onclick={() => open = false}><Gamepad2 /></Navigation.Tile>
					<Navigation.Tile label="Moteur" href="/engine" onclick={() => open = false}><Info /></Navigation.Tile>
					<Navigation.Tile label="La suite" href="/roadmap" onclick={() => open = false}><ListChecks /></Navigation.Tile>
					<Navigation.Tile label="Guide bêta" href="/beta" onclick={() => open = false}><BookOpen /></Navigation.Tile>
				{/snippet}
				{#snippet footer()}
					<div class="flex flex-col gap-2 items-center p-2">
						<Navigation.Tile label="Réglages" href="/settings" onclick={() => open = false}><Settings /></Navigation.Tile>
					</div>
				{/snippet}
			</Navigation.Rail>
		</div>
	</div>
{/if}

<div class="h-screen w-full flex flex-col md:flex-row overflow-hidden">
	<!-- Mobile Header -->
	<div class="md:hidden w-full">
		<AppBar>
			{#snippet lead()}
				<button aria-label="Ouvrir le menu" class="btn p-0" onclick={() => (open = !open)}>
					<img src="/icons/android-chrome-192x192.png" alt="menu" class="w-12 h-12 rounded" />
				</button>
			{/snippet}
			{#snippet headline()}
				<h1 class="font-metal-mania text-3xl text-center">Alacod</h1>
			{/snippet}
			{#snippet trail()}
				<a class="btn preset-tonal-primary" href="/games">Jouer</a>
			{/snippet}
		</AppBar>
	</div>

	<!-- Desktop Navigation -->
	<div class="hidden md:flex h-full z-50 bg-surface-100-800-token border-t border-surface-300-600-token">
		<Navigation.Rail>
			{#snippet header()}
				<Navigation.Tile label="" href="/"><img class="logo" src="/icons/android-chrome-192x192.png" alt="logo" /></Navigation.Tile>
				<CurrentVersion></CurrentVersion>
			{/snippet}
			{#snippet tiles()}
				<Navigation.Tile label="Accueil" href="/"><House /></Navigation.Tile>
				<Navigation.Tile label="En ligne" href="/online"><Gamepad2 /></Navigation.Tile>
				<Navigation.Tile label="Jeux" href="/games"><Gamepad2 /></Navigation.Tile>
					<Navigation.Tile label="Moteur" href="/engine"><Info /></Navigation.Tile>
					<Navigation.Tile label="La suite" href="/roadmap"><ListChecks /></Navigation.Tile>
					<Navigation.Tile label="Guide bêta" href="/beta"><BookOpen /></Navigation.Tile>
			{/snippet}

			{#snippet footer()}
				<div class="flex flex-col gap-2 items-center p-2">

					<Navigation.Tile label="Réglages" href="/settings"><Settings /></Navigation.Tile>
				</div>
			{/snippet}
		</Navigation.Rail>
	</div>

	<!-- Main Content -->
	<main id="main" class="flex-1 min-w-0 overflow-auto w-full">
		{@render children()}
	</main>
</div>

<Toaster {toaster}></Toaster>

<style>
	.logo {
		border-radius: 10px;
	}
</style>
