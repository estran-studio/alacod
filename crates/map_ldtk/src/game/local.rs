//! Partie sur une map LDtk générée : configure la map dans le lobby et crée les
//! joueurs sur leurs points de spawn une fois la map chargée.
//!
//! Partagé par l'exemple `map_explorer` et les scénarios de test, pour qu'ils jouent
//! exactement la même partie.

use bevy::{platform::collections::HashMap, prelude::*};
use bevy_fixed::fixed_math;
use game::{
    character::{config::CharacterConfig, player::create::create_player},
    collider::CollisionSettings,
    core::AppState,
    global_asset::GlobalAsset,
    jjrs::{GggrsSessionConfigurationState, GgrsSessionBuilding},
    weapons::{melee::MeleeWeaponsConfig, WeaponsConfig},
};
use map::{game::entity::map::player_spawn::PlayerSpawnConfig, generation::config::MapGenerationConfig};
use utils::net_id::GgrsNetIdFactory;

use super::plugin::{LdtkMapLoadingEvent, MapNetIdAssignment};

/// Map et seed de génération de la partie.
#[derive(Resource, Clone, Debug)]
pub struct LdtkGameMap {
    /// Chemin du `.ldtk`, relatif au dossier des assets.
    pub map_path: String,
    pub seed: i32,
}

pub struct LdtkLocalGamePlugin(pub LdtkGameMap);

impl Plugin for LdtkLocalGamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.0.clone())
            .add_systems(OnEnter(AppState::LobbyLocal), configure_map)
            .add_systems(OnEnter(AppState::LobbyOnline), configure_map)
            .add_systems(
                Update,
                spawn_players_when_map_loaded
                    .run_if(on_message::<LdtkMapLoadingEvent>)
                    .after(MapNetIdAssignment),
            );
    }
}

fn configure_map(
    mut commands: Commands,
    map: Res<LdtkGameMap>,
    mut ggrs_state: ResMut<GggrsSessionConfigurationState>,
) {
    commands.insert_resource(MapGenerationConfig {
        seed: map.seed,
        map_path: map.map_path.clone(),
        max_width: 1000,
        max_heigth: 1000,
        ..Default::default()
    });
    ggrs_state.ready = true;
}

fn spawn_players_when_map_loaded(
    mut commands: Commands,
    collision_settings: Res<CollisionSettings>,
    global_assets: Res<GlobalAsset>,
    character_asset: Res<Assets<CharacterConfig>>,
    weapons_asset: Res<Assets<WeaponsConfig>>,
    melee_weapons_asset: Res<Assets<MeleeWeaponsConfig>>,
    mut id_provider: ResMut<GgrsNetIdFactory>,
    ggrs_session_building: Res<GgrsSessionBuilding>,
    player_spawn: Query<(&GlobalTransform, &PlayerSpawnConfig)>,
) {
    let spawns: HashMap<usize, &GlobalTransform> = player_spawn
        .iter()
        .map(|(transform, config)| (config.index, transform))
        .collect();

    info!("Map is loaded with {} player spawns", spawns.len());

    if spawns.is_empty() {
        return;
    }

    for ggrs_player in ggrs_session_building.players.iter() {
        let handle = ggrs_player.handle;
        let transform = spawns
            .get(&handle)
            .unwrap_or_else(|| panic!("pas de point de spawn pour le joueur {handle}"));

        if transform.translation().x == 0.0 && transform.translation().y == 0.0 {
            return;
        }

        create_player(
            &mut commands,
            &global_assets,
            &weapons_asset,
            &melee_weapons_asset,
            &character_asset,
            &collision_settings,
            fixed_math::vec3_to_fixed(transform.translation()),
            ggrs_player.is_local,
            handle,
            ggrs_player.name.clone(),
            ggrs_player.pubkey.clone(),
            &mut id_provider,
        );
    }
}
