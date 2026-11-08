# P-1 — Parité des règles des bancs gigue et armes (Mac de William)

Jalon servi : **S1, dim 15** (plan `docs/taches/plan-1837-cli.md` § 6, ligne P-1 : « la gigue au tempo ; la hache qui renvoie les balles ») et la cible jeu 12
(« 4 fantassins… un cône au sol annonce le tir 1 s avant… rechargent 3 s »). Branche engine `wquintal/p-1-parite-labo` (tête de code `5d2ed156`), branche 1837 `p-1-parite`
(`72a9acb`). Convention : `docs/conventions.md` § 110 ; dettes D194–D197, D199 fermée. Référence : les `DEFAUTS` de `gigue.js`, `armes.js`, `allures.js` (`1867_lore` @ `d5a27d9`,
aucun réglage exporté). Tableau de parité : `1837/docs/parite-labo.md` (§ 12 = état final) ; protocole du soir : `1837/docs/protocole-soir-p-1.md`.
Les sept écarts du § 10 du tableau sont aux **défauts**, marqués « en attente de William » (D194). `game::music_sync` et `MeleeParried` restent retirés (commit `f5bc688e`, D149 et D151,
à revert si William les veut). Reprise de X-3 (`x-3-verbes-d1.md`, `x-3-arret.md`) : le P1 et les trois P2 de sa revue sont fermés ; les « À REMPLIR » de son rapport renvoient ici.

## Engine (générique, aucun nom de 1837)

| Point | Issue | Preuve |
|---|---|---|
| 1. Gigue au tempo : horloge à `tempo`, jugement parfait/bien, bonus de distance et d'i-frames, fenêtre élargie en coop, sur place, coupe, courbe, enchaînement | fait | `ClockDef::tempo`, `sim_core::timing`, `DashConfig` (`profile`, `in_place_without_direction`, `cut_from`, `chain_window_frames`, `on_beat`), `DashCadence` neutre ; scénarios `dash_au_tempo` (parfait 72 u, bien 60, hors temps 48), `dash_au_tempo_coop`, `dash_coupe_sur_place` ; test `iframes_du_pas_au_tempo_parfait_bien_hors_temps` (21 / 18 / 14 frames) |
| 2. Hache qui renvoie : fenêtre de frappe, secteur, renvoi vers la visée, vitesses, coup en file, marche pendant le coup | fait | `strike_frames`, `reflect.sector`, `reflect.direction: Aim`, `speed_mult`, `perfect_speed_mult`, `queue` (`MeleeQueue` neutre), `move_speed_mult` ; scénarios `renvoi_secteur`, `renvoi_secteur_visee`, `coup_en_file`, `parade_projectiles*` |
| 3. Tireur posté (visée et regard qui suivent, attente tirée, départ décalé, salve perdue sur un coup, recul, retour au poste) | fait | `ai.ranged_cycle` (`character::enemy::ai::cycle`, `RangedCycle` neutre) ; `tireur_poste_{cycle,suit,fige_temoin,touche}` ; suivi de revue : un coup reçu garde l'attente due (`max`), `tireur_poste_touche_attente` (échoue avec l'ancien remplacement : flèche dès f110) ; `ranged_cycle` dans le schéma et le lint (3 fixtures) |
| 4. Gestes du fusil : épaulé, geste de tir, rechargement automatique, marche en joue et en rechargeant, rechargement pendant la gigue, clic perdu | fait | `WeaponConfig::gesture` (`WeaponGesture`, `WeaponState::{raise_ends, gesture_ends}`) ; `geste_epaule_tir` (balle 9 f après l'appui, plein 99 f après, marche 112,5 → 56,25 → 28,125), `geste_dash_recharge` (échoue sans `reload_through_dash`), `geste_dash_epaule`, `geste_clic_perdu`, généré `weapon_arme_a_gestes` ; D199 fermée (pose `shoulder` sur `raise_ends`) |
| 5. Zone touchée distincte du collider (défaut § 10, point 6) | fait | `CharacterConfig::hurtbox`, `combat::collider::Hurtbox`, lue par les balles et la mêlée ; `zone_touchee_{bord,centre,temoin}` ; lint « la zone tient dans le collider » (fixture `hurtbox_outside_collider`) |
| 6. Bouche du canon (défaut § 10, point 7) | fait sans code | `bullet_offset_*` tourne déjà avec la visée : `(14.4, 0)` et `weapon_offset` à zéro (RON de 1837) |
| 7. Rayon des balles du banc (2,2 u) | **pas fait** | la balle de l'engine fait 10 px de large (mesuré : fenêtre de touche de 30 px sur une cible de 20) ; D195 |
| 8. Brown Bess du joueur, mode « maintenir », immobilité | **pas fait**, hors D1 | D196 |
| 9. Repères de jugement sur l'épaulé et le geste de tir | **pas fait** | D197 |

Corrections de miroirs de lint trouvées au premier `alacod lint` de 1837 : `dash.profile` et les bonus de `on_beat` s'écrivent sans `Some` (comme `sim_core::dash::DashConfig`), le miroir exigeait
`Some` ; fixture `dash_on_beat_unknown_clock`.

## 1837 (`p-1-parite`)

| Point | Issue | Preuve |
|---|---|---|
| Tableau de parité, une ligne par règle | fait | `docs/parite-labo.md` (étape 1 poussée le 6 nov ; § 12 : état de chaque ligne avec son scénario) |
| RON : Frère (gigue 28 f, EaseOut, coupe 18, enchaînement 21, `on_beat` 129 temps/min ±60/±120 ms, sans inertie, `hit_iframes` 36), horloge `clocks/gigue.ron`, hache (frappe 10–13, secteur 31,2 u / 160°, × 1,5 / × 1,4, coup en file, 60 %), fusil (poids 0,88, `gesture` 9/18/72, × 0,6 / × 0,5, bouche 14,4 u), fantassin (salve, `ranged_cycle`, `stagger_frames` 37, brown_bess 144 u/s), zones touchées 7,2 / 8,4 u, quatre fantassins dans la cour | fait | `alacod lint` : aucune erreur (20 personnages, 6 armes, 6 armes de corps à corps), avec les nouvelles règles `ranged_cycle` et `hurtbox` |
| Le jeu tourne sur l'engine P-1 | fait | `cargo run --no-default-features` avec `ALACOD_HEADLESS=1 ALACOD_INPUT=neutral ALACOD_STATE_TRACE=… ALACOD_EXIT_AT_FRAME=900` sur le `rev` `5d2ed156` : 900 frames, synctest, aucune erreur ni desync (`rollbacks n=898`) |
| Jeu à côté du banc | **non vérifié** (la partie du soir de William) | protocole en dix lignes : `docs/protocole-soir-p-1.md` |

## Preuve engine

- **`make check_rapide` vert sur `5d2ed156`** : `~/orca/runs/alpha/logs/p-1-parite-labo-check-rapide-5d2ed156.log` (1597 tests, 0 échec ; format, tools, forbidden, rollback registration, sim boundary, lint, test_crates) ; tranche 4 seule : `…-c408da3b.log` (1593) ; tranches 1 à 3 : `…-e198f742.log`. Dernières lignes : `✓ Contrôle rapide passé (suite de scénarios : la nuit)`.
- **Scénarios** : 52 scénarios du lot et voisins (dash au tempo, renvoi, tireur posté, X-3, `bomb_*`, `explode_*`, `isaac_bomb*`, explosifs, `idle`, `bench_horde`, `two/four_players_shooting`) rejoués **sans BLESS** après chaque tranche : verts, aucune trace existante ne bouge.
  Traces **neuves** bénies : `dash_au_tempo{,_coop}`, `dash_coupe_sur_place`, `renvoi_secteur{,_visee}`, `coup_en_file`, `tireur_poste_{cycle,suit,fige_temoin,touche,touche_attente}`, `geste_{epaule_tir,dash_recharge,dash_epaule,clic_perdu}`,
  `zone_touchee_{bord,centre,temoin}`, `generated/testbed/weapon_arme_a_gestes`. Traces de X-3 **rebénies** (pas des traces de `main` : elles dataient d'avant les correctifs de X-3 lui-même, `0e57f6f3`, même échec avec le travail P-1 retiré) :
  `fenetre_esquive`, `fenetre_rechargement` (seules les frames de jugement bougent : file `TimingJudged`), `parade_projectiles` (renvoi dès f63, tourelle touchée f112, attentes recalées).
- **Discrimination** : `geste_dash_recharge` échoue avec `reload_through_dash: false` ; `tireur_poste_touche_attente` échoue avec l'ancien remplacement de `ready_at` ; `geste_clic_perdu` et `geste_dash_epaule` ne passent que si le clic perdu et l'annulation par le dash existent.
- **`make schedule_order_diff`**, base `origin/main` = `edce94cc` (dumps de la base et de la tête construits à part, la base dans un worktree jetable puis supprimé ; les dumps sont dans `/tmp/p1-sched`, non gardés) :
  types rollback : 5 neufs (`FrameEvents<ProjectileReflected>`, `FrameEvents<TimingJudged>`, `DashCadence`, `MeleeQueue`, `RangedCycle`), tous posés juste avant les cinq de clôture ; **la tranche 4, `Hurtbox` et le suivi de revue n'en ajoutent aucun**
  (`rollback.txt` identique à la tête d'avant). `GgrsSchedule` : systèmes neufs `sync_carry_weight_system`, `melee_reflect_system`, `ranged_cycle_system` ; un seul déplacement voulu dans le set `Weapon` :
  `bullet_rollback_collision_system` passe après la mêlée (P2a de X-3, une balle renvoyée ne blesse pas dans la même frame) ; les autres lignes du diff (`table_flip_system`, `hazard_system`, `shell_pickup_system`, `boss_drop_system`, `process_obstacle_damage`) sont du tri topologique de systèmes
  non ordonnés entre eux. Preuve de l'ordre du set `Weapon` par les scénarios voisins sans BLESS (explosion et mêlée : `bomb_*`, `explode_effect`, `isaac_bomb*`, `movement_melee`).
- **Checklist GGRS** : `Fixed` partout, `order_iter!`, `BTreeMap`, `FrameEvents`, types rollback neufs en variante neutre ; champs ajoutés à un composant rollback déjà sous checksum (`WeaponState`, `WeaponConfig`) : hash et Debug manuels, champs hachés seulement
  s'ils sont actifs (les traces existantes sont donc intactes, vérifié par le rejeu sans BLESS) ; `Hurtbox` est un composant **statique** posé à la création (comme `Tags`), non rollback.
- **Ce que la preuve ne couvre pas** : la mêlée des ennemis contre le joueur (`enemy_attack_system`, portée) et les explosions ne lisent pas `Hurtbox` (D195) ; aucun lancement p2p (`make test_multiplayer`) ni capture d'écran n'a été fait pour P-1 ; le jeu à côté du banc est la partie de William.

## Ce qui reste

- La réponse de William aux sept écarts du § 10 (D194) : chacun est un nombre de RON de 1837, aucun ne touche l'engine.
- D195 (rayon des balles, zone touchée aux explosions et aux ennemis), D196 (Brown Bess du joueur, mode « maintenir »), D197 (repères de jugement sur l'épaulé).
- Le revert de `f5bc688e` si William veut la parade mêlée contre mêlée et `music_sync` (D149, D151).
- Le `rev` de 1837 sur `main` après le merge de l'engine (le coordinateur 1837).
