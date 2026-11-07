# M-1 « Web d'un jeu externe » — rapport

Voie m, lot M-1 (reprise sur le Mac de William, 7 nov 2026). **Jalon servi : S1 dim 15, cible mer 11 « 1837 jouable dans Chrome par URL »** (plan § 6, ligne M-1).
Branche engine `wquintal/m-1-web-externe`, tête de code `930124e5` (rapport en commit « docs seules » au-dessus) ; branche 1837 `m-1-web` `104afd3f` (le `rev` y vaut `930124e5…`, à réépingler sur `main`).
Aucune trace de scénario ne bouge (outillage et gabarit seulement, aucun `BLESS`, la simulation n'est pas touchée : pas de `schedule_order_diff`).

## Points du lot

| # | Point | Issue | Preuve |
|---|---|---|---|
| 1 | `alacod build --web [jeu] [--out] [--release] [--features] [--version] [--serve --port]` (jeu hors de `games/`) | **fait** | `crates/content/src/web_build.rs`, `bin/alacod.rs` ; `docs/conventions.md` § 108 |
| 2 | Gabarit `templates/starter` : passe la commande sans retouche | **fait** | `make test_alacod_web` vert (ci-dessous) |
| 3 | Test e2e `#[ignore]` + cible `make` | **fait** | `new_game_builds_for_the_web_without_any_added_code`, `make test_alacod_web` (exit 0, 1261 s avec le target du worktree) |
| 4 | `cargo fmt` | **fait** | commit `4f7c61df` |
| 5 | `make check_rapide WASM=1` | **fait, vert** | `~/orca/runs/alpha/logs/m-1-web-externe-check-rapide-930124e5.log` : dernière ligne « ✓ Contrôle rapide passé (suite de scénarios : la nuit) » (`check_wasm` compris) |
| 6 | 1837 : `./jouer.sh --web` | **fait** | transcript ci-dessous ; journal `~/orca/runs/alpha/logs/m-1-web-externe-1837-jouer-web.log` |
| 7 | `cargo check --target wasm32-unknown-unknown` / compilation wasm | **fait** | le build release de 1837 et celui du jeu temporaire compilent en wasm32 ; `check_wasm` dans le contrôle rapide |
| 8 | Taille du `.wasm` | **mesurée** | 1837 release + `wasm-opt` : **84 339 230 octets (80,4 Mo)** ; jeu temporaire, dev sans `wasm-opt` (debuginfo) : 203,5 Mo |
| 9 | Ouverture dans Chrome (une capture) | **fait** | `1837/docs/preuves/m-1-web-cour.png` : la cour dans Chrome headless (Metal), 50 s après l'ouverture ; journal de page « loading of asset is done, now entering lobby », boîte d'erreur de la page vide |
| 10 | Jeu à la manette dans Chrome | **non vérifié** | l'API Gamepad n'a pas été exercée ; William le joue le soir |

## Transcripts

Jeu temporaire (`alacod new demo` puis build, sans retouche ; dev, cache chaud : 4 min 12) :

```
web : demo webtest2 (dev) → /private/tmp/m1web/site
  wasm : …/site/builds/webtest2/demo/wasm_bg.wasm (203.5 Mo, sans wasm-opt)
  assets : 61 fichiers → …/site/builds/webtest2/demo/assets
  page : /private/tmp/m1web/site/index.html
  servir : alacod build --web --serve (ou tout serveur statique sur /private/tmp/m1web/site)
```

1837 (`PORT=8187 ./jouer.sh --web`, release, 15 min 20 de compilation wasm à froid + 1 min 48 d'`alacod` + 1 min 25 de `wasm-bindgen-cli 0.2.129`) :

```
web : 1837 57c4d6bc4f90-dirty1794020460 (release) → …/1837/m-1-web/target/web
  wasm : …/target/web/builds/57c4d6bc4f90-dirty1794020460/1837/wasm_bg.wasm (80.4 Mo, wasm-opt)
  assets : 137 fichiers → …/target/web/builds/57c4d6bc4f90-dirty1794020460/1837/assets
  page : …/target/web/index.html
http://127.0.0.1:8187/   (Ctrl-C pour arrêter)
```

(`-dirty` : le build date d'avant le commit ; sur un arbre propre la version est le SHA court.) `curl -I` : `200`, `application/wasm`, `Content-Length: 84339230`.

## Protocole du soir pour William (cinq lignes)

1. Après le merge de `m-1-web` par le coordinateur 1837, dans `~/Project/bascanada/1837` : `./jouer.sh --web` (première fois ≈ 20 min : `alacod`, `wasm-bindgen`, wasm ; le message `missing key for inline table element` de cargo est inoffensif, D189).
2. Quand la ligne `http://127.0.0.1:8080/` s'affiche, ouvrir cette URL dans Chrome (onglet muet) et attendre ≈ 50 s : 80 Mo à charger, puis la cour de la loge.
3. Cliquer dans la page (focus) : WASD, souris, clic gauche tir, F hache, R recharge, C esquive ; manette : la brancher et appuyer sur un bouton avant de regarder (non vérifié).
4. Console de Chrome (Cmd+Option+J) : seuls les `WARN bevy_ecs_ldtk … EntityInstance needs a tile` sont attendus ; `?perf=1` dans l'URL affiche le compteur FPS (F8).
5. Ctrl-C dans le terminal arrête le serveur ; `PORT=9000 ./jouer.sh --web` si 8080 est pris.

## Ce que le lot a changé (rappel de la passation, état final)

- `web_build.rs` : lint → cargo wasm32 → `wasm-bindgen` épinglé par `Cargo.lock` (auto-installé dans `<target>/alacod-tools`) → `wasm-opt` (release, s'il existe) → assets → `index.html` + `build.json` ; serveur local. `crates/game/build.rs` relit `APP_VERSION`.
- `templates/starter` : `build.rs` d'embarquement du contenu, `main.rs` à chemin `cfg(wasm32)`, `getrandom` js.
- Correctifs faits sur ce lot au Mac : (a) l'assertion du test cherchait `export default` ; `wasm-bindgen 0.2.129` écrit `export { initSync, __wbg_init as default }`, elle cherche maintenant `as default` (la page appelle `module.default()`, inchangé) ;
  (b) `scripts/claude-say.sh` : le programme python est lu hors de `$( )` ; `/bin/bash` 3.2 de macOS ne parse pas un « ` » dans un here-doc imbriqué, ce qui rendait trois tests de `make test_tools` rouges sur le Mac (premier passage du contrôle
  rapide rouge, journal conservé : `…-check-rapide-4f7c61df-rouge-claude-say.log`). Hors lot, mais bloquait le contrôle rapide.
- Côté 1837 : `build.rs` et `src/main.rs` du gabarit (nom `1837`), `getrandom`, `jouer.sh --web` (installe l'`alacod` du `rev` dans `target/outils/`, puis `alacod build --web --release --serve --port ${PORT:-8080}`), `README.md`, `docs/tranche.md` « État de M-1 ».

## Pour M-2 (à deux en ligne dans Chrome)

- Lobby : `crates/game/src/args/web.rs` lit sur le canvas `data-matchbox`, `data-lobby`, `data-number-player`, `data-players` (JSON), `data-telemetry*`. La page de M-1 ne pose que `data-number-player="1"` et `data-telemetry="false"` :
  **M-2 doit lire `?lobby=` et `?matchbox=` de l'URL et les poser sur le canvas** (même endroit que `?perf=1` dans `index_html`), `data-number-player="2"`.
- D190 : la racine d'assets est l'absolu `/builds/<version>/1837/assets` ; une URL publique sous un sous-chemin la casse. Un `AssetPlugin.file_path` relatif (question ouverte de la passation) n'a pas été essayé : le chemin absolu marche à la racine d'une origine.
- Le serveur local envoie `Cache-Control: no-store` ; un hébergement public voudra des en-têtes de cache sur `wasm_bg.wasm` (le nom de dossier porte déjà la version).
- L'atlas du Frère (5632×6120 px) que la tranche jugeait « trop gros pour WebGL2 » s'affiche dans Chrome (Metal via ANGLE) ; non revérifié sur d'autres GPU.
- 80 Mo de wasm : le premier chargement dure ≈ 50 s en local ; un ami en ligne attendra plus longtemps (compression gzip/brotli à servir en M-2).
- `wasm-bindgen` du `PATH` du Mac (0.2.106) ne correspond pas au `Cargo.lock` (0.2.129) : l'auto-installation par version est donc indispensable et fonctionne (1 min 25).
- Bruit connu : cargo affiche `error: missing key for inline table element` sur `templates/starter/Cargo.toml` à chaque résolution du dépôt git de l'engine (D189, inoffensif).

## Reste

- Réépingler le `rev` de 1837 sur le SHA de `main` après le merge (le coordinateur alacod l'envoie) ; vérifier que `./jouer.sh --web` repart.
- `cargo check --locked` natif de 1837 (chemin `cfg(not(wasm32))` du gabarit) : **vert** (8 min, `Finished dev`), fait sur `104afd3f`.
- Manette dans Chrome, première ouverture sur le poste de William : partie du soir.

## Revue du coordinateur (sur `4f7c61df`) et suites

- **P2-1** `crates/game/build.rs` : `rerun-if-env-changed=APP_VERSION` **gardé** (le build script pose `APP_VERSION` par `cargo:rustc-env` : sans la ligne, `game` garde la version de sa première compilation, faux pour
  un jeu externe). Le partage de build est rétabli côté `Makefile` : `APP_VERSION=$(TEST_VERSION)` sur `test_crates`, `gen`, `sim`, `sandbox`, `test_alacod_cli`, `test_alacod_web` et `perf_generation` (« absente » et `v0.0.0` sont
  deux états pour cargo) ; commentaire de `Makefile` et § 108 mis à jour.
- **P2-2** `web_build.rs` : version par défaut stable (`<sha>-dirty`, `local` hors git, plus de secondes) ; `prune_other_versions` supprime `builds/<autre version>/<jeu>` avant de poser le nouveau build (les autres jeux de
  `<out>` restent) ; test `a_new_build_replaces_older_versions_of_its_game_only`.
- **P2-3** rapport, ligne `LIVRÉ`, passation supprimée : faits.
- **P3** message `#[ignore]` du test (`make test_alacod_web`) ; `cargo:rerun-if-changed=build.rs` dans `templates/starter/build.rs` (et le `build.rs` de 1837) ; `CARGO_ENCODED_RUSTFLAGS` retiré du `cargo build` wasm et les `rustflags` de
  `.cargo/config.toml` d'un jeu **documentés comme ignorés** (§ 108) ; `CLAUDE.md` (liste des commandes d'`alacod`).
