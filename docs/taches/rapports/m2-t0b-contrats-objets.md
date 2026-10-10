# Rapport m2-t0b-contrats-objets — contrats d'objets (M2-T0b, chantier C2)

Branche `m2-t0b-contrats-objets` (base `8a9eb2e` + merge de `origin/m2-e1-salles` pour `RoomChanged`),
b1. Conventions : `docs/conventions.md` §36.

## 1. Fait

1. **Input u16 → u32** : `BoxInput.buttons` en `u32`, `INPUT_USE_ACTIVE` (bit 16), `INPUT_BLANK`
   (bit 17) ; touches Espace / Q (`PlayerAction::UseActive/Blank`) ; boutons de scénario `UseActive`,
   `Blank` (`replay.rs`). Taille de `BoxInput` : **8 → 12 octets** en mémoire (u16+i16+i16+2 bool →
   u32+i16+i16+2 bool, alignement 4). Les enregistrements ne stockent que des noms de boutons.
2. **Crate `items`** : `ItemDef` (`kind` Passive | Active(ActiveCharge) | Consumable, `modifiers`
   (stat, op, valeur), `effects: [effects::Effect]` réutilisés tels quels, `tags`, `rarity`,
   `pickup_range`), `ActiveCharge` Rooms | Damage | Frames, `Inventory`, `ActiveSlot`, `ItemPickup`,
   `ItemTable`, `ItemPicked`.
   *Écart à la fiche* : `modifiers` est un `ItemModifier { stat, op, value }` et non un `Modifier`
   de `sim_core` (qui exige `source` et `until` : la source est `item:<id>`, imposée par le moteur).
3. **`Inventory`** : composant rollback **neutre**, posé au premier ramassage seulement (jamais sur un
   joueur qui n'a rien ramassé : contribution 0 au checksum).
4. **Ramassage** (`game::items`) : `ItemPickup` au sol (rollback neutre, `GgrsNetId`, spawn par
   `spawn_item_pickup` / champ de scénario `items`) ; consommable au contact (plus petit `GgrsNetId`
   à portée) ; passif/actif par Interaction (`InteractionType::Item`), un actif remplace l'actif tenu
   qui tombe au sol ; `despawn_rollback()`. Charge `Rooms`/`Damage`/`Frames` (`item_charge_system`,
   après `Run` pour lire `RoomChanged` de la frame), usage `UseActive` (`TimedModifier`, `Modifier`).
5. **Kind `Item`** (`items/*.ron` du dossier déclaré) + lint : `DuplicateId`, `Parse` (stat inconnue),
   `OutOfRange` (charge 0, `pickup_range` ≤ 0, `TimedModifier frames: 0`), `Unsupported` (tout effet
   hors `OnUse` d'un actif, action autre que `TimedModifier`/`Modifier`, consommable avec modificateurs
   ou effets). Fixtures `item_charge_zero`, `item_effect_unsupported`, `item_unknown_stat`.
6. **Attentes** `HasItem`, `ItemCharge`, `Consumable` ; moment clé `item_pickup` ; test
   `objets_attentes` (échecs attendus).
7. **Bout en bout (testbed)** : `objets/bottes` (+25 % vitesse), `fiole` (`Active(Rooms(1))`,
   +100 % dégâts 300 frames), `key`. Scénarios `item_passive` (vitesse 150 → 187,5), `item_active_rooms`
   (charge 0 → 1 à la salle nettoyée, `UseActive` : charge 0 et dégâts ×2 puis retour à ×1 à
   l'expiration), `item_consumable_race` (joueur 0 prend la clé commune, joueur 1 la sienne),
   `item_passive_race` (même frame d'interaction : joueur 0).

## 2. Vérifié

- `make test_scenarios` complet **sans BLESS** : vert, EXIT 0, aucune trace existante modifiée
  (`git status` : seules 4 traces nouvelles) — y compris après l'élargissement de l'input.
- `cargo test` run, combat, game, content, map_ldtk, sim_core, stats, bots, effects, world, items :
  521 réussis, 0 échec, 1 ignoré. `cargo test` scenario (lib + expectations, cave, cave_floors,
  determinism, floors, generated, hunter_doors, placement, run, softlock, spawn_stall, bots) : 87
  réussis, 0 échec. (`recording_replays_identically` fait partie de `make test_scenarios`.)
- `make lint` sans erreur sur les 3 jeux ; `check-rollback-registration` OK ; `check-forbidden` : 4
  avertissements préexistants ; `cargo fmt --all -- --check` propre.
- `fuzz_inputs` (`ALACOD_FUZZ=0:8`, test `#[ignore]` : `-- --ignored`) : voir §5.

## 3. Non fait / incertain

- p2p à deux clients et bench au calme : non vérifiés ici (orch les rejoue). L'input GGRS change de
  taille : à vérifier en p2p réel.
- Effet du blank (compteur et input seulement), synergies, coffres, boutique, HUD : hors périmètre.
- L'usage d'un actif ne gère que `TimedModifier`/`Modifier` ; `Damage(n)` est couverte par la logique
  mais sans scénario dédié.
- Pas de prompt HUD pour `InteractionType::Item` (chaîne vide).

## 4. Dettes / questions

- Les traces de `rooms_*` (E1) ont été blessées avant movement-feel mais passent sur l'état fusionné
  (190 scénarios verts).
- `Inventory` inséré au premier ramassage : un futur système qui lit l'inventaire doit le traiter en
  `Option`.
- Deux fichiers `rustc-ice-*.txt` hérités du merge d'E1 retirés ici (orch les retire aussi sur
  `m2-e1-salles`).
- Disque : un binaire de test de `scenario` pèse ~1 Go ; purger les générations après chaque suite.

## 5. Fuzz

`ALACOD_FUZZ=0:8 cargo test --profile headless -p scenario --test fuzz_inputs -- --ignored` : 1 réussi, 0 échec (1603 s), input élargi inclus.
