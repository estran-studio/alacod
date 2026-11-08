//! Entité LDtk `RespawnPoint` (M-5, `docs/conventions.md` §113) : un point de reprise du feu follet
//! (« le prochain camp, le prochain feu »). Comme `HubStation`, l'entité de la carte ne porte que sa
//! donnée ; `local::spawn_respawn_points_when_map_loaded` crée l'entité rollback
//! `combat::downed::RespawnPoint`.
//!
//! Aucun nombre n'est codé : la **portée** est la moitié du plus grand côté de l'entité dessinée dans
//! LDtk (le level designer la dimensionne à l'œil), ou le champ flottant optionnel `range` (unités
//! monde) s'il est posé.

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::map_const;

/// Portée demandée par l'entité LDtk (unités monde).
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct RespawnPointRange(pub f32);

pub fn respawn_range_from_instance(entity_instance: &EntityInstance) -> RespawnPointRange {
    let drawn = entity_instance.width.max(entity_instance.height) as f32 / 2.0;
    RespawnPointRange(
        entity_instance
            .get_float_field(map_const::FIELD_RANGE_NAME)
            .ok()
            .copied()
            .unwrap_or(drawn),
    )
}

#[derive(Bundle, LdtkEntity)]
pub struct RespawnPointBundle {
    #[with(respawn_range_from_instance)]
    range: RespawnPointRange,
    #[sprite_sheet]
    sprite_sheet: Sprite,
}

impl Default for RespawnPointBundle {
    fn default() -> Self {
        Self {
            range: RespawnPointRange::default(),
            sprite_sheet: Sprite::default(),
        }
    }
}
