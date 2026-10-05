//! Profils v1. Toutes les décisions produisent uniquement des inputs ordinaires ; portes,
//! achats, réparations et réanimations passent par les interactions de jeu existantes.
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_fixed::fixed_math::{Fixed, FixedTransform3D, FixedVec2};
use bevy_ggrs::{LocalInputs, LocalPlayers, Rollback};
use combat::{downed::Downed, inventory::AmmoReserves};
use game::balance::ResolvedBalance;
use game::character::enemy::ai::{navigation::AgentBody, EnemyAiConfig, MonsterState};
use game::character::player::input::{BoxInput, INPUT_INTERACTION, INPUT_RELOAD};
use game::character::{
    enemy::Enemy,
    health::Health,
    player::{jjrs::PeerConfig, Player},
};
use game::collider::{Collider, Wall, Window};
use game::economy::PerkMachine;
use game::interaction::{Interactable, InteractionType};
use game::replay::BotProfile;
use game::weapons::{FiringMode, WeaponInventory, WeaponModesState, WeaponPickup, WeaponState};
use map::game::entity::map::{door::DoorComponent, window::WindowHealth};
use run::{currency::Currency, perks::Perks};
use utils::{frame::FrameCount, net_id::GgrsNetId, order_iter};

use crate::{
    decide::{aim_at, set_direction_buttons},
    input::BotAssignments,
    navigation::{BotNavigation, Rect},
};

/// Vue pure de la décision v1, construite avec la navigation et les règles d'achat.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct HunterView {
    pub position: FixedVec2,
    pub target: Option<FixedVec2>,
    pub direction: FixedVec2,
    pub can_fire: bool,
    pub reload: bool,
    pub switch_weapon: bool,
    pub interact: bool,
    pub downed: bool,
}

pub fn decide_hunter(view: &HunterView) -> BoxInput {
    if view.downed {
        return BoxInput::default();
    }
    let mut input = BoxInput::default();
    if let Some(target) = view.target {
        aim_at(&mut input, view.position, target);
    }
    set_direction_buttons(&mut input, view.direction);
    input.fire = view.can_fire;
    input.switch_weapon = view.switch_weapon;
    if view.reload {
        input.buttons |= INPUT_RELOAD;
    }
    if view.interact {
        input.buttons |= INPUT_INTERACTION;
    }
    input
}

pub(crate) fn fire_trigger_ready(mode: FiringMode, is_firing: bool) -> bool {
    matches!(mode, FiringMode::Automatic { .. }) || !is_firing
}

#[derive(SystemParam)]
pub struct HunterWorld<'w, 's> {
    players: Query<
        'w,
        's,
        (
            &'static GgrsNetId,
            &'static Player,
            &'static FixedTransform3D,
            &'static Health,
            &'static WeaponInventory,
            &'static Collider,
            Option<&'static AmmoReserves>,
            Option<&'static Currency>,
            Option<&'static Perks>,
            Option<&'static Downed>,
        ),
        With<Rollback>,
    >,
    enemies: Query<
        'w,
        's,
        (
            &'static GgrsNetId,
            &'static FixedTransform3D,
            &'static EnemyAiConfig,
            &'static MonsterState,
        ),
        (With<Enemy>, With<Rollback>),
    >,
    geometry: Query<
        'w,
        's,
        (
            &'static GgrsNetId,
            &'static FixedTransform3D,
            &'static Collider,
            Option<&'static Wall>,
            Option<&'static Window>,
            Option<&'static DoorComponent>,
        ),
        With<Rollback>,
    >,
    interactions: Query<
        'w,
        's,
        (
            &'static GgrsNetId,
            &'static FixedTransform3D,
            &'static Interactable,
            Option<&'static Collider>,
            Option<&'static DoorComponent>,
            Option<&'static WeaponPickup>,
            Option<&'static PerkMachine>,
            Option<&'static WindowHealth>,
        ),
        With<Rollback>,
    >,
    weapons: Query<
        'w,
        's,
        (
            &'static GgrsNetId,
            &'static WeaponState,
            &'static WeaponModesState,
        ),
    >,
    // Prix et économie résolus (F5) : mêmes valeurs que les handlers d'interaction
    balance: Option<Res<'w, ResolvedBalance>>,
}

pub fn read_hunter_inputs(
    local_players: Res<LocalPlayers>,
    assignments: Res<BotAssignments>,
    local_inputs: Option<ResMut<LocalInputs<PeerConfig>>>,
    frame: Res<FrameCount>,
    mut nav: ResMut<BotNavigation>,
    state: HunterWorld,
) {
    if !assignments
        .0
        .values()
        .any(|p| matches!(p, BotProfile::Chasseur | BotProfile::Acheteur))
    {
        return;
    }
    let Some(mut inputs) = local_inputs else {
        return;
    };
    let enemies: Vec<_> = order_iter!(state.enemies)
        .into_iter()
        .map(|(id, t, _, _)| (id.0, t.translation.truncate()))
        .collect();
    let activation: Vec<_> = order_iter!(state.enemies)
        .into_iter()
        // An already-awake zombie may be physically stuck. Its aggro radius is
        // not a firing path and must not suppress repairs or affordable doors.
        .filter(|(_, _, _, monster)| matches!(monster, MonsterState::Idle))
        .map(|(id, t, ai, _)| (id.0, t.translation.truncate(), ai.aggro_range))
        .collect();
    let geometry: Vec<_> = order_iter!(state.geometry)
        .into_iter()
        .filter(|(_, _, _, wall, window, door)| {
            wall.is_some() || window.is_some() || door.is_some()
        })
        .map(|(_, t, c, wall, _, door)| {
            (
                Rect::collider(t.translation.truncate(), c),
                wall.is_some() || door.is_some(),
            )
        })
        .collect();
    // Inséré à l'entrée de GameLoading : absent, aucune partie n'est en cours
    let Some(resolved) = state.balance.as_deref() else {
        return;
    };
    let economy = &resolved.economy;
    let perks_config = &resolved.perks;
    let interactions = order_iter!(state.interactions);
    for (_, player, t, _, inventory, collider, reserves, wallet, perks, downed) in
        order_iter!(state.players)
    {
        if !local_players.0.contains(&player.handle) {
            continue;
        }
        let Some(profile @ (BotProfile::Chasseur | BotProfile::Acheteur)) =
            assignments.0.get(&player.handle)
        else {
            continue;
        };
        let position = t.translation.truncate();
        let mut view = HunterView {
            position,
            downed: downed.is_some(),
            ..Default::default()
        };
        if view.downed {
            inputs.0.insert(player.handle, decide_hunter(&view));
            continue;
        }
        nav.update_from(
            &geometry,
            &AgentBody::from_collider(collider),
            &enemies,
            position,
        );
        let ammunition: Vec<_> = inventory
            .weapons
            .iter()
            .map(|(entity, weapon)| {
                state
                    .weapons
                    .get(*entity)
                    .ok()
                    .and_then(|(_, s, modes)| {
                        let mode = modes.modes.get(&s.active_mode)?;
                        let config = weapon.config.firing_modes.get(&s.active_mode)?;
                        let reserve = reserves.map_or(0, |r| r.get(&weapon.config.ammo_type));
                        Some((
                            mode.mag_ammo,
                            mode.can_reload(&config.mag, reserve),
                            config.range,
                            fire_trigger_ready(config.firing_mode, s.is_firing),
                        ))
                    })
                    .unwrap_or((0, false, Fixed::ZERO, false))
            })
            .collect();
        let (ammo, reloadable, range, trigger_ready) = ammunition
            .get(inventory.active_weapon_index)
            .copied()
            .unwrap_or((0, false, Fixed::ZERO, false));
        let usable = ammo > 0 || reloadable;
        view.reload = ammo == 0 && reloadable;
        view.switch_weapon = !usable
            && ammunition
                .iter()
                .any(|(ammo, reload, ..)| *ammo > 0 || *reload);
        let all_empty = !ammunition
            .iter()
            .any(|(ammo, reload, ..)| *ammo > 0 || *reload);
        let nearest = enemies
            .iter()
            .min_by_key(|(id, p)| (position.distance(p), *id));
        let distance = nearest.map_or(Fixed::MAX, |(_, p)| position.distance(p));
        let shootable = enemies
            .iter()
            .filter(|(_, p)| {
                position.distance(p) <= range.min(Fixed::from_num(300)) && nav.visible(position, *p)
            })
            .min_by_key(|(id, p)| (position.distance(p), *id));
        view.target = shootable.or(nearest).map(|(_, p)| *p);
        // Manual/Shotgun/Burst require a release between trigger pulls. Read the
        // rollback weapon state rather than hiding a pulse counter in the bot cache.
        view.can_fire = shootable.is_some() && ammo > 0 && trigger_ready;
        let chase = nav.chase(position).or_else(|| {
            // A zombie outside its aggro radius does not break its window. Approach
            // within that radius along a physical path, even before a firing post exists.
            let mut candidates = activation.clone();
            candidates.sort_by_key(|(id, p, _)| (position.distance(p), *id));
            candidates.into_iter().find_map(|(_, p, reach)| {
                nav.approach(
                    position,
                    Rect { min: p, max: p },
                    (reach - Fixed::from_num(24)).max(Fixed::ZERO),
                )
                .map(|(direction, _)| direction)
            })
        });
        let investigation = if chase.is_none() {
            let mut candidates = enemies.clone();
            candidates.sort_by_key(|(id, p)| (position.distance(p), *id));
            candidates.into_iter().find_map(|(_, p)| {
                nav.investigate(position, p)
                    .filter(|direction| direction.length_squared() > Fixed::from_num(4))
            })
        } else {
            None
        };
        let mut desired = if distance < Fixed::from_num(110) {
            nearest.map_or(FixedVec2::ZERO, |(_, p)| {
                (position - *p).normalize_or_zero() * Fixed::from_num(32)
            })
        } else if shootable.is_some() && distance <= range.min(Fixed::from_num(240)) {
            FixedVec2::ZERO
        } else {
            chase.or(investigation).unwrap_or(FixedVec2::ZERO)
        };

        // Réanimer d'abord ; acheteur : refaire le stock, Juggernog, puis porte quand le
        // champ est inaccessible. Réparer en l'absence de menace, sans courir après une
        // fenêtre inaccessible. Tous les coûts viennent des mêmes données que les handlers.
        let mut choices = Vec::new();
        if distance > Fixed::from_num(150) || (chase.is_none() && shootable.is_none()) {
            for (id, transform, interactable, c, door, pickup, machine, window) in &interactions {
                let pos = transform.translation.truncate();
                let rect = c.map_or(Rect { min: pos, max: pos }, |c| Rect::collider(pos, c));
                let balance = wallet.map_or(0, |w| w.0);
                let priority_cost =
                    match interactable.interaction_type {
                        InteractionType::Revive => Some((0, 0)),
                        InteractionType::Weapon if *profile == BotProfile::Acheteur => pickup
                            .and_then(|p| {
                                if p.can_buy_after_frame.is_some_and(|f| frame.frame < f) {
                                    return None;
                                }
                                let owned_index = inventory
                                    .weapons
                                    .iter()
                                    .position(|(_, w)| w.config.name == p.weapon_id);
                                let owned = owned_index.is_some();
                                if let Some(index) = owned_index {
                                    let (ammo, reloadable, ..) = ammunition[index];
                                    if ammo > 0 || reloadable {
                                        return None;
                                    }
                                } else if !all_empty || (p.price.is_none() && p.mag_ammo == 0) {
                                    return None;
                                }
                                let price = p.price.map_or(0, |price| {
                                    if owned {
                                        economy.refill_price(price)
                                    } else {
                                        price
                                    }
                                });
                                (balance >= price).then_some((1, price))
                            }),
                        InteractionType::Perk if *profile == BotProfile::Acheteur => machine
                            .and_then(|m| {
                                let definition = perks_config.get(&m.perk_id)?;
                                (m.perk_id == "juggernog"
                                    && !perks.is_some_and(|p| p.has(&m.perk_id))
                                    && balance >= definition.price)
                                    .then_some((2, definition.price))
                            }),
                        InteractionType::Door
                            if *profile == BotProfile::Acheteur && chase.is_none() =>
                        {
                            door.and_then(|d| {
                                let price = d.config.cost.max(0) as u32;
                                (d.config.interactable && balance >= price).then_some((3, price))
                            })
                        }
                        // An unreachable enemy is no immediate threat. Repairing also earns
                        // the ordinary points needed to open the first paid door.
                        InteractionType::Window if enemies.is_empty() || chase.is_none() => {
                            window.filter(|w| w.current < w.max).map(|_| (4, 0))
                        }
                        _ => None,
                    };
                if let Some((priority, cost)) = priority_cost {
                    choices.push((
                        priority,
                        cost,
                        if door.is_some() {
                            nearest.map_or(position.distance(&pos), |(_, p)| p.distance(&pos))
                        } else {
                            position.distance(&pos)
                        },
                        id.0,
                        rect,
                        interactable.interaction_range,
                    ));
                }
            }
        }
        choices.sort_by_key(|c| (c.0, c.1, c.2, c.3));
        for (_, _, _, id, rect, reach) in choices {
            let Some((mut direction, _)) = nav.approach(
                position,
                rect,
                (reach - Fixed::from_num(8)).max(Fixed::ZERO),
            ) else {
                continue;
            };
            if direction == FixedVec2::ZERO {
                // Le handler choisit la surface la plus proche : ne maintenir le bouton
                // que si elle correspond au but, sinon on achèterait/réparerait autre chose.
                let selected = interactions
                    .iter()
                    .filter_map(|(id, t, i, c, ..)| {
                        let pos = t.translation.truncate();
                        let distance = c.map_or(position.distance(&pos), |c| {
                            Rect::collider(pos, c).distance(position)
                        });
                        (distance <= i.interaction_range).then_some((distance, id.0))
                    })
                    .min();
                view.interact = selected.is_some_and(|(_, selected)| selected == id);
                if !view.interact {
                    // Being within range is insufficient if another surface owns
                    // the prompt. Move closer until the ordinary handler selects
                    // this goal; otherwise try another reachable interaction.
                    let closer = (rect.distance(position) - Fixed::from_num(8)).max(Fixed::ZERO);
                    let Some((closer_direction, _)) = nav.approach(position, rect, closer) else {
                        continue;
                    };
                    direction = closer_direction;
                }
            }
            desired = direction;
            break;
        }
        view.direction = nav.safe_direction(position, desired);
        inputs.0.insert(player.handle, decide_hunter(&view));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game::character::player::input::{INPUT_RIGHT, INPUT_UP};
    #[test]
    fn manual_shotgun_and_burst_release_between_shots_while_automatic_holds() {
        for mode in [
            FiringMode::Manual {},
            FiringMode::Shotgun {
                pellet_count: 8,
                spread_angle: Fixed::ZERO,
            },
            FiringMode::Burst {
                pellets_per_shot: 3,
                cooldown_frames: 1,
            },
        ] {
            let mut held = false;
            let pulses: Vec<_> = (0..4)
                .map(|_| {
                    let input = decide_hunter(&HunterView {
                        can_fire: fire_trigger_ready(mode, held),
                        ..Default::default()
                    });
                    held = input.fire;
                    input.fire
                })
                .collect();
            assert_eq!(pulses, [true, false, true, false]);
        }
        assert!(fire_trigger_ready(FiringMode::Automatic {}, true));
    }
    #[test]
    fn hunts_while_firing_and_reloads() {
        let input = decide_hunter(&HunterView {
            target: Some(FixedVec2::new(Fixed::from_num(100), Fixed::ZERO)),
            direction: FixedVec2::new(Fixed::ONE, Fixed::ONE),
            can_fire: true,
            reload: true,
            ..Default::default()
        });
        assert!(input.fire);
        assert!(input.pan_x > 0);
        assert_eq!(input.buttons, INPUT_RIGHT | INPUT_UP | INPUT_RELOAD);
    }
    #[test]
    fn buys_and_revives_with_existing_interaction() {
        let input = decide_hunter(&HunterView {
            interact: true,
            switch_weapon: true,
            ..Default::default()
        });
        assert_eq!(input.buttons, INPUT_INTERACTION);
        assert!(input.switch_weapon);
    }
    #[test]
    fn downed_bot_does_not_interact_or_fire() {
        assert_eq!(
            decide_hunter(&HunterView {
                downed: true,
                interact: true,
                can_fire: true,
                ..Default::default()
            }),
            BoxInput::default()
        );
    }
}

#[cfg(test)]
mod profile_tests {
    use super::*;
    use crate::{decide, view::BotView};
    #[test]
    fn both_new_profiles_roundtrip_and_use_the_navigation_view() {
        let hunter = HunterView {
            interact: true,
            can_fire: true,
            ..Default::default()
        };
        let view = BotView {
            position: FixedVec2::ZERO,
            health: Fixed::from_num(100),
            health_max: Fixed::from_num(100),
            ammo: 10,
            wave: 1,
            nearest_enemy: None,
            nearest_window: None,
            portal: None,
            projectiles: vec![],
            body_radius: bevy_fixed::fixed_math::Fixed::from_num(10),
            reload: false,
            switch_weapon: false,
            trigger_ready: true,
            velocity: FixedVec2::ZERO,
            enemy_visible: false,
            route: None,
            enemy_still: false,
            revive: None,
            hunter: Some(hunter),
        };
        for profile in [BotProfile::Chasseur, BotProfile::Acheteur] {
            assert_eq!(BotProfile::parse_name(profile.name()), Some(profile));
            let ron = ron::to_string(&profile).unwrap();
            assert_eq!(ron::from_str::<BotProfile>(&ron).unwrap(), profile);
            assert_eq!(
                decide(profile, &view, &mut bevy_fixed::rng::RollbackRng::new(1)),
                decide_hunter(&hunter)
            );
        }
    }
    #[test]
    fn v0_ignores_the_new_navigation_view() {
        let view = BotView {
            position: FixedVec2::ZERO,
            health: Fixed::from_num(100),
            health_max: Fixed::from_num(100),
            ammo: 10,
            wave: 1,
            nearest_enemy: None,
            nearest_window: None,
            portal: None,
            projectiles: vec![],
            body_radius: bevy_fixed::fixed_math::Fixed::from_num(10),
            reload: false,
            switch_weapon: false,
            trigger_ready: true,
            velocity: FixedVec2::ZERO,
            enemy_visible: false,
            route: None,
            enemy_still: false,
            revive: None,
            hunter: Some(HunterView {
                can_fire: true,
                interact: true,
                ..Default::default()
            }),
        };
        for profile in [
            BotProfile::Immobile,
            BotProfile::Fonceur,
            BotProfile::Prudent,
        ] {
            assert_eq!(
                decide(profile, &view, &mut bevy_fixed::rng::RollbackRng::new(1)),
                BoxInput::default()
            );
        }
    }
}
