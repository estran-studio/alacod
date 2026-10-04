//! Partie sur une map LDtk générée : configure la map dans le lobby et crée les
//! joueurs sur leurs points de spawn une fois la map chargée.
//!
//! Partagé par le jeu `zombies` (`games/zombies`) et les scénarios de test, pour qu'ils jouent
//! exactement la même partie.

use bevy::{ecs::system::SystemParam, platform::collections::HashMap, prelude::*};
use bevy_fixed::fixed_math;
use bevy_ggrs::Rollback;
use game::{
    character::{
        config::CharacterConfig, enemy::create::spawn_enemy, player::create::create_player,
    },
    collider::CollisionSettings,
    core::AppState,
    economy::PerkMachine,
    global_asset::GlobalAsset,
    interaction::{Interactable, InteractionType},
    jjrs::{GggrsSessionConfigurationState, GgrsSessionBuilding},
    weapons::{self, melee::MeleeWeaponsConfig, WeaponsConfig},
};
use map::{
    game::entity::map::{
        character_spawn::CharacterSpawnComponent, player_spawn::PlayerSpawnConfig,
        soda_location::SodaLocationComponent, weapon_location::WeaponLocationComponent,
    },
    generation::config::MapGenerationConfig,
};
use sim_core::team::Team;
use utils::net_id::GgrsNetIdFactory;

use super::floors::{FloorPlan, FloorSlots};
use super::plugin::{LdtkMapLoadingEvent, MapNetIdAssignment};

/// Ressources de contenu nécessaires pour créer les personnages et objets d'un niveau
/// (regroupées : les mêmes servent au chargement de la carte et au passage de niveau du mode
/// `Floors`, `super::floors`).
#[derive(SystemParam)]
pub struct LevelSpawnAssets<'w> {
    pub collision_settings: Res<'w, CollisionSettings>,
    pub global_assets: Res<'w, GlobalAsset>,
    // F5 (chantier m0-v11) : santé max résolue au lancement (`game::balance`).
    pub balance: Res<'w, game::balance::ResolvedBalance>,
    pub character_asset: Res<'w, Assets<CharacterConfig>>,
    pub weapons_asset: Res<'w, Assets<WeaponsConfig>>,
    pub melee_weapons_asset: Res<'w, Assets<MeleeWeaponsConfig>>,
    /// Graine de run (T1.5 : tirage des variantes, `game::character::variant`) : la graine de
    /// carte, celle que `system_after_map_loaded_local`/`p2p` donnera à `RunSeed`/`RngStreams`
    /// — qui ne sont posés qu'à `OnEnter(GameStarting)`, après l'apparition des personnages de
    /// la carte.
    pub map_config: Option<Res<'w, MapGenerationConfig>>,
    /// T1.9 : difficulté (santé des personnages × difficulté de l'étage où ils apparaissent ;
    /// 1 sans difficulté activée).
    pub difficulty: game::clock::DifficultyReader<'w>,
}

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
                    // `.chain()` ci-dessous : net_id déterministe (CLAUDE.md, « Numérotation
                    // des entités ») ; les quatre répondent au même événement, sans lien de
                    // données entre eux, donc un ordre total explicite est requis (sinon la
                    // numérotation varie d'un client à l'autre).
                    spawn_characters_when_map_loaded,
                    // T2.3, chantier C5 v1 (armes murales, machines à perks).
                    spawn_weapon_locations_when_map_loaded,
                    spawn_soda_locations_when_map_loaded,
                )
                    .chain()
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

#[allow(clippy::too_many_arguments)]
fn spawn_players_when_map_loaded(
    mut commands: Commands,
    collision_settings: Res<CollisionSettings>,
    global_assets: Res<GlobalAsset>,
    balance: Res<game::balance::ResolvedBalance>,
    character_asset: Res<Assets<CharacterConfig>>,
    weapons_asset: Res<Assets<WeaponsConfig>>,
    melee_weapons_asset: Res<Assets<MeleeWeaponsConfig>>,
    mut id_provider: ResMut<GgrsNetIdFactory>,
    ggrs_session_building: Res<GgrsSessionBuilding>,
    player_spawn: Query<(Entity, &GlobalTransform, &PlayerSpawnConfig)>,
    slots: FloorSlots,
) {
    // T1.8 : seuls les points de départ du premier niveau (emplacement 0 ; la carte unique
    // hors mode `Floors`).
    let spawns: HashMap<usize, &GlobalTransform> = player_spawn
        .iter()
        .filter(|(entity, _, _)| slots.slot_of(*entity) == 0)
        .map(|(_, transform, config)| (config.index, transform))
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

        // F5 (chantier m0-v11) : santé max résolue au lancement (`game::balance`).
        let health_max = balance
            .health_max_by_character
            .get("player")
            .copied()
            .unwrap_or_else(|| {
                panic!("équilibrage F5 : pas de santé résolue pour le personnage « player »")
            });

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
            health_max,
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
    assets: LevelSpawnAssets,
    mut id_provider: ResMut<GgrsNetIdFactory>,
    character_spawn: Query<(Entity, &GlobalTransform, &CharacterSpawnComponent)>,
    slots: FloorSlots,
    plan: Option<Res<FloorPlan>>,
    mut floor_state: ResMut<run::FloorState>,
) {
    let spawns: Vec<(&GlobalTransform, &CharacterSpawnComponent)> = character_spawn
        .iter()
        .filter(|(entity, _, _)| slots.slot_of(*entity) == 0)
        .map(|(_, transform, spawn)| (transform, spawn))
        .collect();
    let difficulty = assets.difficulty.current();
    let placed =
        spawn_level_characters(&mut commands, &assets, &mut id_provider, spawns, difficulty);
    // T1.8 : premier niveau du mode `Floors` (ennemis placés, pour les kills du résumé).
    if plan.is_some() {
        floor_state.enemies_placed = placed;
    }
}

/// Crée les personnages `CharacterSpawn` d'un niveau (chargement de la carte, ou passage de
/// niveau du mode `Floors`, `super::floors`) ; rend le nombre de personnages créés.
pub(crate) fn spawn_level_characters(
    commands: &mut Commands,
    assets: &LevelSpawnAssets,
    id_provider: &mut ResMut<GgrsNetIdFactory>,
    mut spawns: Vec<(&GlobalTransform, &CharacterSpawnComponent)>,
    // T1.9 : difficulté de l'étage où ces personnages apparaissent (voir
    // `game::clock::DifficultyReader::at_floor`).
    difficulty: bevy_fixed::fixed_math::Fixed,
) -> u32 {
    if spawns.is_empty() {
        return 0;
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

    let mut placed = 0;
    for (transform, spawn) in spawns {
        if spawn.character.is_empty() {
            warn!(
                "CharacterSpawn sans champ « character » à {:?}, ignoré",
                transform.translation()
            );
            continue;
        }
        let team = spawn.team.as_deref().and_then(parse_team);
        if spawn_character(
            commands,
            assets,
            id_provider,
            &spawn.character,
            fixed_math::vec3_to_fixed(transform.translation()),
            team,
            spawn.variant.as_deref(),
            difficulty,
        )
        .is_some()
        {
            placed += 1;
        }
    }
    placed
}

/// Crée un personnage comme un `CharacterSpawn` de carte (T1.13 : aussi le chemin des
/// placements scriptés d'un scénario, `Scenario::characters`) : équipe `team`, sinon
/// `CharacterConfig.team`, sinon `Team::Enemies` ; santé F5 × `difficulty` ; variante imposée
/// ou tirée (`spawn_enemy`). Rend l'entité créée, `None` (rien créé) pour un personnage inconnu.
#[allow(clippy::too_many_arguments)]
pub fn spawn_character(
    commands: &mut Commands,
    assets: &LevelSpawnAssets,
    id_provider: &mut ResMut<GgrsNetIdFactory>,
    character: &str,
    position: fixed_math::FixedVec3,
    team: Option<Team>,
    variant: Option<&str>,
    difficulty: bevy_fixed::fixed_math::Fixed,
) -> Option<Entity> {
    if !assets
        .global_assets
        .character_configs
        .contains_key(character)
    {
        warn!(
            "CharacterSpawn : personnage inconnu « {}» (voir `alacod lint`), ignoré",
            character
        );
        return None;
    }

    let character_config = assets
        .global_assets
        .character_configs
        .get(character)
        .and_then(|handle| assets.character_asset.get(handle));
    let team = team
        .or_else(|| character_config.and_then(|config| config.team))
        .unwrap_or(Team::Enemies);

    // F5 (chantier m0-v11) : santé max résolue au lancement (`game::balance`).
    let health_max = assets
        .balance
        .health_max_by_character
        .get(character)
        .copied()
        .unwrap_or_else(|| {
            panic!(
                "équilibrage F5 : pas de santé résolue pour le personnage « {} »",
                character
            )
        });
    let health_max = game::clock::scale(health_max, difficulty);

    Some(spawn_enemy(
        character.to_string(),
        position,
        commands,
        &assets.weapons_asset,
        &assets.melee_weapons_asset,
        &assets.character_asset,
        &assets.global_assets,
        &assets.collision_settings,
        id_provider,
        team,
        health_max,
        assets
            .map_config
            .as_ref()
            .map_or(0, |config| config.seed as u32),
        variant,
    ))
}

/// `WeaponLocation` (T2.3, chantier C5 v1) : fait apparaître, une fois au chargement de la
/// map, une arme murale (`weapons::WeaponPickup`, `price: Some(...)`, jamais consommée au
/// ramassage — voir `interaction::handle_weapon_pickup_interaction`) par entité LDtk
/// `WeaponLocation`. `docs/conventions.md` §1 notait qu'aucune `WeaponLocation` n'était lue :
/// ce chantier comble le trou.
///
/// Ordre déterministe (CLAUDE.md, « Numérotation des entités ») : trié par position (puis id
/// d'arme) avant `id_factory.next`, comme [`spawn_characters_when_map_loaded`].
fn spawn_weapon_locations_when_map_loaded(
    mut commands: Commands,
    global_assets: Res<GlobalAsset>,
    weapons_asset: Res<Assets<WeaponsConfig>>,
    mut id_provider: ResMut<GgrsNetIdFactory>,
    locations: Query<(Entity, &GlobalTransform, &WeaponLocationComponent)>,
    slots: FloorSlots,
) {
    let spawns = locations
        .iter()
        .filter(|(entity, _, _)| slots.slot_of(*entity) == 0)
        .map(|(_, transform, location)| (transform, location))
        .collect();
    spawn_level_weapon_locations(
        &mut commands,
        &global_assets,
        &weapons_asset,
        &mut id_provider,
        spawns,
    );
}

/// Armes murales d'un niveau (chargement, ou passage de niveau du mode `Floors`).
pub(crate) fn spawn_level_weapon_locations(
    commands: &mut Commands,
    global_assets: &GlobalAsset,
    weapons_asset: &Assets<WeaponsConfig>,
    id_provider: &mut ResMut<GgrsNetIdFactory>,
    mut spawns: Vec<(&GlobalTransform, &WeaponLocationComponent)>,
) {
    if spawns.is_empty() {
        return;
    }
    spawns.sort_by(|(a_transform, a_loc), (b_transform, b_loc)| {
        let a_pos = a_transform.translation();
        let b_pos = b_transform.translation();
        a_pos
            .x
            .total_cmp(&b_pos.x)
            .then_with(|| a_pos.y.total_cmp(&b_pos.y))
            .then_with(|| a_loc.weapon.cmp(&b_loc.weapon))
    });

    info!(
        "Carte chargée : {} emplacements d’armes dans les copies de gabarits de salles",
        spawns.len()
    );

    let Some(weapons_config) = weapons_asset.get(&global_assets.weapons) else {
        return;
    };

    for (transform, location) in spawns {
        if location.weapon.is_empty() {
            warn!(
                "WeaponLocation sans champ « weapon » à {:?}, ignoré",
                transform.translation()
            );
            continue;
        }
        let Some(weapon_asset) = weapons_config.0.get(&location.weapon) else {
            warn!(
                "WeaponLocation : arme inconnue « {} » (voir `alacod lint`), ignoré",
                location.weapon
            );
            continue;
        };
        let mag_ammo = weapons::default_mode_capacity(weapon_asset);
        let weapon: weapons::Weapon = weapon_asset.clone().into();
        weapons::spawn_weapon_pickup(
            commands,
            weapon,
            mag_ammo,
            fixed_math::vec3_to_fixed(transform.translation()),
            Some(location.price),
            id_provider,
        );
    }
}

/// `SodaLocation` (T2.3, chantier C5 v1) : fait apparaître, une fois au chargement de la
/// map, une machine à perk (`game::economy::PerkMachine`, `Interactable { interaction_type:
/// Perk }`) par entité LDtk `SodaLocation` ; consommée par
/// `interaction::handle_perk_purchase_interaction`.
fn spawn_soda_locations_when_map_loaded(
    mut commands: Commands,
    mut id_provider: ResMut<GgrsNetIdFactory>,
    locations: Query<(Entity, &GlobalTransform, &SodaLocationComponent)>,
    slots: FloorSlots,
) {
    let spawns = locations
        .iter()
        .filter(|(entity, _, _)| slots.slot_of(*entity) == 0)
        .map(|(_, transform, location)| (transform, location))
        .collect();
    spawn_level_soda_locations(&mut commands, &mut id_provider, spawns);
}

/// Machines à perk d'un niveau (chargement, ou passage de niveau du mode `Floors`).
pub(crate) fn spawn_level_soda_locations(
    commands: &mut Commands,
    id_provider: &mut ResMut<GgrsNetIdFactory>,
    mut spawns: Vec<(&GlobalTransform, &SodaLocationComponent)>,
) {
    if spawns.is_empty() {
        return;
    }
    spawns.sort_by(|(a_transform, a_loc), (b_transform, b_loc)| {
        let a_pos = a_transform.translation();
        let b_pos = b_transform.translation();
        a_pos
            .x
            .total_cmp(&b_pos.x)
            .then_with(|| a_pos.y.total_cmp(&b_pos.y))
            .then_with(|| a_loc.perk.cmp(&b_loc.perk))
    });

    info!("Map is loaded with {} soda locations", spawns.len());

    for (transform, location) in spawns {
        if location.perk.is_empty() {
            warn!(
                "SodaLocation sans champ « perk » à {:?}, ignoré",
                transform.translation()
            );
            continue;
        }

        let position = fixed_math::vec3_to_fixed(transform.translation());
        let ggrs_transform = fixed_math::FixedTransform3D::new(
            position,
            fixed_math::FixedMat3::IDENTITY,
            fixed_math::FixedVec3::ONE,
        );

        commands.spawn((
            ggrs_transform.to_bevy_transform(),
            ggrs_transform,
            Visibility::default(),
            PerkMachine {
                perk_id: location.perk.clone(),
            },
            Interactable {
                interaction_range: fixed_math::new(30.0),
                interaction_type: InteractionType::Perk,
            },
            id_provider.next(format!("perk_machine_{}", location.perk)),
            Rollback,
        ));
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
