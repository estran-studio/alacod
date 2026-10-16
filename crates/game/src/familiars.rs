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
use std::sync::Arc;

use behaviors::{
    familiar::{follow_step, orbit_position},
    Behavior, FamiliarGun, FamiliarOf,
};
use bevy::prelude::*;
use bevy_fixed::fixed_math::{self, Fixed, FixedVec2};
use bevy_ggrs::{GgrsSchedule, PlayerInputs, Rollback, RollbackDespawnCommandExtension};
use combat::{
    collider::{is_colliding, Collider, CollisionSettings},
    collision_grid::{collider_aabb, union_aabb},
    downed::Downed,
    projectile::Projectile,
    team::team_allows_hit,
    weapons::{
        bullet_rollback_collision_system, bullet_rollback_system, spawn_bullet, Bullet,
        BulletSpawn, BulletType, WeaponInventory, WeaponState, WeaponsConfig,
    },
};
use items::{Inventory, ItemTable};
use sim_core::{tag::Tags, team::Team};
use utils::{
    frame::FrameCount,
    net_id::{GgrsNetId, GgrsNetIdFactory},
    order_iter,
};

use crate::{
    balance::ResolvedBalance,
    character::{
        config::CharacterConfig, create::create_character, player::jjrs::PeerConfig, player::Player,
    },
    global_asset::GlobalAsset,
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

    /// `MirrorShot { weapon, every }`, s'il y en a un.
    pub fn mirror_shot(&self) -> Option<(&str, u32)> {
        self.0.iter().find_map(|rule| match rule {
            Behavior::MirrorShot { weapon, every } => Some((weapon.as_str(), *every)),
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
    global_assets: Res<GlobalAsset>,
    characters: Res<Assets<CharacterConfig>>,
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
                &global_assets,
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
    global_assets: &Res<GlobalAsset>,
    characters: &Res<Assets<CharacterConfig>>,
    balance: &ResolvedBalance,
    collision_settings: &CollisionSettings,
    id_factory: &mut ResMut<GgrsNetIdFactory>,
) -> Option<Entity> {
    let handle = global_assets.character_configs.get(character)?;
    let config = characters.get(handle)?;
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
        global_assets,
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

/// `MirrorShot` : quand l'arme active du porteur a tiré cette frame (`WeaponState::last_fire_frame`),
/// le familier tire **sa** arme (Q13, proposition par défaut : sans les modificateurs du porteur ni
/// ses objets) dans la direction de visée du porteur, au plus une fois tous les `every` frames.
/// Un tir par déclenchement (pas de plomb multiple), sans dispersion ni recul (aucun RNG consommé).
/// La balle porte le `GgrsNetId` et l'équipe du **porteur** (les kills lui reviennent, elle ne le
/// touche jamais) et les tags du familier.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn familiar_mirror_shot_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    inputs: Res<PlayerInputs<PeerConfig>>,
    owners: Query<
        (
            &GgrsNetId,
            &Player,
            &WeaponInventory,
            Has<Downed>,
            Option<&Team>,
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
    global_assets: Res<GlobalAsset>,
    weapons: Res<Assets<WeaponsConfig>>,
    collision_settings: Res<CollisionSettings>,
    mut id_factory: ResMut<GgrsNetIdFactory>,
) {
    if familiars.is_empty() {
        return;
    }
    let Some(weapons) = weapons.get(&global_assets.weapons) else {
        return;
    };
    for (net_id, familiar, behaviors, transform, team, tags, mut gun) in
        utils::order_mut_iter!(familiars)
    {
        let Some((weapon_id, every)) = behaviors.mirror_shot() else {
            continue;
        };
        let Some((owner_id, player, inventory, downed, owner_team)) =
            owners.iter().find(|(id, ..)| id.0 == familiar.owner.0)
        else {
            continue;
        };
        let (input, _) = inputs[player.handle];
        let fired = inventory
            .weapons
            .get(inventory.active_weapon_index)
            .and_then(|(entity, _)| weapon_states.get(*entity).ok())
            .is_some_and(|state| {
                state.last_fire_frame == frame.frame && (frame.frame > 0 || input.fire)
            });
        if downed || !fired || !gun.ready(frame.frame, every) {
            continue;
        }
        let aim = FixedVec2::new(
            Fixed::from_num(input.pan_x) / fixed_math::new(127.0),
            Fixed::from_num(input.pan_y) / fixed_math::new(127.0),
        )
        .normalize_or_zero();
        if aim == FixedVec2::ZERO {
            continue;
        }
        let Some(asset) = weapons.0.get(weapon_id) else {
            warn!(
                "familier {} : arme « {} » inconnue (voir `alacod lint`), pas de tir",
                net_id, weapon_id
            );
            continue;
        };
        let Some(mode) = asset
            .config
            .firing_modes
            .get(&asset.config.default_firing_mode)
        else {
            continue;
        };
        let (damage, speed) = match mode.bullet_type {
            BulletType::Standard { damage, speed }
            | BulletType::Explosive { damage, speed, .. }
            | BulletType::Piercing { damage, speed, .. } => (damage, speed),
        };
        let projectile = (!mode.projectile.is_empty()).then(|| {
            Projectile::new(
                asset.config.name.clone(),
                &mode.projectile,
                Arc::new(asset.config.projectiles.clone()),
                fixed_math::FIXED_ONE,
                0,
            )
        });
        let shooter_tags = tags.cloned().unwrap_or_default();
        let id_label = format!("{}", mode.bullet_type);
        spawn_bullet(
            &mut commands,
            &collision_settings,
            &mut id_factory,
            BulletSpawn {
                position: transform.translation,
                rotation: fixed_math::FixedMat3::IDENTITY,
                velocity: aim * (speed / Fixed::from_num(60)),
                bullet_type: mode.bullet_type,
                damage,
                range: mode.range,
                player_handle: player.handle,
                created_at: frame.frame,
                source: owner_id,
                source_team: team
                    .copied()
                    .or(owner_team.copied())
                    .unwrap_or(Team::Players),
                source_tags: &shooter_tags,
                friendly_fire: asset.config.friendly_fire,
                projectile,
                id_label,
                shot_size: fixed_math::FIXED_ONE,
            },
        );
        gun.last_shot = Some(frame.frame);
        info!(
            "ggrs{{f={} familiar_shot familiar={} owner={}}}",
            frame.frame, net_id.0, owner_id.0
        );
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
        for (familiar_id, _, familiar_transform, familiar_collider, team) in &blockers {
            if !team_allows_hit(
                bullet.source_team,
                **team,
                bullet.friendly_fire,
                &bullet.tags,
            ) {
                continue;
            }
            let body = collider_aabb(&familiar_transform.translation, familiar_collider);
            if swept.overlaps(&body)
                || is_colliding(
                    &transform.translation,
                    collider,
                    &familiar_transform.translation,
                    familiar_collider,
                )
            {
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
            Behavior::MirrorShot {
                weapon: "pistol".into(),
                every: 12,
            },
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
        assert_eq!(rules.mirror_shot(), Some(("pistol", 12)));
        assert!(rules.blocks_bullets());
        assert_eq!(FamiliarBehaviors::default().motion(), None);
    }
}
