//! Armes, côté jeu : tout `combat::weapons` (simulation) plus ce qui touche aux assets et au rendu.
//!
//! `combat` est un crate de simulation (ECS seul, `make check_sim_boundary`) : il ne connaît ni
//! `Asset`, ni chargeur RON, ni `Visibility`. Les tables `weapons.ron` et `melee_weapons.ron` sont
//! désérialisées en [`WeaponAsset`] / [`melee::MeleeWeaponAsset`] (types de `combat`) ; leurs
//! enveloppes d'asset [`WeaponsConfig`] et [`melee::MeleeWeaponsConfig`], le rechargement à chaud
//! et la visibilité des armes portées vivent ici (dette D69). Les chemins `game::weapons::…`
//! d'avant ne changent pas : ce module réexporte `combat::weapons`.

use bevy::{platform::collections::HashMap, prelude::*, reflect::TypePath};
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_ggrs::RunGgrsSystems;
pub use combat::weapons::*;
use serde::{Deserialize, Serialize};

use melee::{MeleeAttackState, MeleeWeapon};

pub mod melee {
    pub use combat::weapons::melee::*;

    use bevy::{platform::collections::HashMap, prelude::*, reflect::TypePath};
    use serde::{Deserialize, Serialize};

    /// Table `melee_weapons.ron` (id -> arme de mêlée), chargée comme asset.
    #[derive(Asset, TypePath, Serialize, Deserialize)]
    pub struct MeleeWeaponsConfig(pub HashMap<String, MeleeWeaponAsset>);
}

/// Table `weapons.ron` (id -> arme à distance), chargée comme asset.
#[derive(Asset, TypePath, Serialize, Deserialize)]
pub struct WeaponsConfig(pub HashMap<String, WeaponAsset>);

/// Chargeurs RON des deux tables, rechargement à chaud, et visibilité des armes portées.
/// Présentation seule : rien ici n'écrit dans la simulation.
pub struct WeaponAssetsPlugin;

impl Plugin for WeaponAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<WeaponsConfig>::new(&["ron"]));
        app.add_plugins(RonAssetPlugin::<melee::MeleeWeaponsConfig>::new(&["ron"]));

        // Une arme de mêlée apparaît cachée (comme `spawn_melee_weapon_for_character` la posait
        // avant D69) : après `GgrsSchedule`, avant tout système d'`Update` qui y attache des sprites.
        app.add_systems(PreUpdate, hide_new_melee_weapons.after(RunGgrsSystems));
        app.add_systems(
            Update,
            (weapon_inventory_system, weapons_config_update_system),
        );
    }
}

/// Cache l'arme de mêlée qu'un personnage vient de recevoir (`Added<MeleeWeapon>`) : sans
/// `Visibility` sur le parent, les calques enfants ne seraient jamais affichés.
fn hide_new_melee_weapons(
    mut commands: Commands,
    new: Query<Entity, (Added<MeleeWeapon>, Without<Visibility>)>,
) {
    for entity in &new {
        commands.entity(entity).try_insert(Visibility::Hidden);
    }
}

/// Non rollback system to display the weapon correct sprite : l'arme active est visible (cachée
/// pendant une attaque de mêlée), les autres cachées. Pose aussi `Visibility` sur une arme qui n'en
/// a pas encore (la simulation n'en pose plus).
pub fn weapon_inventory_system(
    mut commands: Commands,
    query: Query<(Entity, &WeaponInventory, &MeleeAttackState)>,
    mut weapon_entities: Query<(Entity, Option<&mut Visibility>), With<Weapon>>,
) {
    for (_player_entity, inventory, melee_attack_state) in query.iter() {
        if inventory.weapons.is_empty() {
            continue;
        }

        // Update active/inactive weapon visibility
        for (i, (weapon_entity, _)) in inventory.weapons.iter().enumerate() {
            let is_active = i == inventory.active_weapon_index;

            if let Ok((_, visibility)) = weapon_entities.get_mut(*weapon_entity) {
                let target = if is_active {
                    commands.entity(*weapon_entity).insert(ActiveWeapon);
                    // Hide weapon during melee attacks
                    if melee_attack_state.is_attacking {
                        Visibility::Hidden
                    } else {
                        Visibility::Visible
                    }
                } else {
                    commands.entity(*weapon_entity).remove::<ActiveWeapon>();
                    Visibility::Hidden
                };
                match visibility {
                    Some(mut visibility) => *visibility = target,
                    None => {
                        commands.entity(*weapon_entity).insert(target);
                    }
                }
            }
        }
    }
}

pub fn weapons_config_update_system(
    _asset_server: Res<AssetServer>,

    weapons_config: Res<Assets<WeaponsConfig>>,

    mut ev_asset: MessageReader<AssetEvent<WeaponsConfig>>,

    mut query_weapons: Query<(&Children, Entity, &mut Weapon)>,
) {
    for event in ev_asset.read() {
        if let AssetEvent::Modified { id } = event {
            if let Some(weapons_config) = weapons_config.get(*id) {
                for (_childs, _entity, mut weapon) in query_weapons.iter_mut() {
                    if let Some(config) = weapons_config.0.get(&weapon.config.name) {
                        weapon.config = config.config.clone();
                        weapon.sprite_config = config.sprite_config.clone();
                    }
                }
            }
        }
    }
}
