// settingsStore.js
import { writable } from 'svelte/store';

export interface Settings {
	allumetteServerUrl: string;
	matchboxServer: string;
	telemetryEnabled: boolean;
	telemetryUrl: string;
	telemetryAuth: string;
}

// Default settings
const defaultSettings: Settings = {
	allumetteServerUrl: 'https://allumette.estran.studio',
	matchboxServer: 'wss://allumette.estran.studio',
	telemetryEnabled: false,
	telemetryUrl: 'http://localhost:5080/api/default/default/_json',
	telemetryAuth: ''
};

// Create a writable store with default values
const createSettingsStore = () => {
	// Initialize with defaults
	const { subscribe, set, update } = writable(defaultSettings);

	return {
		subscribe,
		set,
		update,
		// Load settings from localStorage
		load: () => {
			try {
				const storedSettings = localStorage.getItem('matchboxSettings');
				if (storedSettings) {
					// Merge with defaults in case new settings were added
					set({ ...defaultSettings, ...JSON.parse(storedSettings) });
				}
			} catch (error) {
				console.error('Failed to load settings:', error);
			}
		},
		// Save current settings to localStorage
		save: (settings: Settings) => {
			try {
				localStorage.setItem('matchboxSettings', JSON.stringify(settings));
				return true;
			} catch (error) {
				console.error('Failed to save settings:', error);
				return false;
			}
		},
		// Reset to defaults
		reset: () => {
			set(defaultSettings);
			localStorage.setItem('matchboxSettings', JSON.stringify(defaultSettings));
		}
	};
};

// Create and export the settings store
export const settingsStore = createSettingsStore();

// Initialize settings on app start
if (typeof window !== 'undefined') {
	settingsStore.load();
}
