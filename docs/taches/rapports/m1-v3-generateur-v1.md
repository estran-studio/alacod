# Rapport m1-v3-generateur-v1 — générateur v1 et placement scripté de personnage (T1.13, V3)

**Branche** `m1-v3-generateur-v1`, partie de la tête livrée de T1.9 `e0c9fd6` (même worktree,
même target ; contient donc T1.9, mergée localement par orch mais pas encore poussée). Fiche :
`docs/taches/m1-v3-generateur-v1.md`.`origin/main` `e26a3e3` déjà contenu (rien à merger à la livraison) : base `e26a3e3`.

## État en cours

- **Fait** : tout le périmètre de la fiche et l'ajout `generate_template` (§1), vérifié (§2).
  Livré.
- **Target** purgé après la suite complète (15 Go, /home 76 Go libres).
- **Reste à l'orchestrateur** : bless des 18 traces générées (et des 3 de T1.9 si la branche
  est vérifiée avant le push de T1.9), décision sur le défaut `kiter` (§4), bench, p2p, merge.

## 1. Fait

- **Placement scripté** : `Scenario.characters: Vec<CharacterPlacement { character, x, y,
  at_frame, variant, team }>` (`game::replay`), conservé au réenregistrement
  (`RecordedSettings::characters`). Système `scenario::runner::apply_scenario_character_placements`
  dans `GgrsSchedule`, `RollbackSystemSet::EnemySpawning`, ordonné après `wave_spawning_system` et
  `enemy_spawn_from_spawners_system` (Bevy refusait l'ambiguïté sur `GgrsNetIdFactory`), avec une
  condition d'exécution : sans placement, il ne tourne pas.
  - **Chemin commun** (amendement 1) : la création d'un personnage de `spawn_level_characters`
    est extraite dans `map_ldtk::game::local::spawn_character` (équipe, santé F5 × difficulté,
    graine de variante, variante imposée, `spawn_enemy`), appelée par la carte **et** par le
    placement. Comportement de la carte inchangé (même ordre, mêmes appels).
  - **`EntityRef::Placed(n)`** (ajout) : désigne le personnage du placement `n`, via un composant
    d'observation `ScriptedPlacement(n)` hors rollback (ni checksum ni trace), reposé quand un
    rollback rejoue le placement. Les attentes d'un `test:` visent ainsi l'ennemi sans connaître
    son net id (plutôt qu'une sonde de net id, comme `discover_target_net_id`).
  - **Validation** (décision 5, côté runner) : `at_frame >= frames`, personnage inconnu,
    variante inconnue de la table du personnage → échec du scénario avec un message (le
    placement refusé n'est pas appliqué ; `PlacementErrors`, hors rollback).
- **`test:` d'un personnage** : `game::character::config::CharacterTest { frames, still, moving,
  expect_still, expect_moving }` (`still`/`moving` vrais par défaut) ; miroir
  `content::registry::CharacterTestEntry` (attentes comptées). Aucun effet en jeu.
- **Gabarits** (`scenario::generate`) : `Template::EnemyVsStillPlayer` (le joueur, armes de
  départ, ne fait rien) et `Template::EnemyVsMovingPlayer` (carré de 90 frames par côté, en
  boucle, sans tirer). Ennemi placé à +200 en x du **spawn du joueur**, à la frame 1 ; spawn lu par
  une sonde d'une frame (`discover_enemy_probe`, amendement 2). Fichiers
  `tests/scenarios/generated/<jeu>/enemy_<id>_{still,moving}.ron`, en-tête commenté. `alacod-gen`
  affiche désormais le nom de fichier dans la colonne « sujet » et compte les gabarits de
  personnage.
  - **Attentes par défaut — écart à la fiche** : immobile → moment clé `Event(kind: "hit",
    "joueur 0 touché")` avant la dernière frame (au lieu de `Health { max < santé de départ }` à
    la dernière frame) + `EnemyNeverInWall(Placed(0))`. Constaté sur `charger` : la charge touche
    (santé 85 à f145) mais **le joueur se régénère** et finit à 100 ; une santé finale ne prouve
    rien. Mobile → `PlayerAlive` à la dernière frame + `EnemyNeverInWall(Placed(0))`, comme prévu.
  - **Où** (amendement 3) : le jeu énuméré lui-même — `testbed/arena.ldtk` pour le testbed,
    `exemples/test_map.ldtk` pour `zombies` (mode `Waves` : `grace_period_frames = frames + 1`,
    vérifié : `ennemis_max = 1`, aucune vague ajoutée), sinon `entry.start_map`.
- **`generate_template`** (ajout d'orch en cours de tâche, pour `games/throne`) :
  `GameManifest::generate_template: Option<(map, target)>`. Présent : gabarits d'armes et
  d'ennemis dans ce jeu et cette carte (`WeaponArena`, `build_weapon_scenario`) ; absent : testbed
  comme avant (`build_scenario` garde sa signature). Lint `lint_generate_template` : carte connue,
  cible connue avec `counts_hits: true` (`CharacterEntry::counts_hits` ajouté). Limite écrite au
  §28 : les inputs du gabarit d'arme sont réglés sur l'arène du testbed, la carte place la cible au
  même décalage du spawn (+128, −48 LDtk).
- **Lint** `lint_character_tests` : `frames > 0`, au moins un gabarit, `expect_*` seulement d'un
  gabarit actif. Fixtures `character_test_frames_zero`, `character_test_no_template`,
  `character_test_expect_inactive`, `generate_template_unknown_map`,
  `generate_template_target_without_hits`.
- **Contenu** : `test: Some((frames: 600))` sur `grunt`, `archer`, `charger`, `kiter`, `turret`,
  `breacher` (testbed) et sur les ennemis de vague de `zombies` : `zombie_1`
  (`zombie_config.ron`), `zombie_2` (`zombie_hard_config.ron`), `zombie_full`
  (`zombie_full_config.ron`) ; `zombie_scaled` (preuve F5, hors vagues) sans `test:`.
  Attentes explicites (besoin avéré) :
  - `grunt` immobile : le grunt **n'attaque pas** (`attack_range` 0, `Chase` seul) ; il rattrape
    le joueur (distance 30 dès f300) → `EnemyDistance(Placed(0), Player(0), max 40, f600)` +
    `EnemyNeverInWall`.
  - `kiter` immobile : `EnemyNeverInWall` borné à f540 — **à f544, en reculant (`KeepDistance`),
    le collider du kiter chevauche un mur de `arena.ldtk`**. Défaut de navigation révélé par le
    gabarit, non corrigé ici (hors périmètre ; à décider : dette ou correctif T1.4).
- **Scénarios générés (18, traces à bénir par orch)** :
  - testbed : `enemy_archer_still`, `enemy_archer_moving`, `enemy_breacher_still`,
    `enemy_breacher_moving`, `enemy_charger_still`, `enemy_charger_moving`, `enemy_grunt_still`,
    `enemy_grunt_moving`, `enemy_kiter_still`, `enemy_kiter_moving`, `enemy_turret_still`,
    `enemy_turret_moving` ;
  - zombies : `enemy_zombie_1_still`, `enemy_zombie_1_moving`, `enemy_zombie_2_still`,
    `enemy_zombie_2_moving`, `enemy_zombie_full_still`, `enemy_zombie_full_moving`.
  Tous verts à `make gen` (attentes), aucun scénario d'arme généré modifié.
- **Tests** (`crates/scenario/tests/placement.rs`) : placement à la frame exacte (absent avant,
  présent après) et ordre des net ids = ordre de déclaration ; `EntityRef::Placed` et variante
  imposée (`blinde`), attente avant la frame en échec ; deux gabarits depuis un `CharacterTest`
  (attentes par défaut, explicites, période de grâce `Waves`).
- `docs/conventions.md` : §28 et une ligne au tableau du §3 ; `CLAUDE.md` : réglage `characters`.

## 2. Vérifié (résultats réels)

Sur la branche (base `e26a3e3`, contient T1.9) :

- **Scénarios** (via la suite des crates) : 127 joués, **0 « trace différente »**, **0 attente en
  échec** ; seuls sans trace : les 18 générés de T1.13 et les 3 de T1.9 (`clock_events`,
  `clock_floor_reset`, `difficulty_scales`). Aucune trace bénie par moi. Les scénarios existants
  n'ont pas de placement : le système ne tourne pas, leurs traces sont identiques.
- **Tests des crates** (`scenario run combat game content map_ldtk map sim_core stats bots
  effects utils behaviors world`, `--include-ignored`) : 488 tests verts hors `scenarios`
  (ci-dessus), dont `lint_fixtures` 70 (5 nouvelles) et `placement` (3 tests à ce moment). Seul
  autre échec : le doctest `rust,ignore` de `game::waves`, préexistant (identique dans main).
  Après la suite, ajout de la validation des placements : `placement` relancé, **4/4 verts**
  (`placements_invalides_en_echec`), `make gen` des deux jeux rejoué après ce changement.
- **`make lint`** : `games/zombies` et `games/testbed` sans erreur.
- **`make gen`** : zombies 16/16 et testbed 32/32 attentes `ok` (armes `ok`/`ok` ; gabarits de
  personnage `ok`/trace `absente`, en attente de bless) ; **aucun fichier généré existant
  modifié** (seuls les 18 nouveaux).
- **`fmt`** propre ; **`check_forbidden`** 4 occurrences (identique à main) ;
  **`check_rollback_registration`** OK ; **exemples racine** compilés.

## 3. Écarts, non fait / incertain

- Attente par défaut du gabarit immobile : `Event hit` au lieu de `Health` (§1, régénération).
- `EntityRef::Placed` ajouté (non prévu par la fiche) pour viser l'ennemi testé.
- Défaut `kiter` dans un mur à f544 : contourné par une borne, pas corrigé.
- Bench strict et p2p : non faits (orch).

## 4. Dettes, questions ouvertes

- `kiter` : `KeepDistance` recule dans un mur (`arena.ldtk`, f544 du gabarit immobile).
- `grunt` testbed sans attaque : voulu par T1.4/T1.5 (personnage de navigation), mais un
  « ennemi » qui ne blesse jamais mérite peut-être une `Melee`.
- Gabarits par statut : quand T1.3 sera dans main.
