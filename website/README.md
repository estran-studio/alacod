# Site public Alacod

SvelteKit statique pour `alacod.estran.studio`. Catalogue éditorial, engine, roadmap et guide
bêta; jeux WASM versionnés et lancés dans une iframe de même origine. Allumette reste une
stack externe et indépendante. Le solo ne l'importe pas et ne nécessite aucun compte.

## Développer le site

```sh
cd website
npm ci
npm run dev -- --host 127.0.0.1
npm run check
npm run build
```

Sans artefacts, `static/releases.json` est vide : les cartes indiquent « en préparation ».
Le build statique est dans `website/build`. Aucun serveur Svelte ne tourne sur le VPS.

## Construire les clones (depuis la racine du dépôt)

Installer la cible `wasm32-unknown-unknown`, **wasm-bindgen-cli 0.2.129** (même version que
Cargo.lock) et Binaryen/wasm-opt. La version locale vérifiée de wasm-opt est 124.

```sh
WEB_ENABLE_SOLO=1 node scripts/build-web-games.mjs zombies throne
```

Le script lint le contenu natif, compile, exécute wasm-bindgen/wasm-opt, copie les assets par
jeu, écrit les manifestes et met à jour le catalogue après succès. Le buildId généré est unique.
La disponibilité online reste false : elle nécessite la recette Allumette/WebRTC.

Options : WEB_PROFILE=release, WEB_BUILD_ID=<id-unique>, WEB_ENABLE_SOLO=1,
WASM_BINDGEN=<binaire>, WASM_OPT=<binaire>, WEB_OPTIMIZE=0 (diagnostic seulement).
Si le composant Rust WASM ne se télécharge pas mais que rust-src est installé, la compilation
nightly peut utiliser `WEB_BUILD_STD=1` (reconstruction de std, compilations plus longues).

Pour la distribution, compiler en release et vérifier les tailles après optimisation.
Une taille supérieure à 25 MiB par fichier impose un autre stockage que Pages (R2 prévu),
pas simplement une compression HTTP. Le script signale les fichiers concernés.
Le registre embarque seulement les définitions texte et l'inventaire des chemins : images,
cartes et sons utilisés par Bevy restent chargés sous `/builds/<id>/<jeu>/assets`.

`make build_website TARGET=web PROFILE=prod` et la recette Docker utilisent les vrais clones.
WEB_ENABLE_SOLO doit être explicitement activé pour exposer le solo dans le catalogue généré.
Les builds sont immuables : réutiliser un ID existant est une erreur. Les dossiers d'artefacts
sont ignorés par git; ne pas committer des entrées de release pointant vers un build local absent.

## Validation

- `cargo test --locked -p content` : lint et parité exacte des registres natifs/embarqués.
- `npm run check` et `npm run build` : validation du site.
- Dans un navigateur : visiter `/games`, lancer chaque clone solo, vérifier erreurs/assets,
  focus, contrôles, défaite/relance et plein écran. Faire aussi une visite après mise à jour.
- Le service worker ne cache que les bundles du site : aucune API, invitation ou release
  ne reçoit de fallback hors ligne; les jeux se téléchargent au lancement.

Le plan et son journal sont dans `docs/plan-site-public-beta.md` (racine). Le contrat de
lancement et le déploiement VPS préparé sont dans `docs/contrat-lancement-web.md` et
`deploy/allumette`. Aucun DNS ou service public n'est modifié par les builds locaux.
