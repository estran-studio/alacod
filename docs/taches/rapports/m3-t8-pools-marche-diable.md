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

## 2. Vérifié

(à compléter)

## 3. Traces bougées

Preuve §5 (dumps détaillés `ALACOD_DUMP_TRACE`, trois états : `main` `ad2be1c2`, branche complète, branche **sans** `boss_drop` ni
marché du diable) sur les 16 traces qui différaient, comparées par `scripts/trace-diff.py --ignore PoolState,Chest` :

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

## 5. Relecture indépendante

(à compléter)
