import { spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync, mkdirSync, cpSync, statSync, existsSync, readdirSync, renameSync, rmSync } from 'node:fs';
import { resolve, join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const games = process.argv.slice(2);
if (!games.length) games.push('zombies', 'throne');
if (games.some(game => !['zombies', 'throne'].includes(game))) throw new Error('Only zombies and throne can be built.');
const commit = run('git', ['rev-parse', '--short=12', 'HEAD'], {}, true).trim();
const buildId = process.env.WEB_BUILD_ID || `${commit}-local-${Date.now()}`;
if (!/^[a-zA-Z0-9][a-zA-Z0-9._-]*$/.test(buildId)) throw new Error('Invalid WEB_BUILD_ID');
const profile = process.env.WEB_PROFILE || 'dev';
if (!['dev', 'release'].includes(profile)) throw new Error('WEB_PROFILE must be dev or release.');
const targetDir = resolve(root, process.env.CARGO_TARGET_DIR || 'target');
const bindgen = process.env.WASM_BINDGEN || 'wasm-bindgen';
const bindgenVersion = run(bindgen, ['--version'], {}, true).trim();
const expectedBindgen = readFileSync(join(root, 'Cargo.lock'), 'utf8').match(/name = "wasm-bindgen"\nversion = "([^"]+)"/)[1];
if (bindgenVersion !== `wasm-bindgen ${expectedBindgen}`) throw new Error(`Install wasm-bindgen-cli ${expectedBindgen} (found ${bindgenVersion}), or set WASM_BINDGEN to its path.`);
const env = { ...process.env, APP_VERSION: buildId, CARGO_TARGET_DIR: targetDir, RUSTFLAGS: '--cfg=web_sys_unstable_apis' };
function run(command, args, overrides = {}, capture = false) {
  const result = spawnSync(command, args, { cwd: root, env: { ...process.env, ...overrides }, stdio: capture ? 'pipe' : 'inherit', encoding: 'utf8' });
  if (result.error || result.status !== 0) throw result.error || new Error(`${command} failed (${result.status})`);
  return result.stdout;
}
function files(dir) { return readdirSync(dir, { withFileTypes: true }).flatMap(entry => entry.isDirectory() ? files(join(dir, entry.name)) : [join(dir, entry.name)]); }
const releasesPath = join(root, 'website/static/releases.json');
const releases = JSON.parse(readFileSync(releasesPath, 'utf8'));
for (const game of games) {
  const output = join(root, 'website/static/builds', buildId, game);
  if (existsSync(output)) throw new Error(`Immutable build already exists: ${output}`);
  run('cargo', ['run', '--locked', '-p', 'content', '--bin', 'alacod', '--', 'lint', `games/${game}`]);
  run('cargo', ['build', ...(process.env.WEB_BUILD_STD === '1' ? ['-Z', 'build-std=std,panic_abort'] : []), '--locked', '-p', game, '--target', 'wasm32-unknown-unknown', '--no-default-features', '--features', 'render', ...(profile === 'release' ? ['--release'] : [])], env);
  const staging = `${output}.partial-${process.pid}`;
  mkdirSync(staging, { recursive: true });
  try {
    run(bindgen, ['--out-dir', staging, '--out-name', 'wasm', '--target', 'web', join(targetDir, 'wasm32-unknown-unknown', profile === 'dev' ? 'debug' : 'release', `${game}.wasm`)]);
    if (process.env.WEB_OPTIMIZE !== '0') {
      run(process.env.WASM_OPT || 'wasm-opt', [join(staging, 'wasm_bg.wasm'), '--strip-debug', '--strip-dwarf', '-O2', '-o', join(staging, 'wasm_bg.optimized.wasm')]);
      renameSync(join(staging, 'wasm_bg.optimized.wasm'), join(staging, 'wasm_bg.wasm'));
    }
    cpSync(join(root, 'games', game, 'assets'), join(staging, 'assets'), { recursive: true });
    const base = `/builds/${buildId}/${game}`;
    const manifest = { schemaVersion: 1, gameId: game, buildId, engineCommit: commit, module: `${base}/wasm.js`, wasm: `${base}/wasm_bg.wasm`, assets: `${base}/assets`, generatedAt: new Date().toISOString() };
    writeFileSync(join(staging, 'build.json'), JSON.stringify(manifest, null, 2) + '\n');
    const oversized = files(staging).filter(file => statSync(file).size > 25 * 1024 * 1024);
    if (oversized.length) console.warn(`Pages 25 MiB limit exceeded; use R2 or an optimized build: ${oversized.join(', ')}`);
    renameSync(staging, output);
  } finally {
    if (existsSync(staging)) rmSync(staging, { recursive: true });
  }
  const base = `/builds/${buildId}/${game}`;
  // Keep new builds hidden from solo/online play unless the caller has validated and
  // explicitly opted this release into the beta catalog.
  releases.games[game] = {
    buildId,
    manifest: `${base}/build.json`,
    solo: process.env.WEB_ENABLE_SOLO === '1',
    online: process.env.WEB_ENABLE_ONLINE === '1',
  };
}
writeFileSync(`${releasesPath}.tmp`, JSON.stringify(releases, null, 2) + '\n');
renameSync(`${releasesPath}.tmp`, releasesPath);
console.log(`Built ${games.join(', ')} as ${buildId}. Set WEB_ENABLE_SOLO=1 for a playable local/preview catalog.`);
