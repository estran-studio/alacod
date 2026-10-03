# m1-v1d — terrain destructible et cavernes (T1.0b + T1.6) + attente `CellState`

Lire d'abord `docs/taches/README.md` (agent **local** : la tâche compile, joue les scénarios et
le bench). Branche : `m1-v1d-terrain-destructible`, worktree
`alacod_tasks/m1-v1d-terrain-destructible/`, créés par l'orchestrateur (`scripts/task-new.sh`,
target amorcé, `source ../env.sh`).

## Contexte

M1, Vague 1, voie V1d monde (plan §6, chantiers E3 et contrats T1.0b). Aujourd'hui **tout** le
monde physique vient de LDtk : chaque carte (y compris `exemples/test_map.ldtk`, les arènes
testbed et `avant_poste`) passe par le callback du loader (`crates/map_ldtk/src/loader/mod.rs`
l.34-48) → `map_generation` (`crates/map/src/generation/mod.rs` l.106, gabarits LDtk assemblés
par `BasicMapGeneration`) → un `LdtkJson` en mémoire → les entités LDtk ; les murs deviennent des
colliders fusionnés gloutonnement (`crates/map_ldtk/src/game/collider.rs` : `spawn_level_walls`,
`generate_collision_rectangles`, `spawn_invisible_wall_collider` → entité `Wall` rollback avec net
id `ldtk_wall_WxH`) et remplissent `FlowFieldCache::intgrid_wall_cells`
(`crates/game/src/character/enemy/ai/navigation.rs` l.479 `load_intgrid_walls`, « immutable
after load »). Le flow field (`update_flow_field_system` l.576, Dijkstra multi-source complet dans
`build_flow_field` l.777) ne se reconstruit que si les cibles ou les obstacles changent, au plus
toutes les 30 frames. La grille de collision des murs (`crates/combat/src/collision_grid.rs`,
`maybe_rebuild_wall_grid` l.115) se reconstruit sur une signature (nombre de murs + somme des
`GgrsNetId`).

Aucun des types prévus par T1.0b n'existe : pas de crate `world`, pas de `CellKind`, `CellGrid`,
`Destructible`, ni de `RollbackSystemSet::World` (`crates/sim_core/src/system_set.rs` : `ORDER`
à 15 entrées). Les explosions de T1.1 (`crates/combat/src/projectile.rs` l.1-29 : projectile de
vitesse nulle, `Lifetime(0)`, `Size`, `Pierce`) touchent les personnages via
`FrameEvents<ProjectileHit>`, mais **un mur touché n'émet aucun événement**
(`projectile.register_wall()`), et `effects::Action` (`crates/effects/src/actions.rs` l.27 :
`TimedModifier`, `RefillAmmo`, `RepairAllWindows`, `KillAllWaveEnemies`, `CurrencyMultiplier`)
n'a aucune action de terrain. `DamageKind::Explosion` existe (`crates/sim_core/src/damage.rs`).

L'attente `CellState` (liste T1.15) n'existe pas : **cette fiche la revendique**.

## Décisions (fixées ici, pas à réinventer)

1. **Crate `crates/world`** (nouveau, dans le workspace, dépend de `sim_core`, `utils`,
   `bevy_fixed`) :
   - `CellKind { Floor, Wall, Rock }` : `Wall` indestructible (bordure), `Rock` destructible.
   - `CellGrid { width, height, cells: Vec<CellKind> }`, case de **16** unités (la même que
     `GRID_CELL_SIZE` de la navigation et que les tuiles LDtk, conventions §1), origine en
     (0,0) comme les mondes LDtk ; ressource **rollback + checksum + trace** enregistrée en
     `rollback_and_trace_resource_neutral` (valeur par défaut = grille vide = contribution 0 au
     checksum : **les 80 traces existantes ne changent pas par construction**, comme
     `FloorState` dans m1-v1e, conventions §17). Le hash couvre toutes les cases.
   - `Destructible` : composant marqueur du *monde* (pas d'une entité par cellule) indiquant
     que la carte courante est une caverne dont les murs suivent `CellGrid`.
   - `RollbackSystemSet::World` inséré dans `ORDER` **après `Projectiles` et avant** le set qui
     calcule le flow field (`.before(EnemyAI)`, voir `crates/game/src/character/mod.rs`
     l.222-224) : la destruction d'une frame est vue par la navigation de la même frame.
     Insertion d'un set = l'ordre des autres ne bouge pas ; preuve : traces inchangées.
2. **Générateur de cavernes** `world::cave::generate(seed: u64, config: &CaveConfig) ->
   CellGrid`, fonction pure, RNG `RollbackRng` (jamais `rand::random`), **automate
   cellulaire** : remplissage aléatoire (`fill_ratio`), N itérations de la règle
   (naissance ≥ 5 voisins rock, survie ≥ 4 — valeurs dans `CaveConfig`, pas dans le code),
   bordure forcée en `Wall`, puis **connexité garantie** : flood fill, on garde la plus grande
   composante de `Floor`, les autres deviennent `Rock`. `CaveConfig` est un kind RON
   (`width`, `height`, `fill_ratio`, `iterations`, `birth`, `survive`, `min_floor_ratio`,
   `enemy_spawns`) déclaré dans `game.ron` du testbed (`(path: "caves", kind: "Cave")`,
   conventions §3, checklist §4), lint compris.
   Points d'intérêt déterministes : `PlayerSpawn` = la case `Floor` la plus proche du centre
   (puis les 3 suivantes pour 4 joueurs, ordre fixe), `ZombieSpawn` × `enemy_spawns` = les
   cases `Floor` les plus éloignées du spawn joueur (distance de grille), espacées d'au moins
   8 cases.
3. **Branchement comme niveau — par le chemin LDtk existant, pas un second chemin.** La caverne
   est un `MapGenerationMode::Cave(config)` à côté de `Basic` (`crates/map/src/generation/
   config.rs`, `imp/mod.rs`) qui produit un `LdtkJson` **en mémoire** : une couche IntGrid
   `Walls` (1 = `Wall` ou `Rock`), les entités `PlayerSpawn{index}` et `ZombieSpawn` (format
   conventions §1), un seul niveau. Tout le reste (colliders, `load_intgrid_walls`, spawns,
   mode Floors : une caverne est un niveau valide d'une séquence `Floors` et de `--floors`) suit
   **sans modification**. Au `LdtkMapLoadingEvent` d'une caverne, un système remplit `CellGrid`
   depuis les mêmes cases et pose `Destructible`. Une carte LDtk ordinaire laisse `CellGrid`
   vide.
   Déclenchement : `start_map`/`Scenario.map`/`--map` prennent déjà un chemin ; une caverne se
   désigne par `cave:<nom>` (ex. `cave:petite`) ou, en Floors, par une entrée
   `Cave("petite")` dans `levels` — choisir **une** syntaxe, l'écrire au § conventions. La
   graine de la caverne = `map_seed` du scénario (même source que `Basic`).
4. **Destruction par explosion.** Nouvelle action `effects::Action::DestroyTerrain { radius }`
   (rayon en unités, `Fixed` sérialisé en chaîne, conventions §2), exécutée par la crate
   `world` : toute case `Rock` dont le centre est à moins de `radius` de la position de
   l'action devient `Floor` ; `Wall` ne change jamais. Elle s'utilise dans `on_expire` (une
   grenade de T1.1 : `on_expire: [DestroyTerrain(radius: "40")]`) **et** dans `on_hit`.
   Comme un mur touché n'émet rien aujourd'hui, ajouter `FrameEvents<ProjectileWallHit>`
   (projectile, position, actions) émis dans la branche `register_wall()` du projectile — un
   projectile qui frappe un mur de caverne avec `on_hit: [DestroyTerrain]` creuse. Ajouter un
   événement = 0 changement de trace (preuve : les 80 inchangées).
5. **Colliders et navigation après destruction.** Les murs d'une caverne sont les entités
   `Wall` fusionnées produites par le chemin LDtk ; après une destruction (événement rare), le
   système `World` **détruit (`despawn_rollback`) et recrée** les colliders de murs de la caverne
   à partir de `CellGrid` (même fusion gloutonne `generate_collision_rectangles`, net ids
   déterministes `cave_wall_<frame>_<i>` par la factory), puis **resynchronise**
   `intgrid_wall_cells` (nouvelle méthode `FlowFieldCache::reload_walls`, la seule exception à
   « immutable after load ») : la signature de `CollisionGrids::walls` et la détection
   d'obstacles de `rebuild_blocked_cells` font le reste (reconstruction complète du flow field
   au prochain pas, ≤ 30 frames). C'est l'« incrémental » de v1 : coût nul hors destruction.
   Un flow field vraiment incrémental **n'est à faire que si `bench_cave` ne tient pas son
   budget** (voir acceptation) — le dire dans le rapport. Alternative « un collider par
   cellule » rejetée (des milliers d'entités rollback au checksum).
6. **Attente `CellState(x, y, kind)`** dans `crates/scenario` (sur le modèle de `FloorIndex`,
   `crates/scenario/src/expectations/` ; ré-export dans `game::replay` et la liste des attentes
   de `CLAUDE.md`), testée unitairement.
7. **Scénarios** (testbed, nouvelle caverne `petite` ~48×32, graine fixée) :
   - `explode_wall` : un joueur immobile tire une grenade `on_expire: [DestroyTerrain]` vers un
     mur `Rock` ; `CellState` affirme `Rock` avant et `Floor` après, `EntityCount` des murs
     change (ou `CellState` sur 2-3 cases du cratère), ~600 frames ;
   - `bench_cave` : 4 joueurs qui lancent des grenades en continu, **50 destructions** au
     moins pendant la partie, ennemis qui naviguent ; budget dans `tests/budgets.ron` au
     plancher par défaut (40 fps simulés), `ALACOD_BENCH_STRICT=1` pour le mesurer au calme.
   Deux nouvelles traces = pas de preuve requise (nouvelles), bénies par l'orchestrateur.
8. **Hors périmètre** : surfaces et tags de cellules (T1.7, E4), `games/throne` (T1.0c, V2),
   bots sur caverne (`--until-floor`, T1.14), rendu des cavernes au-delà du strict minimum
   (les tuiles `Walls` de l'IntGrid se dessinent déjà par le chemin LDtk ; une case détruite
   doit disparaître à l'écran : lire `CellGrid` côté présentation, pas d'état de rendu propre).

## Déterminisme (rappel, `CLAUDE.md`)

`Fixed` partout dans `GgrsSchedule`, `BTreeMap`, `order_iter!` avec `&GgrsNetId` en tête,
RNG par `RollbackRng`/`RngStreams`, `despawn_rollback()`, `FrameEvents<T>`, enregistrement par
`rollback_and_trace_*` (`crates/utils/src/rollback.rs`), logs avec `GgrsNetId`. Le générateur
et la destruction ne lisent jamais de `f32`.

## Règles du worktree

- Une compilation à la fois pour toi, deux au plus sur la machine (b0 travaille en parallèle
  sur la voie V1a) : `pgrep -x cargo` avant de lancer, `--profile headless`,
  `CARGO_BUILD_JOBS=4`, `source ../env.sh`, jamais depuis un target vide. Les `.full` des
  dumps font 300 Mo à 1 Go : les supprimer au fur et à mesure (disque serré).
- **Aucune trace bénie par l'agent.** Attendu : les 80 traces existantes **inchangées** (ressource
  neutre, chemin LDtk intact, set `World` vide pour une carte LDtk). Si un changement s'avère
  nécessaire, preuve `trace-diff` (README §5) et justification dans le commit ; l'orchestrateur
  bénit.
- `docs/conventions.md` : ajouter **uniquement** un §21 « Terrain destructible et cavernes »
  (§19 statuts et §20 patterns sont pris par d'autres branches) : kinds (`Cave`, `CellKind`),
  syntaxe de désignation d'une caverne, règles de l'automate, points d'intérêt, action
  `DestroyTerrain`, événement mur, reconstruction des colliders, attente `CellState`, limites.
  `docs/plan-engine.md` et `docs/taches.md` : ne pas toucher (journal de l'orchestrateur).
- Merger `origin/main` dans la branche juste avant de livrer (règle m0-v9) ; en cas de conflit,
  garder les deux côtés. Ne pas merger dans main, ne pas supprimer le worktree.

## Critères d'acceptation (vérifiés par l'orchestrateur sur l'état fusionné)

1. Tests unitaires de `world::cave` : **1 000 graines** → `Floor` connexe (une seule
   composante), bordure `Wall`, ratio de sol dans `[min_floor_ratio, 0.9]`, spawns sur `Floor`,
   même graine ⇒ même grille, graines différentes ⇒ grilles différentes ; test de
   `DestroyTerrain` (rayon, `Wall` intact) ; test de `CellState`.
2. `explode_wall` vert (cratère observé par `CellState`), `bench_cave` vert au calme avec
   ≥ 50 destructions (compteur dans les métriques ou une attente), chiffre dans le rapport.
3. Les 80 scénarios existants verts, **traces inchangées** (ou preuve jointe) ; mode Floors :
   une séquence avec une caverne se joue (`alacod-sim --floors` ou un test).
4. Suite des crates verte (dont `world`), `make lint` (le testbed déclare `Cave`), `make fmt`,
   `make scripts` (aucune nouvelle occurrence), `make gen GAME=zombies` sans modification.
5. §21 écrit ; rapport honnête.

## Livrer

Rapport `docs/taches/rapports/m1-v1d-terrain-destructible.md` sur la branche (README §7 : fait /
vérifié avec chiffres / non fait / dettes ; sha de tête, base `origin/main`). Commits en
français avec attribution. `git push -u origin m1-v1d-terrain-destructible`, puis `SendMessage`
à `orch` : `LIVRÉ m1-v1d-terrain-destructible <sha> : <une ligne>` (ou `BLOQUÉ … : <pourquoi>`
après 30 minutes de blocage). Ne merge pas, ne bénis pas, attends la réponse.
