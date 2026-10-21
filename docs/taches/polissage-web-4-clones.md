# polissage-web-4-clones — gungeon et isaac dans le catalogue web (S1-2)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1 axe 1), `docs/contrat-lancement-web.md`,
`scripts/build-web-games.mjs` (l. 6-8 : refuse tout jeu autre que zombies et throne), `scripts/embed-game-content.rs`,
`Makefile:379-382`, `.github/workflows/release.yaml`, `website/src/lib/game/catalog.ts`, `website/static/releases.json`,
`website/tests/wasm-assets.test.mjs`, `docs/conventions.md` (registre embarqué). Branche `polissage-web-4-clones` depuis
`origin/main`. Estimation : 3 j. Machine : Mac avec Chrome ; la compilation wasm demande `wasm-bindgen-cli` à la version du
`Cargo.lock` et `wasm-opt`.

## Contexte (inventaire §13)

Seuls zombies et throne sont construits et catalogués. `games/gungeon` et `games/isaac` ont les mêmes dépendances et le même
`build = embed-game-content.rs`, mais **n'ont jamais été compilés en wasm** [INFERENCE : aucune trace]. Limites : fichiers de
25 MiB maximum pour Cloudflare Pages (avertissement du script) ; gungeon (198 RON) et isaac (349 RON) sont les plus gros.
Aucun FPS wasm n'est mesuré (S1-5). `ALACOD_SKIP_HUB` est une variable d'environnement : pas d'équivalent web connu.

## À faire

1. Autoriser les quatre jeux dans `build-web-games.mjs` et `make build_wasm_apps` ; compiler gungeon et isaac en wasm ;
   corriger ce qui casse (`std::fs`, `getrandom`, chemins d'assets) **sans toucher à la simulation**.
2. Lint et parité des registres natifs/embarqués pour les quatre (test existant à étendre).
3. `catalog.ts` : deux entrées (nom, genre, description, `features`, image, `milestone`) ; `releases.json` : quatre jeux, `solo`
   vrai, **`online` selon S1-3** ; `loader.html`/`loader.js` : paramètres propres à chaque clone s'il y en a.
4. Gungeon : le hub sur le web (décision 8) ; si une entrée directe est nécessaire, un **paramètre d'URL** transmis par le
   loader, pas une variable d'environnement.
5. `release.yaml` : construire les quatre jeux ; vérifier la limite de 25 MiB par fichier (R2 ou `wasm-opt` sinon) et le
   temps de chargement (taille des assets de chaque jeu).
6. Chaque clone : charger sous Chrome depuis le site, atteindre la première salle, mourir, relancer ; erreurs de console notées.

## Acceptation

- `make build_wasm_apps` produit quatre `build.json` ; `/play?id=gungeon` et `?id=isaac` chargent sans 404 ni panic.
- Tailles et temps de chargement par clone dans le rapport (mesurés ; sinon « non mesuré »).
- Aucune trace ne bouge (simulation intacte) ; `npm` typecheck/tests du site verts.
- Rapport `docs/taches/rapports/polissage-web-4-clones.md`.
