//! Familiers et orbitaux (M3-T6, chantier C3, `docs/conventions.md` §71) : l'exécution des
//! contrats de §63 (`Follow`, `Orbit`, `MirrorShot`, `BlockBullets`, `FamiliarOf`,
//! `ItemDef.familiar`).
//!
//! **Apparition dérivée de l'état.** [`familiar_sync_system`] compare chaque frame les familiers
//! qu'un joueur *devrait* avoir (ses objets tenus : passifs dans l'ordre de ramassage, puis l'actif ;
//! `per_copy` familiers par exemplaire) à ceux qui existent (`FamiliarOf`). Ramasser un objet en
//! crée, le perdre (actif remplacé) les détruit par `despawn_rollback()` : aucun événement à
//! rejouer, un rollback qui remonte avant le ramassage retombe sur le même état. `GgrsNetId` :
//! porteurs par `GgrsNetId` croissant, puis rang croissant (aucun alloué par le hasard d'un
//! ramassage).
//!
//! **Un familier est un personnage** (`create_character`) de l'équipe de sa fiche (sinon celle du
//! porteur), sans `Enemy` ni `Player` : ni compté comme ennemi (salles, vagues, HUD), ni visé par
//! l'IA (qui ne cible que `With<Player>`), ni obstacle pour personne (couche `environment`, qui ne
//! collisionne avec rien). **Immunisé** : `Health.invulnerable_until_frame = u32::MAX`, ce que les
//! collisions de balles, de projectiles et de mêlée respectent déjà (traversé, jamais touché).
//!
//! Systèmes dans `RollbackSystemSet::Weapon`, après les tirs des joueurs et avant le mouvement des
//! balles : synchronisation, déplacement, `MirrorShot`, puis (entre le mouvement des balles et leurs
//! collisions) `BlockBullets`.

use std::collections::BTreeMap;

use behaviors::{
    familiar::{follow_step, orbit_position},
    Behavior, FamiliarGun, FamiliarOf,
};
use bevy::prelude::*;
use bevy_fixed::fixed_math::{self, Fixed, FixedVec2};
use bevy_ggrs::{GgrsSchedule, PlayerInputs, Rollback, RollbackDespawnCommandExtension};
use combat::{
    collider::{Collider, CollisionSettings},
    collision_grid::{collider_aabb, union_aabb},
    downed::Downed,
    team::team_allows_hit,
    weapons::{
        bullet_rollback_collision_system, bullet_rollback_system, carried_projectile_mods,
        shot_projectile, spawn_bullet, tear_motion, Bullet, BulletSpawn, ShotStats,
        WeaponInventory, WeaponState,
    },
};
use items::{Inventory, ItemTable, SynergyTable};
use sim_core::{
    aim::{aim_direction, AimMode},
    frame_events::FrameEvents,
    stats::StatId,
    tag::Tags,
    team::Team,
    triggers::WeaponFired,
};
use stats::StatReader;
use utils::{
    frame::FrameCount,
    net_id::{GgrsNetId, GgrsNetIdFactory},
    order_iter,
};

use crate::{
    balance::ResolvedBalance,
    character::{
        config::CharacterConfigs, create::create_character, player::jjrs::PeerConfig,
        player::Player,
    },
    system_set::RollbackSystemSet,
};

/// Règles de comportement d'un familier (liste `ai.behaviors` de son personnage, copiée à la
/// création). Statique, hors rollback (comme `EnemyBehaviors` et `Team`).
#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub struct FamiliarBehaviors(pub Vec<Behavior>);

/// Le mouvement d'un familier : la première règle `Orbit` ou `Follow` de sa liste.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FamiliarMotion {
    Follow { gap: Fixed },
    Orbit { radius: Fixed, frames_per_turn: u32 },
}

impl FamiliarBehaviors {
    pub fn motion(&self) -> Option<FamiliarMotion> {
        self.0.iter().find_map(|rule| match rule {
            Behavior::Follow { gap } => Some(FamiliarMotion::Follow { gap: *gap }),
            Behavior::Orbit {
                radius,
                frames_per_turn,
            } => Some(FamiliarMotion::Orbit {
                radius: *radius,
                frames_per_turn: *frames_per_turn,
            }),
            _ => None,
        })
    }

    /// `MirrorShot { every }`, s'il y en a un.
    pub fn mirror_shot(&self) -> Option<u32> {
        self.0.iter().find_map(|rule| match rule {
            Behavior::MirrorShot { every } => Some(*every),
            _ => None,
        })
    }

    pub fn blocks_bullets(&self) -> bool {
        self.0
            .iter()
            .any(|rule| matches!(rule, Behavior::BlockBullets))
    }
}

/// Personnages que le porteur doit avoir, dans l'ordre des rangs : pour chaque objet tenu (passifs
/// dans l'ordre de ramassage, un passif en double compte deux fois, puis l'actif) qui donne un
/// familier, `per_copy` fois son personnage. Pure.
pub fn desired_familiars(table: &ItemTable, inventory: &Inventory) -> Vec<String> {
    let mut out = Vec::new();
    let held = inventory
        .passives
        .iter()
        .map(String::as_str)
        .chain(inventory.active.iter().map(|slot| slot.item.as_str()));
    for id in held {
        if let Some(familiar) = table.0.get(id).and_then(|def| def.familiar.as_ref()) {
            for _ in 0..familiar.per_copy {
                out.push(familiar.character.clone());
            }
        }
    }
    out
}

/// Plan de réconciliation d'un porteur : `(rangs à détruire, rangs à créer)`. On garde le plus long
/// préfixe commun entre l'existant (`existing[rang]`, personnage du familier de ce rang) et le
/// voulu ; tout ce qui suit la première différence est détruit puis recréé (un objet perdu au milieu
/// de la liste décale les rangs suivants, donc les refait ; l'actif remplacé, en queue, ne touche que
/// ses propres familiers). Pure.
pub fn reconcile(existing: &[String], desired: &[String]) -> (Vec<usize>, Vec<usize>) {
    let common = existing
        .iter()
        .zip(desired)
        .take_while(|(have, want)| have == want)
        .count();
    (
        (common..existing.len()).collect(),
        (common..desired.len()).collect(),
    )
}

type OwnerQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static GgrsNetId,
        &'static Player,
        &'static fixed_math::FixedTransform3D,
        Option<&'static Team>,
        Option<&'static Inventory>,
    ),
    (With<Rollback>, Without<FamiliarOf>),
>;

/// Crée et détruit les familiers selon les objets tenus (voir la doc du module). `Weapon`, après
/// les tirs des joueurs (un objet ramassé en `Interaction` cette frame est déjà dans l'inventaire).
#[allow(clippy::too_many_arguments)]
pub fn familiar_sync_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    table: Option<Res<ItemTable>>,
    owners: OwnerQuery,
    familiars: Query<(&GgrsNetId, Entity, &FamiliarOf), With<Rollback>>,
    characters: Res<CharacterConfigs>,
    balance: Res<ResolvedBalance>,
    collision_settings: Res<CollisionSettings>,
    mut id_factory: ResMut<GgrsNetIdFactory>,
) {
    let Some(table) = table else {
        return;
    };
    // Existant par porteur : (rang, personnage, entité), trié par rang.
    let mut existing: BTreeMap<usize, Vec<(u32, String, Entity)>> = BTreeMap::new();
    for (net_id, entity, familiar) in order_iter!(familiars) {
        existing.entry(familiar.owner.0).or_default().push((
            familiar.rank,
            net_id.1.clone(),
            entity,
        ));
    }
    for list in existing.values_mut() {
        list.sort_by_key(|(rank, ..)| *rank);
    }
    if existing.is_empty() && !owners.iter().any(|(.., inventory)| inventory.is_some()) {
        return;
    }

    for (net_id, _player, transform, team, inventory) in order_iter!(owners) {
        let desired = inventory
            .map(|inventory| desired_familiars(&table, inventory))
            .unwrap_or_default();
        let have = existing.remove(&net_id.0).unwrap_or_default();
        let names: Vec<String> = have.iter().map(|(_, name, _)| name.clone()).collect();
        let (destroy, create) = reconcile(&names, &desired);
        for index in destroy {
            commands.entity(have[index].2).despawn_rollback();
            info!(
                "ggrs{{f={} familiar_lost owner={} rank={}}}",
                frame.frame, net_id.0, have[index].0
            );
        }
        for rank in create {
            let character = &desired[rank];
            let spawned = spawn_familiar(
                &mut commands,
                character,
                net_id,
                rank as u32,
                transform.translation,
                team.copied(),
                &characters,
                &balance,
                &collision_settings,
                &mut id_factory,
            );
            if let Some(entity) = spawned {
                info!(
                    "ggrs{{f={} familiar_spawn owner={} rank={} character={}}}",
                    frame.frame, net_id.0, rank, character
                );
                let _ = entity;
            }
        }
    }
    // Familiers dont le porteur n'existe plus (mort, détruit) : ils restent où ils sont, immobiles
    // (`familiar_motion_system` ne les bouge pas), jusqu'à la fin de la partie.
}

#[allow(clippy::too_many_arguments)]
fn spawn_familiar(
    commands: &mut Commands,
    character: &str,
    owner: &GgrsNetId,
    rank: u32,
    position: fixed_math::FixedVec3,
    owner_team: Option<Team>,
    characters: &CharacterConfigs,
    balance: &ResolvedBalance,
    collision_settings: &CollisionSettings,
    id_factory: &mut ResMut<GgrsNetIdFactory>,
) -> Option<Entity> {
    let config = characters.get(character)?;
    let behaviors = config
        .ai
        .as_ref()
        .and_then(|ai| ai.behaviors.clone())
        .unwrap_or_default();
    let health_max = balance
        .health_max_by_character
        .get(character)
        .copied()
        .unwrap_or(fixed_math::FIXED_ONE);
    let entity = create_character(
        commands,
        characters,
        character.to_string(),
        None,
        Color::srgb(0.4, 0.8, 1.0),
        position,
        // Couche `environment` : aucune entrée dans la matrice de collision, donc aucun familier ne
        // gêne un joueur, un ennemi ou une balle (ce que leur effet voulu fait, ils le font eux-mêmes).
        crate::collider::CollisionLayer(collision_settings.environment_layer),
        id_factory,
        &[],
        health_max,
        None,
    );
    let team = config.team.or(owner_team).unwrap_or(Team::Players);
    let mut health = crate::character::health::new_health(health_max);
    health.invulnerable_until_frame = Some(u32::MAX);
    let mirror = behaviors
        .iter()
        .any(|rule| matches!(rule, Behavior::MirrorShot { .. }));
    commands.entity(entity).insert((
        health,
        team,
        FamiliarOf {
            owner: owner.clone(),
            rank,
        },
        FamiliarBehaviors(behaviors),
    ));
    if mirror {
        commands.entity(entity).insert(FamiliarGun::default());
    }
    Some(entity)
}

/// `Follow` et `Orbit` : déplace chaque familier par rapport à son porteur. Les familiers d'un
/// porteur sont traités par rang croissant : les orbitaux se répartissent sur le cercle (phase =
/// rang parmi les orbitaux × 2π / leur nombre), les suiveurs forment une file (chacun vise celui qui
/// le précède, le premier le porteur). Un porteur à terre ou disparu : ses familiers ne bougent pas.
#[allow(clippy::type_complexity)]
pub fn familiar_motion_system(
    frame: Res<FrameCount>,
    owners: Query<
        (&GgrsNetId, &fixed_math::FixedTransform3D, Has<Downed>),
        (With<Player>, Without<FamiliarOf>),
    >,
    mut familiars: Query<
        (
            &GgrsNetId,
            &FamiliarOf,
            &FamiliarBehaviors,
            &mut fixed_math::FixedTransform3D,
        ),
        With<Rollback>,
    >,
) {
    let mut by_owner: BTreeMap<usize, Vec<(u32, usize, FamiliarMotion)>> = BTreeMap::new();
    for (net_id, familiar, behaviors, _) in familiars.iter() {
        if let Some(motion) = behaviors.motion() {
            by_owner
                .entry(familiar.owner.0)
                .or_default()
                .push((familiar.rank, net_id.0, motion));
        }
    }
    if by_owner.is_empty() {
        return;
    }
    let mut targets: BTreeMap<usize, FixedVec2> = BTreeMap::new();
    for (owner_id, list) in by_owner.iter_mut() {
        list.sort_unstable_by_key(|(rank, net, _)| (*rank, *net));
        let Some((_, owner_transform, downed)) = owners.iter().find(|(id, ..)| id.0 == *owner_id)
        else {
            continue;
        };
        if downed {
            continue;
        }
        let center = owner_transform.translation.truncate();
        let orbitals = list
            .iter()
            .filter(|(_, _, motion)| matches!(motion, FamiliarMotion::Orbit { .. }))
            .count() as u32;
        let mut orbital_index = 0;
        let mut anchor = center;
        for (_, net, motion) in list.iter() {
            match motion {
                FamiliarMotion::Orbit {
                    radius,
                    frames_per_turn,
                } => {
                    targets.insert(
                        *net,
                        orbit_position(
                            center,
                            *radius,
                            *frames_per_turn,
                            orbital_index,
                            orbitals,
                            frame.frame,
                        ),
                    );
                    orbital_index += 1;
                }
                FamiliarMotion::Follow { gap } => {
                    // La position courante du suiveur, lue plus bas ; la file avance par pas.
                    let current = familiars
                        .iter()
                        .find(|(id, ..)| id.0 == *net)
                        .map(|(_, _, _, transform)| transform.translation.truncate());
                    if let Some(current) = current {
                        let next = follow_step(current, anchor, *gap);
                        targets.insert(*net, next);
                        anchor = next;
                    }
                }
            }
        }
    }
    for (net_id, _, _, mut transform) in familiars.iter_mut() {
        if let Some(target) = targets.get(&net_id.0) {
            transform.translation.x = target.x;
            transform.translation.y = target.y;
        }
    }
}

/// Le joueur `handle` a réellement tiré à `frame` : un événement [`WeaponFired`] de cette frame et de
/// ce joueur. Pure. (`WeaponState::last_fire_frame` ne convient pas : sa valeur par défaut, 0,
/// ferait croire à un tir à la frame 0.)
pub fn owner_fired(events: &[WeaponFired], handle: usize, frame: u32) -> bool {
    events
        .iter()
        .any(|event| event.frame == frame && event.player_handle == handle)
}

/// `MirrorShot` : quand le porteur a **réellement tiré** cette frame (`WeaponFired`, émis par
/// `weapon_rollback_system` quand la balle part, jamais pour une gâchette que la cadence ou le
/// chargeur refuse), le familier tire **les larmes du porteur** (Q13, réponse de William du
/// 2026-10-16) : l'arme active du porteur dans son mode actif, composée avec ses objets et synergies
/// (`combat::weapons::shot_projectile`, §66), ses stats de larmes (`Damage`, `Range`, `ShotSpeed`,
/// `ShotSize`, `Luck`), dans la direction de visée du porteur, au plus une fois tous les `every`
/// frames. Un tir par déclenchement (pas de plomb multiple), sans dispersion ni recul (aucun RNG
/// consommé) ; ni chargeur ni cadence du porteur consommés. La balle porte le `GgrsNetId` et
/// l'équipe du **porteur** (les kills lui reviennent, elle ne le touche jamais) et les tags du
/// familier.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn familiar_mirror_shot_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    inputs: Res<PlayerInputs<PeerConfig>>,
    fired_events: Res<FrameEvents<WeaponFired>>,
    owners: Query<
        (
            Entity,
            &GgrsNetId,
            &Player,
            Has<Downed>,
            Option<&Team>,
            &WeaponInventory,
            Option<&Inventory>,
            Option<&AimMode>,
        ),
        (With<Rollback>, Without<FamiliarOf>),
    >,
    weapon_states: Query<&WeaponState>,
    mut familiars: Query<
        (
            &GgrsNetId,
            &FamiliarOf,
            &FamiliarBehaviors,
            &fixed_math::FixedTransform3D,
            Option<&Team>,
            Option<&Tags>,
            &mut FamiliarGun,
        ),
        With<Rollback>,
    >,
    stats: StatReader,
    item_table: Option<Res<ItemTable>>,
    synergy_table: Option<Res<SynergyTable>>,
    collision_settings: Res<CollisionSettings>,
    mut id_factory: ResMut<GgrsNetIdFactory>,
) {
    if familiars.is_empty() || fired_events.is_empty() {
        return;
    }
    let fired_events: Vec<WeaponFired> = fired_events.iter().cloned().collect();
    for (net_id, familiar, behaviors, transform, team, tags, mut gun) in
        utils::order_mut_iter!(familiars)
    {
        let Some(every) = behaviors.mirror_shot() else {
            continue;
        };
        let Some((owner_entity, owner_id, player, downed, owner_team, weapons, held, aim_mode)) =
            owners.iter().find(|(_, id, ..)| id.0 == familiar.owner.0)
        else {
            continue;
        };
        let (input, _) = inputs[player.handle];
        let fired = owner_fired(&fired_events, player.handle, frame.frame);
        if downed || !fired || !gun.ready(frame.frame, every) {
            continue;
        }
        let aim = aim_direction(
            aim_mode.copied().unwrap_or_default(),
            input.pan_x,
            input.pan_y,
        );
        if aim == FixedVec2::ZERO {
            continue;
        }
        let Some((weapon_entity, weapon)) = weapons.weapons.get(weapons.active_weapon_index) else {
            continue;
        };
        let Some(mode) = weapon_states
            .get(*weapon_entity)
            .ok()
            .and_then(|state| weapon.config.firing_modes.get(&state.active_mode))
        else {
            continue;
        };
        let damage_mult = stats.get(owner_entity, &StatId::Damage, fixed_math::FIXED_ONE);
        let range_mult = stats.get(owner_entity, &StatId::Range, fixed_math::FIXED_ONE);
        let shot = ShotStats {
            speed: stats.get(owner_entity, &StatId::ShotSpeed, fixed_math::FIXED_ONE),
            size: stats.get(owner_entity, &StatId::ShotSize, fixed_math::FIXED_ONE),
        };
        let carried =
            carried_projectile_mods(held, item_table.as_deref(), synergy_table.as_deref());
        let projectile = shot_projectile(&weapon.config, mode, &carried, damage_mult, || {
            stats.get(owner_entity, &StatId::Luck, Fixed::ZERO)
        });
        let (velocity, damage) = tear_motion(&mode.bullet_type, aim, shot, damage_mult);
        let shooter_tags = tags.cloned().unwrap_or_default();
        let id_label = format!("{}", mode.bullet_type);
        spawn_bullet(
            &mut commands,
            &collision_settings,
            &mut id_factory,
            BulletSpawn {
                position: transform.translation,
                rotation: fixed_math::FixedMat3::IDENTITY,
                velocity,
                bullet_type: mode.bullet_type,
                damage,
                range: mode.range.saturating_mul(range_mult),
                player_handle: player.handle,
                created_at: frame.frame,
                source: owner_id,
                source_team: team
                    .copied()
                    .or(owner_team.copied())
                    .unwrap_or(Team::Players),
                source_tags: &shooter_tags,
                friendly_fire: weapon.config.friendly_fire,
                projectile,
                id_label,
                shot_size: shot.size,
            },
        );
        gun.last_shot = Some(frame.frame);
        info!(
            "ggrs{{f={} familiar_shot familiar={} owner={}}}",
            frame.frame, net_id.0, owner_id.0
        );
    }
}

/// Le disque de rayon `bullet_radius` balayé le long du segment `old → new` touche-t-il le collider
/// d'un familier centré en `center` (offset déjà ajouté) ? Test **géométrique** : l'AABB balayée ne
/// sert qu'à présélectionner (elle recouvre les coins d'un déplacement diagonal et les disques sans
/// qu'il y ait contact). Rond : distance du centre au segment `< R + r`. Rectangle : le segment
/// traverse le rectangle, ou la distance segment–rectangle (point le plus proche d'un bout du segment
/// ou d'un coin du rectangle) est `< r`. Arithmétique `FixedWide` (I64F32), positions relatives au
/// centre, jamais de `f32`.
pub fn swept_hits_collider(
    old: FixedVec2,
    new: FixedVec2,
    bullet_radius: Fixed,
    center: FixedVec2,
    shape: &combat::collider::ColliderShape,
) -> bool {
    use combat::collider::ColliderShape;
    type W = fixed_math::FixedWide;
    let rel = |p: FixedVec2| {
        (
            W::from_num(p.x.saturating_sub(center.x)),
            W::from_num(p.y.saturating_sub(center.y)),
        )
    };
    let (a, b) = (rel(old), rel(new));
    let r = W::from_num(bullet_radius);
    let two = W::from_num(2);
    // Distance² du point (0, 0)-centré `p` au segment `a → b`.
    let seg_point_dist_sq = |p: (W, W)| -> W {
        let d = (b.0 - a.0, b.1 - a.1);
        let len_sq = d.0.saturating_mul(d.0) + d.1.saturating_mul(d.1);
        let mut t = W::ZERO;
        if len_sq > W::ZERO {
            let dot = (p.0 - a.0).saturating_mul(d.0) + (p.1 - a.1).saturating_mul(d.1);
            t = (dot / len_sq).clamp(W::ZERO, W::ONE);
        }
        let q = (
            a.0 + d.0.saturating_mul(t) - p.0,
            a.1 + d.1.saturating_mul(t) - p.1,
        );
        q.0.saturating_mul(q.0) + q.1.saturating_mul(q.1)
    };
    match shape {
        ColliderShape::Circle { radius } => {
            let reach = W::from_num(*radius) + r;
            seg_point_dist_sq((W::ZERO, W::ZERO)) < reach.saturating_mul(reach)
        }
        ColliderShape::Rectangle { width, height } => {
            let (hx, hy) = (W::from_num(*width) / two, W::from_num(*height) / two);
            let point_rect_dist_sq = |p: (W, W)| -> W {
                let dx = (p.0.abs() - hx).max(W::ZERO);
                let dy = (p.1.abs() - hy).max(W::ZERO);
                dx.saturating_mul(dx) + dy.saturating_mul(dy)
            };
            // Le segment traverse le rectangle (tranches par axe).
            let d = (b.0 - a.0, b.1 - a.1);
            let (mut t_min, mut t_max) = (W::ZERO, W::ONE);
            let mut crosses = true;
            for (start, delta, half) in [(a.0, d.0, hx), (a.1, d.1, hy)] {
                if delta == W::ZERO {
                    if start.abs() > half {
                        crosses = false;
                    }
                } else {
                    let (t1, t2) = ((-half - start) / delta, (half - start) / delta);
                    t_min = t_min.max(t1.min(t2));
                    t_max = t_max.min(t1.max(t2));
                }
            }
            if crosses && t_min <= t_max {
                return true;
            }
            let r_sq = r.saturating_mul(r);
            if point_rect_dist_sq(a) < r_sq || point_rect_dist_sq(b) < r_sq {
                return true;
            }
            [(hx, hy), (hx, -hy), (-hx, hy), (-hx, -hy)]
                .into_iter()
                .any(|corner| seg_point_dist_sq(corner) < r_sq)
        }
    }
}

/// `BlockBullets` : une balle (ordinaire ou composable) que `team_allows_hit` dit hostile au
/// familier et dont le déplacement de la frame (AABB balayée) touche son collider est détruite par
/// `despawn_rollback()`, sans `on_expire`, comme le blank. Entre le mouvement des balles et leurs
/// collisions : une balle ne traverse pas l'orbital pour toucher le porteur derrière lui.
#[allow(clippy::type_complexity)]
pub fn familiar_block_bullets_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    familiars: Query<
        (
            &GgrsNetId,
            &FamiliarBehaviors,
            &fixed_math::FixedTransform3D,
            &Collider,
            &Team,
        ),
        With<Rollback>,
    >,
    bullets: Query<
        (
            &GgrsNetId,
            Entity,
            &fixed_math::FixedTransform3D,
            &Collider,
            &Bullet,
        ),
        (With<Rollback>, Without<FamiliarOf>),
    >,
) {
    let blockers: Vec<_> = order_iter!(familiars)
        .into_iter()
        .filter(|(_, behaviors, ..)| behaviors.blocks_bullets())
        .collect();
    if blockers.is_empty() {
        return;
    }
    for (bullet_id, entity, transform, collider, bullet) in order_iter!(bullets) {
        let old = fixed_math::FixedVec3::new(
            transform.translation.x - bullet.velocity.x,
            transform.translation.y - bullet.velocity.y,
            transform.translation.z,
        );
        let swept = union_aabb(
            collider_aabb(&old, collider),
            collider_aabb(&transform.translation, collider),
        );
        let radius = match &collider.shape {
            combat::collider::ColliderShape::Circle { radius } => *radius,
            combat::collider::ColliderShape::Rectangle { width, height } => {
                (*width).max(*height) / Fixed::from_num(2)
            }
        };
        let (old_xy, new_xy) = (old.truncate(), transform.translation.truncate());
        for (familiar_id, _, familiar_transform, familiar_collider, team) in &blockers {
            if !team_allows_hit(
                bullet.source_team,
                **team,
                bullet.friendly_fire,
                &bullet.tags,
            ) {
                continue;
            }
            // AABB balayée : présélection seulement ; le contact se décide par la géométrie.
            let body = collider_aabb(&familiar_transform.translation, familiar_collider);
            if !swept.overlaps(&body) {
                continue;
            }
            let center = (familiar_transform.translation + familiar_collider.offset).truncate();
            if swept_hits_collider(old_xy, new_xy, radius, center, &familiar_collider.shape) {
                commands.entity(entity).despawn_rollback();
                info!(
                    "ggrs{{f={} familiar_block familiar={} bullet={}}}",
                    frame.frame, familiar_id.0, bullet_id.0
                );
                break;
            }
        }
    }
}

pub struct FamiliarsPlugin;

impl Plugin for FamiliarsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            GgrsSchedule,
            (
                familiar_sync_system.after(combat::weapons::weapon_rollback_system),
                familiar_motion_system.after(familiar_sync_system),
                familiar_mirror_shot_system
                    .after(familiar_motion_system)
                    .before(bullet_rollback_system),
                familiar_block_bullets_system
                    .after(bullet_rollback_system)
                    .before(bullet_rollback_collision_system),
            )
                .in_set(RollbackSystemSet::Weapon),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> ItemTable {
        let parse = |src: &str| -> items::ItemDef { ron::from_str(src).unwrap() };
        ItemTable(BTreeMap::from([
            (
                "ami".to_string(),
                parse(
                    r#"(name: "A", kind: Passive, familiar: Some((character: "ami", per_copy: 2)))"#,
                ),
            ),
            (
                "orbe".to_string(),
                parse(r#"(name: "O", kind: Passive, familiar: Some((character: "orbe")))"#),
            ),
            (
                "gardien".to_string(),
                parse(
                    r#"(name: "G", kind: Active(Frames(5)), familiar: Some((character: "gardien")))"#,
                ),
            ),
            ("rien".to_string(), parse(r#"(name: "R", kind: Passive)"#)),
        ]))
    }

    fn inventory(passives: &[&str], active: Option<&str>) -> Inventory {
        Inventory {
            passives: passives.iter().map(|s| s.to_string()).collect(),
            active: active.map(|id| items::ActiveSlot::new(id.to_string())),
            consumables: BTreeMap::new(),
        }
    }

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn familiers_voulus_par_exemplaire_dans_l_ordre_de_ramassage() {
        let table = table();
        assert!(desired_familiars(&table, &Inventory::default()).is_empty());
        assert!(desired_familiars(&table, &inventory(&["rien"], None)).is_empty());
        // `per_copy: 2` ; un passif en double compte deux fois ; l'actif en dernier.
        assert_eq!(
            desired_familiars(
                &table,
                &inventory(&["orbe", "rien", "ami", "orbe"], Some("gardien"))
            ),
            names(&["orbe", "ami", "ami", "orbe", "gardien"])
        );
    }

    #[test]
    fn reconciliation_garde_le_prefixe_commun() {
        let have = names(&["orbe", "ami", "ami", "gardien"]);
        // Rien ne change.
        assert_eq!(reconcile(&have, &have), (vec![], vec![]));
        // Actif perdu (queue) : seul lui est détruit.
        assert_eq!(
            reconcile(&have, &names(&["orbe", "ami", "ami"])),
            (vec![3], vec![])
        );
        // Actif remplacé par un autre : on refait la queue.
        assert_eq!(
            reconcile(&have, &names(&["orbe", "ami", "ami", "orbe"])),
            (vec![3], vec![3])
        );
        // Objet ramassé : création à la suite.
        assert_eq!(
            reconcile(&names(&["orbe"]), &names(&["orbe", "ami", "ami"])),
            (vec![], vec![1, 2])
        );
        // Objet du milieu perdu : les rangs suivants sont refaits.
        assert_eq!(
            reconcile(&have, &names(&["orbe", "ami", "gardien"])),
            (vec![2, 3], vec![2])
        );
        // Premier ramassage.
        assert_eq!(reconcile(&[], &names(&["orbe"])), (vec![], vec![0]));
    }

    #[test]
    fn mouvement_et_tir_d_un_familier_se_lisent_dans_ses_regles() {
        let rules = FamiliarBehaviors(vec![
            Behavior::MirrorShot { every: 12 },
            Behavior::Orbit {
                radius: Fixed::from_num(30),
                frames_per_turn: 90,
            },
            Behavior::Follow {
                gap: Fixed::from_num(20),
            },
            Behavior::BlockBullets,
        ]);
        assert_eq!(
            rules.motion(),
            Some(FamiliarMotion::Orbit {
                radius: Fixed::from_num(30),
                frames_per_turn: 90
            }),
            "la première règle de mouvement l'emporte"
        );
        assert_eq!(rules.mirror_shot(), Some(12));
        assert!(rules.blocks_bullets());
        assert_eq!(FamiliarBehaviors::default().motion(), None);
    }

    fn v(x: f32, y: f32) -> FixedVec2 {
        FixedVec2::new(Fixed::from_num(x), Fixed::from_num(y))
    }

    fn rect(w: f32, h: f32) -> combat::collider::ColliderShape {
        combat::collider::ColliderShape::Rectangle {
            width: Fixed::from_num(w),
            height: Fixed::from_num(h),
        }
    }

    fn disc(r: f32) -> combat::collider::ColliderShape {
        combat::collider::ColliderShape::Circle {
            radius: Fixed::from_num(r),
        }
    }

    /// L'AABB balayée d'un déplacement diagonal recouvre le coin d'un rectangle sans que la balle le
    /// touche : le test géométrique ne bloque pas (distance du coin à la trajectoire 5,66 > 5), il
    /// bloque dès que le rayon de la balle la dépasse.
    #[test]
    fn passage_en_coin_ne_bloque_pas() {
        let (old, new) = (v(30.0, -10.0), v(-10.0, 30.0));
        // Les deux AABB se recouvrent bien : sans test géométrique la balle serait détruite.
        let swept = combat::grid::Aabb::from_center_size(
            v(10.0, 10.0),
            Fixed::from_num(40),
            Fixed::from_num(40),
        );
        let body = combat::grid::Aabb::from_center_size(
            v(0.0, 0.0),
            Fixed::from_num(12),
            Fixed::from_num(12),
        );
        assert!(swept.overlaps(&body));
        assert!(!swept_hits_collider(
            old,
            new,
            Fixed::from_num(5),
            v(0.0, 0.0),
            &rect(12.0, 12.0)
        ));
        assert!(swept_hits_collider(
            old,
            new,
            Fixed::from_num(6),
            v(0.0, 0.0),
            &rect(12.0, 12.0)
        ));
    }

    /// Deux disques dont les boîtes se recouvrent sans contact (trajectoire diagonale à 16,97 du
    /// centre, somme des rayons 11).
    #[test]
    fn disque_en_diagonale_ne_bloque_pas() {
        let (old, new) = (v(30.0, -6.0), v(-6.0, 30.0));
        assert!(!swept_hits_collider(
            old,
            new,
            Fixed::from_num(5),
            v(0.0, 0.0),
            &disc(6.0)
        ));
        assert!(swept_hits_collider(
            old,
            new,
            Fixed::from_num(11),
            v(0.0, 0.0),
            &disc(6.0)
        ));
    }

    #[test]
    fn traversee_et_extremites_bloquent() {
        // Traverse de part en part (tunnel d'une frame) : bloqué, rectangle et disque.
        assert!(swept_hits_collider(
            v(-30.0, 0.0),
            v(30.0, 0.0),
            Fixed::from_num(2),
            v(0.0, 0.0),
            &rect(12.0, 12.0)
        ));
        assert!(swept_hits_collider(
            v(-30.0, 0.0),
            v(30.0, 0.0),
            Fixed::from_num(2),
            v(0.0, 0.0),
            &disc(6.0)
        ));
        // Arrive dans le corps.
        assert!(swept_hits_collider(
            v(-30.0, 0.0),
            v(2.0, 1.0),
            Fixed::from_num(2),
            v(0.0, 0.0),
            &rect(12.0, 12.0)
        ));
        // Passe à côté, parallèle, à 10 : rayon 2 + demi-largeur 6 = 8 < 10.
        assert!(!swept_hits_collider(
            v(-30.0, 10.0),
            v(30.0, 10.0),
            Fixed::from_num(2),
            v(0.0, 0.0),
            &rect(12.0, 12.0)
        ));
        assert!(swept_hits_collider(
            v(-30.0, 7.0),
            v(30.0, 7.0),
            Fixed::from_num(2),
            v(0.0, 0.0),
            &rect(12.0, 12.0)
        ));
        // Balle immobile (segment nul) : contact ou non selon la distance.
        assert!(swept_hits_collider(
            v(5.0, 0.0),
            v(5.0, 0.0),
            Fixed::from_num(2),
            v(0.0, 0.0),
            &disc(6.0)
        ));
        assert!(!swept_hits_collider(
            v(20.0, 0.0),
            v(20.0, 0.0),
            Fixed::from_num(2),
            v(0.0, 0.0),
            &disc(6.0)
        ));
        // Centre décalé : la géométrie est relative au centre donné.
        assert!(swept_hits_collider(
            v(70.0, 100.0),
            v(90.0, 100.0),
            Fixed::from_num(2),
            v(80.0, 100.0),
            &rect(12.0, 12.0)
        ));
    }

    /// Un tir est un événement `WeaponFired` de **cette frame** et de **ce joueur** : à la frame 0, une
    /// gâchette tenue que la cadence refuse n'émet rien et ne déclenche donc pas le familier.
    #[test]
    fn tir_du_porteur_exige_un_evenement_de_la_frame() {
        let fired = |frame, player_handle| WeaponFired {
            frame,
            player_handle,
        };
        assert!(
            !owner_fired(&[], 0, 0),
            "frame 0, tir refusé par la cadence : aucun événement"
        );
        assert!(owner_fired(&[fired(0, 0)], 0, 0));
        assert!(
            !owner_fired(&[fired(0, 1)], 0, 0),
            "le tir d'un autre joueur ne compte pas"
        );
        assert!(
            !owner_fired(&[fired(19, 0)], 0, 20),
            "un tir d'une autre frame ne compte pas"
        );
        assert!(owner_fired(&[fired(5, 1), fired(5, 0)], 0, 5));
    }
}
