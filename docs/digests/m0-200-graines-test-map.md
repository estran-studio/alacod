# Critère M0 rejoué sur `test_map` : 200 graines, quatre acheteurs

m0-200-graines-test-map, 2026-10-05 et 2026-10-06, machine locale (pas la session cloud prévue,
qui n'a jamais livré).

## Mesure

- Binaire : `alacod_tasks/m1-200-throne/alacod-sim-f80b82b`, `main` **`f80b82b`**, aucune
  compilation. C'est **avant D51 et m1-v3-bots-armes** (41 commits de retard sur `main`
  `ed8a274` au 2026-10-06) ; b1 a mesuré `zombies` inchangé après D51 (20/20 sur 20 graines) :
  le résultat vaut pour M0, sur cette référence.
- Commande, par tranches de 10 graines (une seule simulation à la fois, sauf la première demi-
  journée à deux files) :
  ```
  alacod-sim-f80b82b --game zombies --bots 4 --profiles acheteur,acheteur,acheteur,acheteur \
    --map exemples/test_map.ldtk --seeds A..B --until-wave 5 --max-frames 20000 --progress \
    --json sim-A-B.json
  ```
- Machine : 12 cœurs, 15 Go, partagée avec les simulations et compilations des autres sessions
  (7 à 92 fps de simulation selon la charge, 3 à 17 min par graine).
- Déroulé : lancé le 5 à 15 h 48 en deux lots de 100 ; mis en pause (priorité aux mesures de
  b1), repris par tranches de 10 (un JSON par tranche), arrêté une fois par la mémoire de la
  machine, une fois par la fin de la session, repris chaque fois là où il s'était arrêté ; fini
  le 6 vers 18 h. Les graines 1 et 101, jouées avant le découpage, ont été rejouées pour leur JSON.
- Données : `docs/digests/m0-200-test-map/sim-1-200.json` (les 200 graines fusionnées) et les
  journaux par tranche ; copie hors dépôt dans `alacod_tasks/m0-200-test-map/`.

## Résultat

| | Graines |
|---|---:|
| jouées | 200 |
| vague 5 atteinte | **199** |
| desync | **0** |
| soft-lock | **1** (graine 100) |
| avec au moins un mort | **0** |
| frames jusqu'à la vague 5 (min / médiane / max) | 6 737 / 7 637 / 9 280 |
| kills à la vague 5 (min / max) | 48 / 56 |
| portes ouvertes (événements, min / max) | 0 / 14 |

**Critère non atteint** au sens strict du §9.8 (« sans soft-lock ») : 1 soft-lock sur 200, 0
desync. Sur `avant_poste` (nuit du 2026-10-04, `docs/taches.md`), la même mesure donnait 200/200.

## Graine en échec

| Graine | Vague | Frames | Fin | Signature (relevé D42) |
|---:|---:|---:|---|---|
| 100 | 1 | 20 000 | soft-lock | dernier zombie figé dehors devant une fenêtre intacte ; bots enfermés dans une salle sans le voir |

Relevé D42 de la graine 100 (`softlock` du JSON, instantanés à f19400 et f20000) :

- Vague 1 en cours : 6 zombies générés, 5 tués, le dernier kill à f790 ; plus rien jusqu'au
  plafond.
- Le zombie restant (`zombie_full` #233, 50 PV) est en case (26, 25), **dehors, collé à une
  fenêtre intacte** (santé 3). Il est **dans son champ de flux** (`path_cost` 484), en
  `Chasing`, `next_cell` (25, 25), sans mur sous son point de visée ; sa position est
  **identique au millième** entre f19400 et f20000 : immobile, sans casser la fenêtre.
- Les quatre bots `acheteur` sont dans une salle fermée en (12–13, 43), à 364 px du zombie,
  sans ligne de vue, réserves pleines (210 balles, chargeur plein ou presque) ; eux non plus ne
  bougent pas entre f19400 et f20000.
- Ce n'est ni une porte fermée qui bloque, ni un ennemi hors de son champ (D48) : un zombie figé
  devant une fenêtre (moteur : déplacement ou attaque de fenêtre) et des bots qui ne vont pas
  le chercher. **Confié à b1**, fiche `m0-graine-100-fenetre` (rejeu et diagnostic).
