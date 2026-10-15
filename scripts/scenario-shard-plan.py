#!/usr/bin/env python3
"""Plan des tranches de `scripts/test-scenarios-sharded.sh` (tâche perf-suite-scenarios).

usage : scenario-shard-plan.py <racine> <latest.json> <N> [nom,nom,…]

Répartit les scénarios en N tranches de durée égale (plus long d'abord, sur la tranche la
moins chargée) ; imprime une ligne `i<TAB>nom,nom,...` par tranche.

Durée d'un scénario : `sim_seconds` de la dernière suite (`<target>/metrics/latest.json`) plus
un coût fixe de chargement, sinon ses `frames` à FALLBACK_FPS. Déterministe (tri par durée
décroissante puis par nom) : deux appels sur les mêmes fichiers donnent le même plan.
"""
import glob
import json
import os
import re
import sys

LOAD_SECONDS = 1.0      # construction de l'app + chargement du contenu, par scénario
FALLBACK_FPS = 60.0     # sans métrique : frames / FALLBACK_FPS

root, metrics_path, shards = sys.argv[1], sys.argv[2], int(sys.argv[3])
only = {n.strip() for n in sys.argv[4].split(",") if n.strip()} if len(sys.argv) > 4 else set()
paths = sorted(glob.glob(os.path.join(root, "tests/scenarios/*.ron")))
paths += sorted(glob.glob(os.path.join(root, "tests/scenarios/generated/*/*.ron")))
metrics = {}
try:
    metrics = json.load(open(metrics_path))
except (OSError, ValueError):
    pass

scenarios_dir = os.path.join(root, "tests/scenarios")
costs = {}
for path in paths:
    # Même id que le runner (`crates/scenario/tests/scenarios.rs::scenario_id`) : chemin relatif
    # sans extension, `generated/<jeu>/<nom>` pour les générés. Le nom court seul confondrait
    # les scénarios homonymes de deux jeux (une tranche en perdrait un, et `ALACOD_SCENARIO`
    # refuserait l'autre comme ambigu).
    name = os.path.splitext(os.path.relpath(path, scenarios_dir))[0]
    if only and name not in only and os.path.basename(name) not in only:
        continue
    known = metrics.get(name)
    if known and known.get("sim_seconds"):
        cost = known["sim_seconds"]
    else:
        match = re.search(r"^\s*frames:\s*(\d+)", open(path).read(), re.M)
        cost = int(match.group(1)) / FALLBACK_FPS if match else 30.0
    costs[name] = cost + LOAD_SECONDS

load = [0.0] * shards
plan = [[] for _ in range(shards)]
for name in sorted(costs, key=lambda n: (-costs[n], n)):
    target = min(range(shards), key=lambda i: (load[i], i))
    plan[target].append(name)
    load[target] += costs[name]
for i, names in enumerate(plan):
    print(f"{i}\t{','.join(sorted(names))}")
print(f"# charge estimée par tranche (s) : {[round(x) for x in load]}", file=sys.stderr)
