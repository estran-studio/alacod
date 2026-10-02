//! Diagnostic lu entre deux updates, hors simulation et hors checksum GGRS.
//! Un plafond n'est pas en lui-même une preuve de softlock : les deux instantanés
//! permettent de distinguer une partie encore active d'un blocage persistant.

use bevy::prelude::*;
use bevy_fixed::fixed_math::FixedTransform3D;
use combat::{downed::Downed, inventory::AmmoReserves};
use game::{
    character::{
        enemy::{
            ai::{
                navigation::AgentBody, EnemyAiConfig, EnemyTarget, FlowFieldCache, GridPos,
                MonsterState,
            },
            Enemy,
        },
        health::Health,
        player::Player,
    },
    collider::{is_colliding, Collider, Wall},
    waves::{WaveEnemy, WaveState},
    weapons::{WeaponInventory, WeaponModesState, WeaponState},
};
use map::game::entity::map::{door::DoorComponent, window::WindowHealth};
use run::{currency::Currency, Run};
use serde::Serialize;
use utils::{frame::FrameCount, net_id::GgrsNetId};

#[derive(Debug, Serialize)]
pub struct SoftlockDump {
    /// Faits observés, sans attribuer arbitrairement le blocage à une porte.
    pub observations: Vec<String>,
    pub previous: Option<Snapshot>,
    pub final_state: Snapshot,
}

impl SoftlockDump {
    pub fn new(previous: Option<Snapshot>, final_state: Snapshot) -> Self {
        let remaining = final_state.enemies.iter().filter(|e| e.wave_enemy).count();
        let uncovered = final_state
            .enemies
            .iter()
            .filter(|e| e.path_cost.is_none())
            .count();
        let mut observations = vec![
            format!(
                "plafond atteint : phase {:?}, {} ennemis de vague présents, {} encore à générer",
                final_state.wave.phase, remaining, final_state.wave.enemies_to_spawn
            ),
            format!(
                "{uncovered} ennemis sans chemin depuis leur case exacte vers une cible du flow field"
            ),
        ];
        let blocked_points = final_state
            .enemies
            .iter()
            .filter(|e| !e.steering_point_blockers.is_empty())
            .count();
        observations.push(format!("{blocked_points} ennemis dont le point visé par le flow field chevauche un mur physique avec leur collider"));
        if let Some(before) = &previous {
            if before.wave.total_enemies_killed == final_state.wave.total_enemies_killed {
                observations.push(format!(
                    "aucun kill entre les frames {} et {}",
                    before.frame, final_state.frame
                ));
            }
        }
        Self {
            observations,
            previous,
            final_state,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Snapshot {
    pub frame: u32,
    pub wave: WaveState,
    pub run_step: Option<String>,
    pub enemies: Vec<EnemySnapshot>,
    /// Joueurs encore présents, y compris à terre (signalés explicitement).
    pub players: Vec<PlayerSnapshot>,
    pub closed_doors: Vec<DoorSnapshot>,
    pub windows: Vec<WindowSnapshot>,
    pub navigation: String,
}

#[derive(Debug, Serialize)]
pub struct Position {
    pub x: String,
    pub y: String,
    pub cell: (i32, i32),
}

impl Position {
    fn from_transform(t: &FixedTransform3D) -> Self {
        let cell = GridPos::from_fixed(t.translation.truncate());
        Self {
            x: t.translation.x.to_string(),
            y: t.translation.y.to_string(),
            cell: (cell.x, cell.y),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct EnemySnapshot {
    pub net_id: usize,
    pub position: Position,
    pub health: String,
    pub wave_enemy: bool,
    pub state: Option<MonsterState>,
    pub target: Option<EnemyTarget>,
    pub nav_profile: Option<String>,
    /// Coût depuis la case exacte vers une cible courante ; absent si non couverte.
    pub path_cost: Option<u32>,
    pub next_cell: Option<(i32, i32)>,
    pub steering_point: Option<Position>,
    /// Murs qui chevaucheraient le corps au point visé par le flow field.
    /// La direction finale peut aussi inclure évitement et séparation.
    pub steering_point_blockers: Vec<WallSnapshot>,
}

#[derive(Debug, Serialize)]
pub struct WallSnapshot {
    pub net_id: usize,
    /// Bornes du collider, offset compris, en coordonnées monde.
    pub min: (String, String),
    pub max: (String, String),
}

#[derive(Debug, Serialize)]
pub struct PlayerSnapshot {
    pub net_id: usize,
    pub handle: usize,
    pub position: Position,
    pub health: String,
    pub downed: Option<Downed>,
    pub currency: Option<u32>,
    pub active_weapon: Option<String>,
    pub mag_ammo: Option<u32>,
    pub reserves: Vec<(String, u32)>,
}

#[derive(Debug, Serialize)]
pub struct DoorSnapshot {
    pub net_id: usize,
    pub position: Position,
    pub cost: i32,
    pub interactable: bool,
}

#[derive(Debug, Serialize)]
pub struct WindowSnapshot {
    pub net_id: usize,
    pub position: Position,
    pub health: u8,
}

/// Lecture uniquement : ni système ajouté, ni ressource rollback modifiée.
pub fn snapshot(world: &mut World) -> Snapshot {
    let frame = world.resource::<FrameCount>().frame;
    let wave = world.resource::<WaveState>().clone();
    let run_step = world
        .get_resource::<Run>()
        .map(|run| format!("{:?}", run.step));
    let mut walls: Vec<_> = world
        .query_filtered::<(&GgrsNetId, &FixedTransform3D, &Collider), With<Wall>>()
        .iter(world)
        .map(|(id, t, c)| (id.0, t.clone(), c.clone()))
        .collect();
    walls.sort_by_key(|(id, ..)| *id);
    let mut enemies: Vec<_> = world
        .query_filtered::<(
            &GgrsNetId,
            Entity,
            &FixedTransform3D,
            &Health,
            Option<&WaveEnemy>,
            Option<&MonsterState>,
            Option<&EnemyTarget>,
            Option<&EnemyAiConfig>,
        ), With<Enemy>>()
        .iter(world)
        .map(|(id, entity, t, health, wave, state, target, ai)| {
            let cache = world.resource::<FlowFieldCache>();
            let cell = GridPos::from_fixed(t.translation.truncate());
            let field = ai.and_then(|ai| cache.get_flow_field(ai.nav_profile()));
            let collider = world.get::<Collider>(entity);
            let steering = ai.zip(collider).and_then(|(ai, c)| {
                field?.get_direction(cell).map(|next| {
                    cache.steering_point(next, ai.nav_profile(), &AgentBody::from_collider(c))
                })
            });
            let candidate =
                steering.map(|p| bevy_fixed::fixed_math::FixedVec3::new(p.x, p.y, t.translation.z));
            let blockers = candidate
                .zip(collider)
                .map(|(p, c)| {
                    walls
                        .iter()
                        .filter(|(_, wt, wc)| is_colliding(&p, c, &wt.translation, wc))
                        .map(|(id, wt, wc)| {
                            let bounds = game::collision_grid::collider_aabb(&wt.translation, wc);
                            WallSnapshot {
                                net_id: *id,
                                min: (bounds.min.x.to_string(), bounds.min.y.to_string()),
                                max: (bounds.max.x.to_string(), bounds.max.y.to_string()),
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();
            EnemySnapshot {
                net_id: id.0,
                position: Position::from_transform(t),
                health: health.current.to_string(),
                wave_enemy: wave.is_some(),
                state: state.cloned(),
                target: target.cloned(),
                nav_profile: ai.map(|ai| format!("{:?}", ai.nav_profile())),
                path_cost: field.and_then(|f| f.costs.get(&cell).copied()),
                next_cell: field
                    .and_then(|f| f.get_direction(cell))
                    .map(|c| (c.x, c.y)),
                steering_point: candidate.map(|p| {
                    Position::from_transform(&FixedTransform3D {
                        translation: p,
                        ..t.clone()
                    })
                }),
                steering_point_blockers: blockers,
            }
        })
        .collect();
    enemies.sort_by_key(|e| e.net_id);
    let mut players: Vec<_> = world
        .query::<(
            &GgrsNetId,
            &Player,
            &FixedTransform3D,
            &Health,
            Option<&Downed>,
            Option<&Currency>,
            Option<&WeaponInventory>,
            Option<&AmmoReserves>,
        )>()
        .iter(world)
        .map(
            |(id, player, t, health, downed, currency, inventory, reserves)| {
                let active = inventory.and_then(|i| i.weapons.get(i.active_weapon_index));
                let ammo = active.and_then(|(entity, _)| {
                    let state = world.get::<WeaponState>(*entity)?;
                    world
                        .get::<WeaponModesState>(*entity)?
                        .modes
                        .get(&state.active_mode)
                        .map(|m| m.mag_ammo)
                });
                PlayerSnapshot {
                    net_id: id.0,
                    handle: player.handle,
                    position: Position::from_transform(t),
                    health: health.current.to_string(),
                    downed: downed.cloned(),
                    currency: currency.map(|c| c.0),
                    active_weapon: active.map(|(_, w)| w.config.name.clone()),
                    mag_ammo: ammo,
                    reserves: reserves
                        .map(|r| {
                            r.0.iter()
                                .map(|(kind, n)| (format!("{kind:?}"), *n))
                                .collect()
                        })
                        .unwrap_or_default(),
                }
            },
        )
        .collect();
    players.sort_by_key(|p| p.net_id);
    let mut closed_doors: Vec<_> = world
        .query_filtered::<(&GgrsNetId, &FixedTransform3D, &DoorComponent), With<Collider>>()
        .iter(world)
        .map(|(id, t, door)| DoorSnapshot {
            net_id: id.0,
            position: Position::from_transform(t),
            cost: door.config.cost,
            interactable: door.config.interactable,
        })
        .collect();
    closed_doors.sort_by_key(|d| d.net_id);
    let mut windows: Vec<_> = world
        .query::<(&GgrsNetId, &FixedTransform3D, &WindowHealth)>()
        .iter(world)
        .map(|(id, t, health)| WindowSnapshot {
            net_id: id.0,
            position: Position::from_transform(t),
            health: health.current,
        })
        .collect();
    windows.sort_by_key(|w| w.net_id);
    Snapshot {
        frame,
        wave,
        run_step,
        enemies,
        players,
        closed_doors,
        windows,
        navigation: crate::nav_debug::nav_ascii(world, true),
    }
}
