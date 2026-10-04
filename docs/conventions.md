# Conventions de l'engine alacod

Référence pour ceux qui créent du contenu (cartes LDtk, sprites RON) et ceux qui ajoutent un vocabulaire à l'engine.

**Utilité** : lire cette page quand on crée une carte, importe des sprites, ou ajoute un effet/comportement/statut. Chaque section renvoie aux fichiers à lire pour comprendre le pattern.

## 1. Cartes LDtk

**Version** : LDtk 1.5.3 (`jsonVersion` dans le fichier). Consulter `games/zombies/assets/exemples/test_map.ldtk`.

**Grille et taille** : tuiles de 16 pixels (`defaultGridSize: 16`). Un niveau LDtk = une salle de jeu. Les niveaux s'assemblent par leurs **connexions** (voir ci-dessous) ; le générateur (`crates/map/src/generation/`) en calcule position et taille dans le monde.

**Couches** : trois couches visibles dans l'éditeur.

- `Walls` (IntGrid, valeur 1) : les murs que les joueurs et ennemis ne traversent pas. Chaque cellule vaut 1 si c'est un mur, sinon absent (pas de 0 explicite). Détermine aussi le flow field (murs, portes fermées, obstacles cassables intacts) par ordre de priorité : murs > portes fermées > fenêtres intactes. Consulter `crates/map_ldtk/src/game/collider.rs` pour voir comment ces cellules sont converties en colliders physiques ; le flow field les utilise dans `crates/game/src/character/enemy/ai/navigation.rs` pour construire le graphe de navigation.
- `LevelConnection` (IntGrid, valeur 1) : indique les ouvertures (portes ou passages) sur chaque bord d'un niveau. Le générateur détecte ces connexions avec `scan_width_side` (`crates/map/src/generation/context.rs`) : un scan en ligne/colonne du bord compte les cellules contiguës valant 1. Chaque groupe continu = une ouverture possible vers un autre niveau. Exemple : une ouverture de trois cases sur le nord = trois cellules (1, 1, 1) consécutives dans la rangée du haut.
- `Entities` (couche objet Bevy LDtk) : emplacements des joueurs, ennemis, portes, fenêtres, armes et sodas. Chaque entité a une position (pixels dans le niveau) et une grille (cellule de 16 px du monde).

**Entités et leurs champs** (lus par `crates/map_ldtk/src/game/entity/*.rs`) :

- `DoorHorizontal`, `DoorVertical` (32×16 ou 16×32 px) : portes payantes. En éditeur LDtk, deux champs seulement : `price` (entier, coût en points), `electrify` (booléen, desserte électrique si vrai). Le générateur ajoute trois champs aux portes qu'il place : `interactable` (booléen, défaut `true`), `paired_door_x`, `paired_door_y` (entiers), `paired_door_level` (chaîne) pour les portes appariées (une porte achetée ouvre son jumeau simultanément). Lus par `door_component_from_field` dans `crates/map_ldtk/src/game/entity/door.rs`. Les portes **bloquent le flow field** tant qu'elles ont un collider (portes fermées) ; une porte non interactive ne s'ouvre jamais (CLAUDE.md Flow Field, ligne ~539).
- `WindowHorizontal`, `WindowVertical` (16×32 ou 32×16 px) : vitres cassables. Aucun champ en éditeur. Santé fixée à 3 dans le code (`crates/map/src/game/entity/map/window.rs`). Les fenêtres intactes bloquent les ennemis (collision, mais permettent le tir à travers) ; cassées, elles bloquent les joueurs seulement (perméables aux ennemis). Réparation par interaction. Lus par `crates/map_ldtk/src/game/entity/window.rs`.
- `PlayerSpawn` (16×16 px) : points de départ des joueurs. Champ obligatoire : `index` (0, 1, 2, 3) = handle GGRS du joueur. Seul le niveau de départ a besoin de points de départ. Lus par `crates/map_ldtk/src/game/entity/player_spawn.rs`.
- `ZombieSpawn` (16×16 px) : emplacements de spawn des ennemis. Aucun champ. Le mode vagues (`WaveModeEnabled` aujourd'hui, mode `Waves` du run en cible, F1) y fait apparaître les ennemis. Lus par `crates/map_ldtk/src/game/entity/enemy_spawn.rs`.
- `CharacterSpawn` (16×16 px, T2.9, testbed) : fait apparaître, une fois au chargement de la map (pas par les vagues), un personnage précis du registre. Deux champs : `character` (chaîne, `CharacterId`, obligatoire) et `team` (chaîne optionnelle : `players`/`enemies`/`allies`/`neutral` ; absente ou vide → `CharacterConfig.team` du personnage, sinon `Enemies`). Lus par `crates/map_ldtk/src/game/entity/character_spawn.rs` ; spawné par `map_ldtk::game::local::spawn_characters_when_map_loaded` via `character::enemy::create::spawn_enemy`. Utilisé par `games/testbed/assets/testbed/*.ldtk` pour placer `dummy`/`target`/`follower`/`ally`/`civilian`/`breacher` (`games/testbed/assets/characters/*.ron`).
- `WeaponLocation` (16×16 px, T2.3, chantier C5 v1) : arme murale payante. Deux champs en éditeur : `weapon` (chaîne, `WeaponId` de `weapons.ron`), `price` (entier, coût en points). Propagée par le générateur de salles (modèle `CharacterSpawn`, voir `crates/map/src/generation/entity/weapon_location.rs`) ; au chargement de la map, fait apparaître un `weapons::WeaponPickup` mural (`price: Some(...)`, jamais consommé au ramassage — voir `map_ldtk::game::local::spawn_weapon_locations_when_map_loaded`). Lue par `crates/map_ldtk/src/game/entity/weapon_location.rs`. `games/zombies/assets/exemples/test_map.ldtk` en pose quatre depuis T2.6 (pistol 500 dans le gabarit de départ `Level_0`, rifle 1200 et shotgun 1000 dans `Level_1`, machine_gun 1500 dans `Level_2` ; tableau dans `games/zombies/README.md`, qui rappelle aussi que la partie assemble des copies de ces gabarits — deux salles `Level_0` avec la seed des scénarios, donc deux murs `pistol`) ; `test_map_shop.ldtk` en pose une (shotgun, 500) pour les scénarios d'achat.
- `SodaLocation` (16×16 px, T2.3, chantier C5 v1) : machine à perk. Un champ en éditeur : `perk` (chaîne, `PerkId` de `economy/perks.ron`). Même propagation que `WeaponLocation` ; au chargement, fait apparaître une entité `game::economy::PerkMachine` avec `Interactable { interaction_type: Perk }` (voir `map_ldtk::game::local::spawn_soda_locations_when_map_loaded`). Lue par `crates/map_ldtk/src/game/entity/soda_location.rs`.
- `CrateLocation` (16×16 px) : **non lue actuellement** (`crates/map_ldtk/src/map_const.rs` la déclare, aucun bundle implémenté en entity/*.rs). Réservée à un chantier futur (butin, C4).

**Champ de niveau** : le champ `spawn` (booléen) détermine le niveau de départ d'une run. Lus par `crates/map_ldtk/src/generation/from.rs` ligne 150 : si présent et `true`, le niveau est marqué `LevelType::Spawn`.

**Tailles de collision** : les colliders des entités (joueurs, ennemis) font 20×20 pixels aux pieds (voir `crates/game/src/character/config.rs`). Les sprites font 32×32 pixels. Les colliders de porte et fenêtre sont déterminés par leur taille LDtk (32×16 ou 16×32) ; l'engine les lit dans `crates/map_ldtk/src/game/entity/door.rs` et `window.rs`.

**Vérification** : `make map_preview ARGS=games/zombies/assets/exemples/test_map.ldtk` affiche une carte ; `make map_generation ARGS=games/zombies/assets/exemples/test_map.ldtk` la génère avec assemblage des salles par connexions. Un scénario `idle` sur la carte rejouée en vidéo (`make play_scenario SCENARIO=idle`) vérifie visuellement l'assemblage et les entités.

---

## 2. Sprites et animations

**Format de planche** : grille régulière en PNG, décrite par deux fichiers RON (exemple : `games/zombies/assets/sprites/enemies/zombie/`).

**Table des feuilles (D3, kind `SpriteSheet`)** : `sprites/sprites.ron`, déclaré dans `game.ron`
(`(path: "sprites/sprites.ron", kind: "SpriteSheet")`), associe un **id** à une configuration
d'animation et à ses calques. L'id est celui que le contenu cite : `asset_name_ref` d'un
personnage, `sprite_config.name` d'une arme à distance, `slash` pour l'effet de coup de mêlée.
`game::global_asset` charge exactement cette table (aucun chemin de sprite dans le code).
```ron
({
    "player": (
        animation: "sprites/characters/player/player_animation.ron",
        layers: {
            "body": "sprites/characters/player/player_sheet.ron",
            "shadow": "sprites/characters/shadow/shadow_sheet.ron",
        },
    ),
    "pistol": (
        animation: "sprites/characters/player/player_animation.ron",
        layers: { "body": "sprites/weapons/pistol_sheet.ron" },
    ),
})
```
Feuilles, animations et images sont rangées par entité sous `sprites/` : `characters/` (joueur,
ombre), `weapons/`, `enemies/`, `effects/`. Un personnage sans entrée n'a pas de sprite (les
personnages de laboratoire du testbed). Lint : fichiers d'une entrée présents (animation,
feuilles, image `path` de chaque feuille), `sprite_config.name` d'une arme connu de la table, id
répété refusé (tableau du §3). Les configs de contenu (`player_config.ron`, `weapons.ron`,
`zombie_*_config.ron`) restent sous `ZombieShooter/Sprites/**` (dossier historique) : D3 n'a
déplacé que les sprites.

`SpriteSheetConfig` (`crates/animation/src/lib.rs`, une planche = une couche) :
- `path` : chemin du PNG relatif au dossier `assets/` du jeu (`games/<jeu>/assets/`).
- `tile_size` : (largeur, hauteur) en pixels d'une case.
- `columns`, `rows` : grille de la planche.
- `anchor` : `Center`, `BottomLeft`, `BottomCenter`, etc. (enum `ConfigurableAnchor`). Les planches actuelles utilisent `Center` avec des `offset_*` (voir `sprites/characters/player/player_sheet.ron`).
- `offset_x`, `offset_y`, `offset_z` : translation du sprite par rapport à l'entité.
- `scale` : facteur appliqué au rendu.
- `animated` : `true` si la planche a plusieurs frames ; `false` sinon.
- `name` : nom interne du calque (exemple : `"body"`, `"shadow"`).

`AnimationMapConfig` (même dossier que la planche) :
- `frame_duration` : millisecondes par frame.
- `animations` : table `{ "nom": { start: 0, end: 6 } }` ou `{ "nom": { row: 0, end: 7 } }`. Format direct (indices absolus) ou basé sur les lignes (spécifier la ligne et le nombre de frames).
- `columns` : optionnel, dérivé de `SpriteSheetConfig` sinon.

**8 directions, 2 dessinées** : l'engine suit 8 directions par `FacingDirection` (`crates/animation/src/lib.rs`) mais dessine seulement 2 (gauche/droite, obtenues par retournement automatique). La planche n'a pas besoin de 8 orientations : elle est dessinée tournée vers la droite, et le rendu la retourne pour la gauche (`set_sprite_flip`).

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

**Armes** (exemple : `games/zombies/assets/weapons/melee/melee_weapons.ron`, table `{ "bare_hands": (...), ... }`) :
- `config` : `name`, `damage`, `range`, `attack_pattern` (enum : `SingleStrike`, `Combo(strikes_in_combo: N)`, `Sweep(arc_angle)`, `Thrust`), `attack_duration_frames`, `cooldown_frames`, `knockback_force`, `stamina_cost`. Tous les nombres décimaux en chaîne (`"100.0"`).
- `sprite_config` : `name`, `index` (première frame), `weapon_offset`. Pour une arme à distance, `name` est un id de la table `SpriteSheet` (D3).

**Fixed-point** : les valeurs de type `Fixed` s'écrivent en **chaîne** (`"100.0"`, jamais `100` ni `100.0` littéral). Les entiers (`frames`, `mag_size`, indices) s'écrivent nus. La sérialisation Bevy/RON (crate `fixed`) convertit les chaînes en `Fixed` au chargement, garantissant le déterminisme. Exemple : `damage: "10.0"` (Fixed), `attack_duration_frames: 10` (entier). Consulter `CLAUDE.md` §Déterminisme, règle 1 (Fixed-Point Math UNIQUEMENT).

**Vagues** (`games/zombies/assets/waves/wave_config.ron`) : RON, configuration simple. Champs : `base_enemies`, `enemies_per_wave`, `max_random_variance`, `min_wave_delay_frames`, `grace_period_frames`, `max_concurrent_enemies`, `spawn_batch_size`, `spawn_interval_frames`, `min_player_distance`/`max_player_distance` (Fixed en chaîne), `wave_tiers` (liste de `{max_wave, enemy_probabilities}`), `health_multiplier_per_wave`, `damage_multiplier_per_wave`. Un flux RNG dédié est en chantier (T1.6) pour isoler la variante.

---

## 3. Le dossier de jeu

**Depuis T0.3** : structure `games/<jeu>/` autonome. Chaque jeu a son propre crate binaire (`games/<jeu>/Cargo.toml`, `src/main.rs`), ses assets (`games/<jeu>/assets/`), et ses scénarios de test. Le premier jeu implémenté est `zombies` (T0.3) ; un testbed minimal (`testbed`) valide l'engine sans simulation complexe.

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

**`game.ron`** (manifeste du jeu, `crates/content/src/manifest.rs`, T1.5) : nom, dossiers de
contenu typés, point d'entrée (carte de départ, graine par défaut). Exemple
(`games/testbed/assets/game.ron`) :
```ron
(
    name: "testbed",
    content_folders: [
        (path: "ZombieShooter/Sprites/Character/player_config.ron", kind: "Character"),
        (path: "ZombieShooter/Sprites/Character/weapons.ron", kind: "Weapon"),
        (path: "weapons/melee/melee_weapons.ron", kind: "MeleeWeapon"),
        (path: "testbed", kind: "Map"),
        (path: "ui", kind: "Ui"),
        (path: "camera.ron", kind: "Camera"),
    ],
    entry: (
        start_map: "testbed/arena.ldtk",
        default_seed: 123456,
    ),
)
```
Chaque entrée de `content_folders` est un **fichier** ou un **dossier**, relatif à
`assets/`. Un dossier est scanné (non récursif) pour l'extension attendue par son `kind`
(`.ron`, sauf `Map` qui attend `.ldtk`) ; un fichier est lu tel quel. Les jeux actuels
mélangent encore plusieurs kinds dans un même dossier historique
(`ZombieShooter/Sprites/Character/` contient à la fois `player_config.ron` et des feuilles
de sprite) : dans ce cas le manifeste déclare le **fichier** précis plutôt que le dossier
entier (voir `games/zombies/assets/game.ron`). Les kinds connus : `Character`, `Weapon`,
`MeleeWeapon`, `Wave`, `Map`, `Ui`, `Camera`, `Economy`, `Perk`, `PowerUp`, `SpriteSheet` (D3)
(`content::registry::KNOWN_KIND_NAMES`) ; un
autre kind produit une erreur de lint (« kind inconnu ») plutôt qu'un échec RON générique.

**Le registre** (`content::registry::Registry`, chargé par `Registry::build`) est la
source de vérité des ids de contenu : `CharacterId`, `WeaponId`, `MeleeWeaponId`,
`EnemyId`, `WaveConfigId`, `MapId` (newtypes sur chaîne, `Ord`, `BTreeMap`). L'id d'un
personnage vient de son champ `asset_name_ref` (pas du nom de fichier, qui ne le reflète
pas toujours aujourd'hui) ; l'id d'une arme ou d'une arme de corps à corps vient de la clé
dans `weapons.ron`/`melee_weapons.ron` ; l'id d'une carte ou d'une config de vagues vient du
nom de fichier sans extension. Les joueurs reçoivent les armes déclarées dans le champ
`starting_weapons: [WeaponId]` de leur fichier `characters/*.ron` (dans l'ordre déclaré : la
première est l'arme active), pas tout `weapons.ron`.

**Le lint** (`content::lint::run`, appelé par `alacod lint`, par le jeu au démarrage et par
le rechargement à chaud) refuse une référence cassée (`starting_weapons` vers une arme
inconnue, `enemy_probabilities` d'une vague vers un personnage inconnu, `entry.start_map`
vers une carte non chargée), un id dupliqué, une valeur hors plage (santé > 0, vitesse
mouvement >= 0, cadence de tir > 0), un kind de dossier inconnu, ou un littéral RON nu
(entier ou flottant) là où une valeur `Fixed` est attendue (le projet exige une chaîne,
`"1.5"` : voir §2 ci-dessus et CLAUDE.md, règle 1). Chaque erreur nomme le fichier (relatif
à `assets/`) et un message précis (id, champ, valeur).

**Règles de lint** (`crates/content/src/lint.rs` pour les règles sémantiques,
`crates/content/src/registry.rs` pour les erreurs de chargement). Une fixture par règle sous
`crates/content/tests/fixtures/<fixture>/` (mini-jeu : `assets/game.ron` + contenu minimal,
un seul problème en plus de `start_map: "unused"` commun à toutes), un test par fixture dans
`crates/content/tests/lint_fixtures.rs`. Kinds d'erreur (`LintErrorKind`) : `Parse`,
`DuplicateId`, `BrokenReference`, `OutOfRange`, `UnknownKind`.

| Contenu | Règle | Kind | Fixture |
|---|---|---|---|
| tout | RON invalide, littéral nu dans un champ `Fixed` | `Parse` | `float_literal` |
| `game.ron` | kind de dossier inconnu | `UnknownKind` | `unknown_kind` |
| `game.ron` | `entry.start_map` vers une carte non chargée | `BrokenReference` | (toutes) |
| `game.ron` | `entry.mode: Waves` sans dossier `Wave` (T2.4) | `BrokenReference` | `entry_mode_waves_without_waves` |
| personnage | id (`asset_name_ref`) dupliqué | `DuplicateId` | `duplicate_id` |
| personnage | `starting_weapons` vers une arme inconnue | `BrokenReference` | `broken_reference` |
| personnage | `starting_weapons.len() <= weapon_slots` (T3.4) | `OutOfRange` | `starting_weapons_exceed_slots` |
| personnage | `starting_skin` absent de `skins` | `BrokenReference` | — |
| personnage | `base_health.max > 0`, `movement.max_speed >= 0` | `OutOfRange` | `out_of_range` |
| personnage | `stats.<StatId> >= 0` (T1.2) | `OutOfRange` | `out_of_range_stat` |
| personnage | `bleedout_frames`, `revive_frames`, `weapon_slots` > 0 ; `downed_speed_mult` dans ]0, 1] | `OutOfRange` | — |
| arme | `firing_modes.<mode>.firing_rate > 0` ; gabarit `test:` (`frames > 0`, `min_hits <= max_hits`) | `OutOfRange` | — |
| arme | `ammo_type` : variante inconnue (`Laser`) | `Parse` (nomme le champ) | `ammo_type_unknown` |
| arme | `ammo_type: Custom("")` (nom vide) (T2.8) | `OutOfRange` | `ammo_type_empty_custom` |
| arme, corps à corps | `friendly_fire` : valeur inconnue (enum fermé `Never`/`Always`/`Cursed`) (T2.8) | `Parse` (nomme le champ) | `friendly_fire_unknown` |
| arme | `audio_config.modes.<mode>.reloading`/`firing` : fichier absent de `assets/` (T2.8) | `BrokenReference` | `audio_missing_file` |
| arme | clé répétée dans `weapons.ron` (T3.4) | `DuplicateId` | `weapons_duplicate_key` |
| corps à corps | clé répétée dans `melee_weapons.ron` (T3.4) | `DuplicateId` | `melee_duplicate_key` |
| corps à corps | `damage >= 0` | `OutOfRange` | — |
| vagues | `enemy_probabilities` vers un personnage inconnu | `BrokenReference` | — |
| économie | `refill_price_ratio` dans [0, 1] (T2.8) | `OutOfRange` | `economy_ratio_out_of_range` |
| perk | `price > 0` | `OutOfRange` | — |
| perk | `modifiers` non vide (T2.8) | `OutOfRange` | `perk_no_modifiers` |
| perk | `op: Mul` exige `value > 0` (T2.8) | `OutOfRange` | `perk_mul_non_positive` |
| perk | `stat` inconnue (`StatId` : hors liste seulement via `Custom("...")`) | `Parse` (nomme le champ) | `perk_unknown_stat` |
| perk | id écrit deux fois dans `perks.ron` (T2.8) | `DuplicateId` | `perk_duplicate_id` |
| power-up | id dupliqué entre deux fichiers du dossier | `DuplicateId` | `powerup_duplicate_id` |
| power-up | clé répétée dans `items/powerups.ron` (T3.4) | `DuplicateId` | `powerups_duplicate_key` |
| power-up | `actions[].stat` inconnue | `Parse` | `powerup_unknown_stat` |
| power-up | `drop_chance` dans [0, 1] | `OutOfRange` | `powerup_drop_chance_out_of_range` |
| power-up | `weight > 0` | `OutOfRange` | `powerup_weight_zero` |
| power-up | `actions` non vide (T2.8) | `OutOfRange` | `powerup_no_actions` |
| power-up | `lifetime_frames > 0` (T2.8) | `OutOfRange` | `powerup_lifetime_zero` |
| power-up | `pickup_range > 0` (T2.8) | `OutOfRange` | `powerup_pickup_range_non_positive` |
| power-up | `TimedModifier`/`CurrencyMultiplier` : `frames > 0` (T2.8) | `OutOfRange` | `powerup_frames_zero` |
| power-up | `CurrencyMultiplier` : `factor > 0` (T2.8) | `OutOfRange` | `powerup_factor_non_positive` |
| power-up | `TimedModifier` avec `op: Mul` : `value > 0` (T3.4) | `OutOfRange` | `powerup_timed_mul_non_positive` |
| `ui/feedback.ron` | `hit_flash.frames`, `shake.frames` > 0 (T3.4) | `OutOfRange` | `feedback_frames_zero` |
| `ui/feedback.ron` | `shake.amplitude >= 0` (T3.4, zéro désactive la secousse) | `OutOfRange` | `feedback_amplitude_negative` |
| `ui/feedback.ron` | chaque fichier de `sounds` présent sous `assets/` (T3.4) | `BrokenReference` | `feedback_missing_sound` |
| feuilles de sprites | `animation` et feuille de chaque calque présentes sous `assets/` (D3) | `BrokenReference` | `sprite_sheet_missing_file` |
| feuilles de sprites | image (`path`) de chaque feuille présente sous `assets/` (D3) | `BrokenReference` | `sprite_sheet_missing_image` |
| feuilles de sprites | id répété dans `sprites/sprites.ron` (D3) | `DuplicateId` | `sprite_sheet_duplicate_key` |
| arme | `sprite_config.name` absent de la table `SpriteSheet`, si le jeu en déclare une (D3) | `BrokenReference` | `weapon_sprite_unknown` |

Les enums fermés (`FriendlyFire`, `ModifierOp`, `Action`) et les variantes nues inconnues des
enums ouverts (`AmmoType`, `StatId`, qui n'acceptent un nom libre que sous `Custom("...")`)
échouent au chargement RON : la règle est le parse lui-même, la fixture documente le message
(`champ ammo_type : ...`, `champ friendly_fire : ...`, `champ stat : ...`). Une table RON
(`{ "id": ... }`) désérialisée en `BTreeMap` garde silencieusement la dernière valeur d'une
clé répétée. Le registre lit donc `perks.ron`, `weapons.ron`, `melee_weapons.ron` et la table
`powerups` de `items/powerups.ron` en liste (`registry::KeyedEntries`) pour rapporter les
doublons, y compris dans un même fichier. La première entrée est conservée, la seconde
produit `DuplicateId`. Les fichiers nommés `feedback.ron` déclarés sous le kind `Ui` sont
lus avec un schéma typé ; les autres fichiers Ui restent validés syntaxiquement.

**CLI** : `cargo run -p content --bin alacod --profile headless -- lint games/<jeu>` (code
de sortie 1 et messages sur stderr en cas d'erreur, 0 sinon) ; `make lint` l'appelle pour
`zombies` et `testbed`. Un hook `PostToolUse` (`.claude/settings.json`,
`scripts/lint-edited-game.sh`) relance ce lint en arrière-plan quand un fichier sous
`games/**` est édité.

**Rechargement à chaud** : hors partie (`AppState::LobbyLocal`/`LobbyOnline`), si un
fichier de contenu suivi par bevy change (feature `native`, `bevy/file_watcher`), le
registre est relu depuis le disque et le lint relancé
(`crates/game/src/content_hot_reload.rs`, idiome `MessageReader<AssetEvent<T>>` repris de
`crates/game/src/ui/hud.rs`) ; un contenu invalide laisse l'ancien registre en place
(erreur journalisée). Jamais pendant une partie (`GgrsSchedule`) : les snapshots rollback ne
se réécrivent pas à chaud.

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
8. **Vidéo et validation** : `make videos SCENARIO=<nom>` produit la vidéo dans `target/videos/<commit>/` ; pour regarder des images fixes, `cargo run -p scenario --features render --bin play_scenario -- tests/scenarios/<nom>.ron --capture <dossier> --every 2` (PNG 960×540, une image toutes les 2 frames). Moments clés détectés (vague, kills, coups reçus, morts, rechargements, changements d'arme, fenêtres, portes, armes lâchées/ramassées, power-ups, victoire/défaite/abandon) visibles et cliquables dans la page de revue.
9. **Doc** : ajouter une ligne ou un paragraphe à `docs/conventions.md` décrivant le kind, ses champs, les attentes, les invariants, et les fichiers lus.

---

### 4.3 Contrats de M1 (squelettes)

T1.0a fixe le vocabulaire ci-dessous. **La vague 1 étend chaque enum dans son crate** ;
un renommage doit être signalé dans son rapport. Aucun système n'exécute ces contrats
pendant la vague 0, et aucune trace de référence ne change.

| Crate | Contrats | Plugin / futurs sets |
|---|---|---|
| `combat` | `ProjectileModifier` : Bounce, Pierce, Size, Lifetime, Homing, Gravity ; `Pattern` : Aimed, Spread, Ring, Sequence, Telegraph, Wait ; `StatusDef` : Burn, Slow, Stun, Freeze ; état `Statuses(Vec<StatusEntry>)` (statut, piles, frame d'expiration) | `CombatPlugin` ; Projectiles, Effects, Status |
| `behaviors` | `Behavior` : Chase, KeepDistance, Strafe, Charge, Shoot, Melee, Flee, Wander ; `Perception` : Sight, Hearing ; `Targeting::Nearest` ; état `BehaviorState` (indice de règle, frame d'entrée, cible GgrsNetId) | `BehaviorsPlugin` ; EnemyAI |
| `effects` | `Effect { on, if, do }`, `On` (11 déclencheurs), `Condition` (7 conditions), `GaugeThreshold::Above` ; réutilise `Action` des power-ups | `EffectsPlugin` ; Effects |

Les plugins enregistrent les noms Rust exacts (PascalCase) dans les catégories snake_case
`projectile_modifier`, `pattern`, `status`, `behavior`, `perception`, `targeting`,
`effect_trigger`, `effect_condition`. T1.12 fixera la validation des références dans le
contenu. `Kinds` et `KindRegistry` restent inchangés. `Statuses` et `BehaviorState` sont
enregistrés ensemble en rollback + checksum + trace, mais ne sont encore posés sur aucune
entité. Il n'y a pas de nouveau `RollbackSystemSet`.

Les quantités `Fixed` restent des chaînes (§2), les compteurs et durées des entiers en
frames, les angles des radians et les vitesses des unités/seconde. `Chase` porte un id de
profil (`"Ground"`, etc.) ; `Shoot`/`Melee` portent un id de pattern/arme. T1.4 les résoudra,
sans dépendance de `behaviors` vers `game` ou `combat`. `PerceptionConfig` compose une liste
de sens et `needs_light` (vue seulement). Les listes de séquence, conditions, actions et
statuts conservent l'ordre déclaré.

```ron
Sequence([Telegraph(60), Aimed(count: 4, spread: "0.1", projectile: "plomb"), Wait(180)])
Ring(count: 12, speed: "150", projectile: "braise", every: 90)
Chase(profile: "Ground")
Nearest(ignore: ["disguised", "ghost"])
(on: OnGauge("sacre", Above("0.75")), if: [HpBelow("0.2")], do: [RefillAmmo])
```

Le déplacement des armes met simulation, données des acteurs et géométrie de collision
dans `combat`. `game::weapons`, les anciens chemins des composants de personnage,
`game::collider` et `game::collision_grid` restent des réexports. Les attentes typées du
champ `test.expect` vivent dans `combat::weapons::expectations` et sont réexportées par
`game::replay` : métadonnées d'outillage, exclues du hash des armes, jamais exécutées par
`combat`. Les sprites attachés, slashs et HUD restent dans `game::ui` ; les handlers d'achat
et de ramassage et le chargement des armes murales restent dans `game`/`map_ldtk`.
`Interactable`/`InteractionType` sont des contrats `sim_core::interaction`, réexportés par
`game::interaction`, ce qui permet aussi à `combat` de créer une arme lâchée interactable.

---

## 5. Commandes make

| Cible | Rôle |
|---|---|
| `make check` | Format, tests unitaires, scénarios, motifs interdits en avertissement (la CI rapide) |
| `make test_scenarios` | Rejoue tous les scénarios en headless, compare les traces |
| `make map_preview ARGS=<fichier.ldtk>` | Affiche une carte LDtk (fenêtre) |
| `make map_generation ARGS=<entrée.ldtk> <sortie.ldtk> <graine>` | Génère une carte assemblée par connexions (voir `examples/map_generation.rs` pour les arguments exacts) |
| `make play_scenario SCENARIO=<nom>` | Rejoue le scénario avec rendu (l'option `--follow <handle>` du binaire suit un joueur) |
| `make record_session NAME=<nom>` | Enregistre une partie en scénario |
| `make remote [HEADLESS=1]` | Lance une partie en pause, pilotée par `scripts/alacod-remote` |
| `make videos [SCENARIO=<nom>]` | Encode les scénarios en vidéos (960×540, `target/videos/<commit>/`) ; plusieurs noms : `SCENARIO=a,b` ou `SCENARIO="a b"` ; `DRY_RUN=1` liste sans compiler (D23) |
| `make review_videos [TAILSCALE=1]` | Serveur de revue des vidéos (`localhost:8766`) |
| `make diff_log CID_1=alice CID_2=bob` | Compare les logs d'état entre deux clients (déterminisme) |

Voir `Makefile` pour les détails (cibles `test_multiplayer`, etc.).

**Parties de référence du clone `zombies` (T3.1)** : `clone_solo`, `clone_duo` et
`clone_quad` (`tests/scenarios/`, `exemples/test_map.ldtk`, graine 123456) suivent les
vagues 1 à 5 avec achats du pistolet et de Juggernog, Max Ammo et Nuke placés, et des
attentes datées. Le duo ajoute le tir ami bloqué et une réanimation ; le quad combine
deux bots `fonceur`, un `prudent` et un joueur scripté qui réanime un fonceur. Les
commentaires « À regarder » donnent les frames de revue, mesurées avec
`ALACOD_EVENTS=1 make test_scenarios SCENARIO=<nom>`. Les visées scriptées sont
enregistrées en headless, puis rejouées comme inputs fixes ; les bots du quad restent
pilotés par leur profil.

Les effectifs sont réduits par `wave_overrides`. Deux champs optionnels de ce réglage
complètent les effectifs et la préparation : `min_wave_delay_frames` raccourcit
l'attente entre vagues, `max_wave` fixe la condition de victoire. Absents, ils conservent
les valeurs du jeu. Solo et duo déclarent `max_wave: 5` : le mode `Waves` termine à
l'**entrée** en vague 5 et produit le résumé (la cinquième vague n'est pas combattue).
Le quad laisse cette limite absente et attend `RunState(Playing)` avec quatre joueurs
debout à l'entrée en vague 5. Aucune trace antérieure n'est blessée pour ces références.

---

## 6. CI lente (nuit) — T2.14

**Quoi** : chaque nuit (03:00 UTC) et à chaque push sur `main`, un runner auto-hébergé (`[self-hosted, alacod-builder]`) exécute un pipeline complet : benchmarks strict, simulation multi-bots, p2p headless (2 et 4 joueurs via allumette, détection des desyncs), génération des vidéos de tous les scénarios, page de revue interactive, et notes Markdown commitées dans `tests/review-notes/<commit>.md`.

**Où** : `.github/workflows/nightly.yaml` (workflow), `scripts/nightly.sh` (pipeline), `docker-compose.ci.yaml` (allumette headless), `Makefile` (`make nightly`, `make nightly_quick`).

**Comment lancer localement** (rapide, 2 graines, vague 2, vidéos idle+shoot_around) :
```bash
make nightly_quick
```

Résultats dans `target/nightly/<commit>/summary.md`, `sim.json`, `review/index.html`, `p2p-*.trace`, et `tests/review-notes/<commit>.md` (sans commit).

**Options** (full : 50 graines, vague 10, vidéos toutes, 2 et 4 joueurs p2p) :
```bash
NIGHTLY_SEEDS=1..100 NIGHTLY_BOTS=4 NIGHTLY_WAVE=15 NIGHTLY_P2P=2,4,8 NIGHTLY_VIDEOS=all bash scripts/nightly.sh
```

**Étapes du pipeline** :

1. **Bench** (seuils de `tests/budgets.ron`) : rejeu de tous les scénarios en headless, compare FPS simulés par scénario à un plancher de non-régression ; échoue si un scénario dépasse son seuil (`ALACOD_BENCH_STRICT=1 make test_scenarios`).

2. **Sim** (simulation bots) : centaines de parties sans rendu à N bots sur M graines jusqu'à la vague K, détecte les crashes et deadlocks ; JSON de métriques par graine (taux de survie, ennemis tués, vague atteinte, FPS moyenne). L'absence de crash = pas de desync en single-player.

3. **P2P headless** (2 et 4 joueurs) : lance une instance d'allumette (docker compose), puis N clients `zombies` en headless dans le même lobby sans rendu. Chaque client écrit sa trace d'état à chaque frame (`ALACOD_STATE_TRACE`, `ALACOD_EXIT_AT_FRAME=600`). À la fin, compare les traces : identiques = pas de desync, différentes = log la première frame qui diffère et échoue (information précieuse pour debug).

4. **Vidéos** : encode chaque scénario en MP4 (960×540, 60 FPS), stocke dans `target/videos/<commit>/`.

5. **Page de revue** : `scenarios-review.py` génère une page HTML avec tableau des scénarios (trace, FPS, moments clés détectés : vagues, kills, morts, tirs, rechargements, perte de santé) ; tableau d'historique par commit avec courbes FPS (benchmark vs sim).

6. **Notes** : Markdown généré (`tests/review-notes/<commit>.md`) avec lien vers la page de revue, timestamp, commit, résumé des étapes (✅/❌), métriques agrégées (FPS min par scénario, nombre de desyncs, p2p clients testés).

**Résultats** (artefacts GitHub Actions) :
- Vidéos MP4 (évaluables sans machine avec GPU)
- Page de revue HTML (tableau des FPS, moments clés, comparaison historique)
- Traces P2P (fichiers texte, premiers desyncs identifiables avec `diff`)
- JSON de métriques (entrées pour graphiques, alertes de régression)

**Si p2p échoue** : examine les logs dans `logs/p2p-<N>-<cid>.log` (socket, timeout, spawn, GGRS frame error) ; les traces dans `target/nightly/<commit>/p2p-*.trace` montrent la divergence. Deux clients en headless => 10 minutes avec `--quick`.

---

## 8. Combat : équipes et dégâts (T1.1, chantier B1) {#section7}

**Équipe** (`sim_core::team::Team` : `Players`, `Enemies`, `Allies`, `Neutral`) : composant statique posé une fois à la création (`character::create::create_character`, via `Team::Players`/`Team::Enemies` en dur ; `Allies`/`Neutral` réservés aux chantiers futurs). Hors rollback (jamais muté en T1.1 ; voir la doc du composant pour la justification). `Allies` compte comme la même équipe que `Players` pour le tir ami ; `Enemies` est sa propre équipe. `Neutral` bloque toujours un coup (comme un mur) mais ne subit jamais de dégât.

**`DamageEvent`** (`sim_core::damage`, `FrameEvents<DamageEvent>`) : toute blessure passe par là — trois émetteurs (`weapons::bullet_rollback_collision_system`, `weapons::melee::melee_hitbox_collision_system`, `character::enemy::ai::behavior::enemy_attack_damage_translate_system`), un seul résolveur (`character::health::rollback_resolve_damage_events`, `RollbackSystemSet::CollisionDamage`, après les émetteurs, avant `DeathManagement`). Le résolveur applique `combat::damage::resolve_damage` (fonction pure, testée dans `crates/combat/src/damage.rs`) puis écrit `DamageAccumulator`/`HitBy` (appliqués à `Health` par `rollback_apply_accumulated_damage`, inchangé).

**Politique de tir ami** (`sim_core::damage::FriendlyFire` : `Never`/`Always`/`Cursed`, `#[serde(default)]` = `Never`) : champ RON `friendly_fire` sur `WeaponConfig` (`weapons.ron`), `MeleeWeaponConfig` (`melee_weapons.ron`) et `EnemyAiConfig` (en dur, pas encore RON). `Cursed` ne touche un allié que si la source porte le tag `cursed` (`combat::team::CURSED_TAG`).

**Tags et défenses** : `CharacterConfig` (`character/config.rs`) gagne trois champs RON, tous `#[serde(default)]` (vides) : `tags: [...]` (posé en composant `sim_core::tag::Tags`, ex. `["cursed"]`, `["zombie"]`), `immune_to: [...]` et `resistances: {"tag": "mult"}` (posés ensemble en composant `combat::damage::Defenses`). `Tags`/`Defenses` sont hors rollback, comme `Team`. Chaque `DamageEvent` porte `tags: Tags` = tags du personnage source **union** un tag de genre d'attaque posé par l'émetteur (`bullet` pour une balle, `melee` pour une attaque au corps à corps — arme ou griffe) : une griffe de zombie porte donc `melee` et `zombie` (le tag vient de `CharacterConfig::tags` du zombie). Une cible dont `immune_to` contient un de ces tags ne subit aucun dégât ; `resistances` multiplie le montant à la place. `DamageKind::True` ignore résistances, immunités et invulnérabilité (mais pas l'équipe/le tir ami).

**Invulnérabilité** : `Health.invulnerable_until_frame` (déjà présent, T0.2) est enfin lu par `resolve_damage` — aucun dégât tant que `frame <= invulnerable_until_frame`.

**Ce qui reste de `CollisionLayer`/`layer_matrix`** (`collider/mod.rs`) : uniquement les collisions **physiques** avec les murs (balles et personnages qui s'arrêtent sur un mur) et entre personnages/fenêtres. La décision « cette balle/cette mêlée touche-t-elle ce personnage » ne passe plus par la matrice : elle vient de `Team` + `combat::team::team_allows_hit` (équipe + politique de tir ami de l'arme), évaluée pour chaque personnage candidat indépendamment de son `CollisionLayer`.

**Scénarios de référence** (`tests/scenarios/`) : `friendly_fire_never.ron`, `friendly_fire_cursed.ron`, `immune_tag.ron`. Format `Scenario` étendu (`game::replay`) : `PlayerScript` gagne `tags`/`immune_to` (posés en composants après création par `scenario::runner::apply_player_overrides`) ; `WeaponOverride` gagne `friendly_fire` (posé sur l'arme entière, avant la création des joueurs, par `apply_weapon_overrides`).

**Bug pré-existant préservé (pas corrigé par T1.1)** : avant ce chantier, l'attaque directe de `EnemyAiConfig::attack_damage` (`enemy_attack_system`) n'avait presque jamais d'effet — elle écrivait dans `DamageAccumulator` sans `Option`, composant retiré dès qu'appliqué la même frame, donc absent la plupart du temps. Seule la griffe via hitbox (`MeleeWeaponConfig::damage`, ex. `zombie_claws` à 2.5) touchait vraiment le joueur. `enemy_attack_damage_translate_system` reproduit ce comportement à l'identique (garde explicite, voir sa doc) : le corriger changerait l'équilibrage (dégâts ennemis beaucoup plus fréquents) sans rapport avec ce chantier.

## 9. Stats et modificateurs (T1.2, chantier B2)

**Contrats** (`sim_core`) : `stats::{StatId, Stats}` (identifiants et valeurs de base, `BTreeMap<StatId, Fixed>`) et `modifier::{ModifierOp, ModifierSource, Modifier, Modifiers, resolve}`. `StatId` est un enum ouvert (`Custom(String)` pour un vocabulaire propre à un jeu) ; `ModifierOp` a trois opérations `Set` (dernier gagne), `Add` (somme), `Pct` (pourcentage additif, `0.5` = +50 %, plusieurs `Pct` s'additionnent avant de multiplier une seule fois) et `Mul` (produit direct). `resolve(base, modifiers, frame)` calcule `(base + Σ Add) × (1 + Σ Pct) × Π Mul`, `Set` appliqué d'abord ; un `Pct` absent ne multiplie même pas par 1 (aucune dérive possible en l'absence de modificateur). Le crate ne pose rien sur aucune entité — c'est le rôle de `crates/stats` et de `character::create::create_character`.

**Branchement** (`crates/stats`) : `StatsPlugin` enregistre `Stats`/`Modifiers` en rollback (`RollbackTraceApp`, donc dans le `Checksum` GGRS et les traces) et ajoute `expire_modifiers_system` (`RollbackSystemSet::Status`), qui retire les modificateurs expirés de chaque entité à la frame courante (`Modifiers::retain_active`). `StatReader` (`SystemParam`) est le point de lecture unique : `stats.get(entity, &StatId::X, valeur_par_défaut)` résout la stat (base + modificateurs actifs) à la frame courante ; `stats.try_get(entity, &StatId::X)` renvoie `None` plutôt qu'un défaut si l'entité n'a pas la stat (utilisé quand réutiliser une valeur déjà en place comme base la ferait dériver sous un modificateur multiplicatif, voir `character::health::sync_health_from_stats`). Aucun système de `crates/game` ne rappelle `sim_core::modifier::resolve` directement.

**Modificateurs sourcés** : `Modifiers::push_from(source, stat, op, value, until)` construit et ajoute un modificateur ; `Modifiers::remove_by_source(&source)` retire tous les modificateurs d'une même source d'un coup — un statut qui expire ou une salle qu'on quitte appelle cette méthode avec le `ModifierSource` donné à la pose, sans connaître le détail des modificateurs posés (les chantiers de statuts B3 et de salles E1 s'en serviront).

**Stats posées à la création** (`character::create::create_character`) : pour tout personnage, `MoveSpeed`/`EnemyMoveSpeed` = `movement.max_speed`, `Acceleration` = `movement.acceleration`, `SprintMultiplier` = `movement.sprint_multiplier`, `MaxHealth` = `base_health.max`, `HealthRegen` = `base_health.regen_rate` si présent, et les multiplicateurs `FireRate`/`ReloadSpeed`/`Damage`/`Range` à 1 (produit neutre). Pour un ennemi (`enemy::create::spawn_enemy`), en plus : `SeparationDistance`/`SeparationForce`/`SlowDownDistance`/`OptimalAttackDistance` viennent de `PathfindingConfig::default()` (la constante jusque-là partagée par tous les ennemis, lue par `move_enemies`). `CharacterConfig::stats` (RON, `#[serde(default)]`) surcharge ensuite ces valeurs de base, en dernier — un personnage qui ne déclare rien dans `stats:` a un comportement strictement identique à avant ce chantier. Exemple RON :

```ron
stats: {
    MoveSpeed: "180.0",
    Custom("sacre"): "1.0",
},
```

**Lecture branchée** : `character::player::input::apply_inputs` (accélération, multiplicateur de sprint, vitesse max), `character::enemy::ai::pathing::move_enemies` (vitesse, séparation, ralentissement, distance d'attaque optimale — la granularité de la grille spatiale de séparation reste la constante globale de `PathfindingConfig`, seulement pour le partitionnement, voir le commentaire dans `move_enemies`), `weapons::weapon_rollback_system`/`spawn_bullet_rollback` (cadence de tir, temps de rechargement, dégâts et portée d'une balle, multipliés par les stats du porteur) et `character::health::sync_health_from_stats` (`Health.max` et `HealthRegen.regen_rate`, recalculés chaque frame après l'expiration des modificateurs — `current` est borné à `max` s'il le dépasse, jamais relevé automatiquement). `character::player::input::apply_friction` et les champs de dash (`dash_distance`, `dash_duration_frames`, `dash_cooldown_frames`, `sprint_acceleration_per_frame`/`sprint_deceleration_per_frame`) restent des lectures directes de `CharacterConfig` : ils ne sont pas dans la liste de stats de la tâche T1.2.

**Scénario de preuve** (`tests/scenarios/stat_move_speed.ron`) : deux joueurs avancent avec le même script d'inputs, l'un avec un modificateur `(stat: MoveSpeed, op: Mul, value: "0.5")` posé après création (`PlayerScript::modifiers`, comme `tags`/`immune_to`, appliqué par `scenario::runner::apply_player_overrides`), l'autre sans — `PlayerPosition` aux mêmes frames pour les deux, lecture directe de l'écart.

## 10. Blesser une trace : la preuve

Enregistrer un nouveau composant/ressource en rollback (`RollbackTraceApp`) change le `Checksum` GGRS de chaque frame, donc **toutes** les traces de référence (`tests/scenarios/*.trace`) changent, même quand aucune valeur de jeu ne bouge. Avant de blesser (`BLESS=1`), il faut prouver que c'est bien le cas : que la nouvelle trace ne diffère de l'ancienne que par le nouveau composant lui-même.

**Piège de parité des types vides** (mesuré le 2026-10-03, T1.2) : un type rollback enregistré
sous checksum mais porté par aucune entité ajoute une `ChecksumPart` constante `hash(0u64)`,
combinée par XOR : **un** type vide de plus change toutes les traces dès la frame 0, **deux** les
laissent intactes. Un composant qui n'est posé que sur de nouvelles entités s'enregistre avec la
variante **neutre** (`rollback_and_trace_neutral::<C>`, `rollback_and_trace_resource_neutral`) ;
on ne compense jamais par la parité et on ne passe jamais en `no_checksum` pour « sauver » des
traces. Détail et mesure : §20.

**Dump complet** (`ALACOD_DUMP_TRACE=<dossier>`) : `StateTraceRecorderPlugin::dump: Option<PathBuf>` (`crates/game/src/state_trace.rs`) fait garder à `StateTraceRecorder` la ligne détaillée (hash puis, comme en mode `full`, une ligne par ressource tracée et une ligne par entité rollback triée par `GgrsNetId`) de **toutes** les frames simulées, sans la limite de `history` (`HISTORY_LEN`, 400). `crates/scenario/src/runner.rs::build_app` lit la variable d'environnement ; `ScenarioOutcome::full_trace` porte ces lignes (`None` si `ALACOD_DUMP_TRACE` n'est pas défini) ; le test `scenarios` (`crates/scenario/tests/scenarios.rs`) les écrit dans `<dossier>/<scénario>.full` pour chaque scénario joué.

**Comparaison** (`scripts/trace-diff.py <a.full> <b.full> [--ignore Nom1,Nom2,...]`) : retire toujours le checksum de l'en-tête (il change dès qu'un composant est ajouté au rollback, ce n'est pas ce qu'on compare) et neutralise toute valeur `Entity` brute embarquée dans un Debug (`{index}v{generation}`, ex. `WeaponInventory.weapons: Vec<(Entity, Weapon)>`) : elle dépend du nombre exact d'entités déjà créées dans **ce process** au moment du spawn (minutage du chargement des assets), pas déterministe d'un lancement à l'autre même à code strictement identique — c'est déjà pour ça que `WeaponInventory` l'exclut de son `Hash` manuel (§6 et sa doc dans `crates/game/src/weapons/mod.rs`). Avec `--ignore`, retire en plus les paires `Nom=valeur` dont le dernier segment du nom (après `::`) est dans la liste — sur les lignes de ressource et d'entité (une ligne d'entité empile un tracer par composant présent, `--ignore` ne retire que ceux nommés). Lit les deux fichiers en flux (une frame à la fois) : un dump complet de 1500 frames pèse facilement plusieurs centaines de Mo. Affiche la première frame qui diffère et les lignes en cause ; code de sortie 1 si différent, 0 si identique.

**Méthode** : dumper les mêmes scénarios sur la référence (avant le chantier, ex. `main` dans un `git worktree add --detach`) et sur la branche, avec la même variable d'environnement, puis comparer en ignorant le(s) nouveau(x) composant(s)/ressource(s). Aucune différence : le chantier n'a changé que la présence du nouveau composant, jamais une valeur de jeu — on peut blesser en confiance. Une différence : une valeur de stat (ou autre) ne correspond pas exactement à la constante qu'elle remplace ; corriger avant de blesser.

### Exclusions assumées du checksum

Deux champs de test (testbed) sont délibérément hors du checksum GGRS — les inclure déplacerait les traces de référence sans changer aucun comportement de jeu (D5, m0-v10) :

- **`HitCount`** (`crates/game/src/character/health/mod.rs`, T2.9) : compteur de coups reçus, posé à la création uniquement si `CharacterConfig::counts_hits` (`crates/game/src/character/create.rs`, faux par défaut — aucun personnage zombie/joueur ne le pose ; seul `target` de `games/testbed` le déclare, `crates/scenario/src/generate.rs`). Enregistré via `rollback_and_trace_no_checksum::<HitCount>()` (`crates/game/src/character/mod.rs`) : rollback et trace, mais hors checksum. Il sert aux attentes `EntityHits` des scénarios (`crates/combat/src/weapons/expectations.rs`) ; aucune décision de jeu ne le lit.
- **`EnemyAiConfig::stationary`** (`crates/game/src/character/enemy/ai/state.rs`, T2.9) : immobilité totale d'un ennemi de testbed (`dummy`/`target`/`ally`/`civilian`), exclue du `Hash` manuel du composant. Aucun contenu zombie ne la pose ; hacher un champ de plus déplacerait le checksum de toute entité `Enemy` sans aucun changement de gameplay (vérifié empiriquement à l'époque : sans l'impl manuel, `idle.ron` diverge dès f181).

## 9. Feedback (présentation, T2.13)

**Configuration RON** : fichier `games/<jeu>/assets/ui/feedback.ron` charge les paramètres de flash, secousse et sons.

```ron
(
    hit_flash: (
        frames: 4,                  // Durée du flash blanc (frames de simulation)
        color: (1.0, 1.0, 1.0),     // Couleur d'éclaircissement (RGB)
    ),
    shake: (
        frames: 8,                  // Durée de la secousse
        amplitude: 4.0,             // Amplitude du décalage en pixels
    ),
    sounds: {
        "shot": "sounds/machine-gun.ogg",
        "reload": "sounds/machine-gun-reload.ogg",
        // Les clés absentes désactivent le son correspondant
    },
)
```

**Systèmes** : tous en `PostUpdate` (hors `GgrsSchedule`), lisant les événements de simulation dans `FrameEvents<T>` émis par `GgrsSchedule`. Les trois émetteurs sont :
- `DamageEvent` pour les impacts (flash, secousse si joueur local)
- Spawn de `Bullet` pour le son de tir (source = joueur local) : un son par tir (tireur, frame de création), mémorisé pour ne pas repartir quand le rollback de la session locale recrée les balles des dernières frames (elles redeviennent `Added<Bullet>`), et coupé en fondu après 250 ms (`SHOT_SOUND_MAX` : `machine-gun.ogg` est un enregistrement de tir soutenu de 17 s, pas un coup unique ; à remplacer)
- Passage de `WeaponInventory.reloading_ending_frame` de `None` à `Some` pour le son de rechargement : une fois par rechargement, comparé d'une image rendue à la suivante

**Composants non-rollback** :
- `HitFlash { until_frame, original_color }` : pose sur l'entité cible d'un `DamageEvent`, tinte le sprite en blanc jusqu'à `until_frame`.
- `CameraShake { until_frame, amplitude }` : pose sur la caméra quand un joueur local prend des dégâts.

**Déterminisme** : la secousse applique un motif déterministe (décalage indexé par `FrameCount`) pour la reproductibilité des captures (`--capture` de `play_scenario`). Jamais de source aléatoire (`rand`, temps réel).

**Journal de preuve** : chaque effet écrit une ligne `info!("feedback f{frame} <effet> {net_id|kind}")` pour vérification sans écran.

## 11. Munitions et inventaire d'armes (T2.2, chantier B7)

**Type de munition** (`sim_core::ammo::AmmoType`) : enum ouvert (`Plomb`, `Balle`, `Cartouche`, `Energie`, `Special`, `Custom(String)`), comme `StatId`. Champ RON `ammo_type` sur `WeaponConfig` (`weapons.ron`), **obligatoire** pour toute arme à distance (pas de `#[serde(default)]` : une entrée sans `ammo_type` échoue au chargement RON, rapportée par `content::lint` comme n'importe quel autre littéral invalide). Contenu `zombies`/`testbed` : `pistol` → `Plomb`, `machine_gun` → `Balle`, `shotgun` → `Cartouche` (trois types distincts aujourd'hui, donc chaque arme a sa réserve propre en pratique). Exclu du hash manuel de `WeaponConfig` (comme `test`, voir §6 du fichier lui-même) : une vraie valeur de gameplay, mais dont la valeur observable vit dans `AmmoReserves` (rollback), pas dans ce champ de config statique — l'y inclure ferait dériver le checksum de tous les scénarios existants sans qu'aucun comportement ne change.

**Réserves** (`combat::inventory::AmmoReserves`, composant rollback posé sur chaque joueur par `character::player::create::create_player`) : `BTreeMap<AmmoType, u32>`, une entrée par type de munition **effectivement représenté** parmi les armes de départ (rien pour un type jamais rencontré). Initialisée à la somme, par type, de `mag_limit × mag_size` du mode par défaut de chaque arme de départ (`weapons::default_mode_ammo_contribution`) — deux armes qui déclarent le même type s'additionnent (scénario `ammo_shared_reserve`). Une arme `Magless` (fusil à pompe : pas de chargeur séparé, tout son stock est `bullet_limit`) contribue `0` : son ravitaillement ne passe jamais par la réserve, avant comme après ce chantier.

**Rechargement** (`weapons::weapon_rollback_system`) : un rechargement (bouton ou chargeur vide) exige que la réserve du type de l'arme active contienne au moins un chargeur plein (`mag_size` unités) — `WeaponModeState::can_reload(mag, reserve)`. À la fin du délai de rechargement, l'arme retire `mag_size` unités de la réserve et remplit son chargeur (`WeaponModeState::reload`), exactement comme l'ancien `mag_quantity` (un chargeur entier par rechargement, jamais un appoint partiel) — seule l'unité change (munitions plutôt que nombre de chargeurs), pour un total de munitions tirables strictement identique. Une arme `Magless` ne passe jamais ce test (`can_reload` renvoie toujours faux pour `MagBulletConfig::Magless`) : son « rechargement » entre deux tirs (fusil à pompe) reste un délai de pompe qui ne touche jamais la réserve, comme avant.

**Emplacements** (`CharacterConfig::weapon_slots: u32`, `#[serde(default)]` = 2, validé par `content::lint` : `> 0` et `starting_weapons.len() <= weapon_slots`) : nombre d'armes à distance qu'un personnage peut porter. À l'exécution, borne le **ramassage** (voir plus bas) ; la création donne toutes les `starting_weapons`, le lint refuse désormais une liste trop longue (T3.4, D7). `zombies` et `testbed` déclarent `weapon_slots: 3` sur le joueur : ses trois armes de départ étaient déjà créées, aucune n'était perdue.

**Lâcher** (`INPUT_DROP_WEAPON`, touche `G`, bouton `DropWeapon` des scénarios ; `weapons::weapon_drop_system`, avant `weapon_rollback_system` dans `GgrsSchedule`) : l'arme active est retirée de `WeaponInventory.weapons`, son entité despawn (`despawn_rollback`), et une entité rollback `weapons::WeaponPickup { weapon_id, mag_ammo }` apparaît au sol à la position du joueur — `mag_ammo` est celui du **mode actif** au moment du dépôt (un seul nombre, pas un par mode). Le rechargement en cours, s'il y en avait un, est annulé (`WeaponInventory::clear_reloading`) : l'arme qui rechargeait n'existe plus.

**Ramasser** (`INPUT_INTERACTION`, `InteractionType::Weapon`, `interaction::handle_weapon_pickup_interaction`) : équipe l'arme (devient active), restaure son chargeur (`WeaponPickup::mag_ammo`, via `spawn_weapon_for_player`, paramètre `initial_mag_ammo`). En dessous de `weapon_slots`, elle s'ajoute ; à `weapon_slots` ou au-delà, elle **remplace** l'arme active, qui tombe au sol à son tour (même mécanisme, `weapons::spawn_weapon_pickup`). La réserve de munitions n'est jamais transférée : seul le chargeur voyage avec l'arme, la réserve reste attachée au joueur.

**Réutilisation par T2.3** (armes murales) : `weapons::spawn_weapon_pickup(commands, weapon, mag_ammo, position, price, id_factory)` est le point d'entrée commun — `price: Option<u32>` est ignoré par ce chantier (toujours `None`), réservé à l'affichage/vérification d'un coût par les armes murales.

**Scénarios de référence** (`tests/scenarios/`) : `ammo_shared_reserve.ron` (deux armes forcées sur le même type via `weapon_overrides.ammo_type`, la réserve baisse quelle que soit l'arme qui recharge), `drop_pickup_swap.ron` (lâcher, `WeaponPickups`, ramasser). `weapon_overrides` gagne `ammo_type: Option<AmmoType>` (s'applique à l'arme entière, comme `friendly_fire`). Nouvelles attentes : `AmmoReserve { handle, ammo_type, amount, at_frame }` (exact, comme `Ammo`), `WeaponPickups { min, max, at_frame }` (nombre d'entités `WeaponPickup`, tous joueurs confondus). Nouveaux `GameEvents` : `drop`/`pickup` (détectés par `scenario::events::detect_events` sur l'apparition/disparition d'un `GgrsNetId` de `WeaponPickup`).

## 12. Monnaie et achats (T2.3, chantier C5 v1)

**Monnaie** (`run::currency::Currency`, crate minimal `crates/run`) : composant rollback `u32` posé sur chaque joueur à sa création (`character::player::create::create_player`), valeur de départ `CharacterConfig::starting_currency` (défaut 500, `zombies` le déclare explicitement dans `player_config.ron`). API saturante : `can_afford`/`spend` (refuse sans rien changer si le solde est insuffisant)/`earn`. Chaque variation passe par `FrameEvents<run::currency::CurrencyEvent>` (`{ handle, delta, reason }`, `delta > 0` = gain, `< 0` = achat, `== 0` = achat refusé faute de solde) — lu par le HUD (source `currency`), par `scenario::events::detect_events` (`GameEvents` : `points`/`purchase`/`purchase_refused`, classés par le signe de `delta`) et par T2.5 (power-ups).

**Points** (`game::economy`) : détectés au plus près de l'information disponible (kill dans `character::health::rollback_apply_accumulated_damage`, au moment où `Death` est posée sur une entité qui n'est pas un joueur avec `HitBy::Player` en dernier coup ; coup au but dans `rollback_resolve_damage_events`, à chaque `DamageEvent` résolu d'un joueur vers une cible qui n'en est pas un ; réparation de fenêtre dans `interaction::handle_window_repair`), poussés dans `FrameEvents<economy::PointsCredit>` et résolus une seule fois par `economy::award_points_system` (`RollbackSystemSet::Run`, en fin de frame) qui lit `economy.ron` et crédite `Currency`. Réparation plafonnable par joueur et par vague (`EconomyConfig::repair_points_cap_per_wave`, optionnel — `economy::RepairPointsTracking`, remise à zéro au changement de vague).

**Portes payantes** : `handle_door_interaction` débite `DoorConfig::cost` (existant, calculé par le générateur selon la profondeur de la salle — voir `BasicMapGeneration::get_doors`, 750 + 250×profondeur) avant d'ouvrir ; `cost <= 0` reste gratuit (comportement inchangé). Solde insuffisant : refus silencieux, porte fermée, `CurrencyEvent { delta: 0 }`.

**Armes murales** (`WeaponLocation`, §1) : `interaction::handle_weapon_pickup_interaction` distingue une arme murale (`weapons::WeaponPickup::price: Some(...)`) d'une arme au sol ordinaire (`None`, T2.2) — une arme murale n'est jamais consommée au ramassage (le mural reste achetable) et exige `Currency >= price`, ou `EconomyConfig::refill_price(price)` (ratio `refill_price_ratio`, défaut 0.5×) si le joueur possède déjà une arme du même id, auquel cas l'interaction recharge la réserve de munitions du type au lieu d'équiper une seconde copie. Anti-rebond obligatoire (`WeaponPickup::can_buy_after_frame`, comme `WindowHealth::can_repair_after_frame`) : sans lui, maintenir Interaction facturerait le joueur à chaque frame.

**Perks** (`SodaLocation`, §1, kind de contenu `Perk`, `games/<jeu>/assets/economy/perks.ron`) : table `{ "id": (name: "...", price: N, modifiers: [(stat: ..., op: ..., value: "...")]) }` (fichier RON, newtype sur la table — parenthèse ouvrante/fermante autour de tout le fichier, comme `weapons.ron`). Achat = interaction (`InteractionType::Perk`) sur une entité `economy::PerkMachine` ; un seul achat par perk et par joueur (`run::perks::Perks`, `BTreeSet<String>`, vérifié **avant** de débiter — déjà possédé n'émet ni achat ni refus). Modificateurs posés permanents (`until: None`, `sim_core::modifier::ModifierSource::Named("perk:<id>")`) via `Modifiers::push_from`. Un perk qui relève `MaxHealth` (ex. Juggernog, `Mul "2.0"`) relève aussi `Health.current` de la même différence au moment de l'achat (`sync_health_from_stats`, `RollbackSystemSet::Status`, ne fait que plafonner `current` à la baisse, jamais à la hausse — voir sa doc) : décision propre à l'achat d'un perk (un statut temporaire qui relèverait `MaxHealth` puis expirerait ne doit pas, lui, soigner le joueur).

**Économie RON** (kind de contenu `Economy`, `games/<jeu>/assets/economy/economy.ron`, un seul fichier) : `kill_points` (défaut 60), `hit_points` (défaut 10), `repair_points` (défaut 10), `nuke_points` (défaut 400, D17 : crédité à chaque joueur vivant au ramassage d'un nuke), `repair_points_cap_per_wave` (`Option<u32>`, défaut aucun plafond), `refill_price_ratio` (`Fixed` en chaîne, défaut `"0.5"`). Chargé par le registre comme `Wave` (id = nom de fichier). Lint (`content::lint::lint_perks`, `lint_economy`) : `price > 0`, au moins un modificateur, `value > 0` pour un `op: Mul`, id unique dans `perks.ron` ; `refill_price_ratio` dans [0, 1] ; un `StatId` inconnu dans `perks.ron` échoue déjà au chargement RON (pas de variante fourre-tout implicite). Voir le tableau des règles, §3.

**Scénarios de référence** (`tests/scenarios/`) : `buy_door`, `buy_wall_weapon`, `buy_perk` (sur `games/zombies/assets/exemples/test_map_shop.ldtk`, copie de `test_map.ldtk` avec une `WeaponLocation`/`SodaLocation` dans la salle de départ — `test_map.ldtk` n'en avait pas avant T2.6, qui en a posé quatre de chaque et re-blessé toutes les traces de cette carte), `points_on_kill` et `shop_tour` (sur `test_map.ldtk`). Nouvelles attentes de scénario : `Expectation::Currency { handle, min, max, at_frame }`, `Expectation::Stat { handle, stat, value, at_frame }` (valeur résolue, comme `stats::StatReader`) ; `PlayerScript::currency: Option<u32>` (solde de départ, comme `weapon`).

## 13. État de run et mode `Waves` (T2.4, chantier F1)

**`Run`** (`run::run::Run`, crate `run`) : ressource rollback (checksum + trace, `RunPlugin`) qui représente la partie en cours — `seed: u32` (graine, dupliquée de `RunSeed` pour que `Run` seule décrive la partie), `mode: RunMode` (`Waves { config }` = le système de vagues actuel, `Sandbox` = aucune condition de fin hors défaite ; les autres modes du plan, `Floors`/`Campaign`, viendront en M1/M2), `step: RunStep` (`Lobby` — valeur par défaut de l'enum, jamais celle d'un `Run` réellement inséré, voir plus bas —, `Playing { since_frame }`, `Ended { at_frame, outcome: RunEnd }`), `players: Vec<usize>` (handles GGRS triés), `flags: BTreeSet<String>` (ouvert pour de futurs modes, vide aujourd'hui) et `summary: Option<RunSummary>`. Remplace `combat::downed::RunOutcome` (T1.3), qui disparaît : la défaite est maintenant une valeur de `RunStep`, pas une ressource à part. **Décision** : `RunSummary` est un champ de `Run`, pas une ressource rollback séparée — un seul enregistrement, pas de valeur sentinelle à inventer pour « pas encore de résumé » (`RunEnd` n'a pas de variante neutre).

**Création** : `Run::new(seed, mode, players, since_frame)` construit toujours directement `Playing` (jamais `Lobby` dans ce chantier : réservé à un futur état où `Run` existerait déjà pendant un lobby interactif, M1/M2). Appelée par `game::jjrs::{local, p2p}` (`system_after_map_loaded_local`/`system_after_map_loaded`, `OnEnter(AppState::GameStarting)`), au même instant que `RunSeed`/`RngStreams` (T1.6) — avant que `GgrsSchedule` ne tourne pour la première fois (aucune session GGRS encore présente). `players` vient de `GgrsSessionBuilding.players[].handle`. `mode` vient de `game::run_state::resolve_run_mode(manifest, registry)` : `entry.mode` du manifeste (`content::manifest::EntryMode::Waves`/`Sandbox`, enum fermé comme `sim_core::damage::FriendlyFire`) s'il est présent, sinon `Waves` si le jeu déclare un dossier de contenu `Wave`, sinon `Sandbox` — `content::lint` refuse `entry.mode: Waves` sans dossier `Wave` déclaré (`lint_entry_point`), mais pas l'absence d'`entry.mode` avec un dossier `Wave` présent (c'est le chemin par défaut).

**Syntaxe RON du mode optionnel** (T3.4, D16) : `entry: (..., mode: Waves)` ou `mode: Sandbox`. Le chargeur de manifeste active `implicit_some` sans directive en tête du fichier ; `Some(Waves)`/`Some(Sandbox)` et `None` restent acceptés. Les deux jeux déclarent leur mode explicitement. Sans ce champ, le mode est résolu comme décrit ci-dessus.

**Fin de partie** : deux voies, jamais concurrentes (`Run::is_playing()` gardé par les deux avant d'écrire `step`, une seule issue jamais réécrite).
- **Défaite** (universelle, tout mode confondu, T1.3) : `character::health::rollback_check_defeat` (`RollbackSystemSet::DeathManagement`, inchangé sinon) écrit `Ended { outcome: RunEnd::Defeat, .. }` dès que tous les joueurs en jeu sont à terre ou morts.
- **Victoire** (propre au mode, `run::modes::RunModeRules`, trait implémenté pour `RunMode`) : `game::run_state::check_run_victory_system` (`RollbackSystemSet::Run`, après `award_points_system`) appelle `run.mode.check_victory(&ctx)` avec un `RunContext` construit depuis `WaveState`/`WaveConfig::max_wave`. Pour `Waves` : victoire si `WaveConfig::max_wave` (`Option<u32>`, `#[serde(default)]`, absent = jamais de victoire, comme avant ce chantier) est déclaré et atteint. `Sandbox` ne gagne jamais.

`game::run_state::finalize_run_summary_system` (`RollbackSystemSet::Run`, après les deux systèmes ci-dessus) calcule `Run.summary` une seule fois, dès que `step` devient `Ended` et que `summary` est encore `None` : `RunModeRules::summarize` construit `RunSummary { wave_reached, kills, points_total, frames, outcome }` (`wave_reached`/`kills` viennent de `WaveState` ; `points_total` = somme des soldes `Currency` de tous les joueurs à cet instant — pas un total cumulé des gains, qui n'existe nulle part ailleurs dans le jeu ; `frames` = `Run.step`'s `at_frame`).

**`waves/systems.rs`** : logique inchangée ; les systèmes de vagues ne lisent pas `Run.step`. Le spawn s’arrête quand il ne reste plus de joueur (`player_positions.is_empty()`), mais un run terminé avec des joueurs encore présents ne suspend pas les vagues. Cette limitation est conservée pour ne pas changer le gameplay ni les traces de ce chantier.

**Écran de fin** (`game::ui::game_over`) : dérivé de `Run.step` (pas de `RunOutcome`), affiche le résumé lisible une fois `Run.summary` disponible (`finalize_run_summary_system` tourne après la détection de fin, dans la même frame de simulation).

**Relance sans relancer le binaire** (`game::run_state::RunRequest`, hors rollback — ni checksum ni trace, comme `NextState`) : `Restart` (même config : même carte, même graine, mêmes joueurs) ou `ToLobby` (retour `LobbyLocal`/`LobbyOnline` selon `OnlineState`), posée par le bouton de fin de partie ou la touche `R`. `apply_run_request_system` (`Update`, hors `GgrsSchedule`) consomme la commande et change d'`AppState` ; la destruction proprement dite se déclenche sur `OnExit(AppState::InGame)` (quelle que soit la cause), dans deux systèmes indépendants (aucun appel direct entre crates) :
- `game::run_state::cleanup_rollback_world_system` : détruit (`try_despawn`, silencieux sur une entité déjà partie — un despawn racine est récursif, voir sa doc) toute entité `Rollback` (joueurs, ennemis, balles, armes, murs, portes, machines à perk...), retire `Session<PeerConfig>` et `Run`, remet à zéro `FrameCount`, `GgrsNetIdFactory`, `WaveState`, `FlowFieldCache`, `RepairPointsTracking` (ressources rollback globales qu'aucun autre système ne réinitialise).
- `map_ldtk::game::plugin::despawn_ldtk_world_on_exit_ingame` : détruit l'entité racine LDtk (`LdtkProjectHandle`) et donc tout son arbre (niveaux, calques, tuiles, et toute entité LDtk encore dessous — portes, fenêtres, spawners, `WeaponLocation`/`SodaLocation`...). Vit dans `map_ldtk` (pas `game`) parce que `game` ne dépend pas de `bevy_ecs_ldtk`.

Ressources GGRS internes (`RollbackFrameCount`, `ConfirmedFrameCount`, `MaxPredictionWindow`, `LocalPlayers`) : pas touchées explicitement, `bevy_ggrs::schedule_systems::run_ggrs_schedules` les réinitialise de lui-même dès qu'aucune `Session` n'est présente. **Volontairement pas détruites** : `RunSeed`, `RngStreams`, `MapGenerationConfig` (via `LdtkGameMap`, jamais retirée de l'app), `GggrsSessionConfiguration`/`GgrsSessionBuilding` — c'est justement ce qui permet à `Restart` de rejouer « la même configuration » sans reconstruire un lobby : `jjrs::{local, p2p}` les écrase simplement des mêmes valeurs au prochain `OnEnter(AppState::GameStarting)`. `LdtkMapEntityLoadingRegistry` (état interne du chargement de carte, `map_ldtk`) est remise à zéro sur `OnEnter(AppState::GameLoading)` plutôt que sur `OnExit(InGame)` : sinon `wait_for_all_map_rollback_entity` la voit déjà `loading_complete` (partie précédente) et ne relit jamais les entités de la nouvelle carte.

**Restart en p2p** : non supporté par ce chantier — `apply_run_request_system` redirige `Restart` vers `ToLobby` quand `OnlineState::Online` (le bouton « renvoie au lobby », averti par un `warn!`).

**Abandon et lobby local (D13)** : quitter vers le lobby une partie encore en cours la termine en `Ended { outcome: Abandon }` **avec son résumé** (`apply_run_request_system`, mêmes champs qu'une défaite via le calcul partagé `run_summary`). Un retour au lobby **local** pose `LocalLobbyHold { summary }` (hors rollback) : `setup_ggrs_local` attend qu'elle disparaisse (condition dans `core.rs`), l'écran du lobby (`ui/lobby.rs`) affiche le résumé de la partie quittée et la retire sur Entrée ou « Nouvelle partie ». Au premier lancement, pas de `LocalLobbyHold` : le lobby local démarre aussitôt, comme avant (scénarios, `make zombies`). Décision pure testée : `run_state::plan_run_request`.

**Scénarios et tests** : `run_lose_summary` (deux joueurs immobiles sur `test_map.ldtk`, même mise en scène que `downed_all_lose` — défaite complète à f1505) vérifie `RunState(step: Ended(Defeat), at_frame: ...)` et `RunSummary(wave_reached_min: 1, kills_min: 0, at_frame: ...)`. Nouvelles attentes de scénario : `Expectation::RunState { step: RunStepExpectation::{Playing, Ended(RunEnd)}, at_frame }` (vérification ponctuelle, comme `PlayerAlive` — `RunStep::Ended` ne change plus une fois posé) et `Expectation::RunSummary { wave_reached_min, kills_min, at_frame }` (bornes inférieures, échoue si `Run.summary` est encore `None`). `crates/scenario/tests/run.rs` : test d'intégration qui pilote `App::update()` directement (pas via `Scenario::expect`) — joue `idle` jusqu'à la défaite, pose `RunRequest::Restart`, mesure le temps mural jusqu'au retour en `InGame` à la frame 0 (doit rester sous dix secondes), rejoue 300 frames et compare la trace d'état (`StateTraceRecorder::lines_until`) à celle des 300 premières frames de la première partie — identiques si la relance n'a rien laissé traîner. `ToLobby` : vérifié au moins pour le changement d'état (`AppState` devient `LobbyLocal`).

## 14. Power-ups (T2.5, chantier C1 v0)

**Vocabulaire d'actions** (`crates/effects`, données pures ; `bevy` sert uniquement au
plugin de déclaration des kinds depuis T1.0a) : `effects::Action` est la liste fermée des effets qu'un power-up
peut appliquer, la graine du futur système de déclencheurs/effets de `docs/plan-engine.md`
§5 (« C1. Effets ») — pas de déclencheur ni de condition dans T2.5, seulement les
actions ; les contrats de M1 sont décrits au §4.3. Cinq variantes : `TimedModifier { stat, op, value, frames }` (modificateur de stat
temporaire, `frames` = durée relative à la frame de ramassage), `RefillAmmo`,
`RepairAllWindows`, `KillAllWaveEnemies`, `CurrencyMultiplier { factor, frames }`.
`Action::as_modifier(frame, source)` est l'« applicateur déterministe » pur (sans ECS,
testé unitairement) pour les deux variantes modifier-based ; les trois autres sont résolues
directement par `game::powerups` (accès à `AmmoReserves`/`WindowHealth`/`Team`, inconnus
d'`effects`).

**Sémantique CoD (décision)** : un power-up ramassé s'applique à **tous les joueurs** de la
partie, jamais au seul joueur qui l'a ramassé — aucun des cinq power-ups de référence
(Insta-Kill, Double Points, Max Ammo, Carpenter, Nuke) n'est individuel dans le jeu source.

**Table de contenu** (kind `PowerUp`, `games/<jeu>/assets/items/powerups.ron`, déclaré comme
un **dossier** dans `game.ron` — comme `characters/`, pas un fichier unique — pour qu'un
futur id dupliqué entre deux fichiers du dossier produise l'erreur `DuplicateId` habituelle) :
```ron
(
    drop_chance: "0.15",  // probabilité qu'un ennemi tué (équipe Enemies) laisse tomber un power-up
    powerups: {
        "insta_kill": (
            name: "Insta-Kill",
            weight: 15,            // poids de tirage parmi les power-ups, si un drop a lieu
            pickup_range: "30.0",  // portée de ramassage (Fixed), au passage, pas un bouton
            lifetime_frames: 1800, // durée de vie au sol avant disparition (≈ 30 s à 60 FPS)
            actions: [
                TimedModifier(stat: Damage, op: Set, value: "100.0", frames: 1800),
            ],
        ),
        // double_points: CurrencyMultiplier ; max_ammo: RefillAmmo ;
        // carpenter: RepairAllWindows ; nuke: KillAllWaveEnemies.
    },
)
```
`drop_chance` décide **si** un drop a lieu (tirage global), `weight` décide **lequel** parmi
la table (tirage pondéré, même méthode que `waves/wave_config.ron`/`select_enemy_type`).
Lint (`content::lint::lint_powerups`) : `drop_chance` dans `[0, 1]`, `weight > 0`,
`lifetime_frames > 0`, `pickup_range > 0`, au moins une action, `frames > 0` pour
`TimedModifier`/`CurrencyMultiplier` et `factor > 0` pour `CurrencyMultiplier` par
power-up (`LintErrorKind::OutOfRange`, T2.8 pour les cinq dernières), et `value > 0` pour un
`TimedModifier` avec `op: Mul` (T3.4) ; une référence de `StatId` inconnue dans
`actions[].stat` échoue déjà au chargement RON (`actions` réutilise le type réel
`effects::Action`, pas un mirroir — comme `StatId` pour `perks.ron`), rapportée
`LintErrorKind::Parse` ; un id de power-up dupliqué entre deux fichiers du dossier `PowerUp`,
`LintErrorKind::DuplicateId` (même mécanisme que `characters/`/`weapons.ron`). Fixtures :
voir le tableau des règles de lint, §3.

**Entité rollback « power-up au sol »** (`game::powerups::PowerUpPickup { id, expires_at_frame }`,
posée par `spawn_powerup_pickup`, point d'entrée commun au drop et au placement scripté de
scénario — comme `weapons::spawn_weapon_pickup` pour lâcher/armes murales) : **pas** une
`Interactable` (contrairement aux armes/fenêtres/perks) — `powerup_pickup_detect_system`
(`RollbackSystemSet::Effects`) ramasse automatiquement au passage, dès qu'un joueur entre
dans `PowerUpDef::pickup_range` (le premier dans l'ordre `GgrsNetId` s'il y en a plusieurs à
portée la même frame), despawn immédiat (`despawn_rollback`) et émission de
`FrameEvents<PowerUpPickedUp>`, résolu par `apply_powerup_actions_system` (même set,
`.after`). `powerup_expiry_system` (même set) détruit un power-up jamais ramassé à
`expires_at_frame` : à cette frame le ramassage est fermé. `PowerUpPickup` est
enregistré par `rollback_and_trace` (rollback, checksum GGRS et trace). La file
`FrameEvents<PowerUpPickedUp>` est elle aussi checksummée. Tout bless exige la preuve
décrite au §10 ; les résultats figurent dans le rapport de tâche.

**Max Ammo** : remplit tous les modes de chaque arme portée, additionne les capacités
`mag_size × mag_limit` du mode par défaut **par type de munition**, puis relève chaque
réserve à cette somme sans diminuer une réserve déjà supérieure. Un rechargement en
cours est annulé pour éviter un débit après le remplissage. Les armes `Magless` ne
contribuent pas à la réserve, mais leur stock propre est rempli.

**Nuke : points** (D17) : au ramassage, `KillAllWaveEnemies` pousse un
`economy::PointsCredit::Nuke` par joueur vivant (ordre `GgrsNetId`), pas par ennemi tué ;
`award_points_system` le résout en fin de frame avec `EconomyConfig::nuke_points` (défaut 400,
comme CoD) et le multiplicateur de points du joueur (800 sous Double Points). Les ennemis tués
par le nuke ne rapportent pas de points de kill (`Death { last_hit_by: None }`).

**Rafraîchissement** (D18) : un power-up déjà actif ramassé à nouveau ne se cumule pas
(Double Points × Double Points ne fait pas × 4). Au ramassage, `apply_powerup_actions_system`
retire d'abord, sur chaque joueur, les modificateurs de source `powerup:<id>`
(`Modifiers::remove_by_source`), une fois pour toutes les actions du power-up, puis les
repose : la durée recommence à la frame du ramassage, comme dans CoD.

**Drop à la mort** (`game::powerups::loot_drop_on_death_system`,
`RollbackSystemSet::DeathManagement`, `.after(rollback_apply_accumulated_damage)
.before(rollback_apply_death)` — même contrainte que
`waves::systems::wave_enemy_death_tracking_system`, il lui faut la position de l'entité
avant qu'elle ne soit détruite) : portée **équipe `Enemies`** (pas seulement `WaveEnemy`,
absent des personnages de laboratoire du testbed placés par `CharacterSpawn` — sans quoi le
chemin « drop à la mort » ne serait pas prouvable dans le testbed, T2.9 n'y faisant jamais
tourner le mode vagues). Tirage déterministe dans le flux RNG nommé **`loot`**
(`bevy_fixed::rng::RngStreams`, T1.6, comme `"waves"`/`"weapons"`), **dans l'ordre des
`GgrsNetId`** des morts de la frame (`order_iter!`) : un tirage « un power-up tombe-t-il »
(comparé à `drop_chance`), puis, si oui, un tirage pondéré (`weight`) pour lequel.

**Réglages de scénario** (`game::replay::Scenario`, appliqués par `crates/scenario/src/runner.rs`,
même famille que `wave_overrides`/`weapon_overrides`) :
- `powerups: [(id, x, y, at_frame)]` (`PowerUpPlacement`) : fait apparaître un power-up à une
  position (`x` et `y` en chaînes Fixed) et une frame exactes, sans dépendre d'une carte
  LDtk ni du tirage RNG — pour
  prouver l'**effet** de chaque power-up indépendamment du mécanisme de drop. Contrairement
  aux autres réglages (`Update`, une fois avant la première frame), celui-ci tourne dans
  `GgrsSchedule` (`RollbackSystemSet::Effects`, avant la détection de ramassage) : la
  condition `frame == at_frame` est naturellement rollback-safe (même raisonnement qu'un
  spawn de vague), pas de `Local<bool>` par placement. La durée de vie est celle de la
  table du jeu, comme pour un drop réel.
- `powerup_drop_chance_override: Fixed` : force `PowerUpsConfig::drop_chance` pour ce
  scénario (`Update`, comme `apply_wave_overrides`) — pour prouver le chemin « drop à la
  mort » sans dépendre du tirage réel du jeu (scénario `powerup_drop_on_kill`, chance forcée
  à 1). Les scénarios de régression et les essais générés d'armes déclarent explicitement
  une chance nulle pour isoler leur comportement. Dans ce cas aucun tirage ni flux
  `loot` n'est créé. La table du jeu conserve sa chance de 15 %.

**Nouvelle attente** (`game::replay::Expectation::PowerUpPickups { min, max, at_frame }`) :
nombre de `PowerUpPickup` au sol, bornes `[min, max]` inclusives — même forme que
`WeaponPickups`. Les placements et la chance de drop sont préservés dans le scénario
réenregistré par le runner.

**Scénarios de référence** (`tests/scenarios/`, testbed, un par power-up + un pour le drop) :
`powerup_insta_kill`, `powerup_double_points`, `powerup_max_ammo`, `powerup_carpenter`
(seul à utiliser `testbed/window.ldtk` plutôt que `testbed/arena.ldtk` : il faut une fenêtre
à réparer, absente de l'arène — le `breacher` de `window.ldtk` la casse naturellement avant
que le scénario ne pose le power-up), `powerup_nuke`, `powerup_drop_on_kill` (chance forcée
à 1, tue `dummy` par tir soutenu, vérifie qu'un `PowerUpPickup` apparaît).

## 15. HUD : icônes, noms et sources (T2.12)

Le HUD est décrit par `games/<jeu>/assets/ui/hud.ron` (`crates/game/src/ui/hud.rs`, liste des
sources et des ancrages dans `CLAUDE.md` § HUD). Il ne fait que **lire** l'état à chaque
`Update` : aucun système dans `GgrsSchedule`, aucun état rollback, aucune trace touchée.

Champs optionnels à la racine de `hud.ron`, à côté de `widgets` : `font`, `icons`, `names`.
Par widget : `background`.
```ron
(
    // Police de tous les textes du HUD, relative à `assets/`. La police par défaut de Bevy
    // n'a ni accents (« À TERRE », « possédé ») ni tiret long : les deux jeux déclarent celle-ci.
    font: Some("fonts/FiraMono-Medium.ttf"),
    widgets: [
        (kind: Icons(source: "perks"), anchor: BottomLeft, offset: (16.0, 80.0),
         size: Some((200.0, 24.0)), color: None, font_size: None),
        (kind: Text(source: "prompt", prefix: Some("[H] ")), anchor: BottomCenter,
         offset: (0.0, 110.0), size: Some((480.0, 24.0)), color: Some("#f1c40f"), font_size: Some(18.0),
         // Fond derrière le texte (`#rrggbb` ou `#rrggbbaa`), masqué quand le texte est vide ;
         // avec un fond, la hauteur suit le nombre de lignes.
         background: Some("#000000a0")),
        // ...
    ],
    // Icône par id de perk (`economy/perks.ron`) : pas de sprite, un carré de couleur
    // (hex `#rrggbb`) et une étiquette courte (l'initiale).
    icons: {
        "juggernog": (color: "#c0392b", label: "J"),
    },
    // Nom affiché par id d'arme (`weapons.ron`), utilisé par la source `prompt`.
    names: {
        "shotgun": "fusil à pompe",
    },
)
```
- **Icône manquante** : un perk absent de `icons` est affiché en carré gris avec la première
  lettre de son id en majuscule (pas d'erreur).
- **Nom manquant** : une arme absente de `names` est affichée par son id.
- `Icons(source)` : `size.1` est le côté d'un carré ; la rangée s'étend vers la droite depuis
  l'ancre, quel que soit le nombre de perks. `Text(source: "perks")` affiche les étiquettes
  séparées par des espaces.
- Les noms des perks et des power-ups viennent de leur propre contenu (`name` de `perks.ron`
  et de `items/powerups.ron`), pas de `hud.ron`.
- Prix « en icône » : préfixe `$` devant le montant (`Ouvrir — $750`).
- Valeur absente (D22) : les sources du joueur (`health`, `ammo`, `weapon`, `currency`)
  n'affichent rien quand il n'existe plus (mort) ou n'a pas d'arme, pas même leur préfixe :
  plus de « $ » ni de « ? | ? » sur le HUD d'un joueur mort.
- Placement : les textes de debug (`Wave 1 | PREP …` en bas à gauche, caméra en haut à
  gauche) occupent le bas de l'écran jusqu'à ~75 px : les perks (80) et le prompt (110) sont
  placés au-dessus. L'écran de fin utilise la même police (`ui/game_over.rs`).

Captures de référence : `docs/captures/hud-v1/` (prompt d'achat, perk et power-up actifs, à
terre), produites par `play_scenario --capture` (voir `CLAUDE.md` § Vidéos).

## 16. Projectiles composables (T1.1, chantier B5 v1)

Code : `crates/combat/src/projectile.rs` (données, mathématiques pures testées, systèmes),
branché par `combat::weapons::BaseWeaponGamePlugin`. Les contrats de T1.0a
(`ProjectileModifier`, `Pattern`, §4.3) sont exécutés ; aucun renommage, un ajout :
`ExpireAction::Spawn(Pattern)`.

**Contenu** : un mode de tir déclare `projectile:` (absent = balle ordinaire, exactement
comme avant, sans composant `Projectile`) ; l'arme déclare la table `projectiles` des
projectiles que ses patterns font naître (une table **par arme** en v1, résolue au tir et
portée par la balle ; T1.2 pourra la remonter au niveau du jeu pour les ennemis).
```ron
"default": (
    // ... champs habituels du mode (bullet_type, range... : le projectile tiré)
    projectile: (
        modifiers: [Bounce(2), Pierce(8), Lifetime(70)],
        on_hit: [TimedModifier(stat: MoveSpeed, op: Mul, value: "0.5", frames: 60)],
        on_expire: [Spawn(Aimed(count: 1, spread: "0.0", projectile: "explosion"))],
    ),
),
// au niveau de `config`, à côté de `firing_modes` :
projectiles: {
    "explosion": (damage: "20.0", speed: "0.0", range: "1.0",
                  modifiers: [Lifetime(0), Size("10.0"), Pierce(16)],
                  on_expire: [Spawn(Ring(count: 8, speed: "240.0", projectile: "eclat", every: 0))]),
    "eclat": (damage: "4.0", speed: "240.0", range: "96.0"),
},
```

**Modificateurs** (décisions de gameplay ; un même modificateur répété : le dernier
l'emporte, le lint le refuse) :

| Modificateur | Effet |
|---|---|
| `Bounce(n)` | n rebonds sur les murs (`Wall` par `layer_matrix`) ; l'axe du déplacement qui entre dans le mur s'inverse (les deux dans un coin) et le projectile revient à sa position d'avant le déplacement. `n = 0` : fin au premier mur. |
| `Pierce(n)` | traverse n personnages, se termine sur le (n+1)-ième ; **jamais deux fois le même** (liste `hits`). Tous les personnages en contact dans une frame sont touchés, par `GgrsNetId`, tant que `Pierce` le permet. |
| `Size(f)` | rayon de collision × f (base 5, celui de `Standard`) et sprite × f. `f > 0`. |
| `Lifetime(n)` | n frames de vie après la frame de tir, fin à la suivante : `Lifetime(0)` = une seule frame de collisions. La portée (`range`) termine aussi le projectile. |
| `Homing(force)` | à chaque frame, la direction tourne vers le personnage le plus proche que l'équipe et le tir ami permettent de toucher (hors `Neutral`, hors cibles déjà traversées, égalité par `GgrsNetId`) : `v̂ + force·t̂` renormalisé, vitesse conservée. `0 < force <= 1`. Pas de portée de détection en v1. |
| `Gravity(g)` | accélération verticale constante de g unités/s² (positive vers le haut du monde) : `v.y += g/3600` par frame. |

**Ordre dans une frame** : `Weapon` (tir ; déplacement de toutes les balles, la portée
atteinte *termine* un projectile composable au lieu de le détruire ; collisions des balles
ordinaires, inchangées) → `Projectiles` (`projectile_collision_system` : personnages puis
murs ; `apply_projectile_on_hit_system` ; `projectile_expire_system` : `Lifetime`,
`on_expire`, `despawn_rollback` ; `projectile_steering_system` : `Gravity` puis `Homing`,
appliqués au déplacement suivant) → `CollisionDamage` (les `DamageEvent` des projectiles).
`apply_projectile_on_hit_system` est dans `Projectiles` et non `Effects` : `Effects` contient
déjà `game::powerups::apply_powerup_actions_system`, qui écrit aussi `Modifiers` (ordre
ambigu refusé par `GgrsSchedule`).

**`on_hit: [Action]`** : actions de `effects::Action` posées sur **le personnage touché**
(s'il a des `Modifiers`), seulement les actions à modificateur (`TimedModifier`,
`CurrencyMultiplier`) ; les autres (`RefillAmmo`, `RepairAllWindows`, `KillAllWaveEnemies`)
sont des effets globaux de power-up, refusés par le lint. Source du modificateur :
`projectile:<id>:<rang>` ; un nouveau coup du même projectile **rafraîchit** le
modificateur au lieu de l'empiler.

**`on_expire: [Spawn(pattern)]`** : joué à **toute** fin du projectile (durée de vie,
portée, mur sans rebond, perforation épuisée) au point de fin. Patterns instantanés
seulement, sans aléa : `Aimed` (éventail de `count` sur `spread` radians, centré sur le
personnage le plus proche, sinon sur la direction du projectile), `Spread` (même éventail,
centré sur la direction du projectile), `Ring` (`count` directions régulières à partir de
la direction du projectile — axe +x à l'arrêt —, vitesse du pattern, `every` ignoré : une
seule salve), `Sequence` de ceux-ci. `Telegraph`/`Wait` sont réservés aux émetteurs (T1.2) et
refusés par le lint. Les projectiles nés héritent du tireur (source, équipe, tags, tir
ami, multiplicateur de dégâts `Damage` du tir) ; garde-fou de 8 générations (le lint refuse
les cycles). Une **explosion** est un projectile de la table à vitesse nulle,
`Lifetime(0)`, grand `Size` et `Pierce` élevé.

**État rollback** : composant `combat::projectile::Projectile` (compteurs restants, cibles
déjà touchées, `ended`, génération, actions, table partagée par `Arc` ; Debug compact : la
table n'apparaît que par ses clés) et ressource `FrameEvents<ProjectileHit>`, enregistrés
par `RollbackTraceApp`. `FiringModeConfig`/`WeaponConfig` ont un `Hash` et un `Debug`
manuels qui ignorent `projectile`/`projectiles` vides : le contenu existant garde son hash
et sa ligne de trace détaillée (preuve `trace-diff --ignore
Projectile,FrameEvents<combat::projectile::ProjectileHit>`, §10).

**Attentes** (`crates/scenario`) : `BulletCount(count, projectile?, team?, at_frame)` —
nombre **exact** de balles vivantes, filtrées par id de projectile composable (l'arme qui a
tiré, ou l'entrée de la table pour un projectile né) et par équipe du tireur ;
`HitsAtLeast(entity, hits, at_frame)` — `HitCount` de `entity` ≥ `hits`, `entity` :
`NetId(n)` ou `Target` (l'entité qui compte ses coups de plus petit `GgrsNetId`, `target` du
testbed : utilisable dans le `test.expect` d'une arme, où le `GgrsNetId` dépend de l'arme).

**Lint** (`content::lint::lint_weapon_projectiles`) : modificateur répété, `Size <= 0`,
`Homing` hors `]0, 1]`, action `on_hit` sans modificateur, pattern temporel, `count = 0`,
`spread`/`speed` négatifs, projectile absent de la table (`BrokenReference`), définition
avec `damage`/`speed` négatif ou `range <= 0`, cycle de `on_expire`. Fixtures
`projectile_broken_reference`, `projectile_temporal_pattern`, `projectile_cycle`.

**Testbed** : une arme par modificateur (`proj_bounce`, `proj_pierce`, `proj_size`,
`proj_lifetime`, `proj_homing`, `proj_gravity`) et `grenade` (Bounce, Pierce, explosion,
éclats) dans `games/testbed/assets/ZombieShooter/Sprites/Character/weapons.ron`, chacune
avec `test:` et son scénario généré (`make gen GAME=testbed`,
`tests/scenarios/generated/testbed/`).


Patterns joués **dans le temps** (`Telegraph`, `Wait`, `Ring.every`, `Scatter`, `Named`),
émetteurs et tir ennemi : voir §20 (T1.2).
## 17. Mode `Floors` (T1.8, chantier F1)

**Contenu.** Kind de dossier `Floors` (`content::registry::FloorsEntry`) : un fichier RON par
séquence de niveaux, id = nom de fichier sans extension.
```ron
// games/testbed/assets/floors/deux_niveaux.ron
(levels: ["testbed/floor_a.ldtk", "testbed/floor_b.ldtk"])
```
Les chemins sont relatifs à `assets/` (comme `entry.start_map`) ; chaque niveau est une carte
LDtk ordinaire (générée par gabarits comme les autres, même graine pour tous les niveaux).
Mode : `entry: (..., mode: Floors)` joue la première séquence (ordre des ids) ; un scénario
impose une séquence par son champ `floors: Some("deux_niveaux")` (`map` est alors ignorée), et
`alacod-sim` par `--floors <id>` (`game::run_state::FloorsOverride`, hors rollback). `RunMode::
Floors { config }` porte l'id de la séquence.

**Lint** (`content::lint::lint_floors`, `lint_entry_point`) :

| Règle | Kind | Fixture |
|---|---|---|
| `levels` non vide | `OutOfRange` | `floors_empty` |
| chaque niveau désigne une carte chargée (kind `Map`) | `BrokenReference` | `floors_unknown_map` |
| `entry.mode: Floors` exige un dossier `Floors` | `BrokenReference` | `entry_mode_floors_without_floors` |

**Règles du mode.**
- **Portail** : il s'ouvre quand le niveau courant n'a plus aucune entité `Enemy`
  (`EntityCount(enemy) == 0` : alliés et civils du testbed compris — ne pas en placer dans un
  niveau `Floors`). Position : barycentre des `PlayerSpawn` du niveau (centre de la salle de
  départ), sans entité LDtk dédiée. Un joueur debout (ni à terre ni mort) à moins de 24 unités
  (`run::floors::PORTAL_RADIUS`) le franchit.
- **Passage** : à la frame du franchissement, toute entité rollback qui n'appartient pas à un
  joueur (ennemis, murs, portes, fenêtres, spawners, balles, objets au sol, power-ups...) est
  détruite (`despawn_rollback`), le flow field repart de zéro, les entités du niveau suivant sont
  créées et les joueurs placés sur ses `PlayerSpawn` (même handle ; repli sur le plus petit
  index). Les joueurs gardent santé, armes (enfants de l'entité joueur), munitions, monnaie,
  perks, modificateurs ; un joueur à terre le reste.
- **Numérotation** : `GgrsNetIdFactory` n'est jamais remise à zéro pendant la partie ; le
  nouveau niveau est numéroté dans le même ordre qu'au chargement (entités de carte du
  registre triées par nom puis position, murs par iid de niveau, personnages, armes murales et
  machines à perk triés par position) : les ids continuent ceux du niveau quitté.
- **Fin de séquence : boucle infinie au dernier niveau** (décision T1.8). Après le dernier
  niveau de la liste, chaque portail recharge le dernier niveau (`run::floors::level_for_floor`)
  et l'index continue de croître. Pas de victoire en `Floors` : la partie ne se termine que par
  la défaite (universelle, mêmes chemins que `Waves`) ou l'abandon.
- **Résumé** (`RunSummary`, mêmes chemins que `Waves`) : `floor_reached` = index atteint,
  `kills` = ennemis placés par les niveaux chargés moins ceux encore en vie, `wave_reached` = 0.
  L'écran de fin affiche « niveau N » (N = index + 1).

**État rollback.** `run::FloorState` (ressource : `index`, `anchor` — position du portail —,
`portal_open`, `enemies_placed`), enregistrée par `rollback_and_trace_resource_neutral` : sa
valeur par défaut (tout autre mode) contribue `0` au checksum GGRS, les traces des autres modes
ne changent pas ; `RunSummary::floor_reached` n'entre dans son hash que s'il est non nul (même
raison). Attente de scénario `FloorIndex(index, at_frame)` (ponctuelle) et `RunSummary(...,
floor_reached_min)`.

**Chargement : un monde LDtk par niveau.** À l'entrée de `GameLoading`,
`map_ldtk::game::floors::compute_floor_plan` pose `FloorPlan` (hors rollback) si la partie est
en `Floors` ; `loader::setup_generated_map` charge alors toutes les cartes distinctes de la
séquence, un monde chacun (`FloorWorld(emplacement)`, une carte répétée réutilise son monde),
superposés à l'origine ; seul le monde du niveau courant est visible (présentation). Le
chargement attend que tous les mondes aient leurs niveaux, puis ne crée les entités rollback
que du premier niveau. Le passage de niveau (`floor_transition_system`, `GgrsSchedule`,
`RollbackSystemSet::Run`) lit ces mondes déjà chargés, immuables et identiques sur tous les
clients : aucune attente de chargement pendant la partie, et un rollback qui remonte avant le
passage ressuscite l'ancien niveau. Hors `Floors`, la carte unique est le monde `FloorWorld(0)`
et le chargement est inchangé.

**Bots.** `BotView::portal` : un bot `fonceur` ou `prudent` sans ennemi visible marche vers le
portail ouvert (ligne droite, sans pathfinding : les niveaux du testbed sont des salles
ouvertes).

**Limites connues.** Les `RoomBounds` des niveaux non courants existent aussi (mondes
superposés) : les spawners de vagues (`ZombieSpawn`) ne sont pas pris en charge en `Floors`.
Restart p2p : non supporté (comme `Waves`). Pas d'arrêt `--until-floor` dans `alacod-sim`
(T1.14).

## 18. Équilibrage par joueurs (F5)

**Décision (D25, le 2026-10-03) : F5 appartient à M0** — le plan §6 liste F5 dans le jalon M0
(`zombies` complet). Tâche dédiée : `docs/taches/m0-v11-f5-equilibrage-joueurs.md`.

Les champs numériques des **vagues** (`games/zombies/assets/waves/wave_config.ron`), des **prix**
(armes `assets/weapons/`, perks `assets/economy/perks.ron`, `assets/economy/economy.ron`) et de la
**santé** des ennemis (`ZombieShooter/Sprites/Zombie/*_config.ron`) acceptent un littéral **ou** une
expression `content::expr` en chaîne RON (ex. `health: "120.0 + (players - 1) * 30"`). Règles :

- **Contexte** : l'identifiant `players` (nombre de joueurs de la run) est disponible partout où
  une expression est acceptée. Pas d'autres identifiants pour l'instant (voir F5 du plan).
- **Timing** : l'expression est évaluée **une seule fois, au démarrage de la run** (le nombre de
  joueurs de la session est connu : local `NUMBER_PLAYER`, p2p `max_player`, scénario = ses
  joueurs), en valeurs concrètes. **Jamais d'`Expr` dans l'état rollback** : seules les valeurs
  évaluées entrent dans la simulation (déterminisme : mêmes configs + même nombre de joueurs
  partout ⇒ mêmes valeurs).
- **Erreurs** : une expression invalide (parse, division par zéro, identifiant inconnu) fait
  échouer le chargement — jamais de valeur par défaut silencieuse.
- **Traces** : les littéraux actuels restent valides et gardent leurs valeurs ; les 62 scénarios
  existants gardent leurs traces, sauf bless avec preuve (§10). La preuve de F5 passe par de
  nouveaux scénarios (même partie à 2 et 4 joueurs, attente qui diverge) plutôt que par la
  modification de valeurs existantes.

**Implémentation (m0-v11)** :

- **Type** : `content::expr::NumOrExpr` — `Integer(u32)` (entier RON nu, ex. `base_enemies: 6`),
  `Literal(Fixed)` (chaîne numérique, ex. `refill_price_ratio: "0.5"`), `Expression(NumExpr)`
  (chaîne RON évaluée, ex. `max: "10.0 + (players - 1) * 40.0"`). La désérialisation tente dans
  cet ordre : entier nu → `Integer` ; chaîne qui est un nombre → `Literal` ; sinon parse
  `Expr` → `Expression` (un échec de parse est un échec du chargement).
- **Point d'évaluation** : `game::balance::resolve_balance_system`, enregistré sur
  `OnEnter(AppState::GameLoading)` — après le chargement des assets, avant le spawn des
  joueurs/personnages (Update de `GameLoading`) et avant la première vague. Il insère la
  ressource ordinaire (hors rollback, comme `Assets`) `game::balance::ResolvedBalance` :
  `waves`, `economy`, `perks` et `health_max_by_character`, en valeurs concrètes. Les systèmes
  de simulation lisent `ResolvedBalance`, jamais les assets d'origine — aucune expression dans
  l'état rollback. Une relance locale re-entre `GameLoading` et re-résout avec le même nombre
  de joueurs.
- **Nombre de joueurs** : `OnlineState::Online` → `ggrs_config.connection.max_player` (source
  autoritaire en ligne) ; sinon `PlayersCount` (partie locale ou scénario).
- **Champs couverts** (rien d'autre — le reste des chantiers F est pour M1/M2) :
  - **Vagues** (`games/zombies/assets/waves/wave_config.ron`) : `base_enemies`,
    `enemies_per_wave`, `max_random_variance`, `min_wave_delay_frames`, `grace_period_frames`,
    `max_concurrent_enemies`, `spawn_batch_size`, `spawn_interval_frames`,
    `min_player_distance`, `max_player_distance`, `health_multiplier_per_wave`,
    `damage_multiplier_per_wave`, `max_wave`, et par palier `max_wave` + poids des ennemis.
  - **Prix** : `games/zombies/assets/economy/economy.ron` (`kill_points`, `hit_points`,
    `repair_points`, `nuke_points`, `repair_points_cap_per_wave`, `refill_price_ratio`) et
    `games/zombies/assets/economy/perks.ron` (`price` de chaque perk). Les prix d'armes
    murales et de portes sont des champs **Int posés dans l'éditeur LDtk** (`WeaponLocation`,
    portes), pas du RON — ils n'acceptent pas d'expression (et `weapons.ron`/
    `melee_weapons.ron` ne portent aucun prix aujourd'hui) ; la recharge d'une arme murale
    reste `prix_achat × refill_price_ratio` (ratio résolu).
  - **Santé** : `base_health.max` de chaque personnage du registre (`CharacterConfig`) —
    ennemis comme « player », les deux chemins de spawn lisent
    `ResolvedBalance::health_max_by_character`.
- **Erreurs** : `content::expr::EvalError` — identifiant inconnu (seul `players` existe),
  division par zéro, valeur hors domaine (négatif pour un `u32` : erreur, jamais de clamp).
  `resolve_balance_system` panique en nommant le fichier, le champ et le nombre de joueurs.
  Tests unitaires : `content::expr` (désérialisation, résolution, erreurs) et `game::balance`
  (dépendance à `players`, erreurs = panic avec contexte).

## 20. Patterns, émetteurs et tir ennemi (T1.2, chantier B5 v1)

Code : `crates/combat/src/emitter.rs` (état `Emitter`, avancée pure testée, `emitter_system`),
`crates/combat/src/projectile.rs` (`Pattern`, `PatternLibrary`), `crates/combat/src/weapons/mod.rs`
(`spawn_bullet`, la **seule** fonction d'apparition d'une balle : tir de joueur, projectile né
d'un `on_expire`, tir d'émetteur), `crates/game/src/character/enemy/ai/behavior.rs`
(`ranged_attack`), `crates/game/src/patterns.rs` (bibliothèque construite depuis le registre).

**Patterns nommés** : kind de contenu `Pattern` (`(path: "patterns", kind: "Pattern")` dans
`game.ron`), un fichier `patterns/<nom>.ron` par pattern, id = nom de fichier. Contenu : un
`Pattern` RON, ex. `Ring(count: 8, speed: "90.0", projectile: "fireball", every: 60)`.
Référencés par la règle `Shoot(pattern: ...)` d'un personnage (§22) et par `Named("nom")` dans un pattern
(`on_expire` compris). La bibliothèque (`combat::projectile::PatternLibrary`) est une ressource
**hors rollback**, reconstruite à `OnEnter(GameLoading)` depuis le registre ; un nom inconnu
est une erreur de lint, jamais une panique en jeu (en jeu : avertissement, rien n'est tiré).

**Variantes ajoutées** (en fin d'enum : le `derive(Hash)` hache l'index de variante, le hash
des armes et projectiles existants ne bouge pas) : `Scatter(count, spread, projectile)` et
`Named("nom")`.

**Sémantique dans un émetteur** (le pattern, résolu, est aplati : les `Sequence` imbriquées
se déplient dans l'ordre RON) :

| Étape | Effet |
|---|---|
| `Aimed`, `Spread`, `Ring` (`every: 0`) | une salve, comme §16 ; durée 0 frame (l'étape suivante joue la même frame) |
| `Ring(every: n > 0)` | une salve tout de suite, puis une toutes les `n` frames **en tâche de fond**, jusqu'à la fin de la `Sequence` qui la contient ; un `Ring` **seul** est infini |
| `Scatter` | `count` tirs à des angles tirés dans `±spread/2` autour de la visée, flux RNG `"patterns"` |
| `Telegraph(n)` | `n` frames sans tir ; `Emitter::telegraphing() -> Option<frames restantes>` pour la présentation (dessin : T1.17) |
| `Wait(n)` | `n` frames sans tir |

La visée est **figée au départ** de la séquence (direction de la cible à la pose) : direction
de `Aimed`/`Scatter`, centre de `Spread`, premier rayon de `Ring`. L'émetteur est posé à la
frame `f` ; sa première étape joue à `f + 1` ; une étape après `Telegraph(n)` joue `n` frames
après l'entrée dans le télégraphe. Dans une frame, les étapes de la séquence passent avant
les couronnes de fond ; une couronne de fond ne tire plus la frame où la séquence finit
(`Sequence([Ring(every: 20), Wait(60)])` : trois salves). À la fin de la séquence,
`emitter_system` retire le composant.

**Projectiles tirés** : entrée `projectile` de la table `projectiles` de l'**arme de
l'émetteur** (§16), vitesse du `Ring` sinon celle de la définition ; nés au centre du tireur,
génération 0, équipe et tags du tireur (plus `bullet`), tir ami de l'arme, multiplicateur
`Damage` du tireur au départ. Ils suivent la même chaîne que les autres (`Weapon` puis
`Projectiles` : collisions par règles d'équipe §8, `on_hit`, `on_expire`, 8 générations).
`Bullet::player_handle` vaut `weapons::NO_PLAYER_HANDLE` (aucun joueur) ; il n'apparaît dans
aucun log, qui nomme le tireur par son `GgrsNetId` (`ggrs{f=… emitter net_id=… step=…}`).

**Déterminisme** : `emitter_system` (set `Weapon`, après tout tir de joueur et de mêlée)
avance les émetteurs par `GgrsNetId` ; le flux `"patterns"` n'est créé et consommé que par un
`Scatter` qui tire, et par rien d'autre : la suite des tirs d'un émetteur ne dépend que de la
graine de run et de l'ordre des émetteurs, **pas du nombre de joueurs** (test unitaire
`meme_graine_meme_tir_a_un_et_quatre_joueurs`, scénarios `enemy_ring`/`enemy_ring_quad`).

**Tir ennemi** : règle `Shoot` de `ai.behaviors` (T1.4, §22 ; remplace le champ `ai.ranged` de T1.2) :
```ron
behaviors: Some([Shoot(weapon: "fireball_gun", pattern: "ring_8", range: "260.0", cooldown_frames: 90)]),
```
`spawn_enemy` équipe `weapon` (active, dans `WeaponInventory` ; la griffe de corps à corps
reste posée selon `attack_range`) et pose `RangedAttackState`. `enemy_attack_system` joue le
tir **avant** le corps à corps (`ranged_attack`) :
- cible `Player` vivante, debout, à moins de `range`, refroidissement écoulé, état
  `Idle`/`Chasing` → pose un `Emitter` (visée = position de la cible),
  `MonsterState::Attacking { target: Player }` ;
- l'ennemi **ne bouge pas** tant que l'émetteur est posé (`move_enemies`, télégraphe compris) ;
- séquence interrompue si la cible meurt, passe à terre ou sort de `range` (l'émetteur est
  retiré ; le tireur mort emporte le sien) ; fin ou interruption → refroidissement de
  `cooldown_frames`, retour à `Chasing` ;
- un `Attacking` posé par un tir n'émet jamais le coup direct de corps à corps
  (`enemy_attack_damage_translate_system`).
Sans `ranged`, rien ne change (hash de `EnemyAiConfig` : `ranged` n'est haché que s'il est
présent, hacher un `None` déplacerait le checksum de tous les ennemis).

**État rollback et checksum neutre** : `Emitter` (combat) et `RangedAttackState` (game) sont
enregistrés par `RollbackTraceApp::rollback_and_trace_neutral` : un composant que **aucune**
entité ne porte contribue `0` au checksum GGRS. Raison, vérifiée sur `idle` : bevy_ggrs 0.22
fait contribuer à un type enregistré **sans porteur** une part constante `K = hash(0u64)`,
**la même pour tous les types vides**, combinée par XOR — un type vide de plus déplace toutes
les traces dès la ligne 1, deux de plus s'annulent (trace identique). **Piège pour toute
voie** : un nouveau composant rollback absent du contenu existant passe par la variante
neutre, jamais par `rollback_and_trace` en comptant sur la parité.

**Lint** : `Scatter`/`Telegraph`/`Wait` refusés en `on_expire` (même au travers d'un `Named`) ;
pattern nommé inconnu (`BrokenReference`) ; cycle de `Named` ; `count = 0`, `spread`/`speed`
négatifs dans un pattern nommé ; règle `Shoot` : arme inconnue, pattern inconnu, projectile du
pattern absent de la table de l'arme (`BrokenReference`), `cooldown_frames = 0`,
`range <= 0`. Fixtures `pattern_unknown_name`, `pattern_scatter_on_expire`,
`ranged_projectile_missing`, `ranged_cooldown_zero`.

**Testbed** : arme `fireball_gun` (table `fireball`, `arrow`), patterns `ring_8`
(`Ring(count: 8, every: 60)`) et `volee` (`Sequence([Telegraph(30), Aimed(count: 3),
Wait(20), Scatter(count: 4)])`), personnages `turret` (immobile) et `archer` (poursuit),
carte `testbed/arena_tir.ldtk` (copie d'`arena`, tourelle au centre, archer à l'ouest ; le
joueur 0 est le plus proche de la tourelle, les joueurs 1 à 3 sont plus loin et hors des
rayons de la couronne). Scénarios `enemy_ring`, `enemy_ring_quad`, `enemy_telegraph`.

## 22. Behaviors composables (T1.4, chantier D1)

Code : `crates/behaviors` (vocabulaire `Behavior`, `Perception`, `Targeting` ; sélection pure
`select`/`applicable`, faits `SelectionContext`), `crates/game/src/character/enemy/ai/rules.rs`
(`behavior_select_system`, `behavior_motion`, `charge_damage_translate_system`,
`current_rule`), `state.rs` (`EnemyBehaviors`, `BehaviorRuntime`, `default_behaviors`).

**Contenu** : champs optionnels de `ai` (`EnemyAiConfigRon`) :
```ron
ai: Some((
    // ... champs habituels (aggro_range, attack_range, flee_threshold...)
    behaviors: Some([
        Shoot(weapon: "fireball_gun", pattern: "volee", range: "240.0", cooldown_frames: 90),
        KeepDistance(min: "120.0", max: "200.0"),
        Chase(profile: "Ground"),
    ]),
    perception: Some((senses: [Sight("80.0"), Hearing("300.0")])),
    targeting: Some(Nearest(ignore: ["ghost"])),
)),
```

**Sélection par priorité** : la liste est ordonnée par priorité (ordre RON) ; à chaque frame,
dans `RollbackSystemSet::EnemyAI`, la **première règle applicable** gagne (pas de `when:` en
v1). Applicabilité implicite :

| Règle | Applicable quand | Exécution |
|---|---|---|
| `Melee(arme)` | une cible (joueur, ou obstacle cassable sur la route si `can_break`) est à moins de `attack_range` | l'attaque de mêlée d'origine ; l'arme (`melee_weapons.ron`) est équipée au spawn — `zombie_claws` n'est plus codé en dur |
| `Shoot { weapon, pattern, range, cooldown_frames }` | séquence de tir en cours, ou cible vivante et debout à moins de `range`, refroidissement écoulé | `ranged_attack` (§20) ; se compile vers `EnemyAiConfig::ranged` |
| `Charge { telegraph }` | charge en cours, ou cible entre `attack_range` et 3 × `attack_range`, refroidissement écoulé | `telegraph` frames immobile, ruée en ligne droite vers la position **figée** de la cible à 3 × la vitesse jusqu'au contact (`attack_range`) ou 60 frames, murs respectés ; dégât de contact `attack_damage` (émis la frame suivante, `CollisionDamage`, tags `melee`+`charge`) ; refroidissement `attack_cooldown_frames` |
| `KeepDistance { min, max }` | cible à moins de `min`, ou déjà retenue et cible à moins de `max` (hystérésis) | recule : case voisine du champ de flux de **coût le plus élevé** (départage par `GridPos`), repli à l'opposé de la cible |
| `Flee` | santé ≤ `flee_threshold` × max | recule comme `KeepDistance` |
| `Strafe` | cible à moins de `Sight` | perpendiculaire à la cible, sens alterné toutes les 45 frames depuis l'entrée dans la règle |
| `Chase { profile }` | cible connue | `move_enemies` d'origine (v1 : seul le champ `GroundBreaker` est construit, tous les profils l'utilisent) |
| `Wander` | toujours (règle de fond) | direction tirée dans le flux RNG `"behaviors"` toutes les 60 frames (consommé seulement par `Wander`, ordre `GgrsNetId`), demi-vitesse |

Une règle absente n'existe pas pour ce personnage. Les déplacements imposés (`KeepDistance`,
`Flee`, `Strafe`, `Wander`, `Charge`) n'ont pas le ralentissement près du joueur de `Chase` et
passent par la même résolution de collision (glissement sur les murs).

**Liste par défaut** (personnage sans `behaviors:`, ex. testbed T2.9) : `[Flee]` si
`flee_threshold`, puis `Shoot` si `ranged`, puis `Melee("zombie_claws")` si
`attack_range > 0`, puis `Chase(profile: <movement_type>)` sauf si `stationary` — exactement le
comportement d'avant T1.4. Les zombies de `games/zombies` déclarent leur `ai` en entier
(valeurs du préréglage `zombie()`) et `behaviors: [Melee("zombie_claws"),
Chase(profile: "GroundBreaker")]` : « les zombies deviennent un fichier », traces identiques.

**Perception et ciblage** : `Sight(r)` = rayon seul, sans ligne de vue ni lumière ; se compile
vers `aggro_range` (repli). `Hearing(r)` : un tir de joueur né à moins de `r` rend le tireur
connu 120 frames, même hors de vue. `needs_light` ignoré (E8). `Targeting::Nearest { ignore }`
(repli `ignore: []`) : l'algorithme d'origine (plus proche par le champ de flux, repli en ligne
droite, joueurs à terre ignorés) ; un joueur portant un tag d'`ignore` n'est jamais ciblé.

**État et traces** (critère central : aucune trace existante ne change) :
- les règles résolues sont un composant **statique, hors rollback** (`EnemyBehaviors`, comme
  `Team`) : un ennemi existant ne gagne ni ne perd aucun composant rollback ;
- un ennemi qui ne liste que `Melee`/`Shoot`/`Chase` n'a **aucun état nouveau** : sa règle
  retenue est dérivée de son état (`current_rule` : séquence de tir → `Shoot`, `Attacking` →
  `Melee`, cible connue → `Chase`) ;
- les behaviors nouveaux gardent leur état dans `BehaviorRuntime` (règle retenue, frame
  d'entrée, phase de charge, errance, ouïe), **checksum neutre** (§20), posé seulement sur les
  ennemis qui en listent un ;
- `behaviors::BehaviorState` (contrat T1.0a) reste enregistré **tel quel** sous checksum
  ordinaire et n'est posé sur personne : le retirer, le passer en neutre ou le poser
  déplacerait toutes les traces par parité (§20, piège).
Code mort retiré : `enemy_movement_system`, `enemy_stun_recovery_system`, `apply_stun`,
`MonsterState::{Stunned, Breaching, Fleeing}` (jamais posés ; l'étourdissement est un statut,
T1.3). `enemy_target_selection` et `update_enemy_targets` ne font pas doublon (cible d'IA d'un
côté, point de chemin de `EnemyPath` de l'autre) : gardés. Dégât direct `attack_damage`
préservé tel quel (dette D28).

**Attentes** (`crates/scenario`) : `EnemyState(entity, behavior: "Chase", at_frame)` — règle
retenue (nom de variante) ; `EnemyDistance(entity, target: Player(h), min?, max?, at_frame)` ;
continues : `EnemyContactBefore(entity, frames)` (portée de mêlée d'un joueur atteinte au plus
tard à `frames`) et `EnemyNeverInWall(entity, from, to)` (collider sans chevauchement de `Wall`).
`entity` : `NetId(n)` ou `Target` (comme `HitsAtLeast`).

**Lint** : liste vide ; `Shoot` (règles `ranged` de T1.2) ; `Melee` arme inconnue ; `Chase`
profil inconnu (`Ground`, `Flying`, `Phasing`, `GroundBreaker`) ; `KeepDistance` `min >= max` ;
`Charge` télégraphe 0 ; tag d'`ignore` porté par aucun personnage. Fixtures
`behavior_melee_unknown`, `behavior_keep_distance_inverted`, `behavior_charge_zero`,
`behavior_unknown_profile`, `targeting_unknown_tag` (et `ranged_*` migrées vers `Shoot`).

**Testbed** : `kiter`, `charger`, `coward`, `drifter`, une carte chacun
(`testbed/arena_ia_keep|charge|flee|wander.ldtk` : un scénario ne peut pas faire apparaître de
personnage, une arène commune les ferait interagir), scénarios `enemy_keep_distance`,
`enemy_charge`, `enemy_flee`, `enemy_wander`.

## 24. Bots de validation (m0-v7 phase 2)

Les profils RON `chasseur` et `acheteur` complètent `immobile`, `fonceur` et `prudent`.
Les profils v0 gardent leurs décisions. `alacod-sim --bots 4 --seeds 1..20 --until-wave 5`
utilise quatre `acheteur` par défaut ; `--profiles` permet les deux générations.
Pour comparer avec le digest M0, préciser `--map exemples/test_map.ldtk` : sans cet
argument, le manifeste choisit maintenant `maps/avant_poste.ldtk`.

`crates/bots/src/navigation.rs` réutilise `FlowField` et `GridPos` pour un Dijkstra
multi-source vers des postes de tir autour des ennemis, sur une grille physique de 8 px.
Le corps et son offset sont pris en compte ; murs, portes fermées et fenêtres, même cassées,
bloquent le mouvement du joueur. Les fenêtres permettent le tir. Les cibles sont triées par
`GgrsNetId`, et les diagonales ne coupent pas les coins. Le cache se reconstruit quand la
position en grille d'un ennemi, les colliders ou le corps changent. Le calcul se limite
à la composante physique accessible au joueur ; les champs d'approche restent en cache
tant que leurs clés et cette composante sont identiques. Il est dérivé dans
`ReadInputs`, hors simulation rollback, comme les inputs v0 : aucune décision ni RNG caché,
aucune ressource de jeu supplémentaire dans la trace. Le rejeu en `Scripted` capture
les inputs décidés et doit produire la même trace.

`crates/bots/src/hunter.rs` choisit le tir visible, la distance de combat, le changement
vers une arme approvisionnée, le rechargement et la réanimation. L'acheteur cherche aussi
les munitions, Juggernog et une porte abordable quand le champ est inaccessible (prix,
puis distance à l’ennemi, puis `GgrsNetId`). Les approches ponctuelles utilisent des
champs multi-source dérivés et partagés sur la même grille. Sans poste de tir accessible,
le bot s’approche jusqu’au rayon d’aggro d'un zombie `Idle` : il peut alors casser sa
fenêtre. Un zombie déjà en `Chasing` ne doit pas retenir le bot dans ce rayon quand il
n'existe toujours aucun poste de tir ; les portes et réparations restent possibles.
Sans poste de tir ni approche d'aggro, il cherche le point physiquement accessible le
plus proche du zombie. Cette investigation conserve la possibilité d'ouvrir une porte
ou de réparer, même quand le zombie inaccessible est à moins de 150 px.
Sans chemin d’approche, le bot répare aussi pour gagner les points de sa première porte.
Les interactions passent par le bouton existant : les règles de solde, coût, cooldown et
réanimation restent celles du jeu. Les unitaires couvrent navigation et décisions ;
`crates/scenario/tests/bots.rs` vérifie le rejeu v0 et v1.
Si une autre surface possède le prompt à portée du but choisi, le bot se rapproche
encore du but ou essaie une autre interaction, au lieu de rester immobile sans agir.
`crates/scenario/tests/hunter_doors.rs` vérifie que les quatre acheteurs finissent la
première vague des graines 2, 11 et 12 sans cap ni décès.
Les modes Manual, Shotgun et Burst relâchent le tir lorsque `WeaponState.is_firing`
est vrai, puis pressent de nouveau ; Automatic peut maintenir le bouton. Ce choix lit
l'état rollback de l'arme, sans compteur de pulsations caché dans le bot.

Le déplacement du joueur résout un mouvement diagonal bloqué en faisant glisser X,
puis en vérifiant Y à la position X obtenue. Vérifier les deux axes depuis la position
initiale pouvait autoriser leur combinaison à entrer dans le coin d'un mur.

En phase `Spawning`, après 600 frames sans spawn depuis le dernier spawn ou le début
de phase, une plage de distance vide utilise le spawner le plus proche d'un joueur
(égalité départagée par `GgrsNetId`). Les spawners dans la plage normale restent
prioritaires. Le drapeau `WaveState.spawn_fallback`, rollback et tracé avec cette
ressource, maintient ensuite la cadence normale pour cette vague et est remis à faux
dans `prepare_next_wave`. Le hash de `WaveState` conserve l'ordre historique des champs
et ajoute une contribution seulement si ce drapeau est actif, afin de conserver les
traces des vagues qui n'en ont pas besoin. La limite d'ennemis
simultanés et la taille des lots restent appliquées. Le test synctest
`crates/scenario/tests/spawn_stall.rs` force une plage vide et vérifie trois spawns,
sans desync, plutôt qu'un retour indéfini du système.

En `Spawning` ou `InProgress`, après 600 frames sans kill ni spawn, les zombies de
vague peuvent récupérer un point de steering sorti de sa case ou placé dans un mur.
`ai/pathing.rs` échantillonne des points entiers à l'intérieur de la prochaine case,
puis les étapes cardinales d'une diagonale, puis la case courante : un coin de mur
peut couper toutes les routes directes, et un point de la case courante dégage le
corps du coin avant le segment vers la case suivante. Le choix est stable (étape,
distance au point initial, x, y), et le collider avec son offset doit rester libre
sur chaque segment, vérifié à intervalles d'un pixel. Les fenêtres intactes restent
bloquantes. Une étape intermédiaire doit permettre de rejoindre un point libre dans
la case cible. Si la case cible n'offre aucun point libre pour le corps (un mur
couvre toute la case), le glissement le long d'un axe du déplacement de case est
essayé d'abord — une poche de coin est souvent ouverte latéralement (fenêtre cassée
dans le mur sous le coin) quand la diagonale est pincée — puis la rotation du cap
vers le waypoint par paliers de 45°, rotations proches d'abord, demi-tour en
dernier. La première direction dont le segment reste libre sur 24 px à intervalles
d'un pixel gagne, et le zombie glisse hors du coin avant que le champ ne reprenne. La direction récupérée évite le mélange de steering qui
réintroduisait le mur ; le mouvement conserve vitesse, séparation et collision
ordinaires. Le délai dépend des compteurs existants de `WaveState`, sans état caché
ou nouveau type rollback.

## Notes essentielles

**À vérifier** : l'entité `CrateLocation` n'est pas lue actuellement (`WeaponLocation`/`SodaLocation` le sont depuis T2.3, voir §1 ci-dessus). Elle apparaît dans `crates/map_ldtk/src/map_const.rs` (constante) mais aucun bundle Bevy ne la traite (`entity/*.rs` ne la liste pas). Avant d'utiliser une carte avec une entité nouvellement lue, vérifier que `make test_scenarios` accepte un scénario `idle` dessus.

**Colliders vs sprites** (CLAUDE.md) : collider des entités = 20×20 pixels aux pieds (collider rectangle) ; sprite rendu = 32×32 pixels (ils débordent visuellement au-dessus du collider). Cellules du flow field = 16 pixels. Les deux décalages (sprite 32 vs collider 20, flow field 16) sont compensés par `offset_x`, `offset_y` dans `SpriteSheetConfig` et `offset` dans `CharacterConfig`.

**IntGrid `Walls`** : les valeurs ne sont pas nommées dans le JSON LDtk. Ajouter une nouvelle surface (eau, lave, boue) est le chantier E4 du plan ; modifier l'IntGrid et le collider selon le pattern de `Walls`.

**Fixed-point** : les valeurs `Fixed` s'écrivent en chaîne (`"100.0"`). Les entiers (`frames`, `GridPos`) ne sont pas interdits. Aucun `f32` ou `f64` dans les composants rollback. Voir `CLAUDE.md` Déterminisme, règle 1.

**Tests et scénarios** : `make test_scenarios` vérifie que les scénarios passent en synctest (mode déterministe local). Tout changement de code dans `GgrsSchedule` peut casser les traces `.trace` ; rebase sur `main` chaque jour et revalidate en CI rapide. Les seules voies autorisées à changer les traces : V1 (simulation), justifié par `BLESS=1` en commit.
