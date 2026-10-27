# X-2b « CLI : test, vidéo, build » — rapport

Branche : `integration-x2b` (base `adc042d4`, X-2a mergé). Contrat : `docs/taches/plan-1837-cli.md` D-b (X-2b),
`docs/conventions.md` §93.

## Ce qui change

- `alacod test [jeu] [id…] [--list] [--bless] [--json]` joue les scénarios **par le binaire du jeu**
  (plugin Rust compris) : `cargo run -q -- scenarios gen --play` (générés, `<jeu>/tests/generated`)
  puis `cargo run -q -- scenarios play` (`<jeu>/tests/scenarios`), en headless, `APP_VERSION` posée.
  Les traces se **comparent par défaut** (`--bless` les écrit ; X-2a les ignorait) ; un échec de trace
  est un échec (`ÉCHEC`, code 1). Un id inconnu des deux côtés est une erreur qui le nomme.
  `--engine` est accepté sans effet (avertissement). Rien n'est écrit hors du jeu.
- `scenarios play --list/--only` (`crates/scenario/src/cli.rs`), `scenarios capture <ron> --dir …
  [--every] [--follow] [--size] [--at] [--legend]` (`crates/entry/src/native.rs`, avec le plugin installé).
- `alacod video [jeu] <scénario> [--out] [--every 2] [--follow] [--size]` : capture par le binaire
  du jeu (rendu) + ffmpeg en `<jeu>/target/videos/<scénario>.mp4` + `.events.json` (mêmes réglages
  que `scripts/scenario-video`). Sans GPU ou sans ffmpeg : échec expliqué, images conservées.
- `alacod build [jeu] [--release] [--web]` : `cargo build` dans le jeu avec sa version (`APP_VERSION`
  = `APP_VERSION`/`WEB_BUILD_ID` si posée, sinon `v<version du Cargo.toml>`, plus `-<sha git>` pour
  un build immuable). `--web` : wasm + `wasm-bindgen` (version du `Cargo.lock` du jeu) + assets +
  `build.json` dans `<jeu>/target/web/<version>/<jeu>/`.
- Pas de runner dupliqué : tout passe par `scenario::cli` / `scenario::runner` via le binaire du jeu.
  Câblage d'import X-4b tenu hors de `main.rs` (aucune commande d'import ajoutée).

## Preuves (`make test_alacod_cli`)

- `test_command_plays_generated_scenarios_of_a_game_outside_the_engine` : rouge sans trace, vert après
  `--bless` (trace `ok`), rien écrit dans l'engine.
- `plugin_game_full_cli_flow` : `new` + fixture plugin, `new-content Crate`, lint, `test --bless`
  puis `test` vert (scénario `plugin` : 4 attentes, trace `ok`), `run --sandbox --headless` +
  `remote brief`, `build --release` (`APP_VERSION=v0.0.1`), `engine local/status/pin` hors ligne.
- `video_renders_a_game_scenario_when_a_gpu_is_present` : mp4 + moments clés non vides, ou limite
  exacte (ffmpeg/GPU manquant) sans fausse preuve.

## Résultats

(À renseigner à la vérification : commandes exactes, durées, verdicts.)

## Limites

- D144/D145 (`set_modifier`, ordre intra-frame des dons), D148 (pas de `--offline` : premier build
  peut toucher le réseau), D139 (`alacod lint` sans les règles Rust ; seul `<jeu> lint` les exécute).
- Vidéo non prouvée par rendu sur cette machine si ni GPU ni Mac (voir Résultats).
- `--web` non rejoué ici si `wasm-bindgen`/cible wasm absents (voir Résultats).
- Aucune trace de scénario existante n'a été re-BLESSée.
