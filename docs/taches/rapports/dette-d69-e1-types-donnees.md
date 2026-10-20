# dette-d69-e1-types-donnees : types de donnée sortis de leurs fichiers de présentation (D69 étape 2, E1)

Branche `dette-d69-e1-types-donnees`, base `8e25731d` (origin/main). Fiche `docs/taches/dette-d69-etape2.md` § E1. Refactoring pur :
aucune trace de scénario ne bouge, aucun `BLESS`, aucun type rollback créé ni retiré, aucun système déplacé dans `GgrsSchedule`.

## 1. Types déplacés (octet pour octet : derives, `Hash`, `Debug`, champs)

| type | de | vers | appelants migrés |
|---|---|---|---|
| `CameraFollowOverride` | `camera/mod.rs` | `ui/hud_model.rs` | `camera/mod.rs`, `ui/mutation_screen.rs`, `scenario/runner.rs` (`game::ui::hud_model::…`) |
| `WorldLabel` | `legibility.rs` | `visual_requests.rs` (nouveau) | `legibility.rs`, `map_ldtk/game/floors.rs` |
| `SpriteRequest` | `world_sprites.rs` | `visual_requests.rs` | `world_sprites.rs`, `map_ldtk/game/floors.rs` |
| `UiStep`, `ScreenScript` | `ui/screens/mod.rs` | `ui/screens/model.rs` (`UiAction` et `ScreenId` y étaient déjà) | `ui/screens/mod.rs`, `replay.rs` (`Scenario::ui`), `scenario/runner.rs` |
| `FogSettings`, `FogMap`, `FogRoom`, `FogDoor` | `room_fog.rs` | `fog_map.rs` (nouveau) | `room_fog.rs`, `legibility.rs`, `feedback.rs` (4), `ui/hud.rs` (2), `powerup_visuals.rs`, `map_ldtk/game/collider.rs`, `map_ldtk/generation/building.rs` |
| `CharacterAppearance` (rollback, `Hash` manuel sans la couleur) | `character/visuals.rs` | `character/appearance.rs` (nouveau) | `character/create.rs`, `character/mod.rs` (enregistrement **à la même place**), `visuals.rs`, `legibility.rs`, `feedback.rs`, `feedback_v3.rs` |
| `InteractionInput` | `character/player/input.rs` | `interaction.rs` | `input.rs`, `interaction.rs` (2), `character/player/create.rs` (2) |
| `increase_frame_system` | `frame.rs` | `frame_counter.rs` (nouveau) | `core.rs`, `curses.rs`, `profile.rs` |
| `WaveDebugEnabled` | `waves/debug.rs` | `waves/state.rs` ; `init_resource` dans `WaveSystemPlugin` | `waves/debug.rs` ; `game::waves::WaveDebugEnabled` (réexport existant, de `state`) : `scenario/runner.rs`, `games/zombies`, `examples/character_tester.rs` inchangés |

Choix hors tableau, à relire :
- `FogMap::validate` et les trois méthodes de cellule `index`, `world_cell`, `fixed_cell` (privées dans `room_fog.rs`) suivent le type dans
  `fog_map.rs`, et deviennent `pub` : un `impl FogMap` ne peut pas rester dans `game_app` une fois les crates coupés (règle des
  implémentations inhérentes). Aucune valeur ne change.
- `UiStep`/`ScreenScript` : `model.rs` importe `bevy::prelude::Resource` (ECS seul) pour `ScreenScript`.
- Pas de `pub use` de compatibilité (règle 4) ; le réexport `waves::WaveDebugEnabled` existait déjà (même motif que `WaveState`).
- `docs/conventions.md` (§ brouillard) : chemin de `FogMap` corrigé.

## 2. Preuves

| vérification | résultat |
|---|---|
| `make schedule_order_diff BASE=origin/main` | **sortie 0** : `ordre des systèmes et des types rollback identiques (4520 lignes de schedules, 645 types)` ; `CharacterAppearance` en `0037` dans `zombies.rollback.txt`, base et tête |
| `make test_scenarios_par SHARDS=3` sans BLESS | 3 tranches `ok`, 668 scénarios fusionnés ; aucun `.trace` modifié (`git status` propre hors code et docs) |
| `make check_sim_boundary` | OK, 13 crates, 2 en dette D69 |
| `cargo fmt --all -- --check`, `scripts/check-forbidden.sh`, `scripts/check-rollback-registration.sh`, `make test_tools` (25 tests) | OK |
| `make lint` ×5, `make gen GAME=…` ×5 | 0 erreur ; tous les scénarios générés `ok ok` (aucune trace différente) |
| fuzz `ALACOD_FUZZ=0:8` (`fuzz_inputs`, 3600 frames par graine) | OK, 0 desync |
| `cargo check -p game --features lighting,ui_hui,debug_ui` | OK |
| build web (`cargo check -p zombies --target wasm32-unknown-unknown --no-default-features --features render`) | OK |
| `cargo check --workspace --all-targets` | OK |
| `make test_crates` | tous `test result: ok` (voir § 4 pour deux cibles rejouées) |

Le nom de module du type (`character::appearance::CharacterAppearance`) apparaît dans le dump détaillé `ALACOD_STATE_TRACE_FULL`, jamais dans
le hash ni dans les `.trace` (même constat que l'étape 1).

## 3. Sonde `make probe_game_sim` avant / après

| | avant (`8e25731d`) | après |
|---|---|---|
| fichiers de `game` | 114 | 118 (+4 nouveaux) |
| fichiers à fuite | 59 | 59 |
| erreurs de résolution | 781 | 781 |
| cliquet `scripts/d69-sim-files.txt` | 16 fichiers | **20** : + `visual_requests.rs`, `fog_map.rs`, `frame_counter.rs`, `character/appearance.rs` |

Le nombre de fichiers à fuite ne baisse pas : E1 ne retire aucune fuite, elle sépare des types de donnée de fichiers qui restent app
(`room_fog.rs`, `frame.rs`, `legibility.rs`…) ; ce que la sonde mesure ici, c'est le cliquet (+4) et la fermeture. Preuve de R6 : `replay.rs`
ajouté à la liste en essai échoue avec « replay.rs dépend de fichiers hors liste : character/player/input.rs » (et rien d'autre), conforme
à la fiche ; il a été retiré (`input.rs` est coupé en E4).

## 4. Limites

- Disque plein pendant la première passe de `make test_crates` (`No space left on device` à la compilation de `game` en test) : 37 Go
  d'exécutables de test périmés (plus de 24 h) de `../target/headless/deps` supprimés. Deux cibles (`content --bin alacod-profile`, `scenario
  --test bots`) ont alors échoué à l'exécution (binaire supprimé que cargo croyait frais), rejouées après `touch` de leur source : OK
  (5 tests pour `bots`). Reste de `make test_crates` : tous les résultats `ok`.
- Pas de p2p réel, pas de captures `cmp` : la fiche ne les demande pas pour E1 (aucun visuel ne change : mêmes systèmes, mêmes
  ordres).
