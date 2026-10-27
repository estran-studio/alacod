//! Plugin de jeu de la preuve X-1 (`docs/conventions.md` §89) : un behavior, une condition et une
//! action `Custom`, un kind de contenu (`Crate`), une règle de lint, un composant rollback.

use bevy::prelude::*;
use bevy_fixed::fixed_math::Fixed;
use game::character::player::Player;
use game::extension::{
    custom_actions, BehaviorMotion, Carrier, CustomBehaviorImpl, CustomMotionView, CustomParams,
    Extension, GamePlugin, SelectionContext, Trigger,
};
use game::frame_events::FrameEvents;
use game::system_set::RollbackSystemSet;
use serde::Deserialize;
use sim_core::modifier::{Modifier, ModifierOp, ModifierSource, Modifiers};
use sim_core::stats::StatId;
use utils::frame::FrameCount;
use utils::net_id::GgrsNetId;
use utils::order_mut_iter;
use utils::rollback::RollbackTraceApp;

pub struct Demo;

/// Primes versées à un porteur par `grant_bounty` : rollback, sous checksum (un desync ici se voit).
/// Neutre : sans porteur (toute partie qui n'utilise pas l'action), il ne contribue pas au checksum.
#[derive(Component, Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub struct Bounty(pub u32);

/// Contenu du kind `Crate` (`crates/<id>.ron`).
#[derive(Deserialize)]
struct CrateDef {
    weight: u32,
}

impl GamePlugin for Demo {
    fn extension(&self) -> Extension {
        Extension::default()
            .content_kind("Crate")
            .behavior(
                "rush",
                CustomBehaviorImpl {
                    applicable: rush_applicable,
                    motion: Some(rush_motion),
                },
            )
            .condition("after_frame", after_frame)
            .action("grant_bounty")
            .lint_rule("crate_weights", crate_weights)
    }

    fn build(&self, app: &mut App) {
        app.rollback_and_trace_neutral::<Bounty>().add_systems(
            bevy_ggrs::GgrsSchedule,
            grant_bounty_system
                .after(game::effects_v3::effect_status_system)
                .before(RollbackSystemSet::FrameCounter),
        );
    }
}

/// `rush(within)` : retenu quand la cible est à moins de `within` unités.
fn rush_applicable(params: &CustomParams, ctx: &SelectionContext) -> bool {
    let within = params.fixed("within").unwrap_or(Fixed::ZERO);
    ctx.target_distance.is_some_and(|d| d < within)
}

/// `rush(speed)` : droit sur la cible, `speed` fois la vitesse de l'ennemi.
fn rush_motion(params: &CustomParams, view: &CustomMotionView) -> Option<BehaviorMotion> {
    let target = view.target?;
    Some(BehaviorMotion {
        direction: (target - view.position).normalize_or_zero(),
        speed_mult: params.fixed("speed").unwrap_or(Fixed::ONE),
    })
}

/// `after_frame(frame)` : vraie à partir de la frame `frame`.
fn after_frame(params: &CustomParams, carrier: &Carrier, _trigger: Option<&Trigger>) -> bool {
    params.uint("frame").is_some_and(|frame| carrier.frame >= frame)
}

fn crate_weight(registry: &content::Registry, id: &str) -> Option<u32> {
    let entry = registry.plugin_content.get("Crate")?.get(id)?;
    ron::from_str::<CrateDef>(&entry.text).ok().map(|c| c.weight)
}

/// `grant_bounty(crate)` : un Luck de `weight` points (modificateur permanent) et une prime de plus.
fn grant_bounty_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    outbox: Res<FrameEvents<game::effects_v3::EffectOut>>,
    registry: Res<content::Registry>,
    mut carriers: Query<(&GgrsNetId, Entity, &mut Modifiers, Option<&mut Bounty>), With<Player>>,
) {
    for call in custom_actions(&outbox, "grant_bounty") {
        let Some(weight) = call
            .params
            .get("crate")
            .and_then(|id| crate_weight(&registry, id))
        else {
            continue;
        };
        // Le lint refuse un poids hors d'un `Fixed` ; la conversion reste vérifiée (jamais de panique en simulation).
        let Some(value) = Fixed::checked_from_num(weight) else {
            continue;
        };
        for (net_id, entity, mut modifiers, bounty) in order_mut_iter!(carriers) {
            if net_id.0 != call.carrier {
                continue;
            }
            info!(
                "ggrs{{f={} grant_bounty carrier={} weight={}}}",
                frame.frame, net_id.0, weight
            );
            modifiers.push(Modifier {
                stat: StatId::Luck,
                op: ModifierOp::Add,
                value,
                source: ModifierSource::Named(format!("bounty:{}", net_id.0)),
                until: None,
            });
            match bounty {
                Some(mut bounty) => bounty.0 += 1,
                None => {
                    commands.entity(entity).insert(Bounty(1));
                }
            }
        }
    }
}

/// Règle de lint : chaque `Crate` pèse au moins 1 ; chaque `grant_bounty` cite un `Crate` qui existe.
fn crate_weights(
    registry: &content::Registry,
    _manifest: &content::GameManifest,
    errors: &mut Vec<content::LintError>,
) {
    for (id, entry) in registry.plugin_content.get("Crate").into_iter().flatten() {
        match ron::from_str::<CrateDef>(&entry.text) {
            Ok(def) if def.weight >= 1 && Fixed::checked_from_num(def.weight).is_some() => {}
            Ok(def) => errors.push(content::plugin::rule_error(
                entry.file.display().to_string(),
                format!(
                    "Crate « {id} » : weight doit être >= 1 et tenir dans un Fixed (<= {}), lu {}",
                    Fixed::MAX.to_num::<u32>(),
                    def.weight
                ),
            )),
            Err(e) => errors.push(content::plugin::rule_error(
                entry.file.display().to_string(),
                format!("Crate « {id} » : {e}"),
            )),
        }
    }
    for item in registry.items.values() {
        for effect in &item.def.effects {
            for action in &effect.r#do {
                if let effects::Action::Custom { id, params } = action {
                    if id == "grant_bounty"
                        && params
                            .get("crate")
                            .is_none_or(|c| crate_weight(registry, c).is_none())
                    {
                        errors.push(content::plugin::rule_error(
                            item.file.display().to_string(),
                            format!("objet « {} » : grant_bounty : `crate` absent ou inconnu", item.id),
                        ));
                    }
                }
            }
        }
    }
}
