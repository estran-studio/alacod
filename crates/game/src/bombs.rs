//! Bombes, explosions, rochers cassés et piques (M3-T4, `docs/conventions.md` §69).
//!
//! Une explosion a trois sources, une seule règle d'exécution :
//! - une **bombe** posée par un joueur (`INPUT_BOMB`, front montant, un consommable `bomb`) :
//!   [`bomb_place_system`] la fait naître aux pieds du poseur ; [`bomb_fuse_system`] la consomme
//!   quand sa mèche est écoulée ;
//! - l'action **`Explode`** d'un effet (`effects_runtime::apply_action_sourced`) : elle pose une
//!   bombe à mèche d'une frame au point du porteur (même chemin, même ordre) ;
//! - un **baril** mort à la frame précédente ([`crate::props`] : sa mort est lue ici).
//!
//! Deux étages dans la frame :
//! 1. `World` ([`bomb_fuse_system`], [`explosion_terrain_system`], avant
//!    `world::plugin::apply_destroy_terrain_system`) : les bombes dues émettent leur
//!    `world::ExplosionRequest` ; celles qui `destroys` demandent la destruction du terrain de
//!    caverne (`DestroyTerrainRequest`, reconstruction des murs dans la même frame) et cassent les
//!    **rochers** du rayon (`despawn_rollback`, ordre `(y, x)` des cases, butin tiré par
//!    `loot_seed` de la clé [`rock_loot_key`]). Le champ de flux relit les obstacles à chaque
//!    mise à jour : un rocher disparu rouvre le passage sans autre code.
//! 2. `CollisionDamage` ([`explosion_system`], même emplacement que l'ancien système des barils,
//!    avant `rollback_resolve_damage_events`) : les demandes de la frame (bombes, plus les barils
//!    morts) sont triées par `GgrsNetId` de l'objet ; chacune envoie `damage` à tous les
//!    personnages à PV à **strictement** moins de `radius` (`world::within_radius`), jamais ceux
//!    qui ont déjà un `Death` ni ceux qui sont invulnérables (`Health::is_invulnerable_at`, comme
//!    les balles : les familiers de M3-T6 sont immunisés ainsi), crédités à la source.
//!
//! Le mur secret (`map_ldtk::game::rooms::secret_wall_system`) lit les mêmes demandes.
//!
//! Piques ([`hazard_system`]) : un joueur dont les pieds sont sur une case de surface à `hazard`
//! perd `damage` (dégâts `True`, tag `spikes`) au plus une fois par `period` frames.
//!
//! Tout est inerte sans porteur : une carte sans bombe, sans rocher, sans piques ne crée aucun
//! composant ni aucun `GgrsNetId` (traces existantes inchangées).

use bevy::prelude::*;
use bevy_fixed::fixed_math::{self, Fixed, FixedVec2};
use bevy_ggrs::{GgrsSchedule, PlayerInputs, Rollback, RollbackDespawnCommandExtension};
use combat::actors::DashState;
use combat::actors::INPUT_BOMB;
use combat::downed::Downed;
use items::Inventory;
use sim_core::damage::{DamageEvent, DamageKind};
use sim_core::frame_events::{FrameEvents, FrameEventsAppExt};
use sim_core::tag::Tags;
use sim_core::team::Team;
use std::collections::{BTreeMap, BTreeSet};
use utils::{
    frame::FrameCount,
    net_id::{GgrsNetId, GgrsNetIdFactory},
    order_iter, order_mut_iter,
};
use world::{
    cells_in_radius, within_radius, Bomb, BombCarrier, CellGrid, DestroyTerrainRequest, Exploded,
    ExplosionRequest, HazardState, Rock, SurfaceGrid, SurfaceTable,
};

use crate::character::health::{Death, Health, HitBy};
use crate::character::player::jjrs::PeerConfig;
use crate::character::player::Player;
use crate::character::surface::feet_position;
use crate::collider::Collider;
use crate::loot::BOMB_ITEM;
use crate::system_set::RollbackSystemSet;

pub struct BombsPlugin;

impl Plugin for BombsPlugin {
    fn build(&self, app: &mut App) {
        app.add_frame_events_transient::<RockBroken>().add_systems(
            GgrsSchedule,
            (
                // Après les autres systèmes d'objets de `Effects` (le blank lit aussi `Inventory`).
                bomb_place_system
                    .after(crate::items::synergy_modifiers_system)
                    .in_set(RollbackSystemSet::Effects),
                // Étage terrain : avant l'application des destructions de caverne.
                (
                    bomb_fuse_system,
                    explosion_terrain_system.after(bomb_fuse_system),
                )
                    .after(crate::props::pit_system)
                    .before(world::plugin::apply_destroy_terrain_system)
                    .in_set(RollbackSystemSet::World),
                // Piques : après la fosse (même ensemble, mêmes `DamageEvent`).
                hazard_system
                    .run_if(has_hazards)
                    .after(crate::props::pit_system)
                    .in_set(RollbackSystemSet::World),
                // Étage dégâts : à la place de l'ancien système des barils (même emplacement).
                explosion_system
                    .after(crate::character::enemy::ai::rules::charge_damage_translate_system)
                    .before(crate::character::health::rollback_resolve_damage_events)
                    .in_set(RollbackSystemSet::CollisionDamage),
            ),
        );
    }
}

/// Clé de contenu du butin d'un rocher : `rock:<salle>:<x>:<y>` (case de grille monde). Jamais son
/// `Entity` ni son id réseau : le même rocher donne le même objet quel que soit le moment.
pub fn rock_loot_key(room: &str, cell: (i32, i32)) -> String {
    format!("rock:{room}:{}:{}", cell.0, cell.1)
}

/// Pose une bombe aux pieds d'un joueur qui porte des bombes, au front montant de `Bomb`, s'il en
/// reste (consommable `bomb`). Mémoire du bouton réécrite avant tout retour anticipé.
#[allow(clippy::type_complexity)]
pub fn bomb_place_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    inputs: Res<PlayerInputs<PeerConfig>>,
    mut id_factory: ResMut<GgrsNetIdFactory>,
    mut players: Query<
        (
            &GgrsNetId,
            &Player,
            &fixed_math::FixedTransform3D,
            Option<&Collider>,
            &mut BombCarrier,
            Option<&mut Inventory>,
            Has<Downed>,
        ),
        With<Rollback>,
    >,
) {
    for (net_id, player, transform, collider, mut carrier, inventory, downed) in
        order_mut_iter!(players)
    {
        let requested = world::bomb_requested(
            inputs[player.handle].0.buttons,
            Some(&carrier.def),
            INPUT_BOMB,
        );
        let rising = requested && !carrier.held;
        if carrier.held != requested {
            carrier.held = requested;
        }
        if !rising || downed {
            continue;
        }
        let Some(mut inventory) = inventory else {
            continue;
        };
        let Some(stock) = inventory
            .consumables
            .get_mut(BOMB_ITEM)
            .filter(|stock| **stock > 0)
        else {
            continue;
        };
        *stock -= 1;
        let feet = collider.map_or(transform.translation, |c| transform.translation + c.offset);
        let position = fixed_math::FixedVec3::new(feet.x, feet.y, Fixed::ZERO);
        let bomb_transform = fixed_math::FixedTransform3D::new(
            position,
            fixed_math::FixedMat3::IDENTITY,
            fixed_math::FixedVec3::ONE,
        );
        info!(
            "ggrs{{f={} bomb_place owner={} at=({},{}) fuse_until={}}}",
            frame.frame,
            net_id.0,
            position.x,
            position.y,
            frame.frame.saturating_add(carrier.def.fuse_frames)
        );
        commands.spawn((
            Bomb::placed(net_id.clone(), frame.frame, &carrier.def),
            bomb_transform.to_bevy_transform(),
            bomb_transform,
            id_factory.next(format!("bomb_{}", net_id.0)),
            Rollback,
        ));
    }
}

/// Consomme les bombes dont la mèche est écoulée : une `ExplosionRequest` chacune (ordre
/// `GgrsNetId`), puis `despawn_rollback`.
pub fn bomb_fuse_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    mut requests: ResMut<FrameEvents<ExplosionRequest>>,
    bombs: Query<(&GgrsNetId, Entity, &Bomb, &fixed_math::FixedTransform3D), With<Rollback>>,
) {
    for (net_id, entity, bomb, transform) in order_iter!(bombs) {
        if !bomb.is_due(frame.frame) {
            continue;
        }
        requests.send(ExplosionRequest {
            id: net_id.0,
            source: bomb.owner.clone(),
            source_team: bomb.team,
            x: transform.translation.x,
            y: transform.translation.y,
            radius: bomb.radius,
            damage: bomb.damage,
            destroys: bomb.destroys,
            friendly_fire: bomb.friendly_fire,
        });
        commands.entity(entity).despawn_rollback();
    }
}

/// Un rocher vient d'être cassé (`FrameEvents` **transitoire**, émis dans l'ordre `(y, x)` des cases) :
/// lu par `map_ldtk::game::rooms::rock_loot_system`, qui tire le butin d'un rocher `loot` dans la
/// salle de l'étage courant (`CurrentRooms` n'est pas visible d'ici). Position en champs séparés.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RockBroken {
    pub cell: (i32, i32),
    pub x: Fixed,
    pub y: Fixed,
    pub loot: bool,
}

/// Salle d'un point : identifiant de la plus petite salle (ordre des identifiants) dont les bornes
/// `(position, taille)` contiennent `point`, `-` si aucune. `rooms` ne doit contenir que les salles
/// de l'étage courant : en `Floors` les mondes se superposent, un étage inactif donnerait une clé
/// de butin fausse (voir `map_ldtk::game::rooms::rock_loot_system`).
pub fn room_of<'a>(
    rooms: impl IntoIterator<Item = (&'a str, FixedVec2, FixedVec2)>,
    point: FixedVec2,
) -> String {
    rooms
        .into_iter()
        .filter(|(_, position, size)| {
            point.x >= position.x
                && point.x <= position.x + size.x
                && point.y >= position.y
                && point.y <= position.y + size.y
        })
        .map(|(id, ..)| id)
        .min()
        .unwrap_or("-")
        .to_string()
}

/// Étage terrain des explosions qui `destroys` : terrain de caverne et rochers (`despawn_rollback`,
/// ordre `(y, x)`) ; le butin d'un rocher à butin est tiré par `rock_loot_system` (`map_ldtk`).
#[allow(clippy::type_complexity)]
pub fn explosion_terrain_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    requests: Res<FrameEvents<ExplosionRequest>>,
    mut terrain: ResMut<FrameEvents<DestroyTerrainRequest>>,
    mut broken_events: ResMut<FrameEvents<RockBroken>>,
    grid: Res<CellGrid>,
    rocks: Query<(&GgrsNetId, Entity, &Rock, &fixed_math::FixedTransform3D), With<Rollback>>,
) {
    if !requests.iter().any(|request| request.destroys) {
        return;
    }
    let mut broken: BTreeSet<usize> = BTreeSet::new();
    for request in requests.iter().filter(|request| request.destroys) {
        let center = FixedVec2::new(request.x, request.y);
        if !grid.is_empty() {
            terrain.send(DestroyTerrainRequest::new(center, request.radius));
        }
        // Rochers du rayon, dans l'ordre (y, x) de leur case.
        let ordered = order_iter!(rocks);
        let candidates: Vec<((i32, i32), FixedVec2)> = ordered
            .iter()
            .filter(|(net_id, ..)| !broken.contains(&net_id.0))
            .map(|(_, _, rock, transform)| (rock.cell, transform.translation.truncate()))
            .collect();
        for cell in cells_in_radius(center, request.radius, &candidates) {
            let Some((net_id, entity, rock, transform)) =
                ordered.iter().find(|(_, _, r, _)| r.cell == cell)
            else {
                continue;
            };
            broken.insert(net_id.0);
            commands.entity(*entity).despawn_rollback();
            info!(
                "ggrs{{f={} rock_broken net_id={} cell=({},{}) loot={}}}",
                frame.frame, net_id.0, cell.0, cell.1, rock.loot
            );
            broken_events.send(RockBroken {
                cell,
                x: transform.translation.x,
                y: transform.translation.y,
                loot: rock.loot,
            });
        }
    }
}

/// Étage dégâts : les demandes de bombes et les barils morts à la frame précédente, triés par
/// `GgrsNetId` de l'objet qui explose (voir la doc du module).
#[allow(clippy::type_complexity)]
pub fn explosion_system(
    frame: Res<FrameCount>,
    mut commands: Commands,
    requests: Res<FrameEvents<ExplosionRequest>>,
    mut damage_events: ResMut<FrameEvents<DamageEvent>>,
    mut exploded: ResMut<FrameEvents<Exploded>>,
    barrels: Query<
        (
            &GgrsNetId,
            Entity,
            &fixed_math::FixedTransform3D,
            &world::BarrelState,
            &Death,
        ),
        With<Rollback>,
    >,
    targets: Query<
        (
            &GgrsNetId,
            &fixed_math::FixedTransform3D,
            &Health,
            Has<Death>,
        ),
        With<Rollback>,
    >,
    players: Query<(&GgrsNetId, &Player), With<Rollback>>,
) {
    if requests.is_empty() && barrels.is_empty() {
        return;
    }
    let mut pending: Vec<ExplosionRequest> = requests.iter().cloned().collect();
    let mut barrel_entities: BTreeMap<usize, Entity> = BTreeMap::new();
    if !barrels.is_empty() {
        let player_ids: BTreeMap<usize, GgrsNetId> = order_iter!(players)
            .into_iter()
            .map(|(net_id, player)| (player.handle, net_id.clone()))
            .collect();
        for (net_id, entity, transform, state, death) in order_iter!(barrels) {
            // Crédité au dernier joueur qui l'a touché (points de la mort des victimes).
            let source = death
                .last_hit_by
                .iter()
                .flatten()
                .find_map(|hit| match hit {
                    HitBy::Player(handle) => player_ids.get(handle).cloned(),
                    HitBy::Entity(_) => None,
                })
                .unwrap_or_else(|| net_id.clone());
            pending.push(ExplosionRequest {
                id: net_id.0,
                source,
                source_team: Team::Neutral,
                x: transform.translation.x,
                y: transform.translation.y,
                radius: state.radius,
                damage: state.damage,
                destroys: false,
                friendly_fire: sim_core::damage::FriendlyFire::Always,
            });
            barrel_entities.insert(net_id.0, entity);
        }
    }
    // Stable : deux demandes du même objet (rien d'autre n'en produit) gardent leur ordre d'émission.
    pending.sort_by_key(|request| request.id);

    for request in &pending {
        let center = FixedVec2::new(request.x, request.y);
        let mut hit = 0usize;
        for (target_id, target_transform, health, dying) in order_iter!(targets) {
            if target_id.0 == request.id || dying || health.is_invulnerable_at(frame.frame) {
                continue;
            }
            if !within_radius(
                center,
                target_transform.translation.truncate(),
                request.radius,
            ) {
                continue;
            }
            hit += 1;
            damage_events.send(DamageEvent {
                source: request.source.clone(),
                target: target_id.clone(),
                kind: DamageKind::Physical,
                amount: request.damage,
                frame: frame.frame,
                tags: Tags::parse(["explosion"]),
                source_team: request.source_team,
                friendly_fire: request.friendly_fire,
            });
        }
        info!(
            "ggrs{{f={} explosion id={} targets={}}}",
            frame.frame, request.id, hit
        );
        exploded.send(Exploded {
            frame: frame.frame,
            x: request.x,
            y: request.y,
            radius: request.radius,
            destroys: request.destroys,
        });
        if let Some(entity) = barrel_entities.get(&request.id) {
            commands.entity(*entity).despawn_rollback();
        }
    }
}

/// La carte a au moins une case de piques.
fn has_hazards(grid: Res<SurfaceGrid>, table: Option<Res<SurfaceTable>>) -> bool {
    !grid.is_empty()
        && table.is_some_and(|table| table.has_hazards() && grid.has_hazard_cells(&table))
}

/// Piques sous les pieds d'un joueur (voir la doc du module).
#[allow(clippy::type_complexity)]
pub fn hazard_system(
    frame: Res<FrameCount>,
    grid: Res<SurfaceGrid>,
    table: Res<SurfaceTable>,
    mut commands: Commands,
    mut damage_events: ResMut<FrameEvents<DamageEvent>>,
    mut players: Query<
        (
            &GgrsNetId,
            Entity,
            &fixed_math::FixedTransform3D,
            &Collider,
            Option<&mut HazardState>,
            Option<&DashState>,
            Has<Downed>,
        ),
        (With<Player>, With<Rollback>),
    >,
) {
    for (net_id, entity, transform, collider, state, dash, downed) in order_mut_iter!(players) {
        let Some(def) = grid.hazard_at(&table, feet_position(transform, collider)) else {
            continue;
        };
        if downed {
            continue;
        }
        let dashing = dash.is_some_and(|dash| dash.is_dashing);
        let last_hit = state.as_deref().map(|state| state.last_hit);
        if !def.applies(dashing, last_hit, frame.frame) {
            continue;
        }
        info!(
            "ggrs{{f={} spikes net_id={} damage={}}}",
            frame.frame, net_id.0, def.damage
        );
        damage_events.send(DamageEvent {
            source: net_id.clone(),
            target: net_id.clone(),
            kind: DamageKind::True,
            amount: def.damage,
            frame: frame.frame,
            tags: Tags::parse(["spikes"]),
            source_team: Team::Neutral,
            friendly_fire: sim_core::damage::FriendlyFire::Always,
        });
        match state {
            Some(mut state) => state.last_hit = frame.frame,
            None => {
                commands.entity(entity).insert(HazardState {
                    last_hit: frame.frame,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use items::loot_seed;
    use sim_core::damage::FriendlyFire;

    fn fx(v: i32) -> Fixed {
        Fixed::from_num(v)
    }

    /// Cible à PV d'abscisse `x` (y = 0), invulnérable ou à l'agonie si demandé.
    fn spawn_target(app: &mut App, id: usize, x: i32, invulnerable: bool, dying: bool) {
        let transform = fixed_math::FixedTransform3D::new(
            fixed_math::FixedVec3::new(fx(x), Fixed::ZERO, Fixed::ZERO),
            fixed_math::FixedMat3::IDENTITY,
            fixed_math::FixedVec3::ONE,
        );
        let mut entity = app.world_mut().spawn((
            GgrsNetId(id, "cible".into()),
            transform,
            Health {
                current: fx(100),
                max: fx(100),
                invulnerable_until_frame: invulnerable.then_some(u32::MAX),
            },
            Rollback,
        ));
        if dying {
            entity.insert(Death::default());
        }
    }

    fn request(id: usize, x: i32, radius: i32) -> ExplosionRequest {
        ExplosionRequest {
            id,
            source: GgrsNetId(1, "player".into()),
            source_team: Team::Players,
            x: fx(x),
            y: Fixed::ZERO,
            radius: fx(radius),
            damage: fx(10),
            destroys: false,
            friendly_fire: FriendlyFire::Always,
        }
    }

    fn explosion_app(frame: u32) -> App {
        let mut app = App::new();
        app.init_resource::<bevy_ggrs::RollbackOrdered>()
            .insert_resource(FrameCount { frame })
            .init_resource::<FrameEvents<ExplosionRequest>>()
            .init_resource::<FrameEvents<DamageEvent>>()
            .init_resource::<FrameEvents<Exploded>>()
            .add_systems(Update, explosion_system);
        app
    }

    #[test]
    fn une_explosion_epargne_les_invulnerables_et_les_mourants_et_suit_l_ordre_des_ids() {
        let mut app = explosion_app(7);
        // Dans le rayon 20 de x = 0 : 5, 7 (touchés), 3 (invulnérable : familier de M3-T6), 4 (agonie) ;
        // 6 à exactement 20 (strict : hors rayon), 8 loin.
        spawn_target(&mut app, 7, 10, false, false);
        spawn_target(&mut app, 5, -10, false, false);
        spawn_target(&mut app, 3, 0, true, false);
        spawn_target(&mut app, 4, 5, false, true);
        spawn_target(&mut app, 6, 20, false, false);
        spawn_target(&mut app, 8, 200, false, false);
        app.world_mut()
            .resource_mut::<FrameEvents<ExplosionRequest>>()
            .send(request(10, 0, 20));
        app.update();
        let hit: Vec<usize> = app
            .world()
            .resource::<FrameEvents<DamageEvent>>()
            .iter()
            .map(|event| event.target.0)
            .collect();
        assert_eq!(
            hit,
            vec![5, 7],
            "ids croissants, sans invulnérable ni mourant"
        );
        assert_eq!(
            app.world()
                .resource::<FrameEvents<Exploded>>()
                .iter()
                .count(),
            1
        );
    }

    #[test]
    fn plusieurs_explosions_d_une_frame_passent_par_id_croissant() {
        let mut app = explosion_app(7);
        spawn_target(&mut app, 1, 0, false, false);
        {
            let mut requests = app
                .world_mut()
                .resource_mut::<FrameEvents<ExplosionRequest>>();
            // Émises dans le désordre (ordre des systèmes émetteurs) : 9 avant 2
            let mut late = request(9, 0, 20);
            late.damage = fx(90);
            requests.send(late);
            let mut early = request(2, 0, 20);
            early.damage = fx(20);
            requests.send(early);
        }
        app.update();
        let damages: Vec<Fixed> = app
            .world()
            .resource::<FrameEvents<DamageEvent>>()
            .iter()
            .map(|event| event.amount)
            .collect();
        assert_eq!(
            damages,
            vec![fx(20), fx(90)],
            "l'objet d'id 2 explose avant celui d'id 9"
        );
    }

    #[test]
    fn cle_de_butin_d_un_rocher() {
        assert_eq!(rock_loot_key("salle", (3, -2)), "rock:salle:3:-2");
        assert_ne!(rock_loot_key("a", (1, 2)), rock_loot_key("a", (2, 1)));
        assert_ne!(rock_loot_key("a", (1, 2)), rock_loot_key("b", (1, 2)));
    }

    #[test]
    fn salle_du_rocher_parmi_les_salles_de_l_etage_courant() {
        let v = |x: i32, y: i32| FixedVec2::new(fx(x), fx(y));
        // Deux mondes superposés (`Floors`) : « b » (étage courant) et « a » (étage inactif, même
        // emplacement) se recouvrent ; seul l'étage courant est passé à `room_of`.
        let all = [("a", v(0, 0), v(100, 100)), ("b", v(0, 0), v(100, 100))];
        assert_eq!(
            room_of(all, v(50, 50)),
            "a",
            "non filtré : l'étage inactif gagne au min()"
        );
        let current = [("b", v(0, 0), v(100, 100)), ("c", v(200, 0), v(100, 100))];
        assert_eq!(room_of(current, v(50, 50)), "b");
        assert_eq!(room_of(current, v(250, 50)), "c");
        assert_eq!(room_of(current, v(150, 50)), "-", "hors de toute salle");
        // Bord inclusif ; plus petit identifiant quand deux salles se touchent
        assert_eq!(
            room_of(
                [("z", v(0, 0), v(10, 10)), ("m", v(10, 0), v(10, 10))],
                v(10, 5)
            ),
            "m"
        );
    }

    #[test]
    fn la_graine_du_butin_ne_depend_que_de_la_cle() {
        let key = rock_loot_key("salle", (4, 7));
        assert_eq!(loot_seed(5, 0, &key), loot_seed(5, 0, &key));
        assert_ne!(
            loot_seed(5, 0, &key),
            loot_seed(5, 0, &rock_loot_key("salle", (4, 8)))
        );
    }
}
