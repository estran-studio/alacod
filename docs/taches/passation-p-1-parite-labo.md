# Passation P-1 « parité des règles des bancs gigue et armes » (2026-11-07, fin d'après-midi, Mac — bascule de modèle)

Engine : `wquintal/p-1-parite-labo` @ la tête poussée avec cette note (tête de code `453bc8f4` + merge de `origin/main` `b6bbe24d` en `f80b7c8a`
+ traces neuves de la tranche 3) ; 1837 : `p-1-parite` @ `09a7d99` (poussée, rien de neuf).
Référence : les `DEFAUTS` de `gigue.js`, `armes.js`, `allures.js` (`1867_lore` @ `d5a27d9`, aucun écart avec `e02965e`, vérifié par `git diff`).
Les 7 écarts du § 10 du tableau : défauts appliqués, en attente de William. Jalon : S1 dim 15 (tranches 1 et 2), cible jeu 12 (tranche 3).

## Fait et vérifié
- **Tranche 1, gigue au tempo** (`2dacf9c7`) : `ClockDef::tempo`, `run::clock::beat_gap`, `sim_core::timing::grade_gap_ms`, `DashConfig`
  (`profile: EaseOut`, `in_place_without_direction`, `cut_from`, `chain_window_frames`, `on_beat` avec `coop_window_mult` lu sur
  `SessionPlayers`), `DashCadence` neutre. Scénarios neufs verts et bénis : `dash_au_tempo`, `dash_au_tempo_coop`, `dash_coupe_sur_place`.
  Test unitaire `iframes_du_pas_au_tempo_parfait_bien_hors_temps` (21 / 18 / 14 frames).
- **Traces X-3 rebénies** (pas des traces de `main`) : `fenetre_esquive`, `fenetre_rechargement`, `parade_projectiles`. Même échec avec le WIP P-1
  retiré : les traces dataient d'avant les correctifs de X-3 lui-même (`0e57f6f3`). Les deux `fenetre_*` ne diffèrent qu'aux frames de jugement
  (file `TimingJudged`). Pour `parade_projectiles`, le renvoi a lieu dès f63 et la tourelle est touchée à f112 (attentes recalées).
- **Tranche 2, la hache qui renvoie** (`883bbe30`) : `strike_frames`, `MeleeHitbox::strike`, `reflect.sector`, `reflect.direction: Aim`,
  `speed_mult` et `perfect_speed_mult`, `queue` (`MeleeQueue` neutre), `move_speed_mult` pendant le coup, lint et fixture `melee_strike_outside_attack`.
  Scénarios neufs verts et bénis : `renvoi_secteur`, `renvoi_secteur_visee`, `coup_en_file`. La revue indépendante de X-3 et de P-1 n'a trouvé
  ni P0 ni P1 ; les quatre points P1/P2 de X-3 sont fermés. Ses P2 corrigés : l'orbite retirée au renvoi, `sync_carry_weight_system` placé après
  `player_melee_attack_system`.
- **Scénarios voisins sans BLESS, verts sur `883bbe30`** : toute la liste X-3 et dash (`arme_lourde*`, `fenetre_*`, `parade_projectiles*`,
  `dash_*`, `clock_*`, `effect_on_dodge`, `movement_melee`, `pit_dash`, `avant_poste_fenetre_coin`), plus `idle`, `two_players_shooting`,
  `four_players_shooting`, `bench_horde`. Les 7 scénarios à balles `Explosive` (`explosive_bullet_*`, `status_fear`,
  `generated/testbed/weapon_balle_explosive*`, `weapon_status_fear`) sont verts aussi.
- **schedule_order_diff sur `883bbe30`, base `origin/main`** : dumps dans `/tmp/p1-sched/{base,head}` (non gardés). Types rollback : 5 neufs,
  posés juste avant les cinq de clôture. Systèmes neufs : `melee_parry_system`, `melee_reflect_system`, `sync_carry_weight_system`. Dans le set
  `Weapon`, un seul vrai déplacement : `bullet_rollback_collision_system` passe après la mêlée (correctif P2a de X-3). Le reste du diff vient du
  tri topologique (des `clear_frame_events` sans ordre entre eux).
- `claude-say.sh` réparé pour le bash 3.2 du Mac (`1016b3e1`) : `test_tools` passe.
- Les deux tests d'horloge d'étage de X-3 sont réparés (`50428e8c`) ; `effects_runtime::tests` passe.
- **Retrait de la parade mêlée contre mêlée et de `music_sync`** : commit séparé `f5bc688e`, à revert si William les veut (D149, D151, § 110).
  Les tests qui citaient le banc sont renommés, la fixture `hache` devient `coup_long`.
- **Merge de `origin/main` (`b6bbe24d`, M-1) dans la branche** : `f80b7c8a`, sans rebase. Trois conflits de docs seules résolus en gardant les
  deux côtés : `docs/conventions.md` (§ 108 de M-1 placé avant notre § 110), `docs/taches.md` (lignes X-3 puis `LIVRÉ` de M-1),
  `docs/taches/dettes.md` (D149–D152 puis D190–D191). Aucun code en conflit.
- **Traces de la tranche 3 bénies** sur `f80b7c8a` (commit qui suit le merge) : `tireur_poste_cycle`, `tireur_poste_suit`,
  `tireur_poste_fige_temoin`, `tireur_poste_touche` (« trace de référence écrite », `test scenarios ... ok`).

## Fait, non vérifié
- `f5bc688e` et `453bc8f4` n'ont pas tourné en entier. La partie jeu a compilé et les scénarios `tireur_poste_*` sont passés, mais ni les
  tests unitaires (`combat` sans les tests de parade, `content` sans la musique) ni `check_rapide` n'ont tourné.
- **Tranche 3, tireur posté** (`453bc8f4`, WIP) : `ai.ranged_cycle` (`character::enemy::ai::cycle`, `RangedCycle` neutre, enregistré avant la
  clôture). Il couvre la visée et le regard qui suivent, l'attente tirée (flux `ranged_cycle`), le départ décalé par rang, la salve perdue sur
  un coup reçu, le recul et le retour au poste. Les quatre traces sont bénies (voir plus haut) ; le rejeu sans BLESS (étape 1) n'a pas
  encore tourné.
- 1837 `09a7d99` : les RON aux nombres du tableau (Frère, horloge `gigue`, hache, poids du fusil, fantassin, salve, arme
  `brown_bess_fantassin`, quatre fantassins dans `cour.ldtk` et `sandbox/cour.ron`). `alacod lint` n'a pas tourné. Le `rev` de `Cargo.toml`
  est encore `0e7c8d58` : il faut le passer sur l'engine P-1, et `Cargo.lock` avec.

## Prochaine étape exacte
1. Rejouer le lot et ses voisins, **sans BLESS** (le premier essai est tombé sur un nom inconnu, `explosive_bullet_basic` : rien n'a joué ;
   les vrais noms sont `explosive_bullet_composed`, `explosive_bullet_hit`, `explosive_bullet_range`) :
   `env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1 make test_scenarios SCENARIO=tireur_poste_cycle,tireur_poste_suit,tireur_poste_fige_temoin,tireur_poste_touche,dash_au_tempo,dash_au_tempo_coop,dash_coupe_sur_place,renvoi_secteur,renvoi_secteur_visee,coup_en_file,arme_lourde,arme_lourde_temoin,fenetre_esquive,fenetre_etage,fenetre_fraction_esquive,fenetre_rechargement,parade_projectiles,parade_projectiles_expiree,parade_projectiles_instantanee,generated/testbed/weapon_pistolet_lourd,dash_in_place,dash_jump_bullets,dash_shove,dash_aim_change,clock_events,effect_on_dodge,movement_melee,idle,bench_horde,bomb_breaks_rock,bomb_chain_barrel,bomb_digs_cave,bomb_hurts_owner,bomb_pickup_cap,bomb_place_explode,explode_effect,explode_wall,isaac_bomb_rock,isaac_bombe_fiole,enemy_flee,status_charm_contact,generated/isaac/champion_suiveur_explosif,generated/isaac/item_bombe_fiole,generated/testbed/champion_elite_explosif_salve,generated/testbed/champion_elite_explosif_nu,generated/testbed/champion_elite_explosif_explosif,generated/testbed/weapon_balle_explosive,generated/testbed/weapon_balle_explosive_composee,explosive_bullet_composed,explosive_bullet_hit,explosive_bullet_range`
   (les `bomb_*`, `explode_*`, `isaac_bomb*`, `enemy_flee`, `status_charm_contact` et explosifs servent aussi la preuve P2 du point 2).
   Consigne du coordinateur : « E prêt <sha> » dès que les tranches 1 à 3 sont vertes avec `check_rapide` (journal
   `~/orca/runs/alpha/logs/p-1-parite-labo-check-rapide-<sha>.log`) ; `schedule_order_diff` contre `origin/main` **à jour** (`b6bbe24d` ou plus).
2. **Revue du coordinateur sur `1016b3e1`, encore ouverte** :
   - **P2** : l'ordre du set `Weapon` (`bullet_rollback_collision_system` après la mêlée). Le prouver par `make schedule_order_diff BASE=origin/main`
     (`scripts/schedule-order-diff.sh` construit la base dans son propre target, un build à froid de ~20 min et ~7 Go) et par les scénarios
     existants qui ont une explosion et de la mêlée (`bomb_*`, `explode_effect`, `melee_*`, les `isaac_*` à bombes), sans BLESS. Si une trace
     bouge, rétablir l'ordre d'avant, sauf besoin prouvé.
   - **P3** : écrire le rapport `docs/taches/rapports/p-1-parite-labo.md` (§ 110 y renvoie) ; remplir les trois « À REMPLIR » de
     `docs/taches/rapports/x-3-verbes-d1.md:155-163` (renvoyer au rapport P-1). Les noms de tests qui citaient le banc sont déjà renommés (`f5bc688e`).
   - **Retrait** de `music_sync` et de `MeleeParried` : fait en commit séparé `f5bc688e`, à vérifier au contrôle rapide.
3. `scripts/target-sweep.sh target` si le disque passe sous 40 Go (39 à 47 Go le 7 nov), puis `make check_rapide`, journal dans
   `~/orca/runs/alpha/logs/p-1-parite-labo-check-rapide-<sha>.log`, puis status « E prêt <sha> ».
4. 1837 : passer le `rev` de `Cargo.toml` sur la tête engine poussée, puis
   `cargo run -q -p content --bin alacod --profile headless -- lint ~/orca/workspaces/1837/p-1-parite` depuis le worktree engine.
5. La suite : la tranche 4 (gestes du fusil : épaulé de 9 frames, 60 % en joue, 50 % en rechargeant, rechargement qui continue pendant la
   gigue ; zone touchée de 7,2 u ; bouche du canon), le hitstop (il est dans `stagger_frames: 37` = 4 + 33 pour le fantassin), les dettes
   D194–D197, le tableau mis à jour, le protocole de dix lignes pour la partie de William, la ligne `LIVRÉ` de `docs/taches.md`, et la
   suppression de cette note dans le commit du rapport.

## Pièges
- **Jamais de `CARGO_TARGET_DIR` partagé avec un autre checkout** : la base de `schedule_order_diff` a écrasé les artefacts du target (fraîcheur
  à l'mtime) ; il a fallu `touch` tous les `.rs`. Base gardée à part dans le worktree `/tmp/p1-base` (target supprimé).
- Tests unitaires : toujours avec les features de dev (`--features game/dev-dylib`, comme `make test_crates`), sinon tout se recompile (18 min).
- Une `Query` prend 15 éléments au plus. `FixedVec2` n'a pas de `Hash` : on stocke des `(Fixed, Fixed)`. `asset_name_ref` doit être unique.
- `arena.ldtk` : la rangée 11 a un mur à x 312–408, y ≈ 600. Un personnage posé dessus ne peut plus revenir à son poste.
- Scénarios : les inputs agissent 5 frames plus tard, et une position lue à la frame N n'inclut pas le pas du dash lancé à N.
- Pas de `pkill` par motif ; arrêter un job par son PID ou par `proc://<job>/kill`.
- `make test_scenarios SCENARIO=…` refuse la liste entière (panic `scenarios.rs:198`, « scénario inconnu ») si **un** nom n'existe pas :
  vérifier les noms par `ls tests/scenarios` avant une longue liste.
- Disque : 41 Go libres le 7 nov au soir (`target/headless` = 17 Go) ; `scripts/target-sweep.sh target` avant `schedule_order_diff`.
