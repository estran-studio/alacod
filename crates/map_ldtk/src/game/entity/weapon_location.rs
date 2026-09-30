use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use map::game::entity::map::weapon_location::WeaponLocationComponent;

use crate::map_const;

pub fn weapon_location_component_from_field(
    entity_instance: &EntityInstance,
) -> WeaponLocationComponent {
    let weapon = entity_instance
        .get_string_field(map_const::FIELD_WEAPON_NAME)
        .ok()
        .cloned()
        .unwrap_or_default();
    let price = entity_instance
        .get_int_field(map_const::FIELD_PRICE_NAME)
        .ok()
        .copied()
        .unwrap_or(0)
        .max(0) as u32;
    WeaponLocationComponent { weapon, price }
}

/// **Pas** de `MapRollbackMarker` (contrairement à `CharacterSpawnBundle`/`DoorBundle`) :
/// `wait_for_all_map_rollback_entity` (`crates/map_ldtk/src/game/plugin.rs`) promeut
/// **toute** entité qui en porte un en entité rollback à part entière (`GgrsNetId` compris),
/// même sans cas particulier pour son `kind` dans son `match`. `spawn_weapon_locations_when_map_loaded`
/// lit `WeaponLocationComponent` directement sur l'entité LDtk (pas besoin de cette
/// promotion) : lui ajouter le marqueur transformerait chaque `WeaponLocation` déjà posée
/// dans une carte (y compris `test_map.ldtk`, quatre depuis T2.6 — voir
/// `docs/conventions.md` §1) en entité rollback fantôme de plus, décalant la numérotation
/// `GgrsNetId` de tout ce qui est créé après (portes, joueurs, ennemis...) et changeant la
/// trace de **tous** les scénarios existants sans aucune raison de jeu.
#[derive(Bundle, LdtkEntity)]
pub struct WeaponLocationBundle {
    #[with(weapon_location_component_from_field)]
    location: WeaponLocationComponent,
    #[sprite_sheet]
    sprite_sheet: Sprite,
}

impl Default for WeaponLocationBundle {
    fn default() -> Self {
        Self {
            location: WeaponLocationComponent::default(),
            sprite_sheet: Sprite::default(),
        }
    }
}
