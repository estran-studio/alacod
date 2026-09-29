# Conventions de l'engine alacod

Référence pour ceux qui créent du contenu (cartes LDtk, sprites RON) et ceux qui ajoutent un vocabulaire à l'engine. Ce document s'applique dès que les crates de vocabulaire existent (M0 vague 1). Avant M0, les chemins et structures sont en flux ; vérifier la branche de la tâche courante (voir `docs/taches.md`).

**Utilité** : lire cette page quand on crée une carte, importe des sprites, ou ajoute un effect/behavior/statut. Chaque section renvoie aux fichiers à lire pour comprendre le pattern. Pour les violations de convention (Fixed quand entier, HashMap quand BTreeMap, f32 quand Fixed), l'outil `make check` ou `alacod lint` les rejette au chargement.

## 1. Cartes LDtk

**Version** : LDtk 1.5.3 (`jsonVersion` dans le fichier). Consulter `assets/exemples/test_map.ldtk`.

**Grille et taille** : tuiles de 16 pixels (`defaultGridSize: 16`). Un niveau LDtk = une salle de jeu. Les niveaux s'assemblent par leurs **connexions** (voir ci-dessous) ; le générateur (`crates/map/src/generation/`) en calcule position et taille dans le monde.

**Couches** : trois couches visibles dans l'éditeur.

- `Walls` (IntGrid, valeur 1) : les murs que les joueurs et ennemis ne traversent pas. Chaque cellule vaut 1 si c'est un mur, sinon absent (pas de 0 explicite). Détermine aussi le flow field (murs, portes fermées, obstacles cassables intacts) par ordre de priorité : murs > portes fermées > fenêtres intactes. Consulter `crates/map_ldtk/src/game/collider.rs` pour voir comment ces cellules sont converties en colliders physiques ; le flow field les utilise dans `crates/map/src/generation/imp/basic.rs` pour construire le graphe de navigation.
- `LevelConnection` (IntGrid, valeur 1) : indique les ouvertures (portes ou passages) sur chaque bord d'un niveau. Le générateur détecte ces connexions avec `scan_width_side` (`crates/map/src/generation/context.rs`) : un scan en ligne/colonne du bord compte les cellules contiguës valant 1. Chaque groupe continu = une ouverture possible vers un autre niveau. Exemple : une ouverture de trois cases sur le nord = trois cellules (1, 1, 1) consécutives dans la rangée du haut.
- `Entities` (couche objet Bevy LDtk) : emplacements des joueurs, ennemis, portes, fenêtres, armes et sodas. Chaque entité a une position (pixels dans le niveau) et une grille (cellule de 16 px du monde).

**Entités et leurs champs** (lus par `crates/map_ldtk/src/game/entity/*.rs`) :

- `DoorHorizontal`, `DoorVertical` (32×16 ou 16×32 px) : portes payantes. Champs obligatoires : `price` (entier, coût en points), `electrify` (booléen, desserte électrique si vrai). Champs optionnels : `paired_door_x`, `paired_door_y`, `paired_door_level` (chaîne) pour les portes appariées (une porte achetée ouvre son jumeau simultanément, utile pour les portes verrouillées par le générateur). Les portes **bloquent le flow field** tant que fermées (elles posent un collider ; le générateur évalue le coût de les casser) ; une porte intacte coûte du temps aux ennemis (ils la cassent si elle bloque le plus court chemin). Lus par `crates/map_ldtk/src/game/entity/door.rs`.
- `WindowHorizontal`, `WindowVertical` (16×32 ou 32×16 px) : vitres cassables. Pas de champ actuellement (champs réservés pour traitement futur : `health`, `repair_cost`). Les fenêtres intactes bloquent les ennemis (un collider permet de tirer à travers, pas de passer) ; cassées, elles bloquent les joueurs seulement (perméables aux ennemis). Lus par `crates/map_ldtk/src/game/entity/window.rs`.
- `PlayerSpawn` (16×16 px) : points de départ des joueurs. Champ obligatoire : `index` (0, 1, 2, 3) pour sélectionner le joueur parmi 4 (ordre de GgrsNetId). Le générateur les collecte par niveau ; un niveau doit avoir au moins un spawn. Lus par `crates/map_ldtk/src/game/entity/player_spawn.rs`.
- `ZombieSpawn` (16×16 px) : emplacements de spawn des ennemis. Aucun champ. La vague (mode `Waves` de F1) les utilise pour spawner des ennemis. Lus par `crates/map_ldtk/src/game/entity/enemy_spawn.rs`.
- `CrateLocation` (16×16 px), `WeaponLocation` (16×16 px), `SodaLocation` (16×16 px) : **non lues actuellement** (`crates/map_ldtk/src/map_const.rs` les déclare, aucun bundle implémenté en entity/*.rs). Réservées pour T2.3 (achats et économie, lecture des emplacements d'armes murales et de perks).

**Champ de niveau** : aucun actuellement ; réservé pour le champ `spawn` (identifier le niveau initial d'une run).

**Tailles de collision** : les colliders des entités (joueurs, ennemis) font 20×20 pixels aux pieds (voir `crates/game/src/character/config.rs`). Les sprites font 32×32 pixels. Les colliders de porte et fenêtre sont déterminés par leur taille LDtk (32×16 ou 16×32) ; l'engine les lit dans `crates/map_ldtk/src/game/entity/door.rs` et `window.rs`.

**Vérification** : `make map_preview` affiche une carte ; `make map_generation` la génère avec assemblage des salles par connexions. Un scénario `idle` sur la carte rejouée en vidéo (`make play_scenario SCENARIO=idle`) vérifie visually l'assemblage et les entités.

---

## 2. Sprites et animations

**Format de planche** : grille régulière en PNG, décrite par deux fichiers RON (exemple : `assets/ZombieShooter/Sprites/Zombie/`).

`SpriteSheetConfig` (`crates/animation/src/lib.rs`, une planche = une couche) :
- `path` : chemin du PNG relatif à `assets/`.
- `tile_size` : (largeur, hauteur) en pixels d'une case.
- `columns`, `rows` : grille de la planche.
- `anchor` : `Center`, `BottomLeft`, `BottomCenter`, etc. (enum `ConfigurableAnchor`). Pour des personnages, souvent `BottomCenter` (pieds au point (0,0) de la map).
- `offset_x`, `offset_y`, `offset_z` : translation du sprite par rapport à l'entité.
- `scale` : facteur appliqué au rendu.
- `animated` : `true` si la planche a plusieurs frames ; `false` sinon.
- `name` : nom interne du calque (exemple : `"body"`, `"shadow"`).

`AnimationMapConfig` (même dossier que la planche) :
- `frame_duration` : millisecondes par frame.
- `animations` : table `{ "nom": { start: 0, end: 6 } }` ou `{ "nom": { row: 0, end: 7 } }`. Format direct (indices absolus) ou basé sur les lignes (spécifier la ligne et le nombre de frames).
- `columns` : optionnel, dérivé de `SpriteSheetConfig` sinon.

**8 directions, 2 dessinées** : l'engine retourne gauche/droite. Inclure 8 animations orientées (nord, nord-est, est, etc.) dans la planche ; 8 directions suivies par `FacingDirection` (`crates/game/src/character/direction.rs`), 2 dessinées (gauche/droite obtenue par flip). Le retournement est automatique.

**Calques (skins)** : un personnage peut avoir plusieurs calques animés (`body`, `shadow`, `effect`). Chaque calque est une `SpriteSheetConfig` et une `AnimationMapConfig` ; ils sont composés dans `CharacterConfig` (`crates/game/src/character/config.rs`).

**CharacterConfig** (RON, une arme ou un personnage) :
```ron
(
    movement: (...),
    asset_name_ref: "zombie_full",  // Identifiant unique
    collider: (shape: Rectangle(width: "20.", height: "20."), offset: (...)),
    scale: "1.0",
    base_health: (max: "50.0"),
    starting_skin: "1",
    skins: {
        "1": (layers: {
            "body": "",  // Clé dans le registre d'animation
            "shadow": "",
        })
    }
)
```

**Armes** (exemple : `assets/weapons/melee/melee_weapons.ron`, map `{ "bare_hands": (...), ... }`) :
- `config` : `name`, `damage`, `range`, `attack_pattern` (enum : `SingleStrike`, `Combo(strikes: N)`, `Sweep(arc_angle)`, `Thrust`), `attack_duration_frames`, `cooldown_frames`, `knockback_force`, `stamina_cost`.
- `sprite_config` : `name`, `index` (première frame), `weapon_offset`.

**Fixed-point** : tous les nombres (`"80.0"`, `"10.0"`) s'écrivent en **chaîne de caractères**, jamais en entiers nus. Pourquoi ? Les Fixed sont des nombres en virgule fixe qui garantissent le déterminisme ; convertir une chaîne en Fixed au chargement élimine toute incertitude de float. Exemple : `damage: "10.0"`, non `damage: 10` ou `damage: 10.0`. Consulter `CLAUDE.md` §Déterminisme, règle 1 (Fixed-Point Math UNIQUEMENT).

**Armes** (exemple : `assets/weapons/melee/melee_weapons.ron`, voir plus haut en 2) : chaque arme a deux configs indépendantes. Nombre de balles, cadence, dégâts, knockback, tous en Fixed.

**Vagues** (`assets/waves/wave_config.ron`) : RON, table d'ID → définition. Chaque définition : `enemies` (liste d'IDs d'ennemis à spawn), `enemy_count` (nombre total), `spawn_rate_per_second` (fréquence), `difficulty_multiplier` (modificateur appliqué à la santé et dégâts). Les vagues tournent dans un générateur de flux RNG dédié (`stream("waves")`, T1.6) pour que l'aléatoire des variantes n'affecte pas d'autres vocabulaires.

---

## 3. Le dossier de jeu

**Aujourd'hui** (branche `m0-v2-conventions`) : assets à la racine du worktree (`assets/`), chemins codés en dur dans `crates/game/src/global_asset.rs` (chaine `"assets/weapons/melee"`, etc.). Point d'entrée unique `examples/map_explorer.rs` (cible Cargo `ldtk_map_explorer`). Tous les jeux partagent le même binaire, un seul `global_asset.rs` pour tous les chemins.

**Cible (T0.3 et T1.5)** : structure `games/<jeu>/` autonome.

```
games/<jeu>/
├── Cargo.toml         # Dépend de engine (crates/game, etc.)
├── src/
│   └── main.rs        # Boilerplate de l'engine + jeu
├── assets/
│   ├── sprites/       # Planches PNG + RON configs
│   ├── maps/          # LDtk (.ldtk)
│   ├── audio/         # Sons (.ogg)
│   ├── ui/            # Configs HUD/écrans (RON)
│   └── game.ron       # Manifeste : dossiers, ids de contenu
└── scenarios/         # Scénarios de test (.ron)
```

**`game.ron`** (manifeste du jeu, lu par `content` crate en T1.5) :
```ron
(
    name: "zombies",
    content_folders: [
        (path: "assets/characters", type: Character),
        (path: "assets/weapons", type: Weapon),
        (path: "assets/waves", type: Wave),
    ],
)
```

Chaque dossier est scanné ; les fichiers `.ron` créent des entrées typées dans un registre global : `CharacterId`, `WeaponId`, etc. Les références cassées sont rejetées au chargement (sans ambiguïté à l'exécution).

---

## 4. Ajouter un vocabulaire à l'engine : checklist

Exemple : ajouter un nouveau type d'ennemi, un effet, ou un statut. Suivre ce flux avant le code (plan §9.7). Chaque point est une décision qui touche l'architecture ; les valider avant d'écrire.

1. **Écrire le test (scénario)** : créer `tests/scenarios/<nom>.ron` avec attentes (`Expect`), invariants de frame, et commentaire « À regarder » décrivant ce qu'on doit observer à l'écran (quelles frames, quel comportement). Le scénario est le cahier des charges ; le code le réalise.
2. **Déclarer le kind** (enum Rust, non-négo) : exemple, `CombatEffect { firewall: bool, chilling: u32 }`. Le kind va dans la crate de vocabulaire (future `crates/combat` pour les effets). Enregistrer auprès de `KindRegistry` (trait de `sim_core`, T0.2) si le kind est sérialisable en RON.
3. **Composants et état** : déclarer tous les composants Bevy qui portent l'état (exemple, `StatusComponent { effect: CombatEffect, duration_frames: u32 }`). **Pas de HashMap, HashSet**, uniquement BTreeMap/BTreeSet pour l'ordre déterministe. Consulter `CLAUDE.md` Règles critiques du déterminisme.
4. **Enregistrement rollback** : `app.rollback_and_trace::<StatusComponent>()` pour chaque composant mutable. Les ressources mutables : `app.rollback_resource_with_clone::<StatusResourceCache>()`. Aucun appel direct à `rollback_component_*` (plan §9.6, K0).
5. **Flux RNG dédié** (plan §4.6) : si le vocabulaire utilise l'aléatoire (variantes, direction), créer un `stream` unique (ex. `RNG.stream("status")`). Consommer le RNG dans un ordre déterministe (après tri par `GgrsNetId`), jamais à la première occurrence.
6. **FrameEvents** : tous les événements émis (impact, mort, soin) passent par `FrameEvents<T>` (crates/game/src/frame_events.rs), jamais `MessageWriter`. Lire les événements en présentation (`Update`) après la simulation (`GgrsSchedule`), jamais dans la simulation.
7. **Scénario et trace** : `make test_scenarios SCENARIO=<nom>` sans `BLESS` la première fois. Si divergence, corriger le code. Une fois vert : `BLESS=1 make test_scenarios SCENARIO=<nom>` écrit la trace de référence. Cette trace est comparée à chaque commit.
8. **Vidéo et validation** : `make videos SCENARIO=<nom>` capture le scénario en 960×540. Regarder les images clés (PNG dans `target/videos/<commit>/scenarios/<nom>/`) et vérifier le « À regarder ». Les moments clés (spawn, first hit, death) sont détectés et cliquables.
9. **Doc** : ajouter une ligne ou un paragraphe à `docs/conventions.md` décrivant le kind, ses champs, les attentes, le invariants, et les fichiers lus. Exemple : « `StatusEffect::Fire` : durée en frames, dégâts par tick, propagation par toucher. Test : fire_spreading.ron. »

---

## 5. Commandes make

| Cible | Rôle |
|---|---|
| `make check` | Format, tests, lint (CI rapide) |
| `make test_scenarios` | Rejoue tous les scénarios en headless, compare les traces |
| `make map_preview` | Affiche une carte LDtk (GUI, requiert `--` et le chemin) |
| `make map_generation` | Assemble une carte par connexions (affiche le résultat) |
| `make play_scenario SCENARIO=<nom>` | Rejouele scénario avec rendu ; `--follow <handle>` suit un joueur |
| `make record_session NAME=<nom>` | Enregistre une partie en scénario |
| `make remote [HEADLESS=1]` | Lance une partie en pause, pilotée par `scripts/alacod-remote` |
| `make videos [SCENARIO=<nom>]` | Encode les scénarios en vidéos (960×540, `target/videos/<commit>/`) |
| `make review_videos [TAILSCALE=1]` | Serveur de revue des vidéos (`localhost:8766`) |
| `make diff_log CID_1=alice CID_2=bob` | Compare les logs d'état entre deux clients (déterminisme) |

Voir `Makefile` pour les détails (cibles `test_multiplayer`, `bench`, etc.).

---

## Notes essentielles

**À vérifier** : les entités `CrateLocation`, `WeaponLocation`, `SodaLocation` ne sont pas lues actuellement. Elles apparaissent dans `crates/map_ldtk/src/map_const.rs` (constantes) mais aucun bundle Bevy ne les traite (`entity/*.rs` ne les liste pas). T2.3 (Monnaie et achats) les implémentera ; avant d'utiliser une carte avec ces entités, vérifier que `make test_scenarios` accepte un scénario `idle` dessus.

**Colliders vs sprites** (CLAUDE.md) : collider des entités = 20×20 pixels aux pieds (collider rectangle) ; sprite rendu = 32×32 pixels (ils débordent visuellement au-dessus du collider). Cellules du flow field = 16 pixels. Les deux décalages (sprite 32 vs collider 20, flow field 16) sont compensés par `offset_x`, `offset_y` dans `SpriteSheetConfig` et `offset` dans `CharacterConfig`.

**IntGrid `Walls`** : les valeurs ne sont pas nommées (`identifier: None` dans le JSON LDtk). Ajouter une nouvelle surface (eau, lave, boue) : modifier `crates/map_ldtk/src/map_const.rs` (ajouter une constante, ex. `pub const LAYER_WATER: &str = "Water"` et créer l'IntGrid dans LDtk), puis le lire dans `crates/map_ldtk/src/game/plugin.rs` (décommenter le bundle correspondant) et `crates/map_ldtk/src/game/collider.rs` (ajouter le traitement du collider). Consulter `Walls` pour le pattern.

**Fixed-point systématiquement** : aucun `f32`, `f64`, `i32` dans un composant enregistré pour le rollback (`GgrsSchedule`). Les chaînes RON (`"100.0"`) sont parsées en `Fixed` au chargement par l'expression evaluator (T1.4). Les valeurs runtime restent `Fixed` (FixedVec2, etc.). Voir CLAUDE.md Contraintes Techniques.

**Tests et scénarios** : `make test_scenarios` vérifie que les scénarios passent en synctest (mode déterministe local). Tout changement de code dans `GgrsSchedule` peut casser les traces `.trace` ; rebase sur `main` chaque jour et revalidate en CI rapide. Les seules voies autorisées à changer les traces : V1 (simulation), justifié par `BLESS=1` en commit.

**LDtk et génération** : le fichier `test_map.ldtk` a trois niveaux (Level_0, Level_1, Level_2) pour tester l'assemblage. Le générateur (`crates/map/src/generation/imp/basic.rs`) crée un graphe de niveaux possibles et un arbre de profondeur N pour chaque run. Chaque niveau doit avoir au moins un `PlayerSpawn` (index 0 si un seul) et au moins un `ZombieSpawn` (même si c'est hors combat en sandbox).

