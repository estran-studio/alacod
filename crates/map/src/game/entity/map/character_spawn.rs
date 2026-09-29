use bevy::prelude::*;

/// Marqueur posé par l'entité LDtk `CharacterSpawn` (T2.9, testbed) : au chargement de la
/// map, `map_ldtk::game::local::spawn_characters_when_map_loaded` lit ce composant sur
/// l'entité d'origine LDtk et fait apparaître le personnage nommé via
/// `game::character::enemy::create::spawn_enemy`.
///
/// `team` reste une chaîne brute (`players`/`enemies`/`allies`/`neutral`) plutôt que
/// `sim_core::team::Team` : ce crate (`map`) ne dépend pas de `sim_core` ; le parsing vit
/// dans `game`/`map_ldtk`, qui en dépendent déjà.
#[derive(Component, Clone, Debug, Default)]
pub struct CharacterSpawnComponent {
    /// `CharacterId` du registre de contenu (`games/<jeu>/assets/characters/*.ron`).
    pub character: String,
    /// `players`/`enemies`/`allies`/`neutral` ; `None` (champ absent ou vide dans l'éditeur
    /// LDtk) retombe sur `CharacterConfig.team`, puis `Enemies`.
    pub team: Option<String>,
}
