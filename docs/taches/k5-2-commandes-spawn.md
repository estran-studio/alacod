# k5-2-commandes-spawn — commandes de partie indexées par frame, spawns à la voix, `Stats` rechargées (K5, V1)

Lire d'abord `docs/taches/README.md` (§1, §2, §4, §5 preuve de trace, §7), `CLAUDE.md` **en entier** (déterminisme, `FrameEvents`, `despawn_rollback`,
checklist « Nouveau système GGRS », « Parité des types vides », `core_sim.rs`), `docs/editeur-vocal.md`, `docs/conventions.md` §28 (placements de
personnages) et la section « Éditeur par l'IA » ajoutée par K5-1. Branche `k5-2-commandes-spawn` depuis `main` **après le merge de
`k5-1-partie-pilotable`** et, pour `core_sim.rs`, après la frontière S1-6 de la semaine de polissage (ou en le disant à William). Estimation : 3 j.
Cette tâche touche la simulation : **aucune trace existante ne doit bouger** (`BLESS` interdit).

## But

William dit « spawne un zombie_runner devant moi » : `scripts/alacod-remote spawn zombie_runner --at 420,300` crée l'ennemi **dans la
simulation, de façon rejouable** (une session sauvée par `alacod-remote save` puis rejouée par le runner donne la même trace). Et « plus
rapide » dans un RON de personnage (`movement.max_speed`) change aussi le joueur **vivant** (aujourd'hui seuls les spawns futurs le voient).

## À faire

1. **`crates/game/src/game_commands.rs`** (nouveau, `pub mod game_commands;` dans `lib.rs`) :
   ```rust
   #[derive(Debug, Clone, PartialEq)]
   pub enum GameCommand {
       SpawnCharacter { character: String, x: Fixed, y: Fixed, team: Option<Team>, variant: Option<String> },
       SpawnPowerUp { id: String, x: Fixed, y: Fixed },
       SpawnItem { id: String, x: Fixed, y: Fixed },
   }
   /// Non rollback, hors checksum. JAMAIS dépilée : indexée par la frame où la commande s'applique.
   #[derive(Resource, Default, Clone, Debug)]
   pub struct GameCommands(pub BTreeMap<u32, Vec<GameCommand>>);
   ```
   Une file dépilée casserait la resimulation (le spawn disparaîtrait au rejeu, le checksum diffère). Initialiser avec
   `app.init_resource::<GameCommands>()` dans `CoreSimPlugin` (**pas** `rollback_*` : la ressource n'est pas dans le snapshot ; elle est
   identique à chaque resimulation de la même frame).
2. **Systèmes de simulation** (`GgrsSchedule`), calqués sur le runner : `apply_scenario_character_placements`
   (`crates/scenario/src/runner.rs:825-885`, set `RollbackSystemSet::EnemySpawning`, ordonné `.after(wave_spawning_system)` et
   `.after(enemy_spawn_from_spawners_system)`), `apply_scenario_powerup_placements` (`:902-930`, `Effects`,
   `.before(powerup_pickup_detect_system)`), `apply_scenario_item_placements` (`:946-970`, `Effects`, `.after(powerup…)
   .before(item_consumable_pickup_system)`). Écrire `apply_game_commands_spawn` (EnemySpawning) et `apply_game_commands_pickups` (Effects) qui lisent
   `GameCommands.0.get(&frame.frame)` (`FrameCount`), **inertes sans commande** (`run_if(|c: Res<GameCommands>| !c.0.is_empty())`),
   ordre = ordre du `Vec`. Net ids par `GgrsNetIdFactory` (rollback). Chaque exécution journalise
   `info!("ggrs{{f={} editor_command kind=spawn_character net_id={} character={} pos=({},{})}}", …)` (jamais `Entity`).
3. **Déplacer `spawn_character`** : aujourd'hui `map_ldtk::game::local::spawn_character` + `LevelSpawnAssets` (`crates/map_ldtk/src/game/local.rs:39-53`
   et `:311-351`), alors que `game` ne dépend pas de `map_ldtk` (`crates/game/Cargo.toml`). **Choix recommandé** : *ne pas déplacer*, poser les
   deux systèmes de l'étape 2 dans `map_ldtk` (`crates/map_ldtk/src/game/commands.rs`, enregistrés par `LdtkLocalGamePlugin` ou le plugin de
   `map_ldtk/src/game/plugin.rs`), parce que `LevelSpawnAssets` tire `CollisionSettings`, `ResolvedBalance`, `WeaponRegistry`, `DifficultyReader` (tout
   existe dans `game`) mais `spawn_enemy`/`create` sont déjà dans `game` : un déplacement d'une centaine de lignes ne sert à rien tant que le
   runner n'en dépend pas autrement. Si la fiche K5-3 ou le rapport montre que `character_tester` (qui n'appelle pas `LdtkLocalGamePlugin`) doit
   aussi spawner, déplacer alors `LevelSpawnAssets` + `spawn_character` dans `game::character::enemy::spawning` et laisser un `pub use` dans
   `map_ldtk` (mettre à jour les appelants : `grep -rn "spawn_character\|LevelSpawnAssets" crates games`). Dire le choix et la raison au rapport.
   **Ordre** : s'enregistrer avec le même `.after(...)` que le runner pour que les net ids restent déterministes (la numérotation dépend de l'ordre).
4. **Enregistrement dans `core_sim.rs`** : `init_resource::<GameCommands>()` **avant les cinq enregistrements de clôture**
   (`GgrsNetIdFactory`, `FrameCount`, `FixedTransform3D`, `GgrsNetId`, `RngStreams`, `core_sim.rs:~128-145`), les systèmes avec leurs
   `.in_set`. `ambiguity_detection: Error` (`core_sim.rs:63`) : si une paire est signalée, ordonner explicitement ; vérifier avec
   `make schedule_order_diff` (ajouts seulement, aucun déplacement).
5. **Handlers BRP** (`remote.rs`, garde `guard_hot_edit` de K5-1) : `alacod/spawn_character {character, x, y, team?, variant?}`,
   `alacod/spawn_powerup {id, x, y}`, `alacod/spawn_item {id, x, y}`. Le handler lit `FrameCount.frame`, écrit dans `GameCommands` à
   `frame + 1` (la frame suivante, pas la courante : le système de la frame en cours a peut-être déjà tourné) ; coordonnées en `f32` JSON
   converties **ici** par `Fixed::from_num` (hors simulation, une seule fois), jamais dans `GgrsSchedule`. Valider avant d'écrire :
   `CharacterConfigs.get(character)`, `PowerUpsTable`, `ItemTable` (renvoyer une erreur JSON-RPC « inconnu » plutôt que de laisser le système
   avaler un warn). `alacod/state` ajoute `commands: [{frame, kind}]` des commandes futures.
6. **Enregistrement** (`crates/game/src/recording.rs:40-53` `RecordedSettings`, `:98-108` défaut, `from_scenario`) : `RecordedSettings` a déjà
   `characters: Vec<CharacterPlacement>`, `powerups: Vec<PowerUpPlacement>`, `items: Vec<ItemPlacement>` (`replay.rs:157-180`). Quand un handler pose une
   commande, **ajouter aussi** le placement équivalent (`at_frame = frame + 1`) à l'`InputRecorder` (méthode `record_command(&mut self, &GameCommand, frame)`) ;
   `save_recording` (`remote.rs`) écrit alors un scénario que le runner rejoue par ses propres systèmes de placement (`runner.rs:825-970`).
   `Resource` `InputRecorder` n'existe que si l'enregistrement est actif : `Option<ResMut<InputRecorder>>`.
7. **`Stats` des entités vivantes** : `create_character` copie `MoveSpeed`, `EnemyMoveSpeed`, `Acceleration`, `SprintMultiplier`, `MaxHealth`… de la
   config dans `Stats` au spawn (`crates/game/src/character/create.rs:126-233`). Ajouter dans `character/config_asset.rs` (près de
   `sync_character_configs`, `PreUpdate.before(RunGgrsSystems)`) un système `refresh_live_stats` qui, sur `AssetEvent::Modified` d'un
   `CharacterConfigAsset`, pour chaque entité portant `CharacterConfigKey(key)` (`character/config.rs:246`, non rollback) **dans l'ordre
   `GgrsNetId`** (`order_mut_iter!`), réécrit dans `Stats` la **base** de ces cinq stats depuis la nouvelle config, **sans toucher `Modifiers`** ni
   `Stats` posées par `config.stats` du RON (« RON gagne toujours » : refaire la même séquence que `create.rs:128-150`, en factorisant une
   fonction `base_stats(config) -> Stats` utilisée par les deux). Ne pas dépasser le « local d=0 » : écrire hors `GgrsSchedule` est sûr à d=0
   seulement (K5-1). Interaction à connaître : `sync_health_from_stats` (`character/health/mod.rs:666`, `RollbackSystemSet::Status`) recopie
   `MaxHealth` résolu dans `Health.max` à chaque frame et borne `current` ; ne pas écrire `Health` soi-même. Les personnages **non** rechargés
   (autre `key`) ne sont pas touchés.
8. **`scripts/alacod-remote`** : `spawn <personnage> --at X,Y [--team T] [--variant V]`, `spawn-powerup <id> --at X,Y`, `spawn-item <id> --at X,Y` ;
   `brief` montre les commandes en attente. `docs/editeur-vocal.md` : méthodes, vocabulaire « spawne un … devant moi » (la position du joueur
   vient de `alacod-remote state`, `+60` px dans la direction de visée, à décider et écrire).

## Règles

- Tout CLAUDE.md : `Fixed` seul dans `GgrsSchedule`, `GameCommands` en `BTreeMap`, itérations ordonnées par `GgrsNetId`, `despawn_rollback()` si
  une commande détruit, logs `ggrs{f=… editor_command …}`, pas de `Message` bevy, pas de `HashMap`.
- Les systèmes sont **inertes sans commande** et n'ajoutent **aucun** composant ni ressource rollback : donc aucune parité de types vides à
  gérer. Si tu en ajoutes un, relire « Parité des types vides » de CLAUDE.md et `docs/conventions.md` §10.
- Ne pas modifier `RunHistory`, le checksum, ni les scénarios existants. Pas de `BLESS`.
- Frontière : `game_commands.rs` dans `game` (ou `map_ldtk` selon l'étape 3) ; aucun import de rendu (`make check_sim_boundary`).

## Vérification

Build : `--profile headless --no-default-features $(./scripts/dev-features.sh features)`, une commande cargo à la fois.
1. Tests nouveaux : `crates/scenario/tests/game_commands.rs` :
   (a) scénario testbed, `GameCommands` pré-rempli par la closure de `run_with` à la frame 30 (un `SpawnCharacter` de `target`) : l'entité existe
   à la frame 31, avec `CharacterConfigKey("target")` et un net id attendu ;
   (b) **rejeu** : jouer (a), `outcome.recorded` (`ScenarioOutcome.recorded`, runner) contient le placement `characters` à `at_frame 31` ; le rejouer par `scenario::run`
   donne la **même trace** (`outcome.trace` égal) ;
   (c) synctest : le même test en d=2 (défaut du runner) ne desync pas (preuve du « jamais dépilée ») ;
   (d) `max_speed` édité : comme `character_configs.rs:136` (`assets.get_mut(&handle).unwrap().0.movement.max_speed = …` à la frame 30), après
   2 updates `Stats.get(&StatId::MoveSpeed)` du joueur vivant est la nouvelle valeur et `Modifiers` est inchangé ; un modificateur `Mul 0.5` posé avant
   survit (`StatReader` résout `base × 0,5`).
2. `make test_crates` ; sous-ensemble : `ALACOD_SCENARIO=idle,four_players_shooting,clone_solo,throne_solo,gungeon_full_run,isaac_full_run,
   remote_first_fight make test_scenarios` sans `BLESS`, puis la suite par tranches `ALACOD_SCENARIO_SLICE=k/3` : **0 trace différente**.
3. `make schedule_order_diff BASE=main` : l'ordre des systèmes existants est inchangé (seuls les deux systèmes ajoutés apparaissent) ;
   `ALACOD_SCHEDULE_ALL=1` pour les paires non ordonnées. `make check_sim_boundary`, `scripts/check-forbidden.sh`.
4. Preuve en jeu (headless) : `make editor GAME=zombies HEADLESS=1`, `alacod-remote spawn zombie_1 --at 100,100`, `step 5`, `brief` montre
   l'ennemi (`config` = `zombie_1`) ; `alacod-remote save /tmp/s.ron` puis `ALACOD_SCENARIO=…`/runner : trace identique à la session.

## Livraison

Commit `k5-2 : GameCommands, spawn_character/powerup/item, Stats vivantes rechargées` ; rapport `docs/taches/rapports/k5-2-commandes-spawn.md`
(choix de l'étape 3, sorties de `schedule_order_diff`, nombre de traces comparées, ce qui n'a pas pu être vérifié) ; journal `docs/taches.md` §10 ;
`LIVRÉ k5-2-commandes-spawn <sha>`. Mettre à jour `docs/conventions.md` (« Éditeur par l'IA ») et `CLAUDE.md` (méthodes ; ligne « Scénarios »
si une attente est ajoutée : aucune ne l'est ici). Jamais de push.

## Pièges

- **Net ids** : l'ordre des spawns d'une frame décide des `GgrsNetId` ; le runner place ses spawns `.after` les spawns du jeu
  (`runner.rs:~395-407`) : refaire la même chose, sinon les traces de scénarios à vagues bougent… elles ne devraient pas, puisqu'inertes sans commande.
- **`spawn_character` exige** `ResolvedBalance.health_max_by_character` (panique sinon : `local.rs:~325`) : n'accepter que des clés présentes
  (`alacod/spawn_character` valide avant). `difficulty.current()` et `variant_context_at(floor)` : prendre ceux du runner (`runner.rs:860-875`).
- **Resimulation** : si `GameCommands` était vidée après usage, la frame resimulée ne spawnerait plus. Elle grossit sans borne : acceptable (une
  session d'édition), la vider au `GameLoading` (`OnEnter(GameLoading)`, hors rollback) pour qu'un `restart` reparte propre — et vider aussi les placements
  de l'enregistreur n'est **pas** voulu (la session sauvée garde tout).
- Le handler tourne dans `RemoteLast` (`bevy_remote` `lib.rs:832-847`, après `Last`) : la frame lue est celle de l'update courant ; `+1` suffit tant
  que le handler ne s'exécute pas **pendant** `GgrsSchedule` (ce n'est pas le cas).
- `FrameCount` est rollback ; la lire dans le handler donne la dernière frame confirmée simulée. Après une commande, `alacod-remote step N` pour la voir.
- `CharacterConfigKey` n'est pas rollback : après un rollback profond une entité respawnée par bevy_ggrs la perd (CLAUDE.md §9) ; avec d=0 aucun
  rollback n'a lieu, ne pas s'en inquiéter ici, mais ne pas s'en servir dans la simulation.
- `AssetEvent::Modified` pour un RON invalide n'est pas émis (mesuré en K5-0) : `refresh_live_stats` n'a rien à gérer pour ce cas.
- Les armes portées sont déjà rechargées par `weapons_config_update_system` (`weapon_assets.rs:179-199`) mais **pas la mêlée** (risque 7 du plan,
  non mesuré) : si une édition de `melee_weapons.ron` ne change rien en partie, ouvrir une dette (D131) plutôt que de l'étendre ici.
