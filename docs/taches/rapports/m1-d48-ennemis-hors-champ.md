# m1-d48-ennemis-hors-champ — un ennemi n'est jamais hors de son propre champ de flux (D48) ; `alacod-sim --log` (D49)

## État en cours

- Fait : diagnostic des graines 43 et 162 (rejeux `alacod-sim-f80b82b` sauvés, puis scénario
  avec trace détaillée tronquée, un rejeu à la fois) ; code écrit, **pas encore compilé**
  (attente du feu vert d'orch) : `world::nav` (règles partagées, `nav_distances`),
  `points_of_interest(…, enemy_large)`, `CaveConfig::nav_large` dérivé par le registre,
  `FlowFieldCache` branché sur `world::nav`, lint des graines de contrôle + fixture
  `cave_spawns_unreachable` (à calibrer), test 1 000 graines × 3 cavernes, test de cohérence
  champ/points, `alacod-sim --log`, doc §21 et §24.
- Prochaines étapes : compilation, tests, calibrage de la fixture, suite complète (traces
  throne qui bougent + preuve §10), rejeu des graines 43 et 162 avec le binaire de la branche,
  20 graines sans `--log` identiques.

## Diagnostic

Les deux ennemis **naissent** hors de leur champ ; aucun n'y entre en se déplaçant.

- **Graine 162** (`brute`, collider 20 × 20, gabarit petit) : apparaît au troisième étage
  (f2434) en (88, 56), case (5, 3) ; avance jusqu'en (149,6 ; 66), case (9, 4), et y reste
  (f2570 → fin). Champ `GroundBreaker/Small` à f2700 (`FlowFieldCache` de la trace) : aucune
  case de la poche (3–9, 2–5) n'y figure, alors que la grille la montre ouverte et reliée par
  la case (10, 4) ; cette case est un couloir d'une case (`wall_cells` en (10, 3) et (10, 5)) :
  `is_too_narrow_for` l'exclut du Dijkstra. Le point d'apparition (5, 3) est donc hors du champ
  dès la frame 0 de l'étage.
- **Graine 43** (`roi_rat`, 28 px en jeu, gabarit grand) : apparaît en (952, 88), case (59, 5),
  f2462 ; descend en (57, 4) et y reste. Champ à f2690 : (59, 5) et (57, 4) sont dans le champ
  `Small` mais pas dans le champ `Large` : la poche ne communique avec le reste que par un
  passage de deux cases (x 52–56), fermé au gabarit grand (`is_blocked_for` : 8 voisines).
- Ni la règle de dégagement local (D41, `is_open_within`) ni « case libre pour la clé » ne
  suffisaient : en 162 le point est libre, c'est la poche qui n'est pas reliée. La règle retenue
  est la **connexité de navigation** depuis les points des joueurs.
