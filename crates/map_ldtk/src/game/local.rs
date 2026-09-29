//! Partie sur une map LDtk générée : configure la map dans le lobby et crée les
//! joueurs sur leurs points de spawn une fois la map chargée.
//!
//! Partagé par le jeu `zombies` (`games/zombies`) et les scénarios de test, pour qu'ils jouent
//! exactement la même partie.

use bevy::{platform::collections::HashMap, prelude::*};
use bevy_fixed::fixed_math;
use game::{
    character::{
        config::CharacterConfig, enemy::create::spawn_enemy, player::create::create_player,
    },
    collider::CollisionSettings,
    core::AppState,
    global_asset::GlobalAsset,
    jjrs::{GggrsSessionConfigurationState, GgrsSessionBuilding},
    weapons::{melee::MeleeWeaponsConfig, WeaponsConfig},
};
use map::{
    game::entity::map::{
        character_spawn::CharacterSpawnComponent, player_spawn::PlayerSpawnConfig,
    },
    generation::config::MapGenerationConfig,
};
use sim_core::team::Team;
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
                (
                    spawn_players_when_map_loaded,
                    // Ordonné après les joueurs (net_id déterministe : CLAUDE.md, « Numérotation
                    // des entités ») ; les deux répondent au même événement, sans lien de données
                    // entre eux, donc un ordre explicite est requis (sinon la numérotation varie
                    // d'un client à l'autre).
                    spawn_characters_when_map_loaded.after(spawn_players_when_map_loaded),
                )
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

/// `CharacterSpawn` (T2.9, testbed) : fait apparaître, une fois au chargement de la map,
/// chaque personnage de laboratoire placé sur la carte (`dummy`, `target`, `follower`,
/// `ally`, `civilian`, ...), via `spawn_enemy` (le même chemin que les ennemis de vague :
/// `Enemy`, `EnemyAiConfig`, etc., seule la `Team` diffère).
///
/// Équipe : le champ `team` de l'entité LDtk, sinon `CharacterConfig.team` du personnage,
/// sinon `Team::Enemies` (voir `docs/conventions.md`).
///
/// Ordre déterministe (CLAUDE.md, « Numérotation des entités ») : les entités sont triées par
/// position puis par nom de personnage avant d'appeler `id_factory.next`, indépendamment de
/// l'ordre d'itération de la query.
fn spawn_characters_when_map_loaded(
    mut commands: Commands,
    collision_settings: Res<CollisionSettings>,
    global_assets: Res<GlobalAsset>,
    character_asset: Res<Assets<CharacterConfig>>,
    weapons_asset: Res<Assets<WeaponsConfig>>,
    melee_weapons_asset: Res<Assets<MeleeWeaponsConfig>>,
    mut id_provider: ResMut<GgrsNetIdFactory>,
    character_spawn: Query<(&GlobalTransform, &CharacterSpawnComponent)>,
) {
    let mut spawns: Vec<(&GlobalTransform, &CharacterSpawnComponent)> =
        character_spawn.iter().collect();
    if spawns.is_empty() {
        return;
    }
    spawns.sort_by(|(a_transform, a_spawn), (b_transform, b_spawn)| {
        let a_pos = a_transform.translation();
        let b_pos = b_transform.translation();
        a_pos
            .x
            .total_cmp(&b_pos.x)
            .then_with(|| a_pos.y.total_cmp(&b_pos.y))
            .then_with(|| a_spawn.character.cmp(&b_spawn.character))
    });

    info!("Map is loaded with {} character spawns", spawns.len());

    for (transform, spawn) in spawns {
        if spawn.character.is_empty() {
            warn!(
                "CharacterSpawn sans champ « character » à {:?}, ignoré",
                transform.translation()
            );
            continue;
        }
        if !global_assets
            .character_configs
            .contains_key(&spawn.character)
        {
            warn!(
                "CharacterSpawn : personnage inconnu « {}» (voir `alacod lint`), ignoré",
                spawn.character
            );
            continue;
        }

        let character_config = global_assets
            .character_configs
            .get(&spawn.character)
            .and_then(|handle| character_asset.get(handle));
        let team = spawn
            .team
            .as_deref()
            .and_then(parse_team)
            .or_else(|| character_config.and_then(|config| config.team))
            .unwrap_or(Team::Enemies);

        spawn_enemy(
            spawn.character.clone(),
            fixed_math::vec3_to_fixed(transform.translation()),
            &mut commands,
            &weapons_asset,
            &melee_weapons_asset,
            &character_asset,
            &global_assets,
            &collision_settings,
            &mut id_provider,
            team,
        );
    }
}

/// `players`/`enemies`/`allies`/`neutral` (RON en minuscules, champ `team` de l'entité LDtk
/// `CharacterSpawn`) ; toute autre valeur (faute de frappe) revient au repli de l'appelant
/// (`CharacterConfig.team`, puis `Enemies`) plutôt que de faire échouer le chargement.
fn parse_team(name: &str) -> Option<Team> {
    match name {
        "players" => Some(Team::Players),
        "enemies" => Some(Team::Enemies),
        "allies" => Some(Team::Allies),
        "neutral" => Some(Team::Neutral),
        _ => {
            warn!("CharacterSpawn : champ « team » inconnu « {name} », ignoré");
            None
        }
    }
}
