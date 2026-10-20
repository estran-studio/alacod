//! Enveloppes `Asset` des tables d'armes, synchro du registre de simulation et visibilité des armes
//! portées (D69 étape 2, E3b).
//!
//! `weapons.ron` et `melee_weapons.ron` sont désérialisées en [`WeaponAsset`] / [`MeleeWeaponAsset`]
//! (types de `combat`, donnée pure). Ce fichier, côté application, les charge comme assets Bevy
//! ([`WeaponsConfig`], [`MeleeWeaponsConfig`]) et publie [`WeaponRegistry`], que lit la simulation.
//! Il porte aussi tout ce qui touche à `Visibility` et au rechargement à chaud des armes déjà créées.
//!
//! # Moment du remplissage (décision D2 du plan, `docs/taches/rapports/dette-d69-e3b-weapon-registry.md`)
//!
//! [`sync_weapon_registry`] tourne en `PreUpdate`, **avant** `RunGgrsSystems` (donc avant tout tick de
//! `GgrsSchedule`), à chaque frame (même mécanique que `sync_character_configs`, E3a) :
//!
//! - **table vide** du registre dont l'asset est lisible : copiée. C'est le chemin du chargement
//!   initial ; il ne dépend d'aucun `AssetEvent` (écrits en `PostUpdate`, une frame après l'insertion
//!   de l'asset). Une table déjà remplie n'est plus touchée : une surcharge de scénario posée dessus
//!   n'est jamais écrasée ;
//! - **`AssetEvent::Modified`** (rechargement à chaud, `file_watcher` natif) : la table de l'asset
//!   modifié **remplace** celle du registre (une arme retirée du fichier disparaît, une surcharge de
//!   scénario aussi). `Added`/`LoadedWithDependencies` ne déclenchent rien.
//!
//! `loading_asset_system` ne quitte `AppState::Loading` qu'une fois tous les assets chargés, et le
//! remplissage a lieu au `PreUpdate` suivant au plus tard, avant le `StateTransition` qui entre dans le
//! lobby puis `GameLoading` : le registre est plein avant le moindre spawn de personnage.

use std::collections::BTreeMap;

use bevy::{platform::collections::HashMap, prelude::*, reflect::TypePath};
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_ggrs::RunGgrsSystems;
use serde::{Deserialize, Serialize};

use crate::global_asset::GlobalAsset;
use crate::weapons::melee::{MeleeAttackState, MeleeWeapon, MeleeWeaponAsset};
use crate::weapons::{ActiveWeapon, Weapon, WeaponAsset, WeaponInventory, WeaponRegistry};

/// Table `weapons.ron` (id -> arme à distance), chargée comme asset.
#[derive(Asset, TypePath, Serialize, Deserialize)]
pub struct WeaponsConfig(pub HashMap<String, WeaponAsset>);

/// Table `melee_weapons.ron` (id -> arme de mêlée), chargée comme asset.
#[derive(Asset, TypePath, Serialize, Deserialize)]
pub struct MeleeWeaponsConfig(pub HashMap<String, MeleeWeaponAsset>);

/// Chargeurs RON des deux tables, synchro du registre, rechargement à chaud, et visibilité des armes
/// portées. Rien ici n'écrit dans la simulation, hors le registre (donnée statique).
pub struct WeaponAssetsPlugin;

impl Plugin for WeaponAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<WeaponsConfig>::new(&["ron"]));
        app.add_plugins(RonAssetPlugin::<MeleeWeaponsConfig>::new(&["ron"]));
        app.init_resource::<WeaponRegistry>();

        app.add_systems(PreUpdate, sync_weapon_registry.before(RunGgrsSystems));
        app.add_systems(Update, weapons_config_update_system);
    }
}

fn copy_table<T: Clone>(from: &HashMap<String, T>, into: &mut BTreeMap<String, T>) {
    into.clear();
    into.extend(from.iter().map(|(id, weapon)| (id.clone(), weapon.clone())));
}

/// Publie les tables d'armes chargées dans [`WeaponRegistry`] (voir la doc du module).
pub fn sync_weapon_registry(
    global: Option<Res<GlobalAsset>>,
    weapons: Res<Assets<WeaponsConfig>>,
    melee: Res<Assets<MeleeWeaponsConfig>>,
    mut weapon_events: MessageReader<AssetEvent<WeaponsConfig>>,
    mut melee_events: MessageReader<AssetEvent<MeleeWeaponsConfig>>,
    mut registry: ResMut<WeaponRegistry>,
) {
    // Toujours vidés, même sans `GlobalAsset` : les curseurs ne prennent pas de retard.
    let weapons_modified: Vec<AssetId<WeaponsConfig>> = weapon_events
        .read()
        .filter_map(|event| match event {
            AssetEvent::Modified { id } => Some(*id),
            _ => None,
        })
        .collect();
    let melee_modified: Vec<AssetId<MeleeWeaponsConfig>> = melee_events
        .read()
        .filter_map(|event| match event {
            AssetEvent::Modified { id } => Some(*id),
            _ => None,
        })
        .collect();
    let Some(global) = global else {
        return;
    };

    if let Some(asset) = weapons.get(&global.weapons) {
        let is_modified = weapons_modified.contains(&global.weapons.id());
        if is_modified || (registry.weapons.is_empty() && !asset.0.is_empty()) {
            if is_modified {
                info!(
                    "armes rechargées : WeaponRegistry mis à jour ({} armes)",
                    asset.0.len()
                );
            } else {
                debug!("armes : WeaponRegistry rempli ({} armes)", asset.0.len());
            }
            copy_table(&asset.0, &mut registry.weapons);
        }
    }
    if let Some(asset) = melee.get(&global.melee_weapons) {
        let is_modified = melee_modified.contains(&global.melee_weapons.id());
        if is_modified || (registry.melee.is_empty() && !asset.0.is_empty()) {
            if is_modified {
                info!(
                    "armes de mêlée rechargées : WeaponRegistry mis à jour ({} armes)",
                    asset.0.len()
                );
            } else {
                debug!(
                    "armes de mêlée : WeaponRegistry rempli ({} armes)",
                    asset.0.len()
                );
            }
            copy_table(&asset.0, &mut registry.melee);
        }
    }
}

/// Cache l'arme de mêlée qui n'a pas encore de `Visibility` (`spawn_melee_weapon_for_character` ne
/// pose plus que la simulation). Filtre `Without<Visibility>` plutôt que `Added` : une arme
/// ressuscitée par un rollback sans avoir été vue garde le même résultat.
pub fn hide_new_melee_weapons(
    mut commands: Commands,
    new: Query<Entity, (With<MeleeWeapon>, Without<Visibility>)>,
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
                    commands.entity(*weapon_entity).try_insert(ActiveWeapon);
                    // Hide weapon during melee attacks
                    if melee_attack_state.is_attacking {
                        Visibility::Hidden
                    } else {
                        Visibility::Visible
                    }
                } else {
                    commands.entity(*weapon_entity).try_remove::<ActiveWeapon>();
                    Visibility::Hidden
                };
                match visibility {
                    Some(mut visibility) => *visibility = target,
                    None => {
                        commands.entity(*weapon_entity).try_insert(target);
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
