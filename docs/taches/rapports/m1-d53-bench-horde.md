# m1-d53-bench-horde — `bench_horde` a perdu la moitié de sa vitesse pendant M1 (D53)

## État en cours

- Fini : bissection, profil, correctif sans changement de simulation, mesure finale au calme.
  **`bench_horde` 40 → 59 fps** (plancher 38) ; les trois autres benchs inchangés. Rien de béni.

## Bissection

Même machine, même target (worktree temporaire, sources touchées à chaque commit pour ne jamais
réutiliser un artefact périmé), `bench_horde` joué deux fois par commit, charge relevée
(scripts `../d40/d53_step.sh`, `d53_bisect.sh` ; seuil 48 fps entre les bornes).

- Bornes (charge 5 à 7) : `9e85e79` (fin de M0, m1-v1e) **63,7 / 63,6** ; `a8b813e` **33,0 /
  32,7**. Scénario `bench_horde.ron` identique entre les deux.
- First-parent (40 commits qui touchent le code) : `2f250eb` 33,0/32,6 ; `992bbfe` 33,7/33,2 ;
  `4cd0b46` 33,3/32,9 ; `4ec8de0` 62,1/59,3 ; **`716df9e` 32,4/33,5** (merge m0-v7 phase 2).
- Dans la branche de `716df9e` (27 commits) : `4e46688` 63,6/62,9 ; `8785220` 34,5/34,4 ;
  `80a7f67` 66,2/66,4 ; `318affc` 65,3/63,5 ; `4660154` 64,3/62,8 → **coupable `8785220`**
  « map: aligner la salle de spawn sur la grille de tuiles » (parent `4660154`).

## Profil

`perf` absent, `ptrace_scope` = 1 (pas d'attache) : échantillonnage de piles par `gdb` qui lance
lui-même le binaire de test (`../d40/d53_sample.py`, une pile toutes les 0,15 s, fils actifs
seulement). Part des échantillons actifs où la fonction est sur la pile :

| Fonction | `4660154` (parent) | `8785220` (coupable) |
|---|---:|---:|
| `move_enemies` | 12 % | 31 % |
| `recovery_point` | 0 % | 19 % |
| `query_aabb` | 3 % | 17 % |

Cause : le snap ne coûte rien lui-même ; il change la géométrie (alignement sur les tuiles) et
fait entrer les zombies du bench dans le **guidage de récupération** de `318affc`
(`navigation_recovery_due` : 600 frames sans kill ni spawn ; `bench_horde` n'a aucun kill),
dont le point de steering préféré est désormais refusé. `recovery_point` calculait **d'avance**
les destinations (les 225 points de la case suivante, chacun testé contre les grilles de
collision, requêtes qui allouent) même quand le premier candidat de l'étape 0 suffisait, et
retestait les mêmes points (candidat, segment, destination).

## Correctif (`crates/game/src/character/enemy/ai/pathing.rs`, `recovery_point`)

- Destinations calculées **au premier besoin** (`OnceCell`) : elles ne servent qu'aux étapes 1
  et 2, examinées après toute l'étape 0.
- Résultat de `clear(point)` **mémorisé** pour la durée de l'appel (`BTreeMap` sur les bits).
- Même ordre de candidats, même prédicat, `clear` sans effet de bord : résultat identique.
  Coût inhérent restant : la récupération elle-même (voulue par m0-v7) ; pas de changement de
  plancher proposé (59 > 38).

## Mesure finale au calme

Fenêtre calme donnée par orch (charge 1,4 à 1,9, aucune sim ni compilation), binaires de test
de la branche sans correctif (base) et avec (fix), en alternance A B A B :

| Scénario | base (2 passages) | fix (2 passages) |
|---|---|---|
| `bench_horde` | 39,8 / 40,8 | **58,7 / 59,6** |
| `bench_bullets` | 133,6 / 136,3 | 134,4 / 135,1 |
| `bench_cave` | 195,7 / 194,3 | 198,3 / 194,6 |
| `bots_four_mixed` | 108,9 / 106,9 | 106,0 / 106,3 |

`bench_horde` reste sous les 63,6 de `9e85e79` (mesurés sous charge 5) : le reste de l'écart
est le coût de la récupération et de la géométrie alignée, pas un gaspillage repéré.

## Vérifié

- Suite complète des scénarios **sans bless : 0 trace différente** (838 s).
- Crates : 602 tests, 0 échec ; `make lint` des trois jeux, `cargo fmt --check`,
  `check_forbidden` 4 (inchangé), `check_rollback_registration` OK, exemples.

## Incident

Le target partagé de la bissection a grossi jusqu'à 64 Go et /home est tombé à 567 Mo libres
(première suite plantée à l'édition de liens) ; purge faite, orch prévenu ; mémoire ajoutée
(purger après chaque commit construit).
