pub mod ui;

use bevy::{
    log::{tracing::span, Level},
    prelude::*,
};
use bevy_fixed::fixed_math;
use bevy_ggrs::Rollback;
use combat::damage::{resolve_damage, Defenses};
use ggrs::PlayerHandle;
use serde::{Deserialize, Serialize};
use sim_core::damage::DamageEvent;
use sim_core::stats::StatId;
use sim_core::team::Team;
use stats::StatReader;
use std::collections::BTreeMap;
use std::fmt;
use utils::{frame::FrameCount, net_id::GgrsNetId, order_iter, order_mut_iter};

use crate::character::player::Player;
use crate::frame_events::FrameEvents;

#[derive(Component, Reflect, Debug, Clone, Hash, Serialize, Deserialize)]
pub enum HitBy {
    Entity(GgrsNetId),
    Player(PlayerHandle),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthConfig {
    pub max: fixed_math::Fixed,
    #[serde(default)]
    pub regen_rate: Option<fixed_math::Fixed>, // Health per second
    #[serde(default)]
    pub regen_delay_frames: Option<u32>, // Frames to wait after taking damage before regen starts
}

#[derive(Component, Clone, Debug, Serialize, Default, Deserialize, Hash)]
pub struct Health {
    pub current: fixed_math::Fixed,
    pub max: fixed_math::Fixed,
    pub invulnerable_until_frame: Option<u32>, // Optional invulnerability window
}

#[derive(Component, Clone, Debug, Hash, Serialize, Default, Deserialize)]
pub struct HealthRegen {
    pub last_damage_frame: u32,
    pub regen_rate: fixed_math::Fixed,
    pub regen_delay_frames: u32,
}

#[derive(Component, Clone, Debug, Hash, Serialize, Deserialize, Default)]
pub struct Death {
    pub last_hit_by: Option<Vec<HitBy>>,
}

#[derive(Component, Clone, Debug, Hash, Serialize, Deserialize, Default)]
pub struct DamageAccumulator {
    pub total_damage: fixed_math::Fixed,
    pub hit_count: u32,
    pub last_hit_by: Option<Vec<HitBy>>,
}

impl fmt::Display for HitBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HitBy::Entity(net_id) => write!(f, "NetId({})", net_id.0),
            HitBy::Player(player_handle) => write!(f, "Player({})", player_handle),
        }
    }
}

impl fmt::Display for Health {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HP: {}/{}", self.current, self.max)?;
        if let Some(frame) = self.invulnerable_until_frame {
            write!(f, " (Invulnerable until frame {})", frame)?;
        }
        Ok(())
    }
}

impl fmt::Display for Death {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.last_hit_by {
            Some(hits) if !hits.is_empty() => {
                for (i, hit_by) in hits.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", hit_by)?;
                }
                Ok(())
            }
            Some(_) | None => write!(f, "Died (cause unknown or no direct hit)"),
        }
    }
}

impl From<HealthConfig> for Health {
    fn from(value: HealthConfig) -> Self {
        Self {
            current: value.max,
            max: value.max,
            invulnerable_until_frame: None,
        }
    }
}

/// Lit `FrameEvents<DamageEvent>` (émis par les trois émetteurs : collision de balles,
/// collision de mêlée, attaque d'ennemi) dans l'ordre d'émission et applique
/// `combat::damage::resolve_damage` (équipe, tir ami, tags, résistances, immunités,
/// invulnérabilité — `Health.invulnerable_until_frame` enfin honoré) ; accumule le
/// résultat dans `DamageAccumulator` (`HitBy`/`last_hit_by` pour l'attribution des kills,
/// comme avant T1.1). Seul point d'écriture de `DamageAccumulator` : les trois émetteurs
/// n'y touchent plus directement (T1.1, chantier B1).
///
/// `RollbackSystemSet::CollisionDamage`, après les émetteurs (`Weapon`, `Projectiles`, et
/// le traducteur `enemy_attack_damage_translate_system`), avant `DeathManagement`.
pub fn rollback_resolve_damage_events(
    frame: Res<FrameCount>,
    events: Res<FrameEvents<DamageEvent>>,
    mut commands: Commands,
    net_id_query: Query<(&GgrsNetId, Entity), With<Rollback>>,
    player_handle_query: Query<(&GgrsNetId, &Player)>,
    mut target_query: Query<
        (
            &GgrsNetId,
            &Team,
            Option<&Defenses>,
            Option<&Health>,
            Option<&mut DamageAccumulator>,
        ),
        With<Rollback>,
    >,
) {
    if events.is_empty() {
        return;
    }

    let system_span = span!(Level::INFO, "ggrs", f = frame.frame, s = "resolve_damage");
    let _enter = system_span.enter();

    // GgrsNetId -> Entity, déterministe (BTreeMap) : construit une fois pour retrouver la
    // cible de chaque événement (identifiée par net_id, jamais par Entity — CLAUDE.md règle 3).
    let mut entity_by_net_id: BTreeMap<usize, Entity> = BTreeMap::new();
    for (net_id, entity) in order_iter!(net_id_query) {
        entity_by_net_id.insert(net_id.0, entity);
    }
    let mut player_handle_by_net_id: BTreeMap<usize, PlayerHandle> = BTreeMap::new();
    for (net_id, player) in order_iter!(player_handle_query) {
        player_handle_by_net_id.insert(net_id.0, player.handle);
    }

    let default_defenses = Defenses::default();

    for event in events.iter() {
        let Some(&entity) = entity_by_net_id.get(&event.target.0) else {
            continue; // cible déjà disparue (rollback, mort le même frame par un autre coup)
        };
        let Ok((target_net_id, target_team, opt_defenses, opt_health, opt_accumulator)) =
            target_query.get_mut(entity)
        else {
            continue;
        };
        // Pas de santé : rien à blesser (ex. un mur touché par erreur).
        if opt_health.is_none() {
            continue;
        }
        let invulnerable = opt_health
            .and_then(|h| h.invulnerable_until_frame)
            .is_some_and(|until| event.frame <= until);
        let defenses = opt_defenses.unwrap_or(&default_defenses);

        let Some(amount) = resolve_damage(
            event.source_team,
            *target_team,
            event.friendly_fire,
            &event.tags,
            defenses,
            event.kind.clone(),
            event.amount,
            invulnerable,
        ) else {
            continue;
        };

        let mut last_hit_by = Vec::with_capacity(2);
        if let Some(&handle) = player_handle_by_net_id.get(&event.source.0) {
            last_hit_by.push(HitBy::Player(handle));
        }
        last_hit_by.push(HitBy::Entity(event.source.clone()));

        if let Some(mut accumulator) = opt_accumulator {
            accumulator.total_damage = accumulator.total_damage.saturating_add(amount);
            accumulator.hit_count += 1;
            accumulator.last_hit_by = Some(last_hit_by);
        } else {
            commands.entity(entity).insert(DamageAccumulator {
                hit_count: 1,
                total_damage: amount,
                last_hit_by: Some(last_hit_by),
            });
        }

        info!(
            "{} <- {} dmg from {} (kind {:?}, tags {:?})",
            target_net_id, amount, event.source, event.kind, event.tags
        );
    }
}

pub fn rollback_apply_accumulated_damage(
    frame: Res<FrameCount>,
    mut commands: Commands,
    mut query: Query<
        (
            &GgrsNetId,
            Entity,
            &DamageAccumulator,
            &mut Health,
            Option<&mut HealthRegen>,
        ),
        With<Rollback>,
    >,
) {
    let system_span = span!(Level::INFO, "ggrs", f = frame.frame, s = "apply_damage");
    let _enter = system_span.enter();

    for (g_id, entity, accumulator, mut health, opt_regen) in order_mut_iter!(query) {
        if accumulator.total_damage > fixed_math::FIXED_ZERO {
            health.current = health.current.saturating_sub(accumulator.total_damage);

            info!(
                "{} receive {} dmg health is {}",
                g_id, accumulator.total_damage, health.current
            );

            // Update last damage frame for regen
            if let Some(mut regen) = opt_regen {
                regen.last_damage_frame = frame.frame;
            }

            commands.entity(entity).remove::<DamageAccumulator>();

            if health.current <= fixed_math::FIXED_ZERO {
                commands.entity(entity).insert(Death {
                    last_hit_by: accumulator.last_hit_by.clone(),
                });
            }
        }
    }
}

pub fn rollback_apply_death(
    frame: Res<FrameCount>,
    mut commands: Commands,
    query: Query<(&GgrsNetId, Entity, &Death), With<Rollback>>,
) {
    let system_span = span!(Level::INFO, "ggrs", f = frame.frame, s = "apply_death");
    let _enter = system_span.enter();

    for (id, entity, death_info) in order_iter!(query) {
        info!("{} entity killed by {}", id, death_info);

        // Despawn différé : ressuscitable en cas de rollback (voir `RollbackDespawnPlugin`)
        use bevy_ggrs::RollbackDespawnCommandExtension;
        commands.entity(entity).despawn_rollback();
    }
}

// SYSTEM: HEALTH REGENERATION
pub fn rollback_health_regeneration(
    frame: Res<FrameCount>,
    mut query: Query<(&GgrsNetId, &mut Health, &HealthRegen), With<Rollback>>,
) {
    let system_span = span!(Level::INFO, "ggrs", f = frame.frame, s = "health_regen");
    let _enter = system_span.enter();

    for (g_id, mut health, regen) in order_mut_iter!(query) {
        // Check if enough time has passed since last damage
        let frames_since_damage = frame.frame.saturating_sub(regen.last_damage_frame);

        if frames_since_damage >= regen.regen_delay_frames && health.current < health.max {
            let health_before = health.current;
            // Regenerate health (60 frames per second)
            let regen_per_frame = regen.regen_rate / fixed_math::new(60.0);
            health.current = (health.current + regen_per_frame).min(health.max);

            // Log every 60 frames (once per second) or when reaching max health
            if frame.frame % 60 == 0 || health.current >= health.max {
                info!(
                    "{} regen {} -> {} (+{}/s, {}f since dmg)",
                    g_id, health_before, health.current, regen.regen_rate, frames_since_damage
                );
            }
        }
    }
}

/// Stats branchées (T1.2, chantier B2) : `Health.max` suit la stat `MaxHealth` (base +
/// modificateurs actifs à la frame courante) et `HealthRegen.regen_rate` suit `HealthRegen`.
/// `current` est borné à `max` s'il le dépasse (perte de stat, fin d'un buff) ; jamais
/// relevé automatiquement quand `max` augmente (un soin/regen explicite s'en charge).
///
/// N'écrit rien si l'entité n'a pas la stat correspondante (`StatReader::try_get`, pas
/// `get` avec un défaut) : reprendre la valeur déjà en place comme base de `resolve()` la
/// ferait dériver à chaque frame sous un modificateur multiplicatif (`Mul`/`Pct`), au lieu
/// de la laisser simplement inchangée.
///
/// `RollbackSystemSet::Status`, après `stats::expire_modifiers_system` (voir sa doc : les
/// deux touchent `Modifiers`/l'état qui en dérive, l'`ambiguity_detection: Error` du
/// `GgrsSchedule` exige un ordre explicite entre eux).
pub fn sync_health_from_stats(
    stats: StatReader,
    mut query: Query<(&GgrsNetId, Entity, &mut Health, Option<&mut HealthRegen>), With<Rollback>>,
) {
    for (_net_id, entity, mut health, regen) in order_mut_iter!(query) {
        if let Some(max) = stats.try_get(entity, &StatId::MaxHealth) {
            health.max = max;
            if health.current > max {
                health.current = max;
            }
        }
        if let Some(mut regen) = regen {
            if let Some(rate) = stats.try_get(entity, &StatId::HealthRegen) {
                regen.regen_rate = rate;
            }
        }
    }
}
