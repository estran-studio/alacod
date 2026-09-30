pub const LEVEL_FIELD_SPAWN: &str = "spawn";

pub const LAYER_CONNECTION: &str = "LevelConnection";
pub const LAYER_ENTITY: &str = "Entities";

pub const ENTITY_DOOR_HORIZONTAL_LOCATION: &str = "DoorHorizontal";
pub const ENTITY_DOOR_VERTICAL_LOCATION: &str = "DoorVertical";
pub const ENTITY_PLAYER_SPAWN_LOCATION: &str = "PlayerSpawn";
pub const ENTITY_ZOMBIE_SPAWN_LOCATION: &str = "ZombieSpawn";
pub const ENTITY_CRATE_LOCATION: &str = "CrateLocation";
pub const ENTITY_WEAPON_LOCATION: &str = "WeaponLocation";
pub const ENTITY_WINDOW_VERTICAL_LOCATION: &str = "WindowVertical";
pub const ENTITY_WINDOW_HORIZONTAL_LOCATION: &str = "WindowHorizontal";
pub const ENTITY_SODA_LOCATION: &str = "SodaLocation";
/// Entité de laboratoire (T2.9, testbed) : fait apparaître, au chargement de la map, le
/// personnage nommé par son champ `character` (`CharacterId` du registre), avec la `Team` de
/// son champ `team` si présent (voir `crate::game::local::spawn_characters_when_map_loaded`).
pub const ENTITY_CHARACTER_SPAWN_LOCATION: &str = "CharacterSpawn";

// pub const FIELD_BOOL_TYPE: &str = "Bool";
// pub const FIELD_INT_TYPE: &str = "Int";

pub const FIELD_PRICE_NAME: &str = "price";
// pub const FIELD_PRICE_TYPE: &str = FIELD_INT_TYPE;
pub const FIELD_ELECTRIFY_NAME: &str = "electrify";
// pub const FIELD_ELECTRIFY_TYPE: &str = FIELD_BOOL_TYPE;
pub const FIELD_INTERACTABLE_NAME: &str = "interactable";
pub const FIELD_PAIRED_DOOR_X_NAME: &str = "paired_door_x";
pub const FIELD_PAIRED_DOOR_Y_NAME: &str = "paired_door_y";
pub const FIELD_PAIRED_DOOR_LEVEL_NAME: &str = "paired_door_level";

pub const FIELD_PLAYER_SPAWN_INDEX_NAME: &str = "index";

/// Champs de l'entité `CharacterSpawn` (T2.9, testbed).
pub const FIELD_CHARACTER_NAME: &str = "character";
/// Optionnel : `players`/`enemies`/`allies`/`neutral` (voir
/// `crate::game::local::parse_team`) ; absent ou vide → `CharacterConfig.team`, sinon
/// `Enemies`.
pub const FIELD_TEAM_NAME: &str = "team";

/// Champ de l'entité `WeaponLocation` (T2.3, chantier C5 v1) : `WeaponId` du registre
/// (`weapons.ron`). `FIELD_PRICE_NAME` (même champ que `DoorHorizontal`/`DoorVertical`,
/// même type entier) porte le prix.
pub const FIELD_WEAPON_NAME: &str = "weapon";
/// Champ de l'entité `SodaLocation` (T2.3, chantier C5 v1) : `PerkId` du registre
/// (`economy/perks.ron`).
pub const FIELD_PERK_NAME: &str = "perk";
