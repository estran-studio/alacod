# Rapport m2-t5-butin-coffres — butin, coffres et clés (M2, chantier C4)

Branche `m2-t5-butin-coffres` depuis `main` (406eec01). Conventions : §48.

## Livré

- **Pools de butin** : kind `LootTable` (`loot/<id>.ron`), entrées `Item(id)` / `Rarity(r)` pondérées,
  `exclude_held` (opt-in), `reward` (récompense de salle). `items::draw_loot` : tirage pur, deux `next_u32`
  toujours consommés, tout trié (BTreeMap).
- **Flux RNG dédié** : `items::loot_seed(graine, étage, clé)` + `RollbackRng` **local** ; aucun flux de
  `RngStreams` touché (le flux nommé `"loot"` des power-ups serait consommé à chaque mort). La clé est une clé
  de contenu, donc le moment d'ouverture, le joueur et les `Entity` ne comptent pas.
- **Pool par étage** : `Floors.loot` (liste par index d'étage, le dernier vaut ensuite) → `FloorLootTables`.
- **Coffres** : `items::Chest` (rollback neutre), `InteractionType::Chest`, verrouillé = une `key`, refus sans
  clé, objet posé à côté, `FrameEvents<ChestOpened>` (neutre), prompt HUD « Fermé — 1 clé ».
- **Coffre par type de salle** : `rooms/<id>.ron` `chest: Some(...)` ; `room_chest_system` (map_ldtk) le pose
  au centre de chaque salle de l'étage courant. `gungeon` : `recompense` (verrouillé) et `boutique` (ouvert).
- **Récompense de salle** : une salle `combat` `Cleared` laisse un coffre ou un consommable du pool (chance en
  donnée par pool, `room_reward_chance_override` pour un scénario).
- **Lint** : pool vide, poids ≤ 0, objet inconnu, rareté sans objet, chances hors 0..=1, coffre de salle à pool
  inconnu, coffre verrouillé sans consommable `key`, pool inconnu d'une séquence. 4 fixtures.
- **Contenu** : `gungeon` pools `commun`, `salle`, `boss` (non tiré, M2-T7) ; `testbed` pools `commun`,
  `etage2`, `salle` et séquence `deux_butin`.
- **Scénarios** : `loot_seeded`, `loot_seeded_other`, `chest_locked`, `room_reward`, `loot_floor_table`, et
  l'attente `ItemCount`.

## Choix et écarts

- **Exclusion des objets tenus** (« à décider ») : opt-in par pool (`exclude_held`), faux partout dans
  `gungeon`. Une exclusion par défaut ferait dépendre l'objet de l'inventaire de l'ouvreur, donc du moment :
  c'est le contraire du point clé de la fiche. Le choix existe pour un pool qui le veut explicitement.
- **Rareté d'un coffre** = rareté minimale de l'objet (repli sur le pool entier si rien ne passe).
- **Coffres « dans deux gabarits »** : posés par **type de salle** (`rooms/<id>.ron`), pas par une entité LDtk :
  ajouter une entité LDtk passe par l'extraction, l'assembleur d'étage et la transition d'étage (offsets), pour
  un résultat identique côté joueur. Tous les gabarits d'un type en reçoivent un. Les deux types choisis,
  `recompense` et `boutique`, sont ceux des gabarits de la grammaire `etage`/`grand` qui ne servent pas encore.
- **`world` dépend désormais de `items`** (`RoomKindDef::chest` utilise `items::RoomChestDef`) ; `RoomKindDef`
  perd `Copy`.
- **Boss** : la table `boss` existe et est lintée, rien ne la tire avant M2-T7.
- **Seed de scénario** : `RunSeed` = `map_seed` ; la position de départ d'un `Floors` dépend de la graine
  (assemblage `Basic`), d'où des coordonnées de coffre propres à chaque graine dans `loot_seeded*`.

## Vérifications

- `make test_crates` : vert (items 12, content 100, etc.).
- `make test_scenarios` : vert après fusion de `origin/main` (M2-T2, 3205ef14) ; `make lint` ×4 (zombies, testbed, throne, gungeon) vert ; `cargo fmt --all -- --check` propre ; `make gen` ×4 vert ; p2p gungeon 2 clients (port 3556) : « traces identiques 599 ».
- **Traces existantes déplacées : 9**, toutes bénies après preuve §5 : `bench_floor`, `effect_charges`, `effect_on_room_clear`, `gungeon_boss`, `gungeon_clear`, `gungeon_floor_seed_23`, `item_active_rooms`, `rooms_lock_coop`, `rooms_lock_solo`.
  - Cause pour 8 d'entre elles : `room_reward_system` pose un `ItemPickup` (clé ou ration) quand une salle `combat` passe en `Cleared` (+1 entité, `GgrsNetIdFactory` +1).
  - Cause pour `bench_floor` : `room_chest_system` pose 2 coffres (salles `recompense`/`boutique`) dès la frame 0 ; leurs ids (279, 280) suivent les ids existants.
  - Preuve : `ALACOD_DUMP_TRACE` sur `origin/main` (96b7b0aa) puis sur la branche, `scripts/trace-diff.py main.full branche.full --ignore "FrameEvents<items::loot::ChestOpened>"` (ressource neutre nouvelle) : la première frame divergente est la frame du `Cleared` (0 pour `bench_floor`), et les seules lignes en plus sont l'`ItemPickup` ou les `Chest`.
- Incident : le worktree `main-proof` (dumps `main`) a partagé le target et pollué les artefacts de la branche (erreur `RoomChestDef` introuvable) ; résolu en touchant les `lib.rs`. Disque : 44-58 Go libres pendant les suites (`make sweep` fait avant, -32 Go).
- À refaire avant LIVRÉ : fusion de `origin/main` avec M2-T7/M2-T11 (consigne de l'orchestrateur), puis nouvelle preuve §5 de `bench_floor` et des traces gungeon.
