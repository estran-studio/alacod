# Performance — reproduction du playthrough M1

2026-10-10, Apple M4 Max, 14 cœurs, 36 Gio, macOS 26.7.1.
Profil Cargo `headless` (workspace opt-level 1), **rendu actif**, fenêtre 800×600 logique,
même résolution et mêmes inputs, exécutions successives sans compilation ni capture PNG.
La séquence rejouée est la copie de diagnostic documentée dans
[run 01](../m1-f53cd1ae-run-01/README.md), tronquée à 5800 frames.
Elle conserve la progression immédiate historique pour isoler la performance ;
`choice_timing` est temporairement `Immediate` pendant ces mesures puis restauré.
Le défaut d'identité ancien enregistrement/natif ne permet pas de promettre que chaque
frame est celle vue par William ; les **cinq comparaisons** utilisent bien la même séquence.

## Résultats

Fenêtre d'analyse : frames de simulation 3200–5600, 28 balles vivantes au maximum.
Percentiles par rang supérieur, tous les updates rendus inclus (même sans nouvelle frame
simulée). Une frame de rendu peut avancer plusieurs frames de simulation lors du rattrapage.

| Cas | Cadence moyenne (ms) | p95 (ms) | p99 (ms) | Intervalles >33,3 ms | CPU simulation moyen (ms/update) |
| --- | ---: | ---: | ---: | ---: | ---: |
| Avant, info, vérification 2 | 21,52 | 53,79 | 98,15 | 208 / 1860 | 9,31 |
| Avant, warn, vérification 2 | 19,72 | 46,97 | 73,95 | 145 / 2026 | 8,04 |
| Avant, warn, vérification 0 | 16,67 | 21,08 | 22,02 | 3 / 2400 | 2,42 |
| Navigation optimisée, warn, vérification 0 | 16,67 | 17,27 | 17,63 | 0 / 2400 | 1,27 |
| Navigation optimisée, warn, vérification 2 | 16,67 | 17,33 | 17,60 | 0 / 2401 | 3,06 |

Le changement de mode local est distinct de l'optimisation moteur. En conservant la
vérification 2, la navigation optimisée ramène le p99 de 73,95 à 17,60 ms.
À vérification 0 identique, elle réduit le CPU simulation moyen d'environ 47 %.
Le logging seul ne suffit pas à régler les chutes.
La cadence moyenne correspond à 60 updates/s ; le p95 reste légèrement supérieur à la
cible stricte proposée de 16,7 ms. Ce compteur mesure l'intervalle entre updates de l'app,
**pas les timestamps GPU ni la présentation effective à l'écran**. Aucun gain chiffré
pour une charge de 500 projectiles ni pour release n'est revendiqué.
La confirmation humaine sur la fluidité reste attendue.

## Cause et correctif

`sample` dans le passage du boss montre les recherches BTree de `build_flow_field` parmi
les piles actives dominantes. Le dégagement des grands acteurs vérifiait neuf cellules
par test et recommençait à chaque arête de Dijkstra. Le recalcul périodique coûte assez
pour déclencher des rattrapages de plusieurs frames, multipliés par le synctest local.

La géométrie est maintenant calculée une fois par construction dans des tableaux locaux
(murs, dégagement, passage étroit, distances au mur, obstacles cassables). Dijkstra garde
l'ordre du tas, des voisins, les coûts, les directions et propriétaires historiques.
Les tableaux ne persistent pas entre frames et ne participent pas au rollback.
Le client local utilise aussi une seule simulation par frame par défaut ; les scénarios
conservent `ALACOD_CHECK_DISTANCE=2`. L'option permet toujours de vérifier le client joué.

## Reproduire et vérifier

Compilation : `cargo build -p scenario --profile headless --features render --bin play_scenario`.
Préparation de la copie (les inputs restent inchangés) :

```sh
python3 - <<'PY_REPLAY'
from pathlib import Path
source = Path('tests/review-notes/m1-f53cd1ae-run-01/original.ron').read_text()
source = source.replace('game: "zombies",', 'game: "throne",\n    floors: Some("run"),\n    clocks: Some(["etage"]),\n    difficulty: Some(true),\n    mode: Some(Floors),\n    progression: Some("run"),')
source = source.replace('frames: 11882,', 'frames: 5800,')
Path('/tmp/alacod-m1-perf-baseline.ron').write_text(source)
PY_REPLAY
```

Puis, avec cette séquence et la progression historique (`Immediate` pour le comparatif) :

```sh
RUST_LOG=warn ALACOD_CHECK_DISTANCE=2 ALACOD_PERF_CSV=/tmp/throne-perf.csv \
  target/headless/play_scenario /tmp/alacod-m1-perf-baseline.ron --log
```

`ALACOD_PERF_CSV` fonctionne aussi avec `make throne ARGS="--profile headless"` pour la
prochaine partie humaine. Il est opt-in, sans état de gameplay modifié. `main_cpu_ms`
mesure First→Last (ne couvre pas tout le sous-app rendu) ; `simulation_ms` cumule les
exécutions de GgrsSchedule ; `simulation_steps` inclut les resimulations.

Les CSV complets compressés et `summary.json` sont les données des cinq cas ci-dessus.
Validation : comparaison exacte des cartes coût/direction/propriétaire et de leur hash
sur 384 configurations (24 géométries × 4 profils × 2 gabarits × 2 voisinages).
La validation des scénarios se fait avec `choice_timing: Immediate`, sans bénir de trace,
pour isoler cette optimisation des changements de gameplay suivants.

Résultat de la validation : **204 scénarios verts, 204 traces de référence inchangées**,
sans bless, à vérification 2. Aucun avertissement de budget. Les tests de rejeu et de
chemin du dossier de métriques passent également. La progression est restée historique
pendant cette validation, puis `BetweenFloors` a été restauré dans le chantier suivant.
