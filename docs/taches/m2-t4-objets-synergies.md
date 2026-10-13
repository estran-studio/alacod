# m2-t4-objets-synergies — objets v2 et synergies (M2, V1b, chantier C2)

Lire d'abord `docs/taches/README.md` (machine `orca`, §1 : `make sweep`, `make test_crates`,
signaling par session), `CLAUDE.md`, `docs/conventions.md` §36 (objets et inventaire), §42 (effets
v2 : déclencheurs, charges, effets passifs lus dans l'`Inventory`), §41 (blank, consommables), et
les rapports `m2-t0b-contrats-objets` et `m2-t3-effets-v2`. Branche `m2-t4-objets-synergies`
depuis `main`. Estimation : 4 j. Section de conventions réservée : **§44** (§43 = esquive v2,
M2-T2 en cours chez b0).

## Contexte

T0b a posé l'inventaire (passifs, un actif chargé par `Rooms`/`Damage`/`Frames`, consommables) ;
M2-T3 exécute les `effects` des objets (passifs lus dans l'`Inventory`, limites par clé
`item:<id>:<index>`). Il manque : les **synergies** (une paire d'objets tenus ajoute un effet), un
actif réellement jouable dans `gungeon`, et les consommables autres que `blank`/`key`.

## À faire

1. **Synergies** : kind de contenu `Synergy` (`synergies/<id>.ron`) : `requires: [ids]` (deux
   objets ou plus, passifs ou actif), `modifiers` et `effects` (mêmes formes que `ItemDef`), nom.
   Active tant que **tous** les objets requis sont tenus, retirée dès qu'un manque (actif
   remplacé) : modificateurs sous la source `synergy:<id>`, effets sous `synergy:<id>:<index>`.
   État dérivé de l'`Inventory` à chaque frame (pas de nouveau composant si possible) ; sinon
   composant **neutre**. Ordre des synergies par id, des porteurs par `GgrsNetId`.
2. **Actifs v2** : un actif à usage limité par étage (`uses_per_floor`, donnée optionnelle) et un
   actif qui consomme un consommable (`cost: Some((item, n))`) ; sans ces champs, v1 exact.
3. **Consommables** : `Consumable` générique avec `effects` sur `OnUse` (ramassé au contact,
   utilisé… seulement si un bouton existe déjà ; sinon s'appliquant au ramassage, ex. soin, et le
   dire au rapport). Ne pas réserver de nouveau bit d'input sans accord d'orch.
4. **Lint** : synergie à objet inconnu, à un seul objet, doublon de paire (même ensemble requis),
   `uses_per_floor = 0`, `cost` vers un objet non consommable → refusés ; fixtures.
5. **Contenu `gungeon`** : deux synergies entre objets existants (`trefle`, `cartouchiere`,
   `amulette`, `etendard`…), un actif jouable (ex. bouclier temporaire, `Rooms(2)`), un
   consommable de soin.
6. **Scénarios** : `synergy_pair` (bonus présent avec les deux objets, absent avec un seul),
   `synergy_lost` (actif remplacé → synergie retirée, `Stat` revient), `item_active_per_floor`,
   `item_active_cost`, `consumable_heal`.

## Règles

Aucune trace existante ne doit bouger (nouveaux types en variante neutre ; nouveaux champs
optionnels hors hash quand absents, comme `Effect` en §42 ; nouvelles variantes en fin d'enum). Si
une bouge : preuve §5 (`ALACOD_DUMP_TRACE` main / branche + `trace-diff.py`) avant tout bless, cause
dans le commit. Pas de `f32`, `order_iter!`, `FrameEvents` (jamais `Message`), BTreeMap.
`cargo fmt --all -- --check` avant de livrer.

## Hors périmètre

Butin, coffres et tables de drop (M2-T5), boutique (M2-T6), HUD des objets (M2-T18), équilibrage.

## Vérification

README §4 (`make test_scenarios`, `make test_crates`, lint ×4, fmt, scripts, `make gen` ×4) ;
p2p à deux clients sur `gungeon` (port 3556, `MATCHBOX_PORT`) ; rapport
`docs/taches/rapports/m2-t4-objets-synergies.md`, conventions §44. Livrer par
`LIVRÉ m2-t4-objets-synergies <sha>`.
