# m2-t0b-contrats-objets — contrats d'objets de M2 (`gungeon`), vague 0

Lire d'abord `docs/taches/README.md` (machine `orca` : worktree par `/home/debian/orca/task-new.sh`,
voir `docs/taches.md` §7 « Exécution »), `CLAUDE.md`, `docs/conventions.md` §4 (checklist d'un
vocabulaire) et les sections citées. Branche `m2-t0b-contrats-objets` depuis `main`. Estimation :
2 à 3 j. Plan : `docs/plan-engine.md` §5 C2 (objets et inventaire), C4 (butin) ; `docs/taches.md` §7.

## But

Poser les **contrats** dont dépendent M2-T3 (effets v2), M2-T4 (objets et synergies), M2-T5 (butin,
coffres, clés), M2-T6 (boutique) et M2-T1 (blank), sans encore les remplir : types, composants
rollback, kinds au registre, lint, attentes, un chemin minimal de bout en bout dans le testbed.
**Aucune trace existante ne bouge** (sauf l'élargissement de l'input, point 1, qui doit lui aussi
laisser les traces identiques).

## Décisions (fixées)

1. **Input élargi** : les 16 bits de `u16` sont tous pris (`crates/combat/src/actors.rs`,
   bits 13–15 = choix de mutation). Passer les boutons en `u32` et ajouter `INPUT_USE_ACTIVE`
   (bit 16) et `INPUT_BLANK` (bit 17) ; touches (présentation) : Espace = actif, Q = blank ;
   noms dans le format des scénarios (`replay.rs`, `UseActive`, `Blank`). Preuve : toutes les
   traces identiques, enregistrement → rejeu identique (`recording_replays_identically`), p2p à
   deux clients si possible (sinon « non vérifié ici », orch le rejoue). Dire dans le rapport la
   taille d'input GGRS avant/après.
2. **Crate `items`** (simulation, `bevy` en `default-features = false`) : `ItemDef` en RON
   (`id`, `kind: Passive | Active(charge) | Consumable`, `modifiers: [Modifier]` de `sim_core`,
   `effects: [Effect]` de `effects` (T1.10, réutiliser `Effect { on, if, do }` tel quel),
   `tags`, `rarity`), `ActiveCharge` = `Rooms(n)` (salles nettoyées, branché sur
   `world::rooms::RoomChanged` de M2-E1) | `Damage(n)` | `Frames(n)`.
3. **`Inventory`** (composant rollback du joueur, **neutre** au checksum : contribution 0 sans
   porteur) : passifs (liste triée par ordre de ramassage), un emplacement actif + sa charge,
   consommables par id (`BTreeMap<String, u32>` : `key`, `blank`, `shell`…). Les modificateurs
   d'un passif passent par `Modifiers` (source `item:<id>`, comme `powerup:<id>`).
4. **Ramassage générique** : `ItemPickup { item_id }` posé au sol (rollback, `GgrsNetId`),
   ramassé au contact (consommables) ou par Interaction (passif/actif, remplace l'actif tenu qui
   tombe au sol) ; `despawn_rollback()`. Spawn par scénario (champ `items`, comme `powerups`).
5. **Kinds** : `Item` au manifeste/registre (`content`), lint : id dupliqué, effet ou modificateur
   inconnu, charge invalide ; fixtures dans `crates/content/tests/fixtures/`.
6. **Attentes** : `HasItem(player, id)`, `ItemCharge(player, n)`, `Consumable(player, id, n)` ;
   moment clé `item_pickup` dans `crates/scenario/src/events.rs`.
7. **Bout en bout minimal (testbed)** : un passif (+vitesse), un actif `Rooms(1)` qui pose un
   modificateur temporaire, un consommable `key`. Scénarios : ramasser le passif → `Stat` change ;
   actif chargé par une salle nettoyée (carte de M2-E1) puis utilisé par `UseActive` ;
   deux joueurs qui ramassent le même frame (départage par `GgrsNetId`).
8. **Hors périmètre** : synergies, coffres, boutique, effet du blank (juste le compteur et
   l'input), HUD (V4).

## Vérification

README §4 complet, plus `fuzz_inputs` (`ALACOD_FUZZ=0:8`) car l'input change. Rapport
`docs/taches/rapports/m2-t0b-contrats-objets.md` (§7), conventions : section « Objets » (§36) et
liste des attentes de `CLAUDE.md` mise à jour (bloc « Attentes », une ligne M2).
