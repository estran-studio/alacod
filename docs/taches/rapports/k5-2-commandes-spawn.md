# k5-2-commandes-spawn : rapport

Branche `k5-2-commandes-spawn`. La tâche a été commencée par un premier agent (`8b3432f7`, `0f7f93a3`) arrêté sans tests ni vérifications ; reprise, relecture critique et livraison ici.
Aucun type rollback nouveau, aucune parité de types vides à gérer : `GameCommands` est une ressource non rollback, les deux systèmes sont inertes sans commande.

## Fait

- `game::game_commands` : `GameCommand` (`SpawnCharacter`, `SpawnPowerUp`, `SpawnItem`) et `GameCommands(BTreeMap<u32, Vec<GameCommand>>)`, non rollback, **jamais dépilée**, vidée à `OnEnter(GameLoading)` (`clear_game_commands`).
- `map_ldtk::game::commands` : `apply_game_commands_spawn` (`EnemySpawning`, après `wave_spawning_system` et `enemy_spawn_from_spawners_system`) et `apply_game_commands_pickups` (`Effects`, avant `powerup_pickup_detect_system` et `item_consumable_pickup_system`), `run_if` « la table n'est pas vide », net ids par `GgrsNetIdFactory`, journal `ggrs{f=… editor_command kind=… rank=…}` (sans `Entity`).
  **Choix de l'étape 3 : `spawn_character` et `LevelSpawnAssets` ne sont pas déplacés** (ils vivent dans `map_ldtk`, `game` n'en dépend pas ; le runner et le jeu les utilisent déjà tels quels ; `character_tester` n'est pas dans le périmètre). Les deux systèmes sont enregistrés par `LdtkLocalGamePlugin` (`register`).
- `remote.rs` : `alacod/spawn_character`, `alacod/spawn_powerup`, `alacod/spawn_item` (garde `guard_hot_edit`, ids validés → `INVALID_PARAMS`, commande à `frame + 1`, `Fixed::from_num` dans le handler, placement ajouté à l'`InputRecorder`) ; `state` expose `commands` et, par joueur, `facing`.
- `recording.rs` : `InputRecorder::record_command` ; `start_native_recording` reporte les commandes déjà posées avant le premier tick d'une run (voir « Corrigé à la relecture »).
- `character::create::base_stats` (partagée par `create_character`) et `config_asset::refresh_live_stats` (`PreUpdate`, avant `RunGgrsSystems`) : sur `AssetEvent::Modified`, les bases de `Stats` des entités portant la clé rechargée sont réécrites (`Stats::overlay`), dans l'ordre `GgrsNetId`, hors `Modifiers` et `Health`.
- `scripts/alacod-remote` : `spawn`, `spawn-powerup`, `spawn-item` (`--at X,Y` ou `--ahead [PX]`), `brief` montre les commandes en attente.
- **« Devant moi »** : décision écrite au guide (`docs/editeur-vocal.md` §2.1) et implémentée **côté script** : `--ahead` (60 px par défaut) lit `players[].x`, `y`, `facing` (l'une des huit directions du personnage ; `Right` = +x, `Up` = +y, diagonales normalisées) et envoie une position explicite. Le moteur ne connaît que des coordonnées. Le `facing` est l'orientation du personnage, pas le curseur de visée libre (non exposé par `state`).

## Corrigé à la relecture

1. `refresh_live_stats` exigeait `Res<ResolvedBalance>` : **panique à chaque update tant que la ressource n'existe pas** (tous les scénarios auraient échoué ; vu par les trois tests au premier essai). Ressource optionnelle, événements lus d'abord, retour immédiat sans `Modified`.
2. `refresh_live_stats` remplaçait tout `Stats` par `base_stats(config, health_max, &[])` : un ennemi perdait ses défauts de création (`SeparationDistance`, `SeparationForce`, `SlowDownDistance`, `OptimalAttackDistance`, passés à `create_character` par `enemy::create`). Remplacé par `Stats::overlay` (réécrit les bases présentes, garde le reste).
3. Cycle d'ordre : `apply_game_commands_pickups.after(apply_game_commands_spawn)` allait contre l'ordre des sets (`Effects` précède `EnemySpawning`) : retiré (bevy signalait des centaines de cycles).
4. Ambiguïtés (`ambiguity_detection: Error`) avec les placements du runner (`GgrsNetIdFactory` des deux côtés) : le runner déclare ses placements de présentoirs (fin de la chaîne `Effects`) `.before(apply_game_commands_pickups)` et ses placements de personnages `.before(apply_game_commands_spawn)`. Ses net ids viennent donc avant ceux des commandes d'édition.
5. `apply_game_commands_pickups` n'était pas ordonné `.before(item_consumable_pickup_system)` (le runner le fait pour ses placements d'objets).
6. `start_native_recording` remet les réglages à zéro à la frame 0 : un spawn demandé juste après un `restart`, avant le premier tick, disparaissait du scénario sauvé (vu en jeu : `save` sans `characters`). Les commandes encore présentes dans `GameCommands` (vidée à `GameLoading`, donc propres à la run) sont reportées à la remise à zéro.
7. Docs : bullets `alacod/spawn_*` insérés au milieu de la phrase de `set_modifier` dans `editeur-vocal.md` ; §84 de `conventions.md` affirmait « aucun système dans `GgrsSchedule` » et avait perdu le détail des méthodes ; « devant moi » décrit une visée que `state` n'exposait pas.
8. Non corrigé, noté : le journal `editor_command` porte `rank` et non `net_id` (l'id est alloué dans `spawn_character`, l'entité n'existe qu'après l'application des `Commands`).

## Preuves

- `crates/scenario/tests/game_commands.rs` (3 tests, passés) :
  - `commande_spawn_cree_l_entite_a_la_frame_visee` : une « target » de plus une fois la frame 31 simulée, `CharacterConfigKey("target")`, net id égal à celui d'un placement de scénario équivalent (`[31, 32]`).
  - `session_enregistree_rejoue_la_meme_trace_en_synctest` : en synctest d=2 (défaut du runner, sans desync) ; `outcome.recorded.characters` contient le placement (`target`, `at_frame` 31) ; le rejeu par `scenario::run` donne la **même trace** ; la trace diffère de celle sans commande.
  - `max_speed_edite_change_les_stats_vivantes_sans_toucher_aux_modificateurs` (d=0 forcé par `GggrsSessionConfiguration`) : `Stats::MoveSpeed` = nouvelle valeur, `Modifiers` identique, vitesse résolue = base × 0,5 avant et après.
  - `rejoue_une_session_sauvee` (ignoré, diagnostic) : voir ci-dessous.
- Tests unitaires de `remote.rs` (passés) : `spawn_handlers_refuse_unknown_ids_without_command`, `spawn_handlers_refuse_synctest_distance_without_command`, `spawn_powerup_at_distance_zero_queues_for_next_frame` (commande à `frame + 1` = 42).
- `make test_crates` : sortie 0.
- Sous-ensemble sans BLESS (`SCENARIO=idle,four_players_shooting,clone_solo,throne_solo,gungeon_full_run,isaac_full_run,remote_first_fight,bench_horde`) : 8 scénarios joués, sortie 0, `git status` propre (aucune trace modifiée).
- Suite complète : `make test_scenarios` **sans filtre** (journal `/tmp/k5-2-scen.log`, 5790 s) : 809 lignes de scénario jouées, `test result: ok. 5 passed`, sortie 0, **aucune trace modifiée** (`git status` : seul le rapport est non suivi). Quatre avertissements de budget de vitesse (`bot_isaac_two` 38,8 fps, `isaac_devil_run` 20,5, `isaac_duo` 30,5, `isaac_quad` 14,4, plancher 40) : non bloquants hors `make bench` ; cause non mesurée (poste partagé, [INFERENCE]) ; ces scénarios ne jouent aucune commande, les systèmes ajoutés y sont inertes.
- `make schedule_order_diff BASE=k5-0-1-editeur` : les **types rollback sont identiques** (`*.rollback.txt`, 4 jeux). Seuls ajoutés, dans `GgrsSchedule`, par jeu : `apply_game_commands_pickups`, `apply_game_commands_spawn` et **quatre `apply_deferred`** (points de synchronisation des commandes de ces deux systèmes et des arêtes de la clôture). Le script sort pourtant avec des différences : la numérotation de tout le reste décale, y compris des systèmes non liés (`clear_frame_events<…>` dans un autre ordre) : le tri topologique de bevy ne suit pas l'ordre d'insertion (CLAUDE.md, « Preuve d'ordre »). Position des nouveaux : `apply_game_commands_pickups` juste après `apply_scenario_shelf_placements` (0118 → 0120), `apply_game_commands_spawn` juste après `apply_scenario_character_placements`. La preuve que ce décalage n'a pas d'effet observable est la suite complète (traces identiques). La première exécution du script a échoué à compiler (`overlay` introuvable) sur un `sim_core` obsolète du cache ; un `touch` de `stats.rs` a réglé le problème (artefact de cache, pas de code).
- `make check_sim_boundary` : OK, 13 crates, 2 en dette D69 ; `scripts/check-forbidden.sh` : OK (51 occurrences auditées) ; `scripts/check-rollback-registration.sh` : OK ; `cargo fmt --all -- --check` : 0 ; `make lint` : les cinq jeux sans erreur (gungeon : 1 avertissement existant sur le pool `boss`).
- **Preuve headless** (`make editor GAME=zombies HEADLESS=1`, 2026-10-23) :
  - `spawn nope --at 1,1` → `erreur alacod/spawn_character : {'code': -32602, 'message': 'personnage inconnu « nope »'}` ; `spawn-powerup nope` → `power-up inconnu « nope »`.
  - `spawn zombie_1 --at 100,100` → `{"frame": 179}` ; `brief` : `commandes en attente: 179:SpawnCharacter` ; après `step 5` : `ennemi 308 zombie_1 … pv 50.0 Chasing`, `state.commands` vide.
  - Vitesse du joueur (`input Right`, deux relevés de `state`) : **2,5 px/frame** (x −12 → 38 en 20 frames, `max_speed: "150.0"`) ; après édition de `player_config.ron` en `"300.0"` et 4 s d'attente (journal : `personnage « player » rechargé`), **4,87 px/frame** (x 38 → 135,4 en 20 frames, puis blocage contre un mur à x = 213,8) ; aucun `restart`. Fichier restauré par `git checkout --`.
  - Session sauvée, après correction 6 : `restart`, `spawn zombie_1 --ahead`, `step 60`, `save /tmp/k5-2-session.ron` → `characters: [CharacterPlacement(character: "zombie_1", x: "-12", y: "-56", at_frame: 1)]`. Le jeu en direct (d=0, une trace d'état n'y est pas écrite : `ALACOD_STATE_TRACE` demande d=2) donne à la frame 60 : joueur 292 (−75,4, −56,0), boss 297 (1640, −1032), `zombie_1` 311 (−44,0, −56,0). Le rejeu par le runner (`rejoue_une_session_sauvee`, deux rejeux, traces égales entre elles) donne à la frame 60 **les mêmes net ids et les mêmes positions** : joueur 292 (−75,4, −56,0), boss 297 (1640, −1032), `zombie_1` 311 (−44,0, −56,0). Méthode : comparaison de l'état final (identifiants et positions), faute de trace d'état commune entre le jeu local à d=0 et le runner à d=2.

## Non vérifié

- Le rechargement à chaud de `base_health.max` ou d'un `stats:` retiré du RON : `MaxHealth` vient de `ResolvedBalance` (recalculé au `restart`) et `overlay` ne retire pas une stat ; seul le champ `movement` (et les stats présentes dans le RON) suit en direct.
- Un `alacod/spawn_*` en synctest ou en ligne : refusés par la garde (testés en unitaire), pas joués en vrai.
- `--ahead` n'a été joué que vers la droite (`facing: Right`) ; les sept autres directions ne sont que la table du script.
- Les armes de mêlée rechargées (risque 7 du plan) : hors périmètre, non mesurées.
- Pas de preuve p2p : la tâche est locale d=0 par construction.
