use crate::generation::entity::character_spawn::CharacterSpawnConfig;
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
    pub sodas: Vec<EntityLocation>,
    pub player_spawns: Vec<EntityLocation>,
    pub zombie_spawns: Vec<EntityLocation>,
    pub crates: Vec<EntityLocation>,
    pub weapons: Vec<EntityLocation>,
    pub windows: Vec<EntityLocation>,
    /// T2.9 (testbed) : `character`/`team` sont des valeurs d'auteur (voir
    /// `CharacterSpawnConfig`), portées avec leur position dès l'extraction (contrairement à
    /// `doors`, dont la config est recalculée plus tard à partir de la profondeur de la salle).
    pub character_spawns: Vec<(EntityLocation, CharacterSpawnConfig)>,
}
