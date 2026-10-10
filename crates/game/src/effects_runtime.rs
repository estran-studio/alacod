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

use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_fixed::fixed_math::{self, Fixed, FixedVec2};
use bevy_ggrs::Rollback;
use combat::downed::Downed;
use combat::emitter::Emitter;
use combat::projectile::{resolve_pattern, PatternLibrary};
use effects::runtime::{conditions_hold, healed, responds_to, tick_due, Carrier, Trigger};
use effects::{Action, Effect};
use sim_core::damage::DamageEvent;
use sim_core::frame_events::FrameEvents;
use sim_core::modifier::{Modifier, ModifierSource, Modifiers};
use sim_core::tag::Tags;
use utils::frame::FrameCount;
use utils::net_id::GgrsNetId;
use utils::{order_iter, order_mut_iter};

use crate::character::health::{Death, Health, HealthRegen, HitBy};
use crate::character::player::Player;
use crate::global_asset::GlobalAsset;
use crate::weapons::WeaponsConfig;

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
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct EffectState {
    /// Frame de pose de chaque effet (même index qu'[`Effects`]) : `Tick` compte depuis elle.
    /// Un effet sans entrée est posé à la première frame où le système le voit.
    pub posed: Vec<u32>,
    /// Dernière frame où le porteur a subi des dégâts (`NotHitFor`).
    pub last_hit_frame: Option<u32>,
    /// Déclencheurs reportés (jauges, niveaux), consommés à la frame suivante.
    pub pending: Vec<PendingTrigger>,
}

/// Source du modificateur posé par l'action `index` de l'effet `effect` du porteur `carrier`.
pub fn effect_source(carrier: &GgrsNetId, effect: usize) -> ModifierSource {
    ModifierSource::Named(format!("effect:{}:{}", carrier.0, effect))
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct EffectAssets<'w> {
    global: Option<Res<'w, GlobalAsset>>,
    weapons: Res<'w, Assets<WeaponsConfig>>,
    library: Option<Res<'w, PatternLibrary>>,
}

type CarrierQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static GgrsNetId,
        Entity,
        &'static Effects,
        &'static mut EffectState,
        &'static mut Health,
        &'static mut Modifiers,
        Option<&'static mut Gauges>,
        Option<&'static Tags>,
        Option<&'static HealthRegen>,
        Option<&'static Player>,
        Has<Death>,
        Has<Downed>,
        Has<Emitter>,
    ),
    With<Rollback>,
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
    let player_net_ids: BTreeMap<usize, usize> = others
        .iter()
        .filter_map(|(id, _, player)| player.map(|p| (p.handle, id.0)))
        .collect();
    let mut kills: BTreeMap<usize, Vec<Tags>> = BTreeMap::new();
    for (_, death, tags) in order_iter!(dead) {
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
                .push(tags.cloned().unwrap_or_default());
        }
    }
    kills
}

/// Voir la doc du module.
#[allow(clippy::too_many_arguments)]
pub fn apply_effects_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    mut carriers: CarrierQuery,
    dead: DeadQuery,
    others: OthersQuery,
    damage: Option<Res<FrameEvents<DamageEvent>>>,
    assets: EffectAssets,
) {
    if carriers.is_empty() {
        return;
    }
    let frame = frame.frame;

    let kills = kills_by_killer(&dead, &others);
    let tags_by_net_id: BTreeMap<usize, Tags> = others
        .iter()
        .map(|(id, tags, _)| (id.0, tags.cloned().unwrap_or_default()))
        .collect();

    for (
        net_id,
        entity,
        effects,
        mut state,
        mut health,
        mut modifiers,
        mut gauges,
        tags,
        regen,
        _player,
        dead_now,
        downed,
        has_emitter,
    ) in order_mut_iter!(carriers)
    {
        // Pose des effets nouveaux (frame de pose de `Tick`)
        while state.posed.len() < effects.0.len() {
            state.posed.push(frame);
        }
        let pending = std::mem::take(&mut state.pending);
        if dead_now || downed {
            continue;
        }

        // Déclencheurs de la frame
        let mut triggers: Vec<Trigger> = Vec::new();
        for target_tags in kills.get(&net_id.0).into_iter().flatten() {
            triggers.push(Trigger::Kill {
                target_tags: target_tags.clone(),
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
            triggers.push(Trigger::DamageTaken { source_tags });
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
        let carrier_tags = tags.unwrap_or(&no_tags);
        for (index, effect) in effects.0.iter().enumerate() {
            let mut firings: Vec<Option<&Trigger>> = triggers
                .iter()
                .filter(|t| responds_to(&effect.on, t))
                .map(Some)
                .collect();
            if tick_due(&effect.on, state.posed[index], frame) {
                firings.push(None);
            }
            for trigger in firings {
                let carrier = Carrier {
                    tags: carrier_tags,
                    health: health.current,
                    health_max: health.max,
                    frame,
                    last_hit_frame: state.last_hit_frame,
                };
                if !conditions_hold(&effect.r#if, &carrier, trigger) {
                    continue;
                }
                info!(
                    "ggrs{{f={} effect carrier={} index={} on={:?}}}",
                    frame, net_id, index, effect.on
                );
                for action in &effect.r#do {
                    apply_action(
                        action,
                        net_id,
                        index,
                        frame,
                        &mut health,
                        &mut modifiers,
                        &mut state,
                        gauges.as_deref_mut(),
                        (entity, has_emitter),
                        &mut commands,
                        &assets,
                    );
                }
            }
        }
    }
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
    (entity, has_emitter): (Entity, bool),
    commands: &mut Commands,
    assets: &EffectAssets,
) {
    let source = effect_source(net_id, index);
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
            health.current = healed(health.current, health.max, *amount);
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
            if has_emitter {
                warn!(
                    "effet de {} : SpawnPattern « {} » ignoré, un émetteur joue déjà",
                    net_id, pattern
                );
                return;
            }
            let Some(weapon_asset) = assets
                .global
                .as_ref()
                .and_then(|g| assets.weapons.get(&g.weapons))
                .and_then(|config| config.0.get(weapon))
            else {
                warn!(
                    "effet de {} : arme « {} » inconnue (voir `alacod lint`)",
                    net_id, weapon
                );
                return;
            };
            let named = combat::projectile::Pattern::Named(pattern.clone());
            let Ok(resolved) = resolve_pattern(assets.library.as_deref(), &named) else {
                warn!(
                    "effet de {} : pattern « {} » inconnu (voir `alacod lint`)",
                    net_id, pattern
                );
                return;
            };
            commands.entity(entity).insert(Emitter::new(
                pattern.clone(),
                &resolved,
                weapon.clone(),
                std::sync::Arc::new(weapon_asset.config.projectiles.clone()),
                fixed_math::FIXED_ONE,
                weapon_asset.config.friendly_fire,
                FixedVec2::new(Fixed::ONE, Fixed::ZERO),
                frame,
            ));
        }
        Action::RefillAmmo
        | Action::RefillAmmoOf(_)
        | Action::RepairAllWindows
        | Action::KillAllWaveEnemies
        | Action::DestroyTerrain { .. }
        | Action::ApplyStatus { .. } => {
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
            Effects(vec![Effect {
                on: On::OnDamageTaken,
                r#if: vec![Condition::HpBelow(fx(0.5))],
                r#do: vec![Action::Heal(fx(5.0))],
            }]),
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
            .init_resource::<Assets<WeaponsConfig>>()
            .insert_resource(FrameCount { frame: 50 })
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
}
