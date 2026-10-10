//! Mise en œuvre de [`IMapGeneration`] pour la grammaire d'étage (M2-T10) : le plan est calculé
//! une fois (`floor::plan_floor`) au `get_spawning_room`, puis rendu salle par salle ; portes,
//! fenêtres et apparitions viennent de [`BasicMapGeneration`], qui lit les salles du plan.

use bevy_fixed::rng::RollbackRng;

use crate::{
    game::entity::map::player_spawn::PlayerSpawnConfig,
    generation::{
        config::MapGenerationMode,
        context::MapGenerationContext,
        entity::{
            character_spawn::CharacterSpawnConfig, door::DoorConfig, enemy_spawn::EnemySpawnConfig,
            location::EntityLocation, soda_location::SodaLocationConfig,
            weapon_location::WeaponLocationConfig, window::WindowConfig,
        },
        floor::{connection_of, plan_floor, FloorPlan},
        room::{Room, RoomConnection},
        IMapGeneration,
    },
};

use super::basic::BasicMapGeneration;

pub struct FloorMapGeneration {
    basic: BasicMapGeneration,
    plan: Option<FloorPlan>,
    /// Prochaine salle du plan à rendre.
    cursor: usize,
}

impl FloorMapGeneration {
    pub fn create(context: MapGenerationContext) -> Self {
        Self {
            basic: BasicMapGeneration::create(context),
            plan: None,
            cursor: 1,
        }
    }
}

impl IMapGeneration for FloorMapGeneration {
    fn get_spawning_room(&mut self, rng: &mut RollbackRng) -> Room {
        let MapGenerationMode::Floor(grammar) = &self.basic.context.config.mode else {
            unreachable!("FloorMapGeneration n'est créée que pour MapGenerationMode::Floor");
        };
        let plan = plan_floor(grammar, &self.basic.context, rng)
            .unwrap_or_else(|e| panic!("grammaire d'étage impossible : {e} (voir `alacod lint`)"));
        // Les salles finales (connexions complètes) sont celles que lisent les portes, fenêtres
        // et apparitions de `Basic`.
        self.basic.map.rooms = plan.rooms.iter().map(|p| p.room.clone()).collect();
        self.basic.map.room_depths = plan.rooms.iter().map(|p| p.depth).collect();
        self.basic.map.last_generated_room_index = Some(0);
        let spawn = plan.rooms[0].room.clone();
        self.plan = Some(plan);
        spawn
    }

    fn get_next_room(
        &mut self,
        _rng: &mut RollbackRng,
    ) -> Option<(Room, RoomConnection, RoomConnection)> {
        let plan = self.plan.as_ref()?;
        let planned = plan.rooms.get(self.cursor)?;
        self.cursor += 1;
        let (mine, theirs) = planned.via?;
        let parent = &plan.rooms[planned.parent?].room;
        Some((
            planned.room.clone(),
            connection_of(&planned.room, mine),
            connection_of(parent, theirs),
        ))
    }

    fn get_doors(&mut self, rng: &mut RollbackRng) -> Vec<(EntityLocation, DoorConfig)> {
        self.basic.get_doors(rng)
    }
    fn get_windows(&mut self, rng: &mut RollbackRng) -> Vec<(EntityLocation, WindowConfig)> {
        self.basic.get_windows(rng)
    }
    fn get_player_spawn(
        &mut self,
        rng: &mut RollbackRng,
    ) -> Vec<(EntityLocation, PlayerSpawnConfig)> {
        self.basic.get_player_spawn(rng)
    }
    fn get_enemy_spawns(
        &mut self,
        rng: &mut RollbackRng,
    ) -> Vec<(EntityLocation, EnemySpawnConfig)> {
        self.basic.get_enemy_spawns(rng)
    }
    fn get_character_spawns(
        &mut self,
        rng: &mut RollbackRng,
    ) -> Vec<(EntityLocation, CharacterSpawnConfig)> {
        self.basic.get_character_spawns(rng)
    }
    fn get_weapon_locations(
        &mut self,
        rng: &mut RollbackRng,
    ) -> Vec<(EntityLocation, WeaponLocationConfig)> {
        self.basic.get_weapon_locations(rng)
    }
    fn get_soda_locations(
        &mut self,
        rng: &mut RollbackRng,
    ) -> Vec<(EntityLocation, SodaLocationConfig)> {
        self.basic.get_soda_locations(rng)
    }
}
