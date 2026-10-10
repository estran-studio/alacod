//! Boss à phases (M2-T0c, chantier D4, `docs/conventions.md` §37) : contrats purs, sans ECS
//! d'exécution (le système vit dans `game::boss`).
//!
//! Un boss est un ennemi dont le personnage RON porte un champ `boss: Some((phases: [...]))`.
//! Chaque [`Phase`] remplace les règles de comportement de l'ennemi (vocabulaire T1.4), exécute
//! des actions à l'entrée (`on_enter`, [`effects::Action`]) et peut rejouer une [`Timeline`]
//! d'actions datées. Une phase se termine par son [`PhaseEnd`] ; la dernière n'en a pas.
//! L'état est [`BossState`] (rollback, neutre).

use bevy::prelude::Component;
use bevy_fixed::fixed_math::Fixed;
use effects::Action;
use serde::{Deserialize, Serialize};

/// Condition de fin d'une phase (passage à la suivante).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum PhaseEnd {
    /// La santé tombe **strictement** sous ce ratio de la santé max (`]0, 1]`).
    HealthBelow(Fixed),
    /// `n` frames se sont écoulées depuis l'entrée dans la phase (`n > 0`).
    AfterFrames(u32),
}

/// Action datée : se déclenche `at` frames après l'entrée dans la phase (ou, avec `repeat`, à
/// chaque période).
#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub at: u32,
    #[serde(rename = "do")]
    pub r#do: Vec<Action>,
}

/// Actions datées d'une phase. `repeat: Some(p)` : la frise recommence toutes les `p` frames
/// (chaque `at` doit être `< p`) ; `None` : une seule fois.
#[derive(Clone, PartialEq, Eq, Debug, Default, Hash, Serialize, Deserialize)]
pub struct Timeline {
    pub events: Vec<TimelineEvent>,
    #[serde(default)]
    pub repeat: Option<u32>,
}

impl Timeline {
    /// Événements dus à `since` frames de l'entrée dans la phase, dans l'ordre déclaré. Pur et
    /// sans curseur : deux simulations de la même frame donnent la même réponse (rollback).
    pub fn due(&self, since: u32) -> impl Iterator<Item = &TimelineEvent> {
        let local = match self.repeat {
            Some(period) if period > 0 => since % period,
            _ => since,
        };
        self.events.iter().filter(move |event| event.at == local)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default, Hash, Serialize, Deserialize)]
pub struct Phase {
    /// Fin de la phase ; `None` : dernière phase (seule autorisée à n'en pas avoir).
    #[serde(default)]
    pub until: Option<PhaseEnd>,
    /// Règles de comportement de la phase (vocabulaire T1.4) ; vide : celles de l'ennemi.
    #[serde(default)]
    pub behaviors: Vec<crate::Behavior>,
    /// Actions exécutées à l'entrée dans la phase (y compris la première, au premier tick).
    #[serde(default)]
    pub on_enter: Vec<Action>,
    #[serde(default)]
    pub timeline: Option<Timeline>,
}

/// Définition d'un boss : phases dans l'ordre (la première est active à l'apparition).
#[derive(Clone, PartialEq, Eq, Debug, Default, Hash, Serialize, Deserialize)]
pub struct BossDef {
    pub phases: Vec<Phase>,
}

/// État d'un boss (rollback + checksum + trace, variante **neutre** : seuls les boss en
/// portent). La frise n'a pas de curseur : elle se déduit de `entered`.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct BossState {
    /// Indice de la phase courante dans [`BossDef::phases`].
    pub phase: u32,
    /// Frame d'entrée dans la phase courante (valide si `started`).
    pub entered: u32,
    /// Faux jusqu'au premier tick du système, qui date l'entrée de la phase 0.
    pub started: bool,
}

/// Une phase vient de changer (`FrameEvents`, neutre) : moment clé `boss_phase`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BossPhaseChanged {
    pub frame: u32,
    pub net_id: usize,
    pub from: u32,
    pub to: u32,
}

impl BossDef {
    /// Phase vers laquelle passer cette frame, au plus une (ordre du RON). `health_ratio` :
    /// santé courante / santé max.
    pub fn next_phase(&self, state: &BossState, health_ratio: Fixed, frame: u32) -> Option<u32> {
        let current = self.phases.get(state.phase as usize)?;
        let next = state.phase + 1;
        if next as usize >= self.phases.len() {
            return None;
        }
        let done = match current.until? {
            PhaseEnd::HealthBelow(ratio) => health_ratio < ratio,
            PhaseEnd::AfterFrames(n) => frame.saturating_sub(state.entered) >= n,
        };
        done.then_some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fx(v: f32) -> Fixed {
        Fixed::from_num(v)
    }

    fn def() -> BossDef {
        ron::from_str(
            r#"(phases: [
                (until: Some(HealthBelow("0.5")),
                 timeline: Some((events: [(at: 0, do: [Heal("1.0")]), (at: 30, do: [Heal("2.0")])],
                                 repeat: Some(60)))),
                (until: Some(AfterFrames(90)), on_enter: [Heal("5.0")]),
                (),
            ])"#,
        )
        .unwrap()
    }

    #[test]
    fn ron_et_transitions() {
        let def = def();
        let mut state = BossState {
            phase: 0,
            entered: 10,
            started: true,
        };
        assert_eq!(def.next_phase(&state, fx(0.51), 100), None);
        // strictement sous le seuil
        assert_eq!(def.next_phase(&state, fx(0.5), 100), None);
        assert_eq!(def.next_phase(&state, fx(0.49), 100), Some(1));
        state.phase = 1;
        state.entered = 100;
        assert_eq!(def.next_phase(&state, fx(1.0), 189), None);
        assert_eq!(def.next_phase(&state, fx(1.0), 190), Some(2));
        // dernière phase : jamais de suite
        state.phase = 2;
        assert_eq!(def.next_phase(&state, fx(0.0), 10_000), None);
    }

    #[test]
    fn frise_en_boucle() {
        let def = def();
        let timeline = def.phases[0].timeline.as_ref().unwrap();
        let at = |since: u32| timeline.due(since).map(|e| e.at).collect::<Vec<_>>();
        assert_eq!(at(0), vec![0]);
        assert_eq!(at(1), Vec::<u32>::new());
        assert_eq!(at(30), vec![30]);
        assert_eq!(at(60), vec![0]);
        assert_eq!(at(90), vec![30]);
    }

    #[test]
    fn frise_unique() {
        let timeline = Timeline {
            events: vec![TimelineEvent {
                at: 5,
                r#do: vec![],
            }],
            repeat: None,
        };
        assert_eq!(timeline.due(5).count(), 1);
        assert_eq!(timeline.due(6).count(), 0);
        assert_eq!(timeline.due(65).count(), 0);
    }
}
