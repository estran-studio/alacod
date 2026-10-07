SHA de tête vérifiée : voir la ligne `LIVRÉ` (rapport **provisoire**, session interrompue le
2026-10-05 pour reprendre sur l'autre Mac de William ; reprise : `docs/taches/REPRISE-revue-m0.md`).
Fiche : brief de revue humaine M0 (T3.3, `docs/taches/T3.3-revue-humaine.md`), MacBook de William.
Branches : `fix-revue-m0` (correctifs de la revue, base `main` `1a39283`) et `revue-m0-suite`
(`movement-feel` + `fix-revue-m0`, base `26f1496`) — c'est `revue-m0-suite` qui se reprend.
Date : 2026-10-04/05 ; agent : Claude Code (Opus 5.5).

## Fait

- **Mise en route macOS** (notes R1–R3, `docs/digests/revue-m0.md`) : `cargo build -p zombies
  --profile headless` compile sur macOS arm64 (1 min 18 s, dépendances en cache) ;
  `check-forbidden.sh` et `check-rollback-registration.sh` passent sous bash 3.2.57 (D33 bien
  fermée) ; `make record_session` compile en `dev` (R2) ; `ALACOD_HEADLESS=1` sur un binaire
  avec `render` panique sans message utile (R3).
- **R4 — murs traversés après « Rejouer »** (`crates/game/src/run_state.rs`) :
  `CollisionGrids` remise à zéro dans `cleanup_rollback_world_system`. Cause : la grille des
  murs n'est reconstruite que si la signature (nombre, somme des `GgrsNetId`) change ; la
  relance recrée les mêmes murs avec les mêmes ids. **`main` a fait le même correctif en
  parallèle** (`f9f0d8e`, D14) : à la fusion, garder la version de `main`. Ce qui reste propre
  à cette branche : le test `restart_keeps_walls_solid` (`crates/scenario/tests/run.rs`) et le
  scénario `revue_murs_avant_poste` (carte `avant_poste`, graine 123456 du jeu, joueur contre
  chaque mur).
- **R5 — la touche R (rechargement) relançait la partie** (`crates/game/src/ui/game_over.rs`) :
  `button_system` tourne pendant tout `InGame` ; R ne relance plus que si `GameOverUiRoot`
  existe. Test `r_ne_relance_que_sur_l_ecran_de_fin`. **Pas corrigé sur `main`** (`ddb9789`).
- **Carnet de revue** `docs/digests/revue-m0.md` : R1–R9, S1–S2.
- **`movement-feel`** (autre session, voir `docs/taches/rapports/movement-feel.md`) commité avec
  le feu vert de William : `6156d22` (code) et `6d3fd8f` (bless de 182 traces + 2 neuves),
  puis fusionné avec `fix-revue-m0` dans `revue-m0-suite` ; `revue_murs_avant_poste` rebénie
  sur l'état fusionné (`8ac2c0b`, même cause que les 182 : frame 0, `DashState`/`Acceleration` ;
  les 4 attentes de position passent sans retouche).

## Vérifié (chiffres réels)

Sur `fix-revue-m0` (`4593fef`, Mac) :
- `make test_scenarios` : `test result: ok`, 129 scénarios, aucune trace modifiée, 442,6 s.
- Avant correctif, `restart_keeps_walls_solid` échoue : joueur en (282.0, 763.97) à f400 dans
  la première partie, (−200.83, 1080.49) après relance. Après : 4/4 tests de `run.rs` verts.
- `r_ne_relance_que_sur_l_ecran_de_fin` : échoue sans le correctif (`Some(Restart)` au lieu de
  `None`), passe avec.
- `cargo fmt --all -- --check` : propre.

Sur `revue-m0-suite` (`8ac2c0b`) :
- `revue_murs_avant_poste` : attentes vertes avant et après bless ; `run.rs` 4/4 ;
  `r_ne_relance_que_sur_l_ecran_de_fin` vert.
- Suite complète (`make test_scenarios`, Mac, sur `8ac2c0b`) : `test result: ok`, **185
  scénarios** joués (184 de movement-feel + `revue_murs_avant_poste`), aucune trace différente,
  543,3 s.

## Non fait / non vérifié

- Tests des crates (README §4, deuxième commande), `make lint`, `make gen`, `make check` :
  **non lancés** dans cette session.
- p2p à deux clients : non fait (prévu ce soir avec William, puis interrompu).
- Partie à deux, réanimation, fin de partie en ligne : non joués.
- R7 (enregistrement non écrit) : cause non établie.
- Fusion de `origin/main` (`ddb9789`, 163 commits de plus que `26f1496`) : **non faite**.

## Dettes, questions ouvertes

Voir `docs/digests/revue-m0.md` : R2, R3, R6 (touches), R7, R8 (graine fixe : décision de
William), R9 (LCG, bits faibles), S1/S2 (sensations à préciser).

## Fusion dans main (m1-fusion-revue-m0-suite, b0, 2026-10-07)

Branche `m1-fusion-revue-m0-suite` créée depuis `origin/revue-m0-suite` (`91d0f4e`), merge
d'`origin/main` (`836d3cf` au moment des mesures). Rien de béni par l'agent : l'orchestrateur bénit à la
vérification à partir de la liste ci-dessous.

### Conflits et résolutions (132)

- **113 traces** : version de `main` à la fusion, puis toutes rejouées sur l'état fusionné (liste
  plus bas) ; **6 scénarios générés supprimés par D26** sur `main` (`weapon_{axe,bare_hands,club,knife,
  sword,zombie_claws}` du testbed) : supprimés, avec leurs traces.
- `crates/bots/src/decide.rs` : **version de `main`**. movement-feel n'y changeait que des commentaires
  sur l'ancien freinage du portail (« la course actuelle s'arrête en 3 frames : ce freinage reste une
  garde ») ; `main` a remplacé ce freinage par le pilotage en vitesse (m1-v3-bots-portail).
- `crates/game/src/run_state.rs` : **version de `main`** (R4, reset de `CollisionGrids`, déjà sur `main`
  par `f9f0d8e`, et restart p2p D14). Le test R4 `restart_keeps_walls_solid`
  (`crates/scenario/tests/run.rs`), le scénario `revue_murs_avant_poste` et **R5**
  (`crates/game/src/ui/game_over.rs`, test `r_ne_relance_que_sur_l_ecran_de_fin`) étaient hors conflit :
  gardés.
- `docs/conventions.md` : la section « Course et esquive (dash) » de la branche (son §30) devient le
  **§34**, ajoutée au sommaire ; ses renvois « §30 » visant la course corrigés en §34 (conventions
  l. 551 et l. 1779, `dash_iframes.ron`, `dash_stun_buffer.ron`, `docs/plan-engine.md` l. 47,
  `docs/taches/REPRISE-revue-m0.md`).
- `docs/taches/dettes.md` : la **D41 de la branche** (tir ami `Always` qui touche son tireur) devient
  **D54** (D41 est prise sur `main`) ; le rapport historique `movement-feel.md` garde « D41 ».
- **9 scénarios `.ron` en conflit** : version de `main`, puis recalés sur l'état fusionné (ci-dessous).

### Scénarios recalés (attentes remesurées une par une)

- `clone_solo` : **inputs de `revue-m0-suite`** (marche recalée et visée ré-enregistrée par movement-feel),
  attentes de `main` tenues telles quelles (vague 5 à f3017, 13 kills, joueur debout). Avec les inputs de
  `main` et la seule marche recalée, la vague 5 n'était pas atteinte (10 kills).
- `clone_duo` : inputs de `main`, marche recalée comme movement-feel (`Left` f460–484 et f530–539) :
  toutes les attentes tenues (vague 5 à f4474, deux debout).
- `clone_quad` : inputs du joueur 3 de `revue-m0-suite` ; **attente M0 tenue** (vague 5 à f4602,
  13 kills, quatre debout) ; soldes des trois bots remesurés 1310/1660/1350 → **1460/1770/1040**
  (joueur 3 inchangé, 3480).
- Scénarios à bots (frames remesurées depuis les moments clés du rejeu ; objectifs tenus) :
  `throne_floor_1` (étage 1 à f511), `throne_progression` (niveau 1 f314, `sang_froid` f914, niveau 2
  f1010, alerte f1411, étage 2 f1519), `throne_solo` (mutations `sang_froid` + `coriace`, étage 2
  f1519, vivant f3699), `throne_quad` (étages f440/f1062/f1930, quatrième caverne f3190, les quatre
  vivants), `throne_three_floors` (étages f505/f1233/f3537, **personne à terre**, les deux vivants à
  f4999 : la régression de m1-v3-bots-armes ne se reproduit pas), `throne_softlock_recul` (étage 2 à
  f1845, 1320 → 1860 frames), `bot_floors_three` (étage 3 à f729, 660 → 800 frames),
  `bot_prudent_nododge` (inputs figés rejoués avec la nouvelle course : santé 9 → 59, toujours sous les
  100 du jumeau avec esquive).
- **Observation** : les portails s'ouvrent aux mêmes frames, mais les bots y entrent **120 à 190 frames
  plus tard** (f48 → f169 dans `bot_floors_three`) : l'approche pilotée du portail
  (`PORTAL_CRUISE_SPEED` 120) a été réglée pour l'ancienne course.

### Traces (suite sans bless sur l'état fusionné)

186 scénarios, **0 attente en échec** ; **117 traces diffèrent**, 69 inchangées.

- **115 dès la ligne 1 (frame 0)** : `ammo_burst`, `ammo_shared_reserve`, `avant_poste_demo`, `bench_bullets`, `bench_cave`, `bot_floors_three`, `bot_prudent_dodge`, `bot_prudent_nododge`, `bots_four_mixed`, `bots_two_fonceurs`, `bullets_walls`, `clock_floor_reset`, `clone_duo`, `clone_quad`, `clone_solo`, `dash_aim_change`, `difficulty_scales`, `downed_bleedout`, `downed_revive`, `effect_none`, `effect_on_damage_taken`, `effect_on_kill`, `enemy_flee`, `enemy_ring`, `enemy_ring_quad`, `enemy_telegraph`, `equilibrage_joueurs_duo`, `explode_wall`, `four_players_shooting`, `friendly_fire_cursed`, `friendly_fire_never`, `immune_tag`, `levelup_choice`, `levelup_timeout`, `points_on_kill`, `portal_next_floor`, `powerup_double_points`, `powerup_drop_on_kill`, `powerup_max_ammo`, `remote_first_fight`, `revue_murs_avant_poste`, `shoot_around`, `status_freeze_enemy`, `status_slow_enemy`, `test_map_fenetre_graine_100`, `testbed_ally_safe`, `testbed_civilian_blocks`, `testbed_dummy_shoot`, `testbed_target_hits`, `throne_ammo_pickup`, `throne_duo_defaite`, `throne_floor_1`, `throne_mutation_choice`, `throne_progression`, `throne_quad`, `throne_softlock_recul`, `throne_solo`, `throne_three_floors`, `two_players_shooting`, `weapon_pool_drop`, `weapons_workout`, `weapon_fireball_gun`, `weapon_foreuse`, `weapon_grenade`, `weapon_grenade_creuse`, `weapon_machine_gun`, `weapon_pistol`, `weapon_proj_bounce`, `weapon_proj_gravity`, `weapon_proj_homing`, `weapon_proj_lifetime`, `weapon_proj_pierce`, `weapon_proj_size`, `weapon_rifle`, `weapon_status_burn`, `weapon_status_freeze`, `weapon_status_slow`, `weapon_status_stun`, `enemy_arroseur_moving`, `enemy_arroseur_still`, `enemy_brute_moving`, `enemy_brute_still`, `enemy_buffle_moving`, `enemy_buffle_still`, `enemy_chien_moving`, `enemy_chien_still`, `enemy_cracheur_moving`, `enemy_cracheur_still`, `enemy_franc_tireur_moving`, `enemy_franc_tireur_still`, `enemy_pillard_moving`, `enemy_pillard_still`, `enemy_rat_moving`, `enemy_rat_still`, `enemy_rodeur_moving`, `enemy_rodeur_still`, `enemy_roi_rat_moving`, `enemy_roi_rat_still`, `enemy_tourelle_moving`, `enemy_tourelle_still`, `weapon_arsenal`, `weapon_canon_ricochet`, `weapon_disque`, `weapon_lance_grenades`, `weapon_lance_lames`, `weapon_laser`, `weapon_mitraillette`, `weapon_mortier`, `weapon_plasma`, `weapon_revolver`, `weapon_roquette`, `weapon_traqueur`, `weapon_machine_gun`, `weapon_pistol`, `weapon_rifle`.
- **2 plus tard** (scénarios neufs de la branche, rattrapés par M1, D51 compris) : `dash_iframes`
  (ligne 56), `dash_stun_buffer` (ligne 42).
- **69 inchangées** (bénies par movement-feel, non touchées par M1) : `bench_horde`, `buy_door`, `buy_perk`, `buy_wall_weapon`, `clock_events`, `dash_wall`, `door_open`, `downed_all_lose`, `drop_pickup_swap`, `effect_tick`, `enemy_archer_moving`, `enemy_archer_still`, `enemy_breacher_moving`, `enemy_breacher_still`, `enemy_charge`, `enemy_charger_moving`, `enemy_charger_still`, `enemy_grunt_moving`, `enemy_grunt_still`, `enemy_keep_distance`, `enemy_kiter_moving`, `enemy_kiter_still`, `enemy_turret_moving`, `enemy_turret_still`, `enemy_wander`, `enemy_zombie_1_moving`, `enemy_zombie_1_still`, `enemy_zombie_2_moving`, `enemy_zombie_2_still`, `enemy_zombie_full_moving`, `enemy_zombie_full_still`, `equilibrage_joueurs_quad`, `four_players_idle`, `idle`, `movement_melee`, `powerup_carpenter`, `powerup_insta_kill`, `powerup_nuke`, `run_lose_summary`, `shop_tour`, `stat_move_speed`, `surface_enemy`, `surface_ice`, `surface_none`, `surface_walk`, `testbed_arena_idle`, `testbed_corridor_idle`, `testbed_follower`, `testbed_two_rooms_door_idle`, `testbed_window`, `two_players_idle`, `variant_draw`, `variant_elite`, `variant_fast`, `variant_none`, `weapon_axe`, `weapon_bare_hands`, `weapon_bare_hands`, `weapon_club`, `weapon_crocs`, `weapon_fusil_a_pompe`, `weapon_griffes`, `weapon_knife`, `weapon_massue`, `weapon_shotgun`, `weapon_shotgun`, `weapon_sword`, `weapon_zombie_claws`, `window_repair`.

**Preuve du §10 par catégorie** (dumps `ALACOD_DUMP_TRACE` de `main` et de la fusion, un scénario par
jeu ; comparaison composant par composant, `../d40/frame_diff.py`) :
- à la **frame 0**, seuls `combat::actors::DashState` (nouveaux champs du dash) et `sim_core::stats::Stats`
  (`Acceleration` de base des joueurs **150 → 3000**) diffèrent, sur `four_players_shooting` (zombies),
  `testbed_dummy_shoot` (testbed) et `throne_mutation_choice` (throne) ;
- ces deux composants ignorés : `four_players_shooting` et `testbed_dummy_shoot` sont **identiques sur
  toutes les frames** (joueurs immobiles) ; `throne_mutation_choice` diffère d'abord à **f5** par la
  `Velocity` du joueur (−49,99 au lieu de −1,77 : la course nerveuse), puis positions et cibles des ennemis.

### Mesures avant (`main` `836d3cf`) / après (fusion), une sim à la fois

| Mesure | main | fusion |
|---|---|---|
| zombies, 20 graines `avant_poste`, 4 acheteurs, vague 5 | 20/20, 0 desync, 0 soft-lock, 0 mort ; méd. 7 076 frames | 20/20, 0 desync, 0 soft-lock, 0 mort ; méd. **6 383** |
| throne, 20 graines, 2 bots, étage 3 | 20/20, 0 soft-lock, 0 mort ; méd. 3 405 | **19/20, 1 soft-lock**, 0 desync, 3 graines avec un mort ; méd. 3 511 |
| throne, 20 graines, 4 bots, étage 3 | 20/20, 0 soft-lock, 0 mort ; méd. 2 328 | 20/20, 0 soft-lock, 0 mort ; méd. 2 626 |

**Soft-lock ajouté, graine 19 à 2 bots** (décision d'orch : livré tel quel, diagnostic confié à b1) :
bloqué au deuxième étage (index 1) jusqu'à f3050 ; relevé D42 : dernier ennemi, un `rat` à 29,5 PV en
(10, 37), à 173 px des bots et hors de vue, et « 1 ennemi dont le point visé par le flow field chevauche
un mur physique avec son collider » ; les deux bots sont en pleine santé. Sur `main`, la même graine
finit les trois étages (f4321). Les graines 5, 8 et 10 finissent la run avec un mort (aucun sur `main`).

### Autres vérifications

- Tests des crates : 615, 0 échec. `make lint` des trois jeux, `cargo fmt --check`, `check_forbidden` 4
  (inchangé), `check_rollback_registration` OK.
- `make gen` des trois jeux : aucun `.ron` généré modifié, toutes les attentes « ok », traces
  « différente » (dans la liste ci-dessus).
- **p2p à deux clients** (`zombies`, README) : traces identiques, 599 lignes, sha256 `0c2ad16f…`
  (nouvelle référence : la simulation change avec movement-feel).

### Bench au calme (fenêtre d'orch, binaires `--release` de `main` `836d3cf` et de la fusion, alternés, charge 0,8 à 2,3)

| Scénario | main (passage 1 / 2) | fusion (passage 1 / 2) |
|---|---|---|
| `bench_horde` | 58,6 / 58,4 fps | 58,7 / 59,2 fps |
| `bench_bullets` | 136,6 / 133,8 | 135,3 / 132,8 |
| `bench_cave` | 200,6 / 199,9 | 200,9 / 206,5 |
| `bots_four_mixed` | 103,9 / 109,1 | 99,6 / 100,1 |

Les trois bench sont dans le bruit. `bots_four_mixed` perd 4 à 8 %, mais sa simulation n'est plus la même
(course nerveuse, autres trajectoires et combats) : ce n'est pas un coût à simulation égale.
