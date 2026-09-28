# CLAUDE.md - Alacod Engine Context

## Vision

Alacod est un **engine 2D modulaire** pour roguelikes/shooters inspiré de :
- Binding of Isaac
- Enter the Gungeon
- Nuclear Throne
- Call of Duty Zombies

L'objectif est de créer un moteur **data-driven** où les comportements sont définis en fichiers RON et assemblés à partir de modules codés dans l'engine.

## Architecture Data-Driven

### Philosophie
- **Code = Behaviors/Systems** : L'engine fournit des behaviors réutilisables
- **RON = Configuration/Assembly** : Les designers assemblent les behaviors sans coder
- **Plugins = Extensions** : Nouveaux behaviors peuvent être ajoutés via plugins

### Pattern Actuel (Armes)

```ron
// weapons.ron - Exemple de composition
"shotgun": (
    config: (
        firing_mode: Shotgun(pellet_count: 8, spread_angle: "0.4"),
        bullet_type: Piercing(damage: "10.0", penetration: 1),
        // ...
    )
)
```

Les enums `Shotgun`, `Piercing`, `Combo`, etc. sont des **behaviors** codés en Rust.

## Contraintes Techniques

### Déterminisme (GGRS Rollback) - RÈGLES CRITIQUES

Le rollback GGRS exige que **tous les clients produisent exactement les mêmes résultats** pour les mêmes inputs. Toute source de non-déterminisme cause des desyncs.

#### 1. Fixed-Point Math UNIQUEMENT
```rust
// ❌ INTERDIT - f32/f64 dans GgrsSchedule
let speed: f32 = 10.0;
let pos = pos + Vec2::new(speed, 0.0);

// ✅ CORRECT - Fixed point
let speed: Fixed = fixed_math::new(10.0);
let pos = pos + FixedVec2::new(speed, Fixed::ZERO);
```

#### 2. Itération Ordonnée par GgrsNetId - OBLIGATOIRE

**JAMAIS** itérer sur une Query sans ordonner si le résultat affecte le rollback.

```rust
// ❌ INTERDIT - Ordre non-déterministe
for (entity, component) in query.iter() { ... }
for item in query.iter().next() { ... }  // Sélection aléatoire!

// ✅ CORRECT - Utiliser les macros (GgrsNetId DOIT être en premier dans la Query)
use utils::{order_iter, order_mut_iter};

// Query avec GgrsNetId EN PREMIER
Query<(&GgrsNetId, Entity, &mut Transform, ...), With<Enemy>>

for (net_id, entity, transform, ..) in order_iter!(query) { ... }
for (net_id, entity, mut transform, ..) in order_mut_iter!(query) { ... }
```

#### 3. JAMAIS Entity.to_bits() pour Ordonner

```rust
// ❌ INTERDIT - Entity IDs diffèrent entre clients!
entities.sort_by_key(|e| e.to_bits());

// ✅ CORRECT - Utiliser GgrsNetId.0
entities.sort_by_key(|(net_id, _)| net_id.0);
```

#### 4. Sélection Déterministe (Plusieurs Candidats)

Quand on doit choisir UN élément parmi plusieurs (ex: joueur le plus proche):

```rust
// ❌ INTERDIT - Premier dans l'ordre arbitraire
let target = player_query.iter().next();

// ✅ CORRECT - Trier puis prendre le premier
let mut players: Vec<_> = player_query.iter().collect();
players.sort_by_key(|(net_id, _)| net_id.0);
let target = players.first();

// ✅ CORRECT - Tie-breaking déterministe pour "plus proche"
let should_update = match &closest {
    None => true,
    Some((closest_id, closest_dist)) => {
        distance < *closest_dist ||
        (distance == *closest_dist && net_id.0 < closest_id.0)  // Tie-break par net_id
    }
};
```

#### 5. Collections Déterministes

```rust
// ❌ INTERDIT - HashMap/HashSet ont un ordre d'itération aléatoire
use std::collections::{HashMap, HashSet};

// ✅ CORRECT - BTreeMap/BTreeSet garantissent l'ordre
use std::collections::{BTreeMap, BTreeSet};

// Note: Les types clés doivent implémenter Ord
#[derive(PartialOrd, Ord)]  // Ajouter ces derives
pub enum MyType { ... }
```

#### 6. RNG Déterministe

```rust
// ❌ INTERDIT - RNG système
use rand::random;

// ✅ CORRECT - RollbackRng (synchronisé par GGRS)
fn my_system(mut rng: ResMut<RollbackRng>) {
    let value = rng.next_fixed();  // Déterministe
}

// IMPORTANT: Consommer RNG dans un ordre déterministe (après tri par net_id)
```

#### 7. Pas de Conversion f32 dans la Simulation

```rust
// ❌ DANGER - Perte de précision, résultats peuvent varier
let fixed_val = Fixed::from_num(some_fixed_wide.to_num::<f32>());

// ✅ CORRECT - Rester en fixed-point
let fixed_val = Fixed::from_fixed_wide(some_fixed_wide);
```

#### 8. Logging Déterministe (pour comparaison des traces)

Les logs GGRS doivent être comparables entre clients. **JAMAIS** logger des valeurs non-déterministes.

```rust
// ❌ INTERDIT - Entity IDs diffèrent entre clients, logs incomparables
info!("Enemy {:?} attacked at frame {}", entity, frame);
info!("Spawned hitbox for entity {:?}", entity);

// ✅ CORRECT - Utiliser GgrsNetId (identique sur tous les clients)
info!("Enemy {} attacked at frame {}", net_id, frame);
info!("Spawned hitbox for {}", net_id);

// ✅ CORRECT - Autres valeurs déterministes acceptées
info!("Player {} attacked", player.handle);  // Handle GGRS
info!("Frame {}: damage {} applied", frame.frame, damage);  // Valeurs de jeu
```

**Pourquoi?** On compare les logs entre clients avec `diff` pour détecter les desyncs.
Si les logs contiennent des Entity IDs, le diff montrera des différences même si la simulation est synchronisée.

#### 9. Format GGRS Trace Logs (pour diff_log Makefile)

Les logs utilisés pour comparaison entre clients doivent suivre un format précis compatible avec `make diff_log`.

**Format obligatoire:**
```
ggrs{f=FRAME system_name key=value key=value...}
```

**Règles:**
1. Préfixe `ggrs{` - requis pour le grep filter du Makefile
2. `f=FRAME` - numéro de frame en premier (requis pour le filtre perl)
3. `system_name` - nom descriptif du système (sans `=`)
4. `key=value` - paires clé-valeur pour les données
5. Fermeture `}` - fin du log

**Exemples corrects:**
```rust
// État de jeu (info! pour événements importants)
info!(
    "ggrs{{f={} wave_system phase=GracePeriod wave={} enemies={}}}",
    frame.frame, wave_state.current_wave, enemy_count
);

// Mouvement IA (trace! pour logs haute fréquence)
trace!(
    "ggrs{{f={} ai_move net_id={} pos=({},{}) vel=({},{})}}",
    frame.frame, net_id.0, pos.x, pos.y, vel.x, vel.y
);

// Événement ponctuel
trace!(
    "ggrs{{f={} ai_attack net_id={} target=window dist={}}}",
    frame.frame, net_id.0, distance
);
```

**Note:** Double accolades `{{` `}}` dans le format string Rust pour produire `{` `}` littéraux.

**Utilisation:**
```bash
# Comparer les logs entre deux clients
make diff_log CID_1=alice CID_2=bob
```

### Checklist pour Nouveau Système GGRS

- [ ] Query a `&GgrsNetId` en PREMIER si itération affecte l'état
- [ ] Utilise `order_iter!` ou `order_mut_iter!` pour itérer
- [ ] Trie par `net_id.0` (pas `entity.to_bits()`) avant traitement
- [ ] Utilise `BTreeMap`/`BTreeSet` (pas `HashMap`/`HashSet`)
- [ ] Tie-breaking déterministe quand plusieurs candidats à distance égale
- [ ] Pas de `.iter().next()` sans tri préalable
- [ ] Pas de `f32`/`f64` - uniquement `Fixed`/`FixedWide`
- [ ] RNG via `RollbackRng` consommé dans ordre déterministe
- [ ] Resource registered avec `rollback_resource_with_clone` si mutable
- [ ] Logs utilisent `GgrsNetId`/`player.handle` (pas `Entity`) pour comparaison
- [ ] Trace logs suivent format `ggrs{{f={} system_name key=value...}}` pour diff_log

### Schedules
- `GgrsSchedule` : Simulation rollback (tout le gameplay)
- `PostUpdate` : Sync visual (`FixedTransform3D` -> `Transform`)

#### Événements dans la simulation
**Jamais de `Message` bevy (`MessageReader`/`MessageWriter`) dans `GgrsSchedule`** : ils ne sont pas
dans les snapshots et leurs curseurs ne sont pas rollbackés. Utiliser `FrameEvents<T>`
(`crates/game/src/frame_events.rs`, `app.add_frame_events::<T>()`) : file vidée au début de chaque
frame (`RollbackSystemSet::FrameStart`), lue par les systèmes ordonnés après l'émetteur.

Les visuels (portes, barres de vie, game over) se **dérivent de l'état** dans `Update`, jamais
d'un événement émis par la simulation : ils restent justes après un rollback.

#### Simulation et présentation
La simulation ne dépend jamais du rendu. Tout ce qui sert à afficher (caméra, lumière, audio,
UI de debug) va dans `PresentationPlugin` (`core.rs`), absent en headless.

## Headless, trace d'état et déterminisme

Variables d'environnement (natif) :
- `ALACOD_HEADLESS=1` : sans fenêtre ni GPU, une frame GGRS par update. Compiler sans le rendu des
  tilemaps : `--no-default-features` (et `--profile headless` pour la vitesse).
- `ALACOD_INPUT=neutral` : ignore clavier et souris (sinon la position du curseur entre dans l'input).
- `ALACOD_STATE_TRACE=<fichier>` + `ALACOD_EXIT_AT_FRAME=<n>` : hash de l'état rollback à chaque frame,
  puis arrêt. `ALACOD_STATE_TRACE_FULL=1` ajoute l'état détaillé pour trouver une divergence.

Avant/après un refactoring de la simulation, comparer les traces : elles doivent être identiques.

## Scénarios (tests de comportement et de régression)

`crates/scenario` joue des parties scriptées en headless, dans le processus de test :
- `tests/scenarios/<nom>.ron` : map, seed, un script d'inputs par joueur (segments de frames avec
  boutons et visée), nombre de frames, attentes à une frame donnée (`PlayerAlive`, `PlayerDead`,
  `WaveAtLeast`, `KillsAtLeast`, `WindowsBrokenAtLeast`, `WindowHealth`, `DoorsOpenAtLeast`,
  `ActiveWeapon`, `Ammo`, `PlayerPosition`). Format documenté dans `crates/game/src/replay.rs`.
- `tests/scenarios/<nom>.trace` : trace d'état de référence. Toute différence fait échouer le test.
- `make test_scenarios` (profil `headless`, sans rendu) ; `SCENARIO=<nom>` pour un seul ;
  `BLESS=1` pour réécrire les traces après un **changement de gameplay voulu** (le dire dans le commit).
- `make play_scenario SCENARIO=<nom>` : affiche le scénario avec rendu, mêmes inputs.

### Jouer et enregistrer
- **Contrôle remote** (`game::remote`, `ALACOD_REMOTE=1`) : `make remote` (ou `make remote HEADLESS=1`)
  lance la partie en pause ; `scripts/alacod-remote` la pilote : `brief`/`state` (joueurs, ennemis
  avec dx/dy, vague, fenêtres, portes), `input Fire --pan 100,0`, `step 30` (avance puis affiche),
  `screenshot`, `save <fichier.ron>`, `pause`/`resume`.
- **Enregistrement** (`game::recording`) : les inputs réellement envoyés à GGRS sont capturés à chaque
  frame ; `save` (remote) ou `ALACOD_RECORD=<fichier>` (écrit à la fermeture, ex.
  `make record_session NAME=x`) produit un scénario rejouable. Ajouter des `expect`, puis
  `make test_scenarios SCENARIO=<nom> BLESS=1`.

### Vidéos (validation visuelle)
`scripts/scenario-video` rejoue les scénarios avec rendu et capture chaque frame (image n = frame n,
960×540 hors écran, indépendant de la fenêtre), puis encode avec ffmpeg :
- `make videos [SCENARIO=<nom>]` : une vidéo par scénario + `montage.mp4` (grille), dans
  `target/videos/<commit>/` ;
- `make compare_video SCENARIO=<nom> BASE=<réf> [HEAD=<réf>]` : avant/après côte à côte, le même
  scénario joué par le code des deux références (worktree git, target partagé). La référence doit
  contenir `play_scenario --capture`.
- Moments clés : pendant la capture, `crates/scenario/src/events.rs` détecte vague, kills, coups reçus,
  morts, rechargements, changements d'arme, fenêtres cassées/réparées, portes ouvertes ; écrits dans
  `<scénario>.events.json`, affichés en pastilles cliquables sous chaque vidéo. En test :
  `ALACOD_EVENTS=1 make test_scenarios` les affiche.
- Chaque scénario commence par un commentaire `// À regarder : ...` (ce qu'on doit voir, avec les frames),
  affiché en tête de sa vidéo. Le mettre à jour quand le comportement change.
- `make review_videos [TAILSCALE=1]` : page de revue (`target/videos/index.html`, regénérée après chaque rendu) sur
  http://localhost:8766 : commits et comparaisons, lecture synchronisée image par image, notes par
  vidéo. Servie par `scripts/scenario-review.py --serve` (requêtes Range, requises pour se
  positionner dans les vidéos).

La partie jouée est celle de `map_explorer` (plugin partagé `map_ldtk::game::local::LdtkLocalGamePlugin`).
Tout changement de simulation doit garder les scénarios verts, ou justifier le `BLESS`.

### Numérotation des entités (`GgrsNetId`)
Les ids doivent être attribués dans un ordre indépendant du timing et de l'allocation des `Entity` :
trier par une clé de contenu (type, position, level iid) avant `id_factory.next`, et ordonner tout
système qui crée des entités rollback sur `LdtkMapLoadingEvent` `.after(MapNetIdAssignment)`.

## Système IA (En Refonte)

### Problème Actuel
- `ZombieState` hardcode les comportements spécifiques (Window, Player)
- Pathfinding individuel par ennemi (coûteux pour hordes)

### Nouvelle Architecture

#### Flow Field Navigation
```rust
// Shared pathfinding - O(1) lookup per enemy
// GGRS: Utilise BTreeMap pour itération déterministe
#[derive(Resource, Clone)]  // Clone requis pour rollback
pub struct FlowFieldCache {
    pub layers: BTreeMap<NavProfile, FlowField>,
    pub blocked_cells: BTreeMap<ObstacleType, BTreeSet<GridPos>>,
    pub wall_cells: BTreeSet<GridPos>,
    // ...
}

// GGRS: Ord requis pour BTreeMap
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum NavProfile {
    Ground,        // Respecte tous obstacles
    GroundBreaker, // Ignore obstacles cassables
    Flying,        // Ignore Water/Pit
    Phasing,       // Ignore tout sauf Wall
}
```

#### Obstacles Génériques
```rust
#[derive(Component)]
pub struct Obstacle {
    pub obstacle_type: ObstacleType,
    pub blocks_movement: bool,
    pub allows_attack_through: bool,
    pub breakable: bool,
}

pub enum ObstacleType {
    Wall,      // Jamais cassable
    Window,    // Cassable, permet attaque through
    Barricade, // Cassable, bloque attaque
    Water,     // Bloque Ground, pas Flying
    Pit,       // Bloque tout sauf Flying
}
```

#### Configuration Ennemi (RON)
```ron
// enemies/zombie_runner.ron
(
    movement: (...),
    collider: (...),

    // NEW: AI Configuration
    ai: (
        movement_type: Ground,
        aggro_range: "300.0",
        attack_range: "35.0",

        // Obstacles que cet ennemi peut casser
        can_break: [Window, Barricade],

        // Peut attaquer à travers ces obstacles
        attack_through: [Window],

        // Ignore ces obstacles pour le pathfinding
        ignores: [],
    ),

    // Behaviors additionnels (composables)
    behaviors: [
        ChasePlayer,
        AttackMelee(weapon: "zombie_claws"),
        BreakObstacles,
    ],
)

// enemies/ghost.ron
(
    ai: (
        movement_type: Phasing,  // Passe à travers tout sauf Wall
        ignores: [Window, Barricade, Water, Pit],
        can_break: [],
        attack_through: [Window, Barricade],
    ),
    behaviors: [
        ChasePlayer,
        AttackMelee(weapon: "ghost_touch"),
    ],
)

// enemies/flying_demon.ron
(
    ai: (
        movement_type: Flying,
        ignores: [Water, Pit],
        can_break: [],
        attack_through: [],
    ),
    behaviors: [
        ChasePlayer,
        AttackRanged(projectile: "fireball"),
        KeepDistance(min: "100.0", max: "200.0"),
    ],
)
```

#### State Machine Générique
```rust
#[derive(Component)]
pub enum MonsterState {
    Idle,
    Chasing,
    Attacking { target: AttackTarget, last_attack_frame: u32 },
    Stunned { recover_at: u32 },
    // Extensible via plugin
}
```

### Behaviors Engine (Futur)

Liste des behaviors planifiés :
- `ChasePlayer` - Suit le joueur via FlowField
- `ChaseClosest` - Suit l'entité la plus proche (player ou autre)
- `Wander` - Errance aléatoire
- `Patrol(waypoints)` - Patrouille entre points
- `AttackMelee(weapon)` - Attaque corps à corps
- `AttackRanged(projectile)` - Attaque à distance
- `KeepDistance(min, max)` - Maintient une distance
- `BreakObstacles` - Casse les obstacles sur son chemin
- `FleeWhenLowHealth(threshold)` - Fuit si HP bas
- `CallReinforcements` - Appelle d'autres ennemis
- `Explode(on_death, radius, damage)` - Explose à la mort

## Debug Systems

### Flow Field Visualization
```rust
#[derive(Resource)]
pub struct FlowFieldDebug {
    pub enabled: bool,
    pub show_grid: bool,
    pub show_arrows: bool,
    pub show_costs: bool,
}
```
- Toggle via touche (ex: F3)
- Flèches colorées par distance au target
- Affiche les cellules bloquées en rouge

### Enemy State Debug
- Affiche l'état actuel au-dessus de l'ennemi
- Montre la cible actuelle
- Visualise l'aggro range

## Structure des Fichiers

```
crates/game/src/character/enemy/
├── mod.rs
├── create.rs              # Spawn enemies from RON config
├── spawning.rs            # Spawner logic
└── ai/
    ├── mod.rs             # Re-exports + legacy modules
    ├── pathing.rs         # Cibles et déplacement des ennemis (move_enemies)
    ├── navigation.rs      # [NEW] FlowField, GridPos, NavProfile
    ├── obstacle.rs        # [NEW] Generic Obstacle component
    ├── state.rs           # [NEW] MonsterState, EnemyAiConfig
    ├── behavior.rs        # [NEW] Behavior systems
    └── debug.rs           # [NEW] Debug visualization (F3/F4/F5)
```

## Debug Keys

- **F3** : Toggle Flow Field visualization
- **F4** : Cycle NavProfile (Ground → GroundBreaker → Flying → Phasing)
- **F5** : Toggle Enemy State visualization

## Collision Layers

```
Layer 1: Enemy
Layer 2: Environment
Layer 3: Player
Layer 4: Wall
Layer 5: Window
Layer 6: Bullet
```

Matrix définit qui collide avec qui. Les obstacles ont leur propre layer selon type.

## Notes Importantes

### GGRS - Règles de Base (MÉMORISER)
1. **Jamais de f32 dans GgrsSchedule** - Utiliser `Fixed` partout
2. **Jamais `.iter().next()`** - Trier par `net_id.0` puis `.first()`
3. **Jamais `entity.to_bits()` pour trier** - Utiliser `net_id.0`
4. **Jamais `HashMap`/`HashSet`** - Utiliser `BTreeMap`/`BTreeSet`
5. **Toujours `order_iter!`/`order_mut_iter!`** - Pour queries qui affectent l'état
6. **Query: `&GgrsNetId` EN PREMIER** - Requis pour les macros
7. **Jamais logger `Entity`** - Utiliser `GgrsNetId` pour logs comparables

### Autres
8. **RON strings pour Fixed** - ex: `"100.0"` pas `100.0`
9. **Behaviors sont composables** - Un ennemi peut avoir plusieurs behaviors
10. **FlowField par NavProfile** - Pas par ennemi individuel
11. **Clone + rollback_resource_with_clone** - Pour Resources mutables dans GgrsSchedule

## Flow Field - Implémentation Actuelle

Le système utilise **Dijkstra** vers le joueur (le premier par `net_id`), déterministe (tas trié
par coût puis case) :

- **Cellules** : 16 unités, 1:1 avec les tuiles LDtk (`GRID_CELL_SIZE`)
- **Couverture** : toute la map (boîte englobante des murs + marge), pas un rayon autour du joueur :
  chaque spawner doit être couvert
- **Cases bloquées** : murs IntGrid, **portes fermées** (une porte bloque tant qu'elle a un collider ;
  une porte non interactive ne s'ouvre jamais), obstacles selon le profil (fenêtres intactes pour `Ground`)
- **Cases trop étroites** : bloquées des deux côtés opposés (couloir d'une case) → infranchissables,
  les zombies font 20 px de large pour des cases de 16
- **Diagonales** : interdites si elles coupent un coin (les deux cases orthogonales doivent être libres)
- **Coûts** : 10 orthogonal, 14 diagonal, + pénalité près des murs (`wall_penalty` : +30 à 1 case,
  +10 à 2 cases) : les chemins passent au large quand il y a de la place (les sprites, 32×32, sont plus
  grands que les colliders, 20×20 aux pieds) et serrent les murs seulement dans les ouvertures
- **Mise à jour** : quand le joueur change de case **ou** quand les cases bloquées changent
  (porte ouverte, fenêtre cassée/réparée)
- **Suivi** : un ennemi vise le `steering_point` de la case suivante — son centre écarté d'au moins
  une demi-case de chaque mur voisin (plus si son collider dépasse, `AgentBody`, offset compris), et
  d'une demi-case en diagonale d'un coin : dans une ouverture de 2 cases il vise le milieu du passage,
  et il se centre devant une porte/fenêtre avant de s'y engager

Outils de diagnostic (`crates/scenario/tests/scenarios.rs`, tests ignorés) :
`nav_map` (grille ASCII avec directions, `ALACOD_NAV=idle:700 ALACOD_NAV_ARROWS=1`),
`nav_stats` (par zombie : apparition, contact, blocages, sprite qui entre dans un mur et où),
`nav_probe` (un zombie à une frame).
Aussi `weapon_probe` (état des armes frame par frame) et `map_probe` (positions et état des
joueurs, portes et fenêtres).

### GGRS Compliance
- `FlowFieldCache` est `Clone` et enregistré avec `rollback_resource_with_clone`
- Player target sélectionné par tri `net_id.0` (pas `.iter().next()`)
- `GridPos` et `NavProfile` implémentent `Ord` pour `BTreeMap`

Note: `pathing.rs` fournit encore `update_enemy_targets` et `move_enemies` ; il utilise aussi les macros `order_iter!`/`order_mut_iter!`.

**Fenêtres** : une fenêtre intacte (`Obstacle::blocks_movement`) bloque les zombies (collision dans
`move_enemies`). Quand elle est sur leur chemin (case actuelle ou 3 suivantes) et à portée, elle
devient leur cible (`enemy_target_selection`) : ils la frappent avec le cooldown d'attaque
(`enemy_attack_system`) puis reprennent la poursuite une fois cassée. Une fenêtre cassée garde son
collider : elle bloque toujours les joueurs, plus les zombies ; la réparer la rend bloquante à nouveau.
Dans le flow field, une fenêtre intacte coûte `breakable_penalty` (le temps de la casser) : les
zombies prennent un passage ouvert s'il n'est pas beaucoup plus long.
