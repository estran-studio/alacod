//! **Prototype jetable** (m1-proto-etage-salles, voie B « un étage = un monde assemblé ») :
//! salles typées et verrouillées sur l'assembleur `Basic` existant.
//!
//! - Le type de salle vient du champ de niveau LDtk `room_kind` (recopié du gabarit par
//!   l'assembleur), posé en [`RoomKindTag`] sur l'entité du niveau, à côté de `RoomBounds`.
//! - Au premier tick, toutes les portes des cartes typées s'ouvrent (une porte Gungeon est
//!   ouverte par défaut) ; [`DoorShape`] garde de quoi les refermer.
//! - Une salle `combat` passe `Dormant → Locked` quand un joueur debout est à l'intérieur et
//!   qu'un ennemi y est vivant (portes de la salle et portes jumelles refermées), puis
//!   `Locked → Cleared` quand plus aucun ennemi n'y est (portes rouvertes).
//!
//! État rollback : [`RoomLocks`] (ressource neutre). Les colliders des portes sont déjà
//! rollback (`Collider`, `CollisionLayer`). Le champ de flux suit tout seul (portes avec
//! `Collider` = murs, `rebuild_blocked_cells`).

use bevy::prelude::*;
use bevy_fixed::fixed_math::{self, FixedTransform3D, FixedVec2};
use bevy_ggrs::GgrsSchedule;
use std::collections::BTreeMap;

use game::character::enemy::Enemy;
use game::character::player::Player;
use game::collider::{Collider, CollisionLayer};
use map::game::entity::map::door::DoorComponent;
use map::game::entity::map::level_id::LevelId;
use map::game::entity::map::room::RoomBounds;
use sim_core::system_set::RollbackSystemSet;
use utils::net_id::GgrsNetId;
use utils::order_iter;
use utils::rollback::RollbackTraceApp;

/// Type de salle (`depart`, `combat`, `recompense`…), statique.
#[derive(Component, Clone, Debug)]
pub struct RoomKindTag(pub String);

/// Collider d'une porte fermée, pour la refermer (statique, hors rollback).
#[derive(Component, Clone, Debug)]
pub struct DoorShape {
    pub collider: Collider,
    pub layer: CollisionLayer,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RoomStep {
    #[default]
    Dormant,
    Locked,
    Cleared,
}

/// État des salles typées, par `LevelId` (rollback, neutre : vide hors cartes typées).
#[derive(Resource, Clone, Debug, Default, PartialEq, Hash)]
pub struct RoomLocks {
    pub doors_opened: bool,
    pub rooms: BTreeMap<String, RoomStep>,
}

pub struct RoomsProtoPlugin;

impl Plugin for RoomsProtoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoomLocks>()
            .rollback_and_trace_resource_neutral::<RoomLocks>()
            .add_systems(
                GgrsSchedule,
                room_lock_system
                    .after(super::floors::floor_transition_system)
                    .in_set(RollbackSystemSet::Run),
            );
    }
}

/// Marge (px) : une porte appartient à une salle si elle est à moins de `DOOR_MARGIN` de ses
/// bords (la porte de la salle et sa jumelle, posée au bord de la salle voisine) ; un joueur
/// est « dans » la salle s'il est à plus de `PLAYER_INSET` des bords (passé la porte).
const DOOR_MARGIN: f32 = 24.0;
const PLAYER_INSET: f32 = 24.0;

fn inside(bounds: &RoomBounds, p: FixedVec2, margin: f32) -> bool {
    let m = fixed_math::new(margin);
    p.x >= bounds.position.x + m
        && p.y >= bounds.position.y + m
        && p.x <= bounds.position.x + bounds.size.x - m
        && p.y <= bounds.position.y + bounds.size.y - m
}

#[allow(clippy::type_complexity)]
pub fn room_lock_system(
    mut commands: Commands,
    mut locks: ResMut<RoomLocks>,
    rooms: Query<(&LevelId, &RoomBounds, &RoomKindTag)>,
    doors: Query<(&GgrsNetId, Entity, &FixedTransform3D, &DoorShape, Has<Collider>), With<DoorComponent>>,
    mut enemies: Query<(&FixedTransform3D, &mut combat::actors::Velocity), With<Enemy>>,
    players: Query<&FixedTransform3D, (With<Player>, Without<combat::downed::Downed>)>,
) {
    let mut typed: Vec<(&LevelId, &RoomBounds, &RoomKindTag)> = rooms.iter().collect();
    if typed.is_empty() {
        return;
    }
    typed.sort_by(|a, b| a.0 .0.cmp(&b.0 .0));

    let door_near = |bounds: &RoomBounds, t: &FixedTransform3D| {
        inside(bounds, t.translation.truncate(), -DOOR_MARGIN)
    };

    // Portes ouvertes par défaut (une fois)
    if !locks.doors_opened {
        for (_, entity, _, _, closed) in order_iter!(doors) {
            if closed {
                commands
                    .entity(entity)
                    .remove::<(Collider, CollisionLayer, game::interaction::Interactable)>();
            }
        }
        locks.doors_opened = true;
    }

    for (level_id, bounds, kind) in typed {
        if kind.0 != "combat" {
            continue;
        }
        let step = locks.rooms.get(&level_id.0).copied().unwrap_or_default();
        let enemies_inside = enemies
            .iter()
            .filter(|(t, _)| inside(bounds, t.translation.truncate(), 0.0))
            .count();
        let next = match step {
            RoomStep::Dormant
                if enemies_inside > 0
                    && players
                        .iter()
                        .any(|t| inside(bounds, t.translation.truncate(), PLAYER_INSET)) =>
            {
                for (_, entity, t, shape, closed) in order_iter!(doors) {
                    if !closed && door_near(bounds, t) {
                        commands
                            .entity(entity)
                            .insert((shape.collider.clone(), shape.layer.clone()));
                    }
                }
                info!("salle {} verrouillée ({enemies_inside} ennemis)", level_id.0);
                RoomStep::Locked
            }
            RoomStep::Locked if enemies_inside == 0 => {
                for (_, entity, t, _, closed) in order_iter!(doors) {
                    if closed && door_near(bounds, t) {
                        commands
                            .entity(entity)
                            .remove::<(Collider, CollisionLayer)>();
                    }
                }
                info!("salle {} nettoyée", level_id.0);
                RoomStep::Cleared
            }
            step => step,
        };
        // Activation par salle : tant qu'une salle de combat dort, ses ennemis ne bougent pas
        // (vitesse annulée après l'IA ; le déplacement de la frame suivante l'applique).
        if next == RoomStep::Dormant {
            for (t, mut velocity) in enemies.iter_mut() {
                if inside(bounds, t.translation.truncate(), 0.0) {
                    velocity.main = fixed_math::FixedVec2::ZERO;
                }
            }
        }
        if next != RoomStep::Dormant {
            locks.rooms.insert(level_id.0.clone(), next);
        }
    }
}
