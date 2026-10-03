//! Power-ups (T2.5, chantier C1 v0, `docs/taches.md` et `docs/plan-engine.md` §5 C1) :
//! contenu RON (`PowerUpsConfig`), entité rollback ramassée au passage d'un joueur
//! ([`PowerUpPickup`]), drop à la mort d'un ennemi et application des actions
//! (`effects::Action`) à tous les joueurs.
//!
//! # Sémantique CoD (décision)
//!
//! Dans Call of Duty Zombies, aucun des cinq power-ups de référence (Insta-Kill, Double
//! Points, Max Ammo, Carpenter, Nuke) n'est individuel : un power-up ramassé par un joueur
//! bénéficie à **toute l'équipe**. [`apply_powerup_actions_system`] applique donc chaque
//! action à tous les joueurs, quel que soit celui qui a ramassé le power-up — voir la doc
//! d'`effects::Action`.
//!
//! # Trois systèmes, dans `RollbackSystemSet::Effects`
//!
//! - [`powerup_pickup_detect_system`] : ramassage au passage (portée en `Fixed`, pas une
//!   interaction maintenue comme les armes/fenêtres/perks — un power-up s'applique dès
//!   qu'un joueur marche dessus, sans bouton). Despawn immédiat (`despawn_rollback`) et
//!   émission de `FrameEvents<PowerUpPickedUp>`.
//! - [`apply_powerup_actions_system`] (`.after` le précédent) : résout chaque action de la
//!   définition ramassée, sur tous les joueurs/toutes les fenêtres/tous les ennemis selon
//!   l'action.
//! - [`powerup_expiry_system`] : durée de vie au sol expirée (`PowerUpPickup::
//!   expires_at_frame`) → despawn_rollback, comme une arme jamais ramassée ne disparaît
//!   jamais (T2.2) mais un power-up, lui, a une fenêtre limitée (CoD : ~30 s).
//!
//! Un quatrième, [`loot_drop_on_death_system`], vit dans `RollbackSystemSet::DeathManagement`
//! (`.after(rollback_apply_accumulated_damage).before(rollback_apply_death)`, même
//! contrainte que `waves::systems::wave_enemy_death_tracking_system` : voir sa doc — il lui
//! faut la position de l'entité avant qu'elle ne soit détruite).
//!
//! # Déterminisme du drop (flux RNG `loot`, T1.6)
//!
//! [`loot_drop_on_death_system`] itère les morts de la frame **triées par `GgrsNetId`**
//! (`order_iter!`) et tire dans le flux nommé `"loot"` (`bevy_fixed::rng::RngStreams`,
//! comme `"waves"`/`"weapons"`) : un tirage pour « est-ce qu'un power-up tombe » (comparé à
//! `PowerUpsConfig::drop_chance`), puis, si oui, un second tirage pondéré (même méthode que
//! `waves::systems::select_enemy_type` : somme des poids, tirage uniforme, parcours trié
//! par id de `BTreeMap`) pour choisir lequel.
//!
//! # État rollback
//!
//! `TimedModifier`/`CurrencyMultiplier` réutilisent [`Modifiers`] sur chaque joueur.
//! `RefillAmmo`/`RepairAllWindows` modifient les composants rollback existants. Le pickup
//! et sa file d'événements participent tous deux au checksum GGRS : la durée de vie et
//! l'identité du power-up doivent être couvertes par la détection des désynchronisations.
//! Les preuves avant bless sont consignées dans le rapport de tâche.

use bevy::{log::tracing::span, log::Level, prelude::*};
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_fixed::{fixed_math, rng::RngStreams};
use bevy_ggrs::{GgrsSchedule, Rollback, RollbackDespawnCommandExtension};
use combat::inventory::AmmoReserves;
use effects::Action;
use serde::{Deserialize, Serialize};
use sim_core::frame_events::{FrameEvents, FrameEventsAppExt};
use sim_core::modifier::{ModifierSource, Modifiers};
use sim_core::team::Team;
use std::collections::BTreeMap;
use utils::{
    frame::FrameCount,
    net_id::{GgrsNetId, GgrsNetIdFactory},
    order_iter, order_mut_iter,
};

use crate::character::enemy::ai::obstacle::Obstacle;
use crate::character::enemy::Enemy;
use crate::character::health::Death;
use crate::character::player::Player;
use crate::economy::PointsCredit;
use crate::global_asset::GlobalAsset;
use crate::rollback::RollbackTraceApp;
use crate::system_set::RollbackSystemSet;
use crate::weapons::{MagBulletConfig, Weapon, WeaponInventory, WeaponModesState};
use map::game::entity::map::window::WindowHealth;

/// Un power-up de `games/<jeu>/assets/items/powerups.ron` (kind de contenu `PowerUp`,
/// T2.5). `name` est seulement pour l'affichage futur (HUD, T2.12) : la clé de
/// [`PowerUpsConfig::powerups`] est l'id stable (`insta_kill`...).
#[derive(Debug, Clone, Deserialize)]
pub struct PowerUpDef {
    pub name: String,
    /// Poids relatif de tirage parmi les power-ups (`content::lint` : doit être `> 0`).
    pub weight: u32,
    /// Portée de ramassage (Fixed, unités monde) — distance centre à centre entre le
    /// joueur et l'entité au sol, voir [`powerup_pickup_detect_system`].
    pub pickup_range: fixed_math::Fixed,
    /// Durée de vie au sol avant disparition si jamais ramassé (frames).
    pub lifetime_frames: u32,
    pub actions: Vec<Action>,
}

/// Table de power-ups d'un jeu (`items/powerups.ron`, kind de contenu `PowerUp`, T2.5).
/// `drop_chance` (racine, `[0, 1]`) est la probabilité qu'un ennemi tué en laisse tomber un
/// **du tout** ; `PowerUpDef::weight` décide ensuite **lequel** parmi ceux de cette table
/// (voir la doc du module).
#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
pub struct PowerUpsConfig {
    pub drop_chance: fixed_math::Fixed,
    pub powerups: BTreeMap<String, PowerUpDef>,
}

/// Power-up au sol (T2.5). Posé par [`loot_drop_on_death_system`] (drop à la mort) ou par
/// `scenario::runner` (placement scripté déterministe, `Scenario::powerups`, voir sa doc) —
/// même point d'entrée commun, [`spawn_powerup_pickup`], que `weapons::spawn_weapon_pickup`
/// pour lâcher/armes murales.
///
/// Rollback, checksum GGRS et trace enregistrés ensemble par `rollback_and_trace`.
#[derive(Component, Debug, Clone, Hash, Serialize, Deserialize)]
pub struct PowerUpPickup {
    /// Id dans `items/powerups.ron` (clé de `PowerUpsConfig::powerups`), relu à chaque
    /// frame pour retrouver la définition complète (comme `WeaponPickup::weapon_id`).
    pub id: String,
    /// Frame à laquelle ce power-up disparaît s'il n'a pas été ramassé avant
    /// ([`powerup_expiry_system`]).
    pub expires_at_frame: u32,
}

/// Émis par [`powerup_pickup_detect_system`], résolu par [`apply_powerup_actions_system`].
/// `picked_up_by` est purement informatif (log) : les actions s'appliquent à tous les
/// joueurs quel que soit le ramasseur (voir la doc du module).
#[derive(Debug, Clone, Hash)]
pub struct PowerUpPickedUp {
    pub id: String,
    pub picked_up_by: GgrsNetId,
}

/// Point d'entrée commun pour faire apparaître un power-up au sol : drop à la mort
/// ([`loot_drop_on_death_system`]) et placement scripté de scénario
/// (`scenario::runner`, `Scenario::powerups`) — même mécanisme que
/// `weapons::spawn_weapon_pickup`, réutilisé pour lâcher/armes murales (T2.2/T2.3).
pub fn spawn_powerup_pickup(
    commands: &mut Commands,
    id: String,
    position: fixed_math::FixedVec3,
    expires_at_frame: u32,
    id_factory: &mut ResMut<GgrsNetIdFactory>,
) -> Entity {
    let transform = fixed_math::FixedTransform3D::new(
        position,
        fixed_math::FixedMat3::IDENTITY,
        fixed_math::FixedVec3::ONE,
    );
    let g_id = id_factory.next(format!("powerup_{id}"));

    commands
        .spawn((
            transform.to_bevy_transform(),
            transform,
            PowerUpPickup {
                id,
                expires_at_frame,
            },
            g_id,
        ))
        .insert(Rollback)
        .id()
}

/// Ramassage au passage (T2.5) : pas d'`Interactable`/bouton (contrairement aux armes,
/// fenêtres, perks) — un power-up s'applique dès qu'un joueur entre dans son
/// `PowerUpDef::pickup_range`. Un power-up n'est ramassé qu'une fois (despawn immédiat) ;
/// si plusieurs joueurs sont dans la portée la même frame, le premier dans l'ordre
/// `GgrsNetId` déclenche le ramassage (peu importe lequel : l'effet est le même pour tous,
/// voir la doc du module).
///
/// `RollbackSystemSet::Effects` : la spec de la tâche place l'effet du ramassage ici plutôt
/// que dans `Interaction` (réservé aux interactions à bouton, voir `interaction.rs`).
pub fn powerup_pickup_detect_system(
    frame: Res<FrameCount>,
    global_assets: Res<GlobalAsset>,
    powerup_assets: Res<Assets<PowerUpsConfig>>,
    mut commands: Commands,
    mut events: ResMut<FrameEvents<PowerUpPickedUp>>,
    pickups: Query<
        (
            &GgrsNetId,
            Entity,
            &PowerUpPickup,
            &fixed_math::FixedTransform3D,
        ),
        With<Rollback>,
    >,
    players: Query<(&GgrsNetId, &fixed_math::FixedTransform3D), With<Player>>,
) {
    let system_span = span!(Level::INFO, "ggrs", f = frame.frame, s = "powerup_pickup");
    let _enter = system_span.enter();

    let Some(config) = global_assets
        .powerups_config
        .as_ref()
        .and_then(|h| powerup_assets.get(h))
    else {
        return;
    };

    for (pickup_net_id, pickup_entity, pickup, pickup_transform) in order_iter!(pickups) {
        // À la frame d'expiration, le ramassage est déjà fermé ; le système d'expiration
        // retire ensuite l'entité, sans émettre d'action.
        if frame.frame >= pickup.expires_at_frame {
            continue;
        }
        let Some(def) = config.powerups.get(&pickup.id) else {
            continue;
        };
        let pickup_pos = fixed_math::FixedVec2::new(
            pickup_transform.translation.x,
            pickup_transform.translation.y,
        );

        let mut picked_up_by: Option<GgrsNetId> = None;
        for (player_net_id, player_transform) in order_iter!(players) {
            let player_pos = fixed_math::FixedVec2::new(
                player_transform.translation.x,
                player_transform.translation.y,
            );
            if pickup_pos.distance(&player_pos) <= def.pickup_range {
                picked_up_by = Some(player_net_id.clone());
                break;
            }
        }

        if let Some(picked_up_by) = picked_up_by {
            info!(
                "ggrs{{f={} powerup_pickup net_id={} powerup={} by={}}}",
                frame.frame, pickup_net_id.0, pickup.id, picked_up_by.0
            );
            commands.entity(pickup_entity).despawn_rollback();
            events.send(PowerUpPickedUp {
                id: pickup.id.clone(),
                picked_up_by,
            });
        }
    }
}

/// Résout chaque `effects::Action` d'un power-up ramassé (voir la doc du module : tous les
/// joueurs, jamais seulement le ramasseur). `.after(powerup_pickup_detect_system)`, même
/// `RollbackSystemSet::Effects`.
#[allow(clippy::too_many_arguments)]
pub fn apply_powerup_actions_system(
    frame: Res<FrameCount>,
    events: Res<FrameEvents<PowerUpPickedUp>>,
    global_assets: Res<GlobalAsset>,
    powerup_assets: Res<Assets<PowerUpsConfig>>,
    mut players: Query<
        (
            &GgrsNetId,
            &Player,
            Has<Death>,
            &mut Modifiers,
            &mut AmmoReserves,
            &mut WeaponInventory,
        ),
        With<Player>,
    >,
    mut weapon_modes: Query<(&Weapon, &mut WeaponModesState)>,
    mut windows: Query<(&GgrsNetId, &mut WindowHealth, Option<&mut Obstacle>), With<Rollback>>,
    enemies: Query<(&GgrsNetId, Entity, &Team, Has<Death>), With<Enemy>>,
    mut points_credits: ResMut<FrameEvents<PointsCredit>>,
    mut commands: Commands,
) {
    if events.is_empty() {
        return;
    }
    let system_span = span!(Level::INFO, "ggrs", f = frame.frame, s = "powerup_apply");
    let _enter = system_span.enter();

    let Some(config) = global_assets
        .powerups_config
        .as_ref()
        .and_then(|h| powerup_assets.get(h))
    else {
        return;
    };

    for event in events.iter() {
        let Some(def) = config.powerups.get(&event.id) else {
            continue;
        };
        let source = ModifierSource::Named(format!("powerup:{}", event.id));
        // D18 : rafraîchissement, comme CoD. Un power-up déjà actif ramassé à nouveau
        // remplace ses modificateurs (la durée recommence) au lieu de les cumuler (Double
        // Points × Double Points ferait × 4). Retrait une fois par ramassage, avant toutes
        // les actions : un power-up à plusieurs actions modificatrices garde chacune.
        if def.actions.iter().any(|action| {
            matches!(
                action,
                Action::TimedModifier { .. } | Action::CurrencyMultiplier { .. }
            )
        }) {
            for (_net_id, _player, _dead, mut modifiers, _reserves, _inventory) in
                order_mut_iter!(players)
            {
                modifiers.remove_by_source(&source);
            }
        }
        for action in &def.actions {
            match action {
                Action::TimedModifier { .. } | Action::CurrencyMultiplier { .. } => {
                    for (net_id, _player, _dead, mut modifiers, _reserves, _inventory) in
                        order_mut_iter!(players)
                    {
                        if let Some(modifier) = action.as_modifier(frame.frame, source.clone()) {
                            info!(
                                "ggrs{{f={} powerup_effect powerup={} target={} kind=modifier stat={:?}}}",
                                frame.frame, event.id, net_id.0, modifier.stat
                            );
                            modifiers.push(modifier);
                        }
                    }
                }
                Action::RefillAmmo => {
                    for (net_id, _player, _dead, _modifiers, mut reserves, mut inventory) in
                        order_mut_iter!(players)
                    {
                        let mut capacities = BTreeMap::<sim_core::ammo::AmmoType, u32>::new();
                        for (weapon_entity, _weapon) in &inventory.weapons {
                            let Ok((weapon, mut modes_state)) =
                                weapon_modes.get_mut(*weapon_entity)
                            else {
                                continue;
                            };
                            for (mode_name, mode_state) in modes_state.modes.iter_mut() {
                                let Some(mode_config) = weapon.config.firing_modes.get(mode_name)
                                else {
                                    continue;
                                };
                                mode_state.mag_ammo = match mode_config.mag {
                                    MagBulletConfig::Mag { mag_size, .. } => mag_size,
                                    MagBulletConfig::Magless { bullet_limit } => bullet_limit,
                                };
                            }
                            // `default_mode_ammo_contribution` prend un `&WeaponAsset` (le
                            // type du registre) ; `weapon` ici est un `Weapon` (composant,
                            // mêmes champs) — clone ponctuel plutôt qu'élargir la signature
                            // d'une fonction partagée par `create_player`/`scenario::runner`
                            // (même décision que `interaction::handle_weapon_pickup_interaction`,
                            // voir sa doc, pour une arme murale déjà possédée).
                            let weapon_asset = crate::weapons::WeaponAsset {
                                config: weapon.config.clone(),
                                sprite_config: weapon.sprite_config.clone(),
                            };
                            let (ammo_type, full_amount) =
                                crate::weapons::default_mode_ammo_contribution(&weapon_asset);
                            let capacity = capacities.entry(ammo_type).or_default();
                            *capacity = capacity.saturating_add(full_amount);
                        }
                        for (ammo_type, full_amount) in capacities {
                            let current = reserves.get(&ammo_type);
                            if full_amount > current {
                                reserves.add(ammo_type, full_amount - current);
                            }
                        }
                        // Un rechargement entamé ne doit pas retirer un chargeur de la
                        // réserve après le remplissage instantané.
                        inventory.clear_reloading();
                        info!(
                            "ggrs{{f={} powerup_effect powerup={} target={} kind=refill_ammo}}",
                            frame.frame, event.id, net_id.0
                        );
                    }
                }
                Action::RepairAllWindows => {
                    for (net_id, mut health, obstacle) in order_mut_iter!(windows) {
                        health.current = health.max;
                        health.can_repair_after_frame = None;
                        if let Some(mut obstacle) = obstacle {
                            obstacle.blocks_movement = true;
                            if obstacle.health.is_some() {
                                obstacle.health = obstacle.max_health;
                            }
                        }
                        info!(
                            "ggrs{{f={} powerup_effect powerup={} target={} kind=repair_window}}",
                            frame.frame, event.id, net_id.0
                        );
                    }
                }
                Action::KillAllWaveEnemies => {
                    for (net_id, entity, team, already_dead) in order_iter!(enemies) {
                        if *team != Team::Enemies || already_dead {
                            continue;
                        }
                        info!(
                            "ggrs{{f={} powerup_effect powerup={} target={} kind=nuke}}",
                            frame.frame, event.id, net_id.0
                        );
                        commands.entity(entity).insert(Death { last_hit_by: None });
                    }
                    // D17 : points au ramassage, un crédit par joueur vivant (pas par ennemi
                    // tué), montant `EconomyConfig::nuke_points` résolu par
                    // `economy::award_points_system` en fin de frame.
                    for (net_id, player, dead, _modifiers, _reserves, _inventory) in
                        order_iter!(players)
                    {
                        if dead {
                            continue;
                        }
                        info!(
                            "ggrs{{f={} powerup_effect powerup={} target={} kind=nuke_points}}",
                            frame.frame, event.id, net_id.0
                        );
                        points_credits.send(PointsCredit::Nuke {
                            handle: player.handle,
                        });
                    }
                }
            }
        }
    }
}

/// Power-up au sol dont la durée de vie est écoulée sans avoir été ramassé (T2.5, même idée
/// qu'une arme murale n'est jamais consommée mais, contrairement à elle, un power-up a une
/// fenêtre limitée — CoD : environ 30 s).
pub fn powerup_expiry_system(
    frame: Res<FrameCount>,
    mut commands: Commands,
    pickups: Query<(&GgrsNetId, Entity, &PowerUpPickup), With<Rollback>>,
) {
    for (net_id, entity, pickup) in order_iter!(pickups) {
        if frame.frame >= pickup.expires_at_frame {
            info!(
                "ggrs{{f={} powerup_expire net_id={} powerup={}}}",
                frame.frame, net_id.0, pickup.id
            );
            commands.entity(entity).despawn_rollback();
        }
    }
}

/// Drop à la mort d'un ennemi (T2.5) : tirage dans le flux RNG nommé `"loot"`
/// (`RngStreams`, T1.6), consommé **dans l'ordre des `GgrsNetId`** pour les morts
/// simultanées d'une même frame — déterministe en rollback (voir la doc du module).
/// `RollbackSystemSet::DeathManagement`, `.after(rollback_apply_accumulated_damage)`
/// (`Death` est posée là, et aussi par `apply_powerup_actions_system::KillAllWaveEnemies`
/// plus tôt dans la frame, `RollbackSystemSet::Effects` — les deux sont visibles ici via
/// `Added<Death>`), `.before(rollback_apply_death)` (il faut la position de l'entité avant
/// qu'elle ne soit détruite, même contrainte que
/// `waves::systems::wave_enemy_death_tracking_system`).
///
/// Portée : équipe `Enemies` (pas seulement `WaveEnemy`, absent des personnages de
/// laboratoire du testbed placés par `CharacterSpawn` — voir le rapport de la tâche T2.5,
/// décision « portée de KillAllWaveEnemies/du drop »).
pub fn loot_drop_on_death_system(
    frame: Res<FrameCount>,
    mut rng_streams: ResMut<RngStreams>,
    global_assets: Res<GlobalAsset>,
    powerup_assets: Res<Assets<PowerUpsConfig>>,
    mut id_factory: ResMut<GgrsNetIdFactory>,
    mut commands: Commands,
    dying: Query<
        (&GgrsNetId, &Team, &fixed_math::FixedTransform3D),
        (With<Rollback>, Added<Death>),
    >,
) {
    let Some(config) = global_assets
        .powerups_config
        .as_ref()
        .and_then(|h| powerup_assets.get(h))
    else {
        return;
    };
    if config.powerups.is_empty() || config.drop_chance <= fixed_math::FIXED_ZERO {
        return;
    }

    for (net_id, team, transform) in order_iter!(dying) {
        if *team != Team::Enemies {
            continue;
        }

        let drop_roll = rng_streams.get_mut("loot").next_fixed();
        if drop_roll >= config.drop_chance {
            continue;
        }

        let total_weight: u32 = config.powerups.values().map(|def| def.weight).sum();
        if total_weight == 0 {
            continue;
        }
        let pick = rng_streams.get_mut("loot").next_u32_range(0, total_weight);
        let mut cumulative = 0u32;
        let mut chosen: Option<(&String, &PowerUpDef)> = None;
        for (id, def) in config.powerups.iter() {
            cumulative += def.weight;
            if pick < cumulative {
                chosen = Some((id, def));
                break;
            }
        }
        let Some((id, def)) = chosen else {
            continue;
        };

        info!(
            "ggrs{{f={} powerup_drop enemy_net_id={} powerup={}}}",
            frame.frame, net_id.0, id
        );

        let expires_at_frame = frame.frame.saturating_add(def.lifetime_frames);
        spawn_powerup_pickup(
            &mut commands,
            id.clone(),
            transform.translation,
            expires_at_frame,
            &mut id_factory,
        );
    }
}

pub struct PowerUpsPlugin;

impl Plugin for PowerUpsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<PowerUpsConfig>::new(&["ron"]));
        app.add_frame_events::<PowerUpPickedUp>();
        app.rollback_and_trace::<PowerUpPickup>();

        app.add_systems(
            GgrsSchedule,
            (
                powerup_pickup_detect_system,
                apply_powerup_actions_system.after(powerup_pickup_detect_system),
                powerup_expiry_system.after(apply_powerup_actions_system),
            )
                .in_set(RollbackSystemSet::Effects),
        );

        app.add_systems(
            GgrsSchedule,
            loot_drop_on_death_system
                .after(crate::character::health::rollback_apply_accumulated_damage)
                .before(crate::character::health::rollback_apply_death)
                .in_set(RollbackSystemSet::DeathManagement),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use sim_core::{ammo::AmmoType, modifier::ModifierOp, stats::StatId};

    fn world_with_config(action: Action, drop_chance: fixed_math::Fixed) -> World {
        let mut assets = Assets::<PowerUpsConfig>::default();
        let handle = assets.add(PowerUpsConfig {
            drop_chance,
            powerups: BTreeMap::from([(
                "test".to_string(),
                PowerUpDef {
                    name: "test".to_string(),
                    weight: 1,
                    pickup_range: fixed_math::Fixed::from_num(30),
                    lifetime_frames: 60,
                    actions: vec![action],
                },
            )]),
        });
        let mut world = World::new();
        world.init_resource::<bevy_ggrs::RollbackOrdered>();
        world.insert_resource(assets);
        world.insert_resource(GlobalAsset {
            spritesheets: Default::default(),
            animations: Default::default(),
            character_configs: Default::default(),
            weapons: Default::default(),
            melee_weapons: Default::default(),
            slash_effect_spritesheet: Default::default(),
            slash_effect_animation: Default::default(),
            wave_config: None,
            economy_config: None,
            perks_config: None,
            powerups_config: Some(handle),
        });
        world.insert_resource(FrameCount { frame: 10 });
        world.insert_resource(FrameEvents::<PowerUpPickedUp>::default());
        world.insert_resource(FrameEvents::<PointsCredit>::default());
        world.insert_resource(RngStreams::new(123456));
        world.insert_resource(GgrsNetIdFactory::default());
        world
    }

    fn add_pickup(world: &mut World, expires_at_frame: u32) -> Entity {
        world
            .spawn((
                Rollback,
                GgrsNetId(100, "pickup".to_string()),
                PowerUpPickup {
                    id: "test".to_string(),
                    expires_at_frame,
                },
                fixed_math::FixedTransform3D::IDENTITY,
            ))
            .id()
    }

    #[test]
    fn un_seul_ramasseur_avec_le_plus_petit_net_id() {
        let mut world = world_with_config(Action::RefillAmmo, fixed_math::FIXED_ZERO);
        for id in [20, 10] {
            world.spawn((
                GgrsNetId(id, "player".to_string()),
                Player::default(),
                fixed_math::FixedTransform3D::IDENTITY,
            ));
        }
        let pickup = add_pickup(&mut world, 11);
        world.run_system_once(powerup_pickup_detect_system).unwrap();
        let events = world.resource::<FrameEvents<PowerUpPickedUp>>();
        assert_eq!(events.iter().count(), 1);
        assert_eq!(events.iter().next().unwrap().picked_up_by.0, 10);
        assert!(world.get_entity(pickup).is_err());
    }

    #[test]
    fn expiration_ferme_le_ramassage_a_la_frame_exacte() {
        let mut world = world_with_config(Action::RefillAmmo, fixed_math::FIXED_ZERO);
        world.spawn((
            GgrsNetId(1, "player".to_string()),
            Player::default(),
            fixed_math::FixedTransform3D::IDENTITY,
        ));
        let pickup = add_pickup(&mut world, 10);
        world.run_system_once(powerup_pickup_detect_system).unwrap();
        assert!(world.resource::<FrameEvents<PowerUpPickedUp>>().is_empty());
        world.run_system_once(powerup_expiry_system).unwrap();
        assert!(world.get_entity(pickup).is_err());
    }

    #[test]
    fn effet_temporise_applique_a_tous_les_joueurs() {
        let mut world = world_with_config(
            Action::TimedModifier {
                stat: StatId::Damage,
                op: ModifierOp::Set,
                value: fixed_math::Fixed::from_num(100),
                frames: 60,
            },
            fixed_math::FIXED_ZERO,
        );
        let players: Vec<_> = [20, 10]
            .into_iter()
            .map(|id| {
                world
                    .spawn((
                        GgrsNetId(id, "player".to_string()),
                        Player::default(),
                        Modifiers::default(),
                        AmmoReserves::default(),
                        WeaponInventory::default(),
                    ))
                    .id()
            })
            .collect();
        world
            .resource_mut::<FrameEvents<PowerUpPickedUp>>()
            .send(PowerUpPickedUp {
                id: "test".to_string(),
                picked_up_by: GgrsNetId(10, "player".to_string()),
            });
        world.run_system_once(apply_powerup_actions_system).unwrap();
        for entity in players {
            let modifiers = world.get::<Modifiers>(entity).unwrap();
            assert_eq!(modifiers.0.len(), 1);
            let modifier = &modifiers.0[0];
            assert_eq!(
                modifier.source,
                ModifierSource::Named("powerup:test".to_string())
            );
            assert_eq!(modifier.until, Some(70));
            assert!(!modifier.is_expired(70));
            assert!(modifier.is_expired(71));
        }
    }

    #[test]
    fn meme_power_up_rafraichit_sans_cumuler() {
        let mut world = world_with_config(
            Action::CurrencyMultiplier {
                factor: fixed_math::Fixed::from_num(2),
                frames: 60,
            },
            fixed_math::FIXED_ZERO,
        );
        // Un modificateur d'une autre source (perk) ne doit pas être touché.
        let perk = sim_core::modifier::Modifier {
            stat: StatId::MaxHealth,
            op: ModifierOp::Mul,
            value: fixed_math::Fixed::from_num(2),
            source: ModifierSource::Named("perk:juggernog".to_string()),
            until: None,
        };
        let player = world
            .spawn((
                GgrsNetId(1, "player".to_string()),
                Player::default(),
                Modifiers(vec![perk.clone()]),
                AmmoReserves::default(),
                WeaponInventory::default(),
            ))
            .id();
        let pickup = || PowerUpPickedUp {
            id: "test".to_string(),
            picked_up_by: GgrsNetId(1, "player".to_string()),
        };

        world
            .resource_mut::<FrameEvents<PowerUpPickedUp>>()
            .send(pickup());
        world.run_system_once(apply_powerup_actions_system).unwrap();

        // Second ramassage 30 frames plus tard : la durée recommence, un seul ×2.
        world.resource_mut::<FrameCount>().frame = 40;
        world.insert_resource(FrameEvents::<PowerUpPickedUp>::default());
        world
            .resource_mut::<FrameEvents<PowerUpPickedUp>>()
            .send(pickup());
        world.run_system_once(apply_powerup_actions_system).unwrap();

        let modifiers = world.get::<Modifiers>(player).unwrap();
        assert_eq!(modifiers.0.len(), 2, "{:?}", modifiers.0);
        assert_eq!(modifiers.0[0], perk);
        let powerup = &modifiers.0[1];
        assert_eq!(
            powerup.source,
            ModifierSource::Named("powerup:test".to_string())
        );
        assert_eq!(powerup.value, fixed_math::Fixed::from_num(2));
        assert_eq!(powerup.until, Some(100));
        assert_eq!(
            sim_core::modifier::resolve(
                fixed_math::FIXED_ONE,
                modifiers.0.iter().filter(|m| {
                    m.stat == StatId::Custom(effects::CURRENCY_MULTIPLIER_STAT.to_string())
                }),
                40,
            ),
            fixed_math::Fixed::from_num(2)
        );
    }

    #[test]
    fn nuke_credite_chaque_joueur_vivant_une_fois() {
        let mut world = world_with_config(Action::KillAllWaveEnemies, fixed_math::FIXED_ZERO);
        // Deux joueurs vivants (handles 1 et 0, net_ids dans l'ordre inverse) et un joueur
        // déjà mort cette frame : seuls les vivants sont crédités, dans l'ordre net_id.
        for (id, handle, dead) in [(20, 0, false), (10, 1, false), (30, 2, true)] {
            let mut player = world.spawn((
                GgrsNetId(id, "player".to_string()),
                Player {
                    handle,
                    ..Default::default()
                },
                Modifiers::default(),
                AmmoReserves::default(),
                WeaponInventory::default(),
            ));
            if dead {
                player.insert(Death { last_hit_by: None });
            }
        }
        // Trois ennemis : le crédit ne dépend pas du nombre d'ennemis tués.
        let enemies: Vec<_> = [40, 41, 42]
            .into_iter()
            .map(|id| {
                world
                    .spawn((GgrsNetId(id, "enemy".to_string()), Enemy {}, Team::Enemies))
                    .id()
            })
            .collect();
        world
            .resource_mut::<FrameEvents<PowerUpPickedUp>>()
            .send(PowerUpPickedUp {
                id: "test".to_string(),
                picked_up_by: GgrsNetId(20, "player".to_string()),
            });
        world.run_system_once(apply_powerup_actions_system).unwrap();
        let credited: Vec<_> = world
            .resource::<FrameEvents<PointsCredit>>()
            .iter()
            .map(|credit| match credit {
                PointsCredit::Nuke { handle } => *handle,
                other => panic!("crédit inattendu : {other:?}"),
            })
            .collect();
        assert_eq!(credited, vec![1, 0]);
        for enemy in enemies {
            assert!(world.get::<Death>(enemy).is_some());
        }
    }

    #[test]
    fn max_ammo_additionne_les_reserves_et_annule_le_rechargement() {
        let mut world = world_with_config(Action::RefillAmmo, fixed_math::FIXED_ZERO);
        let weapons: crate::weapons::WeaponsConfig = ron::from_str(include_str!(
            "../../../games/testbed/assets/ZombieShooter/Sprites/Character/weapons.ron"
        ))
        .unwrap();
        let mut inventory = WeaponInventory {
            reloading_ending_frame: Some(100),
            ..Default::default()
        };
        for id in ["machine_gun", "pistol"] {
            let mut weapon = Weapon::from(weapons.0.get(id).unwrap().clone());
            weapon.config.ammo_type = AmmoType::Balle;
            let entity = world
                .spawn((
                    weapon.clone(),
                    WeaponModesState {
                        modes: BTreeMap::from([(
                            "default".to_string(),
                            crate::weapons::WeaponModeState::default(),
                        )]),
                    },
                ))
                .id();
            inventory.weapons.push((entity, weapon));
        }
        let player = world
            .spawn((
                GgrsNetId(1, "player".to_string()),
                Player::default(),
                Modifiers::default(),
                AmmoReserves::default(),
                inventory,
            ))
            .id();
        world
            .resource_mut::<FrameEvents<PowerUpPickedUp>>()
            .send(PowerUpPickedUp {
                id: "test".to_string(),
                picked_up_by: GgrsNetId(1, "player".to_string()),
            });
        world.run_system_once(apply_powerup_actions_system).unwrap();
        assert_eq!(
            world
                .get::<AmmoReserves>(player)
                .unwrap()
                .get(&AmmoType::Balle),
            288
        );
        let inventory = world.get::<WeaponInventory>(player).unwrap();
        assert!(!inventory.is_reloading());
        for (entity, weapon) in &inventory.weapons {
            let mode = &world.get::<WeaponModesState>(*entity).unwrap().modes["default"];
            assert_eq!(
                mode.mag_ammo,
                match weapon.config.firing_modes["default"].mag {
                    MagBulletConfig::Mag { mag_size, .. } => mag_size,
                    MagBulletConfig::Magless { bullet_limit } => bullet_limit,
                }
            );
        }
    }

    #[test]
    fn chance_nulle_ne_cree_pas_le_flux_loot() {
        let mut world = world_with_config(Action::RefillAmmo, fixed_math::FIXED_ZERO);
        world.spawn((
            Rollback,
            GgrsNetId(1, "enemy".to_string()),
            Team::Enemies,
            fixed_math::FixedTransform3D::IDENTITY,
            Death { last_hit_by: None },
        ));
        let before = world.resource::<RngStreams>().clone();
        world.run_system_once(loot_drop_on_death_system).unwrap();
        assert_eq!(*world.resource::<RngStreams>(), before);
        assert_eq!(world.query::<&PowerUpPickup>().iter(&world).count(), 0);
    }

    #[test]
    fn morts_simultanees_tirent_dans_loot_par_net_id() {
        let mut world = world_with_config(Action::RefillAmmo, fixed_math::FIXED_ONE);
        world
            .resource_mut::<RngStreams>()
            .get_mut("waves")
            .next_u32();
        let waves_before = *world.resource::<RngStreams>().get("waves").unwrap();
        for id in [20, 10] {
            let mut transform = fixed_math::FixedTransform3D::IDENTITY;
            transform.translation.x = fixed_math::Fixed::from_num(id);
            world.spawn((
                Rollback,
                GgrsNetId(id, "enemy".to_string()),
                Team::Enemies,
                transform,
                Death { last_hit_by: None },
            ));
        }
        world.run_system_once(loot_drop_on_death_system).unwrap();
        let mut drops: Vec<_> = world
            .query::<(&GgrsNetId, &PowerUpPickup, &fixed_math::FixedTransform3D)>()
            .iter(&world)
            .map(|(id, pickup, transform)| (id.0, pickup.expires_at_frame, transform.translation.x))
            .collect();
        drops.sort_by_key(|drop| drop.0);
        assert_eq!(drops.len(), 2);
        assert_eq!(drops[0].2, fixed_math::Fixed::from_num(10));
        assert_eq!(drops[1].2, fixed_math::Fixed::from_num(20));
        assert!(drops.iter().all(|drop| drop.1 == 70));
        let streams = world.resource::<RngStreams>();
        assert_eq!(*streams.get("waves").unwrap(), waves_before);
        let mut expected = RngStreams::new(123456);
        for _ in 0..2 {
            expected.get_mut("loot").next_fixed();
            expected.get_mut("loot").next_u32_range(0, 1);
        }
        assert_eq!(streams.get("loot"), expected.get("loot"));
    }
}
