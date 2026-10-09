const cacheControl = 'public, max-age=31536000, immutable';
const missing = () => new Response('Build absent.', { status: 404, headers: { 'Cache-Control': 'no-store' } });

export async function onRequest({ request, env }) {
	if (!['GET', 'HEAD'].includes(request.method)) {
		return new Response(null, { status: 405, headers: { Allow: 'GET, HEAD' } });
	}
	const key = new URL(request.url).pathname.slice(1);
	if (!/^builds\/[a-zA-Z0-9][a-zA-Z0-9._-]*\/(zombies|throne)\/wasm_bg\.wasm$/.test(key)) return missing();
	if (!env.GAME_BUILDS) return new Response('Stockage indisponible.', { status: 503, headers: { 'Cache-Control': 'no-store' } });
	const metadata = await env.GAME_BUILDS.head(key);
	if (!metadata) return missing();
	const headers = new Headers({
		'Content-Type': 'application/wasm',
		'Cache-Control': cacheControl,
		ETag: metadata.httpEtag,
		'Accept-Ranges': 'bytes',
		'X-Content-Type-Options': 'nosniff'
	});
	const noneMatch = request.headers.get('If-None-Match');
	if (noneMatch?.split(',').some((tag) => tag.trim() === '*' || tag.trim().replace(/^W\//, '') === metadata.httpEtag)) {
		return new Response(null, { status: 304, headers });
	}
	if (request.method === 'HEAD') {
		headers.set('Content-Length', String(metadata.size));
		return new Response(null, { headers });
	}
	let range;
	const requested = request.headers.get('Range');
	const ifRange = request.headers.get('If-Range');
	if (requested && (!ifRange || ifRange === metadata.httpEtag)) {
		const match = /^bytes=(\d*)-(\d*)$/.exec(requested);
		if (match && (match[1] || match[2])) {
			const start = match[1] ? Number(match[1]) : Math.max(0, metadata.size - Number(match[2]));
			const end = match[1] && match[2] ? Math.min(Number(match[2]), metadata.size - 1) : metadata.size - 1;
			if (
				Number.isSafeInteger(start) &&
				Number.isSafeInteger(end) &&
				start <= end &&
				start < metadata.size &&
				(match[1] || Number(match[2]) > 0)
			) {
				range = { offset: start, length: end - start + 1 };
			}
		}
		if (!range) return new Response(null, { status: 416, headers: { 'Content-Range': `bytes */${metadata.size}` } });
	}
	const object = await env.GAME_BUILDS.get(key, range ? { range } : undefined);
	if (!object) return missing();
	headers.set('Content-Length', String(range?.length ?? metadata.size));
	if (range) headers.set('Content-Range', `bytes ${range.offset}-${range.offset + range.length - 1}/${metadata.size}`);
	// Stream from R2; never buffer the complete WASM in the function's memory.
	return new Response(object.body, { status: range ? 206 : 200, headers });
}
