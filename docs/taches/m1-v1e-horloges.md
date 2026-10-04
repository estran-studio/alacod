# m1-v1e-horloges — horloges d'étage et de run, difficulté (T1.9, F2) + attente `Clock`

Lire d'abord `docs/taches/README.md`. Agent **local** (worktree `alacod_tasks/m1-v1e-horloges/`,
branche `m1-v1e-horloges`, target amorcé, `source ../env.sh`) **ou cloud** (variante README §1 :
clone froid, `CARGO_BUILD_JOBS=4`, pas de bench ni de p2p, livraison en bundle git via Syncthing
`1867_lore/tmp/`). Petite tâche (2 j), voie V1e run.

## Contexte

M1, Vague 1, voie V1e (plan §6, chantier F2 : « horloges d'étage et de run (nuit, marée, aube,
minuit, difficulté), événements planifiés, lisibles par l'UI »). Rien n'existe : pas de `Clock`,
pas de `Timeline`, pas de notion de difficulté ; le contrat T1.0b (`Clock` dans `crates/run`) n'a
jamais été posé.

Ce qui existe et qu'il faut connaître :
- `crates/run` : `Run { seed, mode: RunMode, step: RunStep, players, flags, summary }` (`run.rs`
  l.145), `rollback_and_trace_resource::<Run>()` ; `FloorState { index, anchor, portal_open,
  enemies_placed }` (`floors.rs` l.24) en `rollback_and_trace_resource_neutral` (défaut =
  contribution 0 au checksum, conventions §17) ; `RunContext { frame, current_wave, max_wave,
  kills, points_total, floor }` et `RunModeRules` (`modes.rs` l.231-263). Frame courante :
  `utils::frame::FrameCount` (`rollback_and_trace_copy_resource`, `game/src/core.rs` l.178),
  incrémentée dans le dernier set `FrameCounter` (`sim_core/src/system_set.rs`).
- Passage d'étage : `floor_transition_system` (`map_ldtk/src/game/floors.rs` l.353-505, set
  `Run`) mute `floor_state.index` ; **aucun `FrameEvents` n'est émis** ; l'étage 0 est posé hors
  simulation (`init_floor_state_when_map_loaded`, `Update`).
- F5 (m0-v11, conventions §18) : `content::expr` — `Expr::try_eval(&dyn Context)`,
  `Context` implémenté pour `BTreeMap<String, Fixed>`, `NumOrExpr { Integer, Literal,
  Expression }` dont `resolve(players: u32)` ne connaît que `players` ; le lint
  (`lint_num_or_expr`, `content/src/lint.rs` l.724) refuse tout autre identifiant ;
  `resolve_balance_system` évalue **une fois** sur `OnEnter(GameLoading)` dans `ResolvedBalance`
  (hors rollback) ; règle §18 : « jamais d'`Expr` dans l'état rollback ».
- **Dette découverte en préparant cette fiche** : `health_multiplier_per_wave` et
  `damage_multiplier_per_wave` (`wave_config.ron` l.52-53, 0,05/0,03) sont résolus, calculés
  dans `WaveState::current_health_multiplier`/`current_damage_multiplier` (au checksum)… et
  **lus par personne** : la difficulté du clone zombies ne monte que par le nombre d'ennemis et
  les paliers. Les appliquer changerait toutes les traces de vagues : **hors de cette tâche**,
  ligne de dette à ajouter (voir Décisions 6).
- Attentes : `Expectation` (`combat/src/weapons/expectations.rs` l.41, `at_frame()` l.291),
  `check()` dans `scenario/src/runner.rs` l.932 (`FloorIndex` l.1348 lit `run::FloorState`) ;
  `Event` lit `scenario::events::GameEvents` (détection par différence d'état dans `Last`,
  kinds `"portal"`, `"floor"`), pas `FrameEvents`.

L'attente `Clock` (liste T1.15) n'existe pas : **cette fiche la revendique**.

## Décisions (fixées ici, pas à réinventer)

1. **Ressource `run::Clock`** (crate `run`), **rollback + checksum neutre**
   (`rollback_and_trace_resource_neutral`, défaut = tout à zéro et aucun événement) :
   `run_started_frame: u32`, `floor_started_frame: u32`, `floor_index: u32`,
   `fired: BTreeSet<String>` (ids des événements déjà déclenchés dans la portée courante),
   `difficulty: Fixed` (défaut 1). Remise à zéro avec les autres ressources globales sur
   `OnExit(InGame)` (`cleanup_rollback_world_system`, conventions §13). Accesseurs purs :
   `run_frames(now)`, `floor_frames(now)`, `run_seconds`, `floor_seconds` (60 frames = 1 s).
2. **Nouvel étage** : `floor_transition_system` écrit `floor_started_frame = frame`,
   `floor_index = next`, vide `fired` des événements de portée `Floor` ; en plus il émet
   `FrameEvents<FloorEntered { index }>` (nouveau, `add_frame_events`, modèle
   `sim_core/src/frame_events.rs`) pour les futurs consommateurs (T1.10 `OnFloorEntered`).
   `init_floor_state_when_map_loaded` pose `run_started_frame = floor_started_frame = 0` (hors
   simulation, même valeur partout : déterministe).
3. **Événements planifiés = contenu** : kind `Clock` déclaré par `game.ron`
   (`(path: "clocks", kind: "Clock")`, conventions §3, checklist §4) ; fichiers
   `clocks/<nom>.ron` : `( scope: Run | Floor, events: [ (id: "nuit", at: Seconds("90")),
   (id: "minuit", at: Frames(9000)), (id: "renfort", at: Seconds("30"), repeat: Some(Seconds("30"))) ] )`.
   Un système `clock_system` (set `Run`, après `floor_transition_system`) parcourt les horloges
   du registre dans l'ordre des ids, déclenche chaque événement dont l'échéance (relative au début
   de sa portée) est atteinte et pas encore dans `fired` : insère l'id dans `fired`, émet
   `FrameEvents<ClockFired { id }>`, log `ggrs{f=… clock id=…}` ; `repeat` : l'id est suffixé
   `#n` (`"renfort#2"`) pour chaque répétition. Les actions attachées aux événements sont pour
   T1.10 (effets `OnClock`) : v1 ne fait **que** déclencher ; un jeu sans dossier `clocks` n'a
   aucun événement et sa `Clock` reste à la valeur par défaut (sauf en mode Floors, où
   `floor_started_frame`/`floor_index` bougent — voir Traces).
4. **Difficulté** : kind optionnel `Difficulty` (`difficulty.ron`, une expression
   `content::expr` : `( value: "1 + floor * 0.25 + minutes * 0.1 + (players - 1) * 0.2" )`).
   Identifiants nouveaux dans le contexte : `floor` (index d'étage), `minutes` et `seconds`
   (temps de **run**), `floor_minutes`/`floor_seconds` (temps d'étage), plus `players`.
   `clock_system` évalue l'expression **chaque seconde de simulation** (toutes les 60 frames,
   à partir de l'asset immuable — l'`Expr` ne vit pas dans l'état rollback, seul le résultat
   `Fixed` y est : conforme à §18, à préciser dans §18 d'une phrase) et range le résultat dans
   `Clock.difficulty`. **Consommateurs v1** : santé max des ennemis à l'apparition
   (`spawn_enemy` : `health_max × difficulty`, les deux chemins — vagues et `CharacterSpawn`)
   et dégâts infligés par les ennemis (multiplicateur appliqué au `DamageEvent` émis par un
   ennemi, un seul point : là où les dégâts ennemis sont construits). Sans kind `Difficulty`,
   `difficulty` vaut 1 et rien ne change (zombies : traces intactes).
   Lint (`crates/content`) : identifiants autorisés étendus pour **ce seul kind** (`players`
   reste le seul autorisé pour vagues/prix/santé F5), évaluation de l'expression pour
   `floor ∈ {0, 3}`, `minutes ∈ {0, 10}`, `players ∈ {1, 4}` ; valeur ≤ 0 = erreur ; `Clock` :
   ids uniques, échéances croissantes, `repeat` > 0.
5. **Attente `Clock { id, fired: bool, at_frame }`** dans `crates/scenario` (modèle
   `FloorIndex`, ré-export `game::replay`, liste `CLAUDE.md`, test unitaire) ; kind d'événement
   `"clock"` ajouté à `scenario::events::detect_events` (visible dans les vidéos).
6. **Dette D29** (une ligne dans `dettes.md`, la seule que la tâche y ajoute) : les
   multiplicateurs par vague du clone zombies sont calculés mais jamais appliqués ; décision à
   prendre par William (les appliquer = bless de toutes les traces de vagues avec preuve ; ou les
   retirer de `WaveState`, ce qui change aussi les traces par le checksum).
7. **Contenu et scénarios testbed** : `games/testbed/assets/clocks/arene.ron` (scope `Floor` :
   `"tic"` à 1 s, `"tac"` à 2 s, `"renfort"` toutes les 2 s à partir de 2 s) et
   `difficulty.ron` (`"1 + floor * 0.5 + floor_minutes * 0.5"`) ; scénarios :
   - `clock_events` (testbed, arène `arena` existante, un joueur immobile, 200 frames) :
     `Clock("tic", false)` à f59, `Clock("tic", true)` à f61, `Clock("tac", true)` à f121,
     `Clock("renfort#1", true)` à f181 ;
   - `clock_floor_reset` (testbed, séquence `trois_niveaux` de T1.8, portail franchi comme
     `portal_next_floor`) : `Clock("tic", true)` avant le portail, `Clock("tic", false)` juste
     après le passage (portée `Floor` remise à zéro), `FloorIndex(1)` ;
   - `difficulty_scales` (testbed, séquence de **2 étages** avec un `CharacterSpawn target` sur
     chaque carte) : `EntityHealth` de la cible = base à l'étage 0 et base × 1,5 à l'étage 1
     (santé résolue à l'apparition, `floor` dans l'expression).
   Trois nouvelles traces (bless orchestrateur).
8. **Hors périmètre** : actions des événements (T1.10), HUD « lisible par l'UI » (T1.18 : la
   source `clock` du HUD lira `Clock`), appliquer les multiplicateurs par vague (D29), nuit/marée
   (contenu 1837, M3), `games/throne` (T1.11).

## Traces attendues

- **Activation par scénario** : `clocks` et `difficulty` ne s'activent pas par le seul fait
  d'être déclarés dans `game.ron` ; un scénario les demande par des champs optionnels
  `clocks: Some(["arene"])` et `difficulty: Some(true)` (comme `floors`, `powerups` ; le jeu
  joué à la main les active par `game.ron` : `entry.clocks`, `entry.difficulty`). Ainsi seuls
  les trois nouveaux scénarios déclenchent des événements et évaluent la difficulté.
- Jeux et scénarios sans activation (tous les existants, zombies compris) : **aucune trace ne
  change** — ressource neutre, événement ajouté sans porteur, `difficulty` = 1.
- Exception : en mode Floors, `floor_started_frame`/`floor_index` bougent au passage d'étage,
  activation ou non → la trace de **`portal_next_floor`** change : preuve README §5 / §10
  (dumps main vs branche, `trace-diff.py --ignore Clock` = « identiques »), bless par
  l'orchestrateur. Toute autre trace qui change est un bug.

## Déterminisme (rappel, `CLAUDE.md`)

`Fixed` (évaluation `content::expr` en `Fixed`), `BTreeSet`/`BTreeMap`, ordre des ids, RNG
inutile, `FrameEvents<T>`, `rollback_and_trace_resource_neutral` pour `Clock`, logs
`ggrs{f=… clock …}`. Checklist « piège de parité » : aucun type ordinaire ajouté.

## Règles

- Deux compilations au plus sur la machine (`pgrep -x cargo`), `--profile headless`,
  `CARGO_BUILD_JOBS=4` ; **aucune trace bénie par l'agent** ; preuve pour chaque trace existante
  qui change (attendu : `portal_next_floor` seule).
- `docs/conventions.md` : **uniquement** un §23 « Horloges et difficulté (F2) » + une phrase dans
  §18 (l'`Expr` de difficulté est évaluée en simulation depuis l'asset, le résultat seul est
  rollback) ; `CLAUDE.md` : attente `Clock` dans la liste ; `dettes.md` : D29 seulement ;
  `docs/taches.md` : ne pas toucher.
- Merger `origin/main` juste avant de livrer ; conflits : garder les deux (b0 et b1 touchent
  `spawn_enemy`, `floors.rs`, `conventions.md`).

## Critères d'acceptation (vérifiés par l'orchestrateur sur l'état fusionné)

1. Tests unitaires : échéances (`Frames`/`Seconds`, `repeat`), remise à zéro de portée `Floor`,
   `Clock` par défaut neutre, contexte d'expression (`floor`, `minutes`, `floor_seconds`,
   `players`), lint (identifiants interdits hors `Difficulty`, ids dupliqués, valeur ≤ 0),
   attente `Clock`.
2. `clock_events`, `clock_floor_reset`, `difficulty_scales` verts ; scénarios existants verts
   sauf `portal_next_floor` (preuve `--ignore Clock` jointe) ; `alacod-sim --floors` toujours
   sans desync (5 graines).
3. Suite des crates verte, `make lint` (deux jeux), `make fmt`, scripts, `make gen` sans
   modification ; exemples de la racine compilés (`cargo build --example character_tester
   --profile headless --no-default-features`).
4. §23 écrit, D29 ajoutée ; rapport honnête.

## Livrer

Rapport `docs/taches/rapports/m1-v1e-horloges.md` sur la branche (README §7 ; sha de tête, base
`origin/main`). Commits en français avec attribution. Local : `git push -u origin
m1-v1e-horloges` puis `SendMessage` à `orch` : `LIVRÉ m1-v1e-horloges <sha> : <une ligne>`.
Cloud : bundle git dans `1867_lore/tmp/` via William + ligne `LIVRÉ …` en fin de réponse. Ne
merge pas, ne bénis pas.
