mod imp;

pub mod config;
pub mod context;
pub mod entity;
pub mod position;
pub mod room;

use bevy_fixed::rng::RollbackRng;

use crate::{
    game::entity::map::player_spawn::PlayerSpawnConfig, generation::imp::get_implementation,
};

use self::{
    context::MapGenerationContext,
    entity::{
        character_spawn::CharacterSpawnConfig, door::DoorConfig, enemy_spawn::EnemySpawnConfig,
        location::EntityLocation, soda_location::SodaLocationConfig,
        weapon_location::WeaponLocationConfig, window::WindowConfig,
    },
    room::{Room, RoomConnection},
};

pub const LEVEL_PROPERTIES_SPAWN_NAME: &str = "spawn";
pub const LEVEL_PROPERTIES_GENERATION_NAME: &str = "generation";

trait IMapGeneration {
    // generate the first room that will be the game starting point
    fn get_spawning_room(&mut self, rng: &mut RollbackRng) -> Room;
    // generate the next room and provide the two connection used to create this room
    fn get_next_room(
        &mut self,
        rng: &mut RollbackRng,
    ) -> Option<(Room, RoomConnection, RoomConnection)>;

    fn get_doors(&mut self, rng: &mut RollbackRng) -> Vec<(EntityLocation, DoorConfig)>;
    fn get_windows(&mut self, rng: &mut RollbackRng) -> Vec<(EntityLocation, WindowConfig)>;
    fn get_player_spawn(
        &mut self,
        rng: &mut RollbackRng,
    ) -> Vec<(EntityLocation, PlayerSpawnConfig)>;
    fn get_enemy_spawns(
        &mut self,
        rng: &mut RollbackRng,
    ) -> Vec<(EntityLocation, EnemySpawnConfig)>;
    /// T2.9 (testbed) : personnages de laboratoire (`CharacterSpawn`), portés tels quels
    /// depuis les salles source (pas de règle de recalcul, contrairement aux portes).
    fn get_character_spawns(
        &mut self,
        rng: &mut RollbackRng,
    ) -> Vec<(EntityLocation, CharacterSpawnConfig)>;
    /// T2.3, chantier C5 v1 : armes murales (`WeaponLocation`), portées telles quelles
    /// (voir [`IMapGeneration::get_character_spawns`]).
    fn get_weapon_locations(
        &mut self,
        rng: &mut RollbackRng,
    ) -> Vec<(EntityLocation, WeaponLocationConfig)>;
    /// T2.3, chantier C5 v1 : machines à perks (`SodaLocation`), portées telles quelles.
    fn get_soda_locations(
        &mut self,
        rng: &mut RollbackRng,
    ) -> Vec<(EntityLocation, SodaLocationConfig)>;
}

pub trait IMapGenerator {
    fn add_room(
        &mut self,
        rng: &mut RollbackRng,
        room: &Room,
        connection_used: Option<&RoomConnection>,
        connected_to: Option<&RoomConnection>,
    );
    fn add_doors(&mut self, rng: &mut RollbackRng, doors: &Vec<(EntityLocation, DoorConfig)>);
    fn add_windows(&mut self, rng: &mut RollbackRng, windows: &Vec<(EntityLocation, WindowConfig)>);
    fn add_player_spawns(
        &mut self,
        rng: &mut RollbackRng,
        player_spawns: &Vec<(EntityLocation, PlayerSpawnConfig)>,
    );
    fn add_enemy_spawns(
        &mut self,
        rng: &mut RollbackRng,
        enemy_spawns: &Vec<(EntityLocation, EnemySpawnConfig)>,
    );
    /// T2.9 (testbed) : voir [`IMapGeneration::get_character_spawns`].
    fn add_character_spawns(
        &mut self,
        rng: &mut RollbackRng,
        character_spawns: &Vec<(EntityLocation, CharacterSpawnConfig)>,
    );
    /// T2.3, chantier C5 v1 : voir [`IMapGeneration::get_weapon_locations`].
    fn add_weapon_locations(
        &mut self,
        rng: &mut RollbackRng,
        weapon_locations: &Vec<(EntityLocation, WeaponLocationConfig)>,
    );
    /// T2.3, chantier C5 v1 : voir [`IMapGeneration::get_soda_locations`].
    fn add_soda_locations(
        &mut self,
        rng: &mut RollbackRng,
        soda_locations: &Vec<(EntityLocation, SodaLocationConfig)>,
    );
}

pub fn map_generation(
    context: MapGenerationContext,
    map_generator: &mut impl IMapGenerator,
) -> Result<(), ()> {
    //let mut generated_map = GeneratedMap::create(map_json.levels);
    let mut rng = RollbackRng::new(context.config.seed as u32);
    let mut generator = get_implementation(context);

    // select the spawning room
    let room = generator.get_spawning_room(&mut rng);

    map_generator.add_room(&mut rng, &room, None, None);

    while let Some((next_room, next_room_connection, other_room_connection)) =
        generator.get_next_room(&mut rng)
    {
        map_generator.add_room(
            &mut rng,
            &next_room,
            Some(&next_room_connection),
            Some(&other_room_connection),
        );
    }

    let doors = generator.get_doors(&mut rng);
    let windows = generator.get_windows(&mut rng);
    let player_spawns = generator.get_player_spawn(&mut rng);
    let enemy_spawns = generator.get_enemy_spawns(&mut rng);
    let character_spawns = generator.get_character_spawns(&mut rng);
    let weapon_locations = generator.get_weapon_locations(&mut rng);
    let soda_locations = generator.get_soda_locations(&mut rng);

    map_generator.add_doors(&mut rng, &doors);

    map_generator.add_windows(&mut rng, &windows);

    map_generator.add_player_spawns(&mut rng, &player_spawns);

    map_generator.add_enemy_spawns(&mut rng, &enemy_spawns);

    map_generator.add_character_spawns(&mut rng, &character_spawns);

    map_generator.add_weapon_locations(&mut rng, &weapon_locations);

    map_generator.add_soda_locations(&mut rng, &soda_locations);

    Ok(())
}
