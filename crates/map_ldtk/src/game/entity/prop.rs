//! Entités LDtk `Table` et `Barrel` (M2-T11, `docs/conventions.md` §46) : posent un
//! `MapRollbackMarker` (`table` / `barrel`) et le nom de l'objet de décor (champ `prop`) ;
//! `plugin::spawn_map_items` crée l'entité rollback d'après `world::PropTable`.

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use map::game::entity::map::map_rollback::MapRollbackMarker;

use super::door::{ldtk_entity_size_from_instance, LdtkEntitySize};
use crate::map_const;

/// Id de l'objet de décor demandé par l'entité LDtk (champ `prop`), vide si absent.
#[derive(Component, Clone, Debug, Default)]
pub struct PropRef(pub String);

pub fn prop_ref_from_field(entity_instance: &EntityInstance) -> PropRef {
    PropRef(
        entity_instance
            .get_string_field(map_const::FIELD_PROP_NAME)
            .ok()
            .cloned()
            .unwrap_or_default(),
    )
}

#[derive(Bundle, LdtkEntity)]
pub struct TableBundle {
    #[with(prop_ref_from_field)]
    prop: PropRef,
    rollback_marker: MapRollbackMarker,
    #[sprite_sheet]
    sprite_sheet: Sprite,
    #[with(ldtk_entity_size_from_instance)]
    ldtk_size: LdtkEntitySize,
}

impl Default for TableBundle {
    fn default() -> Self {
        Self {
            prop: PropRef::default(),
            rollback_marker: MapRollbackMarker("table".into()),
            sprite_sheet: Sprite::default(),
            ldtk_size: LdtkEntitySize {
                width: 32.0,
                height: 16.0,
            },
        }
    }
}

#[derive(Bundle, LdtkEntity)]
pub struct BarrelBundle {
    #[with(prop_ref_from_field)]
    prop: PropRef,
    rollback_marker: MapRollbackMarker,
    #[sprite_sheet]
    sprite_sheet: Sprite,
    #[with(ldtk_entity_size_from_instance)]
    ldtk_size: LdtkEntitySize,
}

impl Default for BarrelBundle {
    fn default() -> Self {
        Self {
            prop: PropRef::default(),
            rollback_marker: MapRollbackMarker("barrel".into()),
            sprite_sheet: Sprite::default(),
            ldtk_size: LdtkEntitySize {
                width: 16.0,
                height: 16.0,
            },
        }
    }
}
