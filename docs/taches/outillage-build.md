# outillage-build — espace disque et temps de compilation (machine `orca`)

Lire d'abord `docs/taches/README.md` et `docs/taches.md` §7 « Exécution ». Branche `outillage-build`
depuis `main`. Décision de William du 2026-10-09 : points 2, 3 et 5 ci-dessous (le target partagé
et `debug = 0` sont écartés ; le binaire de test unique n'est pas demandé).

## Constat (orch, 2026-10-09)

- Trois sessions (b0, b1, orch) ont chacune un target de 35 à 56 Go ; le disque (ext4, 197 Go,
  pas de reflink) est monté deux fois à 99 % et a fait planter un build.
- Le gros poste : les binaires (~1 Go chacun, Bevy lié statiquement, `line-tables-only`) : 14
  binaires de test de `scenario`, `alacod-sim`, `alacod-gen`, `play_scenario`, `alacod`, les
  tests des autres crates, et plusieurs générations par crate selon les commandes.
- Le linker est déjà LLD (rustc nightly 1.101). L'amorçage d'une tâche copie 26 Go
  (`/home/debian/orca/task-new.sh`, depuis `/home/debian/alacod-target-nightly/headless`).

## À faire

1. **`dynamic_linking` de Bevy en développement (point 2)**.
   - Une feature de workspace (ex. `dev-dylib`) qui active `bevy/dynamic_linking`, **jamais**
     en `release` ni dans les builds de la CI de PR si elle les fausse.
   - Elle s'active pour les profils `dev` et `headless` par les commandes du `Makefile`
     (`test_scenarios`, `lint`, `gen`, `sim`, `zombies`, `throne`…) et par l'`env.sh` des tâches.
   - Vérifier que les crates de simulation restent en `bevy` `default-features = false` (frontière
     console, `CLAUDE.md`) : la feature ne doit pas y tirer le rendu.
   - Les binaires lancés hors de `cargo run`/`cargo test` (`target/headless/alacod-sim` dans
     `scripts/nightly.sh`, `scripts/scenario-video`, `make sim`…) doivent trouver la bibliothèque
     partagée (`LD_LIBRARY_PATH` ou rpath) : tous les scripts de `scripts/` et le `Makefile`
     relus et testés.
   - Builds sans rendu (`--no-default-features`) et avec rendu (`play_scenario --features render`,
     une vidéo) tous deux vérifiés.
2. **Purge automatique (point 3)**.
   - `cargo-sweep` installé (`cargo install cargo-sweep`) ou un script équivalent
     `scripts/target-sweep.sh`, qui ne garde que la dernière génération des crates du workspace et
     retire `incremental/`.
   - Appelé à la fin de `make test_scenarios` (option, ex. `SWEEP=1`) et par
     `/home/debian/orca/task-new.sh` (le script est hors dépôt : proposer le diff à orch).
3. **`sccache` (point 5)**.
   - `cargo install sccache` ; wrapper activé **par tâche** (`RUSTC_WRAPPER=sccache` dans
     `env.sh`), pas dans `~/.cargo/config.toml` global : le runner CI nightly
     (`/home/debian/actions-runner`) tourne sous le même utilisateur et ne doit pas changer sans
     décision.
   - Cache borné (`SCCACHE_CACHE_SIZE`, ex. 20G) dans `/home/debian/.cache/sccache`.
   - Mesurer si un target **neuf** (sans copie de 26 Go) se construit en un temps acceptable
     depuis le cache ; si oui, proposer à orch un `task-new.sh` sans copie.

## Mesures (avant / après, machine aussi calme que possible, dans le rapport)

- Taille du target après `make test_scenarios` + tests des crates + `make lint`.
- Temps d'une recompilation incrémentale après une modification d'une ligne dans `crates/game`
  (`make test_scenarios SCENARIO=idle`, compilation seule).
- Temps de `make test_scenarios` complet.
- Premier build d'un target neuf avec `sccache` (cache froid, puis cache chaud).

## Contraintes

- **Aucune trace ne bouge** (`make test_scenarios` complet sans bless) ; p2p à deux clients
  (Docker disponible : `docker compose -f docker-compose.ci.yaml up -d signaling`) traces
  identiques.
- Ne pas casser macOS (William joue et développe sur Mac : `dynamic_linking` y marche, mais
  vérifier les chemins `DYLD_*` si un script en dépend ; dire ce qui n'a pas pu être testé ici).
- Doc : `docs/taches/README.md` §1 (règles de compilation et d'hygiène du target) mis à jour.

Rapport `docs/taches/rapports/outillage-build.md`, livraison `LIVRÉ outillage-build <sha>`.
