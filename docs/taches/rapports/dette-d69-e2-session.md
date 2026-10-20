# dette-d69-e2-session : états et constantes de session hors de `core.rs` (D69 étape 2, E2)

Branche `dette-d69-e2-session`, base `201d083d` (fin de E1, `dette-d69-e1-types-donnees` ; fusion après E1). Fiche
`docs/taches/dette-d69-etape2.md` § E2. Refactoring pur : aucune trace ne bouge, aucun `BLESS`.

## 1. Déplacements

`crates/game/src/session.rs` (nouveau, devient `game_sim::session`) reçoit, **sans changer une ligne** (derives, doc, corps) :
`CoreSetupConfig` (+ `from_env`), `AppState`, `OnlineState`, `single_threaded_rollback_schedules`, `SIM_FPS`, `is_headless`.
`core.rs` garde `CoreSetupPlugin`, `get_default_plugin`, `PresentationPlugin`, `GameInfo`, `debug_ui_enabled` ; `CoreSetupPlugin::build`
et `PresentationPlugin::build` ne bougent pas d'une ligne (seul l'import de `core.rs` change, `finish` appelle la fonction déplacée).

Appelants migrés vers `…::session::…` (aucun alias, aucun `pub use`) : 31 fichiers de `game` (`args`, `balance`, `camera`, `clock`,
`content_hot_reload`, `feedback`, `global_asset`, `hub`, `interaction`, `jjrs/{local,mod,p2p,restart}`, `performance`, `remote`,
`room_fog`, `run_state`, `state_trace`, `ui/{disconnected,floor_transition,game_over,hud,hud_model,lobby,mutation_hui,mutation_screen,
screens/{hui,mod},weapon_hud}`), `bots`, `map_ldtk` (`game/local.rs`, `game/plugin.rs`), `scenario` (`runner.rs`, `events.rs`, 4 tests),
`games/{zombies,gungeon,throne,isaac,testbed}`, `examples/{character_tester,map_preview}`, `templates/starter`. Les lignes touchées de
`jjrs/p2p.rs`, `args/mod.rs` et `map_ldtk/game/local.rs` (code de D105/D109) sont des changements d'import seulement. `docs/etat-engine.md` :
chemin de `single_threaded_rollback_schedules`.

`grep -rnE "core::\(?(AppState|OnlineState|SIM_FPS|CoreSetupConfig)"` ne renvoie plus rien (`is_headless` et
`single_threaded_rollback_schedules` non plus), hors rapports historiques et fiche.

## 2. Preuves

| vérification | résultat |
|---|---|
| `make schedule_order_diff BASE=201d083d` | **sortie 0** : 4520 lignes de schedules, 645 types identiques |
| `make test_scenarios_par SHARDS=3` sans BLESS | 3 tranches `ok`, 668 scénarios ; aucun `.trace` modifié |
| `make check_sim_boundary`, `cargo fmt --check`, `check-forbidden.sh`, `check-rollback-registration.sh`, `make test_tools` | OK |
| `make test_crates` | OK (aucun échec) |
| `make lint` ×5, `make gen GAME=…` ×5 | 0 erreur, tous `ok ok` |
| fuzz `ALACOD_FUZZ=0:8` | OK |
| `cargo check -p game --features lighting,ui_hui,debug_ui`, build web wasm32 | OK |
| `cargo check --workspace --all-targets` | OK |
| `make probe_game_sim SIM_LIST=scripts/d69-sim-files.txt` | sortie 0 ; 119 fichiers, 59 à fuite, 781 erreurs (inchangé) ; **21** fichiers au cliquet (+ `session.rs`) |

## 3. Limites

Comme E1, E2 ne retire aucune fuite : elle rend `AppState`, `OnlineState`, `SIM_FPS`, `CoreSetupConfig` lisibles depuis un module de
simulation, ce qui lève la moitié « types » de R4 ; `core.rs` (APP) n'est plus importé par les 32 fichiers de simulation et
d'écrans. Pas de p2p réel ni de capture `cmp` (non demandés pour E2 : aucun système ni visuel ne change).
