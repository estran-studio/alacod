# K5-3 — remote et mode éditeur sans variable dans les apps de test

Branche `k5-3-apps-de-test` (depuis `k5-2-commandes-spawn`). Outillage seulement : aucune simulation, aucun enregistrement rollback, aucune trace.

## Fait
- `CoreSetupConfig::editor` (`#[serde(default)]`, `from_env` → `false`) ; les trois constructions littérales (`core_sim.rs`, `scenario/runner.rs`, `tests/schedule_order.rs`) posent `editor: false`.
- `core.rs` : `editor = config.editor || ALACOD_EDITOR=1` → insertion d'`EditorMode` **avant** le plugin remote ; `RemoteControlPlugin::new(editor)` remplace `from_env()` (seul appelant : `core.rs`) : `Some` si `editor` ou `ALACOD_REMOTE=1`.
- `examples/character_tester.rs` : `game_config.editor = true`. `alacod/state` retombe bien sur le dossier de `GameRoot` (`jeu=zombies`, `carte=None`).
- `Makefile` : `character_tester` reçoit `$(DEV_FEATURES)` (une seule configuration de features par arborescence).
- Docs : `docs/editeur-vocal.md` §1 (tableau des apps de test, `map_preview` inchangé, Map Tester = `make editor` + `restart --seed`), `docs/jouer.md` (« Éditer en jouant »), `docs/taches/plan-editeur-vocal.md` (K5-1/2/3 livrés).

## Décisions
- **`check_distance = 0` (point 3)** : `BaseArgsPlugin` lit l'environnement seul et passe avant `CoreSetupPlugin`. Solution retenue, la plus locale : l'exemple fait `std::env::set_var("ALACOD_EDITOR", "1")` en première ligne de `main()` (`#[cfg(not(target_arch = "wasm32"))]`, édition 2021 donc sans `unsafe`, aucun autre thread à ce stade), en plus de `game_config.editor = true`. `BaseArgsPlugin` n'a pas changé. Conséquence : l'exemple est en mode éditeur même si `CoreSetupConfig` seul le dirait ; les jeux qui voudront la même chose sans variable devront répéter ce `set_var` ou on généralisera `BaseArgsPlugin` (non fait : pas demandé).
- **Jeux (point 5)** : `games/*/src/main.rs` inchangés (un jeu lancé à la main reste un jeu) ; `make editor` garde `ALACOD_EDITOR/REMOTE/LABELS`. À confirmer par William.
- `map_preview` inchangé (pas de session GGRS).

## Vérifications
- `make test_crates` : vert (tous les `test result: ok`, 0 échec).
- `cargo check --examples --profile headless --no-default-features --features native` + features de dev : `Finished`.
- Wasm : cible `wasm32-unknown-unknown` installée ; `cargo check -p game --target wasm32-unknown-unknown --no-default-features` : `Finished` (l'exemple lui-même n'est pas compilé en wasm ici ; son `set_var` est sous `cfg(not(wasm32))`).
- `make test_scenarios SCENARIO=idle,four_players_shooting,clone_solo,remote_first_fight` sans BLESS : `5 passed; 0 failed`, aucune trace modifiée ; `schedule_order` : `1 passed; 0 failed` (2 ignorés).
- Preuve headless, sans `ALACOD_EDITOR` ni `ALACOD_REMOTE` :
  `ALACOD_HEADLESS=1 ALACOD_MUTE=1 cargo run --example character_tester --profile headless --no-default-features --features native <dev-features> -- --local-port 7000 --players localhost`
  - `alacod-remote discover` : liste `alacod/clear_modifiers … alacod/spawn_character|item|powerup, alacod/state, alacod/step, rpc.discover, world.*`.
  - `alacod-remote brief` : `editor=True online=False check_distance=0 run.step=Playing { since_frame: 0 } jeu=zombies carte=None graine=12345` / `frame 2138 pause=False vague={'killed': 0, 'number': 1, 'phase': 'Spawning', 'to_spawn': 5}` / `joueur 0 (0.0,0.0) pv 100.0/100.0 arme machine_gun 30x240`.
  - `make editor GAME=zombies HEADLESS=1` : `brief` → `editor=True online=False check_distance=0 run.step=Ended { at_frame: 1051, outcome: Defeat } jeu=zombies carte=floor:la_fonderie graine=961307729` (le joueur immobile est mort : la partie tourne sans `resume`). Processus tués.
- `cargo fmt --all -- --check` propre ; `check-forbidden.sh` OK (51 occurrences auditées) ; `make check_sim_boundary` : 13 crates respectés, 2 en dette D69.

## Non essayé
Fenêtre et clavier (pas d'affichage sur cette machine) ; `make character_tester` lui-même (essayé par la commande cargo équivalente, en headless).

## Incident
Une première `make test_crates` lancée sans `CARGO_TARGET_DIR` a créé `alacod-k5-1/target` (26 Go) et saturé le disque ; supprimé (artefact à moi), relance avec la variable. L'export doit être refait dans **chaque** appel bash.
