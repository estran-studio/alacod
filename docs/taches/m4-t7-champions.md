# m4-t7-champions — champions et élites v2 (M4, V1c, D2 v2)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/taches.md` §8.1, `docs/conventions.md` §18
(expressions), §25 (variantes), §48 (graine de butin), §62 (`Explode`) et §63 (contrats de M4-T0d). Branche
`m4-t7-champions` depuis `origin/main`, **après le merge de M4-T0d et M4-T0c**. Estimation : 2 j. Section de
conventions réservée : **§72**.

## À faire

1. **`on_death`** d'une variante exécuté à la mort de l'ennemi (même frame que `BossDefeated`/`Death`, ordre
   `GgrsNetId`) : `Explode`, division (`SpawnCharacters`, ids par clé de contenu `champion:<net_id>:<k>`),
   autres actions de l'effet permises par le lint.
2. **`behaviors`** d'une variante : remplacent ceux du personnage à la création.
3. **`loot`** : table tirée par `loot_seed` (clé `champion:<net_id>`), objet posé au sol.
4. **`tint`** : lu par la présentation seulement (T17 le dessine ; ici, champ disponible et attente nulle).
5. **`chance` en expression** (`players`, `floor`), valeur figée à l'apparition.
6. **Scénarios** (testbed) : `champion_explodes`, `champion_splits`, `champion_drops`, `champion_behaviors`,
   `champion_chance_floor` (étage 2 : plus de champions à graine égale).

## Règles

Un personnage sans `variants` ou dont les variantes n'ont aucun champ v2 garde exactement son chemin
(traces gungeon et throne identiques). RNG local de §25, jamais `RngStreams`.

## Hors périmètre

Contenu `isaac` (T11c), teinte dessinée (T17).

## Vérification

README §4 ; rapport `docs/taches/rapports/m4-t7-champions.md`, conventions §72.

## Atteignabilité

T11c donne des champions à au moins trois ennemis placés dans les salles `combat` de `isaac` ; graine nommée.

## Session cloud

Tâche prévue pour une session Claude Code cloud. Ignorer les consignes `orca` du README (§1). Pas de Docker :
le dire au rapport. Pousser la branche régulièrement. Voie rapide du README §6 : fusionner `origin/main` et
rejouer la suite juste avant le commit `LIVRÉ m4-t7-champions`, qui contient le rapport.
