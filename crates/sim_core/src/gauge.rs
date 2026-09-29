//! Jauge bornée avec plancher et seuils qui émettent des événements en la traversant
//! (santé, jauges 1837 comme le sacre ou la faim — voir `docs/plan-engine.md` §4.7).

use bevy_fixed::fixed_math::Fixed;
use serde::{Deserialize, Serialize};

use crate::team::Team;

/// Étiquette d'un événement de seuil (ex. `"sacre_plein"`), lue par les vocabulaires
/// d'effets (chantier C1, `OnGauge(id, Above(x))`) pour déclencher des effets de contenu.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GaugeEvent(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrossingDirection {
    Rising,
    Falling,
}

/// Un seuil franchi par [`Gauge::add`], avec la direction du franchissement.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GaugeCrossing {
    pub event: GaugeEvent,
    pub direction: CrossingDirection,
}

/// Valeur bornée, avec un plancher dynamique indépendant de la borne basse structurelle.
///
/// `min`/`max` sont les bornes structurelles de la jauge (ex. 0 et 100). `floor` est un
/// plancher **dynamique**, distinct de `min` : l'effet `OnEquip: GaugeFloor(sacre, +x)`
/// du plan (§4.7) le relève sans changer `min`. La borne basse effective utilisée par
/// [`Gauge::add`] est donc toujours `max(min, floor)`.
///
/// **Pas encore posée sur d'entité en T0.2** (contrainte « traces identiques » de la
/// tâche) : ni composant ni ressource enregistrés en rollback ici. Ce sera le travail
/// d'un chantier futur (B2/F1) de l'attacher et de l'enregistrer, avec le `BLESS=1` que
/// ça implique pour `tests/scenarios/*.trace`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Gauge {
    pub value: Fixed,
    pub min: Fixed,
    pub max: Fixed,
    pub floor: Fixed,
    pub thresholds: Vec<(Fixed, GaugeEvent)>,
    /// Si `Some`, la jauge est conceptuellement partagée par toute l'équipe (ex. une
    /// jauge de sacre commune en coop). Ce type ne fait que porter l'information : la
    /// propagation d'un `add` aux autres membres de l'équipe est le travail d'un futur
    /// système, pas de `Gauge` lui-même.
    pub shared: Option<Team>,
}

impl Gauge {
    /// Ajoute `delta` (peut être négatif) et clampe à `[max(min, floor), max]`. Renvoie
    /// les seuils de `thresholds` franchis par ce déplacement, dans l'ordre du vecteur :
    /// `Rising` si `ancienne_valeur < seuil <= nouvelle_valeur`, `Falling` si
    /// `nouvelle_valeur < seuil <= ancienne_valeur`. Aucun événement si la valeur ne
    /// bouge pas (delta nul ou clampée au même point).
    pub fn add(&mut self, delta: Fixed) -> Vec<GaugeCrossing> {
        let lower = self.min.max(self.floor);
        let old = self.value;

        let mut new = old.saturating_add(delta);
        if new < lower {
            new = lower;
        }
        if new > self.max {
            new = self.max;
        }
        self.value = new;

        let mut crossed = Vec::new();
        if new > old {
            for (threshold, event) in &self.thresholds {
                if old < *threshold && *threshold <= new {
                    crossed.push(GaugeCrossing {
                        event: event.clone(),
                        direction: CrossingDirection::Rising,
                    });
                }
            }
        } else if new < old {
            for (threshold, event) in &self.thresholds {
                if new < *threshold && *threshold <= old {
                    crossed.push(GaugeCrossing {
                        event: event.clone(),
                        direction: CrossingDirection::Falling,
                    });
                }
            }
        }
        crossed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fixed::fixed_math::new as fx;

    fn gauge() -> Gauge {
        Gauge {
            value: fx(50.0),
            min: fx(0.0),
            max: fx(100.0),
            floor: fx(0.0),
            thresholds: vec![
                (fx(25.0), GaugeEvent("quart".into())),
                (fx(75.0), GaugeEvent("trois_quarts".into())),
            ],
            shared: None,
        }
    }

    #[test]
    fn add_clamps_to_max() {
        let mut g = gauge();
        let crossed = g.add(fx(1000.0));
        assert_eq!(g.value, fx(100.0));
        assert_eq!(crossed.len(), 1);
        assert_eq!(crossed[0].direction, CrossingDirection::Rising);
        assert_eq!(crossed[0].event, GaugeEvent("trois_quarts".into()));
    }

    #[test]
    fn add_clamps_to_min() {
        let mut g = gauge();
        let crossed = g.add(fx(-1000.0));
        assert_eq!(g.value, fx(0.0));
        assert_eq!(crossed.len(), 1);
        assert_eq!(crossed[0].direction, CrossingDirection::Falling);
        assert_eq!(crossed[0].event, GaugeEvent("quart".into()));
    }

    #[test]
    fn floor_overrides_min_when_higher() {
        let mut g = gauge();
        g.floor = fx(40.0);
        let crossed = g.add(fx(-1000.0));
        assert_eq!(
            g.value,
            fx(40.0),
            "le plancher dynamique l'emporte sur `min`"
        );
        // 50 -> 40 : aucun seuil (25, 75) franchi
        assert!(crossed.is_empty());
    }

    #[test]
    fn floor_below_threshold_reports_the_crossing() {
        let mut g = gauge();
        g.floor = fx(20.0);
        let crossed = g.add(fx(-1000.0));
        assert_eq!(g.value, fx(20.0));
        assert_eq!(crossed.len(), 1);
        assert_eq!(crossed[0].direction, CrossingDirection::Falling);
        assert_eq!(crossed[0].event, GaugeEvent("quart".into()));
    }

    #[test]
    fn no_crossing_within_bounds() {
        let mut g = gauge();
        let crossed = g.add(fx(1.0)); // 50 -> 51, aucun seuil (25, 75) franchi
        assert_eq!(g.value, fx(51.0));
        assert!(crossed.is_empty());
    }

    #[test]
    fn stationary_add_emits_nothing() {
        let mut g = gauge();
        g.value = g.max;
        let crossed = g.add(fx(50.0)); // déjà au max, clampe sans bouger
        assert!(crossed.is_empty());
    }

    #[test]
    fn rising_then_falling_through_same_threshold() {
        let mut g = gauge();
        g.value = fx(20.0);

        let up = g.add(fx(10.0)); // 20 -> 30 : franchit 25 en montant
        assert_eq!(up.len(), 1);
        assert_eq!(up[0].direction, CrossingDirection::Rising);

        let down = g.add(fx(-10.0)); // 30 -> 20 : franchit 25 en descendant
        assert_eq!(down.len(), 1);
        assert_eq!(down[0].direction, CrossingDirection::Falling);
    }

    #[test]
    fn crossing_multiple_thresholds_at_once_preserves_order() {
        let mut g = gauge();
        g.value = fx(0.0);
        let crossed = g.add(fx(100.0)); // 0 -> 100 : franchit 25 puis 75, dans cet ordre
        assert_eq!(crossed.len(), 2);
        assert_eq!(crossed[0].event, GaugeEvent("quart".into()));
        assert_eq!(crossed[1].event, GaugeEvent("trois_quarts".into()));
    }
}
