# m4-t9-curses — curses d'étage (M4, V1e, F4)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md` (« Simulation et présentation »), `docs/taches.md` §8.1,
`docs/conventions.md` §15 et §56 (HUD), §17, §40, §64 (contrats de M4-T0e), §70 (grille). Branche
`m4-t9-curses` depuis `origin/main`, **après le merge de M4-T0e et M4-T5**, et la réponse de William à
**Q7** (liste des curses). Estimation : 3 j. Section de conventions réservée : **§74**.

## À faire

1. **Tirage** au début de chaque étage (`FloorEntered`) selon la règle pure de T0e ; `RunCurses` écrit ;
   trace `ggrs{f=… curse floor=… id=…}` ; moment clé `curse`.
2. **Effets** retenus en Q7 : ceux de simulation (`Modifier`, `FloorRooms(mult)` lu par la grammaire de
   l'étage suivant ou du même étage selon le moment du tirage : le dire) exécutés ici ; ceux de présentation
   (`HideMinimap`, `HideHealth`, `HideItemNames`) exposés au HUD par une source (`curses`) et lus par T16.
3. **Scénarios** : `curse_drawn_seeded` (même graine, même curse ; autre graine, autre ou aucune),
   `curse_modifier`, `curse_bigger_floor`, `curse_hides_minimap` (attente `HudText`).

## Règles

Une séquence `Floors` sans `curses` ne tire rien : traces de tous les jeux identiques. L'affichage dérive de
l'état (`RunCurses`), jamais d'un événement.

## Hors périmètre

Obscurité (visibilité dans la simulation : E8, hors M4 sauf décision), défis et modificateurs de run choisis
par le joueur (heat, Carême).

## Vérification

README §4 ; rapport `docs/taches/rapports/m4-t9-curses.md`, conventions §74.

## Atteignabilité

`curses` déclaré dans la séquence par défaut de `isaac` (T11d) ; graine nommée où une curse est tirée à
l'étage 1 ou 2.

## Session cloud

Tâche prévue pour une session Claude Code cloud. Ignorer les consignes `orca` du README (§1). Pas de Docker :
le dire au rapport. Pousser la branche régulièrement. Voie rapide du README §6 : fusionner `origin/main` et
rejouer la suite juste avant le commit `LIVRÉ m4-t9-curses`, qui contient le rapport.
