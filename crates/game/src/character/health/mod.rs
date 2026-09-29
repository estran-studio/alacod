
pub mod ui;

use bevy::{log::{tracing::span, Level}, prelude::*};
use bevy_fixed::fixed_math;
use bevy_ggrs::Rollback;
use ggrs::PlayerHandle;
use serde::{Deserialize, Serialize};
use std::fmt;
use utils::{frame::FrameCount, net_id::GgrsNetId, order_iter, order_mut_iter};

#[derive(Component, Reflect, Debug, Clone, Serialize, Deserialize)]
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

#[derive(Component, Clone, Debug, Serialize, Default, Deserialize)]
pub struct HealthRegen {
    pub last_damage_frame: u32,
    pub regen_rate: fixed_math::Fixed,
    pub regen_delay_frames: u32,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize, Default)]
pub struct Death {
    pub last_hit_by: Option<Vec<HitBy>>,
}

#[derive(Component, Clone, Serialize, Deserialize, Default)]
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


pub fn rollback_apply_accumulated_damage(
    frame: Res<FrameCount>,
    mut commands: Commands,
    mut query: Query<(&GgrsNetId, Entity, &DamageAccumulator, &mut Health, Option<&mut HealthRegen>), With<Rollback>>,
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

        // Despawn the rollback entity
        commands.entity(entity).despawn();
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
