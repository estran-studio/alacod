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
use combat::downed::Downed;
use combat::emitter::Emitter;
use combat::projectile::{resolve_pattern, PatternLibrary};
use effects::runtime::{
    conditions_hold, healed, limits_allow, limits_consume, limits_recharge, responds_to, tick_due,
    Carrier, Limits, Trigger,
};
use effects::{Action, Effect, On, Recharge};
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
type AliveQuery<'w, 's> = Query<
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

/// Un effet à exécuter pour un porteur : le sien ou celui d'un objet tenu.
struct Listed<'a> {
    /// Clé de `EffectState::limits` : `effect:<index>` ou `item:<id>:<index>`.
    key: String,
    source: ModifierSource,
    /// Index dans le journal de trace (`index=`) : celui d'`Effects`, sinon celui du `ItemDef`.
    index: usize,
    effect: &'a Effect,
    /// Frame depuis laquelle `Tick` compte.
    posed: u32,
}

/// Les effets d'un porteur dans l'ordre : ceux d'[`Effects`] par index, puis ceux des objets
/// (passifs dans l'ordre de ramassage, puis l'actif) par index de `ItemDef`, `OnUse` exclu
/// (exécuté par `game::items`).
fn listed_effects<'a>(
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
            });
        }
    }
    if let (Some(inventory), Some(table)) = (inventory, table) {
        let held = inventory
            .passives
            .iter()
            .map(String::as_str)
            .chain(inventory.active.iter().map(|slot| slot.item.as_str()));
        for id in held {
            let Some(def) = table.0.get(id) else { continue };
            for (index, effect) in def.effects.iter().enumerate() {
                if effect.on == On::OnUse {
                    continue;
                }
                let key = format!("item:{id}:{index}");
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
                });
            }
        }
    }
    // M2-T4 : effets des synergies actives (par id), clé `synergy:<id>:<index>`
    if let (Some(inventory), Some(synergies)) = (inventory, synergies) {
        for (id, def) in synergies.active(inventory) {
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
                });
            }
        }
    }
    listed
}

/// Ce que les effets d'un porteur peuvent modifier.
struct Body<'a, 'w, 's> {
    entity: Entity,
    has_emitter: bool,
    health: &'a mut Health,
    modifiers: &'a mut Modifiers,
    gauges: Option<&'a mut Gauges>,
    commands: &'a mut Commands<'w, 's>,
}

/// Lecture seule du porteur pour les conditions.
struct CarrierView<'a> {
    tags: &'a Tags,
    carried: &'a [&'a str],
    squad_size: u32,
}

/// Exécute les effets de `listed` qui répondent à `triggers` (et, si `tick`, à leur `Tick`).
/// Ordre : déclencheurs, puis effets par ordre de liste. Les recharges (`PerRoom` sur
/// `RoomCleared`, `PerFloor` sur `FloorStart`) précèdent les exécutions de la frame.
#[allow(clippy::too_many_arguments)]
fn run_listed(
    listed: &[Listed],
    triggers: &[Trigger],
    tick: bool,
    frame: u32,
    net_id: &GgrsNetId,
    view: &CarrierView,
    state: &mut EffectState,
    body: &mut Body,
    assets: &EffectAssets,
) {
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
            .filter(|t| responds_to(&effect.on, t))
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
            for action in &effect.r#do {
                apply_action_sourced(
                    action,
                    net_id,
                    entry.source.clone(),
                    frame,
                    body.health,
                    body.modifiers,
                    state,
                    body.gauges.as_deref_mut(),
                    (body.entity, body.has_emitter),
                    body.commands,
                    assets,
                );
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
    table: Option<Res<ItemTable>>,
    synergies: Option<Res<SynergyTable>>,
    assets: EffectAssets,
) {
    if carriers.is_empty() {
        return;
    }
    let frame = frame.frame;
    let squad_size = alive.iter().count() as u32;

    let kills = kills_by_killer(&dead, &others);
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
        _transform,
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
        let carried = carried_items(inventory);
        let view = CarrierView {
            tags: tags.unwrap_or(&no_tags),
            carried: &carried,
            squad_size,
        };
        let mut body = Body {
            entity,
            has_emitter,
            health: &mut health,
            modifiers: &mut modifiers,
            gauges: gauges.as_deref_mut(),
            commands: &mut commands,
        };
        run_listed(
            &listed, &triggers, true, frame, net_id, &view, state, &mut body, &assets,
        );
        if !had_state && local != EffectState::default() {
            commands.entity(entity).insert(local);
        }
    }
}

/// Objets tenus (passifs puis actif), pour `Carrying`.
fn carried_items(inventory: Option<&Inventory>) -> Vec<&str> {
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
    table: Option<Res<ItemTable>>,
    synergies: Option<Res<SynergyTable>>,
    assets: EffectAssets,
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
            };
            let mut body = Body {
                entity,
                has_emitter,
                health: &mut health,
                modifiers: &mut modifiers,
                gauges: gauges.as_deref_mut(),
                commands: &mut commands,
            };
            run_listed(
                &listed, &triggers, false, frame, net_id, &view, state, &mut body, &assets,
            );
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
        health,
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
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_action_sourced(
    action: &Action,
    net_id: &GgrsNetId,
    source: ModifierSource,
    frame: u32,
    health: &mut Health,
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
            health.current = healed(health.current, health.max, *amount);
        }
        Action::GainShield(points) => {
            combat::actors::queue_gain_shield(commands, entity, *points);
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
        Action::RefillAmmo
        | Action::RefillAmmoOf(_)
        | Action::RepairAllWindows
        | Action::KillAllWaveEnemies
        | Action::DestroyTerrain { .. }
        | Action::ApplyStatus { .. }
        // M2-T7 : frise de boss seulement (exécutées par `game::boss`).
        | Action::SpawnEnemies { .. }
        | Action::Telegraph(_)
        // M3-T0c : contrat seul, exécutée par M3-T4 (le lint la refuse d'ici là)
        | Action::Explode { .. }
        // M3-T0d : mort d'un champion, exécutée par M3-T7 (refusée dans un effet par le lint)
        | Action::SpawnCharacters { .. }
        | Action::Scaled { .. } => {
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
            .init_resource::<Assets<WeaponsConfig>>()
            .insert_resource(FrameCount { frame })
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
}
