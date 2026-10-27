//! Exécution v1 des effets (T1.10, `docs/conventions.md` §27) : règles pures, sans ECS. Le
//! système (`game::effects_runtime`) construit les [`Trigger`] de la frame et le
//! [`Carrier`] de chaque porteur, puis applique les actions des effets qui se déclenchent.

use bevy_fixed::fixed_math::Fixed;
use sim_core::tag::{Tag, Tags};

use crate::{Condition, Effect, GaugeThreshold, On, TimingGradeFilter, TimingReference};

/// Déclencheurs exécutés (v1 + M2-T3 : `OnRoomClear`, `OnDodge`, `OnReload`, `OnPickup`,
/// `OnFloorStart`, `OnBlank` + M3-T2 : `OnHit`, `OnEvent`). Seul `OnUse` hors objet actif reste un
/// contrat : le lint le refuse.
pub fn trigger_supported(on: &On) -> bool {
    matches!(
        on,
        On::OnKill
            | On::OnDamageTaken
            | On::Tick(_)
            | On::OnGauge(..)
            | On::OnLevelUp
            | On::OnRoomClear
            | On::OnDodge
            | On::OnReload
            | On::OnPickup
            | On::OnFloorStart
            | On::OnBlank
            | On::OnHit
            | On::OnEvent(_)
    )
}

/// Conditions exécutées (v1 + M2-T3 : `Carrying`, `SquadSize` + M3-T2 : `TargetInRange`, lue sur la
/// cible de `OnHit`, `OnKill` et `OnDamageTaken` ; fausse sans cible).
pub fn condition_supported(condition: &Condition) -> bool {
    matches!(
        condition,
        Condition::HpBelow(_)
            | Condition::HasTag(_)
            | Condition::TargetTag(_)
            | Condition::NotHitFor(_)
            | Condition::Carrying(_)
            | Condition::SquadSize(_)
            | Condition::TargetInRange(_)
            | Condition::Timing { .. }
    )
}

/// Repères temporels du porteur (X-3, `Condition::Timing`), calculés par l'application à
/// chaque évaluation : début du dash et du coup de mêlée en cours (`None` hors action) et
/// occurrence la plus proche d'un événement d'horloge (`beat`, `None` : horloge ou événement
/// inconnu). Tout en frames absolues de simulation.
#[derive(Clone, Copy, Default)]
pub struct TimingRefs<'a> {
    pub dash_start: Option<u32>,
    pub melee_start: Option<u32>,
    pub beat: Option<&'a dyn Fn(&str) -> Option<u32>>,
}

impl std::fmt::Debug for TimingRefs<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TimingRefs")
            .field("dash_start", &self.dash_start)
            .field("melee_start", &self.melee_start)
            .field("beat", &self.beat.map(|_| "<fn>"))
            .finish()
    }
}

impl TimingRefs<'_> {
    /// Frame de `reference` (avant `offset`), `None` si inconnue.
    pub fn frame_of(&self, reference: &TimingReference) -> Option<u32> {
        match reference {
            TimingReference::Clock(id) => self.beat.and_then(|beat| beat(id)),
            TimingReference::DashStart => self.dash_start,
            TimingReference::MeleeStart => self.melee_start,
        }
    }
}

/// `Condition::Timing` : la frame `now` tombe-t-elle dans la note `grade` autour de
/// `repère + offset` ? Repère inconnu : faux (jamais de panique, aucune note inventée).
pub fn timing_holds(
    refs: &TimingRefs,
    now: u32,
    reference: &TimingReference,
    offset: u32,
    perfect_frames: u32,
    good_frames: u32,
    grade: TimingGradeFilter,
) -> bool {
    use sim_core::timing::{grade_timing, TimingGrade};
    let Some(base) = refs.frame_of(reference) else {
        return false;
    };
    let noted = grade_timing(
        now,
        base.saturating_add(offset),
        perfect_frames,
        good_frames,
    );
    match grade {
        TimingGradeFilter::Perfect => noted == TimingGrade::Perfect,
        TimingGradeFilter::AtLeastGood => noted != TimingGrade::Miss,
        TimingGradeFilter::Miss => noted == TimingGrade::Miss,
    }
}

/// La cible d'un déclencheur (`OnHit`, `OnKill`, `OnDamageTaken`) : son `GgrsNetId` et sa position
/// monde (`None` si inconnue), pour `TargetInRange` et `ApplyStatus` (M3-T2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetRef {
    pub id: usize,
    pub position: Option<(Fixed, Fixed)>,
}

/// Ce qui est arrivé au porteur dans la frame.
#[derive(Debug, Clone)]
pub enum Trigger {
    /// Le porteur a tué une cible portant ces tags (`target` : la victime, M3-T2).
    Kill {
        target_tags: Tags,
        target: Option<TargetRef>,
    },
    /// Le porteur a subi des dégâts ; tags de la source (vides si inconnue) ; `target` : la source.
    DamageTaken {
        source_tags: Tags,
        target: Option<TargetRef>,
    },
    /// M3-T2 : le porteur a touché `target` (dégât réel, après résolution), de tags `target_tags`.
    Hit {
        target_tags: Tags,
        target: TargetRef,
    },
    /// M3-T2 : un effet du porteur a émis l'événement nommé `name` (`Action::Emit`).
    Event { name: String },
    /// La jauge `id` est passée de `before` à `after` (`OnGauge(id, Above(x))` : franchie à
    /// la hausse si `before < x <= after`).
    GaugeMoved {
        id: String,
        before: Fixed,
        after: Fixed,
    },
    /// Le porteur est monté de niveau.
    LevelUp,
    /// M2-T3 : une salle a été nettoyée (`RoomChanged` vers `Cleared`).
    RoomCleared,
    /// M2-T3 : le porteur a commencé un dash.
    Dodge,
    /// M2-T3 : le porteur a fini de recharger.
    Reload,
    /// M2-T3 : le porteur a ramassé l'objet `item`.
    Pickup { item: String },
    /// M2-T3 : un étage commence pour le porteur.
    FloorStart,
    /// M2-T3 : le porteur a utilisé une Blank (`BlankUsed`, M2-T1).
    Blank,
}

/// État du porteur lu par les conditions.
#[derive(Debug, Clone)]
pub struct Carrier<'a> {
    pub tags: &'a Tags,
    pub health: Fixed,
    pub health_max: Fixed,
    pub frame: u32,
    /// Dernière frame où le porteur a subi des dégâts (`None` : jamais).
    pub last_hit_frame: Option<u32>,
    /// Objets tenus (passifs et actif), pour `Carrying`.
    pub carried: &'a [&'a str],
    /// Joueurs en vie dans la partie, pour `SquadSize`.
    pub squad_size: u32,
    /// M3-T2 : position monde du porteur, pour `TargetInRange` (`None` : inconnue, condition fausse).
    pub position: Option<(Fixed, Fixed)>,
    /// X-3 : repères temporels du porteur, pour `Timing` (`TimingRefs::default()` : tous
    /// inconnus, la condition est fausse).
    pub timing: TimingRefs<'a>,
}

/// L'effet répond-il à ce déclencheur : [`responds_to`] et, pour `OnPickup`, son filtre `item`
/// (M3-T2 : absent, tout ramassage).
pub fn effect_responds(effect: &Effect, trigger: &Trigger) -> bool {
    if !responds_to(&effect.on, trigger) {
        return false;
    }
    match (&effect.item, trigger) {
        (Some(wanted), Trigger::Pickup { item }) => wanted == item,
        _ => true,
    }
}

/// L'effet répond-il à ce déclencheur (hors `Tick`, voir [`tick_due`]) ?
pub fn responds_to(on: &On, trigger: &Trigger) -> bool {
    match (on, trigger) {
        (On::OnKill, Trigger::Kill { .. }) => true,
        (On::OnDamageTaken, Trigger::DamageTaken { .. }) => true,
        (On::OnHit, Trigger::Hit { .. }) => true,
        (On::OnEvent(wanted), Trigger::Event { name }) => wanted == name,
        (On::OnLevelUp, Trigger::LevelUp) => true,
        (On::OnRoomClear, Trigger::RoomCleared) => true,
        (On::OnDodge, Trigger::Dodge) => true,
        (On::OnReload, Trigger::Reload) => true,
        (On::OnPickup, Trigger::Pickup { .. }) => true,
        (On::OnFloorStart, Trigger::FloorStart) => true,
        (On::OnBlank, Trigger::Blank) => true,
        (
            On::OnGauge(id, GaugeThreshold::Above(x)),
            Trigger::GaugeMoved {
                id: g,
                before,
                after,
            },
        ) => id == g && *before < *x && *x <= *after,
        _ => false,
    }
}

/// `Tick(n)` : toutes les `n` frames depuis la pose de l'effet (`posed_frame`), jamais à la
/// frame de pose elle-même ; `n = 0` ne se déclenche jamais (refusé par le lint).
pub fn tick_due(on: &On, posed_frame: u32, frame: u32) -> bool {
    match on {
        On::Tick(n) if *n > 0 && frame > posed_frame => (frame - posed_frame) % n == 0,
        _ => false,
    }
}

/// Toutes les conditions (ET). `TargetTag` lit la cible d'`OnKill` ou la source
/// d'`OnDamageTaken` (faux sans cible). Une condition non supportée est fausse.
pub fn conditions_hold(
    conditions: &[Condition],
    carrier: &Carrier,
    trigger: Option<&Trigger>,
) -> bool {
    conditions.iter().all(|condition| match condition {
        Condition::HpBelow(fraction) => {
            carrier.health_max > Fixed::ZERO && carrier.health < carrier.health_max * *fraction
        }
        Condition::HasTag(tag) => carrier.tags.has(tag),
        Condition::TargetTag(tag) => target_tags(trigger).is_some_and(|tags| tags.has(tag)),
        Condition::NotHitFor(n) => carrier
            .last_hit_frame
            .is_none_or(|hit| carrier.frame.saturating_sub(hit) >= *n),
        Condition::Carrying(item) => carrier.carried.iter().any(|c| c == item),
        Condition::SquadSize(n) => carrier.squad_size >= *n,
        Condition::TargetInRange(range) => {
            target_in_range(carrier.position, target_position(trigger), *range)
        }
        Condition::Timing {
            reference,
            offset,
            perfect_frames,
            good_frames,
            grade,
        } => timing_holds(
            &carrier.timing,
            carrier.frame,
            reference,
            *offset,
            *perfect_frames,
            *good_frames,
            *grade,
        ),
    })
}

fn target_tags(trigger: Option<&Trigger>) -> Option<&Tags> {
    match trigger? {
        Trigger::Kill { target_tags, .. } => Some(target_tags),
        Trigger::DamageTaken { source_tags, .. } => Some(source_tags),
        Trigger::Hit { target_tags, .. } => Some(target_tags),
        _ => None,
    }
}

/// La cible du déclencheur, s'il en a une (`OnHit`, `OnKill`, `OnDamageTaken`).
pub fn trigger_target(trigger: Option<&Trigger>) -> Option<&TargetRef> {
    match trigger? {
        Trigger::Kill { target, .. } | Trigger::DamageTaken { target, .. } => target.as_ref(),
        Trigger::Hit { target, .. } => Some(target),
        _ => None,
    }
}

fn target_position(trigger: Option<&Trigger>) -> Option<(Fixed, Fixed)> {
    trigger_target(trigger)?.position
}

/// `TargetInRange(range)` : la cible est à **au plus** `range` du porteur (distance au carré en
/// `FixedWide`, sans racine) ; faux si l'une des positions manque ou si `range` est négatif.
pub fn target_in_range(
    carrier: Option<(Fixed, Fixed)>,
    target: Option<(Fixed, Fixed)>,
    range: Fixed,
) -> bool {
    use bevy_fixed::fixed_math::FixedWide;
    let (Some((cx, cy)), Some((tx, ty))) = (carrier, target) else {
        return false;
    };
    if range < Fixed::ZERO {
        return false;
    }
    let dx = FixedWide::from_num(tx) - FixedWide::from_num(cx);
    let dy = FixedWide::from_num(ty) - FixedWide::from_num(cy);
    let r = FixedWide::from_num(range);
    dx * dx + dy * dy <= r * r
}

/// Soin borné : `health + amount`, au plus `health_max`, jamais en dessous de `health`.
pub fn healed(health: Fixed, health_max: Fixed, amount: Fixed) -> Fixed {
    if amount <= Fixed::ZERO {
        return health;
    }
    (health + amount).min(health_max).max(health)
}

/// État de limite d'un effet pour un porteur (M2-T3) : `cooldown_until` = première frame où il
/// peut de nouveau s'exécuter, `charges_left` = charges restantes (`None` : pleines).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Limits {
    pub cooldown_until: Option<u32>,
    pub charges_left: Option<u32>,
}

impl Limits {
    /// Vrai si l'état est celui d'un effet jamais limité (rien à stocker).
    pub fn is_default(&self) -> bool {
        self.cooldown_until.is_none() && self.charges_left.is_none()
    }
}

/// L'effet peut-il s'exécuter à `frame` ? Faux pendant le cooldown (`frame < cooldown_until`)
/// ou sans charge restante. Un effet sans `cooldown` ni `charges` répond toujours vrai.
pub fn limits_allow(effect: &Effect, limits: &Limits, frame: u32) -> bool {
    if effect.cooldown.is_some() && limits.cooldown_until.is_some_and(|until| frame < until) {
        return false;
    }
    if let Some(max) = effect.charges {
        if limits.charges_left.unwrap_or(max) == 0 {
            return false;
        }
    }
    true
}

/// Consomme une exécution : le cooldown repart de `frame`, une charge est retirée.
pub fn limits_consume(effect: &Effect, limits: &mut Limits, frame: u32) {
    if let Some(cooldown) = effect.cooldown {
        limits.cooldown_until = Some(frame.saturating_add(cooldown));
    }
    if let Some(max) = effect.charges {
        limits.charges_left = Some(limits.charges_left.unwrap_or(max).saturating_sub(1));
    }
}

/// Redonne toutes ses charges à l'effet si sa recharge est `recharge`.
pub fn limits_recharge(effect: &Effect, limits: &mut Limits, recharge: crate::Recharge) {
    if effect.charges.is_some() && effect.recharge == Some(recharge) {
        limits.charges_left = None;
    }
}

/// L'effet est-il exécutable en v1 (déclencheur et conditions supportés) ?
pub fn effect_supported(effect: &Effect) -> bool {
    trigger_supported(&effect.on) && effect.r#if.iter().all(condition_supported)
}

/// Raccourci de tags pour les tests et le contenu.
pub fn tags_of(names: &[&str]) -> Tags {
    let mut tags = Tags::new();
    for name in names {
        tags.insert(Tag::new(*name));
    }
    tags
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fx(v: f32) -> Fixed {
        Fixed::from_num(v)
    }

    fn carrier(tags: &Tags, health: f32, last_hit: Option<u32>) -> Carrier<'_> {
        Carrier {
            tags,
            health: fx(health),
            health_max: fx(100.0),
            frame: 200,
            last_hit_frame: last_hit,
            carried: &["fiole"],
            squad_size: 2,
            position: Some((fx(0.0), fx(0.0))),
            timing: TimingRefs::default(),
        }
    }

    #[test]
    fn chaque_declencheur_v1() {
        let kill = Trigger::Kill {
            target_tags: tags_of(&["zombie"]),
            target: None,
        };
        let hurt = Trigger::DamageTaken {
            source_tags: Tags::new(),
            target: None,
        };
        assert!(responds_to(&On::OnKill, &kill));
        assert!(!responds_to(&On::OnKill, &hurt));
        assert!(responds_to(&On::OnDamageTaken, &hurt));
        assert!(responds_to(&On::OnLevelUp, &Trigger::LevelUp));
        let gauge = Trigger::GaugeMoved {
            id: "rads".into(),
            before: fx(1.0),
            after: fx(2.0),
        };
        assert!(responds_to(
            &On::OnGauge("rads".into(), GaugeThreshold::Above(fx(2.0))),
            &gauge
        ));
        assert!(
            !responds_to(
                &On::OnGauge("rads".into(), GaugeThreshold::Above(fx(1.0))),
                &gauge
            ),
            "déjà au-dessus avant"
        );
        assert!(!responds_to(
            &On::OnGauge("rads".into(), GaugeThreshold::Above(fx(4.0))),
            &gauge
        ));
        assert!(!responds_to(
            &On::OnGauge("autre".into(), GaugeThreshold::Above(fx(2.0))),
            &gauge
        ));
        assert!(trigger_supported(&On::OnHit));
        assert!(trigger_supported(&On::OnEvent("x".into())));
        assert!(!trigger_supported(&On::OnUse));
    }

    #[test]
    fn tick_depuis_la_pose() {
        let on = On::Tick(120);
        assert!(!tick_due(&on, 30, 30), "pas à la pose");
        assert!(!tick_due(&on, 30, 149));
        assert!(tick_due(&on, 30, 150));
        assert!(tick_due(&on, 30, 270));
        assert!(!tick_due(&On::Tick(0), 0, 10));
    }

    #[test]
    fn conditions_v1() {
        let tags = tags_of(&["pilote"]);
        let kill = Trigger::Kill {
            target_tags: tags_of(&["zombie"]),
            target: None,
        };
        let c = carrier(&tags, 40.0, Some(150));
        assert!(conditions_hold(&[Condition::HpBelow(fx(0.5))], &c, None));
        assert!(!conditions_hold(&[Condition::HpBelow(fx(0.4))], &c, None));
        assert!(conditions_hold(
            &[Condition::HasTag(Tag::new("pilote"))],
            &c,
            None
        ));
        assert!(conditions_hold(
            &[Condition::TargetTag(Tag::new("zombie"))],
            &c,
            Some(&kill)
        ));
        assert!(!conditions_hold(
            &[Condition::TargetTag(Tag::new("zombie"))],
            &c,
            None
        ));
        // NotHitFor : touché à f150, on est à f200
        assert!(conditions_hold(&[Condition::NotHitFor(50)], &c, None));
        assert!(!conditions_hold(&[Condition::NotHitFor(51)], &c, None));
        assert!(conditions_hold(
            &[Condition::NotHitFor(1000)],
            &carrier(&tags, 40.0, None),
            None
        ));
        // ET ; `TargetInRange` sans cible : fausse
        assert!(!conditions_hold(
            &[
                Condition::HpBelow(fx(0.5)),
                Condition::TargetInRange(fx(5.0))
            ],
            &c,
            None
        ));
    }

    #[test]
    fn declencheurs_et_conditions_v2() {
        assert!(responds_to(&On::OnRoomClear, &Trigger::RoomCleared));
        assert!(responds_to(&On::OnDodge, &Trigger::Dodge));
        assert!(responds_to(&On::OnReload, &Trigger::Reload));
        assert!(responds_to(&On::OnFloorStart, &Trigger::FloorStart));
        assert!(responds_to(
            &On::OnPickup,
            &Trigger::Pickup { item: "x".into() }
        ));
        assert!(!responds_to(&On::OnDodge, &Trigger::Reload));
        for on in [
            On::OnRoomClear,
            On::OnDodge,
            On::OnReload,
            On::OnPickup,
            On::OnFloorStart,
            On::OnBlank,
        ] {
            assert!(trigger_supported(&on), "{on:?}");
        }
        assert!(condition_supported(&Condition::TargetInRange(fx(10.0))));
        let tags = Tags::new();
        let c = carrier(&tags, 40.0, None);
        assert!(conditions_hold(
            &[Condition::Carrying("fiole".into())],
            &c,
            None
        ));
        assert!(!conditions_hold(
            &[Condition::Carrying("autre".into())],
            &c,
            None
        ));
        assert!(conditions_hold(&[Condition::SquadSize(2)], &c, None));
        assert!(!conditions_hold(&[Condition::SquadSize(3)], &c, None));
    }

    #[test]
    fn cooldown_et_charges() {
        let mut effect = Effect::new(On::OnReload, vec![], vec![]);
        effect.cooldown = Some(100);
        effect.charges = Some(2);
        effect.recharge = Some(crate::Recharge::PerRoom);
        let mut limits = Limits::default();
        assert!(limits_allow(&effect, &limits, 10));
        limits_consume(&effect, &mut limits, 10);
        assert!(!limits_allow(&effect, &limits, 109), "cooldown");
        assert!(limits_allow(&effect, &limits, 110));
        limits_consume(&effect, &mut limits, 110);
        assert!(!limits_allow(&effect, &limits, 500), "plus de charge");
        limits_recharge(&effect, &mut limits, crate::Recharge::PerFloor);
        assert!(!limits_allow(&effect, &limits, 500), "autre recharge");
        limits_recharge(&effect, &mut limits, crate::Recharge::PerRoom);
        assert!(limits_allow(&effect, &limits, 500));
        // Sans champs v2 : jamais limité
        let free = Effect::new(On::OnReload, vec![], vec![]);
        let mut l = Limits::default();
        limits_consume(&free, &mut l, 5);
        assert!(l.is_default() && limits_allow(&free, &l, 5));
    }

    #[test]
    fn soin_borne() {
        assert_eq!(healed(fx(90.0), fx(100.0), fx(25.0)), fx(100.0));
        assert_eq!(healed(fx(50.0), fx(100.0), fx(10.0)), fx(60.0));
        assert_eq!(healed(fx(50.0), fx(100.0), fx(-5.0)), fx(50.0));
        // Au-dessus du max (max réduit) : jamais rabaissé par un soin
        assert_eq!(healed(fx(120.0), fx(100.0), fx(5.0)), fx(120.0));
    }

    fn hit_at(id: usize, x: f32, y: f32) -> Trigger {
        Trigger::Hit {
            target_tags: tags_of(&["zombie"]),
            target: TargetRef {
                id,
                position: Some((fx(x), fx(y))),
            },
        }
    }

    #[test]
    fn on_hit_on_event_et_filtre_de_ramassage() {
        let hit = hit_at(7, 3.0, 4.0);
        assert!(responds_to(&On::OnHit, &hit));
        assert!(!responds_to(&On::OnKill, &hit));
        let event = Trigger::Event { name: "tic".into() };
        assert!(responds_to(&On::OnEvent("tic".into()), &event));
        assert!(!responds_to(&On::OnEvent("autre".into()), &event));
        assert!(!responds_to(&On::OnEvent("tic".into()), &hit));
        // OnPickup filtré par objet
        let pick = Trigger::Pickup {
            item: "bottes".into(),
        };
        let any = Effect::new(On::OnPickup, vec![], vec![]);
        let mut only_boots = Effect::new(On::OnPickup, vec![], vec![]);
        only_boots.item = Some("bottes".into());
        let mut only_lame = only_boots.clone();
        only_lame.item = Some("lame".into());
        assert!(effect_responds(&any, &pick));
        assert!(effect_responds(&only_boots, &pick));
        assert!(!effect_responds(&only_lame, &pick));
        // le filtre ne concerne que `Pickup`
        assert!(effect_responds(
            &only_boots,
            &Trigger::Pickup {
                item: "bottes".into()
            }
        ));
    }

    #[test]
    fn target_in_range_distance_inclusive_et_positions_manquantes() {
        let tags = Tags::new();
        let c = carrier(&tags, 40.0, None);
        let in_range = |r: f32, trigger: &Trigger| {
            conditions_hold(&[Condition::TargetInRange(fx(r))], &c, Some(trigger))
        };
        let hit = hit_at(7, 3.0, 4.0); // à 5 du porteur (0, 0)
        assert!(in_range(5.0, &hit), "à exactement 5 : à portée");
        assert!(!in_range(4.9, &hit));
        assert!(in_range(100.0, &hit));
        // OnKill et OnDamageTaken portent aussi une cible
        let kill = Trigger::Kill {
            target_tags: Tags::new(),
            target: Some(TargetRef {
                id: 1,
                position: Some((fx(30.0), fx(0.0))),
            }),
        };
        assert!(in_range(30.0, &kill) && !in_range(29.0, &kill));
        // pas de cible, pas de position (cible ou porteur) : faux
        assert!(!conditions_hold(
            &[Condition::TargetInRange(fx(1000.0))],
            &c,
            None
        ));
        let no_pos = Trigger::Hit {
            target_tags: Tags::new(),
            target: TargetRef {
                id: 2,
                position: None,
            },
        };
        assert!(!in_range(1000.0, &no_pos));
        let mut blind = c.clone();
        blind.position = None;
        assert!(!conditions_hold(
            &[Condition::TargetInRange(fx(1000.0))],
            &blind,
            Some(&hit)
        ));
        assert!(!target_in_range(
            Some((fx(0.0), fx(0.0))),
            Some((fx(0.0), fx(0.0))),
            fx(-1.0)
        ));
        assert_eq!(trigger_target(Some(&hit)).map(|t| t.id), Some(7));
        assert_eq!(trigger_target(Some(&Trigger::Reload)), None);
    }

    /// X-3 : `Condition::Timing` — bornes des fenêtres, décalage, filtres de note, repère
    /// inconnu (faux, jamais de panique) et repères de dash, de coup et d'horloge.
    #[test]
    fn fenetre_de_jugement() {
        use crate::{TimingGradeFilter as G, TimingReference as R};
        let tags = Tags::new();
        let beat = |id: &str| (id == "pouls").then_some(196);
        let refs = TimingRefs {
            dash_start: Some(190),
            melee_start: None,
            beat: Some(&beat),
        };
        let carrier = Carrier {
            timing: refs,
            ..carrier(&tags, 100.0, None)
        };
        // frame du porteur : 200
        let t = |reference: R, offset, perfect, good, grade| Condition::Timing {
            reference,
            offset,
            perfect_frames: perfect,
            good_frames: good,
            grade,
        };
        let holds = |c: Condition| conditions_hold(&[c], &carrier, None);
        // Horloge : repère 196, écart 4 → parfait avec perfect = 4, bien avec perfect = 3
        assert!(holds(t(R::Clock("pouls".into()), 0, 4, 7, G::Perfect)));
        assert!(!holds(t(R::Clock("pouls".into()), 0, 3, 7, G::Perfect)));
        assert!(holds(t(R::Clock("pouls".into()), 0, 3, 7, G::AtLeastGood)));
        // Hors de la fenêtre large : « raté »
        assert!(holds(t(R::Clock("pouls".into()), 0, 1, 3, G::Miss)));
        assert!(!holds(t(R::Clock("pouls".into()), 0, 1, 4, G::Miss)));
        // Décalage : début du dash 190 + 10 = 200 : écart 0
        assert!(holds(t(R::DashStart, 10, 0, 0, G::Perfect)));
        assert!(!holds(t(R::DashStart, 9, 0, 0, G::Perfect)));
        // Repères inconnus : toutes les notes sont fausses, `Miss` comprise
        assert!(!holds(t(R::MeleeStart, 0, 9, 9, G::Perfect)));
        assert!(!holds(t(R::MeleeStart, 0, 9, 9, G::Miss)));
        assert!(!holds(t(
            R::Clock("inconnu".into()),
            0,
            9,
            9,
            G::AtLeastGood
        )));
        // Sans repères du tout
        let nu = Carrier {
            timing: TimingRefs::default(),
            ..carrier
        };
        assert!(!conditions_hold(
            &[t(R::DashStart, 0, 9, 9, G::AtLeastGood)],
            &nu,
            None
        ));
        assert!(condition_supported(&t(R::DashStart, 0, 1, 2, G::Perfect)));
    }
}
