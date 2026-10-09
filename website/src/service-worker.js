/// <reference types="@sveltejs/kit" />
import { build, version } from '$service-worker';

const CACHE = `alacod-shell-${version}`;
// Only cache the site's own hashed bundles. Games are fetched on demand; APIs,
// manifests, release catalogs and invitations never receive an offline fallback.
const SHELL = new Set(build);
self.addEventListener('install', (event) => {
	event.waitUntil(caches.open(CACHE).then((cache) => cache.addAll([...SHELL])));
});
self.addEventListener('activate', (event) => {
	event.waitUntil(
		(async () => {
			for (const key of await caches.keys()) {
				if ((key.startsWith('cache-') || key.startsWith('alacod-shell-')) && key !== CACHE) await caches.delete(key);
			}
		})()
	);
});
self.addEventListener('fetch', (event) => {
	const url = new URL(event.request.url);
	if (event.request.method !== 'GET' || url.origin !== self.location.origin || !SHELL.has(url.pathname)) return;
	event.respondWith(
		(async () => {
			const cache = await caches.open(CACHE);
			return (await cache.match(event.request)) || fetch(event.request);
		})()
	);
});
