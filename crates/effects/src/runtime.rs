//! Exécution v1 des effets (T1.10, `docs/conventions.md` §27) : règles pures, sans ECS. Le
//! système (`game::effects_runtime`) construit les [`Trigger`] de la frame et le
//! [`Carrier`] de chaque porteur, puis applique les actions des effets qui se déclenchent.

use bevy_fixed::fixed_math::Fixed;
use sim_core::tag::{Tag, Tags};

use crate::{Condition, Effect, GaugeThreshold, On};

/// Déclencheurs exécutés (v1 + M2-T3 : `OnRoomClear`, `OnDodge`, `OnReload`, `OnPickup`,
/// `OnFloorStart`, `OnBlank`). Les autres
/// (`OnHit`, `OnUse` hors objet actif, `OnEvent`) restent des contrats : le lint les refuse.
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
    )
}

/// Conditions exécutées (v1 + M2-T3 : `Carrying`, `SquadSize`). `TargetInRange` reste un
/// contrat (aucune position de cible dans les déclencheurs) : le lint la refuse.
pub fn condition_supported(condition: &Condition) -> bool {
    matches!(
        condition,
        Condition::HpBelow(_)
            | Condition::HasTag(_)
            | Condition::TargetTag(_)
            | Condition::NotHitFor(_)
            | Condition::Carrying(_)
            | Condition::SquadSize(_)
    )
}

/// Ce qui est arrivé au porteur dans la frame.
#[derive(Debug, Clone)]
pub enum Trigger {
    /// Le porteur a tué une cible portant ces tags.
    Kill { target_tags: Tags },
    /// Le porteur a subi des dégâts ; tags de la source (vides si inconnue).
    DamageTaken { source_tags: Tags },
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
}

/// L'effet répond-il à ce déclencheur (hors `Tick`, voir [`tick_due`]) ?
pub fn responds_to(on: &On, trigger: &Trigger) -> bool {
    match (on, trigger) {
        (On::OnKill, Trigger::Kill { .. }) => true,
        (On::OnDamageTaken, Trigger::DamageTaken { .. }) => true,
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
        _ => false,
    })
}

fn target_tags(trigger: Option<&Trigger>) -> Option<&Tags> {
    match trigger? {
        Trigger::Kill { target_tags } => Some(target_tags),
        Trigger::DamageTaken { source_tags } => Some(source_tags),
        _ => None,
    }
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
        }
    }

    #[test]
    fn chaque_declencheur_v1() {
        let kill = Trigger::Kill {
            target_tags: tags_of(&["zombie"]),
        };
        let hurt = Trigger::DamageTaken {
            source_tags: Tags::new(),
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
        assert!(!trigger_supported(&On::OnHit));
        assert!(!trigger_supported(&On::OnEvent("x".into())));
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
        // ET ; non supportée = fausse
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
        assert!(!condition_supported(&Condition::TargetInRange(fx(10.0))));
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
}
