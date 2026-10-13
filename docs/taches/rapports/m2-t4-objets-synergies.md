# Rapport m2-t4-objets-synergies — objets v2 et synergies (M2, chantier C2)

Branche `m2-t4-objets-synergies` depuis `main` (61cccdbf). Conventions : §44.

## Livré

- **Synergies** : kind `Synergy` (`synergies/<id>.ron`), `items::SynergyDef`/`SynergyTable`. Active tant que
  tous les objets requis (passifs ou actif) sont tenus. État **dérivé de l'`Inventory`** à chaque frame, sans
  composant : `synergy_modifiers_system` pose/retire les modificateurs `synergy:<id>` ; les effets sont ajoutés
  à la liste des effets du porteur sous la clé `synergy:<id>:<index>` (limites, `Tick`, source de modificateur
  comme un effet d'objet). Ordre : synergies par id, porteurs par `GgrsNetId`.
- **Actifs v2** : `uses_per_floor` (mémoire `ActiveSlot::uses = (étage, usages)`, aucun reset à écrire, hash
  manuel : hors hash tant que non posé) et `cost: (consommable, n)` (usage refusé sans stock, charge conservée).
  `OnUse` accepte `Heal`.
- **Consommables à effets** : `effects` `OnUse` appliqués **au ramassage** (aucun bouton d'usage pour un
  consommable générique, pas de nouveau bit d'input) ; le consommable n'entre pas dans l'inventaire.
- **Lint** : synergie à objet inconnu, à un seul objet, doublon d'ensemble, à un consommable, `OnUse` dans une
  synergie ; `uses_per_floor = 0`, `uses_per_floor`/`cost` hors actif, `cost` vers un objet inconnu ou non
  consommable. 5 fixtures.
- **Contenu `gungeon`** : synergies `tresor_de_poche` (trèfle + amulette, +10 vie max) et `tir_nourri`
  (cartouchière + étendard, +25 % cadence) ; actif `pierre_de_soin` (`Rooms(1)`, soin 20, 2 usages par étage) ;
  consommable `ration` (soin 15).
- **Scénarios** (testbed) : `synergy_pair`, `synergy_lost`, `item_active_per_floor`, `item_active_cost`,
  `consumable_heal`.

## Choix et écarts

- **Pas de bouclier temporaire** : `StatId::Armor` n'est lue par aucun système de dégâts (grep : seul le
  décrit d'effet la nomme). Un actif sur cette stat serait sans effet ; l'actif jouable de `gungeon` est donc une
  pierre de soin à usages limités. À reprendre avec une réduction de dégâts (hors périmètre).
- **Consommable à effets = effet au ramassage**, borné par la santé max (pas de refus à pleine santé) : dit au
  rapport comme la fiche le demande.
- **Cadre des scénarios** : les objets sont posés à `424,696` de `testbed/arena.ldtk` (comme `effect_on_blank`) ;
  le soin est mesuré sous la spirale de `tourelle_spirale` (santé 92 → 97 avec `ration` du testbed, soin 5, pour
  ne pas être borné par la santé max).
- Les effets d'une synergie sont lus à chaque frame dans `Inventory` (comme les effets d'objet de M2-T3) :
  une synergie perdue n'a rien à retirer côté effets ; ses modificateurs sont retirés par source.

## Vérifications

- `make test_crates` : 0 (tous verts). `make test_scenarios` (suite complète) : 0, 7712 s.
- Traces existantes : aucune ne bouge, aucun `BLESS` sur un scénario existant ; seules 5 traces nouvelles
  (`synergy_pair`, `synergy_lost`, `item_active_per_floor`, `item_active_cost`, `consumable_heal`).
- `make lint` ×4 (zombies, testbed, throne, gungeon) : aucune erreur. `cargo fmt --all -- --check` propre.
- `make gen` ×4 : exit 0, aucun fichier modifié.
- p2p à deux clients sur `gungeon` (port 3556, 600 frames) : traces identiques, 599 lignes.
