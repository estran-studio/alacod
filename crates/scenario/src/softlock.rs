//! Diagnostic lu entre deux updates, hors simulation et hors checksum GGRS.
//! Un plafond n'est pas en lui-même une preuve de softlock : les deux instantanés
//! permettent de distinguer une partie encore active d'un blocage persistant.

use bevy::prelude::*;
use bevy_fixed::fixed_math::FixedTransform3D;
use bots::navigation::{walls_clear, Rect};
use combat::{downed::Downed, inventory::AmmoReserves};
use game::{
    character::{
        enemy::{
            ai::{
                navigation::{AgentBody, MOVEMENT_FLOW_PROFILE},
                EnemyAiConfig, EnemyTarget, FlowFieldCache, GridPos, MonsterState,
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
        let mut observations = vec![format!(
            "plafond atteint : phase {:?}, {} ennemis de vague présents, {} encore à générer",
            final_state.wave.phase, remaining, final_state.wave.enemies_to_spawn
        )];
        // D42 : le chemin se lit dans le champ que le déplacement suit réellement
        // (`MOVEMENT_FLOW_PROFILE`), pas dans celui du profil déclaré par l'ennemi (D38).
        if final_state.enemies.is_empty() {
            // Rien à diagnostiquer côté chemins.
        } else if final_state.flow_field.is_none() {
            observations.push(format!(
                "chemins non calculés : aucun champ de flux {MOVEMENT_FLOW_PROFILE:?} (celui que suit le déplacement)"
            ));
        } else {
            let uncovered = final_state
                .enemies
                .iter()
                .filter(|e| e.path_cost.is_none())
                .count();
            observations.push(format!(
                "{uncovered} ennemis hors du champ de flux {MOVEMENT_FLOW_PROFILE:?} depuis leur case exacte"
            ));
        }
        for enemy in &final_state.enemies {
            observations.push(format!(
                "restant : {} #{} case ({}, {}), {} PV, {}",
                enemy.character,
                enemy.net_id,
                enemy.position.cell.0,
                enemy.position.cell.1,
                enemy.health,
                match &enemy.nearest_player {
                    Some(p) => format!(
                        "joueur {} le plus proche à {} ({})",
                        p.handle,
                        p.distance,
                        if p.line_of_sight {
                            "en vue"
                        } else {
                            "hors de vue"
                        }
                    ),
                    None => "aucun joueur vivant".to_string(),
                }
            ));
        }
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
    /// Champ de flux lu pour les chemins (D42) : celui que suit le déplacement
    /// (`MOVEMENT_FLOW_PROFILE`) ; `None` : non calculé (aucun champ construit), les
    /// `path_cost` absents ne disent alors rien.
    pub flow_field: Option<String>,
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
    /// Nom du personnage (`GgrsNetId`, ex. `tourelle`).
    pub character: String,
    pub position: Position,
    pub health: String,
    pub wave_enemy: bool,
    pub state: Option<MonsterState>,
    pub target: Option<EnemyTarget>,
    /// Profil déclaré par l'ennemi ; le chemin, lui, se lit dans le champ suivi par le
    /// déplacement (`Snapshot::flow_field`, D38/D42).
    pub nav_profile: Option<String>,
    /// Coût depuis la case exacte dans le champ suivi par le déplacement ; absent si la case
    /// n'est pas couverte (ou si aucun champ n'existe : voir `Snapshot::flow_field`).
    pub path_cost: Option<u32>,
    /// Joueur vivant le plus proche (distance euclidienne), avec la ligne de vue.
    pub nearest_player: Option<NearestPlayer>,
    pub next_cell: Option<(i32, i32)>,
    pub steering_point: Option<Position>,
    /// Murs qui chevaucheraient le corps au point visé par le flow field.
    /// La direction finale peut aussi inclure évitement et séparation.
    pub steering_point_blockers: Vec<WallSnapshot>,
}

/// Joueur vivant le plus proche d'un ennemi restant (D42).
#[derive(Debug, Serialize)]
pub struct NearestPlayer {
    pub handle: usize,
    pub net_id: usize,
    /// Distance entre les centres, en unités monde.
    pub distance: String,
    /// Ligne fine sans `Wall` entre les centres (même test que les bots :
    /// `bots::navigation::walls_clear`) ; ne tient pas compte de la taille des projectiles.
    pub line_of_sight: bool,
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
    let geometry: Vec<(Rect, bool)> = walls
        .iter()
        .map(|(_, t, c)| (Rect::collider(t.translation.truncate(), c), true))
        .collect();
    let mut alive_players: Vec<(usize, usize, bevy_fixed::fixed_math::FixedVec2)> = world
        .query_filtered::<(&GgrsNetId, &Player, &FixedTransform3D, &Health), Without<Downed>>()
        .iter(world)
        .filter(|(_, _, _, health)| health.current > bevy_fixed::fixed_math::FIXED_ZERO)
        .map(|(id, player, t, _)| (player.handle, id.0, t.translation.truncate()))
        .collect();
    alive_players.sort_by_key(|(handle, ..)| *handle);
    let flow_field = world
        .resource::<FlowFieldCache>()
        .get_flow_field(MOVEMENT_FLOW_PROFILE)
        .map(|_| format!("{MOVEMENT_FLOW_PROFILE:?}"));
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
            let field = cache.get_flow_field(MOVEMENT_FLOW_PROFILE);
            let collider = world.get::<Collider>(entity);
            let steering = collider.and_then(|c| {
                field?.get_direction(cell).map(|next| {
                    cache.steering_point(next, MOVEMENT_FLOW_PROFILE, &AgentBody::from_collider(c))
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
            let here = t.translation.truncate();
            let nearest_player = alive_players
                .iter()
                .map(|(handle, net_id, p)| (here.distance(p), *handle, *net_id, *p))
                .min_by_key(|(distance, handle, ..)| (*distance, *handle))
                .map(|(distance, handle, net_id, p)| NearestPlayer {
                    handle,
                    net_id,
                    distance: distance.to_string(),
                    line_of_sight: walls_clear(&geometry, here, p),
                });
            EnemySnapshot {
                net_id: id.0,
                character: id.1.clone(),
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
                nearest_player,
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
        flow_field,
        players,
        closed_doors,
        windows,
        navigation: crate::nav_debug::nav_ascii(world, true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enemy(path_cost: Option<u32>, nearest_player: Option<NearestPlayer>) -> EnemySnapshot {
        EnemySnapshot {
            net_id: 544,
            character: "tourelle".into(),
            position: Position {
                x: "920".into(),
                y: "648".into(),
                cell: (57, 40),
            },
            health: "1".into(),
            wave_enemy: false,
            state: None,
            target: None,
            nav_profile: Some("Ground".into()),
            path_cost,
            nearest_player,
            next_cell: None,
            steering_point: None,
            steering_point_blockers: vec![],
        }
    }

    fn state(flow_field: Option<&str>, enemies: Vec<EnemySnapshot>) -> Snapshot {
        Snapshot {
            frame: 4569,
            wave: WaveState::default(),
            run_step: None,
            enemies,
            flow_field: flow_field.map(str::to_string),
            players: vec![],
            closed_doors: vec![],
            windows: vec![],
            navigation: String::new(),
        }
    }

    #[test]
    fn chemins_non_calcules_sans_champ() {
        let dump = SoftlockDump::new(None, state(None, vec![enemy(None, None)]));
        assert!(
            dump.observations
                .iter()
                .any(|o| o.starts_with("chemins non calculés")),
            "{:?}",
            dump.observations
        );
        assert!(!dump
            .observations
            .iter()
            .any(|o| o.contains("hors du champ")));
    }

    #[test]
    fn hors_champ_compte_dans_le_champ_suivi_et_releve_par_ennemi() {
        let seen = NearestPlayer {
            handle: 1,
            net_id: 78,
            distance: "176".into(),
            line_of_sight: false,
        };
        let dump = SoftlockDump::new(
            None,
            state(
                Some("GroundBreaker"),
                vec![enemy(Some(30), Some(seen)), enemy(None, None)],
            ),
        );
        let obs = &dump.observations;
        assert!(
            obs.contains(
                &"1 ennemis hors du champ de flux GroundBreaker depuis leur case exacte"
                    .to_string()
            ),
            "{obs:?}"
        );
        assert!(obs.contains(&"restant : tourelle #544 case (57, 40), 1 PV, joueur 1 le plus proche à 176 (hors de vue)".to_string()), "{obs:?}");
        assert!(
            obs.contains(
                &"restant : tourelle #544 case (57, 40), 1 PV, aucun joueur vivant".to_string()
            ),
            "{obs:?}"
        );
    }
}
