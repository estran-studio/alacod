//! Salles typées et verrouillées (M2-E1, voie B « un étage = un monde assemblé »,
//! `docs/conventions.md` §35). Les contrats (types, état rollback neutre, géométrie pure) sont
//! dans `world::rooms` ; ce module est le branchement ECS qui a besoin des portes et des
//! bornes de salle du chargement LDtk.
//!
//! - Le type de salle vient du champ de niveau LDtk `room_kind` (recopié du gabarit par
//!   l'assembleur), posé en [`world::RoomKind`] sur l'entité du niveau, à côté de `RoomBounds`.
//!   Sur une carte sans `room_kind`, aucun système de ce module ne fait quoi que ce soit.
//! - Au premier tick d'une carte typée, les portes posées sur le bord d'une salle typée
//!   s'ouvrent (une porte de salle est ouverte par défaut) ; [`DoorShape`] garde de quoi les
//!   refermer.
//! - Une salle dont le type verrouille (`RoomKindDef::locks`) passe `Dormant → Locked` quand
//!   un joueur debout est à l'intérieur et qu'un ennemi y vit : ses portes (et les portes
//!   jumelles, posées au bord de la salle voisine) se referment, les joueurs restés dehors
//!   sont téléportés à l'entrée. Puis `Locked → Cleared` quand plus aucun ennemi n'y vit : les
//!   portes se rouvrent. Une porte n'est jamais refermée sur un occupant : le verrouillage est
//!   différé tant qu'un joueur ou un ennemi chevauche l'une d'elles.
//! - Tant qu'une salle verrouillante dort, ses ennemis portent [`world::RoomDormant`] : sélection
//!   de cible, déplacement et attaque sont sautés (`game::character::enemy::ai`).
//!
//! Les colliders des portes sont déjà rollback (`Collider`, `CollisionLayer`) ; le champ de flux
//! suit tout seul (une porte avec collider est un mur, `rebuild_blocked_cells`).

use bevy::prelude::*;
use bevy_fixed::fixed_math::{Fixed, FixedVec2};
use bevy_ggrs::GgrsSchedule;
use sim_core::frame_events::FrameEvents;
use sim_core::system_set::RollbackSystemSet;
use utils::frame::FrameCount;
use utils::net_id::GgrsNetId;
use utils::order_iter;
use world::{
    entrance_point, inside_box, RoomChanged, RoomDormant, RoomKind, RoomKindTable, RoomState,
    RoomStates,
};

use combat::actors::Velocity;
use combat::downed::Downed;
use game::character::enemy::Enemy;
use game::character::player::Player;
use game::collider::{is_colliding, Collider, CollisionLayer};
use game::interaction::Interactable;
use map::game::entity::map::door::DoorComponent;
use map::game::entity::map::level_id::LevelId;
use map::game::entity::map::room::RoomBounds;

/// Collider d'une porte fermée, pour la refermer (statique, hors rollback : la même valeur
/// pour toute la partie).
#[derive(Component, Clone, Debug)]
pub struct DoorShape {
    pub collider: Collider,
    pub layer: CollisionLayer,
}

pub struct RoomsPlugin;

impl Plugin for RoomsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            GgrsSchedule,
            (
                // Avant l'IA : les ennemis d'une salle dormante sautent leur tour.
                room_dormancy_system.in_set(RollbackSystemSet::EnemySpawning),
                room_lock_system
                    .after(super::floors::floor_transition_system)
                    .in_set(RollbackSystemSet::Run),
            ),
        );
    }
}

/// Une porte appartient à une salle si elle est à moins de `DOOR_MARGIN` de ses bords (la porte
/// de la salle et sa jumelle, posée au bord de la salle voisine).
const DOOR_MARGIN: i32 = 24;
/// Un joueur est « dans » la salle s'il est à plus de `PLAYER_INSET` des bords (passé la porte).
const PLAYER_INSET: i32 = 24;
/// Profondeur (dans la salle) du point où l'on téléporte les joueurs absents.
const ENTRANCE_DEPTH: i32 = 40;
/// Écart entre deux joueurs téléportés au même point d'entrée.
const ENTRANCE_SPACING: i32 = 14;

/// Salles typées triées par position (x, y) puis identifiant : ordre stable, indépendant des
/// `Entity`.
fn sorted_rooms<'a>(
    rooms: &'a Query<(&LevelId, &RoomBounds, &RoomKind)>,
) -> Vec<(&'a LevelId, &'a RoomBounds, &'a RoomKind)> {
    let mut typed: Vec<_> = rooms.iter().collect();
    typed.sort_by(|a, b| {
        (a.1.position.x, a.1.position.y, &a.0 .0).cmp(&(b.1.position.x, b.1.position.y, &b.0 .0))
    });
    typed
}

fn door_near(bounds: &RoomBounds, p: FixedVec2) -> bool {
    inside_box(
        bounds.position,
        bounds.size,
        p,
        -Fixed::from_num(DOOR_MARGIN),
    )
}

/// Pose [`RoomDormant`] sur les ennemis qui vivent dans une salle verrouillante encore
/// dormante, le retire des autres. Ne fait rien sur une carte sans salle typée.
#[allow(clippy::type_complexity)]
pub fn room_dormancy_system(
    mut commands: Commands,
    states: Res<RoomStates>,
    table: Res<RoomKindTable>,
    rooms: Query<(&LevelId, &RoomBounds, &RoomKind)>,
    enemies: Query<
        (
            &GgrsNetId,
            Entity,
            &bevy_fixed::fixed_math::FixedTransform3D,
            Has<RoomDormant>,
        ),
        With<Enemy>,
    >,
) {
    let dormant: Vec<&RoomBounds> = rooms
        .iter()
        .filter(|(id, _, kind)| table.locks(&kind.0) && states.get(&id.0) == RoomState::Dormant)
        .map(|(_, bounds, _)| bounds)
        .collect();
    if rooms.is_empty() {
        return;
    }
    for (_, entity, transform, is_dormant) in order_iter!(enemies) {
        let p = transform.translation.truncate();
        let want = dormant
            .iter()
            .any(|bounds| inside_box(bounds.position, bounds.size, p, Fixed::ZERO));
        if want && !is_dormant {
            commands.entity(entity).insert(RoomDormant);
        } else if !want && is_dormant {
            commands.entity(entity).remove::<RoomDormant>();
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn room_lock_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    table: Res<RoomKindTable>,
    mut states: ResMut<RoomStates>,
    mut changed: ResMut<FrameEvents<RoomChanged>>,
    rooms: Query<(&LevelId, &RoomBounds, &RoomKind)>,
    doors: Query<
        (
            &GgrsNetId,
            Entity,
            &bevy_fixed::fixed_math::FixedTransform3D,
            &DoorShape,
            Has<Collider>,
        ),
        (With<DoorComponent>, Without<Player>, Without<Enemy>),
    >,
    enemies: Query<
        (
            &GgrsNetId,
            &bevy_fixed::fixed_math::FixedTransform3D,
            Option<&Collider>,
        ),
        (With<Enemy>, Without<Player>),
    >,
    mut players: Query<
        (
            &GgrsNetId,
            &mut bevy_fixed::fixed_math::FixedTransform3D,
            Option<&Collider>,
            Option<&mut Velocity>,
            Has<Downed>,
        ),
        (With<Player>, Without<Enemy>),
    >,
) {
    let typed = sorted_rooms(&rooms);
    if typed.is_empty() {
        return;
    }

    // Portes de salle ouvertes par défaut (une fois).
    if !states.doors_opened {
        for (_, entity, t, _, closed) in order_iter!(doors) {
            let p = t.translation.truncate();
            if closed && typed.iter().any(|(_, bounds, _)| door_near(bounds, p)) {
                commands
                    .entity(entity)
                    .remove::<(Collider, CollisionLayer, Interactable)>();
            }
        }
        states.doors_opened = true;
    }

    let inset = Fixed::from_num(PLAYER_INSET);
    for (index, (level_id, bounds, kind)) in typed.iter().enumerate() {
        if !table.locks(&kind.0) {
            continue;
        }
        let step = states.get(&level_id.0);
        let enemies_inside = enemies
            .iter()
            .filter(|(_, t, _)| {
                inside_box(
                    bounds.position,
                    bounds.size,
                    t.translation.truncate(),
                    Fixed::ZERO,
                )
            })
            .count();
        match step {
            RoomState::Dormant if enemies_inside > 0 => {
                // Joueurs debout dans la salle, par `GgrsNetId` croissant : le premier est celui
                // dont l'entrée verrouille (porte d'entrée = la sienne la plus proche).
                let mut standing: Vec<(usize, FixedVec2)> = players
                    .iter()
                    .filter(|(_, t, _, _, downed)| {
                        !*downed
                            && inside_box(
                                bounds.position,
                                bounds.size,
                                t.translation.truncate(),
                                inset,
                            )
                    })
                    .map(|(id, t, ..)| (id.0, t.translation.truncate()))
                    .collect();
                if standing.is_empty() {
                    continue;
                }
                standing.sort_by_key(|(id, _)| *id);
                let trigger = standing[0].1;

                // Portes à refermer (ouvertes, au bord de la salle), par `GgrsNetId`.
                let to_close: Vec<_> = order_iter!(doors)
                    .into_iter()
                    .filter(|(_, _, t, _, closed)| {
                        !*closed && door_near(bounds, t.translation.truncate())
                    })
                    .collect();

                // Jamais sur un occupant : un joueur ou un ennemi qui chevauche une porte
                // retarde le verrouillage (réévalué à la frame suivante).
                let occupied = to_close.iter().any(|(_, _, door_t, shape, _)| {
                    let overlaps =
                        |t: &bevy_fixed::fixed_math::FixedTransform3D,
                         collider: Option<&Collider>| {
                            collider.is_some_and(|c| {
                                is_colliding(
                                    &door_t.translation,
                                    &shape.collider,
                                    &t.translation,
                                    c,
                                )
                            })
                        };
                    enemies.iter().any(|(_, t, c)| overlaps(t, c))
                        || players.iter().any(|(_, t, c, _, _)| overlaps(t, c))
                });
                if occupied {
                    trace!(
                        "ggrs{{f={} room_lock_deferred room={} kind={}}}",
                        frame.frame,
                        index,
                        kind.0
                    );
                    continue;
                }

                for (_, entity, _, shape, _) in &to_close {
                    commands
                        .entity(*entity)
                        .insert((shape.collider.clone(), shape.layer.clone()));
                }

                // Coop : les joueurs absents sont téléportés à l'entrée (la porte la plus
                // proche du joueur qui a verrouillé), côte à côte.
                let entrance = to_close
                    .iter()
                    .map(|(id, _, t, _, _)| {
                        let p = t.translation.truncate();
                        (p.distance_squared(&trigger), id.0, p)
                    })
                    .min_by_key(|(distance, id, _)| (*distance, *id))
                    .map(|(_, _, p)| {
                        entrance_point(
                            bounds.position,
                            bounds.size,
                            p,
                            Fixed::from_num(ENTRANCE_DEPTH),
                        )
                    });
                if let Some((entrance, tangent)) = entrance {
                    let mut ordered: Vec<_> = players.iter_mut().collect();
                    ordered.sort_by_key(|(id, ..)| id.0);
                    for (k, (id, mut t, _, velocity, _)) in ordered
                        .into_iter()
                        .filter(|(_, t, ..)| {
                            !inside_box(
                                bounds.position,
                                bounds.size,
                                t.translation.truncate(),
                                Fixed::ZERO,
                            )
                        })
                        .enumerate()
                    {
                        let spread = Fixed::from_num(ENTRANCE_SPACING * k as i32);
                        t.translation.x = entrance.x + tangent.x * spread;
                        t.translation.y = entrance.y + tangent.y * spread;
                        if let Some(mut velocity) = velocity {
                            velocity.main = FixedVec2::ZERO;
                            velocity.knockback = FixedVec2::ZERO;
                        }
                        info!(
                            "ggrs{{f={} room_teleport room={} net_id={}}}",
                            frame.frame, index, id.0
                        );
                    }
                }

                states.rooms.insert(level_id.0.clone(), RoomState::Locked);
                changed.send(RoomChanged {
                    frame: frame.frame,
                    room: level_id.0.clone(),
                    kind: kind.0.clone(),
                    from: RoomState::Dormant,
                    to: RoomState::Locked,
                });
                info!(
                    "ggrs{{f={} room_state room={} kind={} state=Locked enemies={}}}",
                    frame.frame, index, kind.0, enemies_inside
                );
            }
            RoomState::Locked if enemies_inside == 0 => {
                for (_, entity, t, _, closed) in order_iter!(doors) {
                    if closed && door_near(bounds, t.translation.truncate()) {
                        commands
                            .entity(entity)
                            .remove::<(Collider, CollisionLayer)>();
                    }
                }
                states.rooms.insert(level_id.0.clone(), RoomState::Cleared);
                changed.send(RoomChanged {
                    frame: frame.frame,
                    room: level_id.0.clone(),
                    kind: kind.0.clone(),
                    from: RoomState::Locked,
                    to: RoomState::Cleared,
                });
                info!(
                    "ggrs{{f={} room_state room={} kind={} state=Cleared enemies=0}}",
                    frame.frame, index, kind.0
                );
            }
            _ => {}
        }
    }
}
