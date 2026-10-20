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
  source ../env.sh            # CARGO_TARGET_DIR de la tâche, CARGO_INCREMENTAL=0, port de signaling
  export CARGO_BUILD_JOBS=4
  ```
- Machine : 15 Go de RAM partagés. **Une seule compilation à la fois**, toujours
  `--profile headless` (le profil dev pèse 6 Go et n'est pas amorcé), `--no-default-features`
  pour les binaires de jeu (`zombies`, `testbed`). Jamais de compilation depuis un `target` vide,
  jamais dans `/tmp`, jamais de build avec rendu sauf `play_scenario --features render` (vidéos).
- **Bevy en bibliothèque partagée (outillage-build, 2026-10-10).** En développement natif (profils
  `dev` et `headless`), les commandes `make` passent `--features game/dev-dylib`
  (`bevy/dynamic_linking`) : les binaires (14 binaires de test de `scenario`, `alacod-sim`,
  `alacod-gen`, `play_scenario`…) ne portent plus Bevy, ils pèsent ~0,3 Go au lieu de ~1,1 Go, et
  `libbevy_dylib.so` (0,8 Go) est partagée. La règle est dans `scripts/dev-features.sh` (une seule
  source de vérité : Makefile, `scenario-video`, `p2p-restart.sh`). **Jamais** en release
  (`PROFILE=prod`), pour le web (`TARGET=web`, `scripts/build-web-games.mjs`), en CI (variable `CI`,
  donc aussi le runner nightly) ni avec `DYLIB=0` : ces builds restent statiques, identiques à ceux
  d'avant. Règle d'or : **toutes les commandes cargo d'une même arborescence passent les mêmes
  features**, sinon Bevy est compilé deux fois (+25 Go). Pour une commande cargo à la main :
  `cargo test -p scenario --profile headless $(./scripts/dev-features.sh features) …`, ou mieux
  `make test_crates` (la ligne du § 4). Un binaire lancé à la main (hors `cargo run`/`make`) a
  besoin de `LD_LIBRARY_PATH=$(./scripts/dev-features.sh libpath)` (macOS : `DYLD_LIBRARY_PATH`).
- **Hygiène du target et du contexte (règle de William, 2026-10-03, automatisée 2026-10-10)** :
  `make sweep` (ou `SWEEP=1 make test_scenarios`) lance `scripts/target-sweep.sh` : supprime
  `incremental/` et ne garde que la dernière génération de chaque artefact des crates du workspace.
  À lancer **entre** deux commandes cargo (jamais pendant : un build en cours perd ses fichiers).
  Vérifier `df -h /home` (≥ 40 Go libres : trois targets se partagent le disque). Et **compacter
  son contexte** : un point d'état de dix lignes (fait / en cours / prochaines étapes / chiffres)
  dans `docs/taches/rapports/<branche>.md`, commité localement, pour qu'une compaction ou un
  redémarrage de session ne perde rien ; relire la fiche après compaction.
- **Boucle d'édition : `CARGO_INCREMENTAL=1`** (mesuré : 16 s au lieu de 261 s pour une ligne dans
  `crates/game`, ~+5 Go de `incremental/`, effacés par `make sweep`). Pour une série d'éditions,
  `export CARGO_INCREMENTAL=1` après `source ../env.sh` ; le remettre à 0 pour les mesures de
  durée et avant `sweep`. Aucune trace ne bouge (scénario rejoué, 3 réussis).
- **sccache** : `RUSTC_WRAPPER=sccache` (dans `env.sh` si installé, pas global ; `SCCACHE_DIR`
  `~/.cache/sccache`, `SCCACHE_CACHE_SIZE=20G`). Il ne rend que pour **reconstruire dans le même
  chemin de target** (build complet 325 s au lieu de 2845 s) : entre deux worktrees (target de chemins
  différents) 0 hit. Les crates `bin`/`proc-macro`/`dylib` ne sont pas cachées.
- Ne jamais utiliser `git stash` (pile partagée entre worktrees). Jamais de `git push` sur
  `main` : seule la branche de tâche est poussée, à la livraison (`git push -u origin <tâche>`,
  voir `PROMPT-KICKSTART.md`).

### Variante cloud (agent hors de la machine de William : Claude Code sur claude.ai/code, etc.)

L'environnement est un clone de `estran-studio/alacod` (GitHub) seul : pas de meta-repo, pas
de `scripts/task-new.sh`, pas de worktree ni d'`env.sh`. Ce qui change :

- **Branche** : `git fetch origin && git checkout -b <tâche> origin/main` dans le clone. Tout le
  reste du README s'applique avec des chemins relatifs à la racine du clone.
- **Compilation** : à froid, inévitablement (il n'y a pas de target à amorcer) : toujours
  `--profile headless` et `--no-default-features` pour les binaires de jeu, `CARGO_BUILD_JOBS`
  non limité (la machine n'est pas partagée), une seule commande cargo à la fois. Compter 20 à
  40 minutes pour le premier `make test_scenarios` ; ensuite c'est incrémental. Prérequis :
  Rust **nightly** (`rust-toolchain.toml`) et les paquets de la CI (`.github/workflows/pr.yaml`) :
  `pkg-config libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev libx11-dev`.
- **Preuve (§5)** quand une trace doit changer : les deux dumps se font dans le même clone,
  `main` d'abord (`git worktree add ../alacod-main origin/main`, même `CARGO_TARGET_DIR` que le
  clone pour ne pas recompiler les dépendances, un scénario par appel), puis la branche.
- **Hors de portée dans le cloud** : p2p à deux clients (Docker) et bench au calme
  (`ALACOD_BENCH_STRICT=1`). L'orchestrateur les rejoue sur la machine de William à la revue ;
  le rapport (§7) les liste comme « non vérifiés ici ».
- **Livraison** : commits sur la branche, `git push -u origin <tâche>`, rapport commité sur la
  branche, et la ligne `LIVRÉ <tâche> <sha>` en fin de réponse (pas de `SendMessage` depuis le
  cloud) : William la relaie à l'orchestrateur. Pas de PR nécessaire (merge local), une PR
  brouillon ne gêne pas.

## 2. Lire avant de coder

- `CLAUDE.md` (racine) : règles de déterminisme du rollback GGRS — **obligatoires** : nombres
  `Fixed` (jamais `f32` dans `GgrsSchedule`), itération par `order_iter!`/`order_mut_iter!` avec
  `&GgrsNetId` en premier dans la query, `BTreeMap`/`BTreeSet` (jamais `HashMap`), RNG par
  `RngStreams`/`RollbackRng` (jamais `rand::random`), `despawn_rollback()` (jamais `despawn()`),
  `FrameEvents<T>` (jamais de `Message` Bevy dans `GgrsSchedule`), enregistrement de tout état
  rollback par `app.rollback_and_trace_resource::<T>()` / `rollback_and_trace_component::<T>()`
  (`crates/utils/src/rollback.rs`), logs avec `GgrsNetId` (jamais `Entity`).
- `docs/conventions.md` : §1 entités LDtk, §2 RON (`Fixed` en chaînes : `"1.5"`), §3 manifeste
  `game.ron` et kinds de contenu, §10 protocole de preuve, sections « Run », « Power-ups », checklist
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
make test_crates                          # tout le workspace, headless sans features par défaut (sans la suite `scenarios`)
make test_tools                           # garde-fous et comparateurs Python, sans build
make lint                                 # alacod lint games/zombies et games/testbed : « aucune erreur »
cargo fmt --all -- --check                # rien à afficher ; sinon `make fmt`
./scripts/check-forbidden.sh              # strict ; exceptions exactes et justifiées, aucune nouvelle implicite
./scripts/check-rollback-registration.sh  # « OK »
make gen GAME=zombies                     # seulement si une arme a changé (scénarios générés)
ALACOD_BENCH_STRICT=1 make test_scenarios && ./scripts/scenario-metrics.py   # bench : à faire machine calme
```
Les comptes historiques de tests varient avec le contenu ; consulter le rapport du commit
revu et [les critères de sortie](../etat-engine.md), puis consigner les résultats réellement
obtenus. `check-forbidden.sh` échoue aussi si une exception devient périmée : retirer son entrée
ou adapter son compte avec justification, sans régénérer la liste pour accepter une régression.

**p2p à deux clients, avec rollbacks réels** (obligatoire quand la simulation, la session ou l'état
rollback change ; **remplace l'ancienne recette sans stress**, dont les deux clients ne rollbackaient
presque jamais : « frames resimulées : 0 à 3 », donc une comparaison de traces sans valeur) :
```bash
scripts/p2p-stress-inputs.py 3000 > /tmp/inputs.ron    # deux joueurs, entrées qui changent toutes les 1 à 3 frames
scripts/p2p-stress.sh zombies /tmp/inputs.ron 3000       # jeu, script d'inputs, frames (3000 par défaut)
```
Le script démarre et arrête `signaling` (bash ; `MATCHBOX_PORT`, `COMPOSE_PROJECT_NAME` et sous
OrbStack `PATH=$HOME/.orbstack/bin:$PATH`), compile le jeu en `headless`, lance deux clients
(`ALACOD_INPUT_DELAY=0`, `ALACOD_INPUT=script:`, `ALACOD_STATE_TRACE`, `ALACOD_EXIT_AT_FRAME`) et
suspend le client 1 par `SIGSTOP`/`SIGCONT` (12 ms toutes les 30 ms) : le client 0 le devance, prédit
faux et resimule. Il imprime les codes de sortie, les `DesyncDetected`, le **compteur de frames
resimulées lu dans le journal du jeu** (jamais les `ggrs{f=N}` en double : une ligne émise une fois par
exemplaire n'est pas une resimulation), le `cmp` des traces et le nombre de lignes comparées. Il échoue
(code 1) sur un code de sortie non nul, un `DesyncDetected`, des traces différentes ou moins de
`MIN_RESIMULATED` (1000) frames resimulées chez le client le plus resimulé. Autres réglages : `SUSPEND_MS`,
`RUN_MS`, `INPUT_DELAY`, `OUT`, `TIMEOUT`, `KEEP_SIGNALING` (en-tête du script). Attendu : « p2p-stress :
OK », traces de `frames - 1` lignes identiques (la trace n'écrit que des frames **confirmées** : à l'arrêt,
le client en avance attend la confirmation de ses dernières frames, jusqu'à 10 s).
Un système, un état rollback ou un vocabulaire qui n'apparaît que dans une situation précise (ramassage,
bombe, champion…) demande un montage qui l'y met (jeu ou carte de test **temporaire non commité**,
script d'inputs qui y mène : `p2p-stress-inputs.py --pool … --hold Bouton:de:à`) : sans cela le p2p ne prouve
rien de cette situation. Le stress de zombies seul ne couvre que le jeu de base.

**Restart en ligne** (D14, §33 ; quand la session, le lobby ou la sortie de partie changent) :
`./scripts/p2p-restart.sh` (démarre et arrête `signaling` lui-même). Attendu : « p2p partie 1 :
traces identiques », « p2p partie 2 : traces identiques », « partie 2 différente de la partie 1 »,
« local : partie 2 identique à la partie 1 » ; code de sortie 0.

**Run gungeon complète, N clients, réseau dégradé, déconnexion, deux machines** (m2-p2p-run-complete,
`docs/conventions.md` §57 « p2p réel de la run complète » ; commandes et mesures dans
`docs/taches/rapports/m2-p2p-run-complete.md`) :
```bash
scripts/p2p-gungeon-inputs.sh 2 10 5000 /tmp/run.ron      # inputs de 2 bots (graine 10, ~2,5 min)
ALACOD_SKIP_HUB=1 GAME_ARGS="--seed 10" EXPECT_FLOORS=1 scripts/p2p-stress.sh gungeon /tmp/run.ron 4500
CLIENTS=4 … scripts/p2p-stress.sh …                        # 4 clients (script à 4 pistes : `… 4 10 5000 …`)
NETEM="delay 60ms 10ms loss 1%" STRESS=0 … scripts/p2p-stress.sh …   # RTT 120 ms, sans root (`unshare -Urn` + `tc netem`)
scripts/p2p-disconnect.sh gungeon /tmp/run.ron 4500        # un pair tué en cours d'étage
RESTART_KIND=restart RESTART_AT=7200 EXIT_AT=1500 INPUTS=/tmp/run_full.ron … scripts/p2p-restart.sh gungeon
MATCHBOX_URL=wss://… LOBBY=x scripts/p2p-machine.sh gungeon /tmp/run.ron 4500 0   # un client par machine
```
Les cibles `make *_matchbox` ouvrent une fenêtre ; sans affichage, ces scripts (headless) les remplacent.
`P2P_WAIT_CALM=3` attend `uptime` < 3 avant de lancer les clients (machine partagée : sous charge extrême un pair était déconnecté, D87 ; délai porté à 5 s le 2026-10-18) ; `HUB=1` enregistre une run
par la porte du hub (`RESTART_KIND=hub EXPECT_VICTORY=1 … p2p-restart.sh` la rejoue : hub → run → victoire → hub) ; les inputs des bots sont décalés de 5 frames
(délai d'input du synctest, D85).

## 5. Traces et preuve (protocole `docs/conventions.md` §10)

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
- **Voie rapide (décision de l'orchestrateur, 2026-10-15)** : juste avant le commit `LIVRÉ`, fusionner
  `origin/main` dans la branche, puis rejouer la suite complète **sur cette fusion** et le dire au
  rapport (« suite verte sur la fusion avec main `<sha>` »). Si `main` n'a pas bougé depuis à la
  vérification, l'orchestrateur ne rejoue pas la suite : il relit, lance les vérifications courtes
  (fmt, lint, test_crates) et le p2p, puis merge. Si `main` a bougé, il refait la suite sur la
  nouvelle fusion, comme avant.
- Journal : une ligne dans le tableau de `docs/taches.md` §10, sur `main`, après le merge :
  `| AAAA-MM-JJ | T2.x (ce qui a été livré, en une phrase dense) | \`branche\` | modèle | mergée ; comment vérifié (scénarios, tests, preuve, bench, p2p) ; non fait ; dettes |`.

## 7. Rapport de fin (obligatoire, honnête)

1. Fait : fichiers, décisions et pourquoi.
2. Vérifié : chaque commande du §4 (et §5 si bless) **avec son résultat réel** (nombres).
3. Non fait / incertain / non vérifié (un scénario jamais vu passer est « non vérifié »).
4. Dettes laissées, questions ouvertes.
5. Atteignabilité (M2, `docs/taches.md` §7) : le scénario ou la graine `alacod-sim` d'une partie
   ordinaire du clone qui rencontre la mécanique livrée ; sinon, la fiche T14 qui la branchera,
   nommée comme dette.
Ne jamais écrire « complet » ou « vérifié » pour ce qui n'a pas tourné sous vos yeux.
