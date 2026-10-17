//! Boss à phases (M2-T0c, chantier D4, `docs/conventions.md` §37 et §45) : exécution des
//! contrats de `behaviors::boss`.
//!
//! [`boss_phase_system`] tourne dans `RollbackSystemSet::EnemySpawning` (après les dégâts de la
//! frame, avant l'IA) : il date l'entrée dans la phase 0 au premier tick **où le boss est
//! éveillé** (M2-T7 : un boss d'une salle dormante, `world::RoomDormant`, ne fait rien), passe à
//! la phase suivante quand `PhaseEnd` est atteinte (**au plus une transition par frame**, ordre
//! du RON), exécute les `on_enter` de la phase entrée puis les événements dus de sa `Timeline`.
//! Les règles de comportement de la phase active sont lues par l'IA (`EnemyBehaviors::active`).
//!
//! Les actions réutilisent `effects::Action` par le même exécuteur que les effets de
//! personnage (`effects_runtime::apply_action`) avec un `EffectState` jetable : `GaugeAdd` n'a
//! pas d'effet durable sur un boss (pas de jauge). `SpawnPattern` est ignoré (avertissement)
//! tant qu'un émetteur joue déjà. Actions propres à la frise (M2-T7) :
//! - `Telegraph(n)` : le prochain `SpawnPattern` de la même liste est précédé d'un télégraphe
//!   de `n` frames (l'émetteur joue `Sequence([Telegraph(n), pattern])`) ;
//! - `SpawnEnemies(id, count)` : demande de renforts ([`ReinforcementRequest`], événement
//!   transitoire), exécutée dans la même frame par `map_ldtk::game::rooms` (qui connaît la
//!   salle du boss et son flux RNG).
//!
//! [`boss_defeated_system`] (`DeathManagement`, avant le despawn) : à la mort d'un boss, émet
//! [`BossDefeated`] ; `map_ldtk::game::rooms::boss_room_clear_system` tue alors les autres
//! ennemis de sa salle typée (renforts compris), pour que la salle passe `Cleared` dans la même
//! frame.
//!
//! `BossState` : rollback, checksum et trace en variante **neutre** (seuls les boss en portent).
//! `BossPlan` (la définition) est statique, hors rollback, comme `EnemyBehaviors`.

use behaviors::{BossDef, BossDefeated, BossPhaseChanged, BossState};
use bevy::prelude::*;
use bevy_fixed::fixed_math::{Fixed, FixedTransform3D};
use bevy_ggrs::GgrsSchedule;
use combat::emitter::Emitter;
use effects::Action;
use sim_core::frame_events::{FrameEvents, FrameEventsAppExt};
use sim_core::modifier::Modifiers;
use sim_core::system_set::RollbackSystemSet;
use utils::frame::FrameCount;
use utils::net_id::GgrsNetId;
use utils::rollback::RollbackTraceApp;
use utils::{order_iter, order_mut_iter};

use crate::character::enemy::Enemy;
use crate::character::health::{Death, Health};
use crate::effects_runtime::{apply_action, spawn_pattern, EffectAssets, EffectState};

/// Définition du boss (statique, hors rollback).
#[derive(Component, Clone, Debug)]
pub struct BossPlan(pub BossDef);

/// Demande de renforts d'une frise de boss (`SpawnEnemies`, M2-T7) : `FrameEvents`
/// **transitoire**, émis par [`boss_phase_system`] et consommé dans la même frame (après lui)
/// par `map_ldtk::game::rooms::boss_reinforcements_system`, qui tire les positions dans la
/// salle du boss.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReinforcementRequest {
    pub frame: u32,
    /// `GgrsNetId` du boss qui appelle (ordre de traitement).
    pub boss: usize,
    /// Position du boss (coordonnées monde) : désigne sa salle.
    pub x: Fixed,
    pub y: Fixed,
    pub id: String,
    pub count: u32,
}

#[allow(clippy::type_complexity)]
pub fn boss_phase_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    assets: EffectAssets,
    mut changed: ResMut<FrameEvents<BossPhaseChanged>>,
    mut reinforcements: ResMut<FrameEvents<ReinforcementRequest>>,
    mut bosses: Query<
        (
            &GgrsNetId,
            Entity,
            &BossPlan,
            &mut BossState,
            &mut Health,
            &mut Modifiers,
            &FixedTransform3D,
            Has<Emitter>,
            Has<Death>,
        ),
        // M2-T7 : un boss d'une salle dormante attend l'entrée du joueur (phase 0 non datée).
        (With<Enemy>, Without<world::RoomDormant>),
    >,
) {
    let now = frame.frame;
    for (
        net_id,
        entity,
        plan,
        mut state,
        mut health,
        mut modifiers,
        transform,
        has_emitter,
        dead,
    ) in order_mut_iter!(bosses)
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
            let ratio = if health.max > Fixed::ZERO {
                health.current / health.max
            } else {
                Fixed::ZERO
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
        let position = transform.translation.truncate();
        let phase_index = state.phase as usize;
        let mut effect_state = EffectState::default();
        // Exécute une liste d'actions dans l'ordre ; `Telegraph(n)` s'applique au prochain
        // `SpawnPattern` de la liste.
        let mut run_list = |actions: &[Action], health: &mut Health, modifiers: &mut Modifiers| {
            let mut telegraph = 0;
            for action in actions {
                match action {
                    Action::Telegraph(frames) => telegraph = *frames,
                    Action::SpawnPattern { pattern, weapon } => {
                        spawn_pattern(
                            pattern,
                            weapon,
                            telegraph,
                            net_id,
                            now,
                            (entity, has_emitter),
                            &mut commands,
                            &assets,
                        );
                        telegraph = 0;
                    }
                    Action::SpawnEnemies { id, count } => {
                        info!(
                            "ggrs{{f={} boss_reinforcements net_id={} id={} count={}}}",
                            now, net_id.0, id, count
                        );
                        reinforcements.send(ReinforcementRequest {
                            frame: now,
                            boss: net_id.0,
                            x: position.x,
                            y: position.y,
                            id: id.clone(),
                            count: *count,
                        });
                    }
                    other => apply_action(
                        other,
                        net_id,
                        phase_index,
                        now,
                        health,
                        modifiers,
                        &mut effect_state,
                        None,
                        (entity, has_emitter),
                        &mut commands,
                        &assets,
                    ),
                }
            }
        };
        if entered {
            run_list(&phase.on_enter, &mut health, &mut modifiers);
        }
        if let Some(timeline) = &phase.timeline {
            for event in timeline.due(now.saturating_sub(state.entered)) {
                run_list(&event.r#do, &mut health, &mut modifiers);
            }
        }
    }
}

/// Mort d'un boss (M2-T7) : `DeathManagement`, après la pose des `Death` de la frame et avant
/// le despawn. Émet [`BossDefeated`] (trace `boss_defeated`) ; `map_ldtk::game::rooms` tue
/// ensuite (même set, avant le despawn) les autres ennemis de la salle typée du boss.
#[allow(clippy::type_complexity)]
pub fn boss_defeated_system(
    frame: Res<FrameCount>,
    mut defeated: ResMut<FrameEvents<BossDefeated>>,
    mut history: ResMut<crate::profile::RunHistory>,
    run: Option<Res<run::Run>>,
    bosses: Query<(&GgrsNetId, &FixedTransform3D), (With<BossPlan>, With<Death>)>,
) {
    for (net_id, transform) in order_iter!(bosses) {
        let position = transform.translation.truncate();
        info!(
            "ggrs{{f={} boss_defeated net_id={}}}",
            frame.frame, net_id.0
        );
        // M3-T10 : historique des boss vaincus (le nom du personnage est celui de `GgrsNetId`),
        // lu en fin de run seulement (accomplissements, `profile::RunHistory`).
        // Seulement tant que la run se joue : la simulation continue après `Ended`, un boss tué ensuite
        // (poison, explosion) n'est pas un fait de la run, et dépendrait du nombre de frames prédites.
        if run.as_deref().is_some_and(|run| run.is_playing()) {
            history.bosses_defeated.insert(net_id.1.clone());
        }
        defeated.send(BossDefeated {
            frame: frame.frame,
            net_id: net_id.0,
            x: position.x,
            y: position.y,
        });
    }
}

pub struct BossPlugin;

impl Plugin for BossPlugin {
    fn build(&self, app: &mut App) {
        use crate::character::health::{rollback_apply_death, rollback_check_defeat};
        // Le système de mort écrit `RunHistory` ; `ProfilePlugin` l'enregistre en rollback.
        app.init_resource::<crate::profile::RunHistory>()
            .add_frame_events_neutral::<BossPhaseChanged>()
            // M2-T7 : transitoires (émis et lus dans la même frame) : aucun checksum ne bouge
            // quand un boss d'un scénario existant meurt (`boss_two_phases`).
            .add_frame_events_transient::<BossDefeated>()
            .add_frame_events_transient::<ReinforcementRequest>()
            .rollback_and_trace_neutral::<BossState>()
            .add_systems(
                GgrsSchedule,
                (
                    boss_phase_system.in_set(RollbackSystemSet::EnemySpawning),
                    boss_defeated_system
                        .after(rollback_check_defeat)
                        .before(rollback_apply_death)
                        .in_set(RollbackSystemSet::DeathManagement),
                ),
            );
    }
}
