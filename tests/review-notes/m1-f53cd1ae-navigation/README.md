# Recul près des murs — 2026-10-10

Le pillard 220 de la run humaine (graine 123456, étage 2) fuit après f1902.
À f2325, le rejeu historique le plaçait à (581.99936,64.0485), avec un sprite
qui débordait de 6 px dans la roche. Après correction, il se stabilise à
(575.96277,64.99998) : son sprite reste hors des deux murs. La fuite ne doit
pas devenir une poursuite ; un ennemi blessé peut rester dans une cellule sûre.
La sonde et le test `throne_recorded_pillard_leaves_the_wall_corner` retiennent
le timing historique des mutations pour isoler le changement de navigation.

Le recul utilise maintenant les voisins réellement franchissables, les mêmes
restrictions de diagonales/chicanes que la poursuite et le centre sûr du corps.
Un maximum local recentre le corps plutôt que de pousser directement dans un mur.
Quatre tests unitaires couvrent diagonales, chicane, coin enregistré et arrêt.
Les 109 tests unitaires game passent ; la régression de la run humaine passe.

22 scénarios concernés ont été rejoués avec synctest (distance 2). Huit traces
restent identiques ; les 14 autres ont été bénies après contrôle des attentes.
`trace-changes.json` donne le premier changement et les longueurs avant/après.
`first-components.json` compare le premier état détaillé des cas de fuite et
maintien de distance : seuls vitesse/position (et orientation du kiter) changent.
Le pillard généré conserve ses 599 frames à l'identique.

Trois attentes de scénarios throne évoluent avec le comportement corrigé :

- `throne_progression` : entrée étage 3 à f1572, alerte à f2472 ; la durée
  augmente de 2400 à 2500 frames pour conserver le contrôle de cette alerte.
- `throne_solo` : même entrée à f1572, vivant à f2699 dans la caverne du boss,
  puis défaite à f2779. Le bot ne survit plus jusqu'à f3699 sous le timing
  historique. Cette différence est consignée, sans rééquilibrage de contenu.
- `throne_three_floors` : le duo termine à f2783, reste vivant à f4999 ; ses
  deux mutations mesurées sont Sang froid et Chasseur (dernier choix f3908).

Ces dates concernent le recul avec mutations immédiates. Le correctif séparé
« mutations entre étages » modifie ensuite les timings ; ses preuves sont séparées.
La validation humaine du coin vu par William reste à faire lors de la seconde run.
