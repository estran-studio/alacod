//! Exécution des effets v1 (T1.10, chantier C1 v1, `docs/conventions.md` §27).
//!
//! Un personnage porteur d'effets a [`Effects`] (posés par `CharacterConfig::effects` ou par
//! une mutation) et [`EffectState`] ; tous deux sont rollback, checksum **neutre** : aucun
//! personnage existant n'en porte, les traces ne bougent pas.
//!
//! [`apply_effects_system`] tourne dans `RollbackSystemSet::DeathManagement`, **après**
//! `rollback_apply_accumulated_damage` (les dégâts de la frame sont appliqués, les `Death`
//! posés) et **avant** `rollback_apply_bleedout`/`rollback_apply_death` : il voit les morts de
//! la frame (tueur = `Death::last_hit_by`) et les dégâts subis dans la frame
//! (`HealthRegen::last_damage_frame`, sinon un `DamageEvent` qui vise le porteur). Un `Effects` placé avant `DeathManagement` ne verrait
//! jamais ces morts : les `FrameEvents` sont vidés au début de chaque frame et les entités
//! mortes détruites dans la même frame. **La mort de la frame prime** : un porteur posé `Death`
//! ou `Downed` dans la frame ne déclenche rien et n'est pas soigné.
//!
//! **Effets v2 (M2-T3, `docs/conventions.md` §42)** : [`apply_triggered_effects_system`] tourne
//! **après** `RollbackSystemSet::Run` (avant `FrameCounter`) et exécute les déclencheurs lus
//! dans des `FrameEvents` émis plus tôt dans la frame : `OnRoomClear` (`RoomChanged`, émis dans
//! `Run`), `OnDodge` (`DashStarted`, `Movement`), `OnReload` (`ReloadFinished`, `Weapon`),
//! `OnPickup` (`ItemPicked`, `Interaction`/`Effects`), `OnBlank` (`BlankUsed`, après `blank_system`) et `OnFloorStart` (changement d'index
//! d'étage, passage d'étage dans `Run`). Les effets des **objets** tenus (passifs, actif hors
//! `OnUse`) sont lus dans l'`Inventory` à chaque frame, sans copie dans [`Effects`] : un objet
//! perdu n'a rien à retirer, l'index d'effet est celui du `ItemDef` (source des modificateurs
//! `item:<id>:<index>`). `cooldown`/`charges` d'un effet vivent dans [`EffectState::limits`].

use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_fixed::fixed_math::{self, Fixed, FixedVec2};
use bevy_ggrs::Rollback;
use combat::actors::DashState;
use combat::downed::Downed;
use combat::emitter::Emitter;
use combat::projectile::{resolve_pattern, PatternLibrary};
use combat::weapons::melee::MeleeAttackState;
use effects::runtime::{
    conditions_hold, effect_responds, healed, limits_allow, limits_consume, limits_recharge,
    tick_due, trigger_target, Carrier, Limits, TargetRef, TimingRefs, Trigger,
};
use effects::{Action, Condition, Effect, On, Recharge};
use items::{BlankUsed, Inventory, ItemPicked, ItemTable, SynergyTable};
use map::game::entity::map::{level_id::LevelId, room::RoomBounds};
use sim_core::damage::DamageEvent;
use sim_core::frame_events::FrameEvents;
use sim_core::frame_events::FrameEventsAppExt;
use sim_core::modifier::{Modifier, ModifierSource, Modifiers};
use sim_core::tag::Tags;
use sim_core::triggers::{DashStarted, ReloadFinished};
use utils::frame::FrameCount;
use utils::net_id::GgrsNetId;
use utils::{order_iter, order_mut_iter};
use world::{RoomChanged, RoomState};

use crate::character::health::{Death, Health, HealthRegen, HitBy};
use crate::character::player::Player;
use crate::effects_v3::{classify, positions_of, EffectOut, HitLanded, Step};
use crate::weapons::WeaponRegistry;

/// Effets du porteur, dans l'ordre de pose (index stable : source des modificateurs).
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Effects(pub Vec<Effect>);

/// Déclencheur reporté à la frame suivante (franchissement de jauge, montée de niveau),
/// produit après le passage des effets de la frame.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PendingTrigger {
    GaugeMoved {
        id: String,
        before: Fixed,
        after: Fixed,
    },
    LevelUp,
}

/// Jauges du porteur (rads de la progression, jauges de contenu) : rollback, checksum neutre.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Gauges(pub BTreeMap<String, sim_core::gauge::Gauge>);

impl Gauges {
    /// Ajoute `amount` à la jauge `id` et rend (avant, après), `None` si elle n'existe pas.
    pub fn add(&mut self, id: &str, amount: Fixed) -> Option<(Fixed, Fixed)> {
        let gauge = self.0.get_mut(id)?;
        let before = gauge.value;
        gauge.add(amount);
        Some((before, gauge.value))
    }
}

/// État d'exécution des effets du porteur.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub struct EffectState {
    /// Frame de pose de chaque effet (même index qu'[`Effects`]) : `Tick` compte depuis elle.
    /// Un effet sans entrée est posé à la première frame où le système le voit.
    pub posed: Vec<u32>,
    /// Dernière frame où le porteur a subi des dégâts (`NotHitFor`).
    pub last_hit_frame: Option<u32>,
    /// Déclencheurs reportés (jauges, niveaux), consommés à la frame suivante.
    pub pending: Vec<PendingTrigger>,
    /// M2-T3 : cooldown et charges des effets qui en déclarent, par clé d'effet
    /// (`effect:<index>` pour [`Effects`], `item:<id>:<index>` pour un objet tenu). Une entrée
    /// n'existe que pour un effet limité qui s'est déjà exécuté.
    pub limits: BTreeMap<String, Limits>,
    /// M2-T3 : frame de première vue des effets `Tick` d'objets (clé comme `limits`).
    pub item_posed: BTreeMap<String, u32>,
    /// M2-T3 : dernier index d'étage vu par un porteur qui a un effet `OnFloorStart` ou une
    /// recharge `PerFloor` (`None` : pas encore vu).
    pub floor_seen: Option<u32>,
}

/// Hash écrit à la main (M2-T3) : les trois champs d'avant dans le même ordre, puis les champs
/// v2 **seulement s'ils ne sont pas vides** : un porteur sans effet v2 garde exactement le hash
/// de T1.10 (traces existantes inchangées).
impl std::hash::Hash for EffectState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.posed.hash(state);
        self.last_hit_frame.hash(state);
        self.pending.hash(state);
        if !self.limits.is_empty() || !self.item_posed.is_empty() || self.floor_seen.is_some() {
            self.limits.hash(state);
            self.item_posed.hash(state);
            self.floor_seen.hash(state);
        }
    }
}

/// Source du modificateur posé par l'action `index` de l'effet `effect` du porteur `carrier`.
pub fn effect_source(carrier: &GgrsNetId, effect: usize) -> ModifierSource {
    ModifierSource::Named(format!("effect:{}:{}", carrier.0, effect))
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct EffectAssets<'w> {
    weapons: Res<'w, WeaponRegistry>,
    library: Option<Res<'w, PatternLibrary>>,
}

/// Sources des repères temporels d'une fenêtre de jugement (X-3, `effects::Condition::Timing`,
/// `docs/conventions.md` §90) : début du dash et du coup de mêlée en cours du porteur, horloges
/// actives. **Lecture seule**, dérivée de l'état rollback à chaque évaluation : aucun état
/// caché, rien à restaurer après un rollback. Construite seulement pour un porteur dont un
/// effet porte une condition `Timing` ([`has_timing`]) : sans elle, aucun calcul de plus.
#[derive(bevy::ecs::system::SystemParam)]
pub struct TimingSource<'w, 's> {
    clock: Res<'w, run::Clock>,
    schedule: Option<Res<'w, crate::clock::ClockSchedule>>,
    actors: Query<
        'w,
        's,
        (
            Option<&'static DashState>,
            Option<&'static MeleeAttackState>,
        ),
        With<Rollback>,
    >,
}

impl TimingSource<'_, '_> {
    /// Début du dash en cours : le pas du dash est joué à la frame même du lancement
    /// (`DashState::take_step`), donc `frames jouées - 1` frames plus tôt. `None` hors dash.
    pub fn dash_start(&self, entity: Entity, now: u32) -> Option<u32> {
        let (dash, _) = self.actors.get(entity).ok()?;
        let dash = dash?;
        if !dash.is_dashing {
            return None;
        }
        let played = dash
            .dash_duration
            .saturating_sub(dash.dash_frames_remaining);
        Some(now.saturating_sub(played.saturating_sub(1)))
    }

    /// Début du coup de mêlée en cours (`MeleeAttackState::attack_started_frame`), `None` hors coup.
    pub fn melee_start(&self, entity: Entity) -> Option<u32> {
        let (_, melee) = self.actors.get(entity).ok()?;
        let melee = melee?;
        melee
            .is_attacking
            .then_some(melee.attack_started_frame)
            .flatten()
    }

    /// Frame absolue de l'occurrence la plus proche de l'événement `id` de la première horloge
    /// active (ordre des ids) qui le porte (`run::clock::nearest_occurrence`). `None` sans
    /// horloge activée ou événement inconnu.
    pub fn beat(&self, id: &str, now: u32) -> Option<u32> {
        let schedule = self.schedule.as_ref()?;
        for (_name, def) in &schedule.clocks {
            let elapsed = self.clock.scope_frames(def.scope, now);
            if let Some(at) = run::clock::nearest_occurrence(def, id, elapsed) {
                // frame absolue = maintenant + (occurrence − temps écoulé), bornée à 0
                return Some(if at >= elapsed {
                    now.saturating_add(at - elapsed)
                } else {
                    now.saturating_sub(elapsed - at)
                });
            }
        }
        None
    }
}

/// Un effet de `listed` porte-t-il une condition `Timing` ? (Sinon, aucun repère à calculer.)
pub(crate) fn has_timing(listed: &[Listed]) -> bool {
    listed.iter().any(|entry| {
        entry
            .effect
            .r#if
            .iter()
            .any(|condition| matches!(condition, Condition::Timing { .. }))
    })
}

pub(crate) type CarrierQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static GgrsNetId,
        Entity,
        Option<&'static Effects>,
        Option<&'static mut EffectState>,
        &'static mut Health,
        &'static mut Modifiers,
        Option<&'static mut Gauges>,
        Option<&'static Tags>,
        Option<&'static HealthRegen>,
        Option<&'static Player>,
        Has<Death>,
        Has<Downed>,
        Has<Emitter>,
        Option<&'static Inventory>,
        Option<&'static fixed_math::FixedTransform3D>,
    ),
    (With<Rollback>, Or<(With<Effects>, With<Inventory>)>),
>;

/// Joueurs en vie (`SquadSize`).
pub(crate) type AliveQuery<'w, 's> = Query<
    'w,
    's,
    (),
    (
        With<Player>,
        With<Rollback>,
        Without<Death>,
        Without<Downed>,
    ),
>;

/// Morts de la frame (`Death` posé par `rollback_apply_accumulated_damage`).
pub type DeadQuery<'w, 's> =
    Query<'w, 's, (&'static GgrsNetId, &'static Death, Option<&'static Tags>), With<Rollback>>;
/// Toute entité rollback (tags, joueur) : tueurs et sources de dégâts.
pub type OthersQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static GgrsNetId,
        Option<&'static Tags>,
        Option<&'static Player>,
    ),
    With<Rollback>,
>;

/// Tueur (net id) → tags des cibles tuées cette frame, dans l'ordre des net ids des victimes.
/// Le tueur est le premier `HitBy` de `Death::last_hit_by` (un joueur par son handle).
pub fn kills_by_killer(dead: &DeadQuery, others: &OthersQuery) -> BTreeMap<usize, Vec<Tags>> {
    kills_with_victims(dead, others)
        .into_iter()
        .map(|(killer, victims)| (killer, victims.into_iter().map(|(tags, _)| tags).collect()))
        .collect()
}

/// Comme [`kills_by_killer`], avec le `GgrsNetId` de chaque victime (`OnKill` + `TargetInRange`, M3-T2).
pub fn kills_with_victims(
    dead: &DeadQuery,
    others: &OthersQuery,
) -> BTreeMap<usize, Vec<(Tags, usize)>> {
    let player_net_ids: BTreeMap<usize, usize> = others
        .iter()
        .filter_map(|(id, _, player)| player.map(|p| (p.handle, id.0)))
        .collect();
    let mut kills: BTreeMap<usize, Vec<(Tags, usize)>> = BTreeMap::new();
    for (victim, death, tags) in order_iter!(dead) {
        let killer = death.last_hit_by.as_ref().and_then(|hits| {
            hits.iter().find_map(|hit| match hit {
                HitBy::Player(handle) => player_net_ids.get(handle).copied(),
                HitBy::Entity(id) => Some(id.0),
            })
        });
        if let Some(killer) = killer {
            kills
                .entry(killer)
                .or_default()
                .push((tags.cloned().unwrap_or_default(), victim.0));
        }
    }
    kills
}

/// Un effet à exécuter pour un porteur : le sien ou celui d'un objet tenu.
pub(crate) struct Listed<'a> {
    /// Clé de `EffectState::limits` : `effect:<index>` ou `item:<id>:<index>` (`item:<id>#k:<index>`
    /// pour le k-ième exemplaire d'un objet `stack: Independent`, M3-T2).
    key: String,
    source: ModifierSource,
    /// Index dans le journal de trace (`index=`) : celui d'`Effects`, sinon celui du `ItemDef`.
    index: usize,
    pub(crate) effect: &'a Effect,
    /// Frame depuis laquelle `Tick` compte.
    posed: u32,
    /// M3-T2 : rang de cet exemplaire parmi ceux du même objet (0 : le premier, et tout effet qui
    /// n'est pas d'objet) et nombre d'exemplaires tenus, pour `Scaled { PerCopy }`.
    copy_rank: u32,
    copies: u32,
}

/// Les effets d'un porteur dans l'ordre : ceux d'[`Effects`] par index, puis ceux des objets
/// (passifs dans l'ordre de ramassage, puis l'actif) par index de `ItemDef`, `OnUse` exclu
/// (exécuté par `game::items`).
pub(crate) fn listed_effects<'a>(
    effects: Option<&'a Effects>,
    inventory: Option<&'a Inventory>,
    table: Option<&'a ItemTable>,
    synergies: Option<&'a SynergyTable>,
    net_id: &GgrsNetId,
    state: &mut EffectState,
    frame: u32,
    pose: bool,
) -> Vec<Listed<'a>> {
    let mut listed = Vec::new();
    if let Some(effects) = effects {
        // Seul le système v1 pose les effets : le second (après `Run`) ne doit pas avancer la
        // frame de pose d'un effet ajouté dans la frame (mutation), sinon `Tick` et la trace bougent.
        while pose && state.posed.len() < effects.0.len() {
            state.posed.push(frame);
        }
        for (index, effect) in effects.0.iter().enumerate() {
            listed.push(Listed {
                key: format!("effect:{index}"),
                source: effect_source(net_id, index),
                index,
                effect,
                posed: state.posed.get(index).copied().unwrap_or(frame),
                copy_rank: 0,
                copies: 1,
            });
        }
    }
    if let (Some(inventory), Some(table)) = (inventory, table) {
        let held = inventory
            .passives
            .iter()
            .map(String::as_str)
            .chain(inventory.active.iter().map(|slot| slot.item.as_str()));
        // Rang d'exemplaire de chaque id (ordre de ramassage) et nombre tenu (`PerCopy`).
        let mut seen: BTreeMap<&str, u32> = BTreeMap::new();
        for id in held {
            let Some(def) = table.0.get(id) else { continue };
            let rank = seen.entry(id).or_insert(0);
            let copy_rank = *rank;
            *rank += 1;
            // Limites partagées (défaut) : tous les exemplaires ont la clé du premier ; propres à
            // chaque exemplaire (`stack: Independent`) : suffixe `#k`.
            let key_rank = if def.stack == items::StackMode::Independent {
                copy_rank
            } else {
                0
            };
            let copies = if def.effects.iter().any(has_per_copy) {
                inventory.count(id)
            } else {
                1
            };
            for (index, effect) in def.effects.iter().enumerate() {
                if effect.on == On::OnUse {
                    continue;
                }
                // M3-T2 (§67) : un récepteur `OnEvent` écoute une fois par objet, pas par exemplaire
                // (seule `Scaled(PerCopy)` multiplie) ; les émetteurs, eux, s'exécutent par exemplaire.
                if copy_rank > 0 && matches!(effect.on, On::OnEvent(_)) {
                    continue;
                }
                let key = items::item_effect_key(id, key_rank, index);
                let posed = if matches!(effect.on, On::Tick(_)) {
                    *state.item_posed.entry(key.clone()).or_insert(frame)
                } else {
                    frame
                };
                listed.push(Listed {
                    source: ModifierSource::Named(key.clone()),
                    key,
                    index,
                    effect,
                    posed,
                    copy_rank,
                    copies,
                });
            }
        }
    }
    // M2-T4 : effets des synergies actives (par id), clé `synergy:<id>:<index>`
    if let (Some(inventory), Some(synergies), Some(table)) = (inventory, synergies, table) {
        for (id, def) in synergies.active(inventory, table) {
            for (index, effect) in def.effects.iter().enumerate() {
                let key = format!("synergy:{id}:{index}");
                let posed = if matches!(effect.on, On::Tick(_)) {
                    *state.item_posed.entry(key.clone()).or_insert(frame)
                } else {
                    frame
                };
                listed.push(Listed {
                    source: ModifierSource::Named(key.clone()),
                    key,
                    index,
                    effect,
                    posed,
                    copy_rank: 0,
                    copies: 1,
                });
            }
        }
    }
    listed
}

/// Un effet contient-il un `Scaled { PerCopy }`, à n'importe quelle profondeur d'enveloppes (comme le
/// lint et `effects_v3::classify`) ? Le nombre d'exemplaires ne se calcule que pour lui.
fn has_per_copy(effect: &Effect) -> bool {
    effect.r#do.iter().any(|action| {
        let mut action = action;
        while let Action::Scaled {
            action: inner,
            scale,
        } = action
        {
            if *scale == Some(effects::actions::Scale::PerCopy) {
                return true;
            }
            action = inner;
        }
        false
    })
}

/// Ce que les effets d'un porteur peuvent modifier.
pub(crate) struct Body<'a, 'w, 's> {
    pub(crate) entity: Entity,
    pub(crate) has_emitter: bool,
    pub(crate) health: &'a mut Health,
    pub(crate) modifiers: &'a mut Modifiers,
    pub(crate) gauges: Option<&'a mut Gauges>,
    pub(crate) commands: &'a mut Commands<'w, 's>,
    /// M3-T2 : sorties hors du porteur (`Emit`, statuts à poser), vidées par l'appelant.
    pub(crate) out: &'a mut Vec<EffectOut>,
}

/// Lecture seule du porteur pour les conditions.
pub(crate) struct CarrierView<'a> {
    pub(crate) tags: &'a Tags,
    pub(crate) carried: &'a [&'a str],
    pub(crate) squad_size: u32,
    /// Position monde du porteur (`TargetInRange`).
    pub(crate) position: Option<(Fixed, Fixed)>,
}

/// Exécute les effets de `listed` qui répondent à `triggers` (et, si `tick`, à leur `Tick`).
/// Ordre : déclencheurs, puis effets par ordre de liste. Les recharges (`PerRoom` sur
/// `RoomCleared`, `PerFloor` sur `FloorStart`) précèdent les exécutions de la frame.
#[allow(clippy::too_many_arguments)]
pub(crate) fn run_listed(
    listed: &[Listed],
    triggers: &[Trigger],
    tick: bool,
    frame: u32,
    net_id: &GgrsNetId,
    view: &CarrierView,
    state: &mut EffectState,
    body: &mut Body,
    assets: &EffectAssets,
    timing: &TimingSource,
) {
    // X-3 : repères de `Condition::Timing`, calculés une fois par appel et seulement si un effet
    // en porte une.
    let timed = has_timing(listed);
    let (dash_start, melee_start) = if timed {
        (
            timing.dash_start(body.entity, frame),
            timing.melee_start(body.entity),
        )
    } else {
        (None, None)
    };
    let beat = |id: &str| timing.beat(id, frame);
    let refs = TimingRefs {
        dash_start,
        melee_start,
        beat: if timed { Some(&beat) } else { None },
    };
    for trigger in triggers {
        let recharge = match trigger {
            Trigger::RoomCleared => Recharge::PerRoom,
            Trigger::FloorStart => Recharge::PerFloor,
            _ => continue,
        };
        for entry in listed {
            if let Some(limits) = state.limits.get_mut(&entry.key) {
                limits_recharge(entry.effect, limits, recharge);
                if limits.is_default() {
                    state.limits.remove(&entry.key);
                }
            }
        }
    }
    for entry in listed {
        let effect = entry.effect;
        let mut firings: Vec<Option<&Trigger>> = triggers
            .iter()
            .filter(|t| effect_responds(effect, t))
            .map(Some)
            .collect();
        if tick && tick_due(&effect.on, entry.posed, frame) {
            firings.push(None);
        }
        for trigger in firings {
            let carrier = Carrier {
                tags: view.tags,
                health: body.health.current,
                health_max: body.health.max,
                frame,
                last_hit_frame: state.last_hit_frame,
                carried: view.carried,
                squad_size: view.squad_size,
                position: view.position,
                timing: refs,
            };
            if !conditions_hold(&effect.r#if, &carrier, trigger) {
                continue;
            }
            let limited = effect.cooldown.is_some() || effect.charges.is_some();
            if limited {
                let limits = state.limits.get(&entry.key).copied().unwrap_or_default();
                if !limits_allow(effect, &limits, frame) {
                    continue;
                }
                let mut limits = limits;
                limits_consume(effect, &mut limits, frame);
                state.limits.insert(entry.key.clone(), limits);
            }
            info!(
                "ggrs{{f={} effect carrier={} index={} on={:?}}}",
                frame, net_id, entry.key, effect.on
            );
            let target = trigger_target(trigger).map(|target| target.id);
            for action in &effect.r#do {
                match classify(action, entry.copy_rank, entry.copies) {
                    Step::Apply(action) => apply_action_sourced(
                        &action,
                        net_id,
                        entry.source.clone(),
                        frame,
                        Some(&mut *body.health),
                        body.modifiers,
                        state,
                        body.gauges.as_deref_mut(),
                        (body.entity, body.has_emitter),
                        body.commands,
                        assets,
                    ),
                    Step::Emit(name) => body.out.push(EffectOut::Emit {
                        carrier: net_id.0,
                        name,
                    }),
                    Step::Status {
                        to_self,
                        status,
                        stacks,
                    } => {
                        // Sur le porteur, ou sur la cible du déclencheur (sans cible : rien, le lint
                        // refuse `ApplyStatus` hors `OnHit`/`OnDamageTaken`).
                        let (target, source) = if to_self {
                            (Some(net_id.0), None)
                        } else {
                            (target, Some(net_id.clone()))
                        };
                        if let Some(target) = target {
                            body.out.push(EffectOut::Status {
                                source,
                                target,
                                status,
                                stacks,
                            });
                        }
                    }
                    Step::Skip => {}
                }
            }
        }
    }
}

/// Voir la doc du module.
#[allow(clippy::too_many_arguments)]
pub fn apply_effects_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    mut carriers: CarrierQuery,
    dead: DeadQuery,
    others: OthersQuery,
    alive: AliveQuery,
    damage: Option<Res<FrameEvents<DamageEvent>>>,
    hits: Option<Res<FrameEvents<HitLanded>>>,
    positions_q: Query<(&GgrsNetId, &fixed_math::FixedTransform3D), With<Rollback>>,
    mut outbox: ResMut<FrameEvents<EffectOut>>,
    table: Option<Res<ItemTable>>,
    synergies: Option<Res<SynergyTable>>,
    assets: EffectAssets,
    timing: TimingSource,
) {
    if carriers.is_empty() {
        return;
    }
    let frame = frame.frame;
    let squad_size = alive.iter().count() as u32;
    // Positions des cibles : construites à la demande, seulement si un effet lit `TargetInRange`.
    let mut positions: Option<BTreeMap<usize, (Fixed, Fixed)>> = None;

    let kills = kills_with_victims(&dead, &others);
    let tags_by_net_id: BTreeMap<usize, Tags> = others
        .iter()
        .map(|(id, tags, _)| (id.0, tags.cloned().unwrap_or_default()))
        .collect();

    for (
        net_id,
        entity,
        effects,
        state_component,
        mut health,
        mut modifiers,
        mut gauges,
        tags,
        regen,
        _player,
        dead_now,
        downed,
        has_emitter,
        inventory,
        transform,
    ) in order_mut_iter!(carriers)
    {
        let had_state = state_component.is_some();
        let mut state_component = state_component;
        let mut local = EffectState::default();
        let state: &mut EffectState = match state_component.as_deref_mut() {
            Some(state) => state,
            None => &mut local,
        };
        // Pose des effets nouveaux (frame de pose de `Tick`)
        let listed = listed_effects(
            effects,
            inventory,
            table.as_deref(),
            synergies.as_deref(),
            net_id,
            state,
            frame,
            true,
        );
        // Porteur v2 sans `Effects` ni effet d'objet : rien à faire, et surtout pas d'`EffectState`
        // (il entrerait dans la trace d'un joueur qui n'en avait pas, v1 inchangé).
        if effects.is_none() && listed.is_empty() {
            continue;
        }
        let pending = std::mem::take(&mut state.pending);
        if dead_now || downed {
            continue;
        }

        // Déclencheurs de la frame
        let uses_range = listed.iter().any(|l| {
            l.effect
                .r#if
                .iter()
                .any(|c| matches!(c, Condition::TargetInRange(_)))
        });
        let mut target_ref = |id: usize| TargetRef {
            id,
            position: if uses_range {
                positions
                    .get_or_insert_with(|| positions_of(&positions_q))
                    .get(&id)
                    .copied()
            } else {
                None
            },
        };
        let mut triggers: Vec<Trigger> = Vec::new();
        for (target_tags, victim) in kills.get(&net_id.0).into_iter().flatten() {
            triggers.push(Trigger::Kill {
                target_tags: target_tags.clone(),
                target: Some(target_ref(*victim)),
            });
        }
        // Dégâts subis cette frame : `HealthRegen` les date quand le porteur en a un (dégâts
        // réellement appliqués, immunités exclues) ; sinon un `DamageEvent` qui le vise.
        let hit_event = damage
            .as_ref()
            .and_then(|events| events.iter().find(|e| e.target.0 == net_id.0));
        let hurt_now = match regen {
            Some(r) => r.last_damage_frame == frame && frame > 0,
            None => hit_event.is_some(),
        };
        if hurt_now {
            state.last_hit_frame = Some(frame);
            let source_tags = hit_event
                .and_then(|e| tags_by_net_id.get(&e.source.0).cloned())
                .unwrap_or_default();
            triggers.push(Trigger::DamageTaken {
                source_tags,
                target: hit_event.map(|e| target_ref(e.source.0)),
            });
        }
        // M3-T2 : coups portés (dégât réel, après résolution), dans l'ordre d'émission
        for hit in hits
            .iter()
            .flat_map(|events| events.iter())
            .filter(|hit| hit.source == net_id.0)
        {
            triggers.push(Trigger::Hit {
                target_tags: tags_by_net_id.get(&hit.target).cloned().unwrap_or_default(),
                target: target_ref(hit.target),
            });
        }
        for p in pending {
            triggers.push(match p {
                PendingTrigger::GaugeMoved { id, before, after } => {
                    Trigger::GaugeMoved { id, before, after }
                }
                PendingTrigger::LevelUp => Trigger::LevelUp,
            });
        }

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
            &listed, &triggers, true, frame, net_id, &view, state, &mut body, &assets, &timing,
        );
        for event in out {
            outbox.send(event);
        }
        if !had_state && local != EffectState::default() {
            commands.entity(entity).insert(local);
        }
    }
}

/// Objets tenus (passifs puis actif), pour `Carrying`.
pub(crate) fn carried_items(inventory: Option<&Inventory>) -> Vec<&str> {
    inventory
        .map(|inventory| {
            inventory
                .passives
                .iter()
                .map(String::as_str)
                .chain(inventory.active.iter().map(|slot| slot.item.as_str()))
                .collect()
        })
        .unwrap_or_default()
}

/// Déclencheurs v2 (M2-T3) : voir la doc du module. Tourne après `RollbackSystemSet::Run`.
#[allow(clippy::too_many_arguments)]
pub fn apply_triggered_effects_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    mut carriers: CarrierQuery,
    alive: AliveQuery,
    rooms: Query<(&LevelId, &RoomBounds)>,
    room_events: Option<Res<FrameEvents<RoomChanged>>>,
    dash_events: Option<Res<FrameEvents<DashStarted>>>,
    reload_events: Option<Res<FrameEvents<ReloadFinished>>>,
    pick_events: Option<Res<FrameEvents<ItemPicked>>>,
    blank_events: Option<Res<FrameEvents<BlankUsed>>>,
    floor: Option<Res<run::FloorState>>,
    mut outbox: ResMut<FrameEvents<EffectOut>>,
    table: Option<Res<ItemTable>>,
    synergies: Option<Res<SynergyTable>>,
    assets: EffectAssets,
    timing: TimingSource,
) {
    if carriers.is_empty() {
        return;
    }
    let frame = frame.frame;
    let squad_size = alive.iter().count() as u32;
    let floor_index = floor.map_or(0, |floor| floor.index);
    let cleared: Vec<&str> = room_events
        .as_deref()
        .map(|events| {
            events
                .iter()
                .filter(|e| e.to == RoomState::Cleared)
                .map(|e| e.room.as_str())
                .collect()
        })
        .unwrap_or_default();

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
        player,
        dead_now,
        downed,
        has_emitter,
        inventory,
        transform,
    ) in order_mut_iter!(carriers)
    {
        if dead_now || downed {
            continue;
        }
        let had_state = state_component.is_some();
        let mut state_component = state_component;
        let mut local = EffectState::default();
        let state: &mut EffectState = match state_component.as_deref_mut() {
            Some(state) => state,
            None => &mut local,
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

        let mut triggers: Vec<Trigger> = Vec::new();
        if let Some(player) = player {
            let handle = player.handle;
            for _ in dash_events
                .iter()
                .flat_map(|events| events.iter())
                .filter(|e| e.player_handle == handle)
            {
                triggers.push(Trigger::Dodge);
            }
            for _ in reload_events
                .iter()
                .flat_map(|events| events.iter())
                .filter(|e| e.player_handle == handle)
            {
                triggers.push(Trigger::Reload);
            }
            for _ in blank_events
                .iter()
                .flat_map(|events| events.iter())
                .filter(|e| e.player_handle == handle)
            {
                triggers.push(Trigger::Blank);
            }
            for event in pick_events
                .iter()
                .flat_map(|events| events.iter())
                .filter(|e| e.player_handle == handle)
            {
                triggers.push(Trigger::Pickup {
                    item: event.item_id.clone(),
                });
            }
        }
        // Salle du porteur nettoyée : sa position est dans les bornes de la salle de l'événement
        if let Some(transform) = transform {
            let position = transform.translation.truncate();
            for room in &cleared {
                if rooms
                    .iter()
                    .any(|(id, bounds)| id.0 == *room && bounds.contains(position))
                {
                    triggers.push(Trigger::RoomCleared);
                }
            }
        }
        // Début d'étage : l'index change (le premier étage compte pour qui est là au départ)
        let tracks_floor = listed.iter().any(|l| {
            l.effect.on == On::OnFloorStart || l.effect.recharge == Some(Recharge::PerFloor)
        });
        if tracks_floor {
            match state.floor_seen {
                Some(seen) if seen == floor_index => {}
                None if frame > 1 => state.floor_seen = Some(floor_index),
                _ => {
                    state.floor_seen = Some(floor_index);
                    triggers.push(Trigger::FloorStart);
                }
            }
        }
        if !triggers.is_empty() {
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
                &listed, &triggers, false, frame, net_id, &view, state, &mut body, &assets, &timing,
            );
            for event in out {
                outbox.send(event);
            }
        }
        if !had_state && local != EffectState::default() {
            commands.entity(entity).insert(local);
        }
    }
}

/// `SpawnPattern` : pose un émetteur jouant `pattern` avec la table de projectiles de `weapon`,
/// visée fixe (+x). `telegraph > 0` (M2-T7, `Telegraph(n)` d'une frise de boss) : l'émetteur
/// joue `Sequence([Telegraph(n), pattern])` ; 0 : le pattern tel quel (chemin d'avant). Rend
/// vrai si un émetteur est posé (ignoré si un émetteur joue déjà).
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_pattern(
    pattern: &str,
    weapon: &str,
    telegraph: u32,
    net_id: &GgrsNetId,
    frame: u32,
    (entity, has_emitter): (Entity, bool),
    commands: &mut Commands,
    assets: &EffectAssets,
) -> bool {
    if has_emitter {
        warn!(
            "effet de {} : SpawnPattern « {} » ignoré, un émetteur joue déjà",
            net_id, pattern
        );
        return false;
    }
    let Some(weapon_asset) = assets.weapons.weapons.get(weapon) else {
        warn!(
            "effet de {} : arme « {} » inconnue (voir `alacod lint`)",
            net_id, weapon
        );
        return false;
    };
    let named = combat::projectile::Pattern::Named(pattern.to_string());
    let Ok(mut resolved) = resolve_pattern(assets.library.as_deref(), &named) else {
        warn!(
            "effet de {} : pattern « {} » inconnu (voir `alacod lint`)",
            net_id, pattern
        );
        return false;
    };
    if telegraph > 0 {
        resolved = combat::projectile::Pattern::Sequence(vec![
            combat::projectile::Pattern::Telegraph(telegraph),
            resolved,
        ]);
    }
    commands.entity(entity).insert(Emitter::new(
        pattern.to_string(),
        &resolved,
        weapon.to_string(),
        std::sync::Arc::new(weapon_asset.config.projectiles.clone()),
        fixed_math::FIXED_ONE,
        weapon_asset.config.friendly_fire,
        FixedVec2::new(Fixed::ONE, Fixed::ZERO),
        frame,
    ));
    true
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_action(
    action: &Action,
    net_id: &GgrsNetId,
    index: usize,
    frame: u32,
    health: &mut Health,
    modifiers: &mut Modifiers,
    state: &mut EffectState,
    gauges: Option<&mut Gauges>,
    entity: (Entity, bool),
    commands: &mut Commands,
    assets: &EffectAssets,
) {
    apply_action_sourced(
        action,
        net_id,
        effect_source(net_id, index),
        frame,
        Some(health),
        modifiers,
        state,
        gauges,
        entity,
        commands,
        assets,
    );
}

/// Comme [`apply_action`], avec la source de modificateur choisie par l'appelant
/// (`effect:<porteur>:<index>`, ou `item:<id>:<index>` pour un objet tenu).
///
/// Seule liste d'actions exécutées sur un porteur (D80) : les effets de personnage et d'objet tenu
/// (`apply_triggered_effects_system`), la frise de boss (`apply_action`) et les effets `OnUse` d'un
/// objet (`items::apply_instant_effects`) passent tous ici. `health` est absent pour un porteur sans
/// `Health` (joueur qui n'en a pas encore) : `Heal` est alors sans effet.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_action_sourced(
    action: &Action,
    net_id: &GgrsNetId,
    source: ModifierSource,
    frame: u32,
    health: Option<&mut Health>,
    modifiers: &mut Modifiers,
    state: &mut EffectState,
    gauges: Option<&mut Gauges>,
    (entity, has_emitter): (Entity, bool),
    commands: &mut Commands,
    assets: &EffectAssets,
) {
    match action {
        Action::Modifier { stat, op, value } => modifiers.push(Modifier {
            stat: stat.clone(),
            op: *op,
            value: *value,
            source,
            until: None,
        }),
        Action::TimedModifier { .. } | Action::CurrencyMultiplier { .. } => {
            if let Some(modifier) = action.as_modifier(frame, source) {
                modifiers.push(modifier);
            }
        }
        Action::Heal(amount) => {
            if let Some(health) = health {
                health.current = healed(health.current, health.max, *amount);
            }
        }
        Action::GainShield(points) => {
            combat::actors::queue_gain_shield(commands, entity, *points);
        }
        // M3-T3 (§68) : cœurs d'âme (`ExtraHealth`), posés au premier gain.
        Action::GainSoul(points) => {
            combat::actors::queue_gain_soul(commands, entity, *points);
        }
        Action::GaugeAdd(id, amount) => {
            // Franchissements : `OnGauge` à la frame suivante (déclencheur reporté)
            match gauges.and_then(|g| g.add(id, *amount)) {
                Some((before, after)) if before != after => {
                    state.pending.push(PendingTrigger::GaugeMoved {
                        id: id.clone(),
                        before,
                        after,
                    })
                }
                Some(_) => {}
                None => warn!("effet de {} : jauge « {} » absente", net_id, id),
            }
        }
        Action::SpawnPattern { pattern, weapon } => {
            spawn_pattern(
                pattern,
                weapon,
                0,
                net_id,
                frame,
                (entity, has_emitter),
                commands,
                assets,
            );
        }
        // M3-T4 (§69) : l'explosion est une bombe à mèche d'une frame au point du porteur : même
        // chemin que la bombe d'un joueur (`game::bombs`), explosion à la frame suivante, comme un
        // baril. Posée par une commande différée : le `GgrsNetIdFactory` et la position du porteur
        // se lisent à l'application, dans l'ordre déterministe des systèmes.
        Action::Explode {
            radius,
            damage,
            destroys,
            friendly_fire,
        } => {
            let (radius, damage, destroys) = (*radius, *damage, *destroys);
            let friendly_fire = sim_core::damage::FriendlyFire::from(*friendly_fire);
            let owner = net_id.clone();
            commands.queue(move |world: &mut World| {
                let Some(position) = world
                    .get::<fixed_math::FixedTransform3D>(entity)
                    .map(|transform| transform.translation)
                else {
                    return;
                };
                let team = world
                    .get::<sim_core::team::Team>(entity)
                    .copied()
                    .unwrap_or(sim_core::team::Team::Neutral);
                let id = world
                    .resource_mut::<utils::net_id::GgrsNetIdFactory>()
                    .next(format!("explode_{}", owner.0));
                let at = fixed_math::FixedVec3::new(position.x, position.y, Fixed::ZERO);
                let transform = fixed_math::FixedTransform3D::new(
                    at,
                    fixed_math::FixedMat3::IDENTITY,
                    fixed_math::FixedVec3::ONE,
                );
                world.spawn((
                    world::Bomb {
                        owner,
                        fuse_until: frame.saturating_add(1),
                        radius,
                        damage,
                        friendly_fire,
                        team,
                        destroys,
                    },
                    transform.to_bevy_transform(),
                    transform,
                    id,
                    Rollback,
                ));
            });
        }
        Action::RefillAmmo
        | Action::RefillAmmoOf(_)
        | Action::RepairAllWindows
        | Action::KillAllWaveEnemies
        | Action::DestroyTerrain { .. }
        | Action::ApplyStatus { .. }
        // M2-T7 : frise de boss seulement (exécutées par `game::boss`).
        | Action::SpawnEnemies { .. }
        | Action::Telegraph(_)
        // M3-T0d : mort d'un champion, exécutée par M3-T7 (refusée dans un effet par le lint)
        | Action::SpawnCharacters { .. }
        // M3-T1 : chance d'`on_hit` de projectile seulement
        | Action::Chance { .. }
        // M3-T2 : `Scaled`, `Emit` et `ApplyStatus*` sont triés par `effects_v3::classify` avant
        // d'arriver ici (la frise de boss les refuse au lint)
        | Action::Scaled { .. }
        | Action::Emit(_)
        | Action::ApplyStatusSelf { .. } => {
            warn!(
                "effet de {} : action {:?} non exécutée en v1 (voir `alacod lint`)",
                net_id, action
            );
        }
    }
}

/// Enregistre les composants d'effets (rollback, checksum neutre) et place
/// [`apply_effects_system`] (voir la doc du module).
pub struct EffectsRuntimePlugin;

impl Plugin for EffectsRuntimePlugin {
    fn build(&self, app: &mut App) {
        use utils::rollback::RollbackTraceApp;
        app.rollback_and_trace_neutral::<Effects>()
            .rollback_and_trace_neutral::<EffectState>()
            .rollback_and_trace_neutral::<Gauges>()
            .add_frame_events_transient::<DashStarted>()
            .add_frame_events_transient::<ReloadFinished>()
            .add_systems(
                bevy_ggrs::GgrsSchedule,
                apply_triggered_effects_system
                    .after(sim_core::system_set::RollbackSystemSet::Run)
                    .after(crate::items::item_charge_system)
                    .after(crate::items::blank_system)
                    .before(sim_core::system_set::RollbackSystemSet::FrameCounter),
            )
            .add_systems(
                bevy_ggrs::GgrsSchedule,
                apply_effects_system
                    .after(crate::character::health::rollback_apply_accumulated_damage)
                    .before(crate::character::health::rollback_apply_bleedout)
                    .in_set(sim_core::system_set::RollbackSystemSet::DeathManagement),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use effects::{Condition, On};

    fn fx(v: f32) -> Fixed {
        Fixed::from_num(v)
    }

    /// Porteur touché cette frame (`HealthRegen` datée de la frame), effet
    /// `OnDamageTaken` → `Heal(5)` ; `dead` : posé `Death` dans la même frame.
    fn spawn_hurt_carrier(app: &mut App, net_id: usize, frame: u32, dead: bool) -> Entity {
        let mut entity = app.world_mut().spawn((
            GgrsNetId(net_id, "pilote".into()),
            Effects(vec![Effect::new(
                On::OnDamageTaken,
                vec![Condition::HpBelow(fx(0.5))],
                vec![Action::Heal(fx(5.0))],
            )]),
            EffectState::default(),
            Health {
                current: fx(if dead { 0.0 } else { 20.0 }),
                max: fx(100.0),
                invulnerable_until_frame: None,
            },
            Modifiers::default(),
            HealthRegen {
                last_damage_frame: frame,
                regen_rate: fx(0.0),
                regen_delay_frames: 0,
            },
            Rollback,
        ));
        if dead {
            entity.insert(Death { last_hit_by: None });
        }
        entity.id()
    }

    #[test]
    fn la_mort_de_la_frame_prime_sur_le_soin() {
        let mut app = App::new();
        app.init_resource::<bevy_ggrs::RollbackOrdered>()
            .init_resource::<WeaponRegistry>()
            .insert_resource(FrameCount { frame: 50 })
            .init_resource::<FrameEvents<EffectOut>>()
            .add_systems(Update, apply_effects_system);
        let alive = spawn_hurt_carrier(&mut app, 1, 50, false);
        let dead = spawn_hurt_carrier(&mut app, 2, 50, true);
        app.update();
        let health = |e: Entity| app.world().get::<Health>(e).unwrap().current;
        assert_eq!(health(alive), fx(25.0), "touché sous 50 % : soigné de 5");
        assert_eq!(
            health(dead),
            fx(0.0),
            "mort dans la frame : aucun effet, pas de soin"
        );
    }

    /// `Explode` d'un effet (M3-T4, §69) : une bombe à mèche d'une frame au point du porteur, de
    /// l'équipe du porteur, créditée à son `GgrsNetId`, qui casse les rochers si `destroys`.
    #[test]
    fn explode_pose_une_bombe_a_meche_d_une_frame_de_l_equipe_du_porteur() {
        fn run(
            mut commands: Commands,
            assets: EffectAssets,
            mut carriers: Query<(Entity, &GgrsNetId, &mut Health, &mut Modifiers)>,
        ) {
            for (entity, net_id, mut health, mut modifiers) in &mut carriers {
                apply_action_sourced(
                    &Action::Explode {
                        radius: Fixed::from_num(40),
                        damage: Fixed::from_num(7),
                        destroys: true,
                        friendly_fire: effects::ExplodeFriendlyFire::Always,
                    },
                    net_id,
                    effect_source(net_id, 0),
                    50,
                    Some(&mut *health),
                    &mut modifiers,
                    &mut EffectState::default(),
                    None,
                    (entity, false),
                    &mut commands,
                    &assets,
                );
            }
        }
        let mut app = App::new();
        app.init_resource::<bevy_ggrs::RollbackOrdered>()
            .init_resource::<WeaponRegistry>()
            .insert_resource(utils::net_id::GgrsNetIdFactory::default())
            .add_systems(Update, run);
        let transform = fixed_math::FixedTransform3D::new(
            fixed_math::FixedVec3::new(fx(12.0), fx(-8.0), Fixed::ZERO),
            fixed_math::FixedMat3::IDENTITY,
            fixed_math::FixedVec3::ONE,
        );
        app.world_mut().spawn((
            GgrsNetId(3, "kamikaze".into()),
            transform,
            sim_core::team::Team::Enemies,
            Health {
                current: fx(50.0),
                max: fx(50.0),
                invulnerable_until_frame: None,
            },
            Modifiers::default(),
        ));
        app.update();
        let world = app.world_mut();
        let bombs: Vec<(world::Bomb, fixed_math::FixedVec3)> = world
            .query::<(&world::Bomb, &fixed_math::FixedTransform3D)>()
            .iter(world)
            .map(|(bomb, t)| (bomb.clone(), t.translation))
            .collect();
        assert_eq!(bombs.len(), 1);
        let (bomb, at) = &bombs[0];
        assert_eq!(
            bomb.owner,
            GgrsNetId(3, "kamikaze".into()),
            "crédité au porteur"
        );
        assert_eq!(
            bomb.team,
            sim_core::team::Team::Enemies,
            "équipe du porteur"
        );
        assert_eq!(bomb.fuse_until, 51, "mèche d'une frame");
        assert!(bomb.is_due(51) && !bomb.is_due(50));
        assert_eq!((bomb.radius, bomb.damage), (fx(40.0), fx(7.0)));
        assert!(bomb.destroys);
        assert_eq!((at.x, at.y), (fx(12.0), fx(-8.0)), "au point du porteur");
    }

    use std::hash::{Hash, Hasher};

    fn hash_of<T: Hash>(value: &T) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn effect_state_sans_champ_v2_garde_le_hash_de_t1_10() {
        let mut state = EffectState {
            posed: vec![3, 9],
            last_hit_frame: Some(12),
            pending: vec![PendingTrigger::LevelUp],
            ..EffectState::default()
        };
        // Le hash d'avant M2-T3 : les trois champs de T1.10, dans l'ordre
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        state.posed.hash(&mut hasher);
        state.last_hit_frame.hash(&mut hasher);
        state.pending.hash(&mut hasher);
        assert_eq!(hash_of(&state), hasher.finish());
        state.floor_seen = Some(0);
        assert_ne!(
            hash_of(&state),
            hasher.finish(),
            "champ v2 non vide : haché"
        );
    }

    fn floor_app(frame: u32, floor_index: u32) -> App {
        let mut app = App::new();
        app.init_resource::<bevy_ggrs::RollbackOrdered>()
            .init_resource::<WeaponRegistry>()
            .insert_resource(FrameCount { frame })
            .init_resource::<FrameEvents<EffectOut>>()
            .insert_resource(run::FloorState {
                index: floor_index,
                ..Default::default()
            })
            .add_systems(Update, apply_triggered_effects_system);
        app
    }

    fn floor_carrier(app: &mut App, effect: Effect) -> Entity {
        app.world_mut()
            .spawn((
                GgrsNetId(1, "pilote".into()),
                Effects(vec![effect]),
                EffectState::default(),
                Health {
                    current: fx(50.0),
                    max: fx(100.0),
                    invulnerable_until_frame: None,
                },
                Modifiers::default(),
                Rollback,
            ))
            .id()
    }

    fn heal_on_floor_start() -> Effect {
        Effect::new(On::OnFloorStart, vec![], vec![Action::Heal(fx(5.0))])
    }

    #[test]
    fn debut_d_etage_au_depart_compte_le_premier_etage() {
        let mut app = floor_app(0, 0);
        let carrier = floor_carrier(&mut app, heal_on_floor_start());
        app.update();
        assert_eq!(
            app.world().get::<Health>(carrier).unwrap().current,
            fx(55.0)
        );
        // Deuxième passe à la même frame d'étage : rien de plus
        app.update();
        assert_eq!(
            app.world().get::<Health>(carrier).unwrap().current,
            fx(55.0)
        );
        // L'index change : l'étage suivant déclenche
        app.world_mut().resource_mut::<run::FloorState>().index = 1;
        app.update();
        assert_eq!(
            app.world().get::<Health>(carrier).unwrap().current,
            fx(60.0)
        );
    }

    #[test]
    fn effet_pose_apres_le_depart_ne_se_declenche_pas_au_premier_etage() {
        let mut app = floor_app(500, 2);
        let carrier = floor_carrier(&mut app, heal_on_floor_start());
        app.update();
        assert_eq!(
            app.world().get::<Health>(carrier).unwrap().current,
            fx(50.0),
            "vu en cours d'étage : silencieux"
        );
        app.world_mut().resource_mut::<run::FloorState>().index = 3;
        app.update();
        assert_eq!(
            app.world().get::<Health>(carrier).unwrap().current,
            fx(55.0)
        );
    }

    #[test]
    fn charges_rendues_a_chaque_etage() {
        let mut app = floor_app(0, 0);
        let mut effect = Effect::new(On::OnFloorStart, vec![], vec![Action::Heal(fx(5.0))]);
        effect.charges = Some(1);
        effect.recharge = Some(Recharge::PerFloor);
        let carrier = floor_carrier(&mut app, effect);
        app.update();
        let health = |app: &App| app.world().get::<Health>(carrier).unwrap().current;
        assert_eq!(health(&app), fx(55.0));
        app.world_mut().resource_mut::<run::FloorState>().index = 1;
        app.update();
        assert_eq!(health(&app), fx(60.0), "charge rendue à l'étage 1");
    }

    /// Clés de limites (M3-T2, Q16) : `Shared` (défaut) → une seule clé pour tous les exemplaires ;
    /// `Independent` → une clé par exemplaire (`#k`). Rang et nombre d'exemplaires pour `PerCopy`.
    #[test]
    fn cles_de_limites_partagees_ou_independantes() {
        fn listed_keys(stack: &str) -> Vec<(String, u32, u32)> {
            let def: items::ItemDef = ron::from_str(&format!(
                r#"(name: "A", kind: Passive, stack: {stack}, effects: [
                    (on: OnKill, charges: Some(1), do: [Scaled(action: Heal("1"), scale: Some(PerCopy))]),
                ])"#
            ))
            .unwrap();
            let table = ItemTable([("amulette".to_string(), def)].into_iter().collect());
            let inventory = Inventory {
                passives: vec!["amulette".into(), "amulette".into()],
                ..Default::default()
            };
            let mut state = EffectState::default();
            listed_effects(
                None,
                Some(&inventory),
                Some(&table),
                None,
                &GgrsNetId(1, "t".into()),
                &mut state,
                10,
                false,
            )
            .iter()
            .map(|l| (l.key.clone(), l.copy_rank, l.copies))
            .collect()
        }
        assert_eq!(
            listed_keys("Shared"),
            vec![
                ("item:amulette:0".to_string(), 0, 2),
                ("item:amulette:0".to_string(), 1, 2)
            ]
        );
        assert_eq!(
            listed_keys("Independent"),
            vec![
                ("item:amulette:0".to_string(), 0, 2),
                ("item:amulette#1:0".to_string(), 1, 2)
            ]
        );
    }
}
