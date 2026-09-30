use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use map::game::entity::map::soda_location::SodaLocationComponent;

use crate::map_const;

pub fn soda_location_component_from_field(
    entity_instance: &EntityInstance,
) -> SodaLocationComponent {
    let perk = entity_instance
        .get_string_field(map_const::FIELD_PERK_NAME)
        .ok()
        .cloned()
        .unwrap_or_default();
    SodaLocationComponent { perk }
}

/// **Pas** de `MapRollbackMarker` : voir la doc de `weapon_location::WeaponLocationBundle`,
/// même raisonnement (`spawn_soda_locations_when_map_loaded` lit `SodaLocationComponent`
/// directement, `test_map.ldtk` a des `SodaLocation` déjà posées sans champ configuré).
#[derive(Bundle, LdtkEntity)]
pub struct SodaLocationBundle {
    #[with(soda_location_component_from_field)]
    location: SodaLocationComponent,
    #[sprite_sheet]
    sprite_sheet: Sprite,
}

impl Default for SodaLocationBundle {
    fn default() -> Self {
        Self {
            location: SodaLocationComponent::default(),
            sprite_sheet: Sprite::default(),
        }
    }
}
