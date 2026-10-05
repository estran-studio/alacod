# m1-assembleur-d45-d47 — D45 (`.skip(r).last()`), D46 (chevauchement), D47 (appariement Spawn)

## État en cours

- Fait : code, tests, preuve §10, 20 graines avant/après. Livré pour avis (William : traces M0
  de référence) ; **rien de béni**, `avant_poste_demo.trace` à bénir si le changement est accepté.

## Correctifs (`crates/map`)

- **D45** : `.nth(r)` aux trois choix de `generation::imp::basic` (connexion libre, gabarit
  compatible, gabarit de départ). `.skip(r).last()` rendait toujours le dernier élément. Même
  nombre de tirages RNG : seul l'élu change.
- **D46** : avant `is_outside`, la nouvelle salle est testée contre les salles déjà placées ; un
  chevauchement ferme la connexion (`DeadEnd`). `Room::is_overlapping` passe en `<=` : avec `<`,
  deux salles voisines (bord commun, le cas de toute connexion) se chevauchaient.
- **D47** : saut des gabarits `Spawn` supprimé dans `populate_level_connections` (sans `ii += 1`,
  boucle infinie dès qu'un `Spawn` n'était pas premier ; asymétrique sinon). Les 31 `.ldtk` du dépôt
  ont un unique `Spawn` en tête : appariement inchangé, et désormais indépendant de l'ordre.
- `map_ldtk::GeneratedRoom` : champ `template` et `world_rect()` (hors simulation, pour les tests).

## Tests

- `map` : `chevauchement_exclut_les_salles_adjacentes` (D46), `appariement_independant_de_la_place_du_spawn` (D47, trois ordres).
- `map_ldtk` : `avant_poste_cartes_distinctes_sans_chevauchement` — avant_poste, graines 1..20,
  config du jeu (1000×1000, `max_room` 10) ; aucune paire de salles chevauchante, > 1 carte.
  `ALACOD_MAP_SIGNATURES=1 … -- --nocapture` imprime les gabarits par graine.

## Mesure : cartes distinctes (avant_poste, graines 1..20)

Signature = liste « gabarit@position relative à la salle de départ ».

| | cartes distinctes | chevauchements | salles par graine |
|---|---|---|---|
| main (`0e4125b`) | **7** | 0 | 4 à 9 |
| branche | **19** | 0 | 4 à 9 (mêmes comptes par graine) |

Les cartes variaient déjà avec la graine avant D45 (la salle à étendre est tirée correctement,
`rooms_possible.get(r)`, et la position de départ fait varier les sorties `OutSide`), mais chaque
connexion élisait toujours le dernier gabarit compatible : peu de formes. Journaux : `../d40/map_avant.log`, `../d40/map_apres.log`.

## Preuve §10 : traces qui changent

Suite complète (`cargo test -p scenario --test scenarios`, dumps `ALACOD_DUMP_TRACE=../d40/dump_d45_apres`) :
**une seule trace change, `avant_poste_demo`** (dès la ligne 1 : `9397584e…` → `170edda2…`).

- Cause : carte `maps/avant_poste.ldtk`, `map_seed: 2`. Ordre des gabarits placés, main :
  Depart, Couloir, Chambre, Cuisine, Armurerie, Cave, Poste, Cellier, Jug ; branche : Depart,
  Couloir, Chambre, Cuisine, Cave, Poste, Armurerie, Jug, Cellier (positions différentes) → entités
  de salle différentes dès la frame 0.
- Le déroulé de la démo est inchangé : frame 3600, vague 6, 15 tués, joueur vivant, `entités_max=128`,
  identique aux suites précédentes.
- Inchangées : tout le reste, dont `testbed_two_rooms_door_idle`, `testbed_corridor_idle` (un seul
  gabarit compatible par connexion : `nth`/`last` coïncident) et les scénarios sur
  `exemples/test_map.ldtk` (carte par défaut de `Scenario` : `clone_*`, `equilibrage_*`, etc.).
  Seuls `testbed_*` multi-salles et `avant_poste_demo` utilisent une carte multi-salles hors `test_map`.

## 20 graines zombies avant/après

`alacod-sim --game zombies --bots 4 --profiles fonceur,fonceur,prudent,immobile --seeds 1..20
--until-wave 5 --max-frames 20000` (carte par défaut `maps/avant_poste.ldtk`, `map_seed` = graine),
deux binaires construits dans ce worktree (branche mergée avec origin/main `4e8fe93`, et la même
avec les trois fichiers de `crates/map/src/generation` de main) :

| | vague 5 atteinte | desync | bloquées à 20000 frames | défaite |
|---|---|---|---|---|
| avant | 16/20 | 0 | 3 (graines 7 : v3, 10 : v2, 15 : v4) | 1 (graine 5, v4, f9775) |
| après | 16/20 | 0 | 3 (mêmes) | 1 (même) |

**Identiques graine par graine** (vague, frames, morts, kills, dégâts). Les cartes diffèrent
pourtant (19 formes contre 7). Hypothèse non vérifiée : les bots n'achètent pas les portes et
restent dans `Depart`, dont la place relative ne change pas ; les vagues ne prennent que les
spawners à portée des joueurs. La diversité des cartes ne pèsera donc qu'une fois les portes
achetées (bots ou joueurs). Résultats : `../d40/sim_d45_{avant,apres}.log`.
