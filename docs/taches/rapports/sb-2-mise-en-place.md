# sb-2-mise-en-place : rapport

Lot SB-2 du plan `docs/taches/plan-bac-a-sable.md` (§2, §3 D4, §4, §5). Branche `sb-2-mise-en-place`, depuis `origin/main` `9381b3ac`. Aucun type rollback, aucune ressource rollback, aucun système ajouté ou retiré :
c'est un **déplacement** de code, à résultat identique.

## API publique de `map_ldtk::game::profile` (`crates/map_ldtk/src/game/profile.rs`)

Données (les types de profil restent ceux de `game::replay`) :

| Item | Rôle |
|---|---|
| `PlayerProfile` (`from_script(&PlayerScript)`, `is_empty()`) | réglages d'un joueur : `tags`, `immune_to`, `modifiers`, `weapon`, `currency`, `mutations`, `dash`, `items`, `bomb`, `aim` (les champs de `PlayerScript` sans `inputs`/`bot`) |
| `Profile` (`from_scenario(&Scenario)`) | un `PlayerProfile` par handle + `characters`, `powerups`, `items`, `chests`, `shelves` |
| `ProfilePlugin(Profile)` | pose les ressources et les systèmes (ordre ci-dessous) |
| `ScenarioCharacterPlacements`, `ScenarioPowerUpPlacements`, `ScenarioItemPlacements`, `ScenarioChestPlacements`, `ScenarioShelfPlacements`, `PlayerOverrides` | ressources lues par les systèmes (ex-privées du runner) |
| `PlacementErrors` (`BTreeMap<usize, String>`), `ScriptedPlacement(usize)` | observation hors rollback (échecs de placement, `EntityRef::Placed`) |

Fonctions et systèmes (publics, réutilisables hors du runner) :

| Item | Rôle |
|---|---|
| `apply_player_profile(commands, id_factory, &ProfileTables, &PlayerTarget, &PlayerProfile, is_weapon) -> Result<(), String>` | **nouveau** : le corps de l'ancien `apply_player_overrides`, pour **un** joueur déjà créé. Valide d'abord (arme, objet, mutation inconnus, tables absentes) : `Err` ⇒ **rien en file** (le runner panique comme avant ; un appelant interactif, par ex. `alacod/give` de SB-3, peut refuser proprement) |
| `ProfileTables { progression, items, weapons }`, `PlayerTarget { entity, handle, children, effects, modifiers }` | arguments de la fonction ci-dessus |
| `spawn_character_placement(commands, &LevelSpawnAssets, id_factory, floor, index, &CharacterPlacement) -> Result<Entity, String>` | **nouveau** : validation (personnage, variante) + `local::spawn_character` ; mêmes messages qu'avant |
| `apply_scenario_character_placements`, `apply_scenario_powerup_placements`, `apply_scenario_item_placements`, `apply_scenario_chest_placements`, `apply_scenario_shelf_placements` | systèmes `GgrsSchedule`, inchangés (noms conservés) |
| `apply_player_overrides` | système `Update`, une fois, quand tous les joueurs du profil existent (et que les tables de mutations/objets sont posées) |

## Ordre des systèmes, avant / après

Avant (`scenario/src/runner.rs::build_app`) : `insert_resource` + `add_systems` en ligne ; après : `ProfilePlugin::build`, mêmes appels dans le même ordre, mêmes contraintes :

`GgrsSchedule` :
1. `apply_scenario_powerup_placements` : `Effects`, `.before(powerup_pickup_detect_system)`
2. `apply_scenario_item_placements` : `Effects`, `.after(1)`, `.before(item_consumable_pickup_system)`
3. `apply_scenario_chest_placements` : `Effects`, `.after(2)`, `.before(item_consumable_pickup_system)`
4. `apply_scenario_shelf_placements` : `Effects`, `.after(3)`, `.before(item_consumable_pickup_system)`, `.before(commands::apply_game_commands_pickups)`
5. `apply_scenario_character_placements` : `EnemySpawning`, `run_if(placements non vides)`, `.after(wave_spawning_system)`, `.after(enemy_spawn_from_spawners_system)`, `.before(commands::apply_game_commands_spawn)`

`Update` : `apply_player_overrides` (après `apply_weapon_overrides`/`apply_wave_overrides`/`apply_powerup_drop_chance_override`, qui restent dans le runner : mêmes positions relatives qu'avant).

Le seul changement d'emplacement : `ProfilePlugin` est ajouté **à la place** des anciennes lignes (entre `apply_powerup_drop_chance_override` et `LootOverrides`/`ShopOverrides`), dont les deux `insert_resource` restent dans le runner.
Ordre d'attribution des `GgrsNetId` : identique (même ordre d'itération des placements, même `GgrsNetIdFactory`, mêmes contraintes), prouvé par les traces.

## Ce qui a bougé

- `runner.rs` : −580 lignes (placements, `PlayerOverride(s)`, `apply_player_overrides`, `PlacementErrors`, `ScriptedPlacement`) ; il importe `PlacementErrors`, `Profile`, `ProfilePlugin`, `ScriptedPlacement`. Imports morts retirés. **Aucune copie** ne subsiste dans `crates/scenario`.
- `map_ldtk/src/game/mod.rs` : `pub mod profile;`.
- Commentaires de renvoi mis à jour (`game/replay.rs`, `combat/weapons/mod.rs`, `scenario/tests/weapon_registry.rs`, `docs/conventions.md` §28, §54 et les renvois de §6x) ; `docs/conventions.md` §85 (règles du module).
- Différence de comportement voulue et nulle sur les scénarios : `apply_player_profile` valide **avant** de poser (l'ancien code posait `dash`/`bomb`/`aim`/`tags` puis paniquait sur une arme inconnue) ; le runner panique toujours avec le même texte, l'état du monde à la panique n'est pas observable.

## Ce qui reste dans le runner

`apply_weapon_overrides`, `apply_wave_overrides`, `apply_powerup_drop_chance_override` (surcharges d'**enveloppes** de contenu, avant création des joueurs : pas du profil), `player_inventory`/`probe_player_inventory` (sondes d'attentes), `LootOverrides`/`ShopOverrides` (réglages de scénario, pas de profil), les attentes, les invariants, les entrées scriptées, les bots.
Non couvert, pour SB-3 : une application **à chaud** d'un profil à un joueur vivant (`alacod/give`) appellera `apply_player_profile` depuis un système de `GgrsSchedule` ou `Update` ; ce module n'impose pas encore de commande rejouable (cf. `GameCommands`).

## Tests

- Nouveau, `map_ldtk::game::profile::tests` (2) : `un_profil_pose_argent_tags_et_visee` (composants posés) ; `un_contenu_inconnu_est_refuse_sans_rien_poser` (arme/objet/mutation inconnus ou tables absentes ⇒ `Err`, **aucun** composant posé : l'argent valide du même profil n'est pas posé).
- Existants : `scenario/tests/{placement,weapon_registry,...}` et la suite complète de scénarios jouent le chemin déplacé.

## Preuves

Machine partagée (charge 7 à 62), tout joué au premier plan, **sans BLESS** ; `git status` propre hors code et docs après chaque étape (aucun `.trace`, aucun scénario généré modifié).

| Vérification | Résultat |
|---|---|
| `make schedule_order_diff BASE=origin/main` (sur `07cdf794`) | **vide** : « OK : ordre des systèmes et des types rollback identiques (4 550 lignes de schedules, 645 types) » (la base `9381b3ac` a la même taille que celle citée par les lots D69 : 4 520 lignes + les deux systèmes de K5-2 et leurs `apply_deferred`) |
| `make test_scenarios_par SHARDS=3` (sans BLESS) | **809 scénarios** fusionnés, 3 tranches `ok`, 3 787 s ; **0 trace bougée** |
| `test_crates` par paquet (`-p`, mêmes options que la cible, `--exact --skip scenarios`) | animation 2, sim_core 46, bevy_fixed 19, utils 4, behaviors 17, effects 38, bots 99, combat 119, items 52, run 38, stats 0, world 41, game 285, content 359, meta 18, map 26, telemetry 0, scenario 143, gungeon/isaac/testbed/throne/zombies/alacod 0 : **0 échec** ; `map_ldtk` avec `--features runtime` (sans, le module `game` n'est pas compilé, donc ni ses tests ni les miens) : 33 passés, 1 ignoré, dont les 2 nouveaux |
| `make lint` ×5 | 0 erreur (1 avertissement `loot/boss.ron` de gungeon, préexistant) |
| `make gen GAME=` ×5 (zombies, testbed, throne, gungeon, isaac) | exit 0 ×5, aucun fichier modifié |
| `cargo fmt --all -- --check` | OK |
| `scripts/check-forbidden.sh` | OK (51 occurrences auditées) |
| `make check_sim_boundary` | OK (13 crates SIM ; 2 en dette D69) |
| `make test_tools` | 38 tests OK |

Non fait : rendu (impossible sur le VPS). Aucune dette ouverte (D128–D129 inutilisées).
