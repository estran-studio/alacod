# outillage-build : Bevy dynamique, purge du target, sccache

Branche `outillage-build`. Machine orca (8 cœurs, 22 Go, charge élevée pendant les mesures : les
durées absolues sont bruitées, les ordres de grandeur sont fiables).

## Ce qui change

1. **Bevy en bibliothèque partagée** : feature `dev-dylib` (`game`, `scenario`, `zombies`, `testbed`,
   `throne`) = `bevy/dynamic_linking`. Règle unique dans `scripts/dev-features.sh` (Makefile,
   `scenario-video`, `p2p-restart.sh`). **Vide** si `CI` est défini, `PROFILE=prod`, `TARGET=web`
   ou `DYLIB=0` : le build web, la CI, les PR et la release restent statiques, inchangés. Les crates de
   simulation gardent `bevy default-features=false`. Binaires lancés à la main : `LD_LIBRARY_PATH=$(./scripts/dev-features.sh libpath)`
   (exporté par le Makefile et `scenario-video`). `make test_crates` ajouté (tests des crates avec les mêmes features).
2. **Purge automatique** : `scripts/target-sweep.sh` (`--dry-run`), `make sweep`, `SWEEP=1 make test_scenarios`.
   Supprime `incremental/` et ne garde que la dernière génération de chaque artefact des crates du workspace.
3. **sccache** : mesuré, utile seulement dans le même chemin de target (voir ci-dessous).
4. **Signaling p2p par session** : `docker-compose.ci.yaml` lit `MATCHBOX_PORT` (défaut 3536),
   `p2p-restart.sh` aussi ; `COMPOSE_PROJECT_NAME` par session.

## Mesures (avant → après)

| mesure | avant (statique) | après (dylib) |
|---|---|---|
| taille d'un binaire (`alacod-sim`) | 1,07 Go | 0,36 Go (+ `libbevy_dylib.so` 0,79 Go partagée) |
| target après `make test_scenarios` + `test_crates` + `lint` | 39–41 Go | 49 Go avant purge (deux générations), **22 Go après `target-sweep`** |
| recompilation, 1 ligne dans `crates/game`, `CARGO_INCREMENTAL=0` | 5 min 05 | 4 min 21 |
| idem avec `CARGO_INCREMENTAL=1` (profil headless) | non mesuré | **16 s** (1ʳᵉ compilation 7 min, +5 Go d'`incremental/`) |
| `make test_scenarios` complet (sans bless) | ≈ 8 min 40 incl. rebuild (idle) | 4679 s sur machine chargée, rc=0 ; relance sans changement 124 s |
| `make test_crates` | — | 7464 s, 0 échec |
| `make lint` | — | 51 s, rc=0 |

**Aucune trace n'a bougé** : `make test_scenarios` complet sans bless est vert. p2p
(`scripts/p2p-restart.sh`, dylib) : parties 1 et 2 identiques entre les deux clients (599 lignes),
partie 2 ≠ partie 1, local identique : conforme à l'attendu.

### sccache (build complet de `scenario` en headless, target vide)

| cas | durée | hits Rust |
|---|---|---|
| à froid | 2845 s | 0 |
| à chaud, **même** chemin de target | **325 s** | 599/599 |
| à chaud, **autre** chemin de target | 2694 s | 0 (199 misses sur `-p world`) |

Conclusion : sccache ne remplace pas l'amorçage du target entre worktrees (le chemin entre dans les
clés), mais accélère la reconstruction après purge totale dans le même target. Crates `bin`,
`proc-macro` et `dylib` non cachables (120 appels). Il est installé sur l'hôte (`cargo install sccache`, 0.18).

## Décision proposée

- `CARGO_INCREMENTAL=1` en boucle d'édition : gain de 16× pour +5 Go ; `sweep` le supprime.
- Le target amorcé (`/home/debian/alacod-target-nightly/headless`, 26 Go **statique**) ne sert plus aux
  builds dev (features différentes : Bevy se recompile). Ré-amorcer depuis le target de cette tâche
  après merge (≈ 22 Go, dylib) ; la copie par `cp --reflink` reste le moyen de démarrer vite.

## Diff proposé pour `/home/debian/orca/task-new.sh` (hors dépôt, non appliqué)

```diff
--- task-new.sh.orig	2026-10-10 23:18:42.252404517 +0000
+++ task-new.sh	2026-10-10 23:18:42.276404817 +0000
@@ -5,7 +5,8 @@
 #   - worktree dans /home/debian/orca/tasks/<branche>/alacod ;
 #   - target Cargo amorcé (cp --reflink=auto) depuis /home/debian/alacod-target-nightly/headless,
 #     puis touch des sources du workspace ;
-#   - env.sh : CARGO_TARGET_DIR de la tâche, CARGO_BUILD_JOBS=3.
+#   - env.sh : CARGO_TARGET_DIR de la tâche, CARGO_BUILD_JOBS=3, sccache (si installé), signaling.
+#   - MATCHBOX_PORT=<port> : port de signaling de la session (b0 3536, orch 3546, b1 3556).
 #   - TARGET_FROM=<tasks/x/target> : recycle (mv) le target d'une tâche mergée au lieu de copier.
 set -euo pipefail
 BR="$1"; BASE="${2:-origin/main}"
@@ -41,7 +42,18 @@
 export CARGO_TARGET_DIR=$DIR/target
 export CARGO_BUILD_JOBS=3
 export CARGO_INCREMENTAL=0
+# p2p : un port de signaling et un projet docker compose par session (README §1)
+export MATCHBOX_PORT=${MATCHBOX_PORT:-3536}
+export COMPOSE_PROJECT_NAME=$BR
+# sccache par tâche (pas global) : ne rend que pour reconstruire DANS LE MÊME target
+# (après purge) : 325 s au lieu de 2845 s ; 0 hit entre deux target de chemins différents.
+if command -v sccache >/dev/null; then
+  export RUSTC_WRAPPER=sccache
+  export SCCACHE_DIR=/home/debian/.cache/sccache
+  export SCCACHE_CACHE_SIZE=20G
+fi
 EOF
 find "$DIR/alacod/crates" "$DIR/alacod/games" -name '*.rs' -exec touch {} +
+# Purge entre deux commandes cargo : ./scripts/target-sweep.sh (ou SWEEP=1 make test_scenarios)
 echo "prêt : cd $DIR/alacod && source ../env.sh"
 df -h /home | tail -1
```

## Non testé / réserves

- **macOS** : non testé (pas de machine). `DYLD_LIBRARY_PATH` est exporté en parallèle de
  `LD_LIBRARY_PATH` ; la lib s'appelle `libbevy_dylib.dylib`. En cas de souci : `DYLIB=0`.
- Windows : non traité (la règle suit le script bash).
- Render : `play_scenario --features render` reçoit les mêmes features dev (`scenario-video`) ; vidéo non rejouée ici.
- Règle d'or : mêmes features sur toutes les commandes cargo d'un worktree, sinon Bevy se compile deux fois.
