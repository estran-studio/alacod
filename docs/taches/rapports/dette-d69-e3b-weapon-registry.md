# dette-d69-e3b-weapon-registry : registre d'armes de simulation (D69 étape 2, E3b)

Branche `dette-d69-e3b-weapon-registry`, base `a3aa8456` (`origin/main` après E3a). Fiche `docs/taches/dette-d69-etape2.md` § E3b.
Refactoring pur : aucune trace ne bouge, aucun `BLESS`. Casse R2 (`Assets<WeaponsConfig>` / `Assets<MeleeWeaponsConfig>` dans la simulation).

## 1. Ce qui change

- `weapons.rs` (simulation) : `WeaponRegistry { weapons: BTreeMap<String, WeaponAsset>, melee: BTreeMap<String, MeleeWeaponAsset> }` (`Resource, Default, Clone`,
  ni rollback ni checksum : écrite hors `GgrsSchedule`, donnée statique d'une partie comme l'étaient les deux `Assets`). Les enveloppes d'asset, `WeaponAssetsPlugin`,
  `hide_new_melee_weapons`, `weapon_inventory_system` et `weapons_config_update_system` **quittent** ce fichier ; il réexporte toujours `combat::weapons`.
  Aucun type rollback créé, retiré ni modifié ; `WeaponRegistry` remplace `EffectAssets.weapons`, `SpawnAssets` et `DivisionAssets` (leurs deux champs d'armes).
- `weapon_assets.rs` (nouveau, **app**) : `WeaponsConfig` et `MeleeWeaponsConfig` (`Asset`, `HashMap` bevy, inchangées, les `.ron` ne changent pas), `WeaponAssetsPlugin`
  (`RonAssetPlugin` ×2, `init_resource::<WeaponRegistry>()`, au même endroit de `CoreSetupPlugin::build`), `sync_weapon_registry`
  (`PreUpdate.before(bevy_ggrs::RunGgrsSystems)`), `hide_new_melee_weapons`, `weapon_inventory_system`, `weapons_config_update_system` (déplacés tels quels).
- `GlobalAsset::weapons` / `melee_weapons` : handles inchangés, source de la synchro (E3d). L'exception `determinism-allowlist.json` suit les lignes déplacées
  (`weapon_assets.rs`) et couvre `copy_table` (itération d'un `HashMap` vers un `BTreeMap` : l'ordre de sortie est celui de la cible).
- `scenario/src/runner.rs` : `apply_weapon_overrides` **écrit dans `WeaponRegistry`** (il attend que `registry.weapons` soit non vide, la synchro le remplit en
  `PreUpdate`, l'override tourne en `Update`) ; `apply_player_overrides` lit le registre (plus de `GlobalAsset` ni de `Assets`). L'ordre
  `.before(game::weapon_assets::weapon_inventory_system)` **reste** (nouveau chemin) : `weapon_inventory_system` est toujours enregistré par `CoreSetupPlugin` (E5c retire
  les deux, fiche § E5c) ; la fiche disait « disparaît si la fonction passe côté app » mais l'ordre protège toujours de la lecture d'un `WeaponInventory` despawné.
- `scenario/src/generate.rs` : parse `WeaponsConfig`/`MeleeWeaponsConfig` en RON directement, chemin d'import seul changé.

### Sites migrés (`Res<Assets<WeaponsConfig>>` + `Assets<MeleeWeaponsConfig>` + `GlobalAsset` → `Res<WeaponRegistry>`)

| groupe | fichiers |
|---|---|
| créateurs | `character/player/create.rs` (`create_player`), `character/enemy/create.rs` (`spawn_enemy`) : le paramètre `global_assets` ne servait qu'à retrouver les handles, retiré ; `character/enemy/spawning.rs`, `waves/systems.rs` (`SpawnAssets.weapon_registry`), `champions.rs` (`DivisionAssets.weapon_registry`), `map_ldtk/game/{local,rooms,floors}.rs` (`LevelSpawnAssets`, `SpawnParams`, armes murales `spawn_level_weapon_locations`), `examples/character_tester.rs` |
| butin, économie, effets | `loot.rs` (`spawn_loot_weapon`, `spawn_drawn_loot`, `chest_interaction_system`), `shop.rs` (`shelf_interaction_system`), `progression.rs` (`weapon_drop_on_death_system`), `effects_runtime.rs` (`EffectAssets` : champ `global` retiré, `SpawnPattern`), tests de `effects_runtime.rs`/`effects_v3.rs` (`init_resource::<WeaponRegistry>()`) |
| présentation et outils | `feedback.rs` (`WeaponAudio` : `audio_config` lu dans le registre), `content_hot_reload.rs` (chemin d'enveloppe), `bots/src/input.rs` (`Res<WeaponRegistry>`, deux entrées de tuple en une), `scenario/src/runner.rs`, `scenario/src/generate.rs`, `powerups.rs` (test, chemin d'enveloppe) |

`grep -rnE "Assets<(Melee)?WeaponsConfig>"` (`--include=*.rs`, `crates games examples templates`) ne renvoie plus que `weapon_assets.rs` (la synchro et `weapons_config_update_system`, côté application)
et deux commentaires de doc. Aucun système de `GgrsSchedule` ni créateur de personnage ne contient plus d'`Assets` d'armes.

## 2. Moment du remplissage (décision D2) : preuve

`sync_weapon_registry` tourne **à chaque frame**, en `PreUpdate`, avant `RunGgrsSystems`, dans `WeaponAssetsPlugin` (installé par `CoreSetupPlugin` sans condition de
`headless` : natif avec ou sans rendu, headless, `run_with` de scénario, p2p). Il copie une table **vide** dont l'asset est lisible (chargement initial : ne dépend d'aucun
`AssetEvent`, écrits en `PostUpdate`) et **remplace** la table sur `AssetEvent::Modified` ; `Added`/`LoadedWithDependencies` ne font rien, donc une surcharge de scénario posée sur le
registre n'est jamais écrasée. `loading_asset_system` ne quitte `Loading` qu'une fois les assets chargés : le registre est plein avant le lobby, `GameLoading` et tout spawn.

| chemin | preuve | résultat |
|---|---|---|
| headless, `run_with` : zombies, testbed, isaac | `crates/scenario/tests/weapon_registry.rs` : au **premier** tick de `GgrsSchedule` (`FrameStart`), `WeaponRegistry` a autant d'armes à distance et de mêlée que les tables de `GlobalAsset` (non vides) ; en zombies chaque joueur porte déjà `machine_gun`, `pistol`, `shotgun` (4 de chaque, inventaires) et `bare_hands` ; chaque entrée d'inventaire a son entité | 3 verts |
| `PlayerScript::weapon` (`apply_player_overrides`) | `generated/zombies/weapon_pistol` : inventaire = `["pistol"]`, aucune mêlée ; `weapon_sword` : aucune arme à distance, une mêlée (lue dans `registry.melee`) | vert |
| surcharges de scénario (`apply_weapon_overrides`, `weapon_overrides`) | `ammo_burst` (`pistol`, `mag_limit: 1`) : au premier tick le registre porte `1`, l'**asset du fichier** a une autre valeur (≠ 1), et tous les pistolets créés (joueurs, avant le premier tick) copient `1` | vert |
| rechargement et ordre `before` | `asset_modification_refreshes_registry_but_overrides_survive` : surcharge posée sur le registre (f10) intacte à f29 (aucun `Modified`), `Assets::get_mut` à f30 → `Modified` → registre remplacé ; **la première exécution de `GgrsSchedule` après la modification voit déjà la valeur rechargée, la précédente voyait la surcharge**. Mutation jouée : synchro posée `.after(RunGgrsSystems)` → le test échoue (`left: Some(41)`, `right: Some(43)`), remise en `.before` → vert | vert |
| binaire natif `games/zombies` (`--no-default-features --features native`, `ALACOD_HEADLESS=1`, `ALACOD_REMOTE=1`, `RUST_LOG=info,game::weapon_assets=debug`) | journal : `loading of asset is done` à 20:34:09.893, `armes : WeaponRegistry rempli (4 armes)` et `armes de mêlée : … (6 armes)` à .900, `Map is loaded` à .987, `start local connection` à 09.999 | rempli avant la carte, les personnages et la session |
| scénarios à `weapon_overrides` dans la suite | `ammo_burst`, `ammo_shared_reserve`, `bench_bullets`, `friendly_fire_cursed`, `immune_tag`, `downed_*`, `powerup_max_ammo`… (15 fichiers) et tous les `weapon_*`/`shotgun_*`/`reload_*`/`effect_on_reload` générés | verts, traces inchangées |
| p2p réel (headless, deux clients, rollbacks réels) | `p2p-stress.sh` zombies et isaac | voir § 4 |

### Rechargement à chaud : essai manuel

Binaire natif zombies (`--features native` = `bevy/file_watcher`), partie pilotée par `scripts/alacod-remote` (`ALACOD_REMOTE_PORT=15733`), fichiers modifiés sur disque en cours de partie puis restaurés
(`git status games/` propre à la fin) :

- `weapons/melee/melee_weapons.ron`, `zombie_claws.damage` `2.5` → `40.0` à la frame 230 : journal `Reloaded weapons/melee/melee_weapons.ron` puis
  `armes de mêlée rechargées : WeaponRegistry mis à jour (6 armes)` dans la seconde (20:34:23), et une seconde fois à la restauration (20:36:06). Effet, joueur à 100 PV, partie avancée **frame par frame** : les coups reçus valent `2,5` (zombie 61, créé **avant**
  la modification) puis `40,0` (un zombie créé **après** : `spawn_enemy` copie `registry.melee["zombie_claws"]` à la création) ; un premier essai, avancé par pas de 40 frames, avait donné les mêmes deux valeurs (`−3,5` puis `−44,5`/`−43,5` quand les deux zombies frappaient dans le même pas). L'attribution
  d'un coup à un zombie précis par `last_attack_frame` est approximative (décalage d'une à quelques frames) ; les valeurs, elles, sont exactes et aucun coup de 40 n'a eu lieu avant la modification.
- `ZombieShooter/Sprites/Character/weapons.ron`, `pistol` : `armes rechargées : WeaponRegistry mis à jour (4 armes)` à la modification (20:36:29) et à la restauration (20:36:32). Les armes **déjà portées** ne changent pas dans cet essai (headless) : voir § 5.

Aucun scénario ne joue ce chemin : le test `asset_modification_refreshes_registry_but_overrides_survive` couvre le mécanisme (`Assets::get_mut` → `AssetEvent::Modified` → registre), l'essai ci-dessus le fichier réel.

## 3. Sonde avant/après (`make probe_game_sim`, `game` en ECS seul)

| | avant (`a3aa8456`) | après |
|---|---|---|
| fichiers du crate | 120 | 121 (`weapon_assets.rs`) |
| fichiers à fuite | **55** | **48** |
| erreurs de résolution | **762** | **752** |
| dont `asset` | 156 | 144 |

Fichiers **libérés de toute fuite** : `weapons.rs`, `loot.rs`, `progression.rs`, `champions.rs`, `effects_runtime.rs`, `character/enemy/create.rs`, `character/enemy/spawning.rs`, `waves/systems.rs`. Moins d'erreurs :
`feedback.rs` 38 → 37, `character/player/create.rs` 3 → 1, `shop.rs` 2 → 1 ; nouveau : `weapon_assets.rs` (21, l'enveloppe, app par construction).

**Cliquet : 21 → 22 fichiers (+ `weapons.rs`).** Les sept autres libérés n'y entrent pas : la sonde exige aussi que leur fermeture interne (`use crate::…`) soit dans la liste ;
`champions.rs` dépend de `balance.rs`, `character/config.rs`, `health/mod.rs`, `clock.rs`, `collider/mod.rs`, `items.rs`, `shop.rs` ; `enemy/create.rs` de `boss.rs`, `character/create.rs`, `ai/pathing.rs`, `ai/state.rs`… ; `loot.rs` de `interaction.rs`, `items.rs`, `shop.rs` ;
`effects_runtime.rs` de `health/mod.rs`, `effects_v3.rs`, `items.rs` ; `progression.rs` de `health/mod.rs`, `powerups.rs` ; `waves/systems.rs` de `global_asset.rs`, `balance.rs`… (essai : la sonde les refuse un à un).
La fiche les annonçait (« si E3a est fusionnée ») : la fermeture est la limite, à relancer à chaque étape (E3c : `powerups.rs`, `economy.rs`, `balance.rs` ; E3d : `global_asset.rs`). Commentaire daté dans `scripts/d69-sim-files.txt`.

## 4. Vérifications (résultats réels)

| vérification | résultat |
|---|---|
| `make schedule_order_diff BASE=origin/main` (deux fois : avant et après la dernière retouche de code) | **sortie 0** : 4 520 lignes de schedules, 645 types rollback identiques |
| `make test_scenarios_par SHARDS=3` sans BLESS (sur la branche, `main` n'a pas bougé : `a3aa8456`) | 3 tranches `ok`, **668 scénarios**, **aucun `.trace` modifié** (`git status` propre hors code et docs) ; jouée **deux fois** (422 s, puis 327 s après la retouche de revue) |
| `make test_crates` | `exit 0` (83 résultats `test result: ok`, dont les 6 tests de `weapon_registry`) |
| `cargo fmt --all -- --check`, `check-forbidden.sh` (40 occurrences auditées), `check-rollback-registration.sh`, `make test_tools` (25 tests), `make check_sim_boundary` | OK (13 crates SIM, 2 en dette) |
| `make lint` ×5 | 0 erreur (zombies 5 personnages 4 armes, testbed 39/25, throne 14/13, gungeon 11/21, isaac 14/3) |
| `make gen GAME=…` ×5 | 313 scénarios générés joués, tous `ok ok` (zombies 16, testbed 104, throne 39, gungeon 59, isaac 95), aucune trace différente |
| fuzz `ALACOD_FUZZ=0:8` / `ALACOD_FUZZ_BOMBS=1 ALACOD_FUZZ=0:8` (`--ignored`) | OK / OK (8 graines × 3 600 frames, 8 explosions par graine avec bombes, sans divergence) |
| `cargo check -p game --features lighting,ui_hui,debug_ui` ; build web `APP_VERSION=e3b cargo check -p zombies --target wasm32-unknown-unknown --no-default-features --features render` ; `cargo check --workspace --all-targets` | OK ×3 |
| **p2p-stress zombies** (`tests/scenarios/p2p_smoke.ron`, 3 000 frames, `MATCHBOX_PORT=3572`, `COMPOSE_PROJECT_NAME=b3`) | **1 486 frames resimulées** (client 0 : 21 rollbacks, profondeur max 11), codes de sortie 0 0, 0 `DesyncDetected`, 2 999 lignes identiques |
| **p2p-stress isaac** (idem) | **1 864 frames resimulées** (16 rollbacks, profondeur max 11), codes 0 0, 0 `DesyncDetected`, 2 999 lignes identiques |
| `make probe_game_sim SIM_LIST=scripts/d69-sim-files.txt` | sortie 0 ; 22 fichiers au cliquet ; § 3 |
| acceptation « simulation sans `Assets` d'armes » | `grep -rnE "Assets<(Melee)?WeaponsConfig>"` : `weapon_assets.rs` seul (synchro + rechargement des armes portées) |

Charge machine pendant les p2p : 43 à 52 (Mac partagé). **Captures `play_scenario --capture` avant/après non rejouées** : la règle 6 les demande « comme l'étape 1 », mais ni la section E3b ni la liste de vérifications
de la livraison ne les exigent et le seul changement de présentation est la source de `audio_config` (son, coupé en capture) ; aucune image ne peut changer.

## 5. Décisions et limites

- **Table vide plutôt qu'entrée absente** : contrairement à `CharacterConfigs` (entrée par entrée), le registre d'armes se remplit **table par table** (`weapons` vide → copiée, `melee` vide → copiée) ;
  une surcharge de scénario modifie une entrée sans rendre la table vide, donc n'est jamais écrasée. Une table d'asset vide ne se « remplit » pas (rien à copier, sans effet).
- **`Modified` remplace la table entière** (une arme retirée du fichier disparaît) : c'est ce que faisait `Assets<WeaponsConfig>` (bevy remplaçait l'asset) ; les surcharges de scénario sont alors perdues,
  comme E3a pour `CharacterConfigs`. Aucun scénario ne recharge un asset.
- **`weapons_config_update_system` inchangé** (déplacé) : il met à jour le composant `Weapon` des armes déjà portées sur `Modified`, mais sa requête exige `&Children` sur l'entité d'arme. `spawn_weapon_for_player` ne crée pas d'enfant ;
  seul `attach_weapon_visuals` (avec rendu) en ajoute (relecture indépendante). Mon test headless le constate (`mag_limit` des armes portées inchangé après `Modified`) : en headless aucune arme déjà portée n'est rechargée, avec rendu oui.
  Comportement d'avant E3b, hors périmètre. Les armes créées après un rechargement lisent le registre à jour (essai manuel ci-dessus).
- **`progression::weapon_drop_on_death_system` : pas de garde « registre vide »** (relecture, P2) : l'ancien `return` portait sur « asset non chargé », jamais sur « chargé et vide » ; un garde de cardinalité aurait supprimé des tirages RNG
  après un rechargement vers une table vide. Le chargement précède la simulation : le garde n'avait pas d'objet.
- **`feedback.rs` (`WeaponAudio`)** lit aussi le registre (présentation ; ce n'était pas dans la liste de la fiche) : une seule source des armes pour les sons, y compris sous surcharge de scénario.
- **Pas de `Arc`** : un clone par entrée au remplissage (jamais par frame), comme E3a.
- `std::sync::Mutex` dans `weapon_registry.rs` (test) : convention des autres tests du crate `scenario` (`character_configs.rs`, `extra_health.rs`) ; `parking_lot` n'est une dépendance directe d'aucun crate du workspace.
- Non fait, hors périmètre (fiche) : `powerups.rs`, `economy.rs`, `waves` (E3c), `GlobalAsset` (E3d), retrait de l'ordre `.before(weapon_inventory_system)` du runner (E5c).

## 6. Relecture indépendante

Agent `reviewer` en lecture seule sur le diff non commité (sans cargo) : **aucun bloquant**. Un P2 : le garde `registry.weapons.is_empty()` que j'avais posé dans `weapon_drop_on_death_system` change la consommation RNG après un rechargement
vers une table vide → **retiré** (suite rejouée ensuite). Deux P3 de doc : « il ne modifie rien » limité au headless (`docs/conventions.md`), `Assets<WeaponsConfig>` de la convention audio remplacé par `WeaponRegistry` → corrigés.
Points confirmés OK : équivalence site par site des créateurs (ordre de `starting_weapons`, `shoot_weapons`, `bare_hands`), des chemins de butin, boutique, champions, boss, bots et son ; moment et logique de la synchro (curseurs vidés,
`Added` sans effet) ; `apply_weapon_overrides` avant la création des joueurs et `apply_player_overrides` qui attend tous les joueurs ; aucun type rollback ni `HashMap` nouveau dans la simulation ; aucun appelant oublié.
Les chiffres du § 4 ne sont pas vérifiés par le relecteur (aucun cargo) : ce sont les miens.
