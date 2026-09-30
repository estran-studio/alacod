use crate::generation::entity::character_spawn::CharacterSpawnConfig;
use crate::generation::entity::soda_location::SodaLocationConfig;
use crate::generation::entity::weapon_location::WeaponLocationConfig;
use crate::generation::position::Position;

#[derive(Debug, Clone)]
pub struct EntityLocation {
    pub level_iid: String,
    pub position: Position,
    pub size: (i32, i32),
}

#[derive(Debug, Clone)]
pub struct EntityLocations {
    pub doors: Vec<EntityLocation>,
    /// T2.3, chantier C5 v1 : `perk` est une valeur d'auteur (voir `SodaLocationConfig`),
    /// portée comme `character_spawns` ci-dessous (pas recalculée).
    pub sodas: Vec<(EntityLocation, SodaLocationConfig)>,
    pub player_spawns: Vec<EntityLocation>,
    pub zombie_spawns: Vec<EntityLocation>,
    pub crates: Vec<EntityLocation>,
    /// T2.3, chantier C5 v1 : `weapon`/`price` sont des valeurs d'auteur (voir
    /// `WeaponLocationConfig`), portées comme `character_spawns` ci-dessous (pas recalculées).
    pub weapons: Vec<(EntityLocation, WeaponLocationConfig)>,
    pub windows: Vec<EntityLocation>,
    /// T2.9 (testbed) : `character`/`team` sont des valeurs d'auteur (voir
    /// `CharacterSpawnConfig`), portées avec leur position dès l'extraction (contrairement à
    /// `doors`, dont la config est recalculée plus tard à partir de la profondeur de la salle).
    pub character_spawns: Vec<(EntityLocation, CharacterSpawnConfig)>,
}
