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

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_fixed::fixed_math::{Fixed, FixedVec2, FixedVec3};
use bevy_fixed::rng::RngStreams;
use bevy_ggrs::GgrsSchedule;
use sim_core::frame_events::FrameEvents;
use sim_core::system_set::RollbackSystemSet;
use utils::frame::FrameCount;
use utils::net_id::GgrsNetId;
use utils::order_iter;
use world::{
    entrance_point, inside_box, room_rng_stream, ArenaClocks, RoomChanged, RoomDormant, RoomKind,
    RoomKindTable, RoomState, RoomStates,
};

use combat::actors::Velocity;
use combat::downed::Downed;
use game::boss::ReinforcementRequest;
use game::character::config::CharacterConfig;
use game::character::enemy::ai::navigation::{FlowFieldCache, GridPos};
use game::character::enemy::create::spawn_enemy;
use game::character::enemy::Enemy;
use game::character::health::Death;
use game::character::player::Player;
use game::collider::CollisionSettings;
use game::collider::{is_colliding, Collider, CollisionLayer};
use game::global_asset::GlobalAsset;
use game::interaction::Interactable;
use game::weapons::{melee::MeleeWeaponsConfig, WeaponsConfig};
use map::game::entity::map::door::DoorComponent;
use map::game::entity::map::level_id::LevelId;
use map::game::entity::map::room::RoomBounds;
use sim_core::team::Team;
use utils::net_id::GgrsNetIdFactory;

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
                room_dormancy_system
                    .before(game::boss::boss_phase_system)
                    .in_set(RollbackSystemSet::EnemySpawning),
                room_lock_system
                    .after(super::floors::floor_transition_system)
                    .in_set(RollbackSystemSet::Run),
                // M2-T7 : vagues internes d'une arène, après son verrouillage de la frame.
                arena_waves_system
                    .after(room_lock_system)
                    .in_set(RollbackSystemSet::Run),
                // M2-T7 : renforts d'une frise de boss, dans la frame de la demande (`Run` suit
                // `EnemySpawning`, où `boss_phase_system` les demande).
                boss_reinforcements_system
                    .after(arena_waves_system)
                    .in_set(RollbackSystemSet::Run),
                // M2-T7 : la mort d'un boss vide sa salle (despawn dans la même frame).
                boss_room_clear_system
                    .after(game::boss::boss_defeated_system)
                    .before(game::character::health::rollback_apply_death)
                    .in_set(RollbackSystemSet::DeathManagement),
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

/// Salles typées de l'étage courant. En mode `Floors`, chaque emplacement est un monde LDtk
/// chargé dès le départ (M2-T10 : deux étages assemblés se superposent dans l'espace) : seules
/// les salles de l'emplacement de l'étage en cours comptent, sinon les salles d'un autre étage
/// se verrouilleraient et se videraient en même temps que celles où se tient le joueur.
#[derive(SystemParam)]
pub struct CurrentRooms<'w, 's> {
    rooms: Query<
        'w,
        's,
        (
            Entity,
            &'static LevelId,
            &'static RoomBounds,
            &'static RoomKind,
        ),
    >,
    slots: super::floors::FloorSlots<'w, 's>,
    plan: Option<Res<'w, super::floors::FloorPlan>>,
    state: Option<Res<'w, run::FloorState>>,
}

impl CurrentRooms<'_, '_> {
    fn iter(&self) -> impl Iterator<Item = (&LevelId, &RoomBounds, &RoomKind)> {
        let current = self
            .plan
            .as_ref()
            .zip(self.state.as_ref())
            .map(|(plan, state)| plan.slot_for_floor(state.index));
        self.rooms
            .iter()
            .filter(move |(entity, ..)| {
                current.is_none_or(|slot| self.slots.slot_of(*entity) == slot)
            })
            .map(|(_, id, bounds, kind)| (id, bounds, kind))
    }
}

/// Salles typées triées par position (x, y) puis identifiant : ordre stable, indépendant des
/// `Entity`.
fn sorted_rooms<'a>(rooms: &'a CurrentRooms) -> Vec<(&'a LevelId, &'a RoomBounds, &'a RoomKind)> {
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
    rooms: CurrentRooms,
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
    if rooms.iter().next().is_none() {
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
    mut arena_clocks: ResMut<ArenaClocks>,
    mut changed: ResMut<FrameEvents<RoomChanged>>,
    rooms: CurrentRooms,
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
            Entity,
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
        // M2-T7 : une arène se verrouille vide et ne se rouvre qu'à la fin de son horloge.
        let arena = table.arena(&kind.0);
        let enemies_inside = enemies
            .iter()
            .filter(|(_, t, _, _)| {
                inside_box(
                    bounds.position,
                    bounds.size,
                    t.translation.truncate(),
                    Fixed::ZERO,
                )
            })
            .count();
        match step {
            RoomState::Dormant if enemies_inside > 0 || arena.is_some() => {
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
                    enemies.iter().any(|(_, t, c, _)| overlaps(t, c))
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
                if arena.is_some() {
                    arena_clocks.0.insert(level_id.0.clone(), frame.frame);
                }
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
            RoomState::Locked if arena.is_some() => {
                // Fin de l'horloge de l'arène, un joueur en vie : les survivants meurent, la
                // salle se rouvre.
                let Some(arena) = arena else { continue };
                let started = arena_clocks
                    .0
                    .get(&level_id.0)
                    .copied()
                    .unwrap_or(frame.frame);
                if frame.frame.saturating_sub(started) < arena.clock || players.is_empty() {
                    continue;
                }
                for (id, t, _, entity) in order_iter!(enemies) {
                    if inside_box(
                        bounds.position,
                        bounds.size,
                        t.translation.truncate(),
                        Fixed::ZERO,
                    ) {
                        info!(
                            "ggrs{{f={} arena_kill room={} net_id={}}}",
                            frame.frame, index, id.0
                        );
                        commands.entity(entity).insert(Death { last_hit_by: None });
                    }
                }
                for (_, entity, t, _, closed) in order_iter!(doors) {
                    if closed && door_near(bounds, t.translation.truncate()) {
                        commands
                            .entity(entity)
                            .remove::<(Collider, CollisionLayer)>();
                    }
                }
                arena_clocks.0.remove(&level_id.0);
                states.rooms.insert(level_id.0.clone(), RoomState::Cleared);
                changed.send(RoomChanged {
                    frame: frame.frame,
                    room: level_id.0.clone(),
                    kind: kind.0.clone(),
                    from: RoomState::Locked,
                    to: RoomState::Cleared,
                });
                info!(
                    "ggrs{{f={} room_state room={} kind={} state=Cleared arena=held}}",
                    frame.frame, index, kind.0
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

/// Paramètres d'apparition d'un personnage en cours de partie (renforts de boss, vagues
/// d'arène, M2-T7).
#[derive(SystemParam)]
pub struct SpawnParams<'w> {
    global_assets: Res<'w, GlobalAsset>,
    collision_settings: Res<'w, CollisionSettings>,
    weapons_asset: Res<'w, Assets<WeaponsConfig>>,
    melee_weapons_asset: Res<'w, Assets<MeleeWeaponsConfig>>,
    characters_asset: Res<'w, Assets<CharacterConfig>>,
    balance: Res<'w, game::balance::ResolvedBalance>,
    difficulty: game::clock::DifficultyReader<'w>,
    id_factory: ResMut<'w, GgrsNetIdFactory>,
    rng: ResMut<'w, RngStreams>,
    flow: Res<'w, FlowFieldCache>,
}

/// Marge (en pixels) gardée entre une apparition et les bords de la salle.
const SPAWN_INSET: i32 = 40;
/// Tirages de position avant de se rabattre sur le centre de la salle.
const SPAWN_TRIES: u32 = 16;

impl SpawnParams<'_> {
    /// Fait apparaître `count` personnages `id` dans la salle `level` : positions tirées du flux
    /// RNG `room:<niveau>` (case libre pour le profil `Ground`), repli sur le centre.
    fn spawn_in_room(
        &mut self,
        commands: &mut Commands,
        level: &str,
        bounds: &RoomBounds,
        id: &str,
        count: u32,
    ) {
        let Some(health_max) = self.balance.health_max_by_character.get(id).copied() else {
            warn!("renforts : personnage « {id} » sans santé résolue (voir `alacod lint`), ignoré");
            return;
        };
        let health_max = game::clock::scale(health_max, self.difficulty.current());
        let inset = Fixed::from_num(SPAWN_INSET);
        let width = (bounds.size.x - inset * 2).to_num::<i32>().max(1) as u32;
        let height = (bounds.size.y - inset * 2).to_num::<i32>().max(1) as u32;
        let stream = room_rng_stream(level);
        for _ in 0..count {
            let mut position = FixedVec2::new(
                bounds.position.x + bounds.size.x / 2,
                bounds.position.y + bounds.size.y / 2,
            );
            for _ in 0..SPAWN_TRIES {
                let rng = self.rng.get_mut(&stream);
                let x = bounds.position.x + inset + Fixed::from_num(rng.next_u32_range(0, width));
                let y = bounds.position.y + inset + Fixed::from_num(rng.next_u32_range(0, height));
                let candidate = FixedVec2::new(x, y);
                if !self.flow.is_blocked(
                    &GridPos::from_fixed(candidate),
                    game::character::enemy::ai::navigation::NavProfile::Ground,
                ) {
                    position = candidate;
                    break;
                }
            }
            spawn_enemy(
                id.to_string(),
                FixedVec3::new(position.x, position.y, Fixed::ZERO),
                commands,
                &self.weapons_asset,
                &self.melee_weapons_asset,
                &self.characters_asset,
                &self.global_assets,
                &self.collision_settings,
                &mut self.id_factory,
                Team::Enemies,
                health_max,
                self.rng.run_seed,
                None,
            );
        }
    }
}

/// Renforts d'une frise de boss (`SpawnEnemies`, M2-T7) : exécute les demandes de la frame dans
/// la salle qui contient le boss, dans l'ordre d'émission (`GgrsNetId` croissant du boss).
pub fn boss_reinforcements_system(
    mut commands: Commands,
    mut requests: ResMut<FrameEvents<ReinforcementRequest>>,
    rooms: CurrentRooms,
    mut spawn: SpawnParams,
) {
    let requests: Vec<ReinforcementRequest> = requests.iter().cloned().collect();
    for request in requests {
        let at = FixedVec2::new(request.x, request.y);
        let Some((level, bounds, _)) = sorted_rooms(&rooms)
            .into_iter()
            .find(|(_, bounds, _)| inside_box(bounds.position, bounds.size, at, Fixed::ZERO))
        else {
            warn!(
                "renforts : le boss {} n'est dans aucune salle, {} « {} » ignoré",
                request.boss, request.count, request.id
            );
            continue;
        };
        spawn.spawn_in_room(&mut commands, &level.0, bounds, &request.id, request.count);
    }
}

/// Vagues internes d'une arène verrouillée (M2-T7) : à `at` frames du verrouillage, les
/// personnages de la vague apparaissent. Aucun curseur : la vague due se déduit de l'horloge.
pub fn arena_waves_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    table: Res<RoomKindTable>,
    states: Res<RoomStates>,
    clocks: Res<ArenaClocks>,
    rooms: CurrentRooms,
    mut spawn: SpawnParams,
) {
    for (level, bounds, kind) in sorted_rooms(&rooms) {
        let Some(arena) = table.arena(&kind.0) else {
            continue;
        };
        if states.get(&level.0) != RoomState::Locked {
            continue;
        }
        let Some(started) = clocks.0.get(&level.0) else {
            continue;
        };
        let elapsed = frame.frame.saturating_sub(*started);
        for wave in arena.waves.iter().filter(|wave| wave.at == elapsed) {
            for (id, count) in &wave.enemies {
                info!(
                    "ggrs{{f={} arena_wave room={} at={} id={} count={}}}",
                    frame.frame, level.0, wave.at, id, count
                );
                spawn.spawn_in_room(&mut commands, &level.0, bounds, id, *count);
            }
        }
    }
}

/// Mort d'un boss (M2-T7) : les autres ennemis de sa salle meurent (renforts compris), pour que
/// la salle passe `Cleared` dans la frame (`room_lock_system`) et que les portes se rouvrent.
#[allow(clippy::type_complexity)]
pub fn boss_room_clear_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    defeated: Res<FrameEvents<behaviors::BossDefeated>>,
    rooms: CurrentRooms,
    enemies: Query<
        (
            &GgrsNetId,
            Entity,
            &bevy_fixed::fixed_math::FixedTransform3D,
            Has<Death>,
        ),
        (With<Enemy>, Without<Player>),
    >,
) {
    for event in defeated.iter() {
        let at = FixedVec2::new(event.x, event.y);
        let Some((_, bounds, _)) = sorted_rooms(&rooms)
            .into_iter()
            .find(|(_, bounds, _)| inside_box(bounds.position, bounds.size, at, Fixed::ZERO))
        else {
            continue;
        };
        for (net_id, entity, t, dead) in order_iter!(enemies) {
            if !dead
                && net_id.0 != event.net_id
                && inside_box(
                    bounds.position,
                    bounds.size,
                    t.translation.truncate(),
                    Fixed::ZERO,
                )
            {
                info!(
                    "ggrs{{f={} boss_room_clear net_id={}}}",
                    frame.frame, net_id.0
                );
                commands.entity(entity).insert(Death { last_hit_by: None });
            }
        }
    }
}
