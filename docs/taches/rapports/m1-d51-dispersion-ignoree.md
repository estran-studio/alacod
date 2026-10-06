# Rapport — m1-d51-dispersion-ignoree (`FiringModeConfig.spread` jamais appliqué, D51)

Branche partie de `origin/main` f198354.

## État en cours

Vérifié, livré. Suite complète sur la branche (f198354 + D51) : **toutes les attentes vertes** ;
seules différences, les traces listées plus bas (à bénir) ; tests des crates verts (seul échec :
`scenarios`, pour ces traces) ; `make lint` ×3, `cargo fmt --check`, interdits, enregistrement
rollback, `cargo check -p throne`, exemples : verts ; `make gen` ×3 : **0 attente en échec**,
32 traces générées différentes (testbed 17, zombies 3, throne 12), code de sortie 1 pour ces seules
traces. Target purgé.

**Fiche amendée par orch** (message du 2026-10-06, la fiche prévoyait « ½ j de code, le reste en
mesure » et des traces seulement) : le correctif fait échouer environ 30 scénarios sur leurs
**attentes** ; décisions : (1) scénarios à bots remesurés, `clone_quad` garde l'attente M0 « vague
5 » en allongeant ses frames ; (2) défaites scriptées : chercher des défaites à 2 bots sur 1..50,
sinon supprimer les trois scénarios ; (3) scénarios de mécanique et tests de crates remesurés un
par un ; (4) jauges `test:` des armes recalibrées.

## Correctif

- `crates/combat/src/weapons/mod.rs` : la branche « tir simple » (tout mode sauf `Shotgun`)
  calcule la direction par `single_shot_direction(aim_dir, random, weapon_config.spread)` :
  angle `(random − ½) × spread`, donc dans `[−spread/2, spread/2]` ; avant, `(random − ½) × 1`
  (±0,5 rad pour toute arme). `spread` = 0 rend `aim_dir` tel quel (`FixedMat2::from_angle(0)`
  n'est pas l'identité exacte en `Fixed` : cos ≈ 1,00002). Un seul tirage RNG par balle (flux
  `weapons`), tiré avant comme avant : les flux ne bougent pas, seule la direction change.
- Tests : `weapons::tests::dispersion_nulle_tir_exact` (spread 0 : direction visée exacte, quel
  que soit le tirage), `dispersion_dans_la_demi_largeur` (angle dans `[−s/2, s/2]`, borne
  `−s/2` au tirage 0).
- Lint : `0 <= spread <= π` par mode (`lint_weapons`, champ `spread` lu dans le schéma miroir,
  0 si absent) ; fixture `weapon_spread_out_of_range` (spread 4).
- Doc : conventions §16 (unité radians, pleine largeur, formule, état avant D51), §29 (valeurs
  des armes de `throne` appliquées seulement depuis D51).
- Patterns ennemis (§20) : leur dispersion est bien appliquée (`emitter.rs`, `fan_angles` de
  `projectile.rs` multiplient par `spread`) ; pas de dette.

### Contenu : aucune conversion

Toutes les valeurs de `spread` des modes de tir sont en radians, cohérentes avec le seul
champ appliqué jusque-là (`spread_angle` du fusil : 0,4) ; aucune n'est > π :

- `throne` : mitraillette 0,15, disque 0,05, revolver 0,0005, les autres 0.
- `zombies` et `testbed` (`ZombieShooter/Sprites/Character/weapons.ron`) : mitraillette 0,15,
  son mode `rafale` 1, une arme à 0,01, les autres 0. `rafale` à 1 = ±0,5 rad, exactement
  l'ancien comportement : l'auteur l'a probablement calé sur ce qu'il voyait (le bug) ; il ne
  change pas.

## Mesure avant/après

« Avant » = `main` ddfc190 (code identique à f198354), « après » = cette branche ; mêmes
commandes, une sim à la fois.

| | avant | après |
|---|---|---|
| zombies `avant_poste`, 4 `acheteur`, 20 graines, vague 5 atteinte | 20/20 | 20/20 |
| zombies : frames moyennes à la vague 5 | 7078 | 7031 |
| zombies : kills (total) / morts / desync | 1040 / 0 / 0 | 1049 / 0 / 0 |
| throne 2 bots, témoins 1..20 : étage 3 | 16/20 | **20/20** |
| throne 2 bots : défaites | 4 (11, 12, 17, 20) | **0** |
| throne 2 bots : soft-locks / desync | 0 / 0 | 0 / 0 |
| throne 4 bots, témoins 1..20 : étage 3 / défaites / desync | 20/20 / 0 / 0 | 20/20 / 0 / 0 |

Balles perdues au troisième étage (méthode du digest de b0 : trace détaillée, un rejeu à la
fois, dump supprimé ; dégâts infligés / (munitions consommées × 8), mitraillette seule dans les
deux cas) :

| graine | avant | après |
|---|---|---|
| 25 | 66 % (défaite, étage 3 à f2819) | 62 % (étage 3 franchi, f3257) |
| 103 | 77 % (fin f5119) | 56 % (fin f3271) |
| 131 | 81 % (fin f5786) | 54 % (fin f2848) |

**À noter (décision 5 de la fiche, sans rééquilibrage)** : sur `throne`, les défaites des
témoins à 2 bots tombent de 4 à **0** et le troisième étage se finit bien plus vite ; sur
`zombies`, presque aucun effet (les acheteurs tirent surtout à la mitraillette, 0,15). La
mitraillette perd encore environ la moitié de ses balles (sa propre dispersion et les cibles
mobiles) : c'est le terrain de m1-v3-bots-armes (score de précision sur la vraie dispersion).
Le réglage des `spread` et des dégâts reste une tâche de contenu à part.

## Traces

**84 traces bougent, 93 ne bougent pas** (177 scénarios joués par la suite, générés
compris ; `throne_defaite_*` supprimés). Rien de béni par l'agent.

Preuve par catégorie (§10) : un scénario par jeu dont l'arme simple tire dès f10 diverge à la
**ligne 16 (f15)**, 5 frames après le premier input de tir (même délai que les preuves par inputs
des bots) : zombies `four_players_shooting` et `two_players_shooting`, testbed
`testbed_dummy_shoot`, throne `weapon_mitraillette` (généré). Les scénarios scriptés ne changent
pas d'inputs : seule la direction des balles change.

**Inchangées** (aucun tir simple : mêlée, fusil à pompe, ennemis seuls, surfaces, boutique,
statuts par contact, inactifs) :
`bench_horde` `buy_door` `buy_perk` `buy_wall_weapon` `clock_events` `dash_wall` `door_open` `downed_all_lose` `drop_pickup_swap` `effect_tick` `enemy_archer_moving` `enemy_archer_still` `enemy_arroseur_moving` `enemy_arroseur_still` `enemy_breacher_moving` `enemy_breacher_still` `enemy_brute_moving` `enemy_brute_still` `enemy_buffle_moving` `enemy_buffle_still` `enemy_charge` `enemy_charger_moving` `enemy_charger_still` `enemy_chien_moving` `enemy_chien_still` `enemy_cracheur_moving` `enemy_cracheur_still` `enemy_franc_tireur_moving` `enemy_franc_tireur_still` `enemy_grunt_moving` `enemy_grunt_still` `enemy_keep_distance` `enemy_kiter_moving` `enemy_kiter_still` `enemy_pillard_moving` `enemy_pillard_still` `enemy_rat_moving` `enemy_rat_still` `enemy_ring` `enemy_ring_quad` `enemy_rodeur_moving` `enemy_rodeur_still` `enemy_roi_rat_moving` `enemy_roi_rat_still` `enemy_telegraph` `enemy_tourelle_moving` `enemy_tourelle_still` `enemy_turret_moving` `enemy_turret_still` `enemy_wander` `enemy_zombie_1_moving` `enemy_zombie_1_still` `enemy_zombie_2_moving` `enemy_zombie_2_still` `enemy_zombie_full_moving` `enemy_zombie_full_still` `equilibrage_joueurs_quad` `four_players_idle` `idle` `movement_melee` `powerup_carpenter` `powerup_insta_kill` `powerup_nuke` `run_lose_summary` `shop_tour` `stat_move_speed` `surface_enemy` `surface_ice` `surface_none` `surface_walk` `testbed_arena_idle` `testbed_corridor_idle` `testbed_follower` `testbed_two_rooms_door_idle` `testbed_window` `throne_duo_defaite` `two_players_idle` `variant_draw` `variant_elite` `variant_fast` `variant_none` `weapon_axe` `weapon_bare_hands` `weapon_club` `weapon_crocs` `weapon_fusil_a_pompe` `weapon_griffes` `weapon_knife` `weapon_massue` `weapon_shotgun` `weapon_sword` `weapon_zombie_claws` `window_repair`.

**Qui bougent** :
`ammo_burst` `ammo_shared_reserve` `avant_poste_demo` `bench_bullets` `bench_cave` `bot_floors_three` `bot_prudent_dodge` `bot_prudent_nododge` `bots_four_mixed` `bots_two_fonceurs` `bullets_walls` `clock_floor_reset` `clone_duo` `clone_quad` `clone_solo` `dash_aim_change` `difficulty_scales` `downed_bleedout` `downed_revive` `effect_none` `effect_on_damage_taken` `effect_on_kill` `enemy_flee` `equilibrage_joueurs_duo` `explode_wall` `four_players_shooting` `friendly_fire_cursed` `friendly_fire_never` `immune_tag` `levelup_choice` `levelup_timeout` `points_on_kill` `portal_next_floor` `powerup_double_points` `powerup_drop_on_kill` `powerup_max_ammo` `remote_first_fight` `shoot_around` `status_freeze_enemy` `status_slow_enemy` `testbed_ally_safe` `testbed_civilian_blocks` `testbed_dummy_shoot` `testbed_target_hits` `throne_ammo_pickup` `throne_floor_1` `throne_mutation_choice` `throne_progression` `throne_quad` `throne_softlock_recul` `throne_solo` `throne_three_floors` `two_players_shooting` `weapon_arsenal` `weapon_canon_ricochet` `weapon_disque` `weapon_fireball_gun` `weapon_foreuse` `weapon_grenade` `weapon_grenade_creuse` `weapon_lance_grenades` `weapon_lance_lames` `weapon_laser` `weapon_machine_gun` `weapon_mitraillette` `weapon_mortier` `weapon_pistol` `weapon_plasma` `weapon_pool_drop` `weapon_proj_bounce` `weapon_proj_gravity` `weapon_proj_homing` `weapon_proj_lifetime` `weapon_proj_pierce` `weapon_proj_size` `weapon_revolver` `weapon_rifle` `weapon_roquette` `weapon_status_burn` `weapon_status_freeze` `weapon_status_slow` `weapon_status_stun` `weapon_traqueur` `weapons_workout`.

### Attentes remesurées (fiche amendée)

1. **Scénarios à bots** (attentes remesurées, `ALACOD_EVENTS=1`) :
   - `clone_quad` (référence M0) : **attente M0 gardée** — vague 5, 13 kills, quatre joueurs
     debout, Run `Playing` — atteinte à **f4602** au lieu de f4372 (vagues 2/3/4/5 à
     f871/f2039/f3544/f4602) ; 2 zombies (au lieu de 3) restent pour la Nuke de f750 ; soldes
     1310/1660/1350/3480. Vague 5 aussi en le jouant jusqu'à f9000 (vague 7 à f8087, 19 kills).
   - `clone_duo` : vagues inchangées (vague 5 à f4474), soldes 3730/3770/3780 et 1460.
   - `bot_floors_three` : portails à f34, f195, f638 (avant f56, f484, f892).
   - `throne_floor_1` : portail franchi à f578 (avant f937) ; `throne_progression` : niveau 1 à
     f363, étage 1 à f578, niveau 2 à f1284, étage 2 à f1591 ; test `hud_text_sur_throne_progression`
     aligné (Niv. 1 à f365, Niv. 2 à f1290, « Étage 3 » à f1912).
   - `throne_solo` : le bot seul **meurt** dans la troisième caverne à f2908 (avant : vivant à
     f3699 ; le scénario fixe l'état, pas la cible).
   - `throne_three_floors` : étages à f298, f1048, f3042 ; `throne_quad` : f428, f829, f2182 ;
     `throne_softlock_recul` : étage 2 à f1304.
2. **Défaites scriptées** : aucune défaite à 2 bots sur les graines 1..50 avec D51 (50/50 à
   l'étage 3) : `throne_defaite_tireurs`, `throne_defaite_boss`, `throne_defaite_coequipier` et
   leurs traces **supprimés**, à recréer après le rééquilibrage de contenu (D50, arroseur,
   difficulté).
3. **Scénarios de mécanique** (scriptés, preuve : mêmes inputs, seule la direction change) :
   - `effect_none` / `effect_on_damage_taken` : toutes les balles portent (26 et 30 coups au lieu
     de 16) : fenêtre de tir ramenée à f10-f106 (16 tirs) pour garder la comparaison du scénario
     (72 contre 92 points de vie).
   - `levelup_choice` : le deuxième kill (niveau 1) passe à f147 : choix `ChoiceC` déplacé à
     f150-f154, `coriace` à f156, 2 rads à f290 ; `levelup_timeout` : niveau 1 à f150, `vampire`
     à f747 ; test `progression_expectations_pass_and_fail` aligné (2 rads).
   - `weapon_pool_drop` : niveau 1 et premier `shotgun` à f147.
   - `shoot_around` : quatre zombies tués, mort du joueur à f1360 (avant f1241).
   - `status_freeze_enemy` : le breacher touche le joueur 1 à f1108 (avant f625) ;
     `status_slow_enemy` : f591 (avant f410).
   - `four_players_shooting` : toutes les balles du joueur 3 sortent par la fenêtre de gauche :
     marge gauche de f150 à −900 (portée).
   - Tests de crates : `status_expectations_pass_and_fail` (deux piles de brûlure à f100, l'échec
     attendu passe à 3) ; `bench_cave_detruit_au_moins_50` : 39 destructions mesurées, seuil à
     35 ; `m1_expectations_survive_rerecording` suit ses scénarios.
4. **Jauges `test:` recalibrées** (métadonnée hors hash) : `traqueur` (throne : les 8 coups vont
   à la cible, le mannequin reste intact ; `min_hits` 8), `grenade` (testbed : 8 éclats en vol à
   f174, 4 coups à f175), `status_burn` (deux piles à f100), `status_stun` (étourdi encore présent
   à f150), `machine_gun` (zombies : 66 coups, plafond 60 → 80). `bench_bullets` et `bench_horde`
   restent dans leur budget.
