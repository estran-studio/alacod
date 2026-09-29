# Conventions de l'engine alacod

Référence pour ceux qui créent du contenu (cartes LDtk, sprites RON) et ceux qui ajoutent un vocabulaire à l'engine.

**Utilité** : lire cette page quand on crée une carte, importe des sprites, ou ajoute un effet/comportement/statut. Chaque section renvoie aux fichiers à lire pour comprendre le pattern.

## 1. Cartes LDtk

**Version** : LDtk 1.5.3 (`jsonVersion` dans le fichier). Consulter `assets/exemples/test_map.ldtk`.

**Grille et taille** : tuiles de 16 pixels (`defaultGridSize: 16`). Un niveau LDtk = une salle de jeu. Les niveaux s'assemblent par leurs **connexions** (voir ci-dessous) ; le générateur (`crates/map/src/generation/`) en calcule position et taille dans le monde.

**Couches** : trois couches visibles dans l'éditeur.

- `Walls` (IntGrid, valeur 1) : les murs que les joueurs et ennemis ne traversent pas. Chaque cellule vaut 1 si c'est un mur, sinon absent (pas de 0 explicite). Détermine aussi le flow field (murs, portes fermées, obstacles cassables intacts) par ordre de priorité : murs > portes fermées > fenêtres intactes. Consulter `crates/map_ldtk/src/game/collider.rs` pour voir comment ces cellules sont converties en colliders physiques ; le flow field les utilise dans `crates/game/src/character/enemy/ai/navigation.rs` pour construire le graphe de navigation.
- `LevelConnection` (IntGrid, valeur 1) : indique les ouvertures (portes ou passages) sur chaque bord d'un niveau. Le générateur détecte ces connexions avec `scan_width_side` (`crates/map/src/generation/context.rs`) : un scan en ligne/colonne du bord compte les cellules contiguës valant 1. Chaque groupe continu = une ouverture possible vers un autre niveau. Exemple : une ouverture de trois cases sur le nord = trois cellules (1, 1, 1) consécutives dans la rangée du haut.
- `Entities` (couche objet Bevy LDtk) : emplacements des joueurs, ennemis, portes, fenêtres, armes et sodas. Chaque entité a une position (pixels dans le niveau) et une grille (cellule de 16 px du monde).

**Entités et leurs champs** (lus par `crates/map_ldtk/src/game/entity/*.rs`) :

- `DoorHorizontal`, `DoorVertical` (32×16 ou 16×32 px) : portes payantes. En éditeur LDtk, deux champs seulement : `price` (entier, coût en points), `electrify` (booléen, desserte électrique si vrai). Le générateur ajoute trois champs aux portes qu'il place : `interactable` (booléen, défaut `true`), `paired_door_x`, `paired_door_y` (entiers), `paired_door_level` (chaîne) pour les portes appariées (une porte achetée ouvre son jumeau simultanément). Lus par `door_component_from_field` dans `crates/map_ldtk/src/game/entity/door.rs`. Les portes **bloquent le flow field** tant qu'elles ont un collider (portes fermées) ; une porte non interactive ne s'ouvre jamais (CLAUDE.md Flow Field, ligne ~539).
- `WindowHorizontal`, `WindowVertical` (16×32 ou 32×16 px) : vitres cassables. Aucun champ en éditeur. Santé fixée à 3 dans le code (`crates/map/src/game/entity/map/window.rs`). Les fenêtres intactes bloquent les ennemis (collision, mais permettent le tir à travers) ; cassées, elles bloquent les joueurs seulement (perméables aux ennemis). Réparation par interaction. Lus par `crates/map_ldtk/src/game/entity/window.rs`.
- `PlayerSpawn` (16×16 px) : points de départ des joueurs. Champ obligatoire : `index` (0, 1, 2, 3) = handle GGRS du joueur. Seul le niveau de départ use besoin d'avoir des spawns. Lus par `crates/map_ldtk/src/game/entity/player_spawn.rs`.
- `ZombieSpawn` (16×16 px) : emplacements de spawn des ennemis. Aucun champ. La vague (mode `Waves` de F1) les utilise pour spawner des ennemis. Lus par `crates/map_ldtk/src/game/entity/enemy_spawn.rs`.
- `CrateLocation` (16×16 px), `WeaponLocation` (16×16 px), `SodaLocation` (16×16 px) : **non lues actuellement** (`crates/map_ldtk/src/map_const.rs` les déclare, aucun bundle implémenté en entity/*.rs). Réservées pour T2.3 (achats et économie).

**Champ de niveau** : le champ `spawn` (booléen) détermine le niveau de départ d'une run. Lus par `crates/map_ldtk/src/generation/from.rs` ligne 150 : si présent et `true`, le niveau est marqué `LevelType::Spawn`.

**Tailles de collision** : les colliders des entités (joueurs, ennemis) font 20×20 pixels aux pieds (voir `crates/game/src/character/config.rs`). Les sprites font 32×32 pixels. Les colliders de porte et fenêtre sont déterminés par leur taille LDtk (32×16 ou 16×32) ; l'engine les lit dans `crates/map_ldtk/src/game/entity/door.rs` et `window.rs`.

**Vérification** : `make map_preview ARGS=assets/exemples/test_map.ldtk` affiche une carte ; `make map_generation ARGS=assets/exemples/test_map.ldtk` la génère avec assemblage des salles par connexions. Un scénario `idle` sur la carte rejouée en vidéo (`make play_scenario SCENARIO=idle`) vérifie visuellement l'assemblage et les entités.

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

**8 directions, 2 dessinées** : l'engine suit 8 directions par `FacingDirection` (`crates/animation/src/lib.rs`) mais dessine seulement 2 (gauche/droite, obtenues par retournement automatique). La planche n'a pas besoin de 8 orientations : une animation comprend la direction de visée, le retournement se fait au rendu.

**Calques (skins)** : un personnage peut avoir plusieurs calques animés (`body`, `shadow`, `effect`). Chaque calque est une `SpriteSheetConfig` et une `AnimationMapConfig` ; ils sont composés dans `CharacterConfig` (`crates/game/src/character/config.rs`).

**CharacterConfig** (RON, décrit un personnage) :
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

**Armes** (exemple : `assets/weapons/melee/melee_weapons.ron`, table `{ "bare_hands": (...), ... }`) :
- `config` : `name`, `damage`, `range`, `attack_pattern` (enum : `SingleStrike`, `Combo(strikes: N)`, `Sweep(arc_angle)`, `Thrust`), `attack_duration_frames`, `cooldown_frames`, `knockback_force`, `stamina_cost`. Tous les nombres décimaux en chaîne (`"100.0"`).
- `sprite_config` : `name`, `index` (première frame), `weapon_offset`.

**Fixed-point** : les valeurs de type `Fixed` s'écrivent en **chaîne** (`"100.0"`, jamais `100` ni `100.0` littéral). Les entiers (`frames`, `mag_size`, indices) s'écrivent nus. La sérialisation Bevy/RON (crate `fixed`) convertit les chaînes en `Fixed` au chargement, garantissant le déterminisme. Exemple : `damage: "10.0"` (Fixed), `attack_duration_frames: 10` (entier). Consulter `CLAUDE.md` §Déterminisme, règle 1 (Fixed-Point Math UNIQUEMENT).

**Vagues** (`assets/waves/wave_config.ron`) : RON, configuration simple. Champs : `base_enemies`, `enemies_per_wave`, `max_random_variance`, `min_wave_delay_frames`, `grace_period_frames`, `max_concurrent_enemies`, `spawn_batch_size`, `spawn_interval_frames`, `min_player_distance`/`max_player_distance` (Fixed en chaîne), `wave_tiers` (liste de `{max_wave, enemy_probabilities}`), `health_multiplier_per_wave`, `damage_multiplier_per_wave`. Un flux RNG dédié est en chantier (T1.6) pour isoler la variante.

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

**`game.ron`** (manifeste du jeu, exemple indicatif ; format fixé par T1.5) :
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

Chaque dossier est scanné ; les fichiers `.ron` créent des entrées typées dans un registre global : `CharacterId`, `WeaponId`, etc. Les références cassées sont rejetées au chargement (sans ambiguïté à l'exécution). Chemin au chargement : `assets/weapons/melee/melee_weapons.ron` (relatif à `assets/`).

---

## 4. Ajouter un vocabulaire à l'engine : checklist

Exemple : ajouter un nouveau type d'ennemi, un effet, ou un statut. Suivre ce flux avant le code (plan §9.7). Chaque point est une décision qui touche l'architecture ; les valider avant d'écrire.

1. **Écrire le test (scénario)** : créer `tests/scenarios/<nom>.ron` avec attentes (`Expect`), invariants de frame, et commentaire « À regarder » décrivant ce qu'on doit observer à l'écran (quelles frames, quel comportement). Le scénario est le cahier des charges ; le code le réalise.
2. **Déclarer le kind** : enum Rust sérialisable en RON dans la crate de vocabulaire (future `crates/combat` pour les effets). Enregistrer auprès de `KindRegistry` (trait de `sim_core`, T0.2).
3. **Composants et état** : déclarer tous les composants Bevy qui portent l'état. **Pas de HashMap, HashSet**, uniquement BTreeMap/BTreeSet pour l'ordre déterministe. Consulter `CLAUDE.md` Règles critiques du déterminisme.
4. **Enregistrement rollback** : `app.rollback_and_trace::<StatusComponent>()` pour chaque composant mutable. Les ressources mutables : `app.rollback_and_trace_resource::<StatusResourceCache>()`. Aucun appel direct à `rollback_component_*` (plan §9.6, K0).
5. **Flux RNG dédié** (plan §4.6) : si le vocabulaire utilise l'aléatoire (variantes, direction), créer un `stream` unique (ex. `RNG.stream("status")`). Consommer le RNG dans un ordre déterministe (après tri par `GgrsNetId`), jamais à la première occurrence.
6. **FrameEvents** : tous les événements émis (impact, mort, soin) passent par `FrameEvents<T>` (`crates/game/src/frame_events.rs`). Lus par les systèmes de la simulation (`GgrsSchedule`), ordonnés après l'émetteur. La présentation en dérive (`Update`, jamais d'événements), restant juste après un rollback.
7. **Scénario et trace** : `make test_scenarios SCENARIO=<nom>` sans `BLESS` la première fois. Si divergence, corriger le code. Une fois vert : `BLESS=1 make test_scenarios SCENARIO=<nom>` écrit la trace de référence. Cette trace est comparée à chaque commit.
8. **Vidéo et validation** : `make play_scenario SCENARIO=<nom> --capture /tmp/out --every 2` capture le scénario (PNG dans `/tmp/out/`, 960×540). Moments clés détectés (vague, kills, coups reçus, morts, rechargements, changements d'arme, fenêtres, portes) visibles et cliquables dans la page de revue.
9. **Doc** : ajouter une ligne ou un paragraphe à `docs/conventions.md` décrivant le kind, ses champs, les attentes, les invariants, et les fichiers lus.

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

Voir `Makefile` pour les détails (cibles `test_multiplayer`, etc.).

---

## Notes essentielles

**À vérifier** : les entités `CrateLocation`, `WeaponLocation`, `SodaLocation` ne sont pas lues actuellement. Elles apparaissent dans `crates/map_ldtk/src/map_const.rs` (constantes) mais aucun bundle Bevy ne les traite (`entity/*.rs` ne les liste pas). T2.3 (Monnaie et achats) les implémentera ; avant d'utiliser une carte avec ces entités, vérifier que `make test_scenarios` accepte un scénario `idle` dessus.

**Colliders vs sprites** (CLAUDE.md) : collider des entités = 20×20 pixels aux pieds (collider rectangle) ; sprite rendu = 32×32 pixels (ils débordent visuellement au-dessus du collider). Cellules du flow field = 16 pixels. Les deux décalages (sprite 32 vs collider 20, flow field 16) sont compensés par `offset_x`, `offset_y` dans `SpriteSheetConfig` et `offset` dans `CharacterConfig`.

**IntGrid `Walls`** : les valeurs ne sont pas nommées dans le JSON LDtk. Ajouter une nouvelle surface (eau, lave, boue) est le chantier E4 du plan ; modifier l'IntGrid et le collider selon le pattern de `Walls`.

**Fixed-point** : les valeurs `Fixed` s'écrivent en chaîne (`"100.0"`). Les entiers (`frames`, `GridPos`) ne sont pas interdits. Aucun `f32` ou `f64` dans les composants rollback. Voir `CLAUDE.md` Déterminisme, règle 1.

**Tests et scénarios** : `make test_scenarios` vérifie que les scénarios passent en synctest (mode déterministe local). Tout changement de code dans `GgrsSchedule` peut casser les traces `.trace` ; rebase sur `main` chaque jour et revalidate en CI rapide. Les seules voies autorisées à changer les traces : V1 (simulation), justifié par `BLESS=1` en commit.

