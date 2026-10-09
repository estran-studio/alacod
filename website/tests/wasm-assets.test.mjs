import test from 'node:test';
import assert from 'node:assert/strict';
import { onRequest } from '../functions/builds/[[path]].js';
const url = 'https://alacod.pages.dev/builds/test-1/zombies/wasm_bg.wasm';
const bytes = Uint8Array.from([0, 97, 115, 109, 1, 0, 0, 0]);
function bucket() {
	const calls = [];
	return {
		calls,
		async head(key) {
			calls.push(['head', key]);
			return { size: bytes.length, httpEtag: '"build-etag"' };
		},
		async get(key, options) {
			calls.push(['get', key, options]);
			const range = options?.range;
			const body = range ? bytes.slice(range.offset, range.offset + range.length) : bytes;
			return {
				body: new ReadableStream({
					start(controller) {
						controller.enqueue(body);
						controller.close();
					}
				})
			};
		}
	};
}
function handle(request, GAME_BUILDS) {
	return onRequest({ request, env: { GAME_BUILDS } });
}
test('GET streams executable WASM with immutable MIME/ETag; HEAD only reads metadata', async () => {
	const b = bucket();
	const response = await handle(new Request(url), b);
	assert.equal(response.status, 200);
	assert.equal(response.headers.get('Content-Type'), 'application/wasm');
	assert.match(response.headers.get('Cache-Control'), /immutable/);
	assert.equal(response.headers.get('Content-Length'), '8');
	assert.deepEqual(new Uint8Array(await response.arrayBuffer()), bytes);
	b.calls.length = 0;
	const head = await handle(new Request(url, { method: 'HEAD' }), b);
	assert.equal(head.body, null);
	assert.equal(head.headers.get('ETag'), '"build-etag"');
	assert.deepEqual(
		b.calls.map((c) => c[0]),
		['head']
	);
});
test('conditional GET skips the body, including weak ETag comparisons', async () => {
	const b = bucket();
	const response = await handle(new Request(url, { headers: { 'If-None-Match': 'W/"build-etag"' } }), b);
	assert.equal(response.status, 304);
	assert.equal(response.body, null);
	assert.deepEqual(
		b.calls.map((c) => c[0]),
		['head']
	);
});
test('byte ranges, suffixes, invalid ranges and stale If-Range behave correctly', async () => {
	for (const [range, expected] of [
		['bytes=0-3', [0, 97, 115, 109]],
		['bytes=-2', [0, 0]]
	]) {
		const response = await handle(new Request(url, { headers: { Range: range } }), bucket());
		assert.equal(response.status, 206);
		assert.deepEqual([...new Uint8Array(await response.arrayBuffer())], expected);
	}
	const b = bucket();
	assert.equal((await handle(new Request(url, { headers: { Range: 'bytes=90-99' } }), b)).status, 416);
	assert.deepEqual(
		b.calls.map((c) => c[0]),
		['head']
	);
	assert.equal((await handle(new Request(url, { headers: { Range: 'bytes=0-3', 'If-Range': '"old-etag"' } }), bucket())).status, 200);
});
test('only game WASM paths are public; missing objects and unsupported methods are explicit', async () => {
	const b = bucket();
	assert.equal((await handle(new Request(url.replace('wasm_bg.wasm', 'secret.json')), b)).status, 404);
	assert.deepEqual(b.calls, []);
	assert.equal((await handle(new Request(url, { method: 'POST' }), b)).status, 405);
	assert.equal((await handle(new Request(url), undefined)).status, 503);
	const missing = await handle(new Request(url), { head: async () => null });
	assert.equal(missing.status, 404);
	assert.equal(missing.headers.get('Cache-Control'), 'no-store');
});
