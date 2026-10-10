//! Boss à phases (M2-T0c, chantier D4, `docs/conventions.md` §37) : exécution des contrats de
//! `behaviors::boss`.
//!
//! [`boss_phase_system`] tourne dans `RollbackSystemSet::EnemySpawning` (après les dégâts de la
//! frame, avant l'IA) : il date l'entrée dans la phase 0 au premier tick, passe à la phase
//! suivante quand `PhaseEnd` est atteinte (**au plus une transition par frame**, ordre du RON),
//! exécute les `on_enter` de la phase entrée puis les événements dus de sa `Timeline`. Les
//! règles de comportement de la phase active sont lues par l'IA (`EnemyBehaviors::active`).
//!
//! Les actions réutilisent `effects::Action` par le même exécuteur que les effets de
//! personnage (`effects_runtime::apply_action`) avec un `EffectState` jetable : `GaugeAdd` n'a
//! pas d'effet durable sur un boss (pas de jauge). `SpawnPattern` est ignoré (avertissement)
//! tant qu'un émetteur joue déjà.
//!
//! `BossState` : rollback, checksum et trace en variante **neutre** (seuls les boss en portent).
//! `BossPlan` (la définition) est statique, hors rollback, comme `EnemyBehaviors`.

use behaviors::{BossDef, BossPhaseChanged, BossState};
use bevy::prelude::*;
use bevy_ggrs::GgrsSchedule;
use combat::emitter::Emitter;
use sim_core::frame_events::{FrameEvents, FrameEventsAppExt};
use sim_core::modifier::Modifiers;
use sim_core::system_set::RollbackSystemSet;
use utils::frame::FrameCount;
use utils::net_id::GgrsNetId;
use utils::order_mut_iter;
use utils::rollback::RollbackTraceApp;

use crate::character::enemy::Enemy;
use crate::character::health::{Death, Health};
use crate::effects_runtime::{apply_action, EffectAssets, EffectState};

/// Définition du boss (statique, hors rollback).
#[derive(Component, Clone, Debug)]
pub struct BossPlan(pub BossDef);

#[allow(clippy::type_complexity)]
pub fn boss_phase_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    assets: EffectAssets,
    mut changed: ResMut<FrameEvents<BossPhaseChanged>>,
    mut bosses: Query<
        (
            &GgrsNetId,
            Entity,
            &BossPlan,
            &mut BossState,
            &mut Health,
            &mut Modifiers,
            Has<Emitter>,
            Has<Death>,
        ),
        With<Enemy>,
    >,
) {
    let now = frame.frame;
    for (net_id, entity, plan, mut state, mut health, mut modifiers, has_emitter, dead) in
        order_mut_iter!(bosses)
    {
        let def = &plan.0;
        if dead || def.phases.is_empty() {
            continue;
        }
        let mut entered = false;
        if !state.started {
            state.started = true;
            state.phase = 0;
            state.entered = now;
            entered = true;
        } else {
            let ratio = if health.max > bevy_fixed::fixed_math::Fixed::ZERO {
                health.current / health.max
            } else {
                bevy_fixed::fixed_math::Fixed::ZERO
            };
            if let Some(next) = def.next_phase(&state, ratio, now) {
                let from = state.phase;
                state.phase = next;
                state.entered = now;
                entered = true;
                info!(
                    "ggrs{{f={} boss_phase net_id={} from={} to={}}}",
                    now, net_id.0, from, next
                );
                changed.send(BossPhaseChanged {
                    frame: now,
                    net_id: net_id.0,
                    from,
                    to: next,
                });
            }
        }
        let Some(phase) = def.phases.get(state.phase as usize) else {
            continue;
        };
        let mut effect_state = EffectState::default();
        let mut run = |action: &effects::Action,
                       index: usize,
                       health: &mut Health,
                       modifiers: &mut Modifiers| {
            apply_action(
                action,
                net_id,
                index,
                now,
                health,
                modifiers,
                &mut effect_state,
                None,
                (entity, has_emitter),
                &mut commands,
                &assets,
            );
        };
        if entered {
            for action in &phase.on_enter {
                run(action, state.phase as usize, &mut health, &mut modifiers);
            }
        }
        if let Some(timeline) = &phase.timeline {
            for event in timeline.due(now.saturating_sub(state.entered)) {
                for action in &event.r#do {
                    run(action, state.phase as usize, &mut health, &mut modifiers);
                }
            }
        }
    }
}

pub struct BossPlugin;

impl Plugin for BossPlugin {
    fn build(&self, app: &mut App) {
        app.add_frame_events_neutral::<BossPhaseChanged>()
            .rollback_and_trace_neutral::<BossState>()
            .add_systems(
                GgrsSchedule,
                boss_phase_system.in_set(RollbackSystemSet::EnemySpawning),
            );
    }
}
