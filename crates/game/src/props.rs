//! Surfaces v2 (M2-T11, chantier E4 v2, `docs/conventions.md` §46) : fosses, tables
//! renversables et barils explosifs. Les données et les règles pures vivent dans `world`
//! (`PropDef`, `PitDef`, `flip_direction`, `flipped_rect`) ; ce module les branche sur la
//! simulation. Tout est inerte sur une carte sans fosse, table ni baril.
//!
//! - **Fosses** (`pit_system`, `RollbackSystemSet::World`) : un joueur dont les pieds sont sur une
//!   case de fosse (`SurfaceGrid`, surface à champ `pit`) perd `fall_damage` PV (dégâts `True`,
//!   événement `fall`) et revient à sa dernière position sûre (`PitSafePosition`). Un dash en
//!   cours traverse la fosse si `dash_crosses` ; il tombe s'il finit dessus. La position sûre
//!   est réécrite **avant** toute chute ou retour anticipé. Les ennemis au sol ne la franchissent
//!   pas : ses cases bloquent `Ground`/`GroundBreaker` dans le champ de flux
//!   (`enemy::ai::navigation`), pas `Flying`.
//! - **Tables** (`table_flip_system`, `table_cover_system`) : debout, une table est un obstacle
//!   `Barricade` (marqueur `Window`, couche `window`) qui laisse passer les balles. Une
//!   interaction la renverse à l'opposé du joueur : elle devient un mur mince (`Wall`, couche
//!   `wall`) au bord renversé qui arrête les balles ; elle se brise après `bullets` balles
//!   encaissées (`combat::projectile::BulletWallHit`).
//! - **Barils** (`barrel_explosion_system`) : un baril est un personnage `Enemies` sans IA, blessé
//!   par les balles des joueurs. À 0 PV il garde son `Death` jusqu'à la frame suivante, où il
//!   explose : `damage` à tout ce qui a des PV dans `radius` (barils voisins compris, qui
//!   explosent à leur tour une frame plus tard), puis `despawn_rollback()`.

use bevy::prelude::*;
use bevy_fixed::fixed_math::{self, Fixed, FixedVec2, FixedWide};
use bevy_ggrs::{GgrsSchedule, Rollback, RollbackDespawnCommandExtension};
use combat::actors::{DashState, Velocity};
use combat::downed::Downed;
use combat::projectile::BulletWallHit;
use sim_core::damage::{DamageEvent, DamageKind, FriendlyFire};
use sim_core::frame_events::FrameEvents;
use sim_core::interaction::{Interactable, InteractionType};
use sim_core::tag::Tags;
use sim_core::team::Team;
use std::collections::BTreeMap;
use utils::{frame::FrameCount, net_id::GgrsNetId, order_iter, order_mut_iter};
use world::{
    flip_direction, flipped_rect, BarrelState, PitSafePosition, PropDef, PropTable, SurfaceGrid,
    SurfaceTable, TableState,
};

use crate::character::health::{Death, HitBy};
use crate::character::player::Player;
use crate::character::surface::feet_position;
use crate::collider::{Collider, CollisionLayer, CollisionSettings, Wall, Window};
use crate::interaction::InteractionEvent;
use crate::system_set::RollbackSystemSet;

pub struct PropsPlugin;

impl Plugin for PropsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            GgrsSchedule,
            (
                // Avant `Movement` comme les autres handlers d'`InteractionEvent`, après eux.
                table_flip_system
                    .after(crate::interaction::handle_perk_purchase_interaction)
                    .after(RollbackSystemSet::Input)
                    .before(RollbackSystemSet::Movement)
                    .in_set(RollbackSystemSet::Interaction),
                // Après les collisions de balles (`Weapon`, `Projectiles`), avant les dégâts.
                (
                    // `Velocity` remise à zéro à la chute : ordonné avec l'amortissement du recul
                    pit_system
                        .run_if(has_pits)
                        .before(crate::character::movement::apply_knockback_damping),
                    table_cover_system.after(pit_system),
                )
                    .in_set(RollbackSystemSet::World),
                barrel_explosion_system
                    .after(crate::character::enemy::ai::rules::charge_damage_translate_system)
                    .before(crate::character::health::rollback_resolve_damage_events)
                    .in_set(RollbackSystemSet::CollisionDamage),
            ),
        );
    }
}

/// La carte a au moins une case de fosse : sinon `pit_system` ne tourne pas (aucun coût, aucun
/// composant posé ; une carte à eau ou à glace dans un jeu qui a aussi une fosse n'en porte pas).
fn has_pits(grid: Res<SurfaceGrid>, table: Option<Res<SurfaceTable>>) -> bool {
    !grid.is_empty()
        && table.is_some_and(|table| table.has_pits() && grid.pit_cells(&table).next().is_some())
}

/// Chute dans une fosse et suivi de la dernière position sûre (voir la doc du module).
#[allow(clippy::type_complexity)]
pub fn pit_system(
    frame: Res<FrameCount>,
    grid: Res<SurfaceGrid>,
    table: Res<SurfaceTable>,
    mut commands: Commands,
    mut damage_events: ResMut<FrameEvents<DamageEvent>>,
    mut players: Query<
        (
            &GgrsNetId,
            Entity,
            &mut fixed_math::FixedTransform3D,
            &Collider,
            Option<&mut PitSafePosition>,
            Option<&DashState>,
            Has<Downed>,
            Option<&mut Velocity>,
        ),
        (With<Player>, With<Rollback>),
    >,
) {
    for (net_id, entity, mut transform, collider, safe, dash, downed, velocity) in
        order_mut_iter!(players)
    {
        let feet = feet_position(&transform, collider);
        let dashing = dash.is_some_and(|dash| dash.is_dashing);
        let (cx, cy) = world::CellGrid::cell_of(feet);
        let pit = grid.at(feet).and_then(|id| table.pit(id));

        // Réécrite à chaque frame, avant tout retour anticipé : une frame resimulée ne lit
        // jamais la valeur d'une frame plus récente.
        if pit.is_none() && !dashing && grid.is_safe_cell(&table, cx, cy) {
            let here = PitSafePosition {
                x: transform.translation.x,
                y: transform.translation.y,
            };
            match safe {
                Some(mut safe) => {
                    if *safe != here {
                        *safe = here;
                    }
                }
                None => {
                    commands.entity(entity).insert(here);
                }
            }
            continue;
        }

        let Some(pit) = pit else { continue };
        if downed || (dashing && pit.dash_crosses) {
            continue;
        }
        let Some(safe) = safe else {
            // Apparu dans la fosse : aucun point de retour, la fosse ne le prend pas.
            continue;
        };
        info!(
            "ggrs{{f={} pit_fall net_id={} from=({},{}) to=({},{}) damage={}}}",
            frame.frame,
            net_id.0,
            transform.translation.x,
            transform.translation.y,
            safe.x,
            safe.y,
            pit.fall_damage
        );
        damage_events.send(DamageEvent {
            source: net_id.clone(),
            target: net_id.clone(),
            kind: DamageKind::True,
            amount: pit.fall_damage,
            frame: frame.frame,
            tags: Tags::parse(["fall"]),
            source_team: Team::Neutral,
            friendly_fire: FriendlyFire::Always,
        });
        transform.translation.x = safe.x;
        transform.translation.y = safe.y;
        // Il repart de l'arrêt (sinon son élan le remet dans la fosse).
        if let Some(mut velocity) = velocity {
            velocity.main = FixedVec2::ZERO;
            velocity.knockback = FixedVec2::ZERO;
        }
    }
}

/// Renverse la table interagie, à l'opposé du joueur. Le collider devient une bande mince au
/// bord renversé (`world::flipped_rect`), arrêtant les balles ; le passage de l'emprise
/// d'origine est rendu.
#[allow(clippy::type_complexity)]
pub fn table_flip_system(
    frame: Res<FrameCount>,
    events: Res<FrameEvents<InteractionEvent>>,
    props: Option<Res<PropTable>>,
    settings: Res<CollisionSettings>,
    mut commands: Commands,
    positions: Query<&fixed_math::FixedTransform3D>,
    mut tables: Query<(&GgrsNetId, &fixed_math::FixedTransform3D, &mut TableState), With<Rollback>>,
) {
    for event in events.iter() {
        if event.interaction_type != InteractionType::Table {
            continue;
        }
        let Ok((net_id, table_transform, mut state)) = tables.get_mut(event.interactable) else {
            continue;
        };
        if state.flipped.is_some() {
            continue;
        }
        let Ok(player_transform) = positions.get(event.interactor) else {
            continue;
        };
        let thickness = props
            .as_deref()
            .and_then(|props| match props.0.get(&state.prop) {
                Some(PropDef::Table(def)) => Some(def.thickness),
                _ => None,
            })
            .unwrap_or_else(|| Fixed::from_num(6));
        let direction = flip_direction(
            player_transform.translation.truncate(),
            table_transform.translation.truncate(),
        );
        let (width, height, x, y) = flipped_rect(state.width, state.height, direction, thickness);
        state.flipped = Some(direction);
        info!(
            "ggrs{{f={} table_flip net_id={} by={} dir={:?}}}",
            frame.frame, net_id.0, event.interactor_net_id.0, direction
        );
        commands
            .entity(event.interactable)
            .remove::<(Window, Interactable)>()
            .insert((
                Wall,
                CollisionLayer(settings.wall_layer),
                Collider {
                    shape: crate::collider::ColliderShape::Rectangle { width, height },
                    offset: fixed_math::FixedVec3::new(x, y, Fixed::ZERO),
                },
            ));
    }
}

/// Compte les balles encaissées par les tables renversées ; à zéro, la table est détruite.
pub fn table_cover_system(
    frame: Res<FrameCount>,
    hits: Res<FrameEvents<BulletWallHit>>,
    mut commands: Commands,
    mut tables: Query<(&GgrsNetId, Entity, &mut TableState), With<Rollback>>,
) {
    if hits.is_empty() {
        return;
    }
    let mut by_net_id: BTreeMap<usize, Entity> = BTreeMap::new();
    for (net_id, entity, _) in order_iter!(tables) {
        by_net_id.insert(net_id.0, entity);
    }
    for hit in hits.iter() {
        let Some(&entity) = by_net_id.get(&hit.wall.0) else {
            continue;
        };
        let Ok((net_id, _, mut state)) = tables.get_mut(entity) else {
            continue;
        };
        if state.flipped.is_none() || state.bullets_left == 0 {
            continue;
        }
        state.bullets_left -= 1;
        info!(
            "ggrs{{f={} table_hit net_id={} left={}}}",
            frame.frame, net_id.0, state.bullets_left
        );
        if state.bullets_left == 0 {
            commands.entity(entity).despawn_rollback();
        }
    }
}

/// Explosion des barils morts à la frame précédente (voir la doc du module). Avant
/// `rollback_resolve_damage_events` : les dégâts sont résolus dans la même frame, un baril
/// voisin touché meurt donc cette frame et explose à la suivante (chaîne déterministe, barils
/// pris par `GgrsNetId`).
#[allow(clippy::type_complexity)]
pub fn barrel_explosion_system(
    frame: Res<FrameCount>,
    mut commands: Commands,
    mut damage_events: ResMut<FrameEvents<DamageEvent>>,
    barrels: Query<
        (
            &GgrsNetId,
            Entity,
            &fixed_math::FixedTransform3D,
            &BarrelState,
            &Death,
        ),
        With<Rollback>,
    >,
    targets: Query<
        (&GgrsNetId, &fixed_math::FixedTransform3D, Has<Death>),
        (With<Rollback>, With<crate::character::health::Health>),
    >,
    players: Query<(&GgrsNetId, &Player), With<Rollback>>,
) {
    if barrels.is_empty() {
        return;
    }
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
        let center: FixedVec2 = transform.translation.truncate();
        let radius_sq = FixedWide::from_num(state.radius).saturating_mul(FixedWide::from_num(state.radius));
        let mut hit = 0usize;
        for (target_id, target_transform, dying) in order_iter!(targets) {
            if target_id.0 == net_id.0 || dying {
                continue;
            }
            let offset = target_transform.translation.truncate() - center;
            if offset.length_squared() > radius_sq {
                continue;
            }
            hit += 1;
            damage_events.send(DamageEvent {
                source: source.clone(),
                target: target_id.clone(),
                kind: DamageKind::Physical,
                amount: state.damage,
                frame: frame.frame,
                tags: Tags::parse(["explosion"]),
                source_team: Team::Neutral,
                friendly_fire: FriendlyFire::Always,
            });
        }
        info!(
            "ggrs{{f={} barrel_explosion net_id={} targets={}}}",
            frame.frame, net_id.0, hit
        );
        commands.entity(entity).despawn_rollback();
    }
}
