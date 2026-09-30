# Fiches de tâches détaillées (préambule commun)

Ce dossier contient une fiche par tâche restante de M0 (`docs/taches.md` §5), écrite pour un
agent **sans contexte** (DeepSeek, Qwen local, Haiku…) : chaque fiche se suffit à elle-même
une fois ce préambule lu. Coller ce fichier **puis** la fiche dans le prompt de l'agent.

Les fiches : `T2.4-verifier-merger.md`, `T2.5-reprise-power-ups.md`, `T2.8-lint-nouveaux-kinds.md`,
`T2.12-hud-v1-resume.md`, `T3.1-scenarios-du-clone.md`, `T3.2-videos-digest.md`, `dettes.md`.
Ordre conseillé : T2.4 → T2.5 → T2.8 → T2.12 → T3.1 → T3.2 (puis T3.3 revue humaine et T3.4
fermeture des notes, qui ne sont pas des tâches d'agent).

## 1. Où travailler

- Meta-repo : `/home/wq/Project/bascanada/alacod_root` (`scripts/task-new.sh`, `scripts/task-merge.sh`).
  Il regroupe trois dépôts git indépendants ; le jeu est `alacod_root/alacod` (workspace Cargo).
- **Checkout principal** `alacod_root/alacod` : toujours sur `main`, propre. On n'y modifie
  **rien** pendant une tâche ; on s'en sert seulement pour les dumps de référence (§5) et, à la
  fin, pour le merge et le journal.
- **Worktree de la tâche** : `alacod_root/../alacod_tasks/<tâche>/alacod`, branche `<tâche>`
  (nom : `m0-v<voie>-<sujet>`, ex. `m0-v1-power-ups`). Créé par :
  ```bash
  cd /home/wq/Project/bascanada/alacod_root && ./scripts/task-new.sh m0-v1-ma-tache
  ```
  Le script crée la branche depuis `main`, le worktree, un `target` Cargo **amorcé** depuis
  `alacod/target/headless` (3 Go : les dépendances ne se recompilent pas) et `env.sh`.
- Avant **toute** commande cargo/make dans le worktree :
  ```bash
  cd /home/wq/Project/bascanada/alacod_tasks/<tâche>/alacod
  source ../env.sh            # CARGO_TARGET_DIR de la tâche
  export CARGO_BUILD_JOBS=4
  ```
- Machine : 15 Go de RAM partagés. **Une seule compilation à la fois**, toujours
  `--profile headless` (le profil dev pèse 6 Go et n'est pas amorcé), `--no-default-features`
  pour les binaires de jeu (`zombies`, `testbed`). Jamais de compilation depuis un `target` vide,
  jamais dans `/tmp`, jamais de build avec rendu sauf `play_scenario --features render` (vidéos).
- Ne jamais utiliser `git stash` (pile partagée entre worktrees). Ne jamais `git push`.

## 2. Lire avant de coder

- `CLAUDE.md` (racine) : règles de déterminisme du rollback GGRS — **obligatoires** : nombres
  `Fixed` (jamais `f32` dans `GgrsSchedule`), itération par `order_iter!`/`order_mut_iter!` avec
  `&GgrsNetId` en premier dans la query, `BTreeMap`/`BTreeSet` (jamais `HashMap`), RNG par
  `RngStreams`/`RollbackRng` (jamais `rand::random`), `despawn_rollback()` (jamais `despawn()`),
  `FrameEvents<T>` (jamais de `Message` Bevy dans `GgrsSchedule`), enregistrement de tout état
  rollback par `app.rollback_and_trace_resource::<T>()` / `rollback_and_trace_component::<T>()`
  (`crates/utils/src/rollback.rs`), logs avec `GgrsNetId` (jamais `Entity`).
- `docs/conventions.md` : §1 entités LDtk, §2 RON (`Fixed` en chaînes : `"1.5"`), §3 manifeste
  `game.ron` et kinds de contenu, §8 protocole de preuve, sections « Run », « Power-ups », checklist
  d'un nouveau vocabulaire.
- `docs/taches.md` : la tâche (§5) et le journal §10 (ce que les tâches précédentes ont décidé).
- `docs/plan-engine.md` §5 : le chantier (lettre) dont la tâche dépend.

## 3. Structure utile

| Chemin | Contenu |
|---|---|
| `crates/sim_core` | contrats : `Team`, `Tag`, `DamageEvent`, `StatId`, `Modifier`/`Modifiers`, `AmmoType`, `FrameEvents` |
| `crates/combat` | grille de collision, règles d'équipe et de dégâts, à terre (`downed.rs`), inventaire/munitions |
| `crates/stats` | `StatsPlugin`, `StatReader` (résolution des modificateurs) |
| `crates/run` | `Currency`, `Perks`, `Run`/`RunMode`/`RunStep`/`RunSummary` (T2.4), `RunModeRules` |
| `crates/effects` (T2.5) | `Action` des power-ups |
| `crates/content` | manifeste, registre typé, lint (`src/lint.rs`), fixtures `tests/fixtures/<erreur>/` + `tests/lint_fixtures.rs`, CLI `alacod lint` |
| `crates/game` | la simulation : `character/`, `weapons/`, `waves/`, `economy.rs`, `interaction.rs`, `powerups.rs` (T2.5), `run_state.rs` (T2.4), `replay.rs` (format des scénarios et **liste des attentes**), `ui/` (HUD, écran de fin), `core.rs` |
| `crates/map_ldtk` | chargement LDtk, génération de la carte à partir des gabarits, entités (`game/entity/*.rs`) |
| `crates/scenario` | runner de scénarios (`src/runner.rs`), tests (`tests/scenarios.rs`, `expectations.rs`, `bots.rs`, `run.rs`…), `alacod-gen`, `alacod-sim`, `play_scenario` |
| `crates/bots` | bots (`immobile`, `fonceur`, `prudent`) |
| `games/zombies`, `games/testbed` | un binaire + `assets/` (`game.ron`, personnages, armes, cartes, `economy/`, `items/`, `ui/`) |
| `tests/scenarios/*.ron` + `.trace` | scénarios de référence et leur trace d'état ; `generated/<jeu>/` : un par arme, produits par `alacod-gen` |
| `tests/budgets.ron` | planchers de fps par scénario (`make bench`) |
| `scripts/` | `trace-diff.py`, `nightly.sh`, `scenario-video`, `scenario-metrics.py`, `check-forbidden.sh`, `check-rollback-registration.sh` |

## 4. Vérification standard (à lancer soi-même, dans cet ordre)

```bash
make test_scenarios                       # tous les scénarios (traces comparées aux .trace) ; SCENARIO=<nom> pour un seul
cargo test -q --profile headless -p scenario -p run -p combat -p game -p content -p map_ldtk -p sim_core -p stats -p bots --no-fail-fast
make lint                                 # alacod lint games/zombies et games/testbed : « aucune erreur »
cargo fmt --all -- --check                # rien à afficher ; sinon `make fmt`
./scripts/check-forbidden.sh              # avertissements préexistants tolérés, aucun nouveau
./scripts/check-rollback-registration.sh  # « OK »
make gen GAME=zombies                     # seulement si une arme a changé (scénarios générés)
ALACOD_BENCH_STRICT=1 make test_scenarios && ./scripts/scenario-metrics.py   # bench : à faire machine calme
```
Résultats attendus aujourd'hui (main 6281b64) : 51 scénarios verts en `make test_scenarios`
(41 + 10 générés ; 52 avec `run_lose_summary` une fois T2.4 mergée), `expectations` 26 tests,
`lint_fixtures` 39 tests, lint des deux jeux sans erreur.

**p2p à deux clients** (obligatoire quand la simulation ou la session change) :
```bash
docker compose -f docker-compose.ci.yaml up -d signaling && sleep 3
cargo build -q -p zombies --profile headless --no-default-features
for i in 0 1; do
  ALACOD_HEADLESS=1 ALACOD_STATE_TRACE=/tmp/p2p-$i.trace ALACOD_EXIT_AT_FRAME=600 APP_VERSION=x \
  cargo run -q -p zombies --profile headless --no-default-features -- --matchbox ws://127.0.0.1:3536 \
    --lobby test-$$ --number-player 2 --players localhost remote --cid client_$i --name client_$i \
    > /tmp/p2p-$i.log 2>&1 &
done; wait
cmp /tmp/p2p-0.trace /tmp/p2p-1.trace && echo "p2p : traces identiques"
docker compose -f docker-compose.ci.yaml down
```
(script bash : en zsh les `wait $pid` d'une variable non éclatée échouent). Attendu :
« traces identiques », 599 lignes.

## 5. Traces et preuve (protocole `docs/conventions.md` §8)

Chaque scénario a une trace de référence (`.trace` : hash de l'état rollback par frame).
`make test_scenarios` échoue si elle change. **Une trace ne se réécrit que pour un changement
de jeu voulu**, avec une preuve que rien d'autre n'a bougé :
1. Dump détaillé sur `main` (checkout principal, **sans** `CARGO_TARGET_DIR`, un scénario par
   appel, rien à y modifier) :
   ```bash
   cd /home/wq/Project/bascanada/alacod_root/alacod
   ALACOD_SCENARIO=idle ALACOD_DUMP_TRACE=/tmp/dump-main APP_VERSION=x \
     cargo test -p scenario --profile headless --test scenarios scenarios -- --nocapture
   ```
   (le dossier doit exister et être vide ; un `.full` de 300 Mo à 1 Go par scénario, à supprimer après).
2. Même chose dans le worktree (`source ../env.sh`), vers `/tmp/dump-branche`.
3. `python3 scripts/trace-diff.py /tmp/dump-main/idle.full /tmp/dump-branche/idle.full --ignore Type1,Type2`
   où `Type1,Type2` sont **exactement** les nouveaux types rollback de la tâche. Résultat attendu :
   « identiques » (ou, sans `--ignore`, seulement des lignes de ces types). Toute autre
   différence est un bug à corriger avant le bless.
4. Bless : `BLESS=1 make test_scenarios` (tous) ou `BLESS=1 SCENARIO=<nom> make test_scenarios`, et
   la justification dans le message de commit (quels scénarios, pourquoi, preuve faite sur quoi).

Scénarios à dumper au minimum : `idle`, `two_players_shooting`, `points_on_kill`, `downed_all_lose`
(un joueur, tir, économie, défaite).

## 6. Commits, merge, journal

- Commits sur la branche de la tâche, messages en français, une ligne de titre puis le détail
  (ce qui change, décisions, bless et sa preuve). Terminer par une ligne d'attribution, ex.
  `Co-Authored-By: DeepSeek <noreply@deepseek.com>`. Avant de commiter un nouveau fichier :
  `git check-ignore -v <fichier>` ne doit rien afficher.
- Merge (par la personne qui vérifie, jamais par l'agent qui a codé, sauf instruction) :
  ```bash
  cd /home/wq/Project/bascanada/alacod_root && ./scripts/task-merge.sh <tâche>   # merge --no-ff, retire worktree et branche
  cd alacod && find crates games -name '*.rs' -exec touch {} +                      # binaires de test périmés sinon
  rm -rf ../alacod_tasks/<tâche>/target ../alacod_tasks/<tâche>                     # 20 à 40 Go par tâche
  ```
  Si `main` a avancé pendant la tâche : `git merge main` dans le worktree d'abord ; conflits sur
  des `.trace` = les deux côtés ont re-blessé → prendre n'importe quelle version puis refaire
  preuve (§5, contre le nouveau `main`) et bless sur l'état fusionné.
- Journal : une ligne dans le tableau de `docs/taches.md` §10, sur `main`, après le merge :
  `| AAAA-MM-JJ | T2.x (ce qui a été livré, en une phrase dense) | \`branche\` | modèle | mergée ; comment vérifié (scénarios, tests, preuve, bench, p2p) ; non fait ; dettes |`.

## 7. Rapport de fin (obligatoire, honnête)

1. Fait : fichiers, décisions et pourquoi.
2. Vérifié : chaque commande du §4 (et §5 si bless) **avec son résultat réel** (nombres).
3. Non fait / incertain / non vérifié (un scénario jamais vu passer est « non vérifié »).
4. Dettes laissées, questions ouvertes.
Ne jamais écrire « complet » ou « vérifié » pour ce qui n'a pas tourné sous vos yeux.
