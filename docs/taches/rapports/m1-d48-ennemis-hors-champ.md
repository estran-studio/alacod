# m1-d48-ennemis-hors-champ — un ennemi n'est jamais hors de son propre champ de flux (D48) ; `alacod-sim --log` (D49)

## État en cours

- Fait : diagnostic (43, 162), correction des apparitions et du portail (ajout d'orch : graine 53),
  `--log`, tests, suite complète des scénarios **sans aucune trace changée**, rejeux 43, 162 et
  53 finis avec le binaire de la branche, 20 graines JSON identiques sans `--log`, lint mesuré.
- Reste : tests des crates (en cours), `make gen` des trois jeux, scripts, exemples, merge
  d'origin/main, purge.

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
