<script lang="ts">
	import { onMount } from 'svelte';
	import { settingsStore } from './settingsStore.js';
	import type { Settings } from './settingsStore.js';
	import { toaster } from '$lib/toaster.js';

	let settings: Settings = { allumetteServerUrl: '', matchboxServer: '', telemetryEnabled: false, telemetryUrl: '', telemetryAuth: '' };

	const unsubscribe = settingsStore.subscribe((value) => {
		settings = { ...value };
	});

	onMount(() => {
		return () => unsubscribe();
	});

	function saveSettings() {
		if (!settings) return;
		const success = settingsStore.save(settings);

		if (success) {
			settingsStore.set(settings);
			toaster.create({
				type: 'info',
				title: 'Settings saved',
				duration: 5000
			});
		} else {
			toaster.create({
				type: 'error',
				title: 'Error saving',
				duration: 5000
			});
		}
	}

	function resetSettings() {
		settingsStore.reset();
		toaster.create({
			type: 'info',
			title: 'Settings reset',
			duration: 5000
		});
	}
</script>

<div class="card p-4 w-full max-w-md mx-auto">
	<header class="card-header">
		<h2 class="h2 mb-4">Application Settings</h2>
	</header>

	<section class="p-4">
		<form on:submit|preventDefault={saveSettings}>
			<div class="form-group">
				<label class="label" for="allumetteServerUrl">
					<span>Allumette Server URL</span>
				</label>
				<input class="input" type="text" id="allumetteServerUrl" bind:value={settings.allumetteServerUrl} placeholder="Enter server URL" />
				<p class="text-sm text-slate-500">The URL for the Allumette lobby server</p>
			</div>

			<div class="form-group mt-4">
				<label class="label" for="matchboxServer">
					<span>Matchbox Server URL</span>
				</label>
				<input class="input" type="text" id="matchboxServer" bind:value={settings.matchboxServer} placeholder="Enter matchbox server URL" />
				<p class="text-sm text-slate-500">The WebSocket URL for the P2P matchbox connection</p>
			</div>

			<hr class="my-6 border-surface-200-700-token" />

			<h3 class="h3 mb-4">Crash Reports & Telemetry</h3>

			<div class="space-y-4">
				<div class="flex items-center justify-between">
					<label class="label cursor-pointer flex items-center gap-2" for="telemetryEnabled">
						<span>Enable Telemetry</span>
						<input class="checkbox" type="checkbox" id="telemetryEnabled" bind:checked={settings.telemetryEnabled} />
					</label>
				</div>

				{#if settings.telemetryEnabled}
					<div class="form-group">
						<label class="label" for="telemetryUrl">
							<span>Telemetry URL</span>
						</label>
						<input
							class="input"
							type="text"
							id="telemetryUrl"
							bind:value={settings.telemetryUrl}
							placeholder="http://localhost:5080/api/..."
						/>
						<p class="text-sm text-slate-500">The endpoint URL for OpenObserve</p>
					</div>

					<div class="form-group">
						<label class="label" for="telemetryAuth">
							<span>Telemetry Token / Auth</span>
						</label>
						<input class="input" type="password" id="telemetryAuth" bind:value={settings.telemetryAuth} placeholder="Basic ..." />
						<p class="text-sm text-slate-500">Authorization header value (e.g. Basic ...)</p>
					</div>
				{/if}
			</div>

			<div class="grid grid-cols-2 gap-4 mt-8">
				<button type="button" class="btn preset-outlined-tertiary-500 w-full" on:click={resetSettings}> Reset to Defaults </button>
				<button type="submit" class="btn preset-tonal-primary w-full"> Save Settings </button>
			</div>
		</form>
	</section>
</div>
