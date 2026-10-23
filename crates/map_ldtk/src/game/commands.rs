//! Commandes d'édition appliquées dans `GgrsSchedule`.
use bevy::prelude::*;
use bevy_fixed::fixed_math;
use bevy_ggrs::GgrsSchedule;
use game::{
    game_commands::{GameCommand, GameCommands},
    items::spawn_item_pickup,
    powerups::{spawn_powerup_pickup, PowerUpsTable},
    system_set::RollbackSystemSet,
};
use items::ItemTable;
use utils::{frame::FrameCount, net_id::GgrsNetIdFactory};

pub fn clear_game_commands(mut commands: ResMut<GameCommands>) {
    commands.0.clear();
}
fn has_commands(commands: Res<GameCommands>) -> bool {
    !commands.0.is_empty()
}

pub fn apply_game_commands_spawn(
    frame: Res<FrameCount>,
    commands_by_frame: Res<GameCommands>,
    assets: super::local::LevelSpawnAssets,
    mut commands: Commands,
    mut ids: ResMut<GgrsNetIdFactory>,
    floor: Option<Res<run::FloorState>>,
) {
    let Some(list) = commands_by_frame.0.get(&frame.frame) else {
        return;
    };
    let floor = floor.as_deref().map_or(0, |state| state.index);
    for (rank, command) in list.iter().enumerate() {
        let GameCommand::SpawnCharacter {
            character,
            x,
            y,
            team,
            variant,
        } = command
        else {
            continue;
        };
        let Some(config) = assets.character_configs.get(character) else {
            continue;
        };
        if variant.as_ref().is_some_and(|name| {
            config
                .variants
                .as_ref()
                .is_none_or(|v| !v.table.contains_key(name))
        }) {
            continue;
        }
        let position = fixed_math::FixedVec3::new(*x, *y, fixed_math::FIXED_ZERO);
        if super::local::spawn_character(
            &mut commands,
            &assets,
            &mut ids,
            character,
            position,
            *team,
            variant.as_deref(),
            assets.difficulty.current(),
            assets.difficulty.variant_context_at(floor),
        )
        .is_some()
        {
            info!(
                "ggrs{{f={} editor_command kind=spawn_character rank={} character={} pos=({},{})}}",
                frame.frame, rank, character, x, y
            );
        }
    }
}

pub fn apply_game_commands_pickups(
    frame: Res<FrameCount>,
    commands_by_frame: Res<GameCommands>,
    powerups: Res<PowerUpsTable>,
    items: Res<ItemTable>,
    mut commands: Commands,
    mut ids: ResMut<GgrsNetIdFactory>,
) {
    let Some(list) = commands_by_frame.0.get(&frame.frame) else {
        return;
    };
    for (rank, command) in list.iter().enumerate() {
        match command {
            GameCommand::SpawnPowerUp { id, x, y } => {
                let Some(def) = powerups.0.as_ref().and_then(|table| table.powerups.get(id)) else {
                    continue;
                };
                spawn_powerup_pickup(
                    &mut commands,
                    id.clone(),
                    fixed_math::FixedVec3::new(*x, *y, fixed_math::FIXED_ZERO),
                    frame.frame.saturating_add(def.lifetime_frames),
                    &mut ids,
                );
                info!(
                    "ggrs{{f={} editor_command kind=spawn_powerup rank={} id={} pos=({},{})}}",
                    frame.frame, rank, id, x, y
                );
            }
            GameCommand::SpawnItem { id, x, y } => {
                if spawn_item_pickup(
                    &mut commands,
                    &items,
                    id.clone(),
                    fixed_math::FixedVec3::new(*x, *y, fixed_math::FIXED_ZERO),
                    &mut ids,
                )
                .is_none()
                {
                    continue;
                }
                info!(
                    "ggrs{{f={} editor_command kind=spawn_item rank={} id={} pos=({},{})}}",
                    frame.frame, rank, id, x, y
                );
            }
            GameCommand::SpawnCharacter { .. } => {}
        }
    }
}

pub fn register(app: &mut App) {
    app.add_systems(
        OnEnter(game::session::AppState::GameLoading),
        clear_game_commands,
    )
    .add_systems(
        GgrsSchedule,
        apply_game_commands_spawn
            .run_if(has_commands)
            .in_set(RollbackSystemSet::EnemySpawning)
            .after(game::waves::systems::wave_spawning_system)
            .after(game::character::enemy::spawning::enemy_spawn_from_spawners_system),
    )
    .add_systems(
        GgrsSchedule,
        apply_game_commands_pickups
            .run_if(has_commands)
            .in_set(RollbackSystemSet::Effects)
            .before(game::powerups::powerup_pickup_detect_system)
            // `Effects` précède `EnemySpawning` : pickups d'abord, personnages ensuite (comme le runner).
            .before(game::items::item_consumable_pickup_system),
    );
}
