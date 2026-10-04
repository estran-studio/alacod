# Rapport — m1-v2-lint-kinds-et-attentes : audit du lint des kinds (T1.12) et des attentes de M1 (T1.15)

**SHA de tête : celui annoncé dans le LIVRÉ** (commit de ce rapport). Branche partie de `c0f657e`
(LIVRÉ T1.3) ; `origin/main` mergé avant de livrer. Fiche locale
`docs/taches/m1-v2-lint-kinds-et-attentes.md` (non poussée). Worktree
`alacod_tasks/m0-v7-phase2-bots-finisent-le-clone/` ; agent : Claude Code (b1) ; date : 2026-10-04.

## État en cours

- **Fait** (écrit en lisant, avant le feu vert de compilation) : les deux tableaux, deux règles
  nouvelles, onze fixtures, test `Clock`, test de réenregistrement, enregistrement complété, liste
  unique des attentes dans `CLAUDE.md`, tableau du §3.
- **Prochaine étape** : compilation (feu vert), tests, `make lint` des trois jeux (la règle
  « personnage de `CharacterSpawn` connu » doit passer sur toutes les cartes existantes), suite
  complète, LIVRÉ.

## T1.12 — références entre contenus

« ✔ » : règle et fixture existaient ; « + fixture » : la règle existait sans fixture (ajoutée) ;
« + règle » : règle et fixture ajoutées. Rien ne change le jeu.

| Kind source | Champ | Cible | Règle (kind) | Fixture | État |
|---|---|---|---|---|---|
| `Pattern` | `Named(nom)` | `Pattern` | inconnu, cycle (`BrokenReference`) | `pattern_unknown_name` | ✔ |
| `Pattern`/arme | `projectile` d'un pattern (`on_expire`) | table `projectiles` de l'arme | absent (`BrokenReference`) | `projectile_broken_reference` | ✔ |
| arme | `on_expire` | `Pattern` instantané | temporel, `Scatter`, cycle (`OutOfRange`) | `projectile_temporal_pattern`, `pattern_scatter_on_expire`, `projectile_cycle` | ✔ |
| `Behavior::Shoot` | `weapon` | arme | inconnue (`BrokenReference`) | `behavior_shoot_unknown` | + fixture |
| `Behavior::Shoot` | `pattern` | `Pattern` | inconnu (`BrokenReference`) | `behavior_shoot_unknown` | + fixture |
| `Behavior::Shoot` | projectiles du pattern | table de l'arme | absent (`BrokenReference`) | `ranged_projectile_missing` | ✔ |
| `Behavior::Melee` | arme | arme de corps à corps | inconnue (`BrokenReference`) | `behavior_melee_unknown` | ✔ |
| `Behavior::Chase` | `profile` | profils de navigation | inconnu (`BrokenReference`) | `behavior_unknown_profile` | ✔ |
| `targeting` | `ignore` | tags des personnages | inconnu (`BrokenReference`) | `targeting_unknown_tag` | ✔ |
| `variants` | `skin` | `skins` du personnage | inconnu (`BrokenReference`) | `variant_skin_unknown` | ✔ |
| `variants` | `modifiers.stat` | `StatId` | inconnue (`Parse`) ; `MoveSpeed` sur IA (`OutOfRange`) | `variant_move_speed_ai` | ✔ |
| carte LDtk | `CharacterSpawn.variant` | variantes du personnage | inconnue (`BrokenReference`) | `variant_ldtk_unknown` | ✔ |
| carte LDtk | `CharacterSpawn.character` | personnage | inconnu (`BrokenReference`) | `map_character_unknown` | **+ règle** |
| `Cave` | `characters` | personnage | inconnu (`BrokenReference`) | `cave_unknown_character` | + fixture |
| `Cave` | gabarit `gabarit.ldtk` | fichier | absent (`BrokenReference`) | `cave_template_missing` | + fixture |
| `Cave` | dimensions, ratios, seuils | — | plages (`OutOfRange`) | `cave_out_of_range` | + fixture |
| `Surface` | `intgrid_value` | autres surfaces | doublon (`DuplicateId`) | `surface_duplicate_value` | ✔ |
| `Floors` | `levels` (carte) | `Map` | inconnue (`BrokenReference`) | `floors_unknown_map` | ✔ |
| `Floors` | `levels` (`cave:<id>`) | `Cave` | inconnue (`BrokenReference`) | `floors_unknown_cave` | + fixture |
| `game.ron` | `entry.clocks` | `Clock` | inconnue (`BrokenReference`) | `entry_clock_unknown` | + fixture |
| `game.ron` | `entry.difficulty` | `Difficulty` | sans fichier (`BrokenReference`) | `entry_difficulty_missing` | + fixture |
| `game.ron` | `entry.progression` | `Progression` | inconnue (`BrokenReference`) | `entry_progression_unknown` | **+ règle** |
| `Effect` | `SpawnPattern.pattern`, `.weapon` | `Pattern`, arme | inconnus (`BrokenReference`) | `effect_broken_reference` | + fixture |
| `Effect` | `OnGauge`/`GaugeAdd` | jauge d'une progression | inconnue (`BrokenReference`) | `progression_broken_reference` | ✔ |
| `Effect` | `Modifier.stat` | `StatId` | inconnue (`Parse`) | — (règle générique `Parse`) | ✔ |
| `Effect` | déclencheur, condition, action v2 | — | `Unsupported` | `effect_unsupported` | ✔ |
| `Progression` | `mutations`, `weapon_pool` | `Mutation`, arme | inconnues (`BrokenReference`) | `progression_broken_reference` | ✔ |
| `Mutation` | `effects` | (règles d'`Effect`) | — | `mutation_out_of_range` | ✔ |
| arme | `on_hit` `ApplyStatus.status` | `Status` | inconnu (`BrokenReference`) | `status_unknown` | ✔ |
| power-up | `ApplyStatus` | — | refusé (`OutOfRange`) | `powerup_apply_status` | + fixture |
| personnage | `test:` (`CharacterTest`) | gabarits actifs | (`OutOfRange`) | `character_test_*` | ✔ |
| `game.ron` | `generate_template` (carte, cible) | `Map`, personnage à `counts_hits` | (`BrokenReference`) | `generate_template_unknown_map`, `generate_template_target_without_hits` | ✔ |
| scénario | `characters`, `mode`, `progression` | personnage, mode, `Progression` | **hors lint** : un scénario n'est pas du contenu (`content::lint` ne lit que `game.ron`) ; le runner rapporte un placement invalide en échec d'attente (`PlacementErrors`, T1.13) et ignore une progression inconnue avec un avertissement | — | constaté |

Tableau du §3 des conventions complété avec toutes les règles de M1 (il s'arrêtait à M0/M2).

## T1.15 — attentes de M1

| Attente | Tâche (§) | Test unitaire (pass et échec) | `CLAUDE.md` | Réenregistrement |
|---|---|---|---|---|
| `BulletCount` | T1.1 (§16) | `expectations.rs` | ✔ | ✔ (`weapon_grenade`) |
| `HitsAtLeast` | T1.1 (§16) | `expectations.rs` | ✔ | ✔ (`weapon_grenade`) |
| `HasStatus` | T1.3 (§19) | `status_expectations_pass_and_fail` | ✔ | ✔ (`weapon_status_burn`, `status_slow_enemy`) |
| `StatusStacks` | T1.3 (§19) | idem | ✔ | ✔ |
| `EnemyState` | T1.4 (§22) | `enemy_state_*` | ✔ | ✔ (`enemy_keep_distance`) |
| `EnemyDistance` | T1.4 (§22) | `enemy_distance_entre_bornes` | ✔ | ✔ |
| `EnemyContactBefore` | T1.4 (§22) | `enemy_contact_before_echeance` | ✔ | ✔ (`enemy_charge`) |
| `EnemyNeverInWall` | T1.4 (§22) | `enemy_never_in_wall_*` | ✔ | ✔ (`enemy_wander`) |
| `EnemyVariant` | T1.5 (§25) | `enemy_variant_lit_la_variante` | ✔ | ✔ (`variant_fast`) |
| `CellState` | T1.6 (§21) | `cave.rs` | ✔ | ✔ (`explode_wall`) |
| `FloorIndex` | T1.8 (§17) | `cave.rs`, `floors.rs` | ✔ | ✔ (`portal_next_floor`) |
| `Clock` | T1.9 (§23) | **ajouté** : `clock_expectation_pass_and_fail` | ✔ | ✔ (`clock_events`) |
| `Gauge`, `Level`, `Mutations` | T1.10 (§27) | `progression_expectations_pass_and_fail` | ✔ | ✔ (`levelup_choice`, `effect_on_kill`) |

**Réenregistrement** (`m1_expectations_survive_rerecording`) : un scénario est rejoué, son
enregistrement (`ScenarioOutcome::recorded`, ce qu'écrivent `ALACOD_RECORD` et
`alacod-sim --save-scenario`) est relu, ses attentes sont recollées, il est rejoué : même trace,
attentes vertes. **Trou comblé** : l'enregistrement perdait `Scenario::progression` et tous les
réglages de joueur (`PlayerScript::weapon`, `tags`, `immune_to`, `modifiers`, `currency`,
`mutations`) — un scénario généré d'arme réenregistré rejouait avec les armes par défaut.
`RecordedSettings` garde désormais la progression et le `PlayerScript` de chaque joueur (sans
inputs ni bot). Une seule liste d'attentes dans `CLAUDE.md`, triée par tâche avec le § de chacune.

## Vérifié

Après merge d'`origin/main` (dbc3de8), `CARGO_BUILD_JOBS=2`, profil `headless` :

- `cargo test -p content` : vert (dont `m1_audit_fixtures` et le tableau « une seule sorte » avec
  les 11 nouvelles fixtures).
- `cargo test -p game recording` : vert (round-trip `RecordedSettings` avec progression et
  réglages des joueurs).
- `cargo test -p scenario --test expectations` : 44 passés, dont `clock_expectation_pass_and_fail`
  et `m1_expectations_survive_rerecording` (12 scénarios réenregistrés : aucune attente en échec,
  trace identique).
- `make test_scenarios` : vert, **aucune trace déplacée**, aucun bless.
- Tests des crates (scenario, run, combat, game, content, map_ldtk, map, sim_core, stats, bots,
  effects, behaviors, world, utils) : verts.
- `make lint` (testbed, zombies, throne) : vert — la nouvelle règle CharacterSpawn ne casse
  aucune carte existante.
- `cargo fmt --check` (après `cargo fmt`), `check-forbidden`, `check-rollback-registration`,
  exemples : verts.
- `make gen` testbed et zombies : vert, aucun fichier généré modifié. `make gen GAME=throne` :
  panique connue sur `arsenal` (arène du testbed, déjà notée dans les rapports throne et D39),
  sans lien avec cette tâche.

## Non fait / dettes

- Scénarios hors lint (voir tableau) : un `content::lint` des scénarios serait une tâche à part.
- `Surface` : une valeur IntGrid de la couche `Surfaces` d'une carte sans surface déclarée n'est
  pas vérifiée (il faudrait lire les couches IntGrid des cartes).
