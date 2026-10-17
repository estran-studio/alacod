# Rapport m3-t8-pools-marche-diable (b3)

Conventions : `docs/conventions.md` §73 (et §64 annoté). Branche `m3-t8-pools-marche-diable`, fusionnée avec `origin/main`
`2ffb95b5`. Décisions de William appliquées : **Q5** (épuisement par run et par pool) et **Q6** (le diable refuse si le
paiement tuerait, sinon la vie maximale baisse définitivement ; les cœurs d'âme paient à la place).

## 1. Fait

| Point de la fiche | Livré |
|---|---|
| 1. Pool par type de salle, épuisement | **Option A retenue** (tirage à la génération de l'étage). `items::PoolState` devient `{ seen, assigned }` (neutre, vide sans pool `exhaust`) ; `PoolState::assign` tire sur `exhaust_seed(graine, étage, clé, nombre d'objets sortis)` avec `draw_exhausting`, retire l'objet du pool et conserve l'assignation (idempotent). Appelé dans l'ordre canonique `(x, y, id)` des salles par `room_chest_system`, `room_shop_system` (`assign_shelves`, un emplacement par présentoir), `room_boss_slot_system` (nouveau) et à la pose d'un placement de scénario. Ouvrir, mourir ou acheter ne tire plus rien : l'objet vient de `assigned`. Déblocages M2-T12 (`unlocked_table`, `hidden_items`) appliqués. `RoomKindDef.boss_drop` (nouveau) + `boss_drop_system` : l'objet tombe à la mort du boss. Lint : `exhaust` + `exclude_held` refusé. |
| 2. Présentoir payé en vie max / âme | `RoomShopDef.cost: Option<Price>`, `Shelf.cost` exécuté : `Price::{soul_split, can_pay, pay}` reçoivent `soul` (âme d'abord, reste en vie max, refus si le reste tuerait) ; `game::shop::pay_cost`, modificateur permanent `MaxHealth Add −perte` (source `price:<clé>`), car `sync_health_from_stats` réécrit `Health.max` chaque frame. `Price::Consumable` aussi. Prompt HUD « Acheter X — 1 cœur » (`items::heart_label`, `shelf_price_label`). |
| 3. Salle du diable | `rooms/diable.ron` d'`isaac` : deux présentoirs du pool `diable` à `MaxHealth("2")` ; chance = celle de la grammaire (T5, un tiers). La variante « chance réduite si dégâts au boss » de la fiche n'est **pas** faite (voir §4). |
| 4. Scénarios | 20 scénarios neufs (tableau au §73) : `pool_exhaust_two_chests`/`_temoin`, `_two_rooms`, `_time_independent_ab`/`_ba`, `pool_unlock`/`_temoin`, `devil_deal_pay_health`, `_refused_lethal`, `_soul_saves`, `_soul_first`, `_refused_multiplied`, `devil_deal_isaac`, `devil_room_chance`/`_absent`, `boss_drop_pool` ; `grid_post_boss`, `grid_floor_seed_23`, `isaac_floor_seed_1` enrichis. Attentes `PoolCount`, `ItemsOnFloor`, `Shelf.cost`. |
| `isaac` en données | pools `tresor`, `boss`, `diable` (`exhaust`), `secrete` ; onze passifs de donnée d'attente (contenu : T11) ; salles `tresor`, `secrete`, `boss`, `diable`. Pas de boutique `isaac` (T11d). |

Autres : bot `explorateur` ne vise pas un présentoir à `cost` ; testbed : types `coffre_a`/`coffre_b`/`boss_butin`, cartes
`salles_coffres.ldtk`/`salles_boss_butin.ldtk`, pool `epuisable_temoin`. 3 fixtures de lint (`m3_t8_fixtures.rs`).

## 2. Vérifié (sur la fusion avec `origin/main` `2ffb95b5`, après les correctifs de revue)

- `make test_scenarios_par SHARDS=3` **sans BLESS** : 535 scénarios, 3 tranches ok (291 s) ; les seules traces blessées sont celles de la
  section 3 et des scénarios neufs.
- `make test_crates` : **1 115 tests passés, 0 échec** (dont `items` : `Price` avec âme, `PoolState::assign`, `heart_label` ; `game::shop` :
  `pay_cost`, `assign_shelves`, `max_health_after` ; `ui::hud` : prompt en cœurs ; `content` : `m3_t8_fixtures`).
- `make lint` ×5 : zombies, testbed, throne, gungeon, isaac « aucune erreur ». `make gen GAME=…` ×5 : code 0, `git status` sans diff.
- `check-forbidden.sh`, `check-rollback-registration.sh`, `check_sim_boundary`, `make test_tools` : OK.
- `cargo fmt --all -- --check` : seul écart, `crates/game/src/feedback.rs` (arrivé par `main`, M3-T17), que je n'ai pas touché ; mes fichiers sont formatés.
- Fuzz `ALACOD_FUZZ=0:8` (`--ignored`) : 1 test passé (61 s) ; `ALACOD_FUZZ_SCENARIO` ×5 (`pool_exhaust_time_independent_ab`,
  `devil_deal_soul_first`, `pool_unlock`, `devil_room_chance`, `devil_deal_refused_multiplied`, 4 graines, 600 frames) : passés.
- **p2p-stress isaac** (Docker OrbStack, `MATCHBOX_PORT=3576`, `COMPOSE_PROJECT_NAME=b3`), montage **temporaire non commité**
  (`floors: "sous_sol_un"`, `--seed 8` : salle du diable, trésor, boss), 3 000 frames, entrées `Haut/Bas/Gauche/Droite/Tir` + Interaction
  maintenue : **9 166 frames resimulées** (client 0 ; client 1 : 0), 0 `DesyncDetected`, codes 0/0, traces identiques (2 999 lignes) ;
  les `slot_assign` (`tresor`, deux `diable`, `boss`) sont dans le journal. Les inputs aléatoires n'ont pas conduit le joueur au marché ni
  tué le boss : le paiement, l'ouverture et le butin du boss ne sont donc **pas** prouvés en p2p réel (seulement l'assignation à la
  génération sous rollbacks) ; ils le sont en synctest (scénarios, fuzz).

## 2 bis. Preuves de non-régression de la règle

Mutations non rejouées. Les témoins par paires prouvent chaque règle : `pool_exhaust_two_chests` / `_temoin`, `pool_unlock` /
`_temoin`, `devil_deal_refused_lethal` / `devil_deal_soul_saves`, `pool_exhaust_time_independent_ab` / `_ba`.

## 3. Traces bougées

Preuve §5 (dumps détaillés `ALACOD_DUMP_TRACE`, trois états : `main` `ad2be1c2`, branche complète, branche **sans** `boss_drop` ni
marché du diable) sur les 17 traces qui différaient, comparées par `scripts/trace-diff.py --ignore PoolState,Chest` :

- **Cause 1, `Chest.table` et `PoolState`** (tresor/secrete de `isaac` ont un pool propre au lieu de `salle`, les pools à
  `exhaust` assignent à la génération) : seule différence des 4 `curse_*`, `generated/isaac/room_combat`, `grid_secret_room`,
  `hit_iframes_isaac`, `isaac_boots` (« identique » une fois `PoolState` et `Chest` ignorés).
- **Cause 2, présentoirs du diable** (`grid_floor_seed_123`, `_23`, `grid_secret_*`, `grid_treasure_*`, `grid_post_boss`, tous
  portant une salle du diable) : sans `boss_drop` ni diable, identique à `main` (état 2). Avec : présentoirs `Shelf` à f0.
- **Cause 3, butin du boss** (`generated/isaac/boss_gardien`, `grid_floor_seed_1`, `isaac_floor_seed_1`, `grid_floor_seed_23`) :
  première frame qui diffère = frame de la mort du boss, une entité `ItemPickup` de plus (`griffe_acier`, `reservoir_vital`).
  Conséquence : le bot ramasse le butin, la victoire / le portail arrivent plus tard ; attentes recalées (`grid_post_boss`,
  `grid_floor_seed_23`, `isaac_floor_seed_1`).
- Aucune trace de zombies, gungeon, testbed, throne ni générée hors `isaac` ne bouge.

## 4. Non fait / à relire

- Chance du diable réduite par les dégâts subis au boss : non faite (le plan est posé avant le combat) ; non confirmée par William.
- Choix « âme partielle puis reste en vie max » (Q6 dit seulement « à la place »).
- Pas de boutique dans `isaac` (le moteur est prouvé au testbed).
- Revue : voir la section 5.

## 5. Relecture indépendante (sous-agent `reviewer`, lecture seule)

Première passe, 2 P2, **corrigés** :
1. Un `Mul`/`Set` sur `MaxHealth` amplifie la perte : un paiement accepté par la règle brute (`4 > 2`) tuait (stat résolue 0 recopiée
   dans la vie). Correctif : `shop::max_health_after`, refus avant tout débit ; scénario `devil_deal_refused_multiplied` ; test unitaire.
2. `boss_drop_system` lisait `BossDefeated` après `floor_transition_system` (mauvais étage si le joueur franchit le portail dans la même
   frame). Correctif : `boss_drop_system.before(floor_transition_system)`.

Seconde passe sur les correctifs : paiement correct ; 1 P2 (boss mort au tout premier tick, avant `room_boss_slot_system`, tirage non
assigné) **corrigé** : `boss_drop_system` assigne lui-même (même fonction idempotente). Cette dernière correction n'a pas eu de 3e
relecture ; elle est couverte par la suite complète (535 scénarios) mais aucun scénario ne tue le boss au tick 0.

## 6. Journal (à ajouter sur `main` après le merge)

`| 2026-10-17 | M3-T8 (pools par type de salle à épuisement tirés à la génération de l'étage (option A, PoolState{seen,assigned}), marché du diable payé en vie max ou en âme (Price avec soul, modificateur permanent MaxHealth), butin du boss, salle du diable d'isaac remplie, prompt « — 1 cœur » ; Q5 et Q6 ; §73) | m3-t8-pools-marche-diable | b3 | mergée ; 535 scénarios, test_crates 1 115, lint/gen ×5, fuzz, revue (3 P2 corrigés), p2p-stress isaac 9 166 frames resimulées ; 17 traces isaac bénies avec preuve §5 ; non fait : chance du diable réduite par les dégâts au boss, boutique isaac (T11d), décision du diable par le bot (T14) |`
