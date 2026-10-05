# m1-d48-ennemis-hors-champ — un ennemi n'est jamais hors de son propre champ de flux (D48) ; `alacod-sim --log` (D49)

## État en cours

- Fait et vérifié (ci-dessous). Branche depuis `f80b82b`, merge d'origin/main `775cb8a` ; target
  purgé après la suite. **Rien de béni** : `throne_defaite_boss` est remplacé (graine 25, plus bas), sa trace est à bénir.

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

## Correction

- **`world::nav`** (nouveau) : `blocked_for`, `too_narrow`, `impassable`, `diagonal_cuts_corner`
  — les règles de passage du champ de flux, **une seule définition** : `FlowFieldCache::
  is_blocked_for`, `is_too_narrow` et la règle de la diagonale de `build_flow_field` les appellent
  (`game`). `nav_distances(grid, sources, large)` : le même parcours sur une grille de caverne
  (8-connexité, roche et bordure bloquent, sources comptées même bloquées).
- **Points d'ennemis** : `points_of_interest(…, enemy_large)` ne garde que les points atteints
  par `nav_distances` depuis les points des joueurs pour le gabarit `CaveConfig::nav_large`
  (calculé par le registre : collider × `scale` > 20 px en largeur ou en hauteur, le seuil
  d'`AgentSize::Large` ; sérialisé seulement s'il est vrai). L'ordre des candidats ne change pas
  (distance de grille) : seules les cavernes où un point était hors champ changent.
- **Portail** (ajout d'orch, graine 53 : ancre (456, 316) dans un mur) :
  `map_ldtk::game::floors::portal_anchor` — barycentre des `PlayerSpawn` comme avant ; sur une
  caverne, si sa case n'est pas atteinte par le champ du gabarit des joueurs (petit), centre de
  la case atteinte la plus proche (`world::nav::nearest_reached_cell`). Inchangé sinon et hors
  caverne. Premier niveau et transitions.
- **Lint** (`lint_caves`) : une caverne peuplée dont l'une des graines de contrôle 1 à 5 n'a aucun
  point atteignable est refusée (`OutOfRange`), fixture `cave_spawns_unreachable`. Coût : `alacod
  lint` 27 ms (zombies), 17 ms (testbed), 13 ms (throne, trois cavernes × 5 graines) sur la
  branche ; une génération + points de caverne coûte environ 1 ms (3 000 cavernes en 2,9 s dans
  le test du portail), soit au plus ~15 ms de plus pour throne. (Pas de mesure « avant » propre :
  ma tentative par `git stash` a mesuré le dernier commit, pas `main` ; écartée.)
- **D49** : `alacod-sim --log` installe un `tracing_subscriber` sur stderr (`RUST_LOG`, défaut
  `info`, sans horodatage ni couleur). `tracing-subscriber` ajouté à `scenario` (même version
  que `utils`, aucune crate nouvelle). Doc : en-tête du binaire, conventions §21 (D48) et §24 (D49).
- Pas de clamp du déplacement (décision 3 de la fiche) : aucun cas prouvé.

## Tests

- `world::nav` : couloir d'une case qui coupe une poche (petit gabarit), gabarit grand qui
  exige les 8 voisines, portail ramené sur une case atteinte.
- `game` `cave_nav_tests` : **cohérence** — sur les trois cavernes de throne × 20 graines × deux
  gabarits, le champ de flux réel (`build_flow_field`) et `nav_distances` atteignent exactement
  les mêmes cases.
- `content` `points_ennemis_des_cavernes_de_throne_dans_le_champ_de_chaque_gabarit` : 1 000
  graines × 3 cavernes, chaque point d'ennemi atteint pour le gabarit de chaque personnage ;
  `nav_large` vrai pour `niveau_3` seulement (boss).
- `map_ldtk` `ancre_du_portail_atteinte_sur_les_cavernes_de_throne` : 1 000 graines × 3
  cavernes, ancre sur une case atteinte ; 27 ancres ramenées sur 3 000.
- Fixture `cave_spawns_unreachable` (lint). L'automate ne produit pas de caverne fermée au seul
  gabarit grand sur les graines de contrôle (balayage de tailles 20 à 32, remplissages 0,40 à
  0,60, trois couples de seuils : aucune) : la fixture ferme la caverne entière ; l'exclusion
  propre à la navigation est couverte par les tests de `world::nav` et les 1 000 graines.

## Vérifié

- Rejeux avec `alacod-sim` de la branche (`--floors run`, 2 bots `prudent`, 15 000 frames) :
  graines **43, 162 et 53 finissent les trois étages** (f5215, f4910, f5366), contre trois
  soft-locks avec `f80b82b`.
- Graines touchées sur 1..200 (balayage temporaire, ancienne règle recopiée) : `niveau_1` 1
  (portail, 95), `niveau_2` 2 (portail, 53 et 145), `niveau_3` 53 (points d'ennemis, gabarit grand
  du boss : 7, 13, 14, 19, **23**, 26, 27, 30, 31, 32, 33, 39, **43**, 44, 47, 54, 59, 64, 73,
  **76**, 80, 84, 87, 89, 92, 93, 95, 106, 108, 114, 115, 116, 117, 120, 121, 124, 132, 136, 138,
  144, 148, 152, 156, 159, **162**, 164, 168, 185, 186, 189, 191, 194 ; portail : **81**). Les
  soft-locks 23, 43, 76, 162 (ennemi hors champ) et 53, 81 (portail) des 200 graines en font partie.
- Suite complète des scénarios (avant merge) : **0 trace différente** (aucun scénario d'avant
  n'utilisait une graine touchée), 1 059 s. Après merge d'origin/main : `throne_defaite_tireurs`
  et `throne_defaite_coequipier` verts ; **`throne_defaite_boss` change** (ci-dessous).
- Tests des crates : 507 (workspace hors `scenario`) + 79 (`scenario` hors la suite), 0 échec.
- `make lint` des trois jeux sans erreur ; `cargo fmt --check` ; `check_forbidden` 4 (inchangé) ;
  `check_rollback_registration` OK ; exemples compilent ; `make gen` des trois jeux sans
  modification.
- `alacod-sim --log` : journaux du jeu sur stderr ; **sans `--log`, 20 graines zombies (vague 1,
  3 000 frames) JSON identiques** à `alacod-sim-f80b82b`.

## Preuve §10 : `throne_defaite_boss` (graine 124, figé par m1-analyse-200-throne)

- Première différence : ligne 2065 de la trace, frame 2064 (`d969d645…` → `fff30fcc…`) : le
  chargement du troisième étage (entrée f2065).
- Cause : la graine 124 est dans la liste ci-dessus (`niveau_3`, gabarit grand). Ennemis à f2070,
  case par case — main : arroseur (52, 3), brute (3, 4), buffle (54, 40), chien (11, 3), cracheur
  (11, 40), franc_tireur (60, 8), rat (3, 31), roi_rat (3, 39), tourelle (4, 12) ; branche :
  arroseur (52, 3), brute (60, 8), buffle (5, 6), chien (3, 31), cracheur (11, 40), franc_tireur
  (54, 40), rat (13, 3), roi_rat (3, 39), tourelle (59, 23). Les points (3, 4), (11, 3) et (4, 12)
  ne sont plus retenus (hors du champ du gabarit grand), remplacés par (5, 6), (13, 3) et
  (59, 23) ; l'attribution à tour de rôle décale les personnages.
- Effet : la défaite arrive à f3520 au lieu de f3676 ; l'attente `RunState(Playing, 3676)` et
  celles de la première mise à terre (relevées sur l'ancien placement) ne tiennent plus.
- Décision d'orch : ne pas rebénir la graine 124 (ses inputs jouent contre des apparitions qui
  n'existent plus), la remplacer par une graine de défaite « boss » **non touchée** par D48.

## `throne_defaite_boss` remplacé : graine 25

- Choix parmi les 12 rejeux de m1-analyse-200-throne : première mise à terre par le roi_rat
  pour 25, 31, 62, 124 et 144 ; 31, 124 et 144 sont touchées par D48, 25 et 62 non (ni points
  d'ennemis ni portail sur aucun des trois étages). 25 : le boss domine le plus nettement.
- Enregistrée avec `alacod-sim` de la branche (`--save-scenario`) : même issue qu'avec `61ac539`
  (`floor_frames` [932, 2921], défaite f4590, un mort).
- Relevé (trace détaillée de la branche, un rejeu) : troisième étage à f2921 ; joueur 0 à terre
  à **f3808** ; dégâts reçus dans les 300 frames d'avant : roi_rat 47, franc_tireur 12, tourelle
  8 ; joueur 1 seul, mort à f4591.
- Attentes : `FloorIndex(2, 3810)`, `PlayerDowned(0, 3810)`, `AmmoReserve` balles 432 et 432 à
  f3810, `EntityCount(Enemy, 5..5, 3810)`, `EntityHealth(roi_rat #750, 44, 3810)`,
  `RunState(Playing, 4590)`, `Defeat(4591)`, `RunState(Ended(Defeat), 4592)` : **toutes vertes**
  sur la branche ; la trace diffère dès la ligne 1 (autre graine) : à bénir par l'orchestrateur.

## Non fait

- Pas de mesure de lint « avant » propre (voir Correction).
