# Rapport — m1-v3-bots-lead (anticiper la cible) : règle non livrée

Branche partie de `origin/main` ed8a274 (m1-v3-bots-armes mergée). **Rapport seul** : la règle
n'améliore pas la médiane des balles perdues (critère de la fiche, décision 2) ; son code est retiré
(décision d'orch : (a), branche avec le rapport seul, mergée pour garder la trace).

## État en cours

Fini. Deux variantes mesurées, aucune livrée. Code conservé sur la branche locale d'archive
`m1-v3-bots-lead-essai` (8f42009, variante 2 ; la variante 1 est son parent bd4dad3), non poussée.
Rien à compiler ni à bénir : la branche ne touche que ce rapport. Target purgé.

## Ce qui a été essayé (`crates/bots`, `prudent` et `fonceur`, tous modes)

`arms::lead_point(cible, vitesse, distance, vitesse_balle)` = `cible + vitesse × min(d / vb,
plafond)` (une itération) ; vitesse du projectile lue dans le mode de tir de l'arme en main ; pas
d'anticipation sur un ennemi immobile ni au-delà de la portée ; repli sur l'ennemi si un mur coupe
la ligne vers le point anticipé (`walls_clear`) ; `decide` vise `BotView::aim`. Tests unitaires
(point anticipé, plafond, repli, déterminisme) verts dans les deux variantes.

- **Variante 1** : vitesse de l'ennemi = `Velocity::main` (vitesse voulue par l'IA, lue sans
  écriture), plafond 1 s.
- **Variante 2** (décision d'orch, une seule fois) : vitesse = **déplacement réel** entre deux
  relevés (`EnemyTracks`, `GgrsNetId` → position et frame de simulation, hors rollback, vidée à
  l'entrée en partie, test de déterminisme), plafond 0,5 s.

## Mesures

Même base (`main` ed8a274 = la branche armes livrée, aucune différence de crate), mêmes commandes
qu'en m1-v3-bots-armes, une sim à la fois ; balles perdues au troisième étage par la méthode du
digest (trace détaillée, un dump à la fois, au moins 8 Go disponibles avant chacun).

| | avant (main) | variante 1 | variante 2 |
|---|---|---|---|
| balles perdues, graine 25 | 56 % | 62 % (défaite) | 58 % |
| graine 103 | 57 % | 59 % | 43 % |
| graine 131 | 61 % | 61 % | 57 % |
| **médiane** | **57 %** | **61 %** | **57 %** |
| throne 2 bots, 1..20 : étage 3 / défaites | 20/20 / 0 | 18/20 / 2 (11, 19) | 20/20 / 0 |
| 2 bots : frames moyennes | 3501 | 3481 | 3398 |
| throne 4 bots, 1..20 : étage 3 / desync | 20/20 / 0 | 20/20 / 0 | 20/20 / 0 |
| 4 bots : frames moyennes | 2358 | 2358 (toutes les graines changent) | 2266 |
| zombies, 4 acheteurs (règle sans effet sur eux, témoin) | 20/20 | identique | — |

- Variante 1 : **pire** (médiane 57 → 61 %, deux défaites de plus à 2 bots).
- Variante 2 : médiane **inchangée** (57 → 57 %), moyenne 58 → 53 % (graine 103 : 57 → 43 %), runs
  plus courts, aucune défaite. Le critère « médiane en baisse nette » n'est pas atteint : règle non
  livrée, comme le prévoit la fiche.

## Hypothèses (non prouvées)

- Variante 1 : `Velocity::main` est la vitesse **voulue** par l'IA : un ennemi qui pousse contre
  un mur, contourne ou encaisse un recul est anticipé là où il n'ira pas ; avec un temps de vol
  jusqu'à 1 s (300 px, balles à 300 px/s), l'erreur d'anticipation dépasse la largeur de la cible.
- Variante 2 : le déplacement réel corrige ce biais (la graine 103 gagne 14 points), mais les
  ennemis de `throne` changent souvent de cap (ils chassent un joueur qui bouge, contournent la
  roche) : sur trois graines, le gain n'est pas régulier. Trois graines donnent une médiane fragile ;
  une mesure sur plus de graines (code d'archive prêt) pourrait départager, si l'orchestrateur le
  juge utile.
- Ce qui reste perdu sans anticipation : la dispersion de la mitraillette (à 250 px, la gerbe de
  ±19 px couvre ≈ 64 % d'une cible de ±12 px, m1-v3-bots-armes) et les cibles qui changent de cap.
