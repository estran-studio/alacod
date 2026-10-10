# Rapport m2-t0d-squelette-gungeon — squelette du clone `gungeon` (M2-T0d)

Branche `m2-t0d-squelette-gungeon` (base `93f30e9` + fusions de `origin/m2-t0b-contrats-objets`,
`origin/m2-t0c-contrats-boss-profil` puis `origin/main` 801e06a), b1. Conventions :
`docs/conventions.md` §39.

## 1. Intégration (commits de fusion séparés)

- `12070d8` : fusion de T0b (sans conflit).
- `dc0e2bd` : fusion de T0c. Conflits **mécaniques** résolus (liste complète dans le message du
  commit) : champs/littéraux `Scenario` (`items` de T0b + `profile` de T0c) dans `replay.rs`,
  `recording.rs` (5 endroits), `generate.rs` (2), `alacod-sim.rs`, 4 tests de `scenario` ;
  `at_frame()` des attentes ; `game/Cargo.toml` et `core.rs` (plugins) ; `events.rs` (trois moments
  clés, `boss_events` réécrite) ; `runner.rs` (arms des attentes) ; `lint_fixtures.rs` ;
  `CLAUDE.md` et `docs/conventions.md` (sommaire, §36 puis §37-§38). Adaptation à la refonte
  `ContentFiles` de main : `load_items` et `load_rooms` prennent `&ContentFiles`.
- `15acefe` : fusion de `origin/main` (correctif de `load_rooms` identique côté main).

## 2. Fait

- `games/gungeon/` : binaire `gungeon` (copie de `throne`), `assets/game.ron` (mode `Floors`),
  `pistolero` (joueur, roulade à i-frames), `pistolet` (réserve 9999 chargeurs, `Automatic`),
  `bullet_kin` (tireur `Shoot` `visee`), `gatling` (boss à 2 phases), `cible`, objets `bottes`
  (passif), `fiole` (`Active(Rooms(2))`), `key`, `blank`, types de salles `depart`/`combat`/`boss`,
  étage `maps/etage_1.ldtk` (script `m2-t0d-squelette-gungeon.make_etage.py`), `floors/etage_1.ron`.
- Ajout moteur non prévu par la fiche : `victory_at_end` (séquences `Floors`, faux par défaut) : le
  dernier niveau vidé termine la partie par une **victoire** (`FloorsEntry`, `FloorPlan`,
  `floor_portal_open_system`, trace `floors_victory`). Nécessaire pour « victoire au boss tué » ;
  aucun contenu existant ne l'active.
- `make gungeon`, `make lint` couvre `games/gungeon`, `make gen GAME=gungeon` (4 scénarios
  générés : `weapon_arsenal`, `weapon_pistolet`, `weapon_bare_hands`, `enemy_bullet_kin_still`,
  `enemy_bullet_kin_moving` ; le boss n'a pas de `test:`), `content/tests/embedded.rs` compare
  natif et embarqué pour `gungeon`.
- Scénarios : `gungeon_start`, `gungeon_clear` (verrouillage f75, nettoyage f231, bottes par
  Interaction, clé au contact), `gungeon_boss` (salle du boss verrouillée f291, phase 1 f448, boss
  mort f568, victoire f569).

## 3. Vérifié

- `make test_scenarios` complet **sans BLESS** : vert (EXIT 0) ; aucune trace existante modifiée
  (seules `gungeon_start`, `gungeon_clear`, `gungeon_boss` et les générées sont nouvelles).
  `gungeon_start` a été rebénie une fois (changement de l'arme `pistolet` en `Automatic` après sa
  première bénédiction), avant la suite complète.
- `cargo test` run, combat, game, content, map_ldtk, sim_core, stats, bots, effects, world,
  behaviors, meta, items : 538 réussis, 0 échec, 1 ignoré. `cargo test` scenario (lib + 12
  binaires de test hors `scenarios`) : 80 réussis, 0 échec.
- `make lint` : « aucune erreur » sur les quatre jeux ; `cargo fmt --all -- --check` propre ;
  `check-rollback-registration` OK ; `check-forbidden` : 4 avertissements préexistants.
- `make gen GAME=gungeon` : bénédiction puis rejeu sans bless, traces identiques.

## 4. Non fait / incertain

- p2p à deux clients sur `gungeon` : non lancé (port Docker réservé par l'orchestrateur ;
  à rejouer avec T0b/T0c/E1 : l'input `u32`, trois types rollback neutres et des événements neutres
  sont nouveaux).
- Le jeu n'est pas dans les scripts de build web (`build-web-games.mjs`, Pages/R2 : `zombies` et
  `throne` seulement) : décision de William.
- `fuzz_inputs` non relancé sur `gungeon` (T0b l'a fait sur l'input élargi).
- Les bots (`chasseur`, `fonceur`) n'avancent pas vers des ennemis dormants d'une autre salle :
  les scénarios de `gungeon` sont scriptés sur des positions mesurées. Les ennemis `bullet_kin` se
  cornent à l'est de leur salle par `KeepDistance`.
- Munitions infinies : approximées par 9999 chargeurs (le moteur n'a pas ce contrat).

## 5. Dettes / questions

- Le boss `gatling` est « vide » (une visée, une couronne) : le contenu vient avec M2-T7.
- La table de tir d'une règle `Shoot` n'est pas par phase de boss (voir §37) ; la frise
  `SpawnPattern` la contourne.
- Disque : 60 Go de target avant purge ; générations périmées à retirer après chaque suite.
