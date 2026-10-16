//! Entité LDtk `HubStation` (M2-T13, `docs/conventions.md` §57) : le champ `service` nomme un
//! service du hub (`hub/<id>.ron`). Comme `SodaLocation`, l'entité de la carte ne porte que ses
//! données ; `local::spawn_hub_stations_when_map_loaded` crée l'entité rollback.

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::map_const;

/// Service demandé par l'entité LDtk (champ `service`), vide si absent.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub struct HubStationRef(pub String);

pub fn hub_station_ref_from_field(entity_instance: &EntityInstance) -> HubStationRef {
    HubStationRef(
        entity_instance
            .get_string_field(map_const::FIELD_SERVICE_NAME)
            .ok()
            .cloned()
            .unwrap_or_default(),
    )
}

#[derive(Bundle, LdtkEntity)]
pub struct HubStationBundle {
    #[with(hub_station_ref_from_field)]
    station: HubStationRef,
    #[sprite_sheet]
    sprite_sheet: Sprite,
}

impl Default for HubStationBundle {
    fn default() -> Self {
        Self {
            station: HubStationRef::default(),
            sprite_sheet: Sprite::default(),
        }
    }
}
