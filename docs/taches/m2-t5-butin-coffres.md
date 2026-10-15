# m2-t5-butin-coffres — butin, coffres et clés (M2, V1b, chantier C4)

Lire d'abord `docs/taches/README.md` (machine `orca`, §1 : `make sweep`, `make test_crates`,
signaling par session), `CLAUDE.md`, `docs/conventions.md` §36 (objets, `ItemPickup`, rareté,
consommable `key`), §44 (synergies, consommables à effets), §42 (effets v2, `OnRoomClear`), §35
(salles, `RoomChanged`), §40 (étages assemblés, graine par emplacement), §14 (power-ups, drop à la
mort, `powerup_drop_chance_override`), et le rapport `m2-t4-objets-synergies`. Branche
`m2-t5-butin-coffres` depuis `main`. Estimation : 3 j. Section de conventions réservée : **§48**
(§45–47 réservés aux tâches cloud M2-T7, M2-T11, M2-T16).

## Contexte

Les objets existent (passifs, actifs, consommables, synergies) mais ne sont posés que par la carte
ou par un scénario (`items:`). Il manque d'où ils viennent : un tirage pondéré par rareté, seedé et
déterministe, des coffres (dont certains fermés à clé), et une récompense à la fin d'une salle.

## À faire

1. **Pools de butin** : kind de contenu `LootTable` (`loot/<id>.ron`) : entrées `(item | rarity,
   weight)`, poids par rareté, exclusions (objets déjà tenus par un joueur, actif si on en tient
   un : à décider, le dire au rapport). Tirage par un flux RNG **dédié** (`"loot"`) dérivé de la
   graine de l'étage et de l'emplacement de salle (§40), consommé dans un ordre déterministe : un
   coffre ouvert plus tôt ou plus tard donne le même objet (tirage à la génération ou indexé par
   salle, pas par frame). Table par étage (`Floors` : liste de tables par index d'étage, défaut
   une seule).
2. **Coffres** : entité `Chest` (LDtk ou posée par la salle), rareté du coffre (donnée), `locked`
   (consomme une `key` à l'ouverture, Interaction ; refus sans clé, prompt HUD « Fermé — 1 clé »).
   Ouverture → `ItemPickup` au sol à côté (`GgrsNetId` par clé de contenu), coffre vidé (état
   rollback **neutre**), `FrameEvents<ChestOpened>` (neutre).
3. **Récompense de salle** : une salle `combat` passée `Cleared` pose, selon une chance en donnée
   (par table), un consommable (clé, blank, munitions) ou un coffre au centre de la salle ; tirage
   par le même flux.
4. **Clés** : les clés tombent comme récompense ; un coffre verrouillé par étage au moins dans la
   grammaire `etage` si elle reste valide sur 1 000 graines (test de M2-T10).
5. **Lint** : table vide, poids ≤ 0, objet inconnu, coffre verrouillé dans un jeu sans `key` →
   refusés ; fixtures.
6. **Contenu `gungeon`** : tables `commun`, `salle`, `boss` (le boss en tirera une fois M2-T7
   mergé : prévoir la table seulement) ; coffres dans deux gabarits de salles.
7. **Scénarios** : `loot_seeded` (même graine → même objet, deux graines → objets différents,
   `HasItem` après ouverture), `chest_locked` (refus sans clé, ouverture avec, `Consumable` key
   décrémenté), `room_reward` (récompense à la sortie de salle, forcée par une chance à 1 dans un
   override de scénario comme `powerup_drop_chance_override`), `loot_floor_table` (table de
   l'étage 2 différente).

## Règles

Aucune trace existante ne doit bouger hors des cartes/gabarits modifiés (nouveaux types en variante
**neutre**, variantes en fin d'enum, champs optionnels hors hash quand absents) ; toute trace qui
bouge : preuve §5 (`ALACOD_DUMP_TRACE` main / branche + `trace-diff.py`), cause dans le commit,
avant tout bless. RNG uniquement via un flux `RollbackRng` dédié, jamais celui du gameplay courant
(sinon tout le reste de la partie bouge). Pas de `f32`, `order_iter!`, `FrameEvents` (jamais
`Message`), BTreeMap. `cargo fmt --all -- --check` avant de livrer.

## Hors périmètre

Boutique et monnaie `shells` (M2-T6), butin du boss (branché quand M2-T7 est mergée), méta-déblocages
qui enrichissent les pools (M2-T12), HUD des clés (M2-T18).

## Vérification

README §4 (`make test_scenarios`, `make test_crates`, lint ×4, fmt, scripts, `make gen` ×4) ;
p2p à deux clients sur `gungeon` (port 3556, `MATCHBOX_PORT`) ; rapport
`docs/taches/rapports/m2-t5-butin-coffres.md`, conventions §48. Livrer par
`LIVRÉ m2-t5-butin-coffres <sha>`.
