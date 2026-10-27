//! Effets v3 (M3-T2, `docs/conventions.md` §67) : `OnHit`, `OnEvent`/`Emit`, `ApplyStatus` dans un
//! effet, `Scaled { PerCopy }`. Le moteur d'exécution (`effects_runtime`) construit les déclencheurs ;
//! ce module porte ce qui est propre à la v3 :
//!
//! - [`HitLanded`] : un coup porté (dégât réel, après résolution), émis par
//!   `character::health::rollback_resolve_damage_events` et lu par `apply_effects_system` pour
//!   `OnHit` ;
//! - [`EffectOut`] : ce que les actions d'un effet produisent hors du porteur — un événement nommé
//!   (`Emit`) ou un statut à poser ; file **transitoire** (hors checksum, vidée au `FrameStart`) ;
//! - [`classify`] : sépare les actions qui touchent le porteur (exécutées par
//!   `effects_runtime::apply_action_sourced`) de celles qui sortent (`Emit`, `ApplyStatus*`) et
//!   applique `Scaled` (`PerCopy` : exécuté une seule fois par objet, par le premier exemplaire,
//!   multiplié par le nombre d'exemplaires) ;
//! - [`apply_event_effects_system`] : les effets `OnEvent(nom)` d'un porteur répondent aux `Emit` de
//!   ses propres effets, dans la frame, par tours (une émission d'un effet `OnEvent` est lue au tour
//!   suivant, [`MAX_EVENT_ROUNDS`] tours au plus) ;
//! - [`effect_status_system`] : pose les statuts demandés (même règle que
//!   `combat::projectile::apply_projectile_on_hit_system`).
//!
//! Aucun état rollback nouveau : tout est transitoire, dérivé du jeu de la frame.

use std::borrow::Cow;
use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_fixed::fixed_math::Fixed;
use bevy_ggrs::Rollback;
use combat::status::{StatusDef, StatusLibrary, Statuses};
use effects::actions::scaled_action;
use effects::actions::Scale;
use effects::runtime::Trigger;
use effects::Action;
use items::{ItemTable, SynergyTable};
use sim_core::custom::CustomParams;
use sim_core::frame_events::{FrameEvents, FrameEventsAppExt};
use sim_core::modifier::{Modifier, ModifierOp, ModifierSource, Modifiers};
use sim_core::stats::StatId;
use sim_core::tag::Tags;
use sim_core::team::Team;
use utils::frame::FrameCount;
use utils::net_id::GgrsNetId;
use utils::{order_iter, order_mut_iter};

use crate::effects_runtime::{
    carried_items, listed_effects, run_listed, AliveQuery, Body, CarrierQuery, CarrierView,
    EffectAssets, EffectState,
};

/// Tours d'`OnEvent` au plus par frame : un effet `OnEvent` peut lui-même émettre ; au-delà, les
/// émissions restantes sont abandonnées (déterministe : même suite sur tous les clients).
pub const MAX_EVENT_ROUNDS: usize = 8;

/// Un coup porté : `source` a infligé un dégât **réel** (après immunités, bouclier et armure) à
/// `target`, dans la frame. Jamais émis pour un dégât que la source s'inflige (piques, chute,
/// bombe sur son poseur).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HitLanded {
    pub frame: u32,
    pub source: usize,
    pub target: usize,
}

/// Sortie d'une action d'effet, hors du porteur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectOut {
    /// `Action::Emit(name)` du porteur `carrier`.
    Emit { carrier: usize, name: String },
    /// Statut à poser sur `target` ; `source` : l'auteur crédité des dégâts (`None` : sur soi).
    Status {
        source: Option<GgrsNetId>,
        target: usize,
        status: String,
        stacks: u32,
    },
    /// X-1 (§89) : `Action::Custom` du porteur `carrier`, à exécuter par le plugin du jeu (qui lit
    /// la file après `effect_status_system`, dernier système de la chaîne d'effets, voir
    /// `crate::extension::custom_actions`). `target` : la cible du déclencheur, s'il en a une.
    Custom {
        carrier: usize,
        target: Option<usize>,
        id: String,
        params: CustomParams,
    },
}

/// Ce qu'une action d'effet devient une fois `Scaled` appliqué.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step<'a> {
    /// Action ordinaire, exécutée sur le porteur par `apply_action_sourced` (éventuellement
    /// multipliée par `PerCopy`).
    Apply(Cow<'a, Action>),
    /// `Emit(nom)`.
    Emit(String),
    /// Pose d'un statut : sur la cible du déclencheur, ou sur le porteur (`to_self`).
    Status {
        to_self: bool,
        status: String,
        stacks: u32,
    },
    /// X-1 : `Action::Custom`, exécutée par le plugin du jeu (`EffectOut::Custom`).
    Custom { id: String, params: CustomParams },
    /// Rien à faire (`PerCopy` d'un exemplaire qui n'est pas le premier, action non multipliable,
    /// zéro exemplaire).
    Skip,
}

impl Step<'_> {
    fn into_owned(self) -> Step<'static> {
        match self {
            Step::Apply(action) => Step::Apply(Cow::Owned(action.into_owned())),
            Step::Emit(name) => Step::Emit(name),
            Step::Status {
                to_self,
                status,
                stacks,
            } => Step::Status {
                to_self,
                status,
                stacks,
            },
            Step::Custom { id, params } => Step::Custom { id, params },
            Step::Skip => Step::Skip,
        }
    }
}

/// Voir [`Step`]. `copy_rank` : rang de l'exemplaire qui exécute l'effet (0 pour le premier ou pour
/// un effet de personnage), `copies` : nombre d'exemplaires de l'objet porté. `Scaled { PerCopy }`
/// n'est exécuté que par le rang 0 (les autres exemplaires sont déjà comptés dans `copies`).
pub fn classify(action: &Action, copy_rank: u32, copies: u32) -> Step<'_> {
    match action {
        Action::Emit(name) => Step::Emit(name.clone()),
        Action::Custom { id, params } => Step::Custom {
            id: id.clone(),
            params: params.clone(),
        },
        Action::ApplyStatus { status, stacks } => Step::Status {
            to_self: false,
            status: status.clone(),
            stacks: *stacks,
        },
        Action::ApplyStatusSelf { status, stacks } => Step::Status {
            to_self: true,
            status: status.clone(),
            stacks: *stacks,
        },
        Action::Scaled {
            action: inner,
            scale: None,
        } => classify(inner, copy_rank, copies),
        Action::Scaled {
            action: inner,
            scale: Some(Scale::PerCopy),
        } => {
            if copy_rank > 0 {
                return Step::Skip;
            }
            match scaled_action(inner, copies) {
                Some(scaled) => classify(&scaled, copy_rank, copies).into_owned(),
                None => Step::Skip,
            }
        }
        other => Step::Apply(Cow::Borrowed(other)),
    }
}

/// Distribue les sorties d'un tour : les `Emit` au tour suivant (par porteur), le reste tel quel.
fn split_outputs(
    out: Vec<EffectOut>,
    next: &mut BTreeMap<usize, Vec<String>>,
    rest: &mut Vec<EffectOut>,
) {
    for event in out {
        match event {
            EffectOut::Emit { carrier, name } => next.entry(carrier).or_default().push(name),
            other => rest.push(other),
        }
    }
}

/// Les effets `OnEvent(nom)` d'un porteur répondent aux `Emit` de ses propres effets (voir la doc du
/// module). Après `apply_triggered_effects_system` : les `Emit` de `DeathManagement` (`OnHit`,
/// `OnKill`, `OnDamageTaken`, `Tick`) et ceux des déclencheurs de la seconde passe sont tous là.
#[allow(clippy::too_many_arguments)]
pub fn apply_event_effects_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    mut carriers: CarrierQuery,
    alive: AliveQuery,
    mut outbox: ResMut<FrameEvents<EffectOut>>,
    table: Option<Res<ItemTable>>,
    synergies: Option<Res<SynergyTable>>,
    assets: EffectAssets,
) {
    let mut queue: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    for event in outbox.iter() {
        if let EffectOut::Emit { carrier, name } = event {
            queue.entry(*carrier).or_default().push(name.clone());
        }
    }
    if queue.is_empty() {
        return;
    }
    let frame = frame.frame;
    let squad_size = alive.iter().count() as u32;
    let mut produced: Vec<EffectOut> = Vec::new();
    // État d'effets des porteurs qui n'en ont pas encore : conservé d'un tour à l'autre (l'insertion par
    // `Commands` n'est visible qu'après le système, sinon chaque tour repartirait d'un état vide et un
    // effet à `charges` se rechargerait à chaque tour).
    let mut locals: BTreeMap<usize, (Entity, EffectState)> = BTreeMap::new();
    for round in 0..MAX_EVENT_ROUNDS {
        if queue.is_empty() {
            break;
        }
        let mut next: BTreeMap<usize, Vec<String>> = BTreeMap::new();
        for (
            net_id,
            entity,
            effects,
            state_component,
            mut health,
            mut modifiers,
            mut gauges,
            tags,
            _regen,
            _player,
            dead_now,
            downed,
            has_emitter,
            inventory,
            transform,
        ) in order_mut_iter!(carriers)
        {
            let Some(names) = queue.remove(&net_id.0) else {
                continue;
            };
            if dead_now || downed {
                continue;
            }
            let mut state_component = state_component;
            let state: &mut EffectState = match state_component.as_deref_mut() {
                Some(state) => state,
                None => {
                    &mut locals
                        .entry(net_id.0)
                        .or_insert((entity, EffectState::default()))
                        .1
                }
            };
            let listed = listed_effects(
                effects,
                inventory,
                table.as_deref(),
                synergies.as_deref(),
                net_id,
                state,
                frame,
                false,
            );
            if listed.is_empty() {
                continue;
            }
            let triggers: Vec<Trigger> = names
                .into_iter()
                .map(|name| Trigger::Event { name })
                .collect();
            let no_tags = Tags::default();
            let carried = carried_items(inventory);
            let view = CarrierView {
                tags: tags.unwrap_or(&no_tags),
                carried: &carried,
                squad_size,
                position: transform.map(|t| (t.translation.x, t.translation.y)),
            };
            let mut out: Vec<EffectOut> = Vec::new();
            let mut body = Body {
                entity,
                has_emitter,
                health: &mut health,
                modifiers: &mut modifiers,
                gauges: gauges.as_deref_mut(),
                commands: &mut commands,
                out: &mut out,
            };
            run_listed(
                &listed, &triggers, false, frame, net_id, &view, state, &mut body, &assets,
            );
            split_outputs(out, &mut next, &mut produced);
        }
        queue = next;
        if round + 1 == MAX_EVENT_ROUNDS && !queue.is_empty() {
            warn!(
                "f{} : {} porteur(s) émettent encore après {} tours d'OnEvent : émissions abandonnées",
                frame,
                queue.len(),
                MAX_EVENT_ROUNDS
            );
        }
    }
    for (_, (entity, local)) in locals {
        if local != EffectState::default() {
            commands.entity(entity).insert(local);
        }
    }
    for event in produced {
        outbox.send(event);
    }
}

/// Pose les statuts demandés par les effets de la frame (`EffectOut::Status`, dans l'ordre
/// d'émission). Même règle que `apply_projectile_on_hit_system` : `Statuses::apply`, plus le
/// modificateur de vitesse d'un statut `Slow`. Une cible disparue (morte et détruite) est ignorée.
#[allow(clippy::type_complexity)]
pub fn effect_status_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    outbox: Res<FrameEvents<EffectOut>>,
    library: Res<StatusLibrary>,
    mut targets: Query<(&GgrsNetId, Entity, &mut Modifiers, Option<&mut Statuses>), With<Rollback>>,
    teams: Query<(&GgrsNetId, &Team), With<Rollback>>,
) {
    if !outbox
        .iter()
        .any(|event| matches!(event, EffectOut::Status { .. }))
    {
        return;
    }
    let team_of: BTreeMap<usize, Team> = teams.iter().map(|(id, team)| (id.0, *team)).collect();
    let mut new_statuses: BTreeMap<usize, (Entity, Statuses)> = BTreeMap::new();
    let mut by_net_id: BTreeMap<usize, (Entity, Mut<Modifiers>, Option<Mut<Statuses>>)> = targets
        .iter_mut()
        .map(|(net_id, entity, modifiers, statuses)| (net_id.0, (entity, modifiers, statuses)))
        .collect();
    for event in outbox.iter() {
        let EffectOut::Status {
            source,
            target,
            status,
            stacks,
        } = event
        else {
            continue;
        };
        let Some((entity, modifiers, statuses)) = by_net_id.get_mut(target) else {
            continue;
        };
        let Some(spec) = library.statuses.get(status) else {
            warn!("statut « {status} » inconnu (voir `alacod lint`)");
            continue;
        };
        let credited = source
            .as_ref()
            .map(|id| (id.clone(), team_of.get(&id.0).copied()));
        let target_statuses = match statuses.as_deref_mut() {
            Some(existing) => existing,
            None => {
                &mut new_statuses
                    .entry(*target)
                    .or_insert_with(|| (*entity, Statuses::default()))
                    .1
            }
        };
        target_statuses.apply(status, spec, credited, frame.frame, *stacks);
        info!(
            "ggrs{{f={} effect_status target={} id={} stacks={}}}",
            frame.frame,
            target,
            status,
            target_statuses.stacks(status)
        );
        if spec.kind == StatusDef::Slow {
            let until = target_statuses
                .0
                .iter()
                .find(|entry| entry.id == *status)
                .map(|entry| entry.expires_at_frame);
            let source = ModifierSource::Named(format!("status:{status}"));
            modifiers.remove_by_source(&source);
            for stat in [StatId::MoveSpeed, StatId::EnemyMoveSpeed] {
                modifiers.push(Modifier {
                    stat,
                    op: ModifierOp::Mul,
                    value: spec.factor,
                    source: source.clone(),
                    until,
                });
            }
        }
    }
    for (_, (entity, statuses)) in new_statuses {
        commands.entity(entity).insert(statuses);
    }
}

/// Position monde `(x, y)` d'une entité, pour `TargetInRange`.
pub fn xy(translation: bevy_fixed::fixed_math::FixedVec3) -> (Fixed, Fixed) {
    (translation.x, translation.y)
}

/// Carte `GgrsNetId → position` des entités rollback, construite à la demande (uniquement quand un
/// effet lit `TargetInRange`).
pub fn positions_of(
    query: &Query<(&GgrsNetId, &bevy_fixed::fixed_math::FixedTransform3D), With<Rollback>>,
) -> BTreeMap<usize, (Fixed, Fixed)> {
    order_iter!(query)
        .into_iter()
        .map(|(id, t)| (id.0, xy(t.translation)))
        .collect()
}

/// Enregistre les files transitoires et place les systèmes (voir la doc du module).
pub struct EffectsV3Plugin;

impl Plugin for EffectsV3Plugin {
    fn build(&self, app: &mut App) {
        app.add_frame_events_transient::<HitLanded>()
            .add_frame_events_transient::<EffectOut>()
            .add_systems(
                bevy_ggrs::GgrsSchedule,
                (
                    apply_event_effects_system
                        .after(crate::effects_runtime::apply_triggered_effects_system),
                    effect_status_system.after(apply_event_effects_system),
                )
                    .before(sim_core::system_set::RollbackSystemSet::FrameCounter),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use effects::actions::Scale;

    fn fx(v: f32) -> Fixed {
        Fixed::from_num(v)
    }

    fn per_copy(action: Action) -> Action {
        Action::Scaled {
            action: Box::new(action),
            scale: Some(Scale::PerCopy),
        }
    }

    #[test]
    fn classify_separe_porteur_emission_et_statut() {
        assert_eq!(
            classify(&Action::Heal(fx(2.0)), 0, 1),
            Step::Apply(Cow::Owned(Action::Heal(fx(2.0))))
        );
        assert_eq!(
            classify(&Action::Emit("tic".into()), 0, 1),
            Step::Emit("tic".into())
        );
        assert_eq!(
            classify(
                &Action::ApplyStatus {
                    status: "brulure".into(),
                    stacks: 2
                },
                0,
                1
            ),
            Step::Status {
                to_self: false,
                status: "brulure".into(),
                stacks: 2
            }
        );
        assert_eq!(
            classify(
                &Action::ApplyStatusSelf {
                    status: "lenteur".into(),
                    stacks: 1
                },
                0,
                1
            ),
            Step::Status {
                to_self: true,
                status: "lenteur".into(),
                stacks: 1
            }
        );
    }

    #[test]
    fn per_copy_une_seule_execution_par_objet_multipliee() {
        let heal = per_copy(Action::Heal(fx(2.0)));
        // 3 exemplaires : le premier exécute ×3, les autres rien
        assert_eq!(
            classify(&heal, 0, 3),
            Step::Apply(Cow::Owned(Action::Heal(fx(6.0))))
        );
        assert_eq!(classify(&heal, 1, 3), Step::Skip);
        assert_eq!(classify(&heal, 2, 3), Step::Skip);
        // un seul exemplaire : tel quel
        assert_eq!(
            classify(&heal, 0, 1),
            Step::Apply(Cow::Owned(Action::Heal(fx(2.0))))
        );
        // zéro exemplaire, ou action non multipliable : rien
        assert_eq!(classify(&heal, 0, 0), Step::Skip);
        let mul = per_copy(Action::Modifier {
            stat: StatId::Damage,
            op: ModifierOp::Mul,
            value: fx(2.0),
        });
        assert_eq!(classify(&mul, 0, 2), Step::Skip);
        // statut multiplié : les piles
        let status = per_copy(Action::ApplyStatus {
            status: "poison".into(),
            stacks: 1,
        });
        assert_eq!(
            classify(&status, 0, 4),
            Step::Status {
                to_self: false,
                status: "poison".into(),
                stacks: 4
            }
        );
        // `Scaled` sans échelle : transparent
        let plain = Action::Scaled {
            action: Box::new(Action::GainShield(1)),
            scale: None,
        };
        assert_eq!(
            classify(&plain, 2, 5),
            Step::Apply(Cow::Owned(Action::GainShield(1)))
        );
    }

    #[test]
    fn partage_des_sorties_par_tour() {
        let mut next = BTreeMap::new();
        let mut rest = Vec::new();
        split_outputs(
            vec![
                EffectOut::Emit {
                    carrier: 3,
                    name: "a".into(),
                },
                EffectOut::Status {
                    source: None,
                    target: 3,
                    status: "x".into(),
                    stacks: 1,
                },
                EffectOut::Emit {
                    carrier: 3,
                    name: "b".into(),
                },
            ],
            &mut next,
            &mut rest,
        );
        assert_eq!(next.get(&3), Some(&vec!["a".to_string(), "b".to_string()]));
        assert_eq!(rest.len(), 1);
    }

    // --- Tests ECS : les systèmes v3 sur de petits mondes ---

    use crate::character::health::Health;
    use crate::effects_runtime::{apply_effects_system, Effects};
    use crate::weapons::WeaponRegistry;
    use effects::{Condition, Effect, On};
    use items::{Inventory, ItemDef};

    fn app() -> App {
        let mut app = App::new();
        app.init_resource::<bevy_ggrs::RollbackOrdered>()
            .init_resource::<WeaponRegistry>()
            .init_resource::<crate::extension::CustomVerbs>()
            .insert_resource(FrameCount { frame: 10 })
            .init_resource::<FrameEvents<EffectOut>>()
            .init_resource::<FrameEvents<HitLanded>>();
        app
    }

    fn id(n: usize) -> GgrsNetId {
        GgrsNetId(n, "t".into())
    }

    fn health(current: f32) -> Health {
        Health {
            current: fx(current),
            max: fx(100.0),
            invulnerable_until_frame: None,
        }
    }

    fn on_event(name: &str, actions: Vec<Action>) -> Effect {
        Effect::new(On::OnEvent(name.into()), Vec::<Condition>::new(), actions)
    }

    fn emits(app: &App, carrier: usize) -> Vec<String> {
        app.world()
            .resource::<FrameEvents<EffectOut>>()
            .iter()
            .filter_map(|e| match e {
                EffectOut::Emit { carrier: c, name } if *c == carrier => Some(name.clone()),
                _ => None,
            })
            .collect()
    }

    fn statuses_of(app: &App, entity: Entity) -> Vec<(String, u32)> {
        app.world()
            .get::<Statuses>(entity)
            .map(|s| s.0.iter().map(|e| (e.id.clone(), e.stacks)).collect())
            .unwrap_or_default()
    }

    fn send_emit(app: &mut App, carrier: usize, name: &str) {
        app.world_mut()
            .resource_mut::<FrameEvents<EffectOut>>()
            .send(EffectOut::Emit {
                carrier,
                name: name.into(),
            });
    }

    #[test]
    fn une_chaine_de_trois_tours_se_resout_dans_la_meme_frame() {
        let mut app = app();
        app.add_systems(Update, apply_event_effects_system);
        // a -> b -> c -> +1 de dégâts, trois tours dans UNE frame
        let carrier = app
            .world_mut()
            .spawn((
                id(1),
                Effects(vec![
                    on_event("a", vec![Action::Emit("b".into())]),
                    on_event("b", vec![Action::Emit("c".into())]),
                    on_event(
                        "c",
                        vec![Action::Modifier {
                            stat: StatId::Damage,
                            op: ModifierOp::Add,
                            value: fx(1.0),
                        }],
                    ),
                ]),
                health(50.0),
                Modifiers::default(),
                Rollback,
            ))
            .id();
        send_emit(&mut app, 1, "a");
        app.update();
        let modifiers = app.world().get::<Modifiers>(carrier).unwrap();
        assert_eq!(modifiers.0.len(), 1, "le dernier maillon a tourné");
        assert_eq!(modifiers.0[0].stat, StatId::Damage);
    }

    #[test]
    fn le_plafond_de_tours_abandonne_les_emissions_restantes() {
        let mut app = app();
        app.add_systems(Update, apply_event_effects_system);
        // « x » s'émet lui-même et pose un statut à chaque tour : exactement MAX_EVENT_ROUNDS poses.
        app.world_mut().spawn((
            id(1),
            Effects(vec![on_event(
                "x",
                vec![
                    Action::Emit("x".into()),
                    Action::ApplyStatusSelf {
                        status: "poison".into(),
                        stacks: 1,
                    },
                ],
            )]),
            health(50.0),
            Modifiers::default(),
            Rollback,
        ));
        send_emit(&mut app, 1, "x");
        app.update();
        let poses = app
            .world()
            .resource::<FrameEvents<EffectOut>>()
            .iter()
            .filter(|e| matches!(e, EffectOut::Status { .. }))
            .count();
        assert_eq!(poses, MAX_EVENT_ROUNDS);
        // le dernier « x » émis au-delà du plafond n'est relu par personne ce tour-là
        assert_eq!(
            emits(&app, 1),
            vec!["x".to_string()],
            "seul l'émission d'origine reste"
        );
    }

    #[test]
    fn apply_status_vise_la_cible_et_apply_status_self_le_porteur() {
        let mut app = app();
        let mut library = StatusLibrary::default();
        library.statuses.insert(
            "poison".into(),
            combat::status::StatusSpec {
                kind: StatusDef::Poison,
                frames: 120,
                damage: fx(2.0),
                period: 30,
                factor: fx(1.0),
            },
        );
        library.statuses.insert(
            "lenteur".into(),
            combat::status::StatusSpec {
                kind: StatusDef::Slow,
                frames: 60,
                damage: fx(0.0),
                period: 0,
                factor: fx(0.5),
            },
        );
        app.insert_resource(library)
            .add_systems(Update, (apply_effects_system, effect_status_system).chain());
        let carrier = app
            .world_mut()
            .spawn((
                id(1),
                Team::Players,
                Effects(vec![Effect::new(
                    On::OnHit,
                    Vec::<Condition>::new(),
                    vec![
                        Action::ApplyStatus {
                            status: "poison".into(),
                            stacks: 1,
                        },
                        Action::ApplyStatusSelf {
                            status: "lenteur".into(),
                            stacks: 1,
                        },
                    ],
                )]),
                health(50.0),
                Modifiers::default(),
                Rollback,
            ))
            .id();
        let target = app
            .world_mut()
            .spawn((
                id(2),
                Team::Enemies,
                health(50.0),
                Modifiers::default(),
                Rollback,
            ))
            .id();
        let bystander = app
            .world_mut()
            .spawn((
                id(3),
                Team::Enemies,
                health(50.0),
                Modifiers::default(),
                Rollback,
            ))
            .id();
        app.world_mut()
            .resource_mut::<FrameEvents<HitLanded>>()
            .send(HitLanded {
                frame: 10,
                source: 1,
                target: 2,
            });
        app.update();
        assert_eq!(statuses_of(&app, target), vec![("poison".to_string(), 1)]);
        assert_eq!(statuses_of(&app, carrier), vec![("lenteur".to_string(), 1)]);
        assert!(statuses_of(&app, bystander).is_empty());
    }

    #[test]
    fn per_copy_s_execute_une_fois_multiplie_par_le_nombre_d_exemplaires() {
        let mut app = app();
        let def: ItemDef = ron::from_str(
            r#"(name: "A", kind: Passive, effects: [
                (on: OnEvent("go"), do: [Scaled(action: Heal("2"), scale: Some(PerCopy))]),
            ])"#,
        )
        .unwrap();
        app.insert_resource(items::ItemTable(
            [("amulette".to_string(), def)].into_iter().collect(),
        ))
        .add_systems(Update, apply_event_effects_system);
        let carrier = app
            .world_mut()
            .spawn((
                id(1),
                Inventory {
                    passives: vec!["amulette".into(), "amulette".into(), "amulette".into()],
                    ..Default::default()
                },
                health(10.0),
                Modifiers::default(),
                Rollback,
            ))
            .id();
        send_emit(&mut app, 1, "go");
        app.update();
        // une seule exécution de Heal(2) × 3 exemplaires = 6 (pas 3 × 6)
        assert_eq!(
            app.world().get::<Health>(carrier).unwrap().current,
            fx(16.0)
        );
    }

    /// Composition linéaire (§67) : un `Emit` non `Scaled` s'exécute par exemplaire, mais le récepteur
    /// `OnEvent` écoute une fois par objet : n exemplaires = n émissions × 1 récepteur (pas n × n).
    #[test]
    fn un_recepteur_on_event_ecoute_une_fois_par_objet() {
        fn damage_modifiers(copies: usize) -> usize {
            let mut app = app();
            let def: ItemDef = ron::from_str(
                r#"(name: "V", kind: Passive, effects: [
                    (on: OnHit, do: [Emit("coup")]),
                    (on: OnEvent("coup"), do: [Modifier(stat: Damage, op: Add, value: "1")]),
                ])"#,
            )
            .unwrap();
            app.insert_resource(items::ItemTable(
                [("vipere".to_string(), def)].into_iter().collect(),
            ))
            .add_systems(
                Update,
                (apply_effects_system, apply_event_effects_system).chain(),
            );
            let carrier = app
                .world_mut()
                .spawn((
                    id(1),
                    Inventory {
                        passives: vec!["vipere".into(); copies],
                        ..Default::default()
                    },
                    health(10.0),
                    Modifiers::default(),
                    Rollback,
                ))
                .id();
            app.world_mut().spawn((
                id(2),
                Team::Enemies,
                health(50.0),
                Modifiers::default(),
                Rollback,
            ));
            app.world_mut()
                .resource_mut::<FrameEvents<HitLanded>>()
                .send(HitLanded {
                    frame: 10,
                    source: 1,
                    target: 2,
                });
            app.update();
            app.world().get::<Modifiers>(carrier).unwrap().0.len()
        }
        assert_eq!(damage_modifiers(1), 1);
        // 2 exemplaires : 2 émissions lues par 1 récepteur = 2 (et non 4)
        assert_eq!(damage_modifiers(2), 2);
        assert_eq!(damage_modifiers(3), 3);
    }

    /// Un porteur sans `EffectState` préalable : l'état (charges) vit d'un tour à l'autre dans le système.
    /// `OnEvent("x")` à une charge qui soigne et réémet « x » : un seul soin, pas un par tour.
    #[test]
    fn un_effet_a_charges_ne_se_recharge_pas_a_chaque_tour() {
        let mut app = app();
        app.add_systems(Update, apply_event_effects_system);
        let mut effect = on_event("x", vec![Action::Heal(fx(1.0)), Action::Emit("x".into())]);
        effect.charges = Some(1);
        let carrier = app
            .world_mut()
            .spawn((
                id(1),
                Effects(vec![effect]),
                health(10.0),
                Modifiers::default(),
                Rollback,
            ))
            .id();
        send_emit(&mut app, 1, "x");
        app.update();
        assert_eq!(
            app.world().get::<Health>(carrier).unwrap().current,
            fx(11.0)
        );
        // l'état est posé sur le porteur, charge consommée
        assert!(app
            .world()
            .get::<crate::effects_runtime::EffectState>(carrier)
            .is_some());
    }

    /// `PerCopy` sous une enveloppe `scale: None` : le nombre d'exemplaires est quand même compté.
    #[test]
    fn per_copy_sous_une_enveloppe_sans_echelle_est_multiplie() {
        let mut app = app();
        let def: ItemDef = ron::from_str(
            r#"(name: "A", kind: Passive, effects: [
                (on: OnEvent("go"), do: [
                    Scaled(action: Scaled(action: Heal("1"), scale: Some(PerCopy)), scale: None),
                ]),
            ])"#,
        )
        .unwrap();
        app.insert_resource(items::ItemTable(
            [("amulette".to_string(), def)].into_iter().collect(),
        ))
        .add_systems(Update, apply_event_effects_system);
        let carrier = app
            .world_mut()
            .spawn((
                id(1),
                Inventory {
                    passives: vec!["amulette".into(); 3],
                    ..Default::default()
                },
                health(10.0),
                Modifiers::default(),
                Rollback,
            ))
            .id();
        send_emit(&mut app, 1, "go");
        app.update();
        assert_eq!(
            app.world().get::<Health>(carrier).unwrap().current,
            fx(13.0)
        );
    }
}
