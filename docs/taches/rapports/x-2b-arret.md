# X-2b « CLI : test, vidéo, build » — brouillon conservé (2026-10-27, gelé avec X-1/X-2a)

- **Branche poussée, jamais à fusionner** : `x-2b-brouillon-sur-x2a` @ `13b464e0`. **Base `adc042d4` (intégration X-1 + X-2a), pas `main`.**
- **Non vérifié** : brouillon d'un sous-agent arrêté par quota, commité tel quel. Aucun build ni test lu ; son rapport dit « À renseigner ».
- **Contenu** (selon `docs/taches/rapports/x-2b-cli-test-video-build.md` de la branche) :
  - `alacod test` joue les scénarios générés puis écrits à la main **par le binaire du jeu**, avec `--list`, des ids, `--bless` et `--json`. Les traces sont comparées par défaut.
  - `scenarios play --list/--only` et `scenarios capture` sont ajoutés à `entry` (avec le plugin).
  - `alacod video` : capture et ffmpeg écrits dans `<jeu>/target/videos/`.
  - `alacod build [--release] [--web]` passe `APP_VERSION` ; `--web` lance wasm-bindgen et écrit `build.json`.
  - Trois e2e (`plugin_game_full_cli_flow`…) sont écrits dans `cli.rs`, non joués.
- **Fichiers** : `bin/alacod/main.rs` (+892 lignes), `entry/src/native.rs`, `scenario/src/cli.rs`, `cli.rs`, docs (§93).
- **Reprise** : seulement si X-1/X-2a dégèlent, après la reprise de `x-2a-arret.md`. Partir de cette branche ou la rejouer, compiler, jouer `make test_alacod_cli`,
  prouver la vidéo sur le Mac et `--web` avec wasm-bindgen.
