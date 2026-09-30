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
//! # Décision : pas de ressource globale toujours présente
//!
//! `TimedModifier`/`CurrencyMultiplier` (Insta-Kill, Double Points) réutilisent le
//! composant [`sim_core::modifier::Modifiers`] **déjà posé sur chaque personnage** depuis
//! T1.2 (`character::create::create_character`) plutôt qu'un nouveau composant ou une
//! ressource globale : aucun scénario existant ne pousse jamais rien dans `Modifiers` pour
//! ce chantier, sa valeur ne bouge donc pas pour eux (aucune trace à blesser côté
//! `Modifiers`). `RefillAmmo`/`RepairAllWindows` mutent directement des composants déjà
//! posés partout où ils s'appliquent (`combat::inventory::AmmoReserves`, `WeaponModesState`,
//! `map::game::entity::map::window::WindowHealth`) — même raisonnement. [`PowerUpPickup`]
//! est enregistré avec `rollback_and_trace_no_checksum` (comme `character::health::HitCount`,
//! T2.9) : aucune entité existante n'en porte jamais, sa présence (valeur) ne bouge donc pas
//! pour les scénarios qui ne placent ni ne font tomber de power-up.
//!
//! **Preuve trace-diff (§8, `docs/conventions.md` §10)** : malgré ce qui précède,
//! **toutes** les traces de référence existantes ont quand même dû être blessées. La cause
//! n'est ni `PowerUpPickup` ni `Modifiers`/`AmmoReserves`/`WindowHealth` (dont la valeur ne
//! change bien pour aucun scénario préexistant, confirmé par `trace-diff.py`), mais
//! `FrameEvents<PowerUpPickedUp>` : `add_frame_events::<T>()`
//! (`sim_core::frame_events::FrameEventsAppExt`) enregistre **toujours** la file via
//! `rollback_and_trace_resource` (checksum inclus, comme `FrameEvents<DamageEvent>`,
//! `FrameEvents<CurrencyEvent>`...), jamais en variante `_no_checksum` — il n'y a pas
//! d'équivalent `add_frame_events_no_checksum`. Une `FrameEvents<T>` vide (`[]`) a beau
//! avoir une valeur strictement constante sur un scénario qui n'émet jamais cet événement,
//! c'est une **nouvelle ressource checksummée** : son enregistrement seul déplace le
//! `Checksum` GGRS agrégé de chaque frame, pour tous les scénarios, même ceux qui n'en
//! voient jamais le contenu varier — exactement le cas « la présence du nouveau type
//! change le checksum, jamais une valeur de jeu » que `docs/conventions.md` §10 documente
//! pour n'importe quel `rollback_and_trace*`. Confirmé par `trace-diff.py --ignore
//! "FrameEvents<game::powerups::PowerUpPickedUp>,PowerUpPickup"` sur `idle`,
//! `two_players_shooting`, `points_on_kill` (main vs branche) : aucune autre différence.

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
/// Rollback **sans checksum** (`rollback_and_trace_no_checksum`, voir la doc du module) :
/// nouveau composant dont aucune entité existante ne doit changer le checksum GGRS comparé
/// par le synctest/désync tant qu'aucun scénario ne place ou ne fait tomber de power-up.
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
        (&GgrsNetId, Entity, &PowerUpPickup, &fixed_math::FixedTransform3D),
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
            &mut Modifiers,
            &mut AmmoReserves,
            &WeaponInventory,
        ),
        With<Player>,
    >,
    mut weapon_modes: Query<(&Weapon, &mut WeaponModesState)>,
    mut windows: Query<(&GgrsNetId, &mut WindowHealth, Option<&mut Obstacle>), With<Rollback>>,
    enemies: Query<(&GgrsNetId, Entity, &Team, Has<Death>), With<Enemy>>,
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
        for action in &def.actions {
            match action {
                Action::TimedModifier { .. } | Action::CurrencyMultiplier { .. } => {
                    for (net_id, mut modifiers, _reserves, _inventory) in
                        order_mut_iter!(players)
                    {
                        let source =
                            ModifierSource::Named(format!("powerup:{}:{}", event.id, net_id.0));
                        if let Some(modifier) = action.as_modifier(frame.frame, source) {
                            info!(
                                "ggrs{{f={} powerup_effect powerup={} target={} kind=modifier stat={:?}}}",
                                frame.frame, event.id, net_id.0, modifier.stat
                            );
                            modifiers.push(modifier);
                        }
                    }
                }
                Action::RefillAmmo => {
                    for (net_id, _modifiers, mut reserves, inventory) in
                        order_mut_iter!(players)
                    {
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
                            let current = reserves.get(&ammo_type);
                            if full_amount > current {
                                reserves.add(ammo_type, full_amount - current);
                            }
                        }
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
                        commands
                            .entity(entity)
                            .insert(Death { last_hit_by: None });
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
    if config.powerups.is_empty() {
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
        app.rollback_and_trace_no_checksum::<PowerUpPickup>();

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
