# Conventions de l'engine alacod

Référence pour ceux qui créent du contenu (cartes LDtk, sprites RON) et ceux qui ajoutent un vocabulaire à l'engine.

**Utilité** : lire cette page quand on crée une carte, importe des sprites, ou ajoute un effet/comportement/statut. Chaque section renvoie aux fichiers à lire pour comprendre le pattern.

**Numérotation du 2026-10-05** (m1-relecture-conventions) : « Feedback (présentation, T2.13) », ancien second §9 placé après le §10, devient le **§7** (il n'y avait pas de §7) et vient après le §6 ; §8 Combat, §9 Stats et §10 Blesser une trace gardent leur numéro, l'ancre `{#section7}` du §8 disparaît. Les fiches et rapports antérieurs qui citent « §9 » pour le feedback visent l'actuel §7 ; ceux qui citent « §7 » pour les stats ou « §8 » pour la preuve visent l'actuel §9 et l'actuel §10 (numérotation de M0).

**Sommaire**

- §1. Cartes LDtk
- §2. Sprites et animations
- §3. Le dossier de jeu
- §4. Ajouter un vocabulaire à l'engine : checklist
- §5. Commandes make
- §6. CI lente (nuit) — T2.14
- §7. Feedback (présentation, T2.13)
- §8. Combat : équipes et dégâts (T1.1, chantier B1)
- §9. Stats et modificateurs (T1.2, chantier B2)
- §10. Blesser une trace : la preuve
- §11. Munitions et inventaire d'armes (T2.2, chantier B7)
- §12. Monnaie et achats (T2.3, chantier C5 v1)
- §13. État de run et mode `Waves` (T2.4, chantier F1)
- §14. Power-ups (T2.5, chantier C1 v0)
- §15. HUD : icônes, noms et sources (T2.12)
- §16. Projectiles composables (T1.1, chantier B5 v1)
- §17. Mode `Floors` (T1.8, chantier F1)
- §18. Équilibrage par joueurs (F5)
- §19. Statuts (T1.3, chantier B3)
- §20. Patterns, émetteurs et tir ennemi (T1.2, chantier B5 v1)
- §21. Terrain destructible et cavernes (T1.0b + T1.6, chantier E3)
- §22. Behaviors composables (T1.4, chantier D1)
- §23. Horloges et difficulté (T1.9, chantier F2)
- §24. Bots de validation (m0-v7 phase 2)
- §25. Variantes et élites (T1.5, chantier D2)
- §26. Surfaces (T1.7, chantier E4 v1)
- §27. Effets v1, jauges et mutations (T1.10, chantiers C1 v1 et C4 v1)
- §28. Générateur v1 et placement scripté (T1.13, voie V3)
- §29. Le jeu `throne` (T1.0c + T1.11, voie V2)
- §30. Écran de mutation et transition (T1.16, voie V4)
- §31. Feedback v1 (T1.17, voie V4)
- §32. HUD throne (T1.18, voie V4)
- §33. Restart en ligne (D14)
- §34. Course et esquive (dash)
- §35. Salles typées et verrouillées (M2-E1, chantier E1)
- §36. Objets et inventaire (M2-T0b, chantier C2)
- §37. Boss à phases (M2-T0c, chantier D4)
- §38. Profil de méta-progression (M2-T0c, chantier G1)
- §39. Le jeu `gungeon` (M2-T0d, squelette)
- Notes essentielles

## 1. Cartes LDtk

**Version** : LDtk 1.5.3 (`jsonVersion` dans le fichier). Consulter `games/zombies/assets/exemples/test_map.ldtk`.

**Grille et taille** : tuiles de 16 pixels (`defaultGridSize: 16`). Un niveau LDtk = une salle de jeu. Les niveaux s'assemblent par leurs **connexions** (voir ci-dessous) ; le générateur (`crates/map/src/generation/`) en calcule position et taille dans le monde.

**Couches** : trois couches visibles dans l'éditeur.

- `Walls` (IntGrid, valeur 1) : les murs que les joueurs et ennemis ne traversent pas. Chaque cellule vaut 1 si c'est un mur, sinon absent (pas de 0 explicite). Détermine aussi le flow field (murs, portes fermées, obstacles cassables intacts) par ordre de priorité : murs > portes fermées > fenêtres intactes. Consulter `crates/map_ldtk/src/game/collider.rs` pour voir comment ces cellules sont converties en colliders physiques ; le flow field les utilise dans `crates/game/src/character/enemy/ai/navigation.rs` pour construire le graphe de navigation.
- `LevelConnection` (IntGrid, valeur 1) : indique les ouvertures (portes ou passages) sur chaque bord d'un niveau. Le générateur détecte ces connexions avec `scan_width_side` (`crates/map/src/generation/context.rs`) : un scan en ligne/colonne du bord compte les cellules contiguës valant 1. Chaque groupe continu = une ouverture possible vers un autre niveau. Exemple : une ouverture de trois cases sur le nord = trois cellules (1, 1, 1) consécutives dans la rangée du haut.
- `Entities` (couche objet Bevy LDtk) : emplacements des joueurs, ennemis, portes, fenêtres, armes et sodas. Chaque entité a une position (pixels dans le niveau) et une grille (cellule de 16 px du monde).

**Entités et leurs champs** (lus par `crates/map_ldtk/src/game/entity/*.rs`) :

- `DoorHorizontal`, `DoorVertical` (32×16 ou 16×32 px) : portes payantes. En éditeur LDtk, deux champs seulement : `price` (entier, coût en points), `electrify` (booléen, desserte électrique si vrai). Le générateur ajoute quatre champs aux portes qu'il place : `interactable` (booléen, défaut `true`), `paired_door_x`, `paired_door_y` (entiers), `paired_door_level` (chaîne) pour les portes appariées (une porte achetée ouvre son jumeau simultanément). Lus par `door_component_from_field` dans `crates/map_ldtk/src/game/entity/door.rs`. Les portes **bloquent le flow field** tant qu'elles ont un collider (portes fermées) ; une porte non interactive ne s'ouvre jamais (`CLAUDE.md`, section « Flow Field - Implémentation Actuelle »).
- `WindowHorizontal`, `WindowVertical` (16×32 ou 32×16 px) : vitres cassables. Aucun champ en éditeur. Santé fixée à 3 dans le code (`crates/map/src/game/entity/map/window.rs`). Les fenêtres intactes bloquent les ennemis (collision, mais permettent le tir à travers) ; cassées, elles bloquent les joueurs seulement (perméables aux ennemis). Réparation par interaction. Lus par `crates/map_ldtk/src/game/entity/window.rs`.
- `PlayerSpawn` (16×16 px) : points de départ des joueurs. Champ obligatoire : `index` (0, 1, 2, 3) = handle GGRS du joueur. Seul le niveau de départ a besoin de points de départ. Lus par `crates/map_ldtk/src/game/entity/player_spawn.rs`.
- `ZombieSpawn` (16×16 px) : emplacements de spawn des ennemis. Aucun champ. Le mode vagues (`WaveModeEnabled`, activé quand le jeu déclare un dossier `Wave` ; `RunMode::Waves` depuis T2.4, §13) y fait apparaître les ennemis. Lus par `crates/map_ldtk/src/game/entity/enemy_spawn.rs`.
- `CharacterSpawn` (16×16 px, T2.9, testbed) : fait apparaître, une fois au chargement de la map (pas par les vagues), un personnage précis du registre. Trois champs : `character` (chaîne, `CharacterId`, obligatoire), `variant` (chaîne optionnelle, variante imposée, T1.5, §25) et `team` (chaîne optionnelle : `players`/`enemies`/`allies`/`neutral` ; absente ou vide → `CharacterConfig.team` du personnage, sinon `Enemies`). Lus par `crates/map_ldtk/src/game/entity/character_spawn.rs` ; spawné par `map_ldtk::game::local::spawn_characters_when_map_loaded` via `character::enemy::create::spawn_enemy`. Utilisé par `games/testbed/assets/testbed/*.ldtk` pour placer `dummy`/`target`/`follower`/`ally`/`civilian`/`breacher` (`games/testbed/assets/characters/*.ron`).
- `WeaponLocation` (16×16 px, T2.3, chantier C5 v1) : arme murale payante. Deux champs en éditeur : `weapon` (chaîne, `WeaponId` de `weapons.ron`), `price` (entier, coût en points). Propagée par le générateur de salles (modèle `CharacterSpawn`, voir `crates/map/src/generation/entity/weapon_location.rs`) ; au chargement de la map, fait apparaître un `weapons::WeaponPickup` mural (`price: Some(...)`, jamais consommé au ramassage — voir `map_ldtk::game::local::spawn_weapon_locations_when_map_loaded`). Lue par `crates/map_ldtk/src/game/entity/weapon_location.rs`. `games/zombies/assets/exemples/test_map.ldtk` en pose quatre depuis T2.6 (pistol 500 dans le gabarit de départ `Level_0`, rifle 1200 et shotgun 1000 dans `Level_1`, machine_gun 1500 dans `Level_2` ; tableau dans `games/zombies/README.md`, qui rappelle aussi que la partie assemble des copies de ces gabarits ; depuis D10 le gabarit de départ `Level_0` n'est plus réutilisé, donc un seul mur `pistol`) ; `test_map_shop.ldtk` en pose une (shotgun, 500) pour les scénarios d'achat.
- `SodaLocation` (16×16 px, T2.3, chantier C5 v1) : machine à perk. Un champ en éditeur : `perk` (chaîne, `PerkId` de `economy/perks.ron`). Même propagation que `WeaponLocation` ; au chargement, fait apparaître une entité `game::economy::PerkMachine` avec `Interactable { interaction_type: Perk }` (voir `map_ldtk::game::local::spawn_soda_locations_when_map_loaded`). Lue par `crates/map_ldtk/src/game/entity/soda_location.rs`.
- `CrateLocation` (16×16 px) : **collectée mais non utilisée** (`crates/map_ldtk/src/map_const.rs` la déclare, le générateur en relève les positions dans le champ `crates` de `generation/from.rs`, aucun bundle implémenté en entity/*.rs). Réservée à un chantier futur (butin, C4).

**Champ de niveau** : le champ `spawn` (booléen) détermine le niveau de départ d'une run. Lu par `crates/map_ldtk/src/generation/from.rs` (`map_const::LEVEL_FIELD_SPAWN`) : si présent et `true`, le niveau est marqué `LevelType::Spawn`. **D10 (décision de William, 2026-10-14)** : le gabarit de départ n'est utilisé que comme salle de départ (`BasicMapGeneration::get_next_room_recursize` l'écarte des candidats des salles suivantes ; une connexion dont tous les gabarits compatibles sont des gabarits de départ est fermée comme `DeadEnd`). Avant, il pouvait réapparaître comme salle ordinaire (deux `Level_0` avec la seed des scénarios).

**Tailles de collision** : les colliders des entités (joueurs, ennemis) font 20×20 pixels aux pieds (voir `crates/game/src/character/config.rs`). Les sprites font 32×32 pixels. Les colliders de porte et fenêtre sont déterminés par leur taille LDtk (32×16 ou 16×32) ; l'engine les lit dans `crates/map_ldtk/src/game/entity/door.rs` et `window.rs`.

**Vérification** : `make map_preview ARGS=games/zombies/assets/exemples/test_map.ldtk` affiche une carte ; `make map_generation ARGS="games/zombies/assets/exemples/test_map.ldtk <sortie.ldtk> [graine]"` la génère avec assemblage des salles par connexions. Un scénario `idle` sur la carte rejouée en vidéo (`make play_scenario SCENARIO=idle`) vérifie visuellement l'assemblage et les entités.

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

**Vagues** (`games/zombies/assets/waves/wave_config.ron`) : RON, configuration simple. Champs : `base_enemies`, `enemies_per_wave`, `max_random_variance`, `min_wave_delay_frames`, `grace_period_frames`, `max_concurrent_enemies`, `spawn_batch_size`, `spawn_interval_frames`, `min_player_distance`/`max_player_distance` (Fixed en chaîne), `wave_tiers` (liste de `{max_wave, enemy_probabilities}`), `health_multiplier_per_wave`, `damage_multiplier_per_wave`, `max_wave` (optionnel, T2.4, §13). Chaque champ numérique accepte aussi une expression (`NumOrExpr`, §18). Les tirages passent par le flux RNG `"waves"` (`RngStreams`).

---

## 3. Le dossier de jeu

**Depuis T0.3** : structure `games/<jeu>/` autonome. Chaque jeu a son propre crate binaire (`games/<jeu>/Cargo.toml`, `src/main.rs`), ses assets (`games/<jeu>/assets/`), et ses scénarios de test. Le premier jeu implémenté est `zombies` (T0.3) ; un testbed minimal (`testbed`) valide l'engine sans simulation complexe ; `throne` (§29) est le deuxième clone.

```
games/<jeu>/
├── Cargo.toml         # Dépend de engine (crates/game, etc.)
├── src/
│   └── main.rs        # Boilerplate de l'engine + jeu
├── assets/
│   ├── sprites/       # Planches PNG + RON configs
│   ├── maps/          # LDtk (.ldtk)
│   ├── sounds/        # Sons (.ogg)
│   ├── ui/            # Configs HUD/écrans (RON)
│   └── game.ron       # Manifeste : dossiers, ids de contenu
```

Les scénarios ne sont pas dans le dossier du jeu : `tests/scenarios/` (écrits) et
`tests/scenarios/generated/<jeu>/` (générés, `make gen`).

**`game.ron`** (manifeste du jeu, `crates/content/src/manifest.rs`, T1.5) : nom, dossiers de
contenu typés, point d'entrée (carte de départ, graine par défaut). Extrait simplifié
(`games/testbed/assets/game.ron`, qui déclare aussi `mode: Sandbox` et d'autres dossiers) :
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
entier (voir `games/zombies/assets/game.ron`). Les kinds connus (20) : `Character`, `Weapon`,
`MeleeWeapon`, `Wave`, `Map`, `Ui`, `Camera`, `Economy`, `Perk`, `PowerUp`, `SpriteSheet` (D3),
`Floors`, `Cave`, `Room`, `Item`, `Surface`, `Pattern`, `Progression`, `Mutation`, `Status`, `Clock`, `Difficulty`
(`content::registry::KNOWN_KIND_NAMES`, qui fait foi) ; un
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
`DuplicateId`, `BrokenReference`, `OutOfRange`, `UnknownKind`, `Unsupported` (T1.10).

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
| personnage | `test:` (T1.13, §28) : `frames > 0`, au moins un gabarit (`still`/`moving`), `expect_*` seulement d'un gabarit actif | `OutOfRange` | `character_test_frames_zero`, `character_test_no_template`, `character_test_expect_inactive` |
| arme | `ammo_type` : variante inconnue (`Laser`) | `Parse` (nomme le champ) | `ammo_type_unknown` |
| arme | `ammo_type: Custom("")` (nom vide) (T2.8) | `OutOfRange` | `ammo_type_empty_custom` |
| arme, corps à corps | `friendly_fire` : valeur inconnue (enum fermé `Never`/`Always`/`Cursed`) (T2.8) | `Parse` (nomme le champ) | `friendly_fire_unknown` |
| arme | `audio_config.modes.<mode>.reloading`/`firing` : fichier absent de `assets/` (T2.8) | `BrokenReference` | `audio_missing_file` |
| arme | `audio_config.modes.<mode>` : mode absent de `firing_modes` (assets) | `BrokenReference` | `audio_unknown_mode` |
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
| `ui/feedback.ron` | arme de `by_weapon` connue du jeu (T1.17, §31) | `BrokenReference` | `feedback_by_weapon_unknown` |
| `ui/feedback.ron` | clé de `volumes` présente dans `sounds` (assets) ; valeur dans [-60, 12] dB | `BrokenReference` / `OutOfRange` | `feedback_volume_unknown_key` |
| `ui/feedback.ron` | surcharges `by_kind`/`by_weapon` dans les bornes (T1.17, §31) | `OutOfRange` | `feedback_override_out_of_range` |
| `ui/hud.ron` | source de widget inconnue (T1.18, §32) | `UnknownKind` | `hud_unknown_source` |
| power-up | `RefillAmmoOf` : munition déclarée par au moins une arme (D40, §14) | `BrokenReference` | `powerup_refill_ammo_unknown` |
| feuilles de sprites | `animation` et feuille de chaque calque présentes sous `assets/` (D3) | `BrokenReference` | `sprite_sheet_missing_file` |
| feuilles de sprites | image (`path`) de chaque feuille présente sous `assets/` (D3) | `BrokenReference` | `sprite_sheet_missing_image` |
| feuilles de sprites | id répété dans `sprites/sprites.ron` (D3) | `DuplicateId` | `sprite_sheet_duplicate_key` |
| arme | `sprite_config.name` absent de la table `SpriteSheet`, si le jeu en déclare une (D3) | `BrokenReference` | `weapon_sprite_unknown` |
| `game.ron` | `entry.mode: Floors` sans dossier `Floors` (T1.8) | `BrokenReference` | `entry_mode_floors_without_floors` |
| `game.ron` | `entry.clocks` vers une horloge inconnue ; `entry.difficulty` sans fichier `Difficulty` (T1.9, §23) | `BrokenReference` | `entry_clock_unknown`, `entry_difficulty_missing` |
| `game.ron` | `entry.progression` vers une progression inconnue (T1.10, §27 ; audit T1.12) | `BrokenReference` | `entry_progression_unknown` |
| `game.ron` | `generate_template` : carte connue, cible à `counts_hits` (T1.13, §28) | `BrokenReference` | `generate_template_unknown_map`, `generate_template_target_without_hits` |
| arme | projectiles composables (T1.1, §16) : projectile de pattern absent de la table, cycle de `on_expire`, pattern temporel ou `Scatter` en `on_expire`, `Size > 0`, `0 < Homing <= 1`, modificateur répété, `on_hit` limité aux actions à cible | `BrokenReference`, `OutOfRange` | `projectile_broken_reference`, `projectile_cycle`, `projectile_temporal_pattern`, `pattern_scatter_on_expire` |
| arme | `on_hit` `ApplyStatus` : statut inconnu, `stacks > 0` (T1.3, §19) | `BrokenReference`, `OutOfRange` | `status_unknown` |
| pattern | `Named` inconnu ou cyclique ; `count > 0`, `spread >= 0`, `speed >= 0` (T1.2, §20) | `BrokenReference`, `OutOfRange` | `pattern_unknown_name` |
| personnage | `ai.behaviors` `Shoot` : arme et pattern connus, projectiles du pattern dans la table de l'arme, `range > 0`, `cooldown_frames > 0` (T1.4, §22) | `BrokenReference`, `OutOfRange` | `behavior_shoot_unknown`, `ranged_projectile_missing`, `ranged_cooldown_zero` |
| personnage | `Melee` : arme de corps à corps connue ; `Chase` : profil connu ; `KeepDistance` : `min < max` ; `Charge` : télégraphe > 0 ; `targeting.ignore` : tag porté par un personnage (T1.4) | `BrokenReference`, `OutOfRange` | `behavior_melee_unknown`, `behavior_unknown_profile`, `behavior_keep_distance_inverted`, `behavior_charge_zero`, `targeting_unknown_tag` |
| personnage | `variants` : `chance` dans [0, 1], `weight > 0`, nom unique, skin connu, pas de `MoveSpeed` sur un IA (T1.5, §25) | `OutOfRange`, `DuplicateId`, `BrokenReference` | `variant_chance_out_of_range`, `variant_weight_zero`, `variant_duplicate`, `variant_skin_unknown`, `variant_move_speed_ai` |
| personnage | `effects` (T1.10, §27) : déclencheur, condition ou action non exécutés (`Unsupported`), `Tick(0)`, `Heal <= 0`, pattern et arme de `SpawnPattern` connus, jauge d'`OnGauge`/`GaugeAdd` connue | `Unsupported`, `OutOfRange`, `BrokenReference` | `effect_unsupported`, `effect_out_of_range`, `effect_broken_reference`, `progression_broken_reference` |
| carte | `CharacterSpawn` : personnage connu (audit T1.12) ; `variant` imposée déclarée par ce personnage (T1.5) | `BrokenReference` | `map_character_unknown`, `variant_ldtk_unknown` |
| caverne | au moins 16 × 16, `fill_ratio` dans [0, 1], `min_floor_ratio` dans [0, 0.9], `birth`/`survive` <= 8 ; `characters` connus ; gabarit `gabarit.ldtk` présent (T1.6, §21) | `OutOfRange`, `BrokenReference` | `cave_out_of_range`, `cave_unknown_character`, `cave_template_missing` |
| séquence `Floors` | `levels` non vide ; carte ou `cave:<id>` chargée (T1.8, §17) ; caverne sans ennemi (`characters` vide ou `enemy_spawns` nul) refusée sauf `transit: true` (D36, §21) | `OutOfRange`, `BrokenReference` | `floors_empty`, `floors_unknown_map`, `floors_unknown_cave`, `floors_cave_without_enemies` |
| surface | `intgrid_value > 0` et unique, facteurs > 0, tags non vides (T1.7, §26) | `DuplicateId`, `OutOfRange` | `surface_duplicate_value`, `surface_factor_non_positive` |
| horloge, difficulté | ids d'événements uniques, échéances croissantes, `repeat > 0` ; expression : identifiants admis, valeur > 0 aux bornes (T1.9, §23) | `DuplicateId`, `OutOfRange`, `Parse` | `clock_duplicate_id`, `clock_unordered`, `clock_repeat_zero`, `difficulty_unknown_identifier`, `difficulty_non_positive` |
| progression | `per_kill > 0`, `levels` croissants, `choices` dans [1, pool], `choice_frames > 0`, chance dans [0, 1] ; mutations et armes du pool connues (T1.10, §27) | `OutOfRange`, `BrokenReference` | `progression_out_of_range`, `progression_broken_reference` |
| mutation | `weight > 0`, `max_stacks > 0`, `effects` non vide (+ règles des effets) (T1.10) | `OutOfRange` | `mutation_out_of_range` |
| statut | `frames > 0` ; `Burn` : `damage > 0`, `period > 0` ; `Slow` : `factor` dans ]0, 1] (T1.3, §19) | `OutOfRange` | `status_out_of_range` |
| power-up | action propre aux projectiles ou aux effets (`DestroyTerrain`, `ApplyStatus`, `Modifier`, `Heal`, `SpawnPattern`, `GaugeAdd`) | `OutOfRange` | `powerup_apply_status` |

Les enums fermés (`FriendlyFire`, `ModifierOp`, `Action`) et les variantes nues inconnues des
enums ouverts (`AmmoType`, `StatId`, qui n'acceptent un nom libre que sous `Custom("...")`)
échouent au chargement RON : la règle est le parse lui-même, la fixture documente le message
(`champ ammo_type : ...`, `champ friendly_fire : ...`, `champ stat : ...`). Une table RON
(`{ "id": ... }`) désérialisée en `BTreeMap` garde silencieusement la dernière valeur d'une
clé répétée. Le registre lit donc `perks.ron`, `weapons.ron`, `melee_weapons.ron` et la table
`powerups` de `items/powerups.ron` en liste (`registry::KeyedEntries`) pour rapporter les
doublons, y compris dans un même fichier. La première entrée est conservée, la seconde
produit `DuplicateId`. Les fichiers `feedback.ron` et `hud.ron` déclarés sous le kind `Ui`
sont lus avec un schéma typé ; les autres fichiers Ui restent validés syntaxiquement.

**CLI** : `cargo run -p content --bin alacod --profile headless -- lint games/<jeu>` (code
de sortie 1 et messages sur stderr en cas d'erreur, 0 sinon) ; `make lint` l'appelle pour
`zombies`, `testbed` et `throne`. Un hook `PostToolUse` (`.claude/settings.json`,
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

1. **Écrire le test (scénario)** : créer `tests/scenarios/<nom>.ron` avec attentes (`expect: [Expectation]`), invariants de frame, et commentaire « À regarder » décrivant ce qu'on doit observer à l'écran (quelles frames, quel comportement). Le scénario est le cahier des charges ; le code le réalise.
2. **Déclarer le kind** : enum Rust sérialisable en RON dans la crate de vocabulaire (`crates/combat`, `crates/behaviors` ou `crates/effects`). Un vocabulaire déclaré par un plugin s'enregistre auprès de `KindRegistry` (trait de `sim_core`, T0.2) ; un **kind de contenu** (dossier de `game.ron`) s'ajoute à `content::registry::KNOWN_KIND_NAMES` et au `match` de `Registry::build`.
3. **Composants et état** : déclarer tous les composants Bevy qui portent l'état. **Pas de HashMap, HashSet**, uniquement BTreeMap/BTreeSet pour l'ordre déterministe. Consulter `CLAUDE.md` Règles critiques du déterminisme.
4. **Enregistrement rollback** : `app.rollback_and_trace::<StatusComponent>()` pour chaque composant mutable, ou sa variante `_neutral` pour un type nouveau posé seulement sur de nouvelles entités (piège de parité du checksum) ; ressources : `app.rollback_and_trace_resource::<StatusResourceCache>()`. Aucun appel direct à `rollback_component_*`/`rollback_resource_*` (plan §9.6, K0, `scripts/check-rollback-registration.sh`). La règle complète et ses pièges font foi dans `CLAUDE.md` (« Checklist pour Nouveau Système GGRS »).
5. **Flux RNG dédié** (plan §4.6) : si le vocabulaire utilise l'aléatoire (variantes, direction), créer un `stream` unique (ex. `rng_streams.get_mut("status")`, `RngStreams`, `crates/bevy_fixed/src/rng.rs`). Consommer le RNG dans un ordre déterministe (après tri par `GgrsNetId`), jamais à la première occurrence.
6. **FrameEvents** : tous les événements émis (impact, mort, soin) passent par `FrameEvents<T>` (`crates/sim_core/src/frame_events.rs`, réexporté par `game::frame_events`). Lus par les systèmes de la simulation (`GgrsSchedule`), ordonnés après l'émetteur. La présentation en dérive (`Update`, jamais d'événements), restant juste après un rollback.
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
| `combat` | `ProjectileModifier` : Bounce, Pierce, Size, Lifetime, Homing, Gravity ; `Pattern` : Aimed, Spread, Ring, Sequence, Telegraph, Wait (+ `Scatter`, `Named`, T1.2) ; `StatusDef` : Burn, Slow, Stun, Freeze ; état `Statuses(Vec<StatusEntry>)` (statut, piles, frame d'expiration) | `CombatPlugin` ; Projectiles, Effects, Status |
| `behaviors` | `Behavior` : Chase, KeepDistance, Strafe, Charge, Shoot, Melee, Flee, Wander ; `Perception` : Sight, Hearing ; `Targeting::Nearest` ; état `BehaviorState` (indice de règle, frame d'entrée, cible GgrsNetId) | `BehaviorsPlugin` ; EnemyAI |
| `effects` | `Effect { on, if, do }`, `On` (12 déclencheurs depuis `OnLevelUp`, T1.10), `Condition` (7 conditions), `GaugeThreshold::Above` ; réutilise `Action` des power-ups | `EffectsPlugin` ; Effects |

Les plugins enregistrent les noms Rust exacts (PascalCase) dans les catégories snake_case
`projectile_modifier`, `pattern`, `status`, `behavior`, `perception`, `targeting`,
`effect_trigger`, `effect_condition`. T1.12 a ajouté la validation des références dans le
contenu. `Kinds` et `KindRegistry` restent inchangés. `Statuses` et `BehaviorState` sont
enregistrés ensemble en rollback + checksum + trace ; `Statuses` est posé par l'`on_hit`
`ApplyStatus` depuis T1.3 (§19), `BehaviorState` reste sans porteur (T1.4 garde l'état dans
`BehaviorRuntime`, §22). Il n'y a pas de nouveau `RollbackSystemSet`.

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
| `make check` | Format, tests unitaires, scénarios, motifs interdits en avertissement, enregistrement rollback (`check_rollback_registration`, strict) (la CI rapide) |
| `make test_scenarios` | Rejoue tous les scénarios en headless, compare les traces |
| `make test_scenarios_par [SHARDS=4]` | Même suite en N processus, tranches réparties par durée mesurée (`scripts/scenario-shard-plan.py`) |
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

**Vitesse de la suite (perf-suite-scenarios, 2026-10-14)** : 243 scénarios en ~17 min dans un
seul processus et ~5 min en 4 tranches sur 4 cœurs (avant : ~51 et ~12 min). Ce qui l'explique,
dans l'ordre du gain, et ce qu'il ne faut pas défaire :

- **Exécuteur monothread en headless** (`runner::single_threaded_schedules`) : la simulation
  compte des centaines de petits systèmes par frame, l'exécuteur multithread de Bevy y passait
  plus de temps à se réveiller (~1 500 `futex` par frame) qu'à calculer, et ses threads se
  disputaient les cœurs entre tranches parallèles. `ALACOD_EXECUTOR=multi` le rétablit. Un
  système sans ordre explicite ne change pas de résultat : `GgrsSchedule` refuse toute ambiguïté
  d'ordre (les 243 traces le prouvent).
- **`FlowFieldCache::layers` en `SharedFlowField`** (`Arc` + image des octets du `Hash`) : le
  snapshot rollback le clonait et le checksum le hachait 2 à 3 fois par frame (~23 000 entrées
  de `BTreeMap`, ~20 % d'une frame). Un champ de flux n'est jamais modifié après sa
  construction : toute nouvelle couche passe par `SharedFlowField::from`. Le flux d'octets du
  `Hash` est celui de `#[derive(Hash)]` sur `FlowField` (test
  `shared_flow_field_hash_equals_derived`) : aucune trace ne bouge.
- Contenu (`Registry`, `GameManifest`) chargé **une fois par processus** (`runner::load_content`),
  lecture des périphériques coupée (`game::character::skip_device_input`, `ActionState` n'est lu
  qu'avec `InputSource::Devices`) et overlay de vagues (texte d'UI) coupé en headless.
- **Tranches par durée** : `scripts/test-scenarios-sharded.sh` lit `sim_seconds` dans
  `target/metrics/latest.json` (sinon les `frames` du `.ron`) et remplit les tranches du plus
  long au plus court (plus long d'abord, sur la tranche la moins chargée). Les métriques d'un
  run partiel écrasent `latest.json` : une suite complète les remet à jour. `SHARD_PLAN=roundrobin`
  rétablit un scénario sur N.
- `ALACOD_PROFILE=1` affiche, par scénario, le temps du runner par poste (`update` de
  Bevy, métriques, invariants) ; les invariants et les attentes pèsent moins de 0,5 % d'une frame.

**perf-navigation (2026-10-15)** : `build_flow_field` et `BotNavigation::build_field` calculent sur
tableaux `Vec` denses (sortie reconstruite en `BTreeMap` triée, même `Hash`) ; `throne_quad`
37,6 → 102 fps, `bench_floor` 96 → 121. Les calculs historiques (`build_flow_field_legacy`,
`build_field_legacy`) restent comme références de test : tout changement de l'un des deux doit
rester égal case par case. Détail : `docs/taches/rapports/perf-navigation.md`.

Ce qui restait avant perf-navigation était la simulation elle-même : `bench_floor` (profil callgrind, 600 frames) passe
~42 % de ses instructions dans `update_flow_field_system` (Dijkstra, dont ~22 % dans
`is_blocked`) et ~14 % dans la navigation des bots ; les `throne_*` ne sont que 1,5 à 2 fois
plus rapides qu'avant, pour la même raison probable (non profilés). Le détail chiffré est dans
`docs/taches/rapports/perf-suite-scenarios.md`.

---

## 6. CI lente (nuit) — T2.14

**Quoi** : chaque nuit (03:03 UTC) ou à la demande (`workflow_dispatch`), un runner auto-hébergé (`[self-hosted, alacod-builder]`) exécute un pipeline complet : benchmarks strict, simulation multi-bots, p2p headless (2 et 4 joueurs via allumette, détection des desyncs), génération des vidéos de tous les scénarios, page de revue interactive, et notes Markdown commitées dans `tests/review-notes/<commit>.md`.

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

1. **Bench** (seuils de `tests/budgets.ron`) : rejeu de tous les scénarios en headless, traces strictes ; les budgets absolus de `tests/budgets.ron` sont désactivés en CI (machine environ 4× plus lente, `scripts/nightly.sh`) : la régression de FPS se lit dans le tableau de `scenario-metrics.py`.

2. **Sim** (simulation bots) : centaines de parties sans rendu à N bots sur M graines jusqu'à la vague K, détecte les crashes et deadlocks ; JSON de métriques par graine (taux de survie, ennemis tués, vague atteinte, FPS moyenne). L'absence de crash = pas de desync en single-player.

3. **P2P headless** (2 et 4 joueurs) : lance une instance d'allumette (docker compose), puis N clients `zombies` en headless dans le même lobby sans rendu. Chaque client écrit sa trace d'état à chaque frame (`ALACOD_STATE_TRACE`, `ALACOD_EXIT_AT_FRAME=600`). À la fin, compare les traces : identiques = pas de desync, différentes = log la première frame qui diffère et échoue (information précieuse pour debug).

4. **Vidéos** : encode chaque scénario en MP4 (960×540, 60 FPS), stocke dans `target/videos/<commit>/`.

5. **Page de revue** : `scenario-review.py` génère une page HTML (`scripts/scenario-review.html`, copiée en `review/index.html` avec un `data.js`) avec tableau des scénarios (trace, FPS, moments clés détectés : vagues, kills, morts, tirs, rechargements, perte de santé) ; tableau d'historique par commit avec courbes FPS (benchmark vs sim).

6. **Notes** : Markdown généré (`tests/review-notes/<commit>.md`) avec lien vers la page de revue, timestamp, commit, résumé des étapes (✅/❌), métriques agrégées (FPS min par scénario, nombre de desyncs, p2p clients testés).

**Résultats** (artefacts GitHub Actions) :
- Vidéos MP4 (évaluables sans machine avec GPU) : `nightly/<commit>/videos/`, copiées de
  `target/videos/<commit>/` après `make videos` (`scripts/nightly.sh`, m1-dettes-doc-lot-2)
- Page de revue HTML (tableau des FPS, moments clés, comparaison historique)
- Traces P2P (fichiers texte, premiers desyncs identifiables avec `diff`)
- JSON de métriques (entrées pour graphiques, alertes de régression)

**Si p2p échoue** : examine les logs dans `logs/p2p-<N>-<cid>.log` (socket, timeout, spawn, GGRS frame error) ; les traces dans `target/nightly/<commit>/p2p-*.trace` montrent la divergence. Deux clients en headless => 10 minutes avec `--quick`.

---

## 7. Feedback (présentation, T2.13)

**Configuration RON** : fichier `games/<jeu>/assets/ui/feedback.ron` charge les paramètres de flash, secousse et sons.

```ron
(
    hit_flash: (
        frames: 4,                  // Durée du flash blanc (frames de simulation)
        color: (2.5, 2.5, 2.5),     // Multiplicateur des calques (> 1 = surexposé ; (1, 1, 1) invisible)
    ),
    shake: (
        frames: 8,                  // Durée de la secousse
        amplitude: 4.0,             // Amplitude du décalage en pixels
    ),
    sounds: {
        // Sons de base. Tir : clé de la famille du mode de tir actif de l'arme (Manual :
        // shot_pistol, Shotgun : shot_shotgun, Automatic/Burst : shot_smg), repli sur "shot".
        "shot_pistol": "sounds/shot_pistol.ogg",
        "shot_shotgun": "sounds/shot_shotgun.ogg",
        "shot_smg": "sounds/shot_smg.ogg",
        "reload": "sounds/reload.ogg",
        // Événements (lot B) : coup porté par un joueur local, dégâts reçus, ennemi tué,
        // dash, blank, porte ouverte.
        "hit": "sounds/hit.ogg",
        "hurt": "sounds/hurt.ogg",
        "death": "sounds/death.ogg",
        "dash": "sounds/dash.ogg",
        "blank": "sounds/blank.ogg",
        "door": "sounds/door.ogg",
        // Les clés absentes désactivent le son correspondant
    },
    // Volume par clé de `sounds`, en dB (0 : inchangé, plage [-60, 12]) ; un son d'arme
    // (`audio_config`) prend le volume de la clé qu'il remplace.
    volumes: {"hit": -3.0},
)
```

**Son propre d'une arme** (`weapons.ron`, champ `audio_config` de l'entrée, à côté de
`config`/`sprite_config`) : un tir et un rechargement par mode de tir, chacun facultatif.
```ron
"pistolet": (
    config: (...),
    sprite_config: (...),
    audio_config: (modes: {
        "default": (firing: "sounds/shot_pistol.ogg", reloading: "sounds/reload_pistolet.ogg"),
    }),
),
```
Priorité du tir : `firing` du mode actif de l'arme, sinon la clé de sa famille dans `sounds`,
sinon `shot` ; du rechargement : `reloading` du mode actif, sinon `reload`. `audio_config` reste
sur l'asset `WeaponAsset` (lu par `game::feedback::WeaponAudio` dans
`Assets<WeaponsConfig>`) et n'est jamais copié dans le composant rollback `Weapon` : aucune
trace ne bouge. Lint : fichiers présents sous `assets/`, mode connu de `firing_modes`.

**Systèmes** : tous en `PostUpdate` (hors `GgrsSchedule`), lisant les événements de simulation dans `FrameEvents<T>` émis par `GgrsSchedule`. Les trois émetteurs sont :
- `DamageEvent` pour les impacts (flash, secousse si joueur local)
- Spawn de `Bullet` pour le son de tir (source = joueur local) : un son par tir (tireur, frame de création), mémorisé pour ne pas repartir quand le rollback de la session locale recrée les balles des dernières frames (elles redeviennent `Added<Bullet>`). Son : celui de l'arme active du tireur et de son mode actif à l'image rendue, selon la priorité ci-dessus (familles : `Shotgun` → `shot_shotgun`, `Automatic`/`Burst` → `shot_smg`, `Manual` → `shot_pistol` ; la balle ne porte pas son arme : un champ de plus ferait bouger les traces). Les échantillons sont des coups uniques courts (≤ 0,5 s, lot A de `docs/plan-assets.md`), joués en entier : plus de coupure en fondu.
- Passage de `WeaponInventory.reloading_ending_frame` de `None` à `Some` pour le son de rechargement (celui de l'arme active, priorité ci-dessus) : une fois par rechargement, comparé d'une image rendue à la suivante
- Sons d'événements (lot B), joués par `FeedbackSounds::play(clé)` ; chaque signal est lu dans l'état ou dans les `FrameEvents` de la dernière frame simulée, jamais écrit :
  - `hit` : un `DamageEvent` dont la source est un joueur local et la cible non, un son par frame simulée (gardé par `DamageEvent.frame` : une image rendue sans nouvelle frame voit encore les mêmes événements). L'événement précède la résolution : un coup annulé (invulnérabilité) sonne aussi.
  - `hurt` : `Health.current` d'un joueur local en baisse d'une image rendue à l'autre, à `max` égal ; au plus un son toutes les 15 frames (`HURT_SOUND_MIN_FRAMES` : les PV peuvent baisser deux frames de suite).
  - `death` : un ennemi portant `Death` et `RollbackDespawned` (requête qui nomme ce composant désactivant), une fois par `GgrsNetId`.
  - `dash` : front montant de `DashState::is_dashing` d'un joueur local.
  - `blank` : `FrameEvents<BlankUsed>` d'un joueur local, gardé par la frame de l'événement.
  - `door` : une porte avec collider à l'image précédente, sans collider maintenant ; rien pour l'ouverture d'office des portes de salle (`RoomStates::doors_opened` qui passe à `true`, début d'étage) ni à la première image (référence). Pas encore de son de coffre : les coffres (M2-T5) ne sont pas dans `main`.

**Composants non-rollback** :
- `HitFlash { until_frame, color }` : posé sur l'entité cible d'un `DamageEvent`, multiplie la couleur des calques enfants (`apply_layer_colors`) jusqu'à `until_frame`.
- `CameraShake { until_frame, frames, amplitude, applied }` : pose sur la caméra quand un joueur local prend des dégâts.

**Déterminisme** : la secousse applique un motif déterministe (décalage indexé par `FrameCount`) pour la reproductibilité des captures (`--capture` de `play_scenario`). Jamais de source aléatoire (`rand`, temps réel).

**Journal de preuve** : chaque effet écrit une ligne `info!("feedback f{frame} <effet> {net_id|kind}")` pour vérification sans écran.

**Feedback v1** (T1.17) : hit stop, chiffres de dégâts, télégraphe au sol, surcharges par genre et par arme, flash sur les calques enfants et journal `FeedbackLog` (headless compris) : voir §31, qui fait foi sur ce paragraphe.

## 8. Combat : équipes et dégâts (T1.1, chantier B1)

**Équipe** (`sim_core::team::Team` : `Players`, `Enemies`, `Allies`, `Neutral`) : composant statique posé une fois à la création, par `character::player::create::create_player` (`Team::Players`) et `character::enemy::create::spawn_enemy` (paramètre `team` : `Team::Enemies` pour les vagues, sinon `CharacterSpawn.team` ou `CharacterConfig::team` ; `Allies`/`Neutral` servent à `ally`/`civilian` du testbed, T2.9). Hors rollback (jamais muté en T1.1 ; voir la doc du composant pour la justification). `Allies` compte comme la même équipe que `Players` pour le tir ami ; `Enemies` est sa propre équipe. `Neutral` bloque toujours un coup (comme un mur) mais ne subit jamais de dégât.

**`DamageEvent`** (`sim_core::damage`, `FrameEvents<DamageEvent>`) : toute blessure passe par là — six émetteurs (`weapons::bullet_rollback_collision_system`, `weapons::melee::melee_hitbox_collision_system`, `character::enemy::ai::behavior::enemy_attack_damage_translate_system`, `combat::projectile::projectile_collision_system` (B5), `combat::projectile::status_tick_system` (T1.3), `character::enemy::ai::rules::charge_damage_translate_system` (T1.4)), un seul résolveur (`character::health::rollback_resolve_damage_events`, `RollbackSystemSet::CollisionDamage`, après les émetteurs, avant `DeathManagement`). Le résolveur applique `combat::damage::resolve_damage` (fonction pure, testée dans `crates/combat/src/damage.rs`) puis écrit `DamageAccumulator`/`HitBy` (appliqués à `Health` par `rollback_apply_accumulated_damage`, inchangé).

**Politique de tir ami** (`sim_core::damage::FriendlyFire` : `Never`/`Always`/`Cursed`, `#[serde(default)]` = `Never`) : champ RON `friendly_fire` sur `WeaponConfig` (`weapons.ron`), `MeleeWeaponConfig` (`melee_weapons.ron`) et `EnemyAiConfig` (en dur, pas encore RON). `Cursed` ne touche un allié que si la source porte le tag `cursed` (`combat::team::CURSED_TAG`). **D54** : quelle que soit la politique, une balle ne touche jamais son propre tireur (`bullet.source`) : exclu dans `bullet_rollback_collision_system`, dans la collision des projectiles composables et dans le choix de cible de `Homing`/`Aimed` (`nearest_target`) ; un test de tir ami a donc besoin d'un second tireur (scénario `dash_stun_buffer`, 3 joueurs).

**Tags et défenses** : `CharacterConfig` (`character/config.rs`) gagne trois champs RON, tous `#[serde(default)]` (vides) : `tags: [...]` (posé en composant `sim_core::tag::Tags`, ex. `["cursed"]`, `["zombie"]`), `immune_to: [...]` et `resistances: {"tag": "mult"}` (posés ensemble en composant `combat::damage::Defenses`). `Tags`/`Defenses` sont hors rollback, comme `Team`. Chaque `DamageEvent` porte `tags: Tags` = tags du personnage source **union** un tag de genre d'attaque posé par l'émetteur (`bullet` pour une balle, `melee` pour une attaque au corps à corps — arme ou griffe) : une griffe de zombie porte donc `melee` et `zombie` (le tag vient de `CharacterConfig::tags` du zombie). Une cible dont `immune_to` contient un de ces tags ne subit aucun dégât ; `resistances` multiplie le montant à la place. `DamageKind::True` ignore résistances, immunités et invulnérabilité (mais pas l'équipe/le tir ami).

**Invulnérabilité** : `Health.invulnerable_until_frame` (déjà présent, T0.2) est enfin lu par `resolve_damage` — aucun dégât tant que `frame <= invulnerable_until_frame`.

**Ce qui reste de `CollisionLayer`/`layer_matrix`** (`crates/combat/src/collider.rs`, `CollisionSettings::layer_matrix`) : uniquement les collisions **physiques** avec les murs (balles et personnages qui s'arrêtent sur un mur) et entre personnages/fenêtres. La décision « cette balle/cette mêlée touche-t-elle ce personnage » ne passe plus par la matrice : elle vient de `Team` + `combat::team::team_allows_hit` (équipe + politique de tir ami de l'arme), évaluée pour chaque personnage candidat indépendamment de son `CollisionLayer`.

**Scénarios de référence** (`tests/scenarios/`) : `friendly_fire_never.ron`, `friendly_fire_cursed.ron`, `immune_tag.ron`. Format `Scenario` étendu (`game::replay`) : `PlayerScript` gagne `tags`/`immune_to` (posés en composants après création par `scenario::runner::apply_player_overrides`) ; `WeaponOverride` gagne `friendly_fire` (posé sur l'arme entière, avant la création des joueurs, par `apply_weapon_overrides`).

**Bug pré-existant préservé (pas corrigé par T1.1)** : avant ce chantier, l'attaque directe de `EnemyAiConfig::attack_damage` (`enemy_attack_system`) n'avait presque jamais d'effet — elle écrivait dans `DamageAccumulator` sans `Option`, composant retiré dès qu'appliqué la même frame, donc absent la plupart du temps. Seule la griffe via hitbox (`MeleeWeaponConfig::damage`, ex. `zombie_claws` à 2.5) touchait vraiment le joueur. `enemy_attack_damage_translate_system` reproduit ce comportement à l'identique (garde explicite, voir sa doc) : le corriger changerait l'équilibrage (dégâts ennemis beaucoup plus fréquents) sans rapport avec ce chantier.

## 9. Stats et modificateurs (T1.2, chantier B2)

**Contrats** (`sim_core`) : `stats::{StatId, Stats}` (identifiants et valeurs de base, `BTreeMap<StatId, Fixed>`) et `modifier::{ModifierOp, ModifierSource, Modifier, Modifiers, resolve}`. `StatId` est un enum ouvert (`Custom(String)` pour un vocabulaire propre à un jeu) ; `ModifierOp` a quatre opérations `Set` (dernier gagne), `Add` (somme), `Pct` (pourcentage additif, `0.5` = +50 %, plusieurs `Pct` s'additionnent avant de multiplier une seule fois) et `Mul` (produit direct). `resolve(base, modifiers, frame)` calcule `(base + Σ Add) × (1 + Σ Pct) × Π Mul`, `Set` appliqué d'abord ; un `Pct` absent ne multiplie même pas par 1 (aucune dérive possible en l'absence de modificateur). Le crate ne pose rien sur aucune entité — c'est le rôle de `crates/stats` et de `character::create::create_character`.

**Branchement** (`crates/stats`) : `StatsPlugin` enregistre `Stats`/`Modifiers` en rollback (`RollbackTraceApp`, donc dans le `Checksum` GGRS et les traces) et ajoute `expire_modifiers_system` (`RollbackSystemSet::Status`), qui retire les modificateurs expirés de chaque entité à la frame courante (`Modifiers::retain_active`). `StatReader` (`SystemParam`) est le point de lecture unique : `stats.get(entity, &StatId::X, valeur_par_défaut)` résout la stat (base + modificateurs actifs) à la frame courante ; `stats.try_get(entity, &StatId::X)` renvoie `None` plutôt qu'un défaut si l'entité n'a pas la stat (utilisé quand réutiliser une valeur déjà en place comme base la ferait dériver sous un modificateur multiplicatif, voir `character::health::sync_health_from_stats`). Aucun système de `crates/game` ne rappelle `sim_core::modifier::resolve` directement (seule exception, assumée : `character::variant::variant_health`, santé de départ d'une variante au spawn, T1.5 — à ce moment `Stats`/`Modifiers` n'existent pas encore sur l'entité, il résout donc lui-même les modificateurs `MaxHealth` de la variante sur la santé de base).

**Modificateurs sourcés** : `Modifiers::push_from(source, stat, op, value, until)` construit et ajoute un modificateur ; `Modifiers::remove_by_source(&source)` retire tous les modificateurs d'une même source d'un coup — un statut qui expire ou une salle qu'on quitte appelle cette méthode avec le `ModifierSource` donné à la pose, sans connaître le détail des modificateurs posés (les chantiers de statuts B3 et de salles E1 s'en serviront).

**Stats posées à la création** (`character::create::create_character`) : pour tout personnage, `MoveSpeed`/`EnemyMoveSpeed` = `movement.max_speed`, `Acceleration` = `movement.acceleration`, `SprintMultiplier` = `movement.sprint_multiplier`, `MaxHealth` = santé max résolue (`ResolvedBalance::health_max_by_character`, F5, §18), `HealthRegen` = `base_health.regen_rate` si présent, et les multiplicateurs `FireRate`/`ReloadSpeed`/`Damage`/`Range` à 1 (produit neutre). Pour un ennemi (`enemy::create::spawn_enemy`), en plus : `SeparationDistance`/`SeparationForce`/`SlowDownDistance`/`OptimalAttackDistance` viennent de `PathfindingConfig::default()` (la constante jusque-là partagée par tous les ennemis, lue par `move_enemies`). `CharacterConfig::stats` (RON, `#[serde(default)]`) surcharge ensuite ces valeurs de base, en dernier — un personnage qui ne déclare rien dans `stats:` a un comportement strictement identique à avant ce chantier. Exemple RON :

```ron
stats: {
    MoveSpeed: "180.0",
    Custom("sacre"): "1.0",
},
```

**Lecture branchée** : `character::player::input::apply_inputs` (accélération, multiplicateur de sprint, vitesse max — aussi la vitesse de sortie du dash, §34), `character::enemy::ai::pathing::move_enemies` (vitesse, séparation, ralentissement, distance d'attaque optimale — la granularité de la grille spatiale de séparation reste la constante globale de `PathfindingConfig`, seulement pour le partitionnement, voir le commentaire dans `move_enemies`), `weapons::weapon_rollback_system`/`spawn_bullet_rollback` (cadence de tir, temps de rechargement, dégâts et portée d'une balle, multipliés par les stats du porteur) et `character::health::sync_health_from_stats` (`Health.max` et `HealthRegen.regen_rate`, recalculés chaque frame après l'expiration des modificateurs — `current` est borné à `max` s'il le dépasse, jamais relevé automatiquement). Le freinage (`movement.deceleration`, multiplié par l'adhérence `Acceleration` résolue / `movement.acceleration`, §34) et les champs de dash (`dash_distance`, `dash_duration_frames`, `dash_cooldown_frames`, `dash_iframes`, `dash_buffer_frames`) et de sprint (`sprint_acceleration_per_frame`/`sprint_deceleration_per_frame`) restent des lectures directes de `CharacterConfig` : ils ne sont pas dans la liste de stats de la tâche T1.2.

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

**Comparaison** (`scripts/trace-diff.py <a.full> <b.full> [--ignore Nom1,Nom2,...]`) : retire toujours le checksum de l'en-tête (il change dès qu'un composant est ajouté au rollback, ce n'est pas ce qu'on compare) et neutralise toute valeur `Entity` brute embarquée dans un Debug (`{index}v{generation}`, ex. `WeaponInventory.weapons: Vec<(Entity, Weapon)>`) : elle dépend du nombre exact d'entités déjà créées dans **ce process** au moment du spawn (minutage du chargement des assets), pas déterministe d'un lancement à l'autre même à code strictement identique — c'est déjà pour ça que `WeaponInventory` l'exclut de son `Hash` manuel (voir le `Hash` manuel de `WeaponInventory` dans `crates/combat/src/weapons/mod.rs`). Avec `--ignore`, retire en plus les paires `Nom=valeur` dont le dernier segment du nom (après `::`) est dans la liste — sur les lignes de ressource et d'entité (une ligne d'entité empile un tracer par composant présent, `--ignore` ne retire que ceux nommés). Lit les deux fichiers en flux (une frame à la fois) : un dump complet de 1500 frames pèse facilement plusieurs centaines de Mo. Affiche la première frame qui diffère et les lignes en cause ; code de sortie 1 si différent, 0 si identique.

**Méthode** : dumper les mêmes scénarios sur la référence (avant le chantier, ex. `main` dans un `git worktree add --detach`) et sur la branche, avec la même variable d'environnement, puis comparer en ignorant le(s) nouveau(x) composant(s)/ressource(s). Aucune différence : le chantier n'a changé que la présence du nouveau composant, jamais une valeur de jeu — on peut blesser en confiance. Une différence : une valeur de stat (ou autre) ne correspond pas exactement à la constante qu'elle remplace ; corriger avant de blesser.

### Exclusions assumées du checksum

Deux champs de test (testbed) sont délibérément hors du checksum GGRS — les inclure déplacerait les traces de référence sans changer aucun comportement de jeu (D5, m0-v10) :

- **`HitCount`** (`crates/game/src/character/health/mod.rs`, T2.9) : compteur de coups reçus, posé à la création uniquement si `CharacterConfig::counts_hits` (`crates/game/src/character/create.rs`, faux par défaut — aucun personnage zombie/joueur ne le pose ; seuls `target` (testbed) et `cible` (throne) le déclarent, `crates/scenario/src/generate.rs`). Enregistré via `rollback_and_trace_no_checksum::<HitCount>()` (`crates/game/src/character/mod.rs`) : rollback et trace, mais hors checksum. Il sert aux attentes `EntityHits` des scénarios (`crates/combat/src/weapons/expectations.rs`) ; aucune décision de jeu ne le lit.
- **`EnemyAiConfig::stationary`** (`crates/game/src/character/enemy/ai/state.rs`, T2.9) : immobilité totale d'un ennemi de testbed (`dummy`/`target`/`ally`/`civilian`), exclue du `Hash` manuel du composant. Aucun contenu zombie ne la pose ; hacher un champ de plus déplacerait le checksum de toute entité `Enemy` sans aucun changement de gameplay (vérifié empiriquement à l'époque : sans l'impl manuel, `idle.ron` diverge dès f181).

## 11. Munitions et inventaire d'armes (T2.2, chantier B7)

**Type de munition** (`sim_core::ammo::AmmoType`) : enum ouvert (`Plomb`, `Balle`, `Cartouche`, `Energie`, `Special`, `Custom(String)`), comme `StatId`. Champ RON `ammo_type` sur `WeaponConfig` (`weapons.ron`), **obligatoire** pour toute arme à distance (pas de `#[serde(default)]` : une entrée sans `ammo_type` échoue au chargement RON, rapportée par `content::lint` comme n'importe quel autre littéral invalide). Contenu `zombies`/`testbed` : `pistol` → `Plomb`, `machine_gun` → `Balle`, `shotgun` → `Cartouche` (trois types distincts aujourd'hui, donc chaque arme a sa réserve propre en pratique). Exclu du hash manuel de `WeaponConfig` (comme `test`, voir la doc du champ `ammo_type` et le `Hash` manuel de `WeaponConfig` dans `crates/combat/src/weapons/mod.rs`) : une vraie valeur de gameplay, mais dont la valeur observable vit dans `AmmoReserves` (rollback), pas dans ce champ de config statique — l'y inclure ferait dériver le checksum de tous les scénarios existants sans qu'aucun comportement ne change.

**Réserves** (`combat::inventory::AmmoReserves`, composant rollback posé sur chaque joueur par `character::player::create::create_player`) : `BTreeMap<AmmoType, u32>`, une entrée par type de munition **effectivement représenté** parmi les armes de départ (rien pour un type jamais rencontré). Initialisée à la somme, par type, de `mag_limit × mag_size` du mode par défaut de chaque arme de départ (`weapons::default_mode_ammo_contribution`) — deux armes qui déclarent le même type s'additionnent (scénario `ammo_shared_reserve`). Une arme `Magless` (fusil à pompe : pas de chargeur séparé, tout son stock est `bullet_limit`) contribue `0` : son ravitaillement ne passe jamais par la réserve, avant comme après ce chantier.

**Rechargement** (`weapons::weapon_rollback_system`) : un rechargement (bouton ou chargeur vide) exige que la réserve du type de l'arme active contienne au moins un chargeur plein (`mag_size` unités) — `WeaponModeState::can_reload(mag, reserve)`. À la fin du délai de rechargement, l'arme retire `mag_size` unités de la réserve et remplit son chargeur (`WeaponModeState::reload`), exactement comme l'ancien `mag_quantity` (un chargeur entier par rechargement, jamais un appoint partiel) — seule l'unité change (munitions plutôt que nombre de chargeurs), pour un total de munitions tirables strictement identique. Une arme `Magless` ne passe jamais ce test (`can_reload` renvoie toujours faux pour `MagBulletConfig::Magless`) : son « rechargement » entre deux tirs (fusil à pompe) reste un délai de pompe qui ne touche jamais la réserve, comme avant.

**Emplacements** (`CharacterConfig::weapon_slots: u32`, `#[serde(default)]` = 2, validé par `content::lint` : `> 0` et `starting_weapons.len() <= weapon_slots`) : nombre d'armes à distance qu'un personnage peut porter. À l'exécution, borne le **ramassage** (voir plus bas) ; la création donne toutes les `starting_weapons`, le lint refuse désormais une liste trop longue (T3.4, D7). `zombies` et `testbed` déclarent `weapon_slots: 3` sur le joueur : ses trois armes de départ étaient déjà créées, aucune n'était perdue.

**Lâcher** (entrée clavier sur nouvel appui uniquement : relâcher puis rappuyer pour une autre arme ; `INPUT_DROP_WEAPON`, touche `L`, bouton `DropWeapon` des scénarios ; `weapons::weapon_drop_system`, avant `weapon_rollback_system` dans `GgrsSchedule`) : l'arme active est retirée de `WeaponInventory.weapons`, son entité despawn (`despawn_rollback`), et une entité rollback `weapons::WeaponPickup { weapon_id, mag_ammo }` apparaît au sol à la position du joueur — `mag_ammo` est celui du **mode actif** au moment du dépôt (un seul nombre, pas un par mode). Le rechargement en cours, s'il y en avait un, est annulé (`WeaponInventory::clear_reloading`) : l'arme qui rechargeait n'existe plus.

**Ramasser** (`INPUT_INTERACTION`, `InteractionType::Weapon`, `interaction::handle_weapon_pickup_interaction`) : équipe l'arme (devient active), restaure son chargeur (`WeaponPickup::mag_ammo`, via `spawn_weapon_for_player`, paramètre `initial_mag_ammo`). En dessous de `weapon_slots`, elle s'ajoute ; à `weapon_slots` ou au-delà, elle **remplace** l'arme active, qui tombe au sol à son tour (même mécanisme, `weapons::spawn_weapon_pickup`). La réserve de munitions n'est jamais transférée : seul le chargeur voyage avec l'arme, la réserve reste attachée au joueur.

**Réutilisation par T2.3** (armes murales) : `weapons::spawn_weapon_pickup(commands, weapon, mag_ammo, position, price, id_factory)` est le point d'entrée commun — `price: Option<u32>` vaut `None` pour un lâcher et `Some(prix)` pour une arme murale (T2.3, §12).

**Scénarios de référence** (`tests/scenarios/`) : `ammo_shared_reserve.ron` (deux armes forcées sur le même type via `weapon_overrides.ammo_type`, la réserve baisse quelle que soit l'arme qui recharge), `drop_pickup_swap.ron` (lâcher, `WeaponPickups`, ramasser). `weapon_overrides` gagne `ammo_type: Option<AmmoType>` (s'applique à l'arme entière, comme `friendly_fire`). Nouvelles attentes : `AmmoReserve { handle, ammo_type, amount, at_frame }` (exact, comme `Ammo`), `WeaponPickups { min, max, at_frame }` (nombre d'entités `WeaponPickup`, tous joueurs confondus). Nouveaux `GameEvents` : `drop`/`pickup` (détectés par `scenario::events::detect_events` sur l'apparition/disparition d'un `GgrsNetId` de `WeaponPickup`).

## 12. Monnaie et achats (T2.3, chantier C5 v1)

**Monnaie** (`run::currency::Currency`, crate minimal `crates/run`) : composant rollback `u32` posé sur chaque joueur à sa création (`character::player::create::create_player`), valeur de départ `CharacterConfig::starting_currency` (défaut 500, `zombies` le déclare explicitement dans `player_config.ron`). API saturante : `can_afford`/`spend` (refuse sans rien changer si le solde est insuffisant)/`earn`. Chaque variation passe par `FrameEvents<run::currency::CurrencyEvent>` (`{ handle, delta, reason }`, `delta > 0` = gain, `< 0` = achat, `== 0` = achat refusé faute de solde) — lu par le HUD (source `currency`), par `scenario::events::detect_events` (`GameEvents` : `points`/`purchase`/`purchase_refused`, classés par le signe de `delta`) et par T2.5 (power-ups).

**Points** (`game::economy`) : détectés au plus près de l'information disponible (kill dans `character::health::rollback_apply_accumulated_damage`, au moment où `Death` est posée sur une entité qui n'est pas un joueur avec `HitBy::Player` en dernier coup ; coup au but dans `rollback_resolve_damage_events`, à chaque `DamageEvent` résolu d'un joueur vers une cible qui n'en est pas un ; réparation de fenêtre dans `interaction::handle_window_repair`), poussés dans `FrameEvents<economy::PointsCredit>` et résolus une seule fois par `economy::award_points_system` (`RollbackSystemSet::Run`, en fin de frame) qui lit `economy.ron` et crédite `Currency`. Réparation plafonnable par joueur et par vague (`EconomyConfig::repair_points_cap_per_wave`, optionnel — `economy::RepairPointsTracking`, remise à zéro au changement de vague).

**Portes payantes** : `handle_door_interaction` débite `DoorConfig::cost` (existant, calculé par le générateur selon la profondeur de la salle — voir `BasicMapGeneration::get_doors`, 750 + 250×profondeur) avant d'ouvrir ; `cost <= 0` reste gratuit (comportement inchangé). Solde insuffisant : refus silencieux, porte fermée, `CurrencyEvent { delta: 0 }`.

**Armes murales** (`WeaponLocation`, §1) : `interaction::handle_weapon_pickup_interaction` distingue une arme murale (`weapons::WeaponPickup::price: Some(...)`) d'une arme au sol ordinaire (`None`, T2.2) — une arme murale n'est jamais consommée au ramassage (le mural reste achetable) et exige `Currency >= price`, ou `ResolvedEconomyConfig::refill_price(price)` (ratio `refill_price_ratio`, défaut 0.5×) si le joueur possède déjà une arme du même id, auquel cas l'interaction recharge la réserve de munitions du type au lieu d'équiper une seconde copie. Anti-rebond obligatoire (`WeaponPickup::can_buy_after_frame`, comme `WindowHealth::can_repair_after_frame`) : sans lui, maintenir Interaction facturerait le joueur à chaque frame.

**Perks** (`SodaLocation`, §1, kind de contenu `Perk`, `games/<jeu>/assets/economy/perks.ron`) : table `{ "id": (name: "...", price: N, modifiers: [(stat: ..., op: ..., value: "...")]) }` (fichier RON, newtype sur la table — parenthèse ouvrante/fermante autour de tout le fichier, comme `weapons.ron`). Achat = interaction (`InteractionType::Perk`) sur une entité `economy::PerkMachine` ; un seul achat par perk et par joueur (`run::perks::Perks`, `BTreeSet<String>`, vérifié **avant** de débiter — déjà possédé n'émet ni achat ni refus). Modificateurs posés permanents (`until: None`, `sim_core::modifier::ModifierSource::Named("perk:<id>")`) via `Modifiers::push_from`. Un perk qui relève `MaxHealth` (ex. Juggernog, `Mul "2.0"`) relève aussi `Health.current` de la même différence au moment de l'achat (`sync_health_from_stats`, `RollbackSystemSet::Status`, ne fait que plafonner `current` à la baisse, jamais à la hausse — voir sa doc) : décision propre à l'achat d'un perk (un statut temporaire qui relèverait `MaxHealth` puis expirerait ne doit pas, lui, soigner le joueur).

**Économie RON** (kind de contenu `Economy`, `games/<jeu>/assets/economy/economy.ron`, un seul fichier) : `kill_points` (défaut 60), `hit_points` (défaut 10), `repair_points` (défaut 10), `nuke_points` (défaut 400, D17 : crédité à chaque joueur vivant au ramassage d'un nuke), `repair_points_cap_per_wave` (`Option<u32>`, défaut aucun plafond), `refill_price_ratio` (`Fixed` en chaîne, défaut `"0.5"`). Chargé par le registre comme `Wave` (id = nom de fichier). Lint (`content::lint::lint_perks`, `lint_economy`) : `price > 0`, au moins un modificateur, `value > 0` pour un `op: Mul`, id unique dans `perks.ron` ; `refill_price_ratio` dans [0, 1] ; un `StatId` inconnu dans `perks.ron` échoue déjà au chargement RON (pas de variante fourre-tout implicite). Voir le tableau des règles, §3.

**Scénarios de référence** (`tests/scenarios/`) : `buy_door`, `buy_wall_weapon`, `buy_perk` (sur `games/zombies/assets/exemples/test_map_shop.ldtk`, copie de `test_map.ldtk` avec une `WeaponLocation`/`SodaLocation` dans la salle de départ — `test_map.ldtk` n'en avait pas avant T2.6, qui en a posé quatre de chaque et re-blessé toutes les traces de cette carte), `points_on_kill` et `shop_tour` (sur `test_map.ldtk`). Nouvelles attentes de scénario : `Expectation::Currency { handle, min, max, at_frame }`, `Expectation::Stat { handle, stat, value, at_frame }` (valeur résolue, comme `stats::StatReader`) ; `PlayerScript::currency: Option<u32>` (solde de départ, comme `weapon`).

## 13. État de run et mode `Waves` (T2.4, chantier F1)

**`Run`** (`run::run::Run`, crate `run`) : ressource rollback (checksum + trace, `RunPlugin`) qui représente la partie en cours — `seed: u32` (graine, dupliquée de `RunSeed` pour que `Run` seule décrive la partie), `mode: RunMode` (`Waves { config }` = le système de vagues actuel, `Sandbox` = aucune condition de fin hors défaite ; `Floors { config }` = séquence de niveaux, T1.8, §17 ; `Campaign` viendra en M2), `step: RunStep` (`Lobby` — valeur par défaut de l'enum, jamais celle d'un `Run` réellement inséré, voir plus bas —, `Playing { since_frame }`, `Ended { at_frame, outcome: RunEnd }`), `players: Vec<usize>` (handles GGRS triés), `flags: BTreeSet<String>` (ouvert pour de futurs modes, vide aujourd'hui) et `summary: Option<RunSummary>`. Remplace `combat::downed::RunOutcome` (T1.3), qui disparaît : la défaite est maintenant une valeur de `RunStep`, pas une ressource à part. **Décision** : `RunSummary` est un champ de `Run`, pas une ressource rollback séparée — un seul enregistrement, pas de valeur sentinelle à inventer pour « pas encore de résumé » (`RunEnd` n'a pas de variante neutre).

**Création** : `Run::new(seed, mode, players, since_frame)` construit toujours directement `Playing` (jamais `Lobby` dans ce chantier : réservé à un futur état où `Run` existerait déjà pendant un lobby interactif, M1/M2). Appelée par `game::jjrs::{local, p2p}` (`system_after_map_loaded_local`/`system_after_map_loaded`, `OnEnter(AppState::GameStarting)`), au même instant que `RunSeed`/`RngStreams` (T1.6) — avant que `GgrsSchedule` ne tourne pour la première fois (aucune session GGRS encore présente). `players` vient de `GgrsSessionBuilding.players[].handle`. `mode` vient de `game::run_state::resolve_run_mode(manifest, registry)` : `entry.mode` du manifeste (`content::manifest::EntryMode::Waves`/`Sandbox`/`Floors`, enum fermé comme `sim_core::damage::FriendlyFire`) s'il est présent, sinon `Waves` si le jeu déclare un dossier de contenu `Wave`, sinon `Sandbox` — `content::lint` refuse `entry.mode: Waves` sans dossier `Wave` déclaré (`lint_entry_point`), mais pas l'absence d'`entry.mode` avec un dossier `Wave` présent (c'est le chemin par défaut).

**Syntaxe RON du mode optionnel** (T3.4, D16) : `entry: (..., mode: Waves)`, `mode: Sandbox` ou `mode: Floors`. Le chargeur de manifeste active `implicit_some` sans directive en tête du fichier ; `Some(Waves)`/`Some(Sandbox)` et `None` restent acceptés. Les trois jeux déclarent leur mode explicitement (zombies `Waves`, testbed `Sandbox`, throne `Floors`). Sans ce champ, le mode est résolu comme décrit ci-dessus.

**Fin de partie** : deux voies, jamais concurrentes (`Run::is_playing()` gardé par les deux avant d'écrire `step`, une seule issue jamais réécrite).
- **Défaite** (universelle, tout mode confondu, T1.3) : `character::health::rollback_check_defeat` (`RollbackSystemSet::DeathManagement`, inchangé sinon) écrit `Ended { outcome: RunEnd::Defeat, .. }` dès que tous les joueurs en jeu sont à terre ou morts.
- **Victoire** (propre au mode, `run::modes::RunModeRules`, trait implémenté pour `RunMode`) : `game::run_state::check_run_victory_system` (`RollbackSystemSet::Run`, après `award_points_system`) appelle `run.mode.check_victory(&ctx)` avec un `RunContext` construit depuis `WaveState`/`ResolvedBalance::waves.max_wave` (et `FloorState` en `Floors`). Pour `Waves` : victoire si `WaveConfig::max_wave` (`Option<u32>`, `#[serde(default)]`, absent = jamais de victoire, comme avant ce chantier) est déclaré et atteint. `Sandbox` ne gagne jamais.

`game::run_state::finalize_run_summary_system` (`RollbackSystemSet::Run`, après les deux systèmes ci-dessus) calcule `Run.summary` une seule fois, dès que `step` devient `Ended` et que `summary` est encore `None` : `RunModeRules::summarize` construit `RunSummary { wave_reached, kills, points_total, frames, outcome }` (`wave_reached`/`kills` viennent de `WaveState` ; `points_total` = somme des soldes `Currency` de tous les joueurs à cet instant — pas un total cumulé des gains, qui n'existe nulle part ailleurs dans le jeu ; `frames` = `Run.step`'s `at_frame`).

**`waves/systems.rs`** : logique inchangée ; les systèmes de vagues ne lisent pas `Run.step`. Le spawn s’arrête quand il ne reste plus de joueur (`player_positions.is_empty()`), mais un run terminé avec des joueurs encore présents ne suspend pas les vagues. Cette limitation est conservée pour ne pas changer le gameplay ni les traces de ce chantier.

**Écran de fin** (`game::ui::game_over`) : dérivé de `Run.step` (pas de `RunOutcome`), affiche le résumé lisible une fois `Run.summary` disponible (`finalize_run_summary_system` tourne après la détection de fin, dans la même frame de simulation).

**Relance sans relancer le binaire** (`game::run_state::RunRequest`, hors rollback — ni checksum ni trace, comme `NextState`) : `Restart` (même config : même carte, même graine, mêmes joueurs ; en ligne, graine dérivée `restart_seed`, §33) ou `ToLobby` (retour `LobbyLocal`/`LobbyOnline` selon `OnlineState`), posée par le bouton de fin de partie ou la touche `R`. `apply_run_request_system` (`Update`, hors `GgrsSchedule`) consomme la commande et change d'`AppState` ; la destruction proprement dite se déclenche sur `OnExit(AppState::InGame)` (quelle que soit la cause), dans deux systèmes indépendants (aucun appel direct entre crates) :
- `game::run_state::cleanup_rollback_world_system` : détruit (`try_despawn`, silencieux sur une entité déjà partie — un despawn racine est récursif, voir sa doc) toute entité `Rollback` (joueurs, ennemis, balles, armes, murs, portes, machines à perk...), retire `Session<PeerConfig>` et `Run`, remet à zéro `FrameCount`, `GgrsNetIdFactory`, `WaveState`, `FloorState`, `run::Clock`, `FlowFieldCache`, `RepairPointsTracking`, `CollisionGrids`, `RollbackOrdered` (ressources rollback globales qu'aucun autre système ne réinitialise).
- `map_ldtk::game::plugin::despawn_ldtk_world_on_exit_ingame` : détruit l'entité racine LDtk (`LdtkProjectHandle`) et donc tout son arbre (niveaux, calques, tuiles, et toute entité LDtk encore dessous — portes, fenêtres, spawners, `WeaponLocation`/`SodaLocation`...). Vit dans `map_ldtk` (pas `game`) parce que `game` ne dépend pas de `bevy_ecs_ldtk`.

Ressources GGRS internes (`RollbackFrameCount`, `ConfirmedFrameCount`, `MaxPredictionWindow`, `LocalPlayers`) : pas touchées explicitement, `bevy_ggrs::schedule_systems::run_ggrs_schedules` les réinitialise de lui-même dès qu'aucune `Session` n'est présente. **Volontairement pas détruites** : `RunSeed`, `RngStreams`, `MapGenerationConfig` (via `LdtkGameMap`, jamais retirée de l'app), `GggrsSessionConfiguration`/`GgrsSessionBuilding` — c'est justement ce qui permet à `Restart` de rejouer « la même configuration » sans reconstruire un lobby : `jjrs::{local, p2p}` les écrase simplement des mêmes valeurs au prochain `OnEnter(AppState::GameStarting)`. `LdtkMapEntityLoadingRegistry` (état interne du chargement de carte, `map_ldtk`) est remise à zéro sur `OnEnter(AppState::GameLoading)` plutôt que sur `OnExit(InGame)` : sinon `wait_for_all_map_rollback_entity` la voit déjà `loading_complete` (partie précédente) et ne relit jamais les entités de la nouvelle carte.

**Restart en p2p** : livré depuis (D14, §33) pour `--matchbox` ; seul `--allumette` reste redirigé vers `ToLobby` par `apply_run_request_system` (averti par un `warn!`).

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
`RefillAmmoOf(munition)` (D40) : `RefillAmmo` restreint à une munition (armes et réserve de cette
munition ; un rechargement en cours n'est annulé que si l'arme active est de cette munition) ;
lint : munition déclarée par au moins une arme ; refusée dans un effet, comme `RefillAmmo`.
`Action::as_modifier(frame, source)` est l'« applicateur déterministe » pur (sans ECS,
testé unitairement) pour les deux variantes modifier-based ; les trois autres sont résolues
directement par `game::powerups` (accès à `AmmoReserves`/`WindowHealth`/`Team`, inconnus
d'`effects`).

Depuis T1.10, le même `Action` sert aux **effets** (`Effect { on, if, do }` exécutés, nouvelles
actions `Modifier`, `Heal`, `SpawnPattern`, `GaugeAdd`) : voir §27.

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
`effects::Action`, pas un miroir — comme `StatId` pour `perks.ron`), rapportée
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
T1.18 : les sources du joueur passent par `game::ui::hud_model` (`HudSnapshot`) et la liste
des sources est fermée et lintée (voir §32).

Champs optionnels à la racine de `hud.ron`, à côté de `widgets` : `font`, `icons`, `names`,
`reference_size: Some((800.0, 540.0))`. Ce dernier ajuste `UiScale` aux petits viewports
sans agrandir les grands, et masque ce HUD pendant mutation/fin de run. Sans ce champ,
les anciens jeux conservent leur mise en page. Par widget : `background`,
`compact: true` (défaut false) : arme lisible sans underscores, munitions `chargeur / réserve`,
effets temporaires limités à deux lignes puis `+N effets`. Le nom de l'arme indique
`· recharge` uniquement pendant le rechargement. Le modèle complet reste accessible
aux scénarios/outils ; les barres ont un fond fixe sous le remplissage.

Les overlays techniques (frame/FPS, caméra, arme, inspecteur natif) sont optionnels
via `ALACOD_DEBUG_UI=1`. La feature native ne les active plus à elle seule. Le debug de
vagues de zombies suit ce défaut ; F3 conserve sa bascule explicite.
```ron
(
    // Police de tous les textes du HUD, relative à `assets/`. La police par défaut de Bevy
    // n'a ni accents (« À TERRE », « possédé ») ni tiret long : les trois jeux déclarent celle-ci.
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
- M2-T18 : `Icons(source: "passives")` lit les objets passifs (ordre de ramassage,
  doublons conservés), avec les mêmes entrées `icons` et le même repli gris/initiale.
- `Map(source: "minimap")` dessine les rectangles des salles dans `size` : même échelle
  sur les deux axes, cadrage stable de l'étage, y monde inversé vers l'UI. Salle courante :
  bordure or ; verrouillée : rouge et `×` ; nettoyée : verte ; boss : violet et `B`.
  Les salles dormantes restent des silhouettes. Voir §56 pour la limite d'exploration.
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
`ExpireAction::Spawn(Pattern)` (puis `ExpireAction::DestroyTerrain { radius }`, T1.6, §21).

**Contenu** : un mode de tir déclare `projectile:` (absent = balle ordinaire, exactement
comme avant, sans composant `Projectile`) ; l'arme déclare la table `projectiles` des
projectiles que ses patterns font naître (une table **par arme** en v1, résolue au tir et
portée par la balle ; T1.2 l'a gardée par arme : un émetteur tire depuis la table de son arme, §20).
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
murs ; `apply_projectile_on_hit_system` ; `status_tick_system` (T1.3, §19) ;
`projectile_wall_terrain_system` (T1.6, §21) ; `projectile_expire_system` : `Lifetime`,
`on_expire`, `despawn_rollback` ; `projectile_steering_system` : `Gravity` puis `Homing`,
appliqués au déplacement suivant) → `CollisionDamage` (les `DamageEvent` des projectiles).
`apply_projectile_on_hit_system` est dans `Projectiles` et non `Effects` : `Effects` contient
déjà `game::powerups::apply_powerup_actions_system`, qui écrit aussi `Modifiers` (ordre
ambigu refusé par `GgrsSchedule`).

**`on_hit: [Action]`** : actions de `effects::Action` posées sur **le personnage touché**
(s'il a des `Modifiers`), seulement les actions à modificateur (`TimedModifier`,
`CurrencyMultiplier`), plus `ApplyStatus` (T1.3, §19) et `DestroyTerrain` (T1.6, §21, qui ne
creuse que sur un mur) ; les autres (`RefillAmmo`, `RepairAllWindows`, `KillAllWaveEnemies`)
sont des effets globaux de power-up, refusés par le lint. Source du modificateur :
`projectile:<id>:<rang>` ; un nouveau coup du même projectile **rafraîchit** le
modificateur au lieu de l'empiler.

**`on_expire: [Spawn(pattern)]`** : joué à **toute** fin du projectile (durée de vie,
portée, mur sans rebond, perforation épuisée) au point de fin. Patterns instantanés
seulement, sans aléa : `Aimed` (éventail de `count` sur `spread` radians, centré sur le
personnage le plus proche, sinon sur la direction du projectile), `Spread` (même éventail,
centré sur la direction du projectile), `Ring` (`count` directions régulières à partir de
la direction du projectile — axe +x à l'arrêt —, vitesse du pattern, `every` ignoré : une
seule salve), `Sequence` de ceux-ci, `Named` (T1.2) ; `on_expire` accepte aussi `DestroyTerrain { radius }`
(T1.6, §21). `Telegraph`/`Wait` et `Scatter` (aléatoire) sont réservés aux émetteurs (T1.2)
et refusés par le lint. Les projectiles nés héritent du tireur (source, équipe, tags, tir
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


**Dispersion du tir** (D51, m1-d51-dispersion-ignoree) : `spread` d'un mode de tir
(`FiringModeConfig`) est en **radians, pleine largeur** : chaque balle d'un tir simple (tout mode
sauf `Shotgun`) part à `(r − ½) × spread` de la direction visée, `r` tiré dans le flux RNG
`weapons` (un tirage par balle), donc dans `[−spread/2, spread/2]` ; `spread` = 0 : tir exact
(`combat::weapons::single_shot_direction`). `Shotgun` garde son `spread_angle` (même formule par
plomb). Lint : `0 <= spread <= π` (fixture `weapon_spread_out_of_range`). Avant D51, l'angle
valait `(r − ½) × 1` (±0,5 rad) pour **toute** arme simple : `spread` n'était jamais appliqué.

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
| un niveau `cave:<id>` désigne une caverne chargée (kind `Cave`, T1.6, §21) | `BrokenReference` | `floors_unknown_cave` |
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
Restart p2p : comme les autres modes (§33). `alacod-sim --until-floor <n>` (T1.14, §24 v1)
arrête une graine au n-ième passage de portail.

## 18. Équilibrage par joueurs (F5)

**Décision (D25, le 2026-10-03) : F5 appartient à M0** — le plan §6 liste F5 dans le jalon M0
(`zombies` complet). Tâche dédiée : `docs/taches/m0-v11-f5-equilibrage-joueurs.md`.

Les champs numériques des **vagues** (`games/zombies/assets/waves/wave_config.ron`), des **prix**
(perks `assets/economy/perks.ron`, `assets/economy/economy.ron` ; les prix d'armes murales sont des champs LDtk, voir plus bas) et de la
**santé** des ennemis (`ZombieShooter/Sprites/Zombie/*_config.ron`) acceptent un littéral **ou** une
expression `content::expr` en chaîne RON (ex. `health: "120.0 + (players - 1) * 30"`). Règles :

- **Contexte** : l'identifiant `players` (nombre de joueurs de la run) est disponible partout où
  une expression est acceptée. C'est le seul identifiant, hors `difficulty.ron` (T1.9 : `floor`, `seconds`, `minutes`, `floor_seconds`, `floor_minutes`, §23).
- **Timing** : l'expression est évaluée **une seule fois, au démarrage de la run** (le nombre de
  joueurs de la session est connu : local `PlayersCount` (`--number-player`, variable `NUMBER_PLAYER` du Makefile), p2p `max_player`, scénario = ses
  joueurs), en valeurs concrètes. **Jamais d'`Expr` dans l'état rollback** : seules les valeurs
  évaluées entrent dans la simulation (déterminisme : mêmes configs + même nombre de joueurs
  partout ⇒ mêmes valeurs). Exception encadrée (T1.9, §23) : l'expression de difficulté est
  réévaluée **en simulation** chaque seconde, depuis l'asset immuable (`game::clock::
  DifficultyConfig`, hors rollback) ; seul son résultat `Fixed` entre dans l'état rollback
  (`run::Clock::difficulty`).
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

### D61 : armure et bouclier

**Armure** (`StatId::Armor`, lue par `game::character::health::rollback_resolve_damage_events`, listée
dans `sim_core::stats::stat_reader`). Un dégât reçu devient `dégât × 100 / (100 + armure)`
(`combat::damage::after_armor`, formule de League of Legends : 100 d'armure = moitié des dégâts), en
`Fixed`, après `resolve_damage` (équipe, résistances, difficulté). Armure `<= 0` ou absente : le montant
est rendu tel quel (identité exacte, aucune trace ne bouge). `Armor` n'a pas de base dans `Stats` : un
modificateur (`objets/plaque`, `Modifier { stat: Armor, .. }`, perk, mutation) s'applique sur 0.
`DamageKind::True` ignore l'armure **et** le bouclier.

**Bouclier** (`combat::actors::Shield(u32)`), distinct de l'armure : chaque point absorbe **un coup
entier** (le dégât est annulé, le point disparaît, `SHIELD_IFRAMES` = 15 frames d'invulnérabilité
`Health.invulnerable_until_frame` pour qu'une rafale n'en consomme pas plusieurs). Composant
**neutre** (`rollback_and_trace_neutral`), posé au premier gain par `GainShield(n)` (variante **en fin
d'enum** d'`Action` : effet, objet `OnUse`/consommable, synergie ; refusée dans un power-up, `n = 0`
refusé par le lint). Le gain passe par `combat::actors::queue_gain_shield` (commande différée : ajoute à
un `Shield` existant). Ordre au résolveur : **bouclier d'abord, puis armure** sur ce qui passe. Trace :
`ggrs{f= shield_absorb target= left=}`.

**Contenu** (`testbed`) : `objets/plaque` (passif, +100 d'armure), `objets/egide` (consommable,
`GainShield(1)` au ramassage). Scénarios : `armor_reduces_damage`, `armor_true_damage` (chute en fosse
= dégât `True`, +100 d'armure par `modifiers`), `shield_absorbs_hit`, `shield_then_armor`. Lint :
fixture `shield_zero`. HUD : source `shield` laissée à M2-T18.

## 19. Statuts (T1.3, chantier B3)

Code : `combat::status` (règles pures, `Statuses`, `StatusLibrary`), `combat::projectile`
(`apply_projectile_on_hit_system`, `status_tick_system`, `status_motion_system`),
`game::statuses` (bibliothèque depuis le registre), `content::lint::{lint_statuses,
lint_weapon_statuses}`.

**Contenu** : kind `Status` (`statuses/<id>.ron`), l'id nomme le statut, `kind` son genre
(`StatusDef`) :
```ron
// games/testbed/assets/statuses/brulure.ron — game.ron : (path: "statuses", kind: "Status")
(kind: Burn, frames: 120, damage: "4", period: 30)
// lenteur : (kind: Slow, frames: 120, factor: "0.5") ; etourdi : (kind: Stun, frames: 60)
// gel : (kind: Freeze, frames: 90)
```
Lint : `frames > 0` ; `Burn` : `damage > 0`, `period > 0` ; `Slow` : `factor` dans `]0, 1]`
(fixtures `status_out_of_range`, `status_unknown`).

**Pose** : `Action::ApplyStatus { status, stacks }` (variant ajouté en fin d'`effects::Action` par T1.3, suivi depuis de `RefillAmmoOf`, D40) dans
l'`on_hit` d'un projectile composable, exécutée sur le personnage touché par
`apply_projectile_on_hit_system` (`RollbackSystemSet::Projectiles`). Statut inconnu ou `stacks`
0 : lint. Dans un effet (T1.10) ou un power-up : lint (`Unsupported` / refus) — v1 n'a pas de
cible pour un statut hors d'un coup. **Suite** : `OnDamageTaken` connaît sa source, un effet
« riposte » pourrait y poser un statut (v2) ; `OnKill` n'a pas de cible vivante.

**État** : composant `Statuses` (rollback + checksum, enregistrement de T1.0a **gardé tel quel** :
les traces de référence l'incluent déjà sans porteur ; la variante neutre, essayée, changeait le
checksum de toutes les frames de tous les scénarios). Un personnage n'en porte que si un statut
lui a été posé, le composant est retiré quand le dernier expire : aucune trace existante, zombies
compris, ne change. Entrée `StatusEntry { status, stacks, expires_at_frame, id, source,
source_team, next_tick_frame }`, dans l'ordre de pose.

**Règles** (pures, testées) :
- `Burn` : un tick de `damage` toutes les `period` frames depuis la pose, tant que le tick tombe
  au plus tard à l'expiration ; chaque tick est un `DamageEvent` de genre `Fire` (tags `status`,
  `<id>`), **crédité à la source** (kill compris), résolu dans la même frame par le résolveur
  unique (immunités, résistances, équipes ; source disparue : équipe relevée à la pose).
  Réapplication : durée restante + durée de base, plafonnée à 2 × la base ; `stacks` =
  applications actives, au plus 2.
- `Slow` : modificateurs `MoveSpeed` et `EnemyMoveSpeed` `Mul factor`, source
  `Named("status:<id>")`, jusqu'à l'expiration ; réapplication : rafraîchie.
- `Stun` : ni déplacement ni tir ; réapplication : rafraîchie. Joueur : inputs ignorés **dans la
  simulation** (`apply_inputs`, tir, mêlée ; visée gardée), jamais à la lecture des inputs (un
  rejeu `Scripted` doit rester identique). Ennemi : aucune règle de comportement retenue (une
  charge est interrompue), pas de déplacement (`move_enemies`), pas d'attaque, émetteur suspendu.
- `Freeze` : `Stun`, plus vitesse **et recul** remis à zéro chaque frame (`status_motion_system`,
  `Movement`, avant `move_characters`).

**Ordre** : pose (`Projectiles`, après les coups de la frame) → `status_tick_system` (même set,
juste après : ticks de `Burn` puis expiration, `expires_at_frame <= frame`, porteurs par
`GgrsNetId`) → résolution des dégâts (`CollisionDamage`, même frame). Un statut posé à la frame
`f` agit sur le déplacement à partir de `f + 1` (le set `Movement` est passé).

**Visuel dérivé** (`game::feedback`, hors simulation) : les calques d'un porteur prennent la
teinte de son statut le plus contraignant (gel, étourdi, brûlure, lenteur), relue chaque frame.

**Attentes** : `HasStatus(entity, status, present: true, at_frame)`, `StatusStacks(entity,
status, stacks, at_frame)` (`entity` : `NetId(n)` ou `Target`). **Testbed** : quatre statuts,
quatre armes `status_burn`, `status_slow`, `status_stun`, `status_freeze` dont le `test:` utilise
les deux attentes (scénarios générés `weapon_status_*`) ; scénarios écrits `status_slow_enemy`,
`status_freeze_enemy`.

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
Sans règle `Shoot` (`EnemyAiConfig::ranged` à `None` ; le champ RON `ai.ranged` n'existe plus
depuis T1.4, §22), rien ne change (hash de `EnemyAiConfig` : `ranged` n'est haché que s'il est
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

## 21. Terrain destructible et cavernes (T1.0b + T1.6, chantier E3)

Code : crate `world` (grille, générateur, destruction, `WorldPlugin`),
`crates/map_ldtk/src/generation/cave.rs` (niveau LDtk), `crates/map_ldtk/src/game/cave.rs`
(branchement ECS), `crates/map_ldtk/src/loader/mod.rs` (`resolve_map_config`, `CaveSlots`), `crates/game/src/cave_assets.rs`
(source d'assets `cave://`).

**Contenu.** Kind de dossier `Cave` (`content::registry::CaveEntry`) : un fichier RON par
caverne, id = nom de fichier sans extension, champs de `world::CaveConfig` ; le dossier contient
aussi le gabarit LDtk `gabarit.ldtk` (définitions de couches et d'entités ; ses tilesets sont
relatifs au dossier des cavernes, `../testbed/atlas/...` dans le testbed : un chemin qui ne se
résout pas bloque le chargement du niveau).
```ron
// games/testbed/assets/caves/petite.ron  — game.ron : (path: "caves", kind: "Cave")
(width: 48, height: 32, fill_ratio: "0.45", iterations: 4, birth: 5, survive: 4,
 min_floor_ratio: "0.3", enemy_spawns: 4)
```

**Désignation : `cave:<id>`** partout où une carte LDtk est attendue : `entry.start_map`,
`Scenario.map`, `--map` d'`alacod-sim`, `levels` d'une séquence `Floors` (ex.
`floors/caverne.ron` du testbed). Au chargement (`setup_generated_map`), la désignation devient
le chemin propre à la caverne `cave://<dossier>/<id>.ldtk` (servi avec le gabarit du dossier par
`game::cave_assets` : une caverne = un asset, sinon l'`AssetServer` rendait la première caverne
à tous les étages, m1-integration-scenarios) + `MapGenerationMode::Cave(config)` ; la ressource `MapGenerationConfig` garde
`cave:<id>` (un enregistrement se rejoue tel quel). La graine est `map_seed`, comme `Basic`.

**Lint** (`content::lint::lint_caves`, `lint_floors`, `lint_entry_point`) : au moins 16 × 16
cases, `fill_ratio` dans `[0, 1]`, `min_floor_ratio` dans `[0, 0.9]`, `birth`/`survive` ≤ 8,
gabarit présent (`BrokenReference`) ; `cave:<id>` inconnu dans `levels` ou `start_map` :
`BrokenReference`. D36 : dans `levels` d'une séquence `Floors`, une caverne **sans ennemi**
(`characters` vide ou `enemy_spawns = 0`) est refusée (`OutOfRange`) : le portail d'un niveau
`Floors` s'ouvre quand il ne reste aucun ennemi, donc aussitôt (et en boucle si c'est le dernier
niveau). Un passage voulu se déclare `transit: true` dans le RON de la caverne (champ de
`CaveConfig`, lint seul, non sérialisé quand il est faux) : `caves/petite.ron` du testbed
(séquence `floors/caverne.ron`). Les cartes LDtk d'une séquence ne sont pas contrôlées (il
faudrait lire leurs entités `CharacterSpawn`).

**Grille.** `world::CellGrid { width, height, cells }`, `CellKind { Floor, Wall, Rock }` :
`Wall` indestructible (bordure), `Rock` destructible. Cases de **16** unités (`GRID_CELL_SIZE`
de la navigation), origine (0, 0), **+y vers le haut** comme le monde et `GridPos` (la rangée 0
est en bas, au contraire des rangées LDtk). Ressource rollback + checksum + trace, enregistrée
en `rollback_and_trace_resource_neutral` : vide (défaut) hors caverne, contribution 0, traces
des autres cartes inchangées. Debug compact (une chaîne par rangée : `.` sol, `#` mur,
`r` roche). `world::Destructible` marque l'entité de niveau LDtk d'une caverne (pas une entité
par case). `RollbackSystemSet::World` : entre `Projectiles` et `CollisionDamage`, donc avant la
navigation (`EnemyAI`), qui voit une destruction dans la frame.

**Générateur** (`world::cave::generate(seed, &config)`, pur, `RollbackRng`) :
1. bordure `Wall`, intérieur `Rock` avec la probabilité `fill_ratio`, sinon `Floor` ;
2. `iterations` passes (voisinage de Moore, hors grille = solide) : `Floor` → `Rock` si au moins
   `birth` voisins solides ; `Rock` reste `Rock` si au moins `survive`, sinon `Floor` ;
3. connexité : la plus grande composante 4-connexe de sol est gardée (égalité : la première au
   balayage depuis le bas), les autres deviennent `Rock` ;
4. ratio de sol hors de `[min_floor_ratio, 0.9]` : nouvel essai avec la suite du même RNG (64 au
   plus). Testé sur 1 000 graines (connexité, bordure, ratio, spawns sur le sol).

**Points d'intérêt** (`world::points_of_interest`), choisis parmi les cases **dégagées** (sol
dont les 8 voisines sont du sol, `world::cave::is_open` : un corps de personnage de 20 × 20,
décalé vers le bas, y apparaît sans toucher de mur — sinon chaque déplacement serait refusé) :
`PlayerSpawn{index}` 0..3 = les plus proches du centre (distance au carré, puis y, puis x) ;
`ZombieSpawn` × `enemy_spawns` = les plus éloignées du `PlayerSpawn` 0 en distance de grille
(4-connexe), espacées d'au moins 8 cases (Tchebychev). Champ optionnel `characters: [...]`
(vide par défaut) : un `CharacterSpawn` (équipe `enemies`) par point `ZombieSpawn`, personnages
pris à tour de rôle (lint : personnages chargés) — un jeu sans vagues (testbed) peuple ainsi
une caverne (`caves/bench.ron` : six `follower`).
Dégagement des `ZombieSpawn` (D41, m1-d41-spawns-degages) : `CaveConfig::spawn_clearance` cases
(`world::is_open_within`), calculé par le registre depuis le plus grand corps **en jeu** (collider
× `scale`) de `characters` (`content::registry::spawn_clearance_for` : 1 jusqu'à 24 px du centre,
une case de plus par 16 px) ; 1 pour tout le contenu actuel (points inchangés).
Accessibilité des `ZombieSpawn` (D48, m1-d48-ennemis-hors-champ) : un point n'est retenu que si
le champ de flux du gabarit de navigation du plus grand corps de `characters` l'atteint depuis
les points des joueurs (`world::nav::nav_distances`). Les règles de passage (`blocked_for`,
`too_narrow`, coin coupé en diagonale) sont **les mêmes fonctions** que le champ de flux de
`game` (`world::nav`, appelées par `FlowFieldCache`) ; un test compare les deux sur les cavernes
de `throne`. `CaveConfig::nav_large`, calculé par le registre (collider × `scale` de plus de
20 px en largeur ou en hauteur, le seuil d'`AgentSize::Large`), choisit le gabarit. Sans cette
règle, une poche ouverte mais reliée par un couloir d'une case (petit gabarit) ou de deux cases
(grand) gardait l'ennemi hors de son champ, immobile et hors d'atteinte des bots (soft-locks des
graines 43 et 162 de `throne`). L'ordre des candidats reste la distance de grille. Lint : une
caverne peuplée dont une des graines de contrôle 1 à 5 n'a aucun point atteignable est refusée
(fixture `cave_spawns_unreachable`).

**Niveau LDtk.** Le callback du loader réécrit le niveau unique du gabarit
(`generation::cave::build_cave_ldtk`) : dimensions, IntGrid `Walls` (1 = `Wall` ou `Rock`),
entités, niveau en (0, 0) monde (`world_y = -px_hei`). Tout le reste (colliders fusionnés,
`load_intgrid_walls`, spawns, mode `Floors`) suit le chemin LDtk ordinaire. Au
`LdtkMapLoadingEvent`, `CellGrid` reçoit la même grille (emplacement 0) ; au passage d'un niveau
`Floors`, la grille du nouvel emplacement (vide pour une carte ordinaire).

**Destruction.** `effects::Action::DestroyTerrain { radius }` (rayon en unités, `Fixed` en
chaîne) : toute case `Rock` dont le centre est à **strictement** moins de `radius` de la
position de l'action devient `Floor` ; `Wall` ne change jamais. Positionnelle : réservée aux
projectiles (lint : refusée dans un power-up, `radius > 0`). Deux usages :
- `on_expire: [DestroyTerrain(radius: "40.0")]` (`combat::projectile::ExpireAction`) : au
  point de fin du projectile, quelle qu'en soit la cause (arme testbed `grenade_creuse`, sans
  rebond : elle s'arrête contre la roche et creuse) ;
- `on_hit: [DestroyTerrain(radius: "24.0")]` : au point d'impact sur un mur. Un mur touché
  émet `combat::projectile::ProjectileWallHit` (projectile, source, position au contact,
  actions `on_hit`) dans la branche `register_wall()`, **seulement** pour un projectile qui
  porte des actions `on_hit` (file neutre : vide, elle laisse les traces existantes intactes) ;
  `projectile_wall_terrain_system` en tire les demandes.

**Décision (D36) : `on_hit: [DestroyTerrain]` ne creuse que sur un mur.** Un projectile qui
touche un **personnage** émet `ProjectileHit`, dont `apply_projectile_on_hit_system` n'exécute
que `ApplyStatus` : `DestroyTerrain` y est ignoré, volontairement. Le terrain réagit au contact
du terrain ; creuser au point d'impact sur un personnage ouvrirait la roche autour de chaque
ennemi touché. Pour creuser à tout impact (mur ou personnage), utiliser `on_expire:
[DestroyTerrain]` : il part du point de fin du projectile, quelle qu'en soit la cause (un
projectile qui s'arrête sur un personnage expire là).

Les deux posent une `world::DestroyTerrainRequest` (file `FrameEvents` neutre) dans
`Projectiles` ; `apply_destroy_terrain_system` (`World`) l'applique et émet `TerrainDestroyed`
(cases creusées), moment clé `terrain` et métrique `terrain_destroyed` des scénarios.
`ExpireAction` a un `Hash` manuel : `#[derive(Hash)]` d'un enum à une seule variante n'écrit pas
le discriminant, ajouter une variante dérivée aurait déplacé le checksum de toute arme à
`on_expire` dès la frame 0 (règle générale : ajouter une variante à un enum haché qui n'en avait
qu'une change les traces).

**Murs et navigation après destruction** (`map_ldtk::game::cave`, set `World`) : quand la frame
a creusé, tous les `Wall` rollback (dans une caverne, ce sont ceux du niveau) sont détruits
(`despawn_rollback`) et recréés depuis `CellGrid` par la même fusion gloutonne
(`generate_collision_rectangles`), net ids `cave_wall_<frame>_<i>` ; les cases murées de la
navigation sont rechargées (`FlowFieldCache::reload_walls`, seule exception à « immutable after
load ») ; la signature des murs de `CollisionGrids` et `rebuild_blocked_cells` reconstruisent le
reste au pas suivant. `CellGrid`, murs et `FlowFieldCache` sont rollback : un rejeu qui remonte
avant la destruction les retrouve cohérents (test synctest
`destruction_creuse_et_reconstruit_les_murs_en_synctest`). Aucun état hors rollback ne doit
retenir « la dernière grille appliquée ». Coût nul hors destruction ; un flow field vraiment
incrémental n'est à faire que si `bench_cave` ne tient pas son budget. Rejeté : un collider par
case (des milliers d'entités rollback au checksum).

**Scénarios.** `explode_wall` (cratère observé par `CellState`), `bench_cave` (caverne `bench` :
quatre joueurs à grenades, six followers, ≥ 50 destructions, plancher de 40 fps simulés dans
`tests/budgets.ron`).

**Attente** `CellState(x, y, kind, at_frame)` : nature de la case à la frame exacte, en
coordonnées de grille ci-dessus ; échoue hors de la grille (donc toujours hors caverne).

**Présentation.** Le gabarit n'a pas de tuiles : un carré par case solide est dessiné depuis
`CellGrid` (`cave_cells_visual_system`, rendu seulement), reconstruit quand la grille change.

**Surfaces** : couche séparée `world::SurfaceGrid` (§26), indépendante de `CellGrid`.

**Limites.** Pas de surfaces de caverne (v2 de §26). (Les bots naviguent dans les cavernes
depuis m1-v3-bots-pathfinding, §24.) Dans
le testbed (pas de vagues), une caverne sans `characters` n'a pas d'ennemi : son portail
`Floors` s'ouvre aussitôt.

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
| `Flee` | santé ≤ `flee_threshold` × max | recule comme `KeepDistance` ; **pas d'attaque au corps à corps** tant que retenue (D38 : marqueur `MeleeHold`, rollback neutre, lu à la frame suivante) |
| `Strafe` | cible à moins de `Sight` | perpendiculaire à la cible, sens alterné toutes les 45 frames depuis l'entrée dans la règle |
| `Chase { profile }` | cible connue | `move_enemies` d'origine, sur le champ de flux de la **clé** de l'ennemi (D41 + D38, m1-navigation-profils-tailles) : `NavKey { profil (EnemyAiConfig::nav_profile), gabarit (Small ≤ 20 px de corps en jeu, Large au-delà : centre à une case des murs) }`, construite seulement pour les ennemis qui se déplacent ; un profil qui voit les mêmes obstacles que `GroundBreaker` sur la carte (pas de fenêtre ni de barricade) partage son champ (`FlowFieldCache::canonical`) |
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
`behavior_unknown_profile`, `behavior_shoot_unknown`, `targeting_unknown_tag` (et `ranged_*` migrées vers `Shoot`).

**Testbed** : `kiter`, `charger`, `coward`, `drifter`, une carte chacun
(`testbed/arena_ia_keep|charge|flee|wander.ldtk` : un scénario ne peut pas faire apparaître de
personnage, une arène commune les ferait interagir), scénarios `enemy_keep_distance`,
`enemy_charge`, `enemy_flee`, `enemy_wander`.

## 23. Horloges et difficulté (T1.9, chantier F2)

Code : `crates/run/src/clock.rs` (`Clock`, `ClockDef`, `due_events`, `ClockFired`,
`FloorEntered`), `crates/game/src/clock.rs` (`resolve_clocks_system`, `clock_system`,
`DifficultyReader`, `scale`), `content::expr::difficulty_context`.

**Activation** : horloges et difficulté ne tournent que si une partie les demande — champs
`clocks: Some(["arene"])` et `difficulty: Some(true)` d'un scénario (`ClocksOverride`,
`DifficultyOverride`), ou `entry.clocks`/`entry.difficulty` du manifeste pour une partie jouée.
Déclarer les kinds dans `game.ron` ne suffit pas. Sans activation, `Clock` reste à sa valeur par
défaut (checksum **neutre**), aucun événement n'est émis, la difficulté vaut 1 : **aucune trace
ne change**.

**Horloges** (kind `Clock`, `clocks/<nom>.ron`) :
```ron
(
    scope: Floor,   // ou Run
    events: [
        (id: "tic", at: Seconds("1")),
        (id: "tac", at: Frames(120)),
        (id: "renfort", at: Seconds("2"), repeat: Some(Seconds("2"))),
    ],
)
```
Échéance relative au début de la portée (`Run` : frame 0 ; `Floor` : entrée dans l'étage
courant). Un événement répété a l'id `"<id>#<n>"` (`renfort#1` à 2 s, `renfort#2` à 4 s...), un
événement simple son id. `clock_system` (`RollbackSystemSet::FrameCounter`, avant l'incrément,
donc après le passage d'étage du set `Run`) : (1) si `FloorState::index` a changé, entrée
d'étage — temps d'étage remis à zéro, ids des horloges de portée `Floor` oubliés,
`FrameEvents<FloorEntered>` ; (2) événements échus par horloge (ordre des ids) puis par
événement (ordre du fichier) : id inséré dans `Clock::fired`, `FrameEvents<ClockFired>`, log
`ggrs{f=… clock id=…}` ; (3) difficulté. **`FloorEntered` n'est émis que si les horloges ou la
difficulté sont activées.** Aucun effet ne s'y attache encore : T1.10 est livrée sans
déclencheur d'étage ni d'horloge (`OnEvent` refusé `Unsupported`, v2, §27) ; un tel
déclencheur devra activer les horloges implicitement.

**Difficulté** (kind `Difficulty`, `difficulty.ron`) : `(value: "1 + floor * 0.5 + floor_minutes * 0.5")`,
une expression `content::expr` aux identifiants `players`, `floor`, `minutes`, `seconds`
(temps de run), `floor_minutes`, `floor_seconds` (temps d'étage) — **seulement pour ce kind**
(`players` reste le seul ailleurs, §18). Réévaluée en simulation chaque seconde et à chaque
entrée d'étage, depuis l'asset immuable (`DifficultyConfig`, hors rollback) ; le résultat seul
est dans `Clock::difficulty`. Consommateurs v1 : **santé des ennemis à l'apparition** (vagues,
spawners, `CharacterSpawn` : `health_max × difficulté`, évaluée à neuf pour l'étage où ils
apparaissent — ceux d'un nouvel étage apparaissent dans la frame du passage, avant la
réévaluation) et **dégâts infligés par les ennemis** (un seul point : le résolveur unique,
`rollback_resolve_damage_events`, pour les `DamageEvent` de `source_team == Enemies`).
`game::clock::scale` ne calcule rien quand la difficulté vaut exactement 1 (test : identique au
bit près).

**Attente et événement** : `Clock(id, fired: bool, at_frame)` ; moment clé `clock`
(`scenario::events`, libellé `horloge : <id>`).

**Lint** : horloges — ids d'événements uniques sur toutes les horloges (ils partagent
`Clock::fired`), échéances croissantes dans un fichier, `repeat > 0`, `entry.clocks` vers une
horloge connue ; difficulté — identifiants hors liste, valeur ≤ 0 pour `floor ∈ {0, 3}`,
`minutes ∈ {0, 10}`, `players ∈ {1, 4}`, `entry.difficulty` sans fichier. Fixtures
`clock_duplicate_id`, `clock_unordered`, `clock_repeat_zero`, `difficulty_unknown_identifier`,
`difficulty_non_positive`.

**Testbed** : horloge `arene`, `difficulty.ron`, séquence `deux_cibles` (`floor_a` puis
`floor_cible_b`) ; scénarios `clock_events`, `clock_floor_reset` (séquence `deux_niveaux`),
`difficulty_scales`. Dette D29 : les multiplicateurs par vague du clone zombies ne sont lus par
personne.

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
initiale pouvait autoriser leur combinaison à entrer dans le coin d'un mur. Les ennemis glissent
comme les joueurs (D39) : `move_enemies` et `move_characters` partagent
`combat::collider::slide_axes`.

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

### v1 (T1.14)

**Esquive** (`prudent`, `crates/bots/src/dodge.rs`, pure en `Fixed`) : `BotView.projectiles` =
balles (`Bullet`, ordinaires ou composables) d'une équipe autre que celle des joueurs, à moins
de 320 px, les 16 plus proches, triées par `GgrsNetId` (`view::projectile_views`) ;
`BotView.body_radius` = demi-diagonale du collider du joueur. Un projectile qui **approche**
(`rel · v < 0`) menace si le point d'approche minimale de sa trajectoire linéaire dans les 30
prochaines frames est à moins de rayon du corps + taille du projectile + 8 px. Direction :
perpendiculaire à sa vitesse, du côté qui s'éloigne du point d'approche (à gauche s'il arrive
droit dessus), somme normalisée sur les menaces. Quand elle existe, elle remplace le
déplacement de `prudent` ; visée et tir vers l'ennemi le plus proche sont conservés.

**Gestion d'arme de `prudent`** (même règle que `chasseur`/`acheteur`) : recharge si le chargeur
est vide et la réserve le permet ; sinon passe à une arme utilisable (`switch_weapon`) ; ne
presse le tir d'une arme non automatique que détente relâchée (`WeaponState::is_firing` faux).
`fonceur`, `immobile`, `chasseur`, `acheteur` : inchangés.

**Complétion de niveau** : `prudent`/`fonceur` marchent vers l'ennemi le plus proche (la vue n'a
pas de limite de portée : `nearest_enemy` ne vaut `None` que s'il n'en reste aucun), puis vers
le portail ouvert ; en ligne droite jusqu'à la navigation ci-dessous. Approche du portail par `prudent` **pilotée en vitesse**
(m1-v3-bots-portail ; avant, freinage sous 48 px de T1.14) : les boutons ne donnent que le
signe de chaque axe et le corps garde son élan, si bien qu'à pleine vitesse le bot tournait
autour du portail sans entrer dans son rayon (24). Vitesse voulue = direction (la route
au-delà de 48 px, le portail en deçà) × min(120, 2 × distance) ; chaque axe est pressé selon
l'écart entre vitesse voulue et vitesse actuelle s'il dépasse 12 (`crates/bots/src/decide.rs`,
`approach_portal`). La vitesse transverse est annulée (plus d'orbite).

**Navigation de `prudent`/`fonceur`** (m1-v3-bots-pathfinding, suite de T1.14) : même calcul que
`chasseur` (`crates/bots/src/navigation.rs`, Dijkstra multi-source sur une grille de 8 px, murs,
fenêtres et portes, murs de caverne compris), dans un cache séparé (`DirectNavigation`), dérivé
hors rollback dans `ReadInputs`, **en mode `Floors` seulement** (en vagues, les ennemis
viennent aux joueurs : `enemy_visible` vaut vrai et `route` est absente, comportement de
T1.14 ; `crates/bots/src/input.rs`). `BotView` gagne `enemy_visible` (aucun `Wall`, élargi de
`SHOT_MARGIN` = 4 px, entre le joueur et l'ennemi le plus proche), `enemy_still` (face à un
ennemi immobile au-delà de `STILL_TARGET_DISTANCE` = 120 px, `prudent` s'approche par `route`
sans recul) et `route` (pas suivant vers le poste de tir de l'ennemi le plus
proche par le chemin, ou vers le point accessible le plus proche de lui ; sans ennemi, vers le
portail, rayon 16 ; composante de moins de 2 px annulée, sinon un pas d'un pixel devient une
diagonale contre un coin). Règles : l'esquive et le recul sous 180 restent prioritaires ;
ennemi caché ou au-delà de la bande de `prudent` (ou caché pour `fonceur`) → `route` ; ennemi
visible → ligne droite comme avant ; portail au-delà de 48 → `route`, puis l'approche pilotée.
Sans chemin (`route` absente), ligne droite en repli. Le champ n'est calculé que quand une règle
s'en sert. Critère à la livraison : `throne` 20/20 sur 20 graines (contre 17/20), `trois_niveaux` du
testbed conservé — mesure invalidée depuis (les trois étages chargeaient alors `niveau_1`,
m1-integration-scenarios) ; mesures actuelles : journal de `docs/taches.md`
(m1-integration-scenarios, m1-v3-bots-portail).

**`alacod-sim`** : `--until-floor <n>` (exige `--floors`) arrête la graine quand
`FloorState::index >= n` ; `--until-wave` ou `--until-floor` est obligatoire. `SimResult` gagne
`floor_frames` (frame de chaque passage de portail), `damage_taken` (somme des baisses de santé
des joueurs) et `dodges` (frames où l'esquive a remplacé le déplacement d'au moins un bot,
compteur `bots::BotStats` hors rollback). Soft-lock `Floors` : sans passage de niveau ni ennemi
en moins pendant 1 200 frames, arrêt avec `softlock` (instantané « précédent » à 600 frames).
`--log` (D49) : installe un subscriber de journaux sur stderr (`RUST_LOG`, défaut `info`, sans
horodatage ni couleur) ; sans lui, `alacod-sim` n'a aucun subscriber et `RUST_LOG` ne produit
rien. Sorties et JSON inchangés sans `--log`.
D43/D44 (m1-d43-d44-fin-de-partie) : la simulation s'arrête aussi dès que la partie est terminée
(`StopEarly::stop_when_run_ended`, `run_end` dans le JSON, colonne « fin ») ; la progression du
soft-lock compte aussi la santé totale des ennemis qui baisse et tout changement d'état d'un
joueur (`runner::FloorsProgress`).
`scripts/scenario-metrics.py` : colonnes `Niveau` et `Frames/niveau`, `Esquives`, résumé
« niveaux finis / graines ».

**Scénarios** : `bot_prudent_dodge` (arena_tir), son jumeau `bot_prudent_nododge` (inputs de
`prudent` sans esquive figés en `Scripted`), `bot_floors_three` (`trois_niveaux` = `floor_a`,
`floor_d`, `floor_c`).

**Armes des bots** (m1-v3-bots-armes, `crates/bots/src/arms.rs`, mode `Floors`, `prudent` et
`fonceur`) : chaque arme portée est notée par ses dégâts attendus par seconde sur la cible
courante, lus dans la config du mode de tir : dégâts par projectile × projectiles par tir
(plombs, rafale) × cadence × part qui touche, nuls hors portée ou sans munitions. Part qui touche
= `min(1, 2 × rayon / (distance × spread))` (angle uniforme sur la pleine largeur `spread`, §16 ;
`spread_angle` pour le fusil). Exemples (cible de rayon 12, armes de `throne`) : à 300 px,
mitraillette ≈ 43/s, lance-lames 36, laser 48 ; à 500 px, mitraillette ≈ 26 (test
`arms::tests::degats_attendus_avec_la_dispersion`). Changement d'arme : seulement pour un gain
≥ 1,5 × (ou si l'arme en main ne fait rien), pas avant 120 frames après le choix précédent
(`WeaponChoices`, hors rollback, frame de simulation, vidée à chaque entrée en partie) ;
`switch_weapon` pressé jusqu'à l'emplacement visé. Pas de tir au-delà de la portée de l'arme en
main (`BotView::fire_range`). Ramassage, sans ennemi visible à moins de 150 px : power-up à moins
de 96 px (partout à sec), arme au sol à moins de 96 px si elle bat l'arme en main à 250 px
(emplacements pleins : c'est elle qui tombe) ou si un emplacement est libre. Pas de cas « boss »
ni d'anticipation (m1-v3-bots-lead).

## 25. Variantes et élites (T1.5, chantier D2)

Code : `crates/game/src/character/variant.rs` (`VariantsConfig`, `draw_variant`,
`variant_health`, composant `Variant`), application dans `character::enemy::create::spawn_enemy`.

**Contenu** : champ optionnel `variants` d'un personnage (`CharacterConfig`) :
```ron
variants: Some((
    chance: "0.5",   // probabilité d'avoir une variante, [0, 1], défaut 1
    table: {
        "rapide": (weight: 1, modifiers: [(stat: EnemyMoveSpeed, op: Mul, value: "1.5")],
                   tags: ["rapide"], skin: Some("rapide")),
        "blinde": (modifiers: [(stat: MaxHealth, op: Mul, value: "2.0")], tags: ["champion"]),
    },
)),
```
`weight` (`u32`, défaut 1) pondère le tirage, dans l'ordre `BTreeMap` des noms ; `modifiers`
au format §9 ; `tags` en **union** avec ceux du personnage (immunités §8, `ignore` de §22) ;
`skin` (clé de `skins`) remplace `starting_skin`. Une **élite** est une variante ordinaire (tag
`champion` + modificateurs) : aucun mécanisme à part, pas de points ×2.

**Tirage** : au `spawn_enemy` de tout personnage qui déclare `variants` (vagues, spawners et
`CharacterSpawn`), un `RollbackRng` **local** de graine `fnv1a("variants") ^ run_seed ^ net_id`
(tronqués en `u32` ; `run_seed` = `RngStreams::run_seed`, la graine de carte ; `MapGenerationConfig::seed` pour un
`CharacterSpawn`, même valeur) : un tirage pour la
`chance`, un pour le poids. Indépendant de l'ordre d'apparition et du moment de chargement de
la carte ; **`RngStreams` n'est jamais touché** (y créer un flux changerait le hash de la
ressource, donc toutes les traces). Le `GgrsNetId` du personnage est alloué par `spawn_enemy`
avant `create_character` (même valeur que s'il l'était dedans : c'est sa seule allocation)
pour que le tirage et le skin soient connus à la création. Un personnage **sans** `variants`
ne tire rien et suit le chemin d'origine.

**Variante imposée** : champ LDtk optionnel `variant` sur `CharacterSpawn` (§1) : rempli,
aucun tirage, la variante est imposée ; vide ou absent, tirage normal.

**Application** (seulement si une variante est choisie) : composant rollback `Variant(nom)`,
checksum **neutre** (§20) ; `Modifiers` = ceux de la variante, source `Named("variant:<nom>")`,
permanents ; `Tags` = union ; skin à la création. **Santé** : un ennemi n'a pas de stat de base
`MaxHealth` (sa santé vient de F5, §18) : pour un personnage **à variantes** seulement,
`spawn_enemy` pose la base `MaxHealth` = santé F5 et crée le personnage à la santé résolue par
les modificateurs `MaxHealth` de la variante (`sync_health_from_stats` le maintient ensuite).
La vitesse d'un ennemi est `EnemyMoveSpeed` (`MoveSpeed` est celle des joueurs).

**Attente et événement** : `EnemyVariant(entity, variant: Some("rapide") | None, at_frame)` ;
moment clé `variant_spawn` (`scenario::events`, libellé `variante <nom> (<net id>)`).

**Lint** : `chance` hors `[0, 1]`, `weight = 0`, nom en double, `skin` absent de `skins`,
`MoveSpeed` sur un personnage à `ai` (renvoie à `EnemyMoveSpeed`), stat inconnue (erreur de
chargement), `variant` LDtk vers un personnage ou une variante inconnus (lecture des
`CharacterSpawn` des `.ldtk`). Fixtures `variant_weight_zero`, `variant_chance_out_of_range`,
`variant_skin_unknown`, `variant_move_speed_ai`, `variant_duplicate`, `variant_ldtk_unknown`.

**Testbed** : `grunt` (`Chase`, variantes `rapide` et `blinde`), `grunt_plain` (le même sans
table), carte `testbed/arena_variantes.ldtk` (grunt `rapide` imposé, grunt `blinde` imposé,
`grunt_plain`, grunt tiré) ; scénarios `variant_fast`, `variant_none`, `variant_elite` (même
partie, chacun son sujet : leurs trois traces sont identiques, c'est voulu) et `variant_draw`
(autre graine de carte). `games/zombies` : aucune variante.

## 26. Surfaces (T1.7, chantier E4 v1)

Code : `world::surface` (grille, table, traduction), `game::character::surface` (système),
`map_ldtk::game::collider::surface_grid_of_levels` (lecture LDtk), `map_ldtk::loader::surface_table`.

**Contenu.** Kind de dossier `Surface` (`content::registry::SurfaceEntry`) : un fichier RON par
surface, id = nom de fichier sans extension.
```ron
// games/testbed/assets/surfaces/eau.ron — game.ron : (path: "surfaces", kind: "Surface")
(intgrid_value: 1, tags: ["eau"], move_speed: "0.5")
// glace : (intgrid_value: 3, tags: ["glace"], move_speed: "1.0", acceleration: Some("0.2"))
```
`move_speed` et `acceleration` (défaut `1.0`) sont des **facteurs abstraits** (`ModifierOp::Mul`)
traduits selon le personnage : joueur → `MoveSpeed` et `Acceleration` (qui règle aussi son
freinage : l'adhérence, §34) ; ennemi au sol →
`EnemyMoveSpeed` (pas d'accélération) ; ennemi volant (`MovementType::Flying`) → rien. Un facteur
`1` ne pose aucun modificateur. Lint (`content::lint::lint_surfaces`) : `intgrid_value` > 0 et
unique (`DuplicateId`), facteurs > 0, tags non vides (fixtures `surface_duplicate_value`,
`surface_factor_non_positive`).

**Source v1 : LDtk.** Couche IntGrid optionnelle **`Surfaces`** d'une carte (valeur IntGrid =
`intgrid_value`, 0 = pas de surface), lue avec les murs (chargement et passage de niveau
`Floors`). Une carte sans la couche laisse la grille vide. Cavernes avec surfaces : v2.

**Grille.** `world::SurfaceGrid` : **creuse** (`BTreeMap<(i32, i32), SurfaceId>`), en cases de
grille monde de 16 (+y vers le haut, même découpage que `CellGrid` mais sans origine imposée :
une carte LDtk est placée n'importe où, alignée sur 16 depuis m0-v7). `SurfaceId` = valeur
IntGrid (`u8` non nul) ; `world::SurfaceTable` (hors rollback, posée au chargement depuis le
registre) donne la définition. Ressource rollback, checksum **neutre** (vide = 0) : traces des
cartes sans surface inchangées. Indépendante de `CellGrid` (une case de roche creusée garde sa
surface, ou n'en a aucune).

**Application** (`surface_modifiers_system`, `RollbackSystemSet::Input`, avant `apply_inputs`,
`order_mut_iter!`) : case sous les pieds = centre du collider avec son offset. Les
modificateurs de source `ModifierSource::Named("surface")` présents sont comparés aux
modificateurs voulus pour cette case ; s'ils diffèrent, `remove_by_source` puis pose, **sans
`until`** ; sinon rien n'est écrit. Effet dans la frame, aucun état nouveau (`Modifiers` est
déjà rollback). Carte sans surface : le système ne fait que retirer d'éventuels restes (joueur
venu d'un niveau `Floors` à surfaces) ; un personnage hors surface n'est jamais touché.

**Attente** `CellState(x, y, surface: "eau", at_frame)` : nom de la surface de la case (`"aucune"`
= sans surface). `kind` y est optionnel (au moins un des deux).

**Scénarios** (testbed) : `surface_walk` (couloir à bandes eau / sable / glace de
`testbed/surfaces.ldtk` : 1,25 et 2,0 px/frame contre 2,5), `surface_none` (couloir nu, même
input), `surface_ice` (glissade au demi-tour : 30 frames contre 6 sur le sol), `surface_enemy`
(`testbed/surfaces_enemy.ldtk` : un breacher qui traverse l'eau touche le joueur 205 frames
après celui du couloir nu).

**v2 / hors périmètre.** Dash par surface (il couvre la même distance sur la glace), freinage
réglé à part de l'accélération par surface (l'adhérence règle les deux), coût de flow field par surface,
dangers (piques, fosses, barils, feu), neige et boue (contenu 1837), surfaces de caverne
(`CaveConfig.surfaces`).

## 27. Effets v1, jauges et mutations (T1.10, chantiers C1 v1 et C4 v1)

Code : `effects::runtime` et `effects::progression` (règles pures, testées sans Bevy),
`game::effects_runtime` (porteurs, `apply_effects_system`), `game::progression` (rads, niveaux,
choix, drop d'arme), `content::lint::{lint_effect, lint_progression, lint_mutations}`.

**Effet** : `Effect { on, if, do }` (§4.3) ; un porteur a `Effects(Vec<Effect>)` et `EffectState`
(frame de pose de chaque effet, dernière frame touchée, déclencheurs reportés), rollback,
checksum **neutre**. Posés par `CharacterConfig::effects` (champ optionnel), par une mutation
prise, ou par `PlayerScript::mutations` ; un personnage sans effet n'a aucun composant (traces
existantes inchangées).

- **Déclencheurs v1** : `OnKill` (tueur = premier `HitBy` de `Death::last_hit_by`, joueur par son
  handle, entité — dont un émetteur — par son net id), `OnDamageTaken` (frame où les dégâts
  accumulés s'appliquent : `HealthRegen::last_damage_frame`, sinon un `DamageEvent` qui vise le
  porteur), `Tick(n)` (toutes les `n` frames depuis la pose, jamais à la pose), `OnGauge(id,
  Above(x))` (jauge franchie à la hausse, `avant < x ≤ après`, frame suivante), `OnLevelUp`
  (nouveau variant, voir plus bas). Les autres (`OnHit`, `OnDodge`, `OnReload`, `OnRoomClear`,
  `OnPickup`, `OnUse`, `OnEvent`) : lint `Unsupported` (« v2 »).
- **Conditions v1** (ET) : `HpBelow(x)` (fraction de `Health.max`), `HasTag(t)` (porteur),
  `TargetTag(t)` (victime d'`OnKill`, source d'`OnDamageTaken`), `NotHitFor(n)`. `Carrying`,
  `SquadSize`, `TargetInRange` : lint `Unsupported`.
- **Actions** : `Action` étendu (même enum que les power-ups, §14) de `Modifier { stat, op,
  value }` (permanent, source `Named("effect:<net_id>:<index>")`), `Heal(x)` (borné à
  `Health.max`, jamais sous la santé courante), `SpawnPattern { pattern, weapon }` (un `Emitter`
  T1.2 sur le porteur, table `projectiles` de `weapon` ; ignoré si un émetteur joue déjà),
  `GaugeAdd(id, x)`. `TimedModifier`/`CurrencyMultiplier` s'appliquent au **porteur seul**
  (la règle « tous les joueurs » est celle des power-ups). `RefillAmmo`, `RefillAmmoOf`, `RepairAllWindows`,
  `KillAllWaveEnemies`, `DestroyTerrain`, `ApplyStatus` : lint `Unsupported` dans un effet.

**Ordre des sets (constaté, non modifié).** Les `FrameEvents` sont vidés au début de chaque frame
et les morts sont détruits dans `DeathManagement` (`rollback_apply_death`, même frame) : un
système dans `Effects` (avant `DeathManagement`) ne verrait **jamais** les kills. D'où :
`apply_effects_system` dans `RollbackSystemSet::DeathManagement`, après
`rollback_apply_accumulated_damage` (dégâts appliqués, `Death` posés) et avant
`rollback_apply_bleedout`, puis `init_progression_players` → `progression_system` →
`weapon_drop_on_death_system` (après `loot_drop_on_death_system` : le flux `loot` sert d'abord
aux power-ups). Ordre interne : porteurs par `GgrsNetId`, puis index d'effet, puis ordre des `do`.
**La mort de la frame prime** : un porteur posé `Death` ou `Downed` dans la frame ne déclenche
rien et n'est pas soigné (test `la_mort_de_la_frame_prime_sur_le_soin`).

**Progression (C4 v1)** : kind `Progression` (`progression/<id>.ron`), **opt-in** : la partie joue
celle que nomme `Scenario::progression`, sinon `entry.progression` du manifeste, sinon aucune
(testbed et zombies : aucune ; flux `loot` intact).
```ron
// games/testbed/assets/progression/base.ron — game.ron : (path: "progression", kind: "Progression")
(gauge: "rads", per_kill: "1", levels: ["2", "4"], choices: 3, choice_frames: 600,
 mutations: [],  // pool ; vide = toutes les mutations du jeu
 weapon_pool: [(level: 0, weapons: ["pistol"]), (level: 1, weapons: ["shotgun"])],
 weapon_drop_chance: "0.0")
```
Chaque joueur reçoit `Gauges` (la jauge `gauge`, bornée à `[0, max]`), `Level`, `Mutations`
(rollback, neutres). Chaque mort attribuée au joueur ajoute `per_kill` ; niveau = nombre de seuils
`levels` atteints (les `GaugeAdd` comptent aussi). Chaque niveau gagné ouvre un choix
(`MutationChoice { options, since_frame, pending, armed }`, neutre) : `choices` mutations
**distinctes** tirées dans le flux `loot` (pondérées par `weight`, `max_stacks` respecté, joueurs
par `GgrsNetId`) ; un niveau gagné pendant un choix s'empile (`pending`). La simulation ne se met
**jamais** en pause. Choix par les bits d'input 13–15 (`INPUT_CHOICE_A/B/C`, boutons `ChoiceA/B/C`,
touches 1/2/3), seulement si aucun bouton de choix n'a été tenu depuis l'ouverture (`armed`) ;
sans choix, la première option est prise à `since_frame + choice_frames`. La mutation prise
ajoute ses effets, est notée dans `Mutations`, et **`OnLevelUp` se déclenche à la frame
suivante** (pour tous les effets `OnLevelUp` du joueur, ceux de la nouvelle mutation compris).
Pool épuisé : pas de choix, `OnLevelUp` directement. Moments clés `levelup`, `mutation`.

`choice_timing: Immediate` est le défaut. `BetweenFloors` (revue humaine M1) conserve
les niveaux gagnés pendant le combat dans `PendingMutations` (rollback neutre, absent
quand vide), puis ouvre les choix après nettoyage de l'étage et ouverture du portail.
Le portail attend que tous les joueurs vivants aient résolu leurs choix ; les projectiles
hostiles restants sont supprimés et les dégâts des joueurs sont ignorés pendant cet
interlude. Le timeout par choix reste actif, les horloges continuent. Le tirage `loot`
est différé avec le choix, donc ses options et les combats suivants peuvent changer.
Cette politique ne s'applique qu'au mode `Floors` ; les autres modes gardent le choix
immédiat. `throne` l'active dans `progression/run.ron`.

**Mutation** : kind `Mutation` (`mutations/<id>.ron`) : `(name, weight: 1, tags, max_stacks: 1,
effects: [Effect])`.

**Drop d'arme** : chaque ennemi mort (ordre `GgrsNetId`) tire `weapon_drop_chance` (flux `loot`) ;
gagné, une arme parmi celles du `weapon_pool` dont `level ≤` niveau **max** des joueurs (niveau
mis à jour dans la même frame), posée par `spawn_weapon_pickup` (chargeur plein, gratuite). Chance
0 : aucun tirage.

**Lint** : déclencheur/condition/action `Unsupported`, `Tick(0)`, `Heal ≤ 0`, pattern ou arme de
`SpawnPattern` inconnus, jauge d'`OnGauge`/`GaugeAdd` qui n'est celle d'aucune progression ;
progression : `per_kill ≤ 0`, `levels` vides ou non strictement croissants, `choices` hors
`[1, pool]`, `choice_frames` 0, chance hors `[0, 1]`, mutation ou arme inconnue ; mutation :
`weight` 0, `max_stacks` 0, `effects` vide ; `entry.progression` vers une progression inconnue
(audit T1.12). Fixtures `effect_unsupported`, `effect_out_of_range`, `effect_broken_reference`,
`progression_out_of_range`, `progression_broken_reference`, `mutation_out_of_range`,
`entry_progression_unknown`.

**Scénarios** : attentes `Gauge(handle, id, min, max, at_frame)`, `Level(handle, level,
at_frame)`, `Mutations(handle, contains, count, at_frame)` ; champs `Scenario::progression` et
`PlayerScript::mutations`. Testbed : `effect_on_damage_taken` / `effect_none` (`pilote` contre
`dummy` : 92 contre 72 points), `effect_on_kill` (`vampire`), `effect_tick` (`tireur`, couronnes
f122/f242/f362), `levelup_choice` (`ChoiceC` → `coriace`), `levelup_timeout` (première option à
f704), `weapon_pool_drop` (progression `armes`).

**Hors périmètre** (écran de choix et HUD des rads livrés depuis : §30, §32) : déclencheurs et conditions v2,
objets passifs/actifs (M2), modificateurs de projectiles (M4), pause, `--progression` dans
`alacod-sim`.

## 28. Générateur v1 et placement scripté (T1.13, voie V3)

Code : `crates/scenario/src/generate.rs` (gabarits), `scenario::runner::
apply_scenario_character_placements` (placement), `map_ldtk::game::local::spawn_character`
(chemin commun avec les `CharacterSpawn` de carte), `game::character::config::CharacterTest`.

**Placement scripté.** `Scenario.characters: [(character: "grunt", x: "120", y: "-40",
at_frame: 5, variant: Some("blinde"), team: Some(Allies))]` (`variant`, `team` optionnels ;
coordonnées monde en chaînes `Fixed`). À `frame == at_frame`, dans l'ordre de déclaration, le
personnage est créé par **le même chemin qu'un `CharacterSpawn` de carte** : net id alloué à
cette frame, santé F5 × difficulté (§23), variante imposée ou tirée (§25), équipe du placement,
sinon `CharacterConfig.team`, sinon `Enemies`. Système dans `GgrsSchedule`,
`RollbackSystemSet::EnemySpawning` (comme un spawn de vague, rejoué à l'identique par un
rollback), avec une condition d'exécution : sans placement, il ne tourne pas, aucune trace
existante ne change. Le réenregistrement le conserve (`RecordedSettings::characters`). C'est
l'outil qui remplace « une arène LDtk par ennemi » ; les arènes de T1.4/T1.5 restent.

**`EntityRef::Placed(n)`** : le personnage créé par le placement d'index `n`. Le runner le
retrouve par un composant d'observation `ScriptedPlacement(n)` posé à la création, **hors
rollback** (non enregistré : ni checksum ni trace), reposé quand un rollback rejoue le
placement. Absent avant `at_frame` (une attente qui le vise échoue).

**`test:` d'un personnage** (opt-in, aucun effet en jeu) : `test: Some((frames: 600, still:
true, moving: true, expect_still: [...], expect_moving: [...]))` (`still`/`moving` vrais par
défaut). `make gen` écrit un scénario par gabarit actif :
`tests/scenarios/generated/<jeu>/enemy_<id>_still.ron` et `enemy_<id>_moving.ron`.

- **Gabarits** : `EnemyVsStillPlayer` (le joueur, armes de départ, ne fait rien) et
  `EnemyVsMovingPlayer` (il marche en carré, 90 frames par côté, en boucle, sans tirer). L'ennemi
  est placé à 200 unités à droite du **spawn du joueur** à la frame 1 ; le spawn est lu par une
  sonde d'une frame (`discover_enemy_probe`).
- **Attentes par défaut** (liste vide) : immobile → moment clé `Event(kind: "hit", "joueur 0
  touché")` avant la dernière frame (cumulatif : la santé du joueur se régénère, la fiche
  proposait `Health` à la dernière frame, qui ne prouvait rien — constaté sur `charger`) et
  `EnemyNeverInWall(Placed(0))` ; mobile →
  `PlayerAlive` à la dernière frame et `EnemyNeverInWall(Placed(0))`. Une liste explicite
  **remplace** les attentes par défaut de son gabarit (viser l'ennemi : `Placed(0)`).
- **Où** : les gabarits ennemis jouent dans le jeu énuméré lui-même (le registre du testbed n'a
  pas les personnages des autres jeux) : `testbed/arena.ldtk` pour le testbed,
  `exemples/test_map.ldtk` pour `zombies`, sinon `entry.start_map`. Jeu en mode `Waves` :
  `wave_overrides.grace_period_frames = frames + 1`, aucune vague ne s'ajoute à l'ennemi testé.

**`generate_template`** (manifeste, ajout de l'orchestrateur pour `games/throne`) :
`generate_template: (map: "maps/salle.ldtk", target: "cible")`. S'il est présent, les gabarits
d'armes **et** d'ennemis du jeu jouent dans ce jeu, sur `map` ; sinon le testbed comme avant
(`zombies` et `testbed` ne déclarent rien : leurs scénarios générés ne changent pas). Les inputs
du gabarit d'arme (visée, marche de la mêlée) sont réglés sur l'arène du testbed : la carte place
la cible (`target`, `counts_hits: true`) au même décalage du spawn du joueur, +128 et −48 en
coordonnées LDtk. Lint : carte connue, cible connue avec `counts_hits: true`
(`generate_template_unknown_map`, `generate_template_target_without_hits`).

**`Scenario.mode`** (suite de T1.13, `m1-dettes-lot-1`) : `mode: Some(Sandbox | Waves |
Floors)` l'emporte sur `entry.mode` du manifeste (le runner remplace `entry.mode` avant de
l'insérer) ; absent, comportement d'avant. `Floors` exige `floors` (échec du runner sinon). Le
générateur écrit `mode: Sandbox` dans les gabarits d'un jeu qui déclare `generate_template` (un
jeu en `Floors`, comme `throne`, jouerait sinon sa séquence au lieu de la carte du gabarit) ; les
générés de `zombies` et `testbed` n'ont pas le champ.

**Hors périmètre** : gabarits par statut (T1.3 mergée depuis, §19 ; ces gabarits ne sont pas écrits), vidéos des gabarits,
génération de cartes, bots dans les gabarits.

## 29. Le jeu `throne` (T1.0c + T1.11, voie V2)

`games/throne/` : le clone, **données seulement** (aucun code d'engine propre : `src/main.rs` est
celui du testbed renommé). Membre du workspace, `make throne` (fenêtré), `make lint` le linte avec
zombies et testbed. Une run = trois étages de caverne générés (mode `Floors`), sans vagues.

**Manifeste** : `entry: (start_map: "cave:niveau_1", default_seed: 123456, mode: Floors)` ;
séquence `floors/run.ron` = `cave:niveau_1`, `cave:niveau_2`, `cave:niveau_3` (même graine pour la
séquence, §17 et §21). Kinds : `Character`, `Weapon`, `MeleeWeapon`, `Pattern`, `Cave`, `Floors`,
`PowerUp`, `Economy`, `Ui`, `Camera`, `SpriteSheet`, `Map` (gabarit des scénarios générés),
`Progression`, `Mutation`, `Clock`, `Difficulty` (phase 2).

**Joueur** : `characters/pilote.ron` déclare le personnage d'id `player` (le moteur crée tous les
joueurs depuis lui), copie de `player` du testbed ; trois armes de départ (`mitraillette`,
`revolver`, `lance_lames`).

**Armes** (`weapons/weapons.ron`) : douze armes du joueur sur cinq munitions `Custom` —
`balles` (`revolver`, `mitraillette`), `obus` (`fusil_a_pompe`, `canon_ricochet` : `Bounce`),
`explosifs` (`lance_grenades` : `Bounce`+`Pierce`+`Lifetime`, explosion puis couronne d'éclats ;
`roquette` : souffle en `on_expire` ; `mortier` : `Gravity`, éventail de fragments), `energie`
(`laser` : `Pierce` ; `plasma` : `Size`+`Lifetime` ; `traqueur` : `Homing`), `lames`
(`lance_lames` : `Bounce`+`Pierce` ; `disque` : rafale, `Pierce`+`Lifetime`). Leurs `spread`
(mitraillette 0,15, disque 0,05, revolver 0,0005, les autres 0) ne sont appliqués que depuis D51
(§16) : avant, toutes tiraient à ±0,5 rad ; aucune valeur n'a été recalée à cette occasion. Plus `arsenal`,
l'arme des ennemis (sa table `projectiles` : `crachat`, `plomb`, `boule`). Mêlée
(`weapons/melee.ron`) : `bare_hands` (joueur), `griffes`, `crocs`, `massue`. Chaque arme à distance porte un
`test:` (jauges calées sur la mesure ; les armes de mêlée sont générées sans). **Gabarits** (§28, m1-throne-gen-et-d40) :
`generate_template: (map: "gabarit_armes.ldtk", target: "cible")` — `make gen GAME=throne` joue
les armes et les onze ennemis (dont `roi_rat` ; `test:` sur chacun, gabarits immobile et mobile) dans `throne`, sur
la carte gabarit (`cible` et `mannequin`, même disposition que l'arène du testbed), en mode
`Sandbox` imposé ; scénarios versionnés sous `tests/scenarios/generated/throne/`.

**Ennemis** (`characters/`, behaviors §22, variantes §25 ; aucun sprite) : mêlée `rat`
(`Melee("griffes")`, variante `rapide`), `chien` (rapide et fragile, variantes `rapide` et
`blinde`), `brute` (`massue`, variante `blinde`), `rodeur` (`Wander` hors de vue, `Sight(300)`) ;
tireurs `cracheur` (pattern `crachat` : `Aimed`), `arroseur` (`eventail` : `Spread`), `tourelle`
(immobile, `couronne` : `Ring` après télégraphe) ; chargeur `buffle` (`Charge`) ; kiter
`franc_tireur` (`KeepDistance` + `visee`) ; fuyard `pillard` (`Flee` sous 50 %).

**Cavernes** (`caves/`, gabarit `caves/gabarit.ldtk`, tilesets `atlas/`) : `niveau_1` 48 × 32,
4 ennemis (`rat`, `chien`, `rodeur`) ; `niveau_2` 56 × 40, 6 ennemis (premiers tireurs, chargeur,
fuyard) ; `niveau_3` 64 × 44, 8 ennemis (tous les profils). `fill_ratio` 0,38 (plus ouvert que le
testbed : les bots `prudent` vont en ligne droite). Pas de surface (impossible dans une caverne
générée, §26 v2).

**Boss** (m1-integration-scenarios) : `roi_rat`, en données seulement (360 PV, tag `champion`, `[Shoot(couronne), Charge, Chase]`, sans phases : D4, M2 ; corps de 20 × 20 avant `scale` 1.4, soit 28 px en jeu : gabarit `Large` de la navigation, D41 fermée par m1-navigation-profils-tailles), premier de `niveau_3.characters` avec `enemy_spawns` 9 : un exemplaire au point 0 (le plus éloigné du départ, toujours présent) ; la tourelle, seul ennemi fixe, est dernière (point le plus proche, ou absente quand la caverne n'a que huit points) ; le portail exige la mort du boss comme celle des autres, et la boucle infinie du dernier niveau en recrée un à chaque rechargement. Chaque caverne est un asset distinct (`cave://<dossier>/<id>.ldtk`, §21 ; avant, les trois étages chargeaient `niveau_1`).

**Butin** (`items/powerups.ron`, §14) : `munitions_balles`, `munitions_obus`,
`munitions_explosifs`, `munitions_energie`, `munitions_lames` (`RefillAmmoOf`, D40, poids 36,
12, 12, 12 et 24), `rage` (dégâts ×2) et `vitesse` (×1,3) (poids 25), `drop_chance` **0,40** (0,25 avant D50 : pénurie de munitions prouvée sur la graine 76, soft-lock des bots ; décision de William du 2026-10-14 : relever la chance, sans pondération par les réserves). Pas de `weapon_pool`.

**Assets** : tous des **placeholders** copiés du testbed (sprites du joueur et des armes, slash,
sons, police, tilesets), enregistrés dans `games/throne/assets/assets.yaml` (licences reprises de
zombies, `statut: placeholder`).

**Scénarios** : `throne_floor_1` (un bot `prudent` finit l'étage 1), `throne_three_floors` (deux
bots finissent la run), `throne_solo`, `throne_quad`, `throne_duo_defaite`, `throne_ammo_pickup`,
joués avec tout ce que le manifeste active (butin compris). Mesure : `alacod-sim --game throne
--bots 2 --profiles prudent,prudent --floors run --seeds 1..20 --until-floor 3 --max-frames
12000`. Les chiffres des phases 1 et 2 (19/20, 17/20) portaient sur trois fois `niveau_1` et
sont caducs ; mesures actuelles : journal de `docs/taches.md` (m1-integration-scenarios,
m1-v3-bots-portail).

**Phase 2 — progression, mutations, horloge, difficulté** (T1.9, T1.10 ; actives par le manifeste :
`entry: (..., progression: "run", clocks: ["etage"], difficulty: true)`, donc en partie jouée
**et** dans tous les scénarios de `throne`) :
- `progression/run.ron` (§27) : un rad par kill, niveaux à 3, 8 et 15 rads, trois mutations
  proposées, la première prise d'office après 600 frames (les bots ne choisissent pas) ;
  `weapon_pool` : munitions de départ au niveau 0, obus et lames au 1, énergie au 2, explosifs au
  3 ; `weapon_drop_chance` 0,08.
- Huit mutations (`mutations/`, effets v1 seulement) : `coriace` (OnLevelUp : +25 MaxHealth,
  soin 25), `vampire` (OnKill : soin 4), `tireur` (Tick(180) : salve de six éclats, pattern
  `salve`), `adrenaline` (touché sous 40 % : vitesse ×1,4 pendant 3 s), `rancune` (touché :
  dégâts ×1,5 pendant 2 s), `sang_froid` (Tick(60) sans coup depuis 5 s : soin 3), `chasseur`
  (tuer un champion : dégâts ×2 pendant 5 s), `irradie` (OnKill : un rad de plus).
- Horloge `clocks/etage.ron` (§23, portée `Floor`) : `alerte` à 15 s, `renfort` à 30 s puis
  toutes les 15 s. **Aucune action n'y est attachée** (`OnEvent` des effets est v2) : moments clés
  et attente `Clock` seulement.
- `difficulty.ron` : `1 + floor * 0.25 + floor_minutes * 0.25` (santé des ennemis à l'apparition,
  dégâts qu'ils infligent).
- Scénarios : `throne_progression` (niveau, rads, mutation prise d'office, horloge),
  `throne_mutation_choice` (inputs enregistrés d'un bot, `ChoiceB` → `chasseur`).
- Gabarit des scénarios générés : `gabarit_armes.ldtk` (copie de l'arène du testbed, `cible` à
  `counts_hits` à +128/−48 du spawn, `mannequin` ailleurs), joué en `Sandbox` imposé (voir
  **Gabarits** plus haut).

## 30. Écran de mutation et transition (T1.16, voie V4)

Présentation seule : aucune trace ne change, la simulation ne fait jamais de pause.

- **Modèle de vue** `game::ui::mutation_screen::MutationScreenView` (ressource `Update`, hors
  rollback, aussi remplie en headless par `MutationScreenModelPlugin`) : dérivé en lecture seule
  du `MutationChoice` du joueur affiché (`CameraFollowOverride`, sinon le joueur local de plus
  petit handle) : `open`, `options` (id, nom `Mutation.name`, description), `frames_left`
  (`since_frame + choice_frames − frame`), `total_frames`, `highlighted`. Fonctions pures
  testées : `build_view`, `apply_navigation`, `confirm_bit`, `confirm_input`.
- **Description** `effects::describe::describe_effect` : une phrase française par effet,
  « <déclencheur>[, <conditions>] : <actions>. » (« Quand vous êtes touché, sous 40 % de vie :
  vitesse × 1,4 pendant 3 s. »), virgule décimale, durées en secondes. Aucun champ de contenu
  en plus ; une variante nouvelle de `On`/`Condition`/`Action` ajoute sa phrase (le `match` est
  exhaustif).
- **Entrées** : ←/→ (flèches, D-pad) déplacent la surbrillance (état UI local, en boucle) ; un
  bouton `ChoiceA/B/C` tenu (1/2/3 ou script) la pose ; un nouveau choix la remet à 0. Valider
  (Entrée, A) ajoute à l'input du joueur affiché le bit de la carte surlignée
  (`read_local_inputs`) : seul chemin vers la simulation, le même que 1/2/3. Le D-pad et → gardent
  aussi leur action de déplacement (la partie continue pendant le choix).
- **Mise en page** (D71, §56) : gabarit `bevy_hui` `ui/mutation_screen.html` (throne, testbed),
  rempli par `game::ui::mutation_hui` depuis `MutationScreenView`. Remplace l'ancien
  `ui/mutation_screen.ron` et son schéma de lint (retirés). Absent : pas d'écran (le choix se
  fait aux touches 1/2/3 ou d'office).
- **Transition de niveau** `game::ui::floor_transition` : à chaque changement de
  `FloorState::index` lu côté présentation (pas `FloorEntered`), voile noir qui s'éclaircit
  linéairement en `floor_fade_seconds` (`camera.ron`, défaut 0,4) et caméra posée sur sa cible
  pendant le fondu (`floor_recenter`, défaut vrai). La première lecture ne déclenche rien.

## 31. Feedback v1 (T1.17, voie V4)

**Présentation seule.** Rien n'est rollback, rien n'entre dans le checksum ni dans les traces ;
la simulation ne lit jamais ces composants ou ressources. Le hit stop **ne fige jamais la
simulation** (rollback, p2p) : il gèle l'avance des animations et le suivi de caméra pendant
`n` ticks de **rendu** (`animation::AnimationFreeze`, un tick par `Update`, avancé en `First`) ;
`Time<Virtual>` n'est jamais touché (ggrs avance sur ce temps).

**`ui/feedback.ron`** (type partagé `content::feedback::FeedbackSettings`, enveloppé par l'asset
`game::feedback::FeedbackConfig`, lu tel quel par le lint) : `hit_flash` (frames, couleur :
multiplicateur linéaire des calques, au-delà de 1.0 le sprite est surexposé, `(1, 1, 1)` est
invisible), `shake` (frames, amplitude), `hit_stop_frames` (0 : aucun), `damage_numbers`,
`telegraph.color` (RGBA), `by_kind: {DamageKind: …}` et `by_weapon: {"arme": …}` (surcharges
`hit_stop_frames`/`hit_flash`/`shake`, champs optionnels), `sounds`. Résolution d'un coup
(`FeedbackSettings::resolve`), champ par champ : arme, puis genre, puis valeur globale.
Lint : frames > 0 (flash, secousse, surcharges comprises), amplitude ≥ 0, couleur du flash dans
`[0, 4]`, couleur du télégraphe dans `[0, 1]`, armes de `by_weapon` déclarées par le jeu (à
distance ou de corps à corps), sons présents.

**Déclencheurs** (par frame simulée) : flash sur la cible de chaque `DamageEvent` et chiffre si
`damage_numbers` ; au plus **une** secousse (cible = joueur local, ou tireur = joueur local quand
la secousse vient d'une surcharge : impact lourd) et **un** hit stop (source ou cible = joueur
local) par frame, le plus fort des coups de la frame.

**Limite connue : l'arme d'un coup.** `DamageEvent` ne porte pas l'arme (il est tracé : lui
ajouter un champ ferait bouger toutes les traces). L'arme retenue pour `by_weapon` est
l'émetteur de la source (`Emitter.weapon`) s'il existe, sinon **l'arme active** de son
inventaire au moment du coup (une balle tirée avant un changement d'arme est attribuée à la
nouvelle) ; aucune pour la mêlée (tag `melee`). Les projectiles infligent aujourd'hui des
dégâts `Physical` (`Fire` pour la brûlure) : `by_kind` sert surtout aux genres futurs, les
explosifs se règlent par `by_weapon`.

**Flash** : posé sur l'entité racine (`HitFlash`), appliqué aux **calques enfants** par
`apply_layer_colors` (même parcours que la teinte des statuts, §19 ; le flash l'emporte tant
qu'il dure). Avant T1.17, le flash cherchait un `Sprite` sur la racine, qui n'en a pas : il
n'était jamais visible. La secousse retire son décalage du tick précédent avant d'appliquer le
suivant (sans ça, les décalages s'additionnent quand la caméra ne suit plus, pendant un hit
stop), et décroît sur sa durée.

**Télégraphe au sol** (`telegraph_shape`, lecture seule de l'état) : phase `Telegraph` d'une
`Charge` (T1.4) → cercle au point visé figé, rayon `attack_range` ; pas `Telegraph` d'un
émetteur (`Emitter::telegraphing`, T1.2) → cercle autour du porteur, rayon la plus grande
portée de sa table de projectiles. Contour au rayon complet, disque intérieur qui grandit
jusqu'au déclenchement (gizmos).

**Preuve sans écran** : `game::feedback::FeedbackLog` (ressource de présentation, plugin
`FeedbackLogPlugin` actif aussi en headless) reçoit les indices `{genre, frame, net_id,
position, valeur}` de chaque **nouvelle** frame simulée (`FrameCount` plus grand que la
dernière frame journalisée : les frames rejouées par un rollback ne sont pas journalisées deux
fois). Limites (présentation, sans conséquence) : un tick qui simule plusieurs frames ne voit
que les événements de la dernière ; un coup prédit puis annulé par un rollback p2p reste
journalisé. `scenario::events` en fait des moments clés `feedback` (`<genre> <net_id> (…)` :
« télégraphe 27 (29 frames, rayon 40) », « hit stop 22 (2 frames) », « secousse 22 (8 frames) » ;
flash et chiffre, un par coup, n'en font pas) ; attente `Event(kind: "feedback",
label_contains: …)`. Scénarios : `enemy_charge` (télégraphe de la ruée vu à f42, hit stop et
secousse au contact à f100), `weapon_grenade` généré (`test.expect` de la grenade : secousse et
hit stop de surcharge à la première explosion, f113). Ces attentes ne touchent pas aux traces.

Hors périmètre : sons (D32), particules, écran de mort.

## 32. HUD throne (T1.18, voie V4)

Présentation seule : aucune trace ne change.

- **Liste fermée** `content::ui::HUD_SOURCES` : `health`, `wave`, `ammo`, `weapon`, `enemies`,
  `players`, `currency`, `perks`, `downed`, `powerups`, `prompt`, et (T1.18) `rads`, `level`,
  `ammo_by_type`, `statuses`, `floor`, puis (M2-T18, §56) `blanks`, `keys`, `shells`,
  `passives`, `active_item`, `active_charge`, `minimap`. Le lint lit la source de chaque
  widget de `ui/hud.ron` (`HudFileSchema`) : source inconnue = `UnknownKind`
  (fixture `hud_unknown_source`).
- **Modèle** `game::ui::hud_model` : `hud_values(&HudPlayerInput) -> HudPlayerValues` (pure,
  testée sans rendu) et `player_source_text(source, prefix, values)` ; `HudSnapshot { frame,
  texts, bars, values }` mis à jour en `Update` par `HudModelPlugin`, **aussi en headless** ; le
  rendu (`ui::hud`) lit le snapshot. Joueur affiché : `CameraFollowOverride`, sinon le joueur
  local de plus petit handle.
- **Textes** : `rads` « 5 / 8 rads » (jauge `gauge` de la progression active / seuil du prochain
  niveau ; « 12 rads » au dernier niveau), barre = (valeur − seuil courant) / (seuil suivant −
  seuil courant), pleine au dernier ; `level` « Niv. 2 » ; `ammo_by_type` une ligne par munition
  `Custom` (« balles 120 », ordre des ids) ; `statuses` une ligne par statut dans l'ordre de pose
  (« brulure ×2 · 3 s », secondes arrondies au-dessus) ; `floor` « Étage 3 » (index + 1, mode
  `Floors` seulement). Sans valeur : texte vide, pas même le préfixe (D22).
- **Attente** `HudText { source, contains, at_frame }` : le texte (sans préfixe) contient
  `contains` ; lue dans `HudSnapshot`, hors simulation et hors trace.
- **Contenu après revue humaine (2026-10-14)** : `games/throne/assets/ui/hud.ron` —
  étage en haut à gauche ; vie, niveau et barres vie/rads en bas à gauche ; arme et
  chargeur/réserve en bas à droite ; statuts/power-ups contextuels en haut. Les textes
  joueurs/ennemis, rads numériques et réserves des armes inactives sont retirés du HUD
  courant. Les sources restent disponibles dans le modèle. Mutation/fin masquent le HUD.
  Les RON HUD de zombies et testbed restent inchangés.

### Polissage de revue M1 (2026-10-14)

Le rendu des cartes raccourcit les descriptions générées (« regagnez 25 points de vie »
→ « +25 PV »), en conservant déclencheurs, conditions, quantités et durées. Le modèle
`MutationScreenView` garde la description complète. L'écran de fin Floors affiche
étage, éliminations et durée min:sec plutôt que frames/score technique ; panneau centré.
`play_scenario --capture <dir> --size 640x360` vérifie les petits viewports ; défaut 960x540.
Ce sont des changements de présentation, aucune mécanique ni trace modifiée.

## 33. Restart en ligne (D14)

« Rejouer » à la fin d'une partie p2p (chemin `--matchbox`) relance une partie avec les mêmes
pairs, sans saisie de lobby et **sans message réseau** : l'accord vient du déterminisme
(`game::jjrs::restart`).

- **Compteur** `OnlineGames` (hors rollback) : parties **en ligne** jouées depuis la dernière
  entrée normale dans `LobbyOnline` (remis à 0 à cette entrée, +1 à `OnEnter(InGame)` en ligne).
  Les parties locales jouées avant la connexion ne comptent pas : les pairs ont donc le même
  compteur.
- **Rejouer** (`RunRequest::Restart` en ligne, `allumette_url` vide) : pas d'abandon (comme le
  restart local), `OnlineRestart { game: n }` puis `LobbyOnline`. Le socket de la partie est
  fermé à `OnExit(InGame)` ; le nouveau s'ouvre sur la salle `{lobby}-r{n}` (jamais la salle
  d'origine : aucun pair fantôme) ; `wait_for_players` et la création de session sont ceux du
  premier lancement.
- **Graine** de la partie relancée : `restart_seed(seed, n)` (`seed ^ fnv1a("restart") ^ n`,
  replié sur 32 bits) depuis la graine de carte ; le restart **local** garde la même graine.
- **Désaccord** : sans les pairs dans la salle de restart au bout de 30 s
  (`RESTART_TIMEOUT_SECS`), retour à la salle `{lobby}` d'origine et compteur à 0, comme
  « Lobby ». Le départ d'un pair après la fin de partie n'affiche plus « GAME DISCONNECTED »
  (l'écran de fin reste) ; l'overlay de déconnexion disparaît à la sortie de la partie.
- **Allumette** (`--allumette`) : pas en v1 (l'API joint un lobby par `game_id`, pas par nom ; le
  créateur bloque jusqu'à 60 s) : « Rejouer » y reste redirigé vers le lobby (dette).
- **Preuve** : `ALACOD_RESTART_AT_FRAME=r` (avec `ALACOD_STATE_TRACE`/`ALACOD_EXIT_AT_FRAME`) :
  trace de la partie 1 écrite dans `<fichier>-g1` à la frame `r`, `RunRequest::Restart` posé
  60 frames plus tard, partie 2 écrite dans `<fichier>-g2` à `ALACOD_EXIT_AT_FRAME` (compté depuis
  son début). `scripts/p2p-restart.sh` : deux clients, traces identiques partie par partie, partie 2
  différente de la 1 (graine dérivée) ; un client local dont les deux parties sont identiques
  (même graine : la sortie de partie remet tout à zéro).

## 34. Course et esquive (dash)

Code : `game::character::movement` (`MovementConfig`, `run_velocity`, `approach_axis`, `grip`),
`combat::actors::DashState` et `dash_step` (règles pures, testées sans Bevy),
`game::character::player::input::apply_inputs` (visée, dash puis course, un seul système).
Réglage des joueurs : `player_config.ron` (zombies, testbed), `characters/pilote.ron` (throne).
Les ennemis ne lisent que `max_speed` (leur IA fait le reste) ; le champ `friction` n'existe plus.

**Course.** Chaque axe va vers la vitesse visée (direction des touches normalisée × `MoveSpeed`
résolue × sprint ; nulle sans touche) : il accélère de `acceleration` (px/s²) vers une cible plus
rapide dans le même sens, freine de `deceleration` (px/s², absent = `acceleration`) vers une cible
plus lente, nulle ou opposée, sans jamais la dépasser. Par axe plutôt qu'en norme : arithmétique
`Fixed` exacte, sans racine, et un demi-tour sur un axe ne freine pas l'autre ; relâcher un axe le
freine même si l'autre est tenu. Réglage (150 px/s, 3000/3000) : pleine vitesse (2,5 px par frame)
en 3 frames, arrêt en 3, demi-tour en 6. Tenir une touche `n` frames déplace de **2,5 × `n` px**
(l'accélération et le freinage se compensent) : c'est la règle de calcul des positions attendues.

**Adhérence.** `grip` = stat `Acceleration` résolue / `movement.acceleration` (1 sans
modificateur) multiplie aussi le freinage : la glace (§26, ×0,2) met 15 frames à lancer ou arrêter
le joueur, 30 au demi-tour.

**Dash.** Part sur l'appui (front montant de `Dash` : tenir le bouton ne relance rien) et avance
dès cette frame ; direction : touches, sinon visée, sinon regard, inversée par `Modifier`. Il
couvre exactement `dash_distance` px en `dash_duration_frames` frames, par un pas qui décroît
linéairement de `2·d/n − course` jusqu'à la vitesse de course (`dash_step`) : on sort du dash en
courant, sans arrêt sec ni glissade ; sans touche à la sortie, la course freine (+2,5 px). Le pas
est une vitesse appliquée par `move_characters` : un dash s'arrête aux murs. `dash_cooldown_frames`
est compté depuis le départ. Un appui refusé (dash ou cooldown en cours) est gardé
`dash_buffer_frames` frames, celle de l'appui comprise, et part à la première frame permise.
Pendant le dash : pas de tir (le bouton de dash tenu, lui, ne bloque plus rien), aucun
ralentissement par les ennemis (on traverse la horde). À terre, étourdi ou gelé (T1.3, §19) : pas
de dash, pas même d'un appui gardé d'avant le statut (il vieillit et s'oublie). Un dash en cours
quand le statut tombe ne bouge plus (vitesse annulée par `status_motion_system`), mais ses frames,
ses i-frames et son cooldown s'écoulent. Réglage : 64 px en 8 frames (13,5 px la première),
cooldown 24, i-frames 6, buffer 8.

**I-frames.** Les `dash_iframes` premières frames du dash, celle de l'appui comprise :
`Health.invulnerable_until_frame` (déjà rollback) prend le maximum de sa valeur et de
`frame + dash_iframes − 1`. `Health::is_invulnerable_at(frame)` est le seul test : balles et
projectiles composables **traversent** le joueur (ni dégât ni arrêt, `Pierce` non consommé) ; les
hitbox de mêlée ne le voient pas (ni dégât ni recul, et il n'est pas marqué touché : une hitbox
encore active après les i-frames le touche) ; tout autre dégât est annulé au résolveur
(`rollback_resolve_damage_events`, sauf `DamageKind::True`). `Homing` et `Aimed` le visent
toujours.

**Scénarios** : `dash_wall` (64 px, puis butée contre le mur), `movement_melee` (sprint, dash
dans la direction des touches, appui refusé pendant le cooldown, appui gardé qui part à la fin du
cooldown), `surface_ice` (demi-tour sur la glace et sur le sol), `stat_move_speed`,
`dash_iframes` (un joueur plaqué contre un mur dashe dedans sous un tir ami continu, une balle
toutes les 2 frames : `NoDamageBetween` sur les 6 i-frames, coups juste avant et juste après ;
`dash_iframes: 0` le fait échouer, 3 coups de plus), `dash_stun_buffer` (un appui gardé pendant le
cooldown ne part pas une fois le joueur étourdi : la balle perçante tirée à bout portant le touche
à la frame où le dash fautif l'aurait rendu invulnérable ; sans la garde, 10 PV de plus),
`dash_aim_change` (la visée change pendant le dash, synctest).

**Recalage des scénarios (2026-10-04).** La course a changé toutes les trajectoires : positions
attendues recalculées (`at_frame N` = état après N frames, segments `from..to` à fin exclue, délai
d'input de 5 frames, 2,5 px par frame de touche), marches raccourcies quand elles visaient un
interactable (achats, réparation), générateur recalé (`WALK_*_FRAMES`). La visée des parties de
référence du clone (`clone_solo`, `clone_duo`, `clone_quad`) a été **ré-enregistrée** en headless à
partir de f1000 (outil temporaire, non versionné) : toutes les 15 frames, chaque joueur concerné
tire vers le zombie vivant le plus proche (égalité : plus petit `GgrsNetId`) s'il est à 400 px au
plus (joueur 3 du quad : 1000), pendant 15 frames (solo) ou 2 (duo, quad) ; `pan` = écart monde
arrondi, +y vers le haut. Mêmes attentes qualitatives (vagues, kills, victoire, joueurs debout),
soldes et frames recalés. Les bots joués en direct (`bot:`) gardent leurs scénarios, frames
recalées (portails, niveaux, mutations) ; ceux dont les inputs sont des enregistrements de bot figés
en `Scripted` ont été **réenregistrés** par la même commande `alacod-sim --save-scenario`
(`throne_mutation_choice`, avec `ChoiceB` réinséré à la frame de son choix ; `bot_prudent_nododge`,
avec le même diff temporaire qui désactive l'esquive).

## 35. Salles typées et verrouillées (M2-E1, chantier E1)

Voie B du prototype `m1-proto-etage-salles` : **un étage = un monde LDtk assemblé de salles**
(l'assembleur `Basic`, inchangé), chaque salle étant un gabarit typé. Code : contrats dans la
crate `world` (`world::rooms`, aucune dépendance au rendu), branchement ECS dans
`map_ldtk::game::rooms` (portes, bornes de salle), contenu dans `content::registry`
(`RoomEntry`) et `content::lint` (`lint_room_kinds`).

**Type de salle (liste ouverte).** Champ de niveau LDtk `room_kind` (chaîne) d'un gabarit, recopié
tel quel par l'assembleur (`fieldInstances` du gabarit) et posé en `world::RoomKind` sur l'entité
du niveau (`add_level_components.rs`, à côté de `RoomBounds`/`LevelId`). Les types sont les
fichiers d'un dossier de contenu de kind `Room` (`rooms/<id>.ron`, id = nom de fichier) :
```ron
// games/testbed/assets/rooms/combat.ron  — game.ron : (path: "rooms", kind: "Room")
(locks: true)   // verrouille et endort ses ennemis ; `locks: false` (défaut) : un lieu comme un autre
```
Types du testbed : `depart`, `combat` (`locks: true`), `recompense`. `boss`, `boutique`… s'ajoutent
par un fichier, sans code. Une carte sans `room_kind` n'a aucune salle typée : **aucun système du
module ne fait rien** (traces existantes inchangées).

**Lint.** `RoomTemplate` : le `room_kind` d'un niveau d'une carte du dossier `Map` doit être un
type chargé ; inconnu → `UnknownKind` (« RoomTemplate « SalleX » : room_kind « y » inconnu »).
Fixture `room_kind_unknown`.

**État rollback.** `world::RoomStates { doors_opened, rooms: BTreeMap<LevelId, RoomState> }`,
`RoomState { Dormant, Locked, Cleared }` (une salle absente de la table est `Dormant`), ressource
**neutre** (`rollback_and_trace_resource_neutral` : vide hors carte typée, contribution 0).
`world::RoomDormant` (marqueur sur les ennemis d'une salle dormante) : composant neutre.
`world::RoomChanged` (`FrameEvents`, neutre) : un changement d'état (frame, salle, type, avant,
après), pour les récompenses `OnRoomClear` à venir. `RoomKindTable` (types du jeu) et `DoorShape`
(collider de porte pour la refermer) sont statiques, hors rollback.

**Comportement** (`room_lock_system`, `RollbackSystemSet::Run`, après la transition d'étage ;
`room_dormancy_system`, `EnemySpawning`, avant l'IA) :
1. Au premier tick d'une carte typée, les portes posées au bord (à moins de 24 px) d'une salle
   typée s'ouvrent (collider et `Interactable` retirés) : une porte de salle est ouverte par défaut.
2. Une salle dont le type verrouille passe `Dormant → Locked` quand un joueur **debout** est à plus
   de 24 px de ses bords et qu'un ennemi y vit : ses portes et leurs jumelles (au bord de la salle
   voisine) se referment (le champ de flux les voit comme des murs) ; puis `Locked → Cleared` quand
   plus aucun ennemi n'y vit : elles se rouvrent pour de bon.
3. **Jamais sur un occupant** : si un joueur ou un ennemi chevauche l'une des portes à refermer,
   aucune ne se ferme ; le verrouillage est réévalué à chaque frame (différé, trace
   `room_lock_deferred`).
4. **Coop** : au verrouillage, les joueurs hors de la salle (debout ou à terre) sont téléportés
   à l'entrée : 40 px dans la salle devant la porte la plus proche du joueur qui a déclenché le
   verrouillage (tangente du bord, 14 px d'écart, `GgrsNetId` croissant), vitesse annulée.
5. **Ennemis dormants** : tant qu'une salle verrouillante est `Dormant`, ses ennemis portent
   `RoomDormant` ; sélection de cible, comportements, déplacement et attaque
   (`enemy_target_selection`, `behavior_select_system`, `update_enemy_targets`, `move_enemies`,
   `enemy_attack_system`) les sautent (`Without<RoomDormant>`). Annuler leur vitesse ne suffisait
   pas : `move_enemies` écrit la position.

**Limites connues (E2 et suite).** Les portes d'une salle sont repérées par leur position (marge
de 24 px), pas par un lien explicite ; deux salles qui se touchent par un coin partagent
d'éventuelles portes voisines. Le mode `Floors` ne remet pas `RoomStates` à zéro au changement
d'étage. Pas de récompense, de minicarte ni de grammaire d'étage (E2).

**Scénarios et attentes.** `RoomState(kind: "combat", nth: 0, state: Locked, doors_closed:
Some(4), at_frame: 100)` : état de la `nth` salle du type (triées par position x puis y) et,
optionnel, nombre de portes fermées de la carte. Moment clé `room` (« salle combat verrouillée »,
« … nettoyée »). Cartes `testbed/salles.ldtk` (deux `dummy`) et `salles_poursuivants.ldtk` (deux
`follower`), produites par `docs/taches/rapports/m2-e1-salles.make_salles.py` depuis
`two_rooms_door.ldtk` ; scénarios `rooms_lock_solo`, `rooms_dormant`, `rooms_lock_coop`,
`rooms_door_occupied`. Diagnostic : `ALACOD_MAP_PROBE=<scénario>:<frame>` affiche aussi les salles
(état) et les ennemis (position, dormant).

## 36. Objets et inventaire (M2-T0b, chantier C2)

Contrats : crate `items` (simulation, sans rendu) ; runtime : `game::items` ; contenu :
`content::registry::ItemEntry` et `content::lint::lint_items`.

**Contenu.** Kind de dossier `Item` : un fichier RON par objet (`objets/<id>.ron`, id = nom de
fichier), champs de `items::ItemDef` :
```ron
(name: "Bottes", kind: Passive,                       // Passive | Active(charge) | Consumable
 modifiers: [(stat: MoveSpeed, op: Pct, value: "0.25")],   // permanents, source `item:<id>`
 effects: [],                                          // effets T1.10 tels quels (voir ci-dessous)
 tags: ["mobilite"], rarity: Common,                  // Common | Uncommon | Rare | Epic | Legendary
 pickup_range: "24.0")                                 // défaut 24
// actif : kind: Active(Rooms(1)) | Active(Damage(200)) | Active(Frames(600))
//   effects: [(on: OnUse, do: [TimedModifier(stat: Damage, op: Pct, value: "1.0", frames: 300)])]
```
`ActiveCharge` : `Rooms(n)` salles nettoyées (`world::RoomChanged` vers `Cleared`, §35),
`Damage(n)` dégâts infligés par le porteur (`DamageEvent`, partie entière), `Frames(n)`. L'actif est
prêt quand sa charge atteint `n` (plafonnée).

**Lint.** Id dupliqué (`DuplicateId`), RON ou stat inconnue (`Parse`), charge 0 et `pickup_range`
≤ 0 (`OutOfRange`), consommable avec modificateurs ou effets et tout effet autre que `OnUse` d'un
actif avec `TimedModifier`/`Modifier` (`Unsupported` : contrats des effets v2, M2-T3).

**Inventaire.** `items::Inventory { passives (ordre de ramassage), active: Option<ActiveSlot
{ item, charge }>, consumables: BTreeMap<id, n> }`, composant rollback **neutre**, posé au
**premier ramassage** (jamais sur un joueur qui n'a rien ramassé : contribution 0, traces
inchangées). `ItemPickup { item_id }` : objet au sol (rollback, neutre, `GgrsNetId`). `ItemTable`
(statique, hors rollback) est posée au chargement depuis le registre
(`map_ldtk::loader::item_table`). `ItemPicked` : `FrameEvents` neutre (moment clé `item_pickup`).

**Comportement** (`game::items`). Consommable : ramassé au contact (portée de l'objet), le plus petit
`GgrsNetId` à portée l'emporte. Passif/actif : `Interactable { Item }` (`InteractionType::Item`), ramassé
par Interaction ; un passif ajoute ses modificateurs à `Modifiers` ; un actif remplace l'actif tenu
(qui tombe aux pieds, charge perdue) avec une charge 0. Charge (`item_charge_system`, après `Run` : lit
les `RoomChanged` de la frame). Usage : bit `INPUT_USE_ACTIVE` avec l'actif chargé : les effets `OnUse`
s'appliquent, la charge retombe à 0.

**Input élargi (u16 → u32).** `BoxInput.buttons` passe de 16 à 32 bits (`BoxInput` : 8 → 12 octets en
mémoire, alignement 4) : `INPUT_USE_ACTIVE` (bit 16, touche Espace, bouton `UseActive`) et `INPUT_BLANK`
(bit 17, touche Q, bouton `Blank` ; contrat seul, l'effet vient de M2-T1). Les enregistrements ne
stockent que des noms de boutons : ils se rejouent à l'identique.

**Scénarios.** `items: [(id: "bottes", x: "-904.0", y: "-584.0", at_frame: 5)]` place un objet à une
position et une frame exactes (comme `powerups`). Attentes : `HasItem(handle, id)`,
`ItemCharge(handle, charge)` (échoue sans actif), `Consumable(handle, id, count)`. Moment clé
`item_pickup`. Scénarios : `item_passive`, `item_active_rooms` (carte de M2-E1), `item_consumable_race`,
`item_passive_race`. Objets du testbed : `bottes`, `fiole` (`Active(Rooms(1))`), `key`.
---

## 37. Boss à phases (M2-T0c, chantier D4)

Contrats : `behaviors::boss` (`BossDef`, `Phase`, `PhaseEnd`, `Timeline`, `BossState`,
`BossPhaseChanged`, fonctions pures `BossDef::next_phase` et `Timeline::due`) ; exécution :
`game::boss` ; contenu : champ `boss` d'un personnage RON ; lint : `content::lint::lint_boss`.

```ron
boss: Some((phases: [
    (until: Some(HealthBelow("0.5")),            // HealthBelow(ratio ]0, 1]) | AfterFrames(n > 0)
     behaviors: [Chase(profile: "Ground")],       // remplace les règles de l'ennemi (vide : inchangées)
     on_enter: [Heal("5.0")],                     // effects::Action, à l'entrée (y compris la phase 0)
     timeline: Some((events: [(at: 10, do: [SpawnPattern(pattern: "volee", weapon: "fireball_gun")])],
                     repeat: Some(150)))),        // frise rejouée toutes les 150 frames ; None : une fois
    (timeline: Some((events: [(at: 0, do: [SpawnPattern(pattern: "couronne", weapon: "fireball_gun")])],
                     repeat: Some(90)))),         // dernière phase : pas de `until`
]))
```
Les options d'un personnage s'écrivent `Some(...)` (pas d'`implicit_some`).

**Transitions.** `boss_phase_system` (`RollbackSystemSet::EnemySpawning`, après les dégâts de la
frame, avant l'IA) date l'entrée dans la phase 0 au premier tick, puis ne fait **qu'une transition
par frame**, dans l'ordre du RON : `HealthBelow` quand santé/santé max est **strictement** sous le
ratio, `AfterFrames` quand `frame - entrée >= n`. À l'entrée : `on_enter` puis, à chaque frame,
les événements dus de la frise. La frise n'a pas de curseur : `due(frame - entrée)` (modulo
`repeat`) se déduit de l'état, donc identique après un rollback. Trace
`ggrs{f=… boss_phase net_id=… from=… to=…}` ; `FrameEvents<BossPhaseChanged>` (neutre).

**Règles par phase.** `EnemyBehaviors::phase_rules` (statique) garde la liste de chaque phase ; l'IA
lit `EnemyBehaviors::active(BossState::phase)` (sélection, déplacement imposé, attaque,
`melee_hold`, attente `EnemyState`). **Limite levée par M2-T7 (§45)** : la règle `Shoot` d'une phase
porte sa propre table de tir (arme, pattern, portée, recharge) ; sans `Shoot` de phase, `ai.ranged` de
la config ; un tir peut aussi passer par la frise (`SpawnPattern`).

**Actions.** Même exécuteur que les effets de personnage (`effects_runtime::apply_action`) avec un
`EffectState` jetable : `GaugeAdd` n'a pas d'effet durable sur un boss ; `SpawnPattern` est ignoré
(avertissement) tant qu'un émetteur joue déjà. Le lint refuse en plus `RefillAmmo*`,
`RepairAllWindows`, `KillAllWaveEnemies`, `DestroyTerrain`, `ApplyStatus`, `CurrencyMultiplier`.

**État.** `BossState { phase, entered, started }` : composant rollback, checksum et trace en
variante **neutre** (seuls les boss en portent). `BossPlan` (la définition) est statique.

**Lint.** Au moins une phase ; seule la dernière peut ne pas avoir de `until` ; `HealthBelow` dans
`]0, 1]`, `AfterFrames` > 0 ; phase non vide ; frise : `repeat` > 0, `at` < `repeat`, événements non
vides ; règles et `SpawnPattern` (pattern et arme connus) comme dans un personnage. Fixtures
`boss_health_out_of_range`, `boss_phase_empty`, `boss_missing_until`, `boss_unknown_pattern`,
`boss_timeline_after_repeat`.

**Scénarios.** Attente `BossPhase(entity: Placed(0), phase: 1, at_frame: 100)` (échoue sans
`BossState` ou si l'entité est morte) ; moment clé `boss_phase` (« boss 36 : phase 0 → 1 »).
Personnage `testbed/assets/characters/gardien.ron` (120 PV, immobile : volée `volee` puis couronne
sous 50 %), scénario `boss_two_phases` (phase 1 à f90, mort à f133).

---

## 38. Profil de méta-progression (M2-T0c, chantier G1)

Crate `meta` (hors simulation, dépend de `run`) et glue `game::profile`. Décision §7 n° 8 : RON
versionné, un profil par joueur, écrit hors simulation.

```ron
(version: 1, id: "local-0", currencies: {"essence": 11}, unlocks: ["first_run", "first_victory"],
 stats: {"runs": 1, "kills": 0, "frames": 50, "wave_best": 1, "floor_best": 0, "victories": 1})
```
**Format.** `meta::Profile { version, id, currencies, unlocks, stats }` (`BTreeMap`/`BTreeSet`).
`Profile::from_ron` migre par `version` (v0 = sans champ : devient v1 ; version plus récente que le
jeu : `ProfileError::TooNew`, fichier non touché). Écriture atomique (`meta::save` : fichier
temporaire puis renommage) ; un fichier illisible est signalé et jamais écrasé.

**Dossier.** `ALACOD_PROFILE_DIR`, sinon `$XDG_DATA_HOME/alacod/profiles`, sinon
`~/.local/share/alacod/profiles` ; en headless (`ALACOD_HEADLESS`) rien n'est écrit sans
`ALACOD_PROFILE_DIR`. Identifiant : `local-<handle>` (la pubkey allumette n'est pas encore connue
ici).

**Écriture.** `game::profile::write_profiles_at_run_end` (`Update`, jamais `GgrsSchedule`) : quand
`Run` passe `Ended` avec un `RunSummary`, `Profile::apply_run` crédite chaque joueur local (sans
`LocalPlayer`, tous les joueurs de la run) puis le fichier est écrit, une fois par run. Règles
provisoires (M2-T12) : stats cumulées (`runs`, `kills`, `frames`, `victories`, meilleures
`wave_best`/`floor_best`), essence = 1 par run + 1 par kill + 10 par victoire, déblocages
`first_run`/`first_victory`. Lecture au lancement dans la ressource `meta::Profiles` ; **la
simulation ne la lit jamais** dans cette tâche.

**Scénarios.** Un scénario n'écrit rien, sauf `profile: true` (dossier temporaire propre au
scénario). Attente `ProfileHas(handle: 0, unlock: "first_run", currency: ("essence", 11),
at_frame: 60)` (déblocage et/ou monnaie minimale ; évaluée après la fin de run). Scénario
`profile_run_end` (`idle` de zombies avec `wave_overrides: (max_wave: 1)` : victoire immédiate).

---

## 39. Le jeu `gungeon` (M2-T0d, squelette)

Dossier `games/gungeon/` (binaire `gungeon`, `make gungeon`, copié de `throne` puis élagué),
manifeste `assets/game.ron`. Il montre les contrats de la vague 0 de M2 ensemble : salles typées
(§35), objets (§36), boss (§37) ; le profil (§38) s'écrit à la fin de la run.

**Contenu.**
- `characters/` : `pistolero` (le joueur, id moteur `player`, course nerveuse et roulade à i-frames
  du §34), `bullet_kin` (ennemi tireur : `Shoot` du pattern `visee`, `KeepDistance`, `Chase`),
  `gatling` (boss immobile ; trois phases depuis M2-T7, §45 : éventail, spirale, rage avec renfort), `cible` (cible des scénarios
  générés d'armes).
- `weapons/weapons.ron` : `pistolet` (le moteur n'a pas de munition infinie : réserve de 9999
  chargeurs de 12, rechargement rapide ; `Automatic` : tenir la gâchette tire en continu) et
  `arsenal` (projectiles des patterns ennemis) ; `patterns/` : `visee` (`Aimed`), `couronne`.
- `objets/` (kind `Item`) : `bottes` (passif), `fiole` (actif `Rooms(2)`), `key` et `blank`
  (consommables, compteurs seulement). `rooms/` (kind `Room`) : `depart`, `combat` et `boss`
  (verrouillantes : `locks: true`).
- `maps/etage_1.ldtk` : l'étage, trois salles `Depart`, `Combat` (deux `bullet_kin`) et `Boss`
  (`gatling`) assemblées par `Basic` (voie B), produit par
  `docs/taches/rapports/m2-t0d-squelette-gungeon.make_etage.py` (qui appelle le script de M2-E1) ;
  `maps/gabarit_armes.ldtk` : gabarit des scénarios générés.
- `floors/etage_1.ron` : séquence `Floors` à un étage avec **`victory_at_end: true`** (nouveau champ
  optionnel, faux par défaut : throne et le testbed bouclent toujours sur le dernier niveau) : le
  dernier niveau vidé de ses ennemis (le boss est le dernier) termine la partie par une victoire
  (`floors_victory`, `FloorPlan::victory_at_end`, `floor_portal_open_system`), au lieu d'ouvrir un
  portail. Défaite à la mort (universelle).
- Sprites, sons, interface et caméra : placeholders copiés de `throne` (`assets.yaml`), aucun asset
  nouveau ; le butin de power-ups est désactivé (`drop_chance: "0.0"`).

**Scénarios.** `gungeon_start` (le clone démarre : joueur vivant, trois salles dormantes, trois
ennemis), `gungeon_clear` (bottes par Interaction, clé au contact, verrouillage à f75, nettoyage à
f231, réouverture), `gungeon_boss` (salle du boss verrouillée f291, phase 1 f448, boss mort f568,
victoire f569), plus les scénarios générés de `make gen GAME=gungeon`
(`tests/scenarios/generated/gungeon/`). Un joueur scripté vise des positions mesurées (les
`bullet_kin` se cornent à l'est de leur salle par `KeepDistance`) ; les bots ne conviennent pas
encore (conçus pour les zombies et leurs fenêtres).

**Vérifications.** `make lint` couvre `games/gungeon` ; le test `content/tests/embedded.rs` compare
registre natif et registre embarqué pour `zombies`, `throne` et `gungeon` ; le jeu n'est pas encore
dans les scripts de build web (`zombies` et `throne` seulement).

## 41. Projectiles v2 et blank (M2-T1, chantier B5 v2)

Code : `combat::projectile::Pattern` (variantes), `combat::emitter` (état multi-salves),
`items::{BlankSpec, BlankUsed}`, `game::items::{blank_system, BlankState, FloorStartConsumables}`,
`content::lint` (règles), contenu `gungeon` et `testbed`.

**Patterns ajoutés** (en **fin** d'enum `Pattern` : le hash des variantes d'avant ne bouge pas) :

| Variante | Effet | Où |
|---|---|---|
| `Fan(count, spread, speed, projectile)` | `count` tirs répartis **exactement** sur `spread` radians (bornes comprises), centrés sur la visée, sans hasard (à la différence de `Scatter`) ; `speed` > 0 remplace celle du projectile, 0 = celle de la définition | émetteur **et** `on_expire` |
| `Spiral(count, step, shots, every, speed, projectile)` | `shots` salves espacées de `every` frames, chacune de `count` bras régulièrement répartis sur le cercle ; la salve `k` tourne de `k × step` radians par rapport à la visée figée | émetteur seulement |
| `Burst(count, per_burst, interval, spread, projectile)` | `count` salves de `per_burst` tirs en éventail de `spread`, espacées de `interval` frames | émetteur seulement |

Une étape multi-salves (`Spiral`, `Burst`) **bloque la séquence** comme un `Wait` : la salve 0 part tout de
suite, la suivante `every`/`interval` frames plus tard, l'étape suivante après la dernière salve (une salve
de fond `Ring.every` continue de tirer). L'état d'avancement est `Emitter::multi` (`MultiVolley { step,
fired }`), dans l'état rollback de l'émetteur ; il n'est **haché que s'il existe** (hacher un `None`
déplacerait le checksum de tous les émetteurs d'avant ; test `spirale_ne_change_pas_le_hash…`). Aucun hasard :
le flux RNG `patterns` n'est pas consommé. Frames mesurées (`pattern_spiral`) : émetteur posé à f6, télégraphe
de 10 frames, salves à f17, f23, … (une salve toutes les `every` frames).

**Lint.** `count`, `per_burst`, `shots` > 0 ; `every` > 0 (`Spiral`) et `interval` > 0 (`Burst`) ; `spread`,
`speed` ≥ 0 ; `Spiral`/`Burst` refusés en `on_expire` (même au travers d'un `Named`), `Fan` admis. Fixtures
`pattern_spiral_on_expire`, `pattern_burst_interval_zero`.

**Balles ennemies qui rebondissent.** `Bounce(n)` joue pour un projectile de n'importe quelle équipe (la
chaîne `projectile_collision_system` ne regarde pas l'équipe pour les murs) : aucun correctif nécessaire,
prouvé par `enemy_bullet_bounce`. `gungeon` : `salve` de `bullet_kin` tire des `ricochet` (`Bounce(2)`).

**Blank.** Le consommable `blank` porte `blank: Some((radius: "140.0", iframes: 10))` (`items::BlankSpec`
dans `objets/blank.ron`, lint : consommable seulement, `radius` > 0). Au **front montant** de `Blank`
(`INPUT_BLANK`, touche Q), `blank_system` (`RollbackSystemSet::Effects`, après l'usage d'actif) :
1. consomme un exemplaire du premier objet porteur de `blank` que le joueur possède (ordre des ids) ;
2. efface (`despawn_rollback`) toute balle dont l'équipe de tir n'est ni celle du joueur ni
   `Allies`/`Neutral`, dans `radius` du joueur (jamais celles des joueurs) ;
3. rend le joueur invulnérable `iframes` frames (`Health::invulnerable_until_frame`, comme le dash) ;
4. émet `FrameEvents<BlankUsed { frame, player_handle, x, y, radius, cleared }>` (neutre ; lu par la
   présentation et par `OnBlank` de M2-T3) et la trace `ggrs{f=… blank net_id=… item=… cleared=…}`.
Sans blank en inventaire (ou sans inventaire) : rien. **Mémoire du front montant** :
`BlankState { held, floor }`, composant rollback **neutre**, posé sur un joueur qui a un `Inventory`
**et** un motif d'en avoir (Blank appuyé, blank en poche, ou recharge d'étage dans le contenu) : un joueur
avec inventaire mais sans blank (objets du testbed) ne le porte pas, donc sa trace ne bouge pas (une
première version le posait sur tout inventaire et déplaçait 4 traces du testbed). Réécrit à chaque frame
**avant** tout retour anticipé (piège « donnée dérivée non rollbackée »). Seuls `gungeon` et les scénarios
qui donnent un blank en portent : les traces des autres jeux ne bougent pas.

**Piège des tables de projectiles.** Les `projectiles:` d'une arme sont dans l'état hashé de ses porteurs :
ajouter un projectile à une arme existante déplace les traces de tous ses porteurs (une première version
l'avait fait sur `fireball_gun` : 33 traces). Les tourelles de motifs du testbed ont donc leur propre arme
`tourelle_gun` (copie de `fireball_gun` + `brume`, `ricochet`).

**Preuve p2p.** `ALACOD_PRESS_BLANK_AT=<frame>` (`player/input.rs`, `press_blank_at`) ajoute `INPUT_BLANK` à
l'input neutre de cette frame : permet de jouer un blank dans un test p2p sans clavier
(`ALACOD_INPUT=neutral`).

**Consommables de départ et d'étage (données).** `CharacterConfig::starting_consumables: {"blank": 2}`
crée l'`Inventory` du joueur à la création (vide par défaut : aucun inventaire). `Floors` :
`consumables_per_floor: {"blank": 2}` dans `floors/<id>.ron` remonte chaque compteur **à au moins** `n` au
début de chaque étage suivant (ressource hors rollback `FloorStartConsumables`, posée par `map_ldtk` au
chargement ; `BlankState::floor` retient l'étage servi). Lint : id consommable connu (`BrokenReference`,
fixture `character_starting_consumable_unknown`).

**Présentation.** `game::items::BlankPresentationPlugin` (`PresentationPlugin`, absent en headless) : onde (cercle
qui s'étend jusqu'au rayon puis s'estompe, 24 frames) dérivée de `BlankUsed` dans `Update` (ressource
`LastBlank`, jamais lue par la simulation). Moment clé `blank` (scénarios, vidéos). HUD : pas de source
« compteur de blanks » dans `HUD_SOURCES` : à M2-T18.

**Scénarios.** Testbed (`arena.ldtk`, tourelles `tourelle_spirale|eventail|salve|ricochet|anneau`) :
`pattern_spiral` (salves f17-f47), `pattern_fan` (5 flèches f17), `pattern_burst` (2 flèches à f17, f25, f33),
`enemy_bullet_bounce` (rebond sur le mur sud vers f63), `blank_clears_bullets` (12 boules ennemies effacées
à f36, 2 balles du joueur gardées, compteur 2 → 1), `blank_empty` (sans blank, rien), `bench_bullets_room`
(535 balles ennemies vivantes, salle verrouillée, plancher `budgets.ron`). Piège de script : **des segments
d'input qui se chevauchent ne se combinent pas** (le premier gagne) : couper les segments (`Fire` seul,
puis `Fire, Blank` une frame, puis `Fire`). Diagnostic : test ignoré `bullet_lines`
(`ALACOD_BULLET_LINES=<scénario>:<de>:<à>` : balles, joueurs, inventaire, santé frame par frame).
`gungeon` : `bullet_kin` joue `salve`, le boss `gatling` `eventail` (phase 0) puis `spirale` (phase 1,
projectile `etincelle`), le pistolero a deux blanks (`gungeon_boss` en utilise deux contre la spirale).

---

## 40. Grammaire d'étage (M2-T10, chantier E2)

Un étage de salles typées, assemblé à partir de gabarits LDtk. Code : `world::FloorGrammar`
(donnée), `map::generation::floor` (planificateur), `map::generation::imp::floor`
(`FloorMapGeneration`, mode `MapGenerationMode::Floor(grammaire)` à côté de `Basic` et `Cave`).
`Basic` est inchangé (traces `zombies`/testbed identiques).

**Contenu.** Kind de dossier `FloorGrammar` : un fichier RON par grammaire, id = nom de fichier,
désignée comme carte par `floor:<id>` (`entry.start_map`, `Scenario.map`, `levels` d'une séquence
`Floors`) :

```ron
// floor_grammars/etage.ron — game.ron : (path: "floor_grammars", kind: "FloorGrammar")
(templates: "maps/salles.ldtk", grammar: (
    rooms: (8, 12),                                    // départ, boss et requises compris
    required: {"boss": 1, "boutique": 1, "recompense": 1},
    min_boss_distance: 3,                              // en salles
    // start: "depart", boss: "boss", filler: "combat", max_attempts: 64 (défauts)
))
```

`templates` est la carte LDtk dont chaque niveau est un gabarit, typé par son champ de niveau
`room_kind` (§35). Le départ est un gabarit `spawn: true`.

**Algorithme** (déterministe, `RollbackRng` de génération ; listes ordonnées, aucun `HashMap`) :
départ au centre ; remplissage par croissance aléatoire (tirage uniforme parmi les poses valides :
parent, connexion libre, gabarit compatible du bon type, sans chevauchement ni sortie de carte) ;
boss posé sur une salle de profondeur maximale, avec un gabarit à **une seule connexion**
(cul-de-sac, donc strictement le plus loin du départ) ; salles requises sur des salles de
profondeur ≤ celle du boss − 2 ; contrôle indépendant `validate_plan`. Échec : nouvelle tentative
avec une graine dérivée (`next_u32` du générateur principal), au plus `max_attempts`, puis
`FloorError::Exhausted` explicite (jamais de boucle infinie ; `NoTemplate` si un type n'a aucun
gabarit).

**Mesure** (1 000 graines, gabarits 10×10 tuiles du test) : génération moyenne 184 µs, max 11,8 ms,
1,05 tentative en moyenne, tailles 8 à 12 réparties uniformément.

**Lint** : `rooms` min > max ou < 2, `max_attempts` 0, `required` à 0 ou contenant le départ,
plus d'un boss, départ + boss + requises > max, `min_boss_distance` impossible (`OutOfRange`) ;
carte `templates` absente, type nécessaire (départ, boss, remplissage, requis) sans niveau de ce
`room_kind`, séquence `Floors` ou `start_map` vers une grammaire inconnue (`BrokenReference`).
Fixtures : `floor_bounds_inverted`, `floor_required_without_template`.

**Étapes 2 : `Floors` d'étages assemblés.** Une séquence `Floors` peut lister des `floor:<id>`
(`levels: ["floor:etage", "floor:etage"]`). Chaque entrée est **son propre monde** (emplacement) :

- *Un chemin d'asset par étage* : `floor://<emplacement>/<gabarits>` (`game::cave_assets`, comme
  `cave://` §21), lu physiquement dans le fichier des gabarits ; sans cela l'`AssetServer` rendrait
  le premier étage chargé à tous (`FloorPlan::slot_of_level` ne fusionne pas les entrées `floor:`).
- *Graine dérivée par emplacement* : `graine_de_la_run + emplacement × 0x3C6E_F372` (wrapping) ;
  l'emplacement 0 garde la graine de la run. Les ids réseau sont triés par contenu comme partout.
- *Ouvertures libres murées* (`GeneratedMap::close_unused_openings`) : une ouverture sans voisin
  reste ouverte en `Basic` ; dans un étage de grammaire elle est refermée.
- *Salles de l'étage courant seulement* : les mondes de tous les emplacements existent dès le
  départ et se superposent dans l'espace ; `rooms::CurrentRooms` ne présente aux systèmes de salle
  que l'emplacement de l'étage en cours. Sans cela les salles de l'étage 2 se verrouillaient et se
  vidaient pendant l'étage 1. Les portes du nouvel étage sont rouvertes à la transition
  (`RoomStates::doors_opened` remis à faux).
- Portail à la mort du boss (barycentre des `PlayerSpawn`, donc au centre de la salle de départ),
  `victory_at_end` sur le dernier étage.

**Piège : la séquence par défaut.** `entry.mode: Floors` prend la **première séquence par ordre
d'id** du dossier `Floors` (`run_state::resolve_run_mode_with_floors`). Ajouter un fichier qui
passe avant l'ancien change la partie par défaut : les séquences de grammaire de `gungeon` sont
nommées `etages_*` pour que `etage_1` (T0d) reste la première. Même piège pour tout nouveau jeu.

**Gungeon.** Gabarits `games/gungeon/assets/maps/salles.ldtk` (9 gabarits de 8×8 tuiles, script
`docs/taches/rapports/m2-t10-grammaire-etage.make_salles.py`) : `Depart`, `CombatEW/NS/T/Croix`,
`Recompense`, `Boutique`, `BossW/N`. Grammaires `etage` (8–12 salles), `grand` (15–20) et `court`
(3 salles : départ, combat, boss) ; séquences `etages_grammaire`, `etages_grand`, `etages_court`,
`etages_court_deux`. T0d (`etage_1`) reste la séquence par défaut : les traces `gungeon_*` d'avant
ne bougent pas, la grammaire est activée par `floors: Some("etages_...")`.

**Scénarios.** `gungeon_floor_seed_{1,23,123}` : étage `court` (même forme, départ, combat en T,
boss à l'est, pour ces trois graines : relevé par `ALACOD_FLOOR_DUMP=court` du test
`gungeon_etages_de_grammaire`), joueur scripté jusqu'à la mort du boss. `gungeon_two_floors` :
graine 123, deux étages `court` de même forme, `FloorIndex`. `bench_floor` : étage `grand`, deux
bots `fonceur` (plancher dans `tests/budgets.ron`). L'atteinte du boss est scriptée : les bots
nettoient les salles de combat mais ne vont pas vers une salle dormante (bot explorateur : T16).

## Notes essentielles

**`CrateLocation`** : collectée mais non utilisée, voir §1 (entités et vérification par un scénario `idle`).

**Colliders vs sprites** : voir §1 et `CLAUDE.md` (« Flow Field - Implémentation Actuelle ») ; le décalage du collider est `collider.offset` de `CharacterConfig`, celui du sprite `offset_x`/`offset_y` de `SpriteSheetConfig`.

**IntGrid** : les surfaces (E4 v1) ont leur propre couche IntGrid `Surfaces`, sans toucher `Walls` : voir §26.

**Fixed-point** : voir §2 et `CLAUDE.md` (Déterminisme, règles 1 et 7).

**Tests et scénarios** : `make test_scenarios` vérifie que les scénarios passent en synctest (mode déterministe local). Tout changement de code dans `GgrsSchedule` peut casser les traces `.trace` ; rebase sur `main` chaque jour et revalider en CI rapide (preuve d'un bless : §10). Une tâche de toute voie peut bénir des traces, **avec la preuve du §10**, bénies par l'orchestrateur (`BLESS=1`, justification dans le commit) ; V1 (simulation) est la voie qui les change le plus souvent.

### Brouillard d'exploration opt-in (revue M0, Le Relais)

`game::room_fog::FogMap` est une ressource de présentation immuable décrivant une grille
LDtk de visibilité : dimensions, origine monde, murs, fenêtres, identifiants de pièces
et portes avec leurs deux extrémités. Elle n'est ni un état de salle M2 ni un état de
combat. La présence de `settings.enabled = true` active le rendu ; sans métadonnées,
les autres jeux conservent leur affichage.

Pour `zombies`, régler `games/zombies/assets/ui/fog.ron`, puis lancer
`python3 scripts/construire-modules-le-relais.py`. Ce script conserve les réglages RON
comme champ `fog_settings` des gabarits ; le générateur produit le champ `fog_layout`
avec les limites exactes de la carte assemblée et ses connexions. Les rectangles de
défense des spawners ne servent pas au champ de vision. La génération reste à un
niveau final fusionné : ne pas confondre ces identifiants avec des `LevelId` distincts.

Le champ de vision utilise les murs et l'état des portes. Les fenêtres laissent passer
la vue avec une portée limitée ; les obstacles bas ne sont pas déduits de leur seule
collision de mouvement. Dans Le Relais, les cases IntGrid `Walls` sont des occluders
pleins, y compris le mobilier actuellement dessiné dans cette couche. Pour du mobilier
bas transparent à la vue, fournir une grille de visibilité différente de celle-ci.

L'observation `capture_vision` est installée seulement par `PresentationPlugin` dans
`GgrsSchedule` : lecture des positions Fixed et des portes, écriture d'un historique
**hors simulation**, sans RNG, dégâts, composants rollback ni effet sur les checksums.
Une resimulation invalide les échantillons prédits ultérieurs. Seules les frames
`ConfirmedFrameCount` alimentent la mémoire explorée. Le calcul de visibilité et la
texture RGBA se font hors GgrsSchedule ; une fermeture masque immédiatement et seule
la révélation est interpolée. La mémoire et les assets sont nettoyés en fin de partie.

Les sprites dynamiques et leurs enfants sont filtrés hors de vue, avec conservation
de leur alpha propre. Les bonus au sol, leurs noms, les chiffres de dégâts,
télégraphes et prompts d'interaction lisent aussi la visibilité courante. Ne pas
restaurer `Visibility` sur les armes : cela pourrait rendre visible une arme inactive.
Le HUD écran reste hors du masque. Toute extension des effets visuels doit respecter
ce filtrage si elle affiche des informations sur une entité cachée.

Les connexions des portes déterminent quelles pièces sont accessibles à l'exploration,
sans les révéler entièrement. Le champ de vision révèle ensuite les seules cases en
vue. Ce filtre empêche une vue par une fenêtre extérieure de dévoiler les objets d'une
pièce encore fermée. Lorsqu'une partie d'une porte est en vue, sa face complète reste
lisible dans les cases de mur du passage ; cela ne révèle aucune case intérieure.


**Séquence par défaut et preuve p2p (D59, 2026-10-15).** `entry.floors` (champ optionnel de
`game.ron`, `content::manifest::EntryPoint`) désigne la séquence du dossier `Floors` que `entry.mode:
Floors` joue par défaut ; absent, c'est l'ancien comportement (la première par ordre d'id). Lint : un id
absent du dossier est une erreur `BrokenReference`, le champ sans `mode: Floors` une erreur `Unsupported`
(fixtures `entry_floors_unknown` et `entry_floors_without_mode`). Un scénario garde la main par son champ
`floors`. `gungeon` déclare `floors: "etages_deux"` : la partie ordinaire démarre sur deux étages assemblés
par la grammaire `etage` (l'étage squelette `etage_1` reste jouable par `floors: Some("etage_1")`).
Diagnostic `ALACOD_CLEAR_ENEMIES_AT=<frame>` (`map_ldtk::game::floors`, avec `ALACOD_INPUT=neutral`, à régler
sur **tous** les clients) : à cette frame tous les ennemis disparaissent (`despawn_rollback`), le portail
(barycentre des `PlayerSpawn`) s'ouvre par la règle ordinaire et les clients traversent la transition sans
clavier. Preuve p2p de `gungeon` (2 clients, `ALACOD_CLEAR_ENEMIES_AT=200`, 900 frames) : séquence
`etages_deux` choisie, `floor_portal_open` puis `floor_transition from=0 to=1` à f200 sur les deux clients,
traces identiques sur 899 frames.

## 42. Effets v2 : déclencheurs, conditions, cooldowns et charges (M2-T3, chantier C1 v2)

Suite de §27 (effets v1) et §36 (objets). Code : contrats `effects::{On, Condition, Effect,
Recharge}`, règles pures `effects::runtime` (`responds_to`, `conditions_hold`, `Limits`,
`limits_allow/consume/recharge`), exécution `game::effects_runtime`
(`apply_effects_system` v1, `apply_triggered_effects_system` v2).

**Déclencheurs exécutés** (le lint refuse les autres) :

| Déclencheur | Source lue | Système / ordre |
|---|---|---|
| `OnRoomClear` | `FrameEvents<RoomChanged>` vers `Cleared`, **si le porteur est dans les bornes de cette salle** (iid de l'événement) | v2, après `Run` |
| `OnDodge` | `FrameEvents<sim_core::triggers::DashStarted>` (émis par `apply_inputs` au début du dash) | v2 |
| `OnReload` | `FrameEvents<sim_core::triggers::ReloadFinished>` (émis par `weapon_rollback_system` quand le chargeur est **réellement rempli** ; le délai de pompe d'une arme sans chargeur n'en émet pas) | v2 |
| `OnPickup` | `FrameEvents<ItemPicked>` du porteur (objet ramassé, **le sien compris**) | v2 |
| `OnFloorStart` | changement de `FloorState::index` vu par le porteur ; le premier étage compte pour un effet présent aux frames 0–1 (un effet vu plus tard, par exemple un objet ramassé en cours d'étage, ne se déclenche qu'au **prochain** étage) | v2 |
| `OnBlank` | `FrameEvents<BlankUsed>` (M2-T1) du porteur (`player_handle`) | v2, **après `blank_system`** (qui émet dans `Effects`) |

Les nouveaux variants (`OnFloorStart`, `OnBlank`) sont en **fin d'enum** `On`. Le système v2 tourne
`.after(RollbackSystemSet::Run).after(item_charge_system).after(blank_system).before(FrameCounter)` : `RoomChanged`
est émis dans `Run` et vidé au `FrameStart` suivant, les autres événements dans des sets plus tôt
de la même frame. Les événements `DashStarted`/`ReloadFinished` sont des `FrameEvents` neutres
(file vide : contribution 0 au checksum).

**Conditions** : `Carrying(item)` (objets tenus, passifs et actif) et `SquadSize(n)` (au moins `n`
joueurs en vie, ni `Death` ni `Downed`) sont exécutées. `TargetInRange` reste un contrat (aucune
position de cible dans les déclencheurs) : le lint la refuse.

**Cooldown et charges** : champs optionnels de l'effet, `cooldown: Some(frames)` (délai minimal entre
deux exécutions), `charges: Some(n)` (exécutions avant épuisement), `recharge: Some(PerRoom |
PerFloor)` (retour de **toutes** les charges à la salle nettoyée ou au début d'étage ; `PerRoom`
suit `RoomChanged` de n'importe quelle salle du porteur, `PerFloor` le changement d'index).
Sans ces champs l'effet est celui de la v1. La condition est évaluée **avant** les limites (une
condition fausse ne consomme rien) ; une exécution consomme la charge et arme le cooldown.
Lint : `cooldown = 0`, `charges = 0`, `recharge` sans `charges` sont refusés.
État : `EffectState::limits` (clé `effect:<index>` ou `item:<id>:<index>`).

**Pas de trace qui bouge** : `Effect` et `EffectState` ont un `impl Hash` manuel qui écrit les champs
v2 **seulement s'ils ne sont pas vides** ; un porteur sans effet v2 garde le hash d'avant
(`effet_v1_garde_son_hash`, `effect_state_sans_champ_v2_garde_le_hash_de_t1_10`).

**Effets des objets (choix : lecture dans l'`Inventory`)**. Un objet passif ou actif tenu applique
ses `effects` (hors `OnUse`, exécuté par `game::items`) au porteur : les systèmes lisent
l'`Inventory` à chaque frame au lieu de copier les effets dans `Effects` au ramassage. Raisons :
un objet perdu (actif remplacé) n'a rien à retirer ; pas de composant en plus sur le joueur (donc
pas de hash qui bouge pour les joueurs sans objet à effets) ; l'index d'effet est celui du
`ItemDef`, stable. Le porteur qui n'a pas d'`EffectState` en reçoit un au premier effet exécuté.
Source des modificateurs d'un effet d'objet : `item:<id>:<index>` (distincte de `item:<id>`,
celle des `modifiers` de l'objet). Ordre : porteurs par `GgrsNetId`, effets du personnage par
index puis ceux des objets (passifs dans l'ordre de ramassage, puis l'actif). Deux exemplaires du
même objet partagent leurs limites (clé identique). Un `Tick` d'objet compte depuis la première
frame où le système voit l'objet. Lint : un effet d'objet suit `lint_effect` ; `OnUse` n'a de sens
que sur un actif.

**Contenu** : `gungeon` : `trefle` (soin à la salle nettoyée, une fois par étage), `cartouchiere`
(cadence après un rechargement, cooldown 600), `amulette` (vie max et soin au ramassage, une charge).
`testbed` : neuf objets d'essai (dont `sceau`, `OnBlank`) (`anneau_salle`, `semelles`, `semelles_lentes`, `bandoulier`,
`bandoulier_comptee`, `talisman`, `boussole`, `porte_bonheur`), dont l'effet est un `Modifier(Damage,
Add, 1)` : la valeur de `Stat(Damage)` compte les exécutions.

**Scénarios** : `effect_on_room_clear`, `effect_on_dodge`, `effect_on_reload`, `effect_on_pickup`,
`effect_on_floor_start` (aussi la recharge `PerFloor`), `effect_cooldown`, `effect_charges`
(recharge `PerRoom`), `item_passive_effect` (`OnKill` d'un objet), `effect_on_blank` (passif
`sceau` : +1 de dégâts à chaque Blank utilisée, `testbed`).

**Règle d'engine : `add_frame_events_transient`** (M2-T3). Un événement émis dans des parties
existantes et lu seulement dans la même frame (`DashStarted`, `ReloadFinished`) s'enregistre avec
`add_frame_events_transient::<T>()` : file hors checksum et hors snapshot, vidée au `FrameStart`.
Les lecteurs sont ordonnés `.after` l'émetteur (ici le système d'effets v2, après `Run`, donc
après `Input` et `Weapon`) ; un lecteur placé avant verrait la file de la frame précédente. La
variante neutre ne convient pas : sa file non vide est hachée et déplace les traces existantes
dès qu'un dash ou un rechargement a lieu.

## 43. Esquive v2 (M2-T2, chantier B6)

Code : `combat::actors::DashExtra` (état), `game::character::movement` (`DashConfig`, `DashEffect`,
`IframesKind`, `MovementConfig::resolve_dash`), `game::character::player::input::apply_inputs`
(lancement), `game::character::dash::dash_shove_system` (bousculade), `content::lint::lint_dash`.

**`DashState` ne change pas.** Il est sous checksum et porté par tous les joueurs : le moindre champ
de plus déplacerait toutes les traces dès la frame 0. L'état nouveau vit dans un composant séparé,
`DashExtra { charges, max_charges, recharge_remaining, projectile_iframes_until, shove_force }`,
enregistré en variante **neutre** (`rollback_and_trace_neutral`) et posé à la création
(`character::create`) **seulement** si le personnage déclare `dash: Some(..)` ou
`movement.dash_charges`. Aucun porteur ailleurs : zombies, testbed, throne gardent leurs traces.
`is_dashing` passe à vrai à la frame de l'appui, comme en v1 (M2-T3 y branche `OnDodge`).

**Réglage.** `MovementConfig` gagne `dash_charges: Option<u32>` et `dash_recharge_frames`. Le RON
du personnage gagne `dash: Option<DashConfig>` ; chaque champ absent retombe sur le réglage v1 de
`movement` : `distance`, `duration_frames`, `cooldown_frames`, `iframes`, `iframes_kind`
(`All` v1 / `Projectiles`), `effect` (`Roll` v1 / `Shove` / `InPlace`, nouveaux effets en fin d'enum),
`shove_force`, `charges`, `recharge_frames`.
```ron
dash: Some((iframes_kind: Projectiles, charges: Some(2), recharge_frames: Some(60))),
```

**Charges.** Sans `charges` : v1 exact, le cooldown seul règle le rythme. Avec : un dash consomme une
charge (refusé à zéro, l'appui est gardé par le buffer comme en v1), `dash_cooldown_frames` reste
l'écart minimal entre deux dashs, une charge est rendue `recharge_frames` frames après la
consommation de la première manquante (le décompte repart tant qu'il en manque). Le test et le
décompte sont dans `apply_inputs`, avant le lancement : une charge rendue à la frame F est utilisable à F.

**Variantes.** `Roll` : v1. `Shove` : pendant le dash, chaque ennemi dont le collider touche celui du
joueur reçoit `Velocity.knockback = direction du dash × shove_force` (assigné, pas cumulé ; l'amortissement
habituel s'applique), ordre `GgrsNetId`. `InPlace` : distance 0 (les pas sont nuls), seulement les
i-frames ; pas de tir pendant la durée, comme tout dash.

**I-frames contre projectiles.** `iframes_kind: Projectiles` n'écrit **pas**
`Health.invulnerable_until_frame` : `DashExtra.projectile_iframes_until` (dernière frame comprise),
lu par la collision des balles (`weapons/mod.rs`) et des projectiles composables
(`projectile.rs`) ; elles traversent le porteur sans être consommées. La mêlée, les contacts, la
ruée (`Charge`) et tout autre dégât le touchent. `All` (défaut) : v1.

**Lint** (`OutOfRange`, seulement pour un personnage qui déclare `dash` ou `dash_charges`) :
`dash_charges = 0`, `dash_recharge_frames = 0` avec des charges, durée 0 hors `InPlace`, i-frames
plus longues que la durée. Fixtures `dash_charges_zero`, `dash_recharge_zero`, `dash_duration_zero`,
`dash_iframes_over_duration`.

**Scénarios** : champ `dash` de `PlayerScript` (réglage de scénario, `DashOverride` + `DashExtra`
posés avant la première frame par `apply_player_overrides` ; conservé au ré-enregistrement) pour
essayer une variante sans changer le contenu des jeux. `dash_charges` (gungeon : deux dashs à f15 et
f45, le troisième, refusé, part à f76 quand la charge de f15 est rendue), `dash_shove` (un
`follower` projeté, 52 px derrière contre 30 pour `Roll`), `dash_in_place` (position inchangée,
`NoDamageBetween` sur les i-frames), `dash_jump_bullets` (les balles de la `tourelle_salve` traversent,
la ruée du `charger` touche dans la même fenêtre).

**Contenu** : `gungeon/characters/pistolero.ron` : deux charges (une rendue toutes les 60 frames),
i-frames contre projectiles seulement. Les traces `gungeon` ne bougent qu'à cause de ce nouveau
composant (voir le rapport).

## 44. Objets v2 et synergies (M2-T4, chantier C2)

**Synergies.** Kind de contenu `Synergy` (dossier `synergies/`, `(path: "synergies", kind: "Synergy")`
dans `game.ron`), un fichier RON par synergie, id = nom de fichier, champs de `items::SynergyDef` :
`name`, `requires: [ids]` (deux objets distincts ou plus : passifs ou actif), `modifiers` et `effects`
(mêmes formes que `ItemDef`). Table `items::SynergyTable` (ressource hors rollback, posée au chargement
de la carte par `map_ldtk`, comme `ItemTable`).

Active **tant que tous** les objets requis sont tenus (`Inventory::has` : un passif ou l'actif
courant), retirée dès qu'un manque (l'actif remplacé tombe au sol). **État dérivé de l'`Inventory` à
chaque frame, aucun composant ni hash en plus** :
- `modifiers` : `game::items::synergy_modifiers_system` (`Effects`, après `blank_system`) pose les
  modificateurs de source `synergy:<id>` quand la synergie devient active, les retire (`remove_by_source`)
  quand elle cesse ; synergies par id, joueurs par `GgrsNetId`. Un joueur sans `Inventory` n'a pas de synergie.
- `effects` : `game::effects_runtime::listed_effects` ajoute, après les effets des objets, ceux des
  synergies actives (par id) sous la clé `synergy:<id>:<index>` (limites, `Tick`, source de modificateur
  `synergy:<id>:<index>`, comme les effets d'objet). Un effet de synergie suit `lint_effect` ; `OnUse` y est
  refusé. Le ramassage qui complète la paire déclenche déjà un `OnPickup` de la synergie (même frame).

**Actifs v2** (`ItemDef`, champs optionnels, v1 exact sans eux) :
- `uses_per_floor: Some(n)` : au plus `n` usages par étage du mode `Floors` (hors `Floors`, par partie).
  Mémoire dans `ActiveSlot::uses = Some((étage, usages))` : les usages d'un autre étage ne comptent plus,
  aucun reset à écrire. **`uses` n'entre dans le hash que s'il est posé** (`impl Hash` manuel d'`ActiveSlot`,
  comme `Effect` en §42) : le hash d'un actif v1 est celui d'avant. Un usage refusé garde la charge.
- `cost: Some((item, n))` : l'usage prélève `n` exemplaires du consommable `item` ; sans le stock, il est
  refusé (charge conservée). Lint : `item` connu et consommable, `n > 0`.
- `OnUse` accepte maintenant `Heal` (en plus de `Modifier` et `TimedModifier`) : `game::items::apply_instant_effects`.

**Consommables à effets.** Un consommable dont `effects` n'est pas vide (uniquement `OnUse`, lint) **s'applique
au ramassage** et n'entre pas dans l'inventaire : aucun bouton d'usage n'existe pour un consommable générique
(pas de nouveau bit d'input, accord d'orch). Actions : `Heal`, `Modifier`, `TimedModifier`. Un consommable
sans effet reste un compteur (`key`, `blank`, `obus`). Pas de refus de ramassage à pleine santé (le soin est
simplement borné).

**Lint** : synergie à objet inconnu (`BrokenReference`), à moins de deux objets distincts (`OutOfRange`),
à un consommable (`Unsupported`), deux synergies pour le même ensemble (`DuplicateId`), `OnUse` dans une
synergie ; `uses_per_floor = 0` (`OutOfRange`), `uses_per_floor`/`cost` sur un non-actif, `cost` vers un
objet inconnu ou non consommable. Fixtures `synergy_unknown_item`, `synergy_single_item`,
`synergy_duplicate_set`, `item_uses_per_floor_zero`, `item_cost_not_consumable`.

**Contenu.** `gungeon` : synergies `tresor_de_poche` (`trefle` + `amulette` : +10 de vie max) et
`tir_nourri` (`cartouchiere` + `etendard` : +25 % de cadence) ; actif `pierre_de_soin` (`Rooms(1)`, soin 20,
`uses_per_floor: 2`) ; consommable `ration` (soin 15 au ramassage). Pas de « bouclier temporaire » :
`StatId::Armor` n'est lue par aucun système (un actif sur cette stat serait sans effet), à reprendre quand
la réduction de dégâts existera. `testbed` : objets d'essai `lame`, `manche`, `braise`, `eclat`, `canon`,
`obus`, `ration`, synergies `lame_manche` (modificateur +2 et effet `OnPickup` +4) et `lame_braise` (+2).

**Scénarios** : `synergy_pair` (bonus avec les deux objets, absent avec un seul : dégâts 1 puis 7),
`synergy_lost` (actif remplacé : dégâts 3 puis 1), `item_active_per_floor` (trois appuis par étage, deux
effets, sur `deux_niveaux`), `item_active_cost` (obus 3 → 1, second usage refusé), `consumable_heal`
(santé 92 → 97 au ramassage). Aucune trace existante ne bouge : types neutres, champs optionnels hors hash.

## 49. Formations simples : escouades en arc et tir alterné (M2-T8, chantier D1 v2)

Code : `behaviors::{Behavior::Formation, FormationShape}`, `game::character::enemy::ai::squad`
(`formation_system`, `SquadSlots`, `arc_places`, `fire_turn`, `nearest`), `rules::behavior_motion`
(déplacement), `rules::behavior_select_system` (tour de tir), `content::lint::lint_formation`.

**Contenu.** Un ennemi se déclare membre d'une escouade par `ai.squad: Some(("id", Some("rôle")))` et
liste une règle `Formation` (variante **en fin d'enum** `Behavior`, aucune discriminante existante ne bouge) :

```ron
ai: Some((
    squad: Some(("bullet_kins", Some("arc"))),
    behaviors: Some([
        Shoot(weapon: "arsenal", pattern: "salve", range: "240.0", cooldown_frames: 90),
        Formation(shape: Arc(radius: "56.0", spread: "1.6"), period: 40),   // volley: true : tir groupé
        Chase(profile: "Ground"),
    ]),
)),
```

**Escouade = donnée dérivée.** Les ennemis vivants (hors salle dormante) de même `squad` qui listent une
`Formation`, rangés par `GgrsNetId` (le rang est l'indice dans cet ordre). `formation_system`
(`EnemyAI`, après `enemy_target_selection`, avant `behavior_select_system`) vide puis réécrit
`SquadSlots` à chaque frame, avant tout retour anticipé : ressource **hors rollback**, aucun état caché,
aucun nouveau type sous checksum. Une escouade vit tant que ses membres vivent ; elle ne dépend pas de la
salle (une seule salle d'escouade est éveillée à la fois dans `gungeon`).

**Cible et places.** La cible de l'escouade est le joueur debout le plus proche de son centre (moyenne
des positions), à distance égale le plus petit `GgrsNetId` (`nearest`). L'arc est centré sur la cible,
son milieu pointe vers le centre de l'escouade ; la place du rang `k` sur `n` membres est l'angle
`spread × (k/(n−1) − 1/2)` (le membre seul est sur l'axe). Les places ne changent pas tant que l'effectif
ne change pas ; un membre mort réattribue les rangs au prochain tour. Angles et trigonométrie en `Fixed`
(`cos_fixed`/`sin_fixed`). Écart avec la fiche : « plus proche par le chemin » est ici la distance
euclidienne au centre de l'escouade (pas de flow field par escouade).

**Déplacement.** À plus de `2 × radius` de la cible : mouvement d'origine (flow field, comme `Chase`).
Plus près : droit vers la place, puis, à moins de 6 unités, `Strafe` léger (quart de vitesse, sens
alterné toutes les 45 frames). **Tir alterné** : `Shoot` n'est candidat que dans la fenêtre du membre,
`(frame / period) mod effectif == rang` (`fire_turn`, état dérivé, aucun curseur) ; une séquence en cours
va à son terme. `volley: true` garde le tir groupé. Effets de bord : un `Shoot` plus long que `period`
chevauche la fenêtre suivante (les patterns d'escouade tiennent dans une fenêtre).

**Lint** (`OutOfRange`, fixtures `formation_*`) : `radius ≤ 0`, `spread` hors ]0, 2π], `period = 0` sans
`volley`, formation sans `Shoot` ni `Chase` dans la liste, formation sans `ai.squad`.

**Testbed** : `escouade_tireur` (trois `salve` en fenêtres de 40, arc de 100). **Gungeon** :
`bullet_kin_squad` (arc de 40, la salle est petite), niveau `EscouadeEW` de `maps/salles.ldtk`
(`room_kind: escouade`, type `rooms/escouade.ron`), grammaire `court_escouade` et séquence
`etages_court_escouade`. Ni `bullet_kin` ni les grammaires `court`/`etage`/`grand` ne changent : le nouveau
type de salle n'est tiré par aucune.

**Attentes** : `EnemyDistance` accepte `target: Entity(<EntityRef>)` (écarts entre membres).
**Scénarios** : `formation_arc` (convergence à 100, écarts angulaires 59/113), `formation_member_dies`
(le rang 0 tombe, les deux restants se placent aux extrémités), `formation_alternate_fire` (2, 4, 6 puis 8
flèches au lieu de 6, 12, 18), `formation_two_players` (arc autour du joueur le plus proche),
`gungeon_squad`.

## 45. Boss et arène (M2-T7, chantier D4)

Complète §37 (phases, frise) et §35 (salles verrouillées) : tir **par phase**, salle de boss,
arène à tenir, actions de frise `SpawnEnemies` et `Telegraph`.

**Tir par phase** (lève la limite de §37). Une règle `Shoot` d'une phase (`Phase::behaviors`)
utilise **sa propre** table de tir : arme (donc table de projectiles), pattern, portée, recharge.
`EnemyBehaviors::phase_shoot(phase)` la rend, `enemy_attack_system` la préfère à `ai.ranged` ;
`spawn_enemy` équipe en plus (inactives) les armes des `Shoot` de phase. Sans `Shoot` de phase,
`ai.ranged` comme avant (aucune trace ne bouge). Personnage `testbed/gardien_tir`.

**Salle de boss.** Type de salle `boss` (`locks: true`, §35). Le boss **dort** tant que sa salle est
`Dormant` : `boss_phase_system` saute les entités `RoomDormant` (la phase 0 est donc datée à l'éveil,
pas au premier tick du jeu : le boss ne tire plus depuis une salle fermée). À sa mort,
`boss_defeated_system` (`DeathManagement`) émet `FrameEvents<BossDefeated>` (**transitoire** :
`add_frame_events_transient`, hors checksum et hors snapshot, vidé à la frame suivante) puis
`map_ldtk::game::rooms::boss_room_clear_system` tue les autres ennemis de sa salle (renforts compris) :
la salle passe `Cleared` dans la frame (portes rouvertes, `RoomChanged`). `floor_portal_open_system`
ne bouge pas : le portail ou la victoire de fin d'étage viennent après, comme avant. Trace
`ggrs{f=… boss_defeated net_id=…}` et `boss_room_clear`.

**Actions de frise** (variantes **en fin d'enum** `effects::Action`, le hash des autres ne bouge pas) :
- `Telegraph(n)` : le prochain `SpawnPattern` de la même liste est précédé d'un télégraphe de `n`
  frames (l'émetteur joue `Sequence([Telegraph(n), pattern])`) ; aucun type nouveau.
- `SpawnEnemies(id, count)` : `count` personnages `id` apparaissent dans la salle du boss, positions
  tirées du flux RNG `room:<niveau>` (`world::room_rng_stream`) sur une case libre du profil `Ground`
  (16 essais, repli sur le centre). Demande `game::boss::ReinforcementRequest` (transitoire) émise
  par `boss_phase_system` et exécutée dans la même frame par `boss_reinforcements_system` (`Run`).
Refusées partout ailleurs que dans un boss (power-up, effet de personnage : `OutOfRange`).

**Arène à tenir.** Champ optionnel `arena` d'un fichier `rooms/<id>.ron` :
`(arena: Some((clock: 600, waves: [(at: 0, enemies: [("dummy", 2)]), …])))`
(`world::ArenaDef`, `ArenaWave`). Une arène verrouille **dès qu'un joueur debout entre**, même vide
(`RoomKindTable::locks` est vrai), et lance son horloge (`ArenaClocks`, ressource rollback **neutre**,
frame de verrouillage par niveau). Vagues : `arena_waves_system` (`Run`) fait apparaître les ennemis
d'une vague quand `frame - début == at` (aucun curseur : tout se déduit de l'horloge et de l'état de
salle). À `clock` frames, si un joueur vit, les survivants meurent et la salle passe `Cleared`
(portes rouvertes). Si tous les joueurs meurent, défaite universelle. Trace
`ggrs{f=… arena_wave …}` et `arena_kill`.

**Lint.** `lint_arenas` : `clock` > 0, au moins une vague, `at` < `clock` et croissants, vague non
vide, personnage connu (`BrokenReference`), `count` > 0 ; dans une frise : `SpawnEnemies` d'un
personnage inconnu, `count` 0, `Telegraph(0)`, `Telegraph` sans `SpawnPattern` après lui. Fixtures
`arena_no_waves`, `arena_clock_zero`, `arena_unknown_enemy`, `arena_count_zero`,
`boss_spawn_enemies_unknown`, `boss_spawn_enemies_count_zero`, `boss_telegraph_alone`.

**Déterminisme.** Types nouveaux tous neutres ou transitoires (`ArenaClocks`, `BossDefeated`,
`ReinforcementRequest`) : aucune trace hors `gungeon` ne bouge. Les flux `room:<niveau>` ne naissent
qu'au premier renfort ou à la première vague. Apparitions dans l'ordre : demandes par `GgrsNetId` du
boss, salles triées par position puis identifiant.

**Contenu.** `testbed` : `gardien_tir`, `gardien_renforts`, types de salle `boss`, `arene`,
`arene_mortelle`, cartes `salles_boss*.ldtk` et `salles_arene*.ldtk`
(`docs/taches/rapports/m2-t7-boss-arene.make_cartes.py`).

**Scénarios.** `boss_room_lock` (le boss dort : aucune flèche avant le verrou f85 ; réouverture à sa mort f184),
`boss_phase_shoot` (flèches de `fireball_gun` en phase 0, balle `ricochet` de `tourelle_gun` en phase 1, projectile
qui n'existe que dans cette table), `boss_reinforcements` (1 → 3 → 0 ennemis), `arena_survive` (verrou f85, rouverte
f685 = 85 + 600), `arena_fail` (le joueur immobile meurt f1514, la salle ne se rouvre pas), `gungeon_boss_bot` (le bot
`fonceur` bat le `gatling` à f403, victoire).

**`gungeon`.** `gatling` : 120 PV (150 avant), phase 0 `eventail` ; phase 1 (sous 50 %) `spirale` ; phase 2 (sous
10 %, la rage) `SpawnEnemies(bullet_kin, 1)` puis `Telegraph(10)` + `couronne`. Les scénarios scriptés de M2-T0d,
T1, T3 et T10 gardent leurs entrées : un renfort plus tôt retardait la mort du boss de plus de 40 frames (il
bloque des balles et le joueur scripté meurt à quelques points de vie près) ; la rage se déclenche donc à la
dernière touche (sous 10 %) et les PV tombent à 120. Type de salle `arene` (`rooms/arene.ron`, horloge de 900
frames, 2 + 2 + 3 `bullet_kin`), deux gabarits `ArenaEW` et `ArenaCroix` dans `maps/salles.ldtk`
(`docs/taches/rapports/m2-t7-boss-arene.make_arene.py`), exigée une fois par la grammaire `etage`
(`required: {"arene": 1}`), validée sur 1 000 graines (`ALACOD_FLOOR_SEEDS=1000 cargo test -p map_ldtk
gungeon_etages_de_grammaire`). Aucun scénario ne joue encore l'arène de gungeon.
## 46. Surfaces v2 : fosses, tables renversables, barils explosifs (M2-T11, chantier E4 v2)

Code : `world::surface` (`PitDef`), `world::props` (`PropDef`, `TableState`, `BarrelState`,
`PitSafePosition`, règles pures `flip_direction` / `flipped_rect`), `game::props` (`PropsPlugin` :
`pit_system`, `table_flip_system`, `table_cover_system`, `barrel_explosion_system`),
`map_ldtk::game::entity::prop` (entités LDtk), `map::generation::entity::prop` (génération).
Script des cartes : `scripts/construire-cartes-t11.py`.

**Fosse.** Une surface (§26) peut porter `pit` ; la case IntGrid `Surfaces` correspondante est
une fosse :
```ron
// games/testbed/assets/surfaces/fosse.ron
(intgrid_value: 4, tags: ["fosse"], move_speed: "1.0",
 pit: (fall_damage: "10.0", dash_crosses: true))   // respawn: LastSafeCell (seul choix)
```
Un joueur (hors à terre) dont les pieds (§26) sont sur une fosse perd `fall_damage` PV (dégâts
`True`, tag `fall`), revient à sa dernière position sûre et sa vitesse est remise à zéro. Un dash en
cours traverse la fosse si `dash_crosses` ; il tombe s'il finit dessus. Position sûre =
`PitSafePosition` (composant rollback **neutre**, posé sur les joueurs seulement quand la carte a
au moins une case de fosse) : translation d'une frame où la case et ses 8 voisines n'étaient pas
des fosses, réécrite à chaque frame **avant** toute chute. Un joueur apparu dans une fosse n'a pas
de point de retour et ne tombe pas. Ennemis : les cases de fosse bloquent `Ground` et
`GroundBreaker` dans le champ de flux (`ObstacleType::Pit`), `Flying` et `Phasing` les passent (sans
pénalité) ; `EnemyNeverInWall` (§22) échoue aussi quand les pieds d'un ennemi au sol sont sur une
fosse. Sans la ligne de `update_flow_field_system`, `pit_enemy_path` échoue à f137.

**Objets de décor.** Kind `Prop` (`(path: "props", kind: "Prop")`), un fichier RON par objet, id =
nom de fichier, lu avec l'extension RON `UNWRAP_VARIANT_NEWTYPES` :
```ron
Table(bullets: 4)                                     // thickness: "6", range: "20" par défaut
Barrel(health: "20.0", radius: "48.0", damage: "30.0")
```
Entités LDtk `Table` et `Barrel` (aucun champ requis ; champ optionnel `prop` = id, défaut `table` /
`barrel`), lues par la génération (`PropConfig`, valeurs d'auteur portées telles quelles comme
`WeaponLocation`) puis créées par `spawn_map_items` avec `MapRollbackMarker` : `GgrsNetId` triés par
nom puis position comme les portes. Une entité dont l'id est inconnu ou d'un autre type reste
inerte (`warn!`). Lint : table à `bullets` 0, baril à `health`/`radius`/`damage` ≤ 0, fosse à
`fall_damage` ≤ 0 (`OutOfRange`), fosse sans `fall_damage` (`Parse`) ; fixtures `pit_*` et `prop_*`.

**Table.** Debout : marqueur `Window`, couche `window`, `Obstacle` `Barricade` non cassable,
`Interactable { Table }` (portée 20 du bord du collider). Les balles passent, les joueurs et les
zombies non. L'interaction (`InteractionType::Table`, variante ajoutée en fin d'enum) la renverse à
l'opposé du joueur (axe dominant, égalité : x) : elle perd `Window` et `Interactable`, gagne
`Wall` + couche `wall`, et son collider devient une bande de `thickness` au bord renversé sur toute
la longueur de ce bord. Chaque balle qui la touche (`combat::projectile::BulletWallHit`, file
**transitoire** hors checksum, émise par les collisions de balles ordinaires et composables) retire
1 à `bullets_left` ; à 0, `despawn_rollback()`. `TableState` : rollback neutre.

**Baril.** Personnage `Enemies` sans IA (Neutral ne subit jamais de dégâts) avec `Health`,
`BarrelState` (neutre) ; sans butin (power-ups, armes de progression ignorent `BarrelState`). À 0 PV
il garde son `Death` (santé figée à 0) : `rollback_apply_death` l'ignore. À la frame suivante
`barrel_explosion_system` (`CollisionDamage`, avant le résolveur) envoie `damage` (genre `Physical`,
tag `explosion`, tir ami `Always`) à tout ce qui a des PV dans `radius` (distance centre à centre,
hors barils déjà morts), crédité au dernier joueur qui l'a touché, puis `despawn_rollback()`. Les
barils touchés meurent dans la même frame (résolution avant `DeathManagement`) et explosent à la
suivante : chaîne déterministe (ordre `GgrsNetId`).

**Scénarios** (testbed, cartes `props_*.ldtk`) : `pit_fall` (100 → 90 PV, retour à x -150,6),
`pit_dash` (traversée, 100 PV), `pit_enemy_path` (le `follower` contourne), `table_flip_cover`
(cible à 5 coups avant, 5 pendant la couverture, 13 après la rupture), `barrel_chain` (f63 premier
baril à 0 PV, f64 second, f65 joueur 1 à 70). Nouvelle catégorie `EntityCount(kind: Barrel | Table)`.

**Gungeon.** `CombatEW` et `CombatNS` de `maps/salles.ldtk` portent deux barils et une table
(définitions d'entités ajoutées au projet LDtk) ; test `gungeon_decor_sur_1000_graines`
(`map_ldtk::generation`) : 1 000 graines de `etage`, 3 objets exactement par salle décorée, dans la
salle, sans recouvrir porte, spawn ni personnage. Seule `bench_floor` bouge (12 barils, 6 tables).

**Pièges.** (1) Une file `FrameEvents` « neutre » est hachée dès qu'elle n'est pas vide : un
événement émis dans des parties existantes (ici un mur touché) exige `add_frame_events_transient`.
(2) Un système qui pose un composant neutre sur les joueurs doit se garder sur une condition de
**carte** (case de fosse présente), pas de **jeu** (une surface fosse existe) : sinon les cartes à
eau du même jeu changent de trace. (3) `Team::Neutral` ne subit aucun dégât : un objet destructible
est `Enemies`. (4) Les bullets partent ~9 unités sous le centre du joueur : ne pas poser le joueur
en bord de couloir. (5) Deux worktrees sur un même `CARGO_TARGET_DIR` collisionnent (chemins
relatifs hachés pareil) : `find crates games -name '*.rs' -exec touch {} +` avant de reconstruire.
## 47. Bot explorateur d'étages (M2-T16, chantier K2 v1)

**Profil `explorateur`** (`BotProfile::Explorateur`, fin de l'enum, `bots::explorer`) : joue seul un étage
de salles typées du mode `Floors` (§35, §40). Aucune simulation touchée : il ne produit que des inputs, les
profils existants sont inchangés (aucune trace existante ne bouge).
- **Combat** : celui de `prudent` (chasse par le chemin, esquive, gestion d'arme, réanimation, ramassage au
  passage), contre les seuls ennemis **éveillés** : un ennemi `RoomDormant` n'est pas une cible mais une salle
  à visiter.
- **Blank** (§41) : appui quand une balle ennemie qui approche est à moins de `BLANK_DISTANCE` (48 px) et que
  le joueur a un blank ; front montant lu dans `BlankState::held` (état rollback).
- **Exploration**, sans ennemi éveillé : le but le plus proche **par le chemin** (champ à une source,
  `BotNavigation::costs_from`). Ordre déterministe : coût, objets avant salles, puis `GgrsNetId` de l'objet
  ou identifiant de salle (`LevelId`). Buts : objet au sol (pas d'actif si le joueur en tient déjà un :
  il échangerait sans fin), salle verrouillante à visiter (`Locked`, ou `Dormant` avec un ennemi dedans), la
  salle du boss en dernier. Portail ouvert : `prudent` va au portail.
- **Mémoire** : aucune. « Visitée » se lit dans l'état (`RoomState` `Cleared`, ou `Dormant` sans ennemi) ;
  les champs de navigation sont des caches dont la clé contient la géométrie.
- **Option** `ExplorerOptions::boss_direct` (`alacod-sim --boss-direct`) : droit au boss, sans autre salle
  ni objet.

**bot-explorateur-v2 (D65).**
- **Arène** : une salle `arene` dormante et vide est un but (`explorer::room_pending`) jusqu'à son
  nettoyage : elle ne se verrouille qu'à l'entrée d'un joueur et ses vagues naissent à ce moment.
  Même ordre que les autres salles (coût du chemin, puis identifiant), le boss reste en dernier.
- **Salles non verrouillantes** (départ, récompense, boutique) : un but jusqu'à ce qu'un joueur y
  entre (dans la salle réduite de 24 px) ; on n'y achète rien. **Seule mémoire du bot**,
  `ExplorerVisits` (ensemble `(index d'étage, salle)`, ressource hors rollback) : une salle sans
  état n'a rien à lire dans `RoomStates`. Elle ne sert qu'à produire des inputs, que GGRS enregistre ;
  un rollback ne rappelle pas le bot pour une frame passée.
- **Réanimation** : l'explorateur relève un coéquipier à terre sans attendre les 600 dernières frames
  d'urgence (`REVIVE_URGENT_FRAMES` reste la règle de `prudent`/`fonceur`). Il attendait l'urgence
  sans autre but : graine 175 à 2 bots, saignement fini à f2459, première interaction f1887.
- **Zone morte** : `surface_step` considère le bot arrivé quand son pas est sous `ROUTE_DEAD_ZONE`
  sur les deux axes ; sinon il restait immobile à 36 px de la portée, sans presser Interaction
  (graine 47 à 2 bots, `etages_deux`, soft-lock).
- Campagne : `docs/taches/rapports/bot-explorateur-v2/campagne.sh` (lots de 10 graines).

**`alacod-sim`** : `--floors <séquence> --until-floor N` existait (T1.14) ; le JSON gagne `outcome`
(`victoire`, `objectif`, `mort`, `soft-lock`, `plafond`, `desync`) et `rooms` (par étage : salles typées,
visitées, nettoyées, verrouillantes) ; un résumé par issue (graines) finit la sortie d'erreur. `--until-floor N`
s'arrête dès que `FloorState::index >= N` (index 0-based) : **atteindre l'étage 2 = `--until-floor 1`**. La
campagne : `alacod-sim --game gungeon --bots 1 --profiles explorateur --floors etages_court_deux --seeds A..B
--until-floor 1 --max-frames 20000`.

**Scénarios** : `bot_explorer_floor` (un bot, graine 1) et `bot_explorer_two` (deux bots) jouent
`etages_court_deux` sans input scripté ; ils vérifient l'état des salles, `FloorIndex` 1 et la survie. La
carte de l'étage suivant est chargée à l'avance : `RoomState` y désigne les salles de l'étage joué par
`nth: 1`. Les scénarios `gungeon_floor_seed_*` (atteinte scriptée) ne changent pas.

## 48. Butin, coffres et clés (M2-T5, chantier C4)

Où les objets (§36, §44) apparaissent : un tirage pondéré, seedé, **indépendant du moment**, des coffres
(dont certains à clé) et une récompense à la fin d'une salle de combat. Code : `items::loot` (contrats et
tirage pur), `game::loot` (systèmes), `map_ldtk::game::rooms::room_chest_system` (coffres des types de salle).

**Pool de butin** (kind `LootTable`, `loot/<id>.ron`, id = nom de fichier ; `game.ron` : `(path: "loot", kind: "LootTable")`) :

```ron
(
    entries: [(pick: Item("key"), weight: 3), (pick: Rarity(Rare), weight: 1)],
    exclude_held: false,                                   // défaut : faux
    reward: Some((chance: "0.33", chest_chance: "0.25")),  // récompense de salle (chest_rarity: Common)
)
```

- `pick: Item(id)` : un objet précis ; `pick: Rarity(r)` : **un** objet de cette rareté, uniforme parmi ceux de
  `items/` (le poids est celui de l'entrée, pas par objet : les rares restent rares quel que soit leur nombre).
- **Tirage** (`items::draw_loot`) : deux `next_u32` toujours consommés (l'entrée par poids, puis le candidat),
  entrées et candidats triés (BTreeMap), donc le flux avance de la même façon quels que soient les filtres.
- **Exclusion des objets tenus** : décision = **opt-in par pool** (`exclude_held`, faux par défaut). Elle rend
  le tirage dépendant de l'inventaire de l'ouvreur, donc du moment ; les pools de `gungeon` ne l'activent pas.
  Un actif tenu compte comme tenu. Si le filtre ne laisse rien, retombée sur le pool sans filtre de rareté ni
  d'exclusion (sauf récompense de salle : un consommable ou rien).
- **Rareté d'un coffre** (`Chest::rarity`) = rareté **minimale** de l'objet tiré (un consommable compte pour `Common`).

**Flux RNG dédié, indépendant du temps.** `items::loot_seed(graine de la partie, étage, clé de contenu)` =
FNV-1a de `loot:<graine>:<étage>:<clé>` replié sur 32 bits ; le tirage lit un **`RollbackRng` local** de cette
graine. Il ne touche **aucun** flux de `RngStreams` : le flux nommé `"loot"` des power-ups est consommé à chaque
mort d'ennemi, il ferait dépendre l'objet du moment d'ouverture (et changerait le hash de la ressource, comme
`variant.rs` le note déjà). La clé de contenu est celle du coffre (`room:<salle>`, `reward:<salle>`, ou la clé
d'un placement de scénario), jamais son `Entity` ni son `GgrsNetId`. Résultat : un coffre ouvert à la frame 30 ou
à la frame 3000, par l'un ou l'autre joueur, donne le même objet (scénario `loot_seeded` : deux coffres de même
clé ouverts à 120 frames d'écart donnent le même objet ; `loot_seeded_other` : une autre graine, un autre objet).

**Pool d'un étage.** `Floors` : champ optionnel `loot: ["commun", "etage2"]` (index = étage, le dernier vaut
pour les suivants) → `game::loot::FloorLootTables` (hors rollback, posé par `map_ldtk::game::floors`). Un coffre
sans `table` utilise celui de l'étage, sinon `commun` ; la récompense de salle utilise celui de l'étage **s'il
définit `reward`**, sinon `salle`. Lint : pool inconnu = `BrokenReference`.

**Coffre** (`items::Chest { key, table, rarity, locked, opened }`, rollback + checksum + trace en variante
**neutre**) : entité au sol avec `Interactable { Chest }` (`InteractionType::Chest`, variante en fin d'enum :
les hash existants ne bougent pas), `GgrsNetId` par clé de contenu (`chest_<clé>`). `chest_interaction_system`
(`Interaction`, après `handle_item_interaction`) : le premier joueur (ordre `GgrsNetId`) l'emporte. **Verrouillé**
(`locked`) : consomme **un** consommable `key` ; sans clé, refus (rien ne change, `ggrs{chest_refused}`). Ouvert :
un `ItemPickup` au sol à 16 unités, `opened = true`, `Interactable` retiré (le prompt et la détection n'y
reviennent pas), `FrameEvents<ChestOpened>` (neutre). Un coffre ouvert reste en place.
Prompt HUD (source `prompt`) : « Ouvrir le coffre », « Fermé — 1 clé » (verrouillé, sans clé), « Ouvrir — 1 clé ».

**Coffre d'un type de salle.** `rooms/<id>.ron` : champ optionnel `chest: Some((table: None, locked: false,
rarity: Common))` (`world::RoomKindDef`, qui perd `Copy`). `room_chest_system` pose le coffre au centre de chaque
salle de l'étage courant qui n'en a pas encore (clé `room:<salle>`) : aucune entité LDtk à ajouter, tout
gabarit d'un type donné en reçoit un. `gungeon` : `recompense` (verrouillé, rareté mini Uncommon, donc au moins un
coffre à clé par étage dans la grammaire `etage` qui exige une récompense) et `boutique` (ouvert, en attendant M2-T6).

**Récompense de salle** (`room_reward_system`, après `Run`) : une salle de type `combat` qui passe `Cleared`
(`FrameEvents<RoomChanged>`) laisse, avec la chance `reward.chance` du pool (forcée par
`Scenario::room_reward_chance_override`), au centre de la salle : un coffre (`chest_chance`, pool de l'étage ou
`commun`) ou un consommable tiré dans le pool (`consumables_only`). Graine : clé `reward:<salle>`. Les clés
tombent donc comme récompense (`gungeon` : `salle.ron`, clé de poids 3).

**Lint** : pool vide, poids ≤ 0, objet inconnu (`BrokenReference`), rareté sans objet, chances hors de 0..=1,
coffre de salle à pool inconnu, **coffre verrouillé dans un jeu sans consommable `key`** (`BrokenReference`),
pool inconnu dans `loot` d'une séquence. Fixtures `loot_empty_table`, `loot_weight_zero`, `loot_unknown_item`,
`loot_locked_chest_without_key`.

**Scénario** : `Scenario::chests` (`ChestPlacement` : clé, pool, rareté, verrou, position, frame), comme `items` ;
`room_reward_chance_override`. Attente `ItemCount { handle, id, count, at_frame }` (exemplaires d'un objet tenus :
0 prouve un refus, 2 un même objet tiré deux fois). Scénarios : `loot_seeded`, `loot_seeded_other`, `chest_locked`,
`room_reward`, `loot_floor_table` (testbed : pools `commun`/`etage2`/`salle`, séquence `deux_butin`).

**Hors périmètre** : le butin du boss (table `boss` de `gungeon` prévue, branchée par M2-T7), la boutique (T6),
le HUD des clés (T18). Les deux types de salle `recompense` et `boutique` de `gungeon` gagnent un coffre : les
scénarios qui jouent ces gabarits (étages `etage`/`grand`) changent de trace, preuve dans le rapport.

## 50. Boutique et monnaie `shells` (M2-T6, chantier C5)

Les ennemis lâchent des douilles (`shells`), dépensées sur les présentoirs d'une salle `boutique`.
Code : contrats dans `items::loot` (`Shelf`, `ShellPickup`, `RoomShopDef`), systèmes dans
`game::shop`, pose des présentoirs dans `map_ldtk::game::rooms::room_shop_system`.

**Monnaie au sol.** `economy.ron` : `shells_drop: Some((min, max))` (inclus, `None` par défaut :
aucun drop ; `min > max` refusé par le lint). `shell_drop_on_death_system`
(`DeathManagement`, même place que le drop de power-ups) tire le montant dans un `RollbackRng`
**local** de graine `items::loot_seed(graine, étage, "shells:<GgrsNetId du mort>")` : aucun flux de
`RngStreams` (le flux `"loot"` des power-ups garde sa consommation, donc ses traces). Équipe
`Enemies` seulement, jamais un baril. Le tas (`ShellPickup { amount }`, rollback **neutre**,
`GgrsNetId` `shells_<id du mort>`) se ramasse au contact (`shell_pickup_system`, `Effects`, portée
20, le plus petit `GgrsNetId` à portée) : `Currency::earn` + `CurrencyEvent` (raison `pickup`),
`despawn_rollback()`. Les points de kill (zombies) sont une autre source, inchangée.

**Boutique.** `rooms/<type>.ron` : `shop: Some((table: "boutique", shelves: 3))` (`world::RoomKindDef`).
`room_shop_system` pose, une fois par salle de ce type que l'étage présente, `shelves` présentoirs
alignés au centre (écart 32). Objets : `shop::draw_shelves`, tirage dans le pool `loot/<table>.ron`
avec un `RollbackRng` local de graine `loot_seed(graine, étage, "shop:<salle>")` — même mécanisme
que les coffres (§48), donc indépendant du moment et de qui entre ; un objet déjà tiré est redemandé
(8 essais au plus). Prix : `economy.ron` `shop_prices: {Common: 15, Uncommon: 30, Rare: 50}` par
rareté de l'objet (absent : `shop::DEFAULT_SHELF_PRICE` = 10). La salle n'est jamais verrouillée.

**Achat.** `items::Shelf { key, item_id, price, sold, retry_after }` (rollback **neutre**,
`GgrsNetId` `shelf_<clé>`, `Interactable { InteractionType::Shop }` — variante en fin d'enum),
`shelf_interaction_system` (`Interaction`, après le coffre) : `Currency::spend(prix)` puis l'objet est
donné comme un ramassage (passif : liste + modificateurs ; actif : remplace l'actif tenu, qui tombe
au sol ; consommable : stock ou effet immédiat) et `ItemPicked` est émis. `sold = true`,
`Interactable` retiré. Solde insuffisant : rien ne change, `CurrencyEvent { delta: 0, reason:
"shop_refused" }` et anti-rebond `retry_after = frame + 30`. Achat : `delta = -prix`, raison `shop`.
Prompt HUD (source `prompt`) : « Acheter Trèfle — 30 » ; rien pour un présentoir vendu.

**Lint.** Prix de `shop_prices` à 0, `shells_drop` min > max (`OutOfRange`) ; `shop` sans présentoir
(`shelves: 0`), pool de boutique vide (`OutOfRange`), pool inconnu (`BrokenReference`). Fixtures
`shop_price_zero`, `shop_shells_inverted`, `shop_no_shelves`, `shop_empty_table`, `shop_without_table`.

**Scénarios.** `Scenario::shop` (`ShopSetup`) : `shelves: [(key, item, price, x, y, at_frame)]` pose
des présentoirs, `shells_drop_override: Some((min, max))` force le drop (le testbed n'a pas de
`shells_drop` : ses traces ne bougent pas). Attentes : `Shelf(nth, item, price, sold, at_frame)`
(présentoirs triés par clé), `ShelfCount(count, at_frame)`, `ShellPiles(count, at_frame)`.
Scénarios : `shells_drop`, `shop_buy`, `shop_refused` (testbed), `shop_seeded` (gungeon, grammaire
`etage`).

**Hors périmètre** : marchand PNJ et déblocages méta (M2-T12, T13), vol, réduction de prix, HUD des
shells (M2-T18).

## 52. Contenu `gungeon` (M2-T14, quatre parties)

### 52.a Armes (T14a)

Données seulement (`games/gungeon/assets/weapons/weapons.ron`), aucune mécanique ajoutée au moteur.
Vingt armes à distance au total (compte du lint : `pistolet`, 18 nouvelles, `arsenal` des ennemis),
plus `bare_hands` (mêlée). Chacune compose des comportements existants et porte un `test:` (scénario
généré `weapon_<arme>`, `make gen GAME=gungeon`) avec bornes `min_hits`/`max_hits` mesurées.

| Arme | Munition | Caractère | Composition |
|---|---|---|---|
| `pistolet` | balles | arme de départ, cadence 3/s | `Automatic`, `Standard` |
| `revolver` | balles | coup lent et lourd | `Manual`, 30 dégâts, chargeur 6 |
| `pistolet_precis` | balles | balle perçante | `Piercing(1)` + `Pierce(1)` |
| `pistolet_jumele` | balles | deux balles par pression | `Burst(2)` |
| `carabine_rafale` | balles | rafale de trois | `Burst(3)` |
| `mitraillette` / `mitrailleuse_lourde` | balles | tir continu 10 et 14/s, dispersion | `Automatic`, `spread` |
| `rebondisseur` | balles | rebondit sur les murs | `Bounce(3)` |
| `fusil_pompe` / `double_canon` | cartouches | cône de 8 / 10 plombs | `Shotgun` |
| `fusil_precision` | calibre | traverse trois ennemis, très lent | `Piercing(3)` + `Pierce(3)` |
| `arc` / `arbalete` | fleches | flèche lente perçante / carreau lourd | `Piercing`, `Pierce`, `Size` |
| `laser` | energie | faisceau rapide qui traverse tout | `Automatic`, `Pierce(8)`, `Lifetime(30)` |
| `chercheur` | energie | tête chercheuse | `Homing("0.1")` |
| `boule_energie` | energie | gros projectile lent | `Size("3.0")`, `Pierce(8)` |
| `lance_roquettes` | roquettes | explose à l'impact | `on_expire: Spawn(Aimed → souffle)` |
| `grenadier` | roquettes | rebondit puis explose en éclats | `Bounce(2)`, explosion + `Ring` (§16) |
| `lance_flammes` | carburant | jet continu à portée courte | `Automatic`, `Lifetime(40)`, `Pierce(2)` |

Munitions : réserves distinctes par type `Custom` (`balles`, `cartouches`, `calibre`, `fleches`,
`energie`, `roquettes`, `carburant`). Pas de munition infinie dans le moteur (D à venir) : seul
`pistolet` a une réserve de 9999 chargeurs ; les autres ont une réserve finie.
`Burst` ne tire pas plusieurs balles par frame mais `pellets_per_shot` balles espacées de
`cooldown_frames` (voir `FiringMode::Burst`). Les traces `gungeon` existantes n'ont pas bougé
(les 5 scénarios générés d'avant sont identiques après `GEN_BLESS=1`).

### 52.b Objets et synergies (T14b)

Données seulement (`games/gungeon/assets/{objets,synergies}/`), aucune mécanique ajoutée. Vingt objets
au total (dix de plus), huit synergies (quatre de plus : la fiche dit « quatre synergies », lue comme
quatre en plus des deux de M2-T4). Les pools de butin (`commun`, `boutique`, `boss`) tirent par rareté :
les nouveaux objets y entrent sans toucher aux pools ; le prix en boutique suit la rareté (§50).

| Objet | Rareté | Genre | Effet |
|---|---|---|---|
| `gilet` | Commun | passif | `OnPickup` (une charge) : +25 d'armure (D61) |
| `egide` | Commun | consommable | `GainShield(1)` au ramassage (D61) |
| `lunettes` | Commun | passif | `Range` +20 % |
| `chargeur_rapide` | Commun | passif | `ReloadSpeed` +30 % |
| `roulade` | Peu commun | passif | `OnDodge`, cooldown 60 : cadence +30 % pendant 120 frames |
| `trophee` | Peu commun | passif | `OnKill`, cooldown 120 : soin 2 |
| `parchemin` | Peu commun | passif | `OnBlank`, cooldown 300 : `GainShield(1)` |
| `sablier` | Peu commun | actif `Frames(1800)` | vitesse +50 % pendant 240 frames |
| `bastion` | Rare | passif | vie max +10 ; `OnFloorStart` : `GainShield(1)` |
| `rancune` | Rare | passif | `OnDamageTaken` sous 50 % de vie, cooldown 600 : dégâts +50 % pendant 180 frames |

Synergies : `rempart` (`gilet` + `bastion` : +25 d'armure), `fuite_armee` (`bottes` + `roulade` : vitesse
+10 %), `sang_froid` (`trophee` + `rancune` : dégâts +20 %), `tireur_d_elite` (`lunettes` + `chargeur_rapide` :
dégâts +15 %).

**Piège (armure).** `Armor` n'a pas de valeur de base chez le joueur : le générateur (`expected_item_stats`)
refuse (`MissingStat`) un passif dont `modifiers` touche `Armor`. `gilet` pose donc son armure par un effet
`OnPickup` (source `item:gilet:0`) ; `rempart` (synergie, sans `test:`) peut l'utiliser en `modifiers`.
Les consommables (`egide`) n'ont pas de `test:` (comme `ration`, `blank`).

### 52.c Ennemis et boss (T14c)

Données seulement (`games/gungeon/assets/`), aucune mécanique de moteur.

**Ennemis** (`characters/`, un `test:` chacun, scénarios générés `enemy_<id>_still`/`_moving`) :
`shotgun_kin` (`Shoot` de `pompe` : télégraphe 25 puis sept chevrotines en `Fan`, portée 170 ; variante
`blinde`), `veteran_kin` (`visee` + `KeepDistance` 140-220 ; variantes `rapide` et `blinde`),
`bandana_kin` (`Charge`, télégraphe 30, 15 de dégât), `tourelle_kin` (immobile, `croix` : deux couronnes de
quatre boules), `fuyard_kin` (`Flee` sous 50 %, `rafale` de quatre balles ; variante `rapide`),
`lanceur_kin` (`KeepDistance` 150-240, `grenade_lente`). Avec `bullet_kin`, `bullet_kin_squad` et le boss
`gatling` : dix ennemis. Patterns nommés `pompe`, `rafale`, `croix`, `grenade_lente` ; arme ennemie
`arsenal_lourd` (table `chevrotine`, `grenade` → `explosion` → `eclat`), à part de `arsenal` pour que le hash
des ennemis existants ne bouge pas.

**Placement** (aucun gabarit ajouté, `docs/taches/rapports/m2-t14c-ennemis.make_placement.py`) :
`CombatEW` (+`shotgun_kin`, `fuyard_kin`), `CombatNS` (+`veteran_kin`, `tourelle_kin`), `CombatCroix`
(+`bandana_kin`, `lanceur_kin`), vagues de l'arène (`fuyard_kin` à 300, `bandana_kin`, `lanceur_kin`,
`tourelle_kin` à 600). `CombatT` et `EscouadeEW` ne bougent pas (les scénarios scriptés de l'étage `court` s'y
appuient). `alacod lint gungeon` : aucun nouvel avertissement d'ennemi.

**Boss `gatling`** : phase 1 = couronne puis spirale 60 frames plus tard (répétition 150) ; phases 0 et 2
inchangées. Un renfort `lanceur_kin` en phase 2 a été essayé : le bot meurt et `gungeon_boss_bot` dépasse son
`EntityCount`, retiré.

## 53. Lint des kinds M2 et atteignabilité statique (M2-T15, voie V2)

Lint seulement : aucune simulation touchée, aucune trace ne bouge.

**Avertissements.** `content::warnings(&registry, &manifest) -> Vec<LintWarning>` (`crates/content/src/reach.rs`,
kinds `Unreachable` et `UnreadStat`). `alacod lint` les liste sur stderr (« avertissement : … ») et le
code de sortie ne change pas ; `LintError`/`load_and_lint` sont inchangés.

**Stat sans lecteur.** `sim_core::stats::stat_reader(&StatId) -> Option<&'static str>` : liste fermée
des stats lues par la simulation, avec l'endroit de lecture ; à tenir à jour **à la main** quand un
système lit une nouvelle stat. Un modificateur (ou une base `stats:`) sur une stat sans lecteur est une
erreur `LintErrorKind::UnreadStat` (`Luck`, tout `Custom` sauf `powerup_currency_multiplier`).
`Armor` est lue depuis D61 (§18, `character::health`) : plus d'exception.
Usages inspectés : `stats` et `effects` des personnages, variantes, perks, objets, synergies, mutations,
power-ups.

**Atteignabilité.** Racines : `entry.start_map` et chaque niveau de chaque séquence `Floors` (carte,
`cave:<id>`, `floor:<id>`). Il n'y a pas de séquence « par défaut » stable (le mode `Floors` prend la
première par ordre d'id, et `etage_1` de gungeon est une carte fixe) : toute séquence déclarée compte.
Types tirables d'une grammaire = `FloorGrammar::kinds_needed()`. Avertissements :
- type de salle (`rooms/<id>.ron`) qu'aucune grammaire ni carte posée ne peut tirer ;
- gabarit d'une carte `templates` dont le type n'est tiré par aucune grammaire atteinte ;
- ennemi (`ai` présent) absent des niveaux jouables (cartes posées, gabarits tirables), des arènes
  des salles tirables, des vagues, des cavernes et des renforts de boss (point fixe) ;
- objet sans source : `starting_consumables`, `consumables_per_floor`, **pool**. Le point de branchement
  de M2-T5 est `reach::loot_pool_items` (et `loot_pool_weapons`), vides aujourd'hui : y renvoyer les
  ids de chaque pool (une dizaine de lignes) ;
- arme sans source : `starting_weapons`, `WeaponLocation` des niveaux jouables, `weapon_pool` de
  progression, pools.
`MapEntry::levels` (`MapLevelInfo` : type de salle, personnages, armes murales par niveau LDtk) porte
les données. Limites : pas de lecture des scénarios (une arme ou un objet donné par scénario seulement
est signalé), ni des cartes assemblées autrement que par les désignations ci-dessus ; la sandbox
`testbed` (arènes de test) produit donc beaucoup d'avertissements voulus.

**Contenu de test (D63).** Champ optionnel `test_only: bool` (défaut `false`) : au sommet d'un fichier
de personnage (`test_only: true,`), dans `config:` d'une arme (`config: (test_only: true, …)`). L'analyse
l'ignore : pas d'avertissement « ennemi » ou « arme » pour lui, et un personnage marqué n'est pas une
source d'armes. Lu par `content` seulement (`CharacterEntry::test_only`, `WeaponEntry::test_only`) : les
types de `game` et `combat` l'ignorent comme tout champ inconnu, donc aucun effet en jeu et aucune trace
qui bouge. Marqués : `cible` et `mannequin` (throne), `cible` (gungeon). Fixtures
`reach_test_only_character` et `reach_test_only_weapon` (jumelles de `reach_enemy_unplaced` et
`reach_weapon_no_source`, test `test_only_content_is_ignored_by_reach`).

**Audit des kinds M2** (références vérifiées / bornes vérifiées / fixtures) :

| Kind | Références | Bornes | Fixtures |
|---|---|---|---|
| `rooms` | `room_kind` des cartes (`lint_room_kinds`) ; ennemis d'arène | arène : clock>0, vagues datées, ordre, count>0 | `room_kind_unknown`, `arena_*` |
| `floor_grammars` | carte `templates`, gabarit par type nécessaire | `problems()` (rooms, requis, distance) | `floor_required_without_template`, `floor_bounds_inverted` |
| `floors` | carte, caverne, grammaire, `consumables_per_floor` | levels non vide, cavernes sans ennemi | `floors_*` |
| `items` | `cost`, stats (Parse) | charge, pickup_range, blank, uses_per_floor, cost>0 ; **stat lue (T15)** | `item_*`, `stat_unread` |
| `synergies` | objets requis | ≥2 objets distincts, doublon d'ensemble | `synergy_*` |
| `props` | — | rayons, dégâts > 0 | `prop_barrel_radius_non_positive`… |
| `surfaces` | — | doublon de valeur, facteurs > 0, fosse | `surface_*`, `pit_*` |
| personnages boss | patterns, ennemis de renfort, pas de répétition sans fin | santé, phases, `until`, count>0 | `boss_*` |
| profil | `entry.progression`, horloges | — | `entry_*` |
| stats (T15) | — | liste fermée `stat_reader` | `stat_unread`, `stat_armor_warning` |
| atteignabilité (T15) | racines → salles/gabarits/ennemis/objets/armes | — | `reach_*` |

Trous comblés : la lecture des stats (nouvelle règle) et l'atteignabilité (nouveaux avertissements).
Aucun autre trou trouvé dans l'audit (les `props` et le profil n'ont pas de référence à vérifier).

**Fixtures.** `stat_unread` (erreur), `stat_armor_warning` (avertissement), `reach_room_unreachable`
(type de salle et gabarit), `reach_enemy_unplaced`, `reach_item_no_source`, `reach_weapon_no_source`.

## 54. Générateur v2 : gabarits par objet, par salle et par boss (M2-T17, voie V3)

Code : `crates/scenario/src/generate.rs` (gabarits), `game::gen_test` (`ItemTest`, `RoomTest`,
`BossFightTest`), `scenario::runner::apply_player_overrides` (objets de départ), `content::lint`
(`lint_item_tests`, `lint_room_tests`, `lint_character_tests`). Complète §28 : mêmes règles
(opt-in par un champ `test:`, aucun effet en jeu, un scénario écrit par `make gen GAME=<jeu>` dans
`tests/scenarios/generated/<jeu>/`, champ absent = rien de généré : aucune trace existante ne bouge).
Contrairement aux gabarits d'armes, ces gabarits jouent **dans le jeu énuméré** (le registre du
testbed n'a ni les objets ni les salles de `gungeon`) : carte de `generate_template` si le jeu en
déclare une (`Sandbox` imposé), sinon l'arène du testbed pour le testbed.

**Objet** (`objets/<id>.ron`, gabarit `ItemHeld`, fichier `item_<id>.ron`) :
`test: Some((frames: 60, expect: [...]))` ; `frames > 16` pour un actif.
- `PlayerScript::items: ["<id>"]` (nouveau, vide par défaut) : le joueur tient l'objet **avant la
  première frame**, comme après un ramassage (passif : `Inventory` et modificateurs `item:<id>` ;
  actif : emplacement actif **chargé** à `ActiveCharge::required()` ; consommable : compteur). Les
  effets `OnPickup` ne sont pas rejoués.
- Attentes par défaut : `HasItem` à la frame 1 ; une `Stat` par stat modifiée (frame 1), attendue =
  `sim_core::modifier::resolve(base, modificateurs)` où `base` est lue par une **sonde** d'une frame
  sans l'objet (`probe_player_stats`) : la valeur ne vient pas de la partie qui tient l'objet ; un
  actif : `ItemCharge(required)` à la frame 5, `UseActive` une frame à `ITEM_USE_FRAME` (10), puis
  `ItemCharge(0)` à la frame 16 (omis pour `Frames(n)`, qui se recharge aussitôt, et pour un actif à
  `cost`, dont l'usage peut être refusé). `test.expect` s'**ajoute** aux attentes par défaut.

**Salle** (`rooms/<id>.ron`, gabarit `RoomClear`, `room_<id>.ron`) :
`test: Some((frames: 500, floors: "etages_court", seed: 1, expect: []))` ; exactement un de `floors`
(séquence `Floors` du jeu) ou `map` (carte LDtk, `Sandbox` imposé si le manifeste est en `Floors`) ;
`seed` = graine de carte (défaut 123456) : le gabarit LDtk tiré par la grammaire est donc fixe.
Un bot `explorateur` (§47), aucun input scripté. Attentes par défaut : moments clés `room` « salle
<id> verrouillée » puis « salle <id> nettoyée » avant `frames` (`Event`, cumulatif : une attente
`RoomState` ponctuelle dépendrait de la frame exacte du bot), et `PlayerAlive` à la dernière frame.
Une salle qui ne verrouille ni n'est une arène (`locks: false`, pas d'`arena`) : seulement
`PlayerAlive`.

**Boss** (personnage qui a `boss:`, gabarit `BossFight`, `boss_<id>.ron`) : le `test:` du personnage
gagne le champ `boss: Some((floors: None, map: None, seed: 123456, expect: []))` (`boss: Some(())`
suffit), `frames` du `test:` fixe la durée, `still: false, moving: false` coupent les gabarits
ennemis. Sans `floors` ni `map`, la partie du manifeste (`entry.start_map`, comme `gungeon_boss_bot`).
Bot `fonceur`, aucun input. Attentes par défaut : un `Event(boss_phase, "phase i-1 → i")` par phase
atteinte (cumulatif : `BossPhase` échoue une fois le boss mort), puis `Event(room, "salle boss
nettoyée")` si le jeu a un type de salle `boss` (§45).

**Lint.** `frames > 0` ; objet : pas sur un consommable (`Unsupported`) ; salle : exactement un de
`floors`/`map`, scène connue, **au moins un gabarit LDtk du type** dans les cartes de la scène
(`BrokenReference`, y compris les gabarits d'une grammaire `floor:<id>`) ; personnage : `test.boss`
seulement avec `boss:`, `floors`/`map` exclusifs et connus, un gabarit actif parmi `still`, `moving`
et `boss`. Fixtures `item_test_frames_zero`, `item_test_consumable`, `room_test_no_template`,
`room_test_both_scenes`, `room_test_frames_zero`, `boss_fight_without_boss`.

## 55. Feedback v2 : télégraphes de patterns, flash du blank, fin de boss (M2-T19, voie V4)

Complète §31. **Tout est présentation** : rien de rollback, rien dans le checksum ni dans les
traces, aucune simulation ralentie (`Time<Virtual>` n'est jamais touché : ggrs avance dessus). Les
trois effets s'écrivent d'abord dans `FeedbackLog` (headless compris), le rendu (`FeedbackPlugin`)
applique ensuite les indices ; `scenario::events` en fait des moments clés `feedback`.

**Réglages** (`ui/feedback.ron`, champs optionnels de `content::feedback::FeedbackSettings`,
défauts sûrs : les trois effets sont actifs sans rien écrire, `enabled: false` en coupe un) :
- `pattern_telegraph: (enabled, color: Option<rgba>)` ; couleur absente : celle du cercle.
- `blank: (enabled, ring_frames, color, flash_alpha)` : 24 frames, bleu clair, flash 0,15.
- `boss_end: (enabled, freeze_frames, zoom, zoom_frames, flash_alpha, flash_color)` : 30 ticks
  de rendu gelés, zoom ×1,5 relâché sur 60 frames, flash 0,6.
Lint (`FeedbackSettings::problems`, `OutOfRange`) : durées dans `[1, 600]` (`freeze_frames`
`[0, 600]`, > 0 si l'effet est actif), opacités et couleurs dans `[0, 1]`, `zoom` dans `[1, 4]`.
Fixtures `feedback_blank_frames_zero`, `feedback_boss_zoom_out_of_range`,
`feedback_boss_freeze_zero`.

**Télégraphe de forme.** Pendant le `Telegraph` d'un émetteur (et le `Telegraph(n)` d'une frise de
boss, §45 : même chemin, l'émetteur joue `Sequence([Telegraph(n), pattern])`), `pattern_shape`
(fonction pure) lit les étapes **après** le télégraphe, jusqu'au prochain `Telegraph`/`Wait` :
`Fan`/`Spread`/`Aimed` → rayons répartis sur l'ouverture autour de la visée figée ; `Spiral` →
rayons de la première salve (l'amorce) ; `Burst`/`Scatter` → cône (deux rayons de bord et arc).
`Ring` n'a pas de forme (le cercle v1 le dit). Gizmos, longueur = rayon du télégraphe v1 (portée du
tir annoncé, bornée par `Shoot.range`). Un indice `PatternTelegraph` par (émetteur, étape), au
début du télégraphe. Moment clé : « télégraphe fan 31 (5 rayons) », « télégraphe burst 31 (cône
0.50 rad) » (`31` : `GgrsNetId` de l'émetteur).

**Blank.** Dérivé de l'état : les charges de blank (`Inventory::consumable` de l'objet blank de la
table) d'un joueur baissent d'un tick à l'autre. Anneau (`blank_ring_radius`, ease-out jusqu'au
rayon de `BlankSpec`) et flash plein écran léger ; il **remplace** l'onde de `LastBlank` (M2-T1)
quand `blank.enabled`. Moment clé : « blank 0 (rayon 160) » (`0` : handle du joueur).

**Fin de boss.** Dérivé de l'état, jamais de `BossDefeated` (transitoire) : un boss (`BossPlan`)
vu vivant au tick précédent a disparu **et** le nombre de salles `Cleared` (`RoomStates`) a
augmenté. Gel de rendu de `freeze_frames` ticks (`AnimationFreeze`, comme le hit stop), zoom sur la
position du boss (`boss_zoom_factor` : monte sur le premier cinquième puis revient à ×1 ; échelle
de projection et décalage de la caméra retirés avant le suivi de caméra, `undo_boss_zoom`),
flash plein écran (`ScreenFlashNode`). Moment clé : « fin de boss 40 » (`40` : `GgrsNetId` du boss).

**Preuve sans écran.** Attentes `Event(kind: "feedback", label_contains: …)`, aucune trace ne
bouge. À voir sur un poste avec GPU : voir le rapport `docs/taches/rapports/m2-t19-feedback-v2.md`.

**Limites.** Comme §31 : un tick qui simule plusieurs frames ne journalise que la dernière ; un
indice prédit puis annulé par un rollback p2p reste dans le journal. Une fin de boss se détecte au
tick où le boss disparaît : un boss retiré sans qu'une salle passe `Cleared` ne déclenche rien.

## 56. Écrans en bevy_hui et HUD gungeon (M2-T18, voie V4)

**Décision de William, 2026-10-15 : `bevy_hui` pour les écrans.** Le HUD reste la couche
maison `ui/hud.ron` (§15). Le comparatif historique et ses mesures restent dans
`docs/taches/rapports/m2-t18-ui-comparatif.md` ; la livraison de la seconde partie est décrite
dans `docs/taches/rapports/m2-t18-hud-gungeon.md`.

### Écrans

`game/ui_hui` est activée par `render`, défaut des quatre jeux natifs et wasm et du rendu
scénario. Les exécutables headless ne chargent pas Hui. Il n'y a plus de sélecteur de backend,
de renderer `screens/custom.rs` ni de fichiers pause/inventaire RON.

Le modèle pur (`screens/model.rs`) conserve `ScreensState`, `UiAction`, `reduce`,
`pause_view`, `inventory_view` et `ScreenView`. Les sections nommées sont liées génériquement :
`Text` → `{nom}`, `Bar` → `{nom_label}` et `{nom_width}`, `List` → conteneur `id="nom"`.
Chaque ligne utilise `ui/hui_row.html`, avec marqueur, couleur et fond du focus. Un changement
de vue remplit à nouveau les listes ; un rechargement HTML recrée leurs lignes.
`sync_screen` tourne **avant `HuiSystems::Build`** : aucune commande de Hui ne doit
construire une ligne déjà détruite lorsque l'inventaire change.

- Pause : Reprendre, Recommencer (`RunRequest::Restart`), Lobby (`RunRequest::ToLobby`),
  Réglages (« Aucun réglage pour l'instant. »).
- Inventaire : armes et munitions, actif et charge, passifs, consommables. Joueur affiché :
  celui de `HudPlayer` (caméra forcée, sinon local de plus petit handle).
- Navigation : Échap / Start, I / Select ; ↑ ↓ / D-pad ; Entrée ou Espace / A ;
  Retour arrière / B. `ALACOD_INPUT=neutral` coupe la lecture des touches.
- **La simulation continue et ses touches restent actives** sous les menus, comme avant :
  aucun verrou d'input ni changement de trace n'est introduit.
- Écran de fin : `ui/game_over_screen.html`, dans les quatre jeux. Résumé dérivé de
  `Run.step`/`Run.summary`, mis à jour sans reconstruire la racine ; rollback de l'issue :
  disparition de l'écran. Rejouer (R) et Lobby gardent leurs `RunRequest`, boutons souris
  inclus. La présentation est chargée seulement dans `PresentationPlugin`.
- **Écran de mutation (D71, 2026-10-15)** : `ui/mutation_screen.html` dans throne et testbed,
  adaptateur `game::ui::mutation_hui` (`MutationHuiPlugin`, dans `PresentationPlugin`, feature
  `ui_hui`). `MutationScreenView` et le protocole sont inchangés : valider émet toujours le bit
  `ChoiceA/B/C` dans l'input (`confirm_input`), l'adaptateur ne lit que la vue. Propriétés du
  gabarit : `name{i}`, `desc{i}`, `tint{i}`, `border{i}` (carte surlignée en or) pour trois cartes,
  `bar_width` (barre de temps, en %). Écran dérivé de l'état : un rollback qui annule le choix le
  fait disparaître. `mutation_screen.ron`, `MutationScreenLayout` et son lint sont retirés (aucune
  trace ne bouge). Tests : `mutation_hui::gabarit_des_jeux_construit_et_recompile` construit les
  vrais gabarits de throne et testbed.

Les fichiers HTML sont découverts directement dans les dossiers `Ui`, sans compagnon RON.
Le registre vérifie leur lecture UTF-8 ; le lint exige aussi `ui/hui_row.html` lisible
si pause/inventaire le référencent. Fixtures `html_screen_missing_row` et
`html_screens_valid`. Les inventaires de texte embarqués pour wasm incluent `.html`.
Ce lint ne valide pas la grammaire complète de Hui.

### HUD gungeon

Nouvelles sources calculées par `hud_values` (pure), présentes dans `HudSnapshot` aussi
en headless et dans la liste fermée `content::ui::HUD_SOURCES` :

| Source | Valeur |
|---|---|
| `blanks` | compteur `Inventory::consumables["blank"]` |
| `keys` | compteur `Inventory::consumables["key"]` |
| `shells` | monnaie `Currency` (les douilles de boutique la créditent) |
| `passives` | noms des passifs ; `Icons` utilise leurs ids |
| `active_item` | nom de l'objet actif |
| `active_charge` | charge / seuil, suffixe « · prêt » ; barre bornée entre 0 et 1 |
| `minimap` | type, état, marqueur boss et salle actuelle ; rendu spatial par `Map` |

Sans joueur/objet : textes vides et charge masquée. `HudRoomGeometry` reçoit uniquement
la géométrie de l'étage courant depuis `CurrentRooms`, en `Update` avant `HudModelSet`.
La carte se dérive de cette géométrie, de la position du joueur et de `RoomStates` :
aucun nouveau composant/état rollback.
**Mémoire de visite (D72, 2026-10-15)** : `RoomStates` ne retient pas une salle non combattante
restée `Dormant` (boutique, récompense). `hud_model::MinimapVisits` (ressource de présentation,
`Update`, hors rollback et hors trace) garde les ids des salles où un joueur **local** est entré
(`track_minimap_visits`, avant `update_hud_snapshot`) ; elle repart de zéro quand la géométrie de
l'étage change (nouvel étage, restart). Une salle visitée puis quittée reste « visitée ». Limite
assumée : un rollback ne défait pas une visite (l'exploration n'est pas un fait de simulation) ;
aucun état `Visited` rollback n'a été nécessaire. Preuve : scénario `minimap_shop_visit`
(gungeon, `etages_grammaire`, graine 12 : la boutique est voisine du départ ; attentes `HudText`
« boutique · inexplorée » f10, « visitée · actuelle » f90, « visitée » après la sortie f199, alors
que `RoomState` reste `Dormant`).

### Scénarios et revue visuelle

Le script `ui: [(at_frame: 10, action: Toggle(Pause)), …]` joue en `Update`, hors input
rollback. `ScreenText(screen, section, contains, closed, at_frame)` vérifie les menus ;
`HudText(source, contains, at_frame)` vérifie les sources du HUD. Les scénarios existants
`ui_inventory`, `gungeon_start`, `gungeon_clear`, `gungeon_boss` et `gungeon_two_floors`
portent les attentes HUD, sans modifier leurs traces.

```bash
cargo run -p scenario --profile headless --features render --bin play_scenario -- tests/scenarios/ui_pause_menu.ron --capture /tmp/ui/pause --every 1
cargo run -p scenario --profile headless --features render --bin play_scenario -- tests/scenarios/ui_inventory.ron --capture /tmp/ui/inventaire --every 1
cargo run -p scenario --profile headless --features render --bin play_scenario -- tests/scenarios/gungeon_boss_bot.ron --capture /tmp/ui/fin --every 10
```

Frames à regarder (`frame_NNNNN.png`) : pause 10, 20–22, 31, 40, 51, 62, 64 ;
inventaire/HUD 9 (actif prêt), 12, 20 (déchargé), 21–24 (focus), 31 (fermé).
Carte : `gungeon_clear` 100 (verrouillage), 650 (nettoyée) ;
`gungeon_boss` 320–450 (boss), 480–520 (blanks consommés) ;
`gungeon_two_floors` 800 (nouvel étage). Fin : `gungeon_boss_bot` 450.
Rechargement en jeu : `cargo run -p gungeon --features native`, ouvrir un écran puis
modifier son `.html`. Le rapport précise les captures effectivement observées.


## 60. Contrats des larmes et de l'empilement de projectiles (M4-T0a, chantier B5 v3)

Contrats posés pour M4-T1 (larmes v3), M4-T6 (familiers qui tirent), M4-T11a (contenu de larmes)
et M4-T15 (bench). **Aucune trace existante ne bouge** : variantes en fin d'enum, stats lues avec
une base 1 sans être posées dans `Stats`, champs d'objet optionnels hors rollback.

### Munitions infinies

`MagBulletConfig::Infinite` (en fin d'enum, `combat::weapons`) : le mode tire tant que la cadence
le permet ; jamais vide, jamais rechargé (pas même le délai de pompe d'un `Shotgun`), `mag_ammo` et
`mag_size` restent à 0 (attente `Ammo` à 0 ; le HUD affichera `∞` ou rien, M4-T16). Réserve
initiale : 0, comme `Magless`.

```ron
"larme": (config: (name: "larme", default_firing_mode: "default", firing_modes: {
    "default": (firing_rate: "3.0", firing_mode: Automatic(), /* … */ mag: Infinite),
})),
```

`ammo_type` devient facultatif dans le RON (`#[serde(default)]`, défaut `Custom("")`, la seule
valeur qu'un RON ne peut pas écrire : T2.8 la refuse) **seulement** pour une arme dont tous les
modes sont `Infinite`. Lint : `ammo_type` absent alors qu'un mode consomme (`Mag`, `Magless`) →
`Parse` (comme l'erreur RON d'avant) ; `ammo_type` donné alors que tous les modes sont `Infinite` →
`OutOfRange`. Une arme mixte (un mode `Mag`, un mode `Infinite`) garde son `ammo_type`.

### Stats de larmes et `Luck`

- `StatId::ShotSpeed` (vitesse de la balle) et `StatId::ShotSize` (rayon de collision et sprite,
  après `Size`), **après `Custom`** dans l'enum : l'`Ord` des clés de `Stats` et le hash des stats
  existantes ne changent pas. Lues au tir (`weapon_rollback_system`, `ShotStats`) par
  `StatReader::get(…, 1)` : base 1 sans être posées dans `Stats` (le `Stats` de tout personnage
  reste identique) ; à 1, aucun produit n'est calculé. `sim_core::stats::default_base` donne cette
  base aux lecteurs hors simulation (attente `Stat`, générateur `ItemHeld`).
- `Luck` a un lecteur de contrat : `effects::chance_with_luck(base, luck, par_point)` =
  `clamp(base + luck × par_point, 0, 1)` en `Fixed`, saturant. Rien ne l'appelle encore dans la
  simulation (M4-T1 : statut posé par une larme ; M4-T2 : effets d'objet). Le lint accepte donc
  désormais un modificateur sur `Luck` (`stat_reader`).

### Modificateurs de projectile portés par un objet

Les contrats `ProjectileModifier`, `Pattern`, `ExpireAction` et `ProjectileSpec` vivent désormais
dans `effects::projectile` (crate de simulation) et sont réexportés par `combat::projectile` :
`items` (simulation, sans rendu) peut les porter sans dépendre de `combat` (dette D69). Le `Hash`,
le `Debug` et le RON ne dépendent pas du crate de définition : aucune trace ne bouge.

```ron
// objets/<id>.ron (ou synergies/<id>.ron)
(name: "Ricochet", kind: Passive, projectile: Some((modifiers: [Bounce(2)], on_hit: [], on_expire: [])))
```

`ItemDef.projectile` et `SynergyDef.projectile` : `Option<ProjectileMods>` (`ProjectileMods` =
`ProjectileSpec` : `modifiers`, `on_hit`, `on_expire`), absent = v1 exacte. Ce sont des
définitions (`ItemTable`/`SynergyTable`, ressources hors rollback, sans `Hash`) : le champ n'entre
dans aucun checksum. **Le tir ne le lit pas encore** (M4-T1) : le lint vérifie sa forme (mêmes
plages qu'une arme, doublon refusé dans la liste d'un objet) puis le refuse (`Unsupported`).

### Composition (`combat::projectile::compose`)

`compose(base, ajouts) -> ResolvedModifiers`, pure, `Fixed` seulement. Chaque liste est d'abord
résolue seule (`resolve_modifiers` : dans une liste, le dernier l'emporte, le lint y refuse les
doublons), puis les sources se combinent (règles par défaut de Q3, retenues par William le
2026-10-15) :

| Modificateur | Règle entre sources | Pourquoi |
|---|---|---|
| `Bounce(n)`, `Pierce(n)` | addition (saturante) | deux objets de rebond rebondissent plus |
| `Size(f)` | produit | comme une stat multiplicative |
| `Lifetime(n)` | maximum des sources qui en ont un | une larme vit au moins aussi longtemps |
| `Homing(f)` | somme plafonnée à 1 | le guidage se renforce sans dépasser le virage complet |
| `Gravity(g)` | somme | |
| `Spectral` | présence | |
| `Split` | addition des `count` ; `spread`, `child` de la première source | addition des branches |
| `Orbit` | présence ; paramètres de la première source | |

Ordre des sources, déterministe : arme (`base`), passifs dans l'ordre de ramassage (un passif en
double compte deux fois), actif, synergies actives par id ; `items::projectile_sources(items,
synergies, inventory)` rend les listes des objets dans cet ordre. Sans ajout,
`compose(base, &[]) == resolve_modifiers(base)` exactement. `resolve_modifiers` reste l'unique
appel au tir (inchangé) ; `ResolvedModifiers` n'est plus `Copy` (il porte `split`, `orbit`,
`spectral`, que `Projectile::new` ne lit pas encore).

### Nouveaux modificateurs (contrats)

En fin d'enum `ProjectileModifier` (les six d'avant gardent leur hash, test
`variantes_historiques_gardent_leur_hash`) :

- `Spectral` : traverse murs et obstacles, pas les personnages ;
- `Split(count, spread, child: Some("id"))` : à l'impact sur un personnage, `count` projectiles en
  éventail de `spread` radians ; `child` = id de la table `projectiles` de l'arme, absent = le
  projectile par défaut du porteur (Q3) ;
- `Orbit(radius, frames_per_turn)` : tourne autour du tireur au lieu d'avancer.

Lint : formes et plages (`Split.count > 0`, `spread >= 0`, `child` présent dans la table de
l'arme ; `Orbit.radius > 0`, `frames_per_turn > 0`), puis `Unsupported` sur une arme tant que
M4-T1 ne les exécute pas. Kinds `projectile_modifier` : `Spectral`, `Split`, `Orbit` (et `Fan`,
`Spiral`, `Burst`, qui manquaient aux `pattern` déclarés par `CombatPlugin`).

### Testbed et scénarios

Arme `larme` (`Automatic`, 3 tirs/s, `mag: Infinite`, sans `ammo_type`) et objet `lentille`
(`ShotSpeed Pct 0.5`, `ShotSize Mul 1.5`). Le testbed n'a pas de choix de personnage joueur dans
un scénario : le « personnage d'essai » de la fiche est l'arme, donnée par `PlayerScript::weapon`.
Scénarios : `tears_infinite` (300 frames de tir, `Ammo` à 0 tout du long, les coups continuent),
`tears_shot_speed` (avec `lentille`, la larme touche `target` à une frame où, dans le témoin
`tears_shot_speed_sans_objet`, elle n'est pas encore arrivée).


## 62. Contrats du monde : bombes, rochers, piques, portes verrouillées, grille (M4-T0c)

Contrats seuls, sans exécution : M4-T4 (bombes, rochers cassés, mur secret, piques) et M4-T5
(planificateur en grille) les exécutent. **Aucune trace existante ne bouge** : tout type nouveau est
neutre ou absent du contenu existant, l'input `Bomb` n'est lu par personne.

### Explosion (`effects::Action::Explode`)

`Explode { radius, damage, destroys }`, en fin d'enum `Action` (les hashs d'avant ne bougent pas).
Règle pure `world::explosion_plan(grid, center, radius, destroys, candidates)` : les cibles sont les
personnages à PV dont le centre est à **strictement** moins de `radius` (carré en `FixedWide`), rendus
dans l'ordre croissant de `GgrsNetId.0` ; `destroys` casse aussi les rochers de `CellGrid` du rayon
(`destroy_terrain`). Elle vit dans `world` (comme `destroy_terrain`, dans la frontière `SIM`), pas dans
`combat` : elle ne connaît que des identifiants et des positions. Lint : `radius > 0`, `damage >= 0`,
puis `Unsupported` dans un effet tant que M4-T4 ne l'exécute pas ; refusée dans un power-up. Le baril
**ne bascule pas** dessus ici (sa trace ne doit pas bouger) : M4-T4 le fera avec preuve. Moment clé
`explosion` (`world::Exploded`, `FrameEvents` neutre) déclaré, émis en T4.

### Bombe

Bit d'input `INPUT_BOMB` (18), bouton `Bomb` des scénarios (`game::replay::Button`), touche **E**
(présentation ; la manette attend Q17). `CharacterConfig.bomb: Option<world::BombDef>`
(`BombDef { fuse_frames, radius, damage }`, absent = pas de bombe, aucun contenu existant n'en a) ;
composant `world::Bomb { owner: GgrsNetId, fuse_until, radius, damage }`, rollback **neutre**.
`world::bomb_requested(buttons, def, bit)` est la lecture de l'input : faux sans `bomb`. Lint :
`fuse_frames > 0`, `radius > 0`, `damage >= 0`. `fuzz_inputs` : le bit 18 n'a aucun effet sans `bomb`
(personne ne le lit), donc aucune desync possible avant T4.

### Rochers de salle

Couche IntGrid optionnelle **`Rocks`** d'un gabarit LDtk (1 = rocher, 2 = rocher à butin ; le nom
final dépend de Q9). **Écart à la proposition de la fiche** (remplir `CellGrid`) : un rocher est **une
entité rollback** `world::Rock { cell, loot }` (neutre) avec collider (couche des murs) et `Obstacle`
`Wall`, créée à côté des murs (`map_ldtk::game::collider::spawn_level_rocks`), numérotée
`rock_<x>_<y>` dans l'ordre de lecture de la couche, niveaux triés par iid. Raison : `CellGrid` est
dense, ancrée à l'origine et non négative, et `rebuild_cave_walls_system` détruit **tous** les murs pour
les recréer depuis la grille et `reload_walls` remplace les cases murées de la navigation par celles
de la grille : sur un étage assemblé (salles à des positions quelconques, murs IntGrid propres) cela
effacerait les murs des salles. Le champ de flux voit l'`Obstacle` sans code de plus. Un niveau sans
la couche ne crée aucune entité ni aucun id : traces inchangées. M4-T4 détruira un rocher par
`despawn_rollback` (explosion `destroys`). Attente : `EntityCount(kind: Rock)`.

### Piques

`SurfaceDef.hazard: Option<HazardDef { damage, period }>` (fin de structure, absent des surfaces
existantes ; fichier `surfaces/*.ron`, champ `hazard`). Règle pure `HazardDef::is_due(last_hit,
frame)` : un coup au plus toutes les `period` frames. Lint : `damage > 0`, `period > 0`. Dégâts `True`
ou typés (demi-cœurs : Q2) et exécution en T4.

### Portes verrouillées

`RoomKindDef.door_lock: Option<DoorLock>`, `DoorLock { Key(n), Secret }` (`rooms/*.ron`, hors
sérialisation quand absent). `Key(n)` : porte fermée jusqu'à une Interaction avec `n` clés
(`KEY_ITEM`, consommées à l'ouverture) ; `Secret` : mur plein jusqu'à une explosion `destroys`. État
rollback **neutre** `world::DoorLocks` (ensemble des salles dont la porte est ouverte, vide par
défaut). Lint : `Key(0)`. Attente : `DoorLocked(kind, nth, locked, at_frame)` (même tri que
`RoomState`). Types réservés dans le testbed, aucun dans `gungeon` : `rooms/secrete.ron` (`Secret`),
`tresor.ron` (`Key(1)`), `diable.ron` (sémantique en T8).

### Disposition en grille

`FloorGrammar.layout: Option<FloorLayout>`, `FloorLayout { Graph (défaut), Grid { cell, max } }`
(Q8, proposition retenue : `cell` = 15 × 9 cases, une seule taille). Lint (`lint_floor_grammars`) :
`cell` et `max` > 0, `max` contient au moins `rooms.0` salles, et chaque gabarit de la grammaire
fait exactement `cell` cases avec une ouverture centrée sur chacun des quatre côtés. `plan_floor`
**refuse** `Grid` (`FloorError::Unsupported`) tant que M4-T5 n'est pas livrée.

### Testbed et scénarios

Carte `testbed/rocks_static.ldtk` (`scripts/construire-cartes-t0c.py`) et scénario `rocks_static` :
le joueur marche vers trois rochers qui ferment un couloir et s'y arrête ; quatre rochers comptés
(dont un à butin) à f5 et à la fin ; `joueur_hors_mur` tient.
