# m2-t10-grammaire-etage — grammaire d'étage (M2, V1d, chantier E2)

Lire d'abord `docs/taches/README.md` (machine `orca`, `docs/taches.md` §7 « Exécution »),
`CLAUDE.md`, `docs/conventions.md` (§1 LDtk, §35 salles), le rapport du prototype
`docs/taches/rapports/m1-proto-etage-salles.md` (§4, §6) et celui de `m2-t0d-squelette-gungeon`.
Branche `m2-t10-grammaire-etage` depuis `main` **après le merge de T0c/T0d** (orch le dit ; en
attendant, partir de `origin/m2-t0d-squelette-gungeon`). Estimation : 5 à 7 j, livrable en deux
étapes.

## Contexte

L'assembleur `Basic` (`crates/map/src/generation/imp/basic.rs`) enchaîne des gabarits LDtk par
leurs connexions jusqu'à `max_room` ; ses défauts D45-D47 (choix toujours le dernier,
chevauchement, appariement du spawn) sont **déjà corrigés** (`m1-assembleur-d45-d47`). Il ne sait
rien des types de salle : l'étage de `gungeon` (T0d) est une chaîne fixe de trois salles écrite par
script. Gungeon veut un graphe : départ, salles de combat en branches, une récompense, une
boutique, un boss au bout, parfois un secret.

## Étape 1 — la grammaire (livrer seule si elle tient)

1. Nouveau mode `MapGenerationMode::Floor(FloorGrammar)` (RON, dans le manifeste ou la séquence
   `Floors`), à côté de `Basic` et `Cave` ; `Basic` reste **inchangé** (aucune trace de
   `zombies`/testbed ne bouge).
2. `FloorGrammar` : nombre de salles (`rooms: (min, max)`), exigences par `room_kind`
   (`required: { "boss": 1, "boutique": 1, "recompense": 1 }`), contraintes
   (`boss` à distance maximale du départ en salles, `boss` cul-de-sac à une seule porte, distance
   minimale départ → boss, nombre de branches/culs-de-sac), gabarits tirés par type parmi ceux du
   dossier (le `room_kind` du niveau LDtk).
3. Algorithme déterministe (le `RollbackRng` de génération, graine de la run) : construire un
   graphe qui satisfait la grammaire, le plaquer sur les gabarits par leurs connexions, sans
   chevauchement ; en cas d'échec, nouvelle tentative avec une graine dérivée, bornée (journaliser
   le nombre de tentatives, échec explicite au-delà).
4. Tests unitaires sur **1 000 graines** : connexité, chaque type requis présent, boss à distance
   maximale et cul-de-sac, aucun chevauchement, nombre de salles dans les bornes, temps de
   génération (rapport : moyenne, max).
5. Lint : grammaire impossible (type requis sans gabarit, bornes incohérentes) ; fixtures.

## Étape 2 — dans le jeu

6. `games/gungeon` : huit à dix gabarits (2-3 combat, récompense, boutique (vide pour l'instant :
   T6), boss, départ ; script de génération LDtk gardé dans `docs/taches/rapports/`), l'étage de
   T0d remplacé par la grammaire.
7. Plusieurs étages par run : `Floors` dont chaque niveau est un étage assemblé (graine dérivée
   par étage, numérotation des net ids triée par contenu comme en T1.8), portail ou escalier dans
   la salle du boss après sa mort (`victory_at_end` sur le dernier).
8. Mesure de perf d'un étage de 15 à 20 salles : temps de chargement, fps headless avec 2 joueurs
   (scénario `bench_floor`, plancher dans `tests/budgets.ron`), coût d'une reconstruction du champ
   de flux au verrouillage d'une salle (§6 du rapport du prototype).
9. Scénarios : `gungeon_floor_seed_<n>` (2 ou 3 graines : l'étage se charge, le joueur atteint la
   salle du boss par un script), `gungeon_two_floors` (étage 2 atteint, `FloorIndex`).

## Hors périmètre

Minicarte (présentation, V4), salles secrètes, récompenses `OnRoomClear` (T3/T5), boutique (T6),
bot explorateur (T16).

## Vérification

README §4 ; `Basic` intact (traces `zombies` et testbed identiques) ; p2p à deux clients sur
`gungeon` si le port 3536 est libre (prévenir orch) ; rapport
`docs/taches/rapports/m2-t10-grammaire-etage.md`, conventions : section « Grammaire d'étage ».
