# Rapport m2-t7-boss-arene — boss à phases jouable et arène (M2-T7, chantier D4)

Branche `m2-t7-boss-arene` (WIP d'une première tentative relu, `origin/main` fusionné au départ). Session
Claude Code cloud (conteneur sans Docker). Conventions : `docs/conventions.md` §45 (et §37, §39 mis à jour).

## 1. Fait

**WIP de la première tentative relu.** Gardé : contrats `BossDefeated` (devenu transitoire), actions
`SpawnEnemies` et `Telegraph` en fin d'enum `effects::Action`, `phase_shoot` / tir par phase, boss dormant,
`ArenaDef` / `ArenaClocks`, lint de frise. Corrigé ou complété : ordre des systèmes (`boss_reinforcements_system`
placé dans `Run` après `arena_waves_system`, `room_dormancy_system` avant `boss_phase_system`), les trois
systèmes qui manquaient dans `map_ldtk::game::rooms` (`boss_reinforcements_system`, `arena_waves_system`,
`boss_room_clear_system`, `SpawnParams`), l'exhaustivité des `match` sur `Action`, la dépendance `behaviors` de
`map_ldtk`, `lint_arenas`.

1. **Tir par phase** (§45) : la règle `Shoot` d'une phase porte son arme, son pattern, sa portée, sa recharge ;
   `spawn_enemy` équipe en plus les armes des `Shoot` de phase. Sans règle de phase : `ai.ranged` comme avant.
2. **Salle de boss** : le boss dort tant que sa salle est `Dormant` (`boss_phase_system` saute `RoomDormant`) ;
   à sa mort `FrameEvents<BossDefeated>` (transitoire) puis `boss_room_clear_system` tue les autres ennemis de la
   salle : `Cleared` dans la frame, portes rouvertes, victoire/portail ensuite comme avant.
3. **Arène à tenir** : `arena: Some((clock, waves))` dans `rooms/<id>.ron` ; verrouille dès l'entrée même vide,
   vagues datées sur l'horloge `ArenaClocks` (ressource rollback neutre), survivants tués et salle rouverte à la fin
   de l'horloge si un joueur vit.
4. **Actions de frise** : `SpawnEnemies(id, count)` (positions seedées, flux `room:<niveau>`, case libre
   `Ground`) et `Telegraph(n)` (devant le prochain `SpawnPattern`, aucun type nouveau).
5. **Lint** : `lint_arenas` et règles de frise ; 7 fixtures (`arena_no_waves`, `arena_clock_zero`,
   `arena_unknown_enemy`, `arena_count_zero`, `boss_spawn_enemies_unknown`, `boss_spawn_enemies_count_zero`,
   `boss_telegraph_alone`) dans un test `arena_and_boss_t7_fixtures_report_their_rule`.
6. **Contenu `gungeon`** : `gatling` à trois phases (éventail ; spirale sous 50 % ; rage sous 10 % avec un
   `bullet_kin` en renfort, `Telegraph` + couronne), 120 PV ; type de salle `arene`, gabarits `ArenaEW` /
   `ArenaCroix` dans `maps/salles.ldtk`, `etage` exige une arène (`required`). Validé sur 1 000 graines
   (`ALACOD_FLOOR_SEEDS=1000`, nouveau réglage du test `gungeon_etages_de_grammaire`, qui vérifie en plus
   « exactement une arène »). `testbed` : `gardien_tir`, `gardien_renforts`, salles `boss` / `arene` /
   `arene_mortelle`, cinq cartes dérivées de `salles.ldtk` (script `m2-t7-boss-arene.make_cartes.py`).
7. **Scénarios** (6 nouveaux, traces créées) : `boss_room_lock`, `boss_phase_shoot`, `boss_reinforcements`,
   `arena_survive`, `arena_fail`, `gungeon_boss_bot` (le bot `fonceur`, sans aucun input scripté, bat le boss à f403).

## 2. Choix

- **Le boss dort** : la phase 0 n'est plus datée au premier tick du jeu mais à l'éveil de la salle. Avant, le
  `gatling` posait son émetteur à f10 dans sa salle fermée (corrigé, voir §4 « traces »).
- **`BossDefeated` et `ReinforcementRequest` sont transitoires** (`add_frame_events_transient`, hors checksum) :
  émis et lus dans la même frame par des lecteurs ordonnés après l'émetteur. `boss_two_phases` (le boss meurt)
  ne bouge donc pas. `ArenaClocks` est la seule ressource rollback nouvelle, en variante neutre.
- **Renforts dans `Run`** (pas `EnemySpawning`) : ils partagent `GgrsNetIdFactory` avec les placements de
  personnages du runner ; l'ordre explicite est `arena_waves_system` puis `boss_reinforcements_system`.
- **Boss de `gungeon` : 120 PV, rage sous 10 %** (écart à « renforts en phase 2 ») : un renfort plus tôt bloque des
  balles et retarde la mort du boss de plus de 40 frames ; les six scénarios scriptés (M2-T0d, T1, T3, T10) ont des
  joueurs qui survivent à quelques points de vie près (le joueur de `gungeon_boss` finit le combat de l'ancienne
  version à ~5 PV). Plutôt que de réécrire leurs entrées, la rage tombe à la dernière touche. Seules leurs attentes
  de phase / NetId et leurs commentaires changent. Le bot, lui, bat le boss avec la même logique.
- **Cartes testbed dérivées par script** (`m2-t7-boss-arene.make_cartes.py`, `make_arene.py`) depuis les gabarits
  existants : le dépôt n'a pas d'éditeur LDtk.
- `arena_fail` utilise le `breacher` (mêlée) : le `follower` n'attaque pas (`attack_range` 0).

## 3. Vérifié (commandes lancées ici, résultats réels)

- `make test_scenarios` complet **sans BLESS** : exit 0, 243 scénarios joués (152 `.ron` + 91 générés), 3 tests
  réussis dont `recording_replays_identically`, 2 773 s. 33 avertissements de budget de fps (machine cloud, non comparés à
  `main` ici : à relire par l'orchestrateur sur `orca` avec `make bench`).
- `make test_crates` : exit 0, 654 réussis, 0 échec, 10 ignorés (ignorés d'origine : diagnostics `#[ignore]`) ; `lint_fixtures` inclut le nouveau test des sept fixtures.
- `make lint` : « aucune erreur » sur zombies (5 personnages), testbed (25), throne (14) et gungeon (4).
- `cargo fmt --all -- --check` propre. `./scripts/check-forbidden.sh` : 6 occurrences, **identique à `origin/main`**
  (mesuré sur un worktree). `./scripts/check-rollback-registration.sh` : OK.
- `make gen GAME=zombies|testbed|throne|gungeon` : exit 0, `git status` propre ensuite (aucun scénario généré ne
  bouge).
- Grammaire : `ALACOD_FLOOR_SEEDS=1000 cargo test -p map_ldtk gungeon_etages_de_grammaire` vert (`etage`, `grand`
  sur 1 000 graines, `court` sur 400 ; exactement une arène dans `etage`).
- Scénarios nouveaux, chacun vu passer ses attentes avant bless : `boss_room_lock` (verrou f85, aucune flèche avant,
  phase 1 f142, boss mort et salle `Cleared` f184), `boss_phase_shoot` (3 flèches `arrow` à f125, 0 `ricochet` ;
  phase 1 f208 ; 1 `ricochet` et 0 `arrow` à f245), `boss_reinforcements` (1 → 3 ennemis à f160 → 0 à f200),
  `arena_survive` (verrou f85, rouverte f685), `arena_fail` (défaite f1514), `gungeon_boss_bot` (`fonceur` : combat
  nettoyé f181, boss verrouillé f218, phases f322 et f382, victoire f403).

### Traces bougées et preuve §5

Une première passe complète du code de la branche a montré **10 traces qui bougent, toutes de `gungeon`** :
`bench_floor`, `dash_charges`, `gungeon_boss`, `gungeon_clear`, `gungeon_floor_seed_{1,23,123}`,
`gungeon_floor_start`, `gungeon_start`, `gungeon_two_floors`. Toutes les autres (dont `boss_two_phases`,
`profile_run_end`, les testbed, throne, zombies, les générés) sont identiques sans BLESS.

Preuve (dumps `ALACOD_DUMP_TRACE`, `scripts/trace-diff.py … --ignore BossState,ArenaClocks`, un scénario par
appel, binaires de test de `origin/main` et de la branche, **contenu gungeon de main** des deux côtés : `gatling.ron`
et `gungeon_boss.ron` de main remis pour les dumps de la branche) sur les **10 scénarios** : la première frame qui
diffère est la frame 10, et la seule différence de cette frame est la ligne du `gatling` (NetId 31, 40, 147…) :
sur main il porte un `Emitter` (`pattern: "eventail"`, `started_at: 10`) et `BossState { phase: 0, entered: 0,
started: true }` ; sur la branche ni émetteur ni `started`. C'est le boss qui dort : sa salle est `Dormant`, il ne tire
plus depuis une salle fermée. `ArenaClocks` (vide) est la seule autre différence de dump (nouveau type neutre,
ignoré). Ensuite les trajectoires divergent (les balles du boss ne naissent plus) : la preuve porte sur la cause
de la première divergence, pas sur une égalité des 600+ frames suivantes.
Puis bless en un commit (message : cause, preuve) avec le nouveau contenu `gatling` (3 phases, 120 PV) et les
attentes ajustées des scénarios gungeon. L'ajout des gabarits d'arène à `salles.ldtk` a ensuite été vérifié neutre :
les scénarios `gungeon_floor_*`, `gungeon_two_floors` rejoués sans bless restent verts (traces identiques).

## 4. Non fait / incertain / non vérifié ici

- **p2p à deux clients : non vérifié** (pas de démon Docker dans le conteneur : `docker info` échoue). Aucun
  changement d'input ni de session ; types rollback neutres ou transitoires. L'orchestrateur doit rejouer
  `make test_multiplayer` / la boucle du README §4. Les tests de rollback en synctest
  (`recording_replays_identically`, `determinism`, `fuzz_inputs` non lancé, voir plus bas) sont verts.
- `fuzz_inputs` (`ALACOD_FUZZ`) **non lancé** : l'input n'a pas changé.
- Bench au calme (`ALACOD_BENCH_STRICT=1`) non lancé (machine partagée / cloud).
- **L'arène de `gungeon` n'est jouée par aucun scénario** (grammaire et lint seulement, 1 000 graines) : les
  scénarios d'arène vivent dans le testbed. Un parcours complet d'un `etage` avec arène demande le bot explorateur
  (M2-T16).
- Vidéos / pastilles non regardées (`make videos` : pas de rendu dans le conteneur) ; les en-têtes
  `// À regarder` des scénarios nouveaux décrivent les frames mesurées.
- Moment clé d'événements pour `arena_wave` / `boss_defeated` : traces `ggrs{…}` seulement, pas de pastille.

## 5. Dettes

- Sélection de position des renforts : 16 essais puis centre de salle ; un boss entouré de murs fins pourrait
  placer un renfort près d'un mur (le profil `Ground` passe, pas la taille de l'ennemi). Aucun cas dans le contenu.
- `bullet_kin` de renfort : bloque les balles du joueur scripté (voir choix) ; un renfort volant ou une apparition
  hors de la ligne de tir serait plus lisible.
- Le HUD du boss, le butin et le ralenti de fin de boss restent à M2-T18 / T5 / T19 (hors périmètre).
- Le moteur de `docs/taches.md` §10 (journal) n'est pas mis à jour : c'est l'orchestrateur, sur `main`, après merge.
