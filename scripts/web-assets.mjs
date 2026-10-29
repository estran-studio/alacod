// Copie des assets d'un jeu vers un build web (D161, docs/conventions.md § 99).
// La surcharge privée (scripts/assets-prives.patterns : images payantes, extraits Sonniss, `*.local.ron`)
// n'entre dans le build que si ALACOD_WEB_PRIVATE_ASSETS=1 ; sans le drapeau, un poste qui a installé la
// surcharge publie exactement le repli public.
// Usage en ligne de commande : node scripts/web-assets.mjs <jeu> <destination>
import { cpSync, readFileSync, readdirSync, mkdirSync } from 'node:fs';
import { resolve, join, dirname, relative, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');

export function privateAssetsEnabled(env = process.env) {
  return env.ALACOD_WEB_PRIVATE_ASSETS === '1';
}

export function privatePatterns() {
  const rules = readFileSync(join(root, 'scripts/assets-prives.patterns'), 'utf8')
    .split('\n').map(line => line.trim()).filter(line => line && !line.startsWith('#'));
  return {
    dirs: rules.filter(rule => rule.endsWith('/')),
    suffixes: rules.filter(rule => rule.startsWith('*')).map(rule => rule.slice(1)),
  };
}

// `rel` : chemin relatif à `games/<jeu>/assets/`, séparateurs `/`.
export function isPrivateAsset(rel, patterns = privatePatterns()) {
  return patterns.dirs.some(dir => rel === dir.slice(0, -1) || rel.startsWith(dir)) || patterns.suffixes.some(suffix => rel.endsWith(suffix));
}

function files(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap(entry => entry.isDirectory() ? files(join(dir, entry.name)) : [join(dir, entry.name)]);
}

// Copie `games/<game>/assets/` vers `destination` ; retourne { privateFound, privateIncluded } (nombres de fichiers).
export function copyGameAssets(game, destination, includePrivate = privateAssetsEnabled()) {
  const source = join(root, 'games', game, 'assets');
  const patterns = privatePatterns();
  const privateSeen = files(source).filter(file => isPrivateAsset(relative(source, file).split(sep).join('/'), patterns)).length;
  mkdirSync(destination, { recursive: true });
  cpSync(source, destination, {
    recursive: true,
    filter: src => includePrivate || !isPrivateAsset(relative(source, src).split(sep).join('/'), patterns),
  });
  return { privateFound: privateSeen, privateIncluded: includePrivate ? privateSeen : 0 };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [game, destination] = process.argv.slice(2);
  if (!['zombies', 'throne'].includes(game) || !destination) throw new Error('Usage : node scripts/web-assets.mjs <zombies|throne> <destination>');
  const result = copyGameAssets(game, resolve(destination));
  console.log(`Assets de ${game} copiés vers ${destination} : surcharge privée ${privateAssetsEnabled() ? `incluse (${result.privateIncluded} fichiers)` : `exclue (${result.privateFound} fichiers présents sur le poste, non copiés)`}.`);
}
