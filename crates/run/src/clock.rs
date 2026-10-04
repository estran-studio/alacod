//! Horloges d'étage et de run, événements planifiés, difficulté (T1.9, chantier F2,
//! `docs/conventions.md` §23).
//!
//! Ce module porte l'état rollback ([`Clock`]), le contenu des horloges ([`ClockDef`],
//! fichiers `clocks/<nom>.ron`, kind `Clock`) et les règles pures, testables sans Bevy
//! ([`due_events`]). Le système qui fait avancer l'horloge (`game::clock::clock_system`) vit
//! dans `game`, qui connaît le registre de contenu, l'étage courant et les expressions.
//!
//! **Activation** : les horloges et la difficulté ne tournent que si une partie les demande
//! (champs `clocks`/`difficulty` d'un scénario, `entry.clocks`/`entry.difficulty` du
//! manifeste). Sans activation, [`Clock`] reste à sa valeur par défaut (checksum neutre) et
//! aucun événement n'est émis : les traces des parties existantes ne bougent pas.

use std::collections::BTreeSet;

use bevy::prelude::*;
use bevy_fixed::fixed_math::{self, Fixed};
use serde::Deserialize;

/// Frames de simulation par seconde.
pub const FRAMES_PER_SECOND: u32 = 60;

/// Portée d'une horloge : depuis le début du run, ou depuis l'entrée dans l'étage courant
/// (remise à zéro à chaque passage d'étage du mode `Floors`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum ClockScope {
    Run,
    Floor,
}

/// Une durée : en frames, ou en secondes (`Fixed`, arrondie à la frame inférieure).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum ClockTime {
    Frames(u32),
    Seconds(Fixed),
}

impl ClockTime {
    pub fn frames(self) -> u32 {
        match self {
            ClockTime::Frames(frames) => frames,
            ClockTime::Seconds(seconds) => seconds
                .saturating_mul(Fixed::from_num(FRAMES_PER_SECOND))
                .max(fixed_math::FIXED_ZERO)
                .to_num::<u32>(),
        }
    }
}

/// Un événement planifié : `id` déclenché quand le temps de la portée atteint `at`, puis, si
/// `repeat`, toutes les `repeat` frames (chaque occurrence a l'id `"<id>#<n>"`, `n` à partir
/// de 1 ; sans `repeat`, l'id tel quel).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
pub struct ClockEventDef {
    pub id: String,
    pub at: ClockTime,
    #[serde(default)]
    pub repeat: Option<ClockTime>,
}

/// Contenu d'une horloge (`clocks/<nom>.ron`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
pub struct ClockDef {
    pub scope: ClockScope,
    pub events: Vec<ClockEventDef>,
}

/// Un événement d'horloge déclenché cette frame (`FrameEvents`, checksum neutre).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ClockFired {
    pub id: String,
}

/// Entrée dans un étage du mode `Floors` (`FrameEvents`, checksum neutre), émise par
/// `game::clock::clock_system` **seulement quand les horloges ou la difficulté sont
/// activées** ; T1.10 (`OnFloorEntered`) devra l'activer implicitement.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FloorEntered {
    pub index: u32,
}

/// État rollback des horloges (checksum **neutre** : la valeur par défaut — toute partie sans
/// horloge ni difficulté — contribue `0` au checksum GGRS).
#[derive(Resource, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Clock {
    /// Frame de début du run (0 : la simulation commence à la frame 0).
    pub run_started_frame: u32,
    /// Frame d'entrée dans l'étage courant.
    pub floor_started_frame: u32,
    /// Index de l'étage courant (`FloorState::index`), vu par l'horloge.
    pub floor_index: u32,
    /// Ids des événements déjà déclenchés dans leur portée courante (ceux de portée `Floor`
    /// sont oubliés à chaque passage d'étage).
    pub fired: BTreeSet<String>,
    /// Multiplicateur de difficulté (kind `Difficulty`), réévalué chaque seconde de
    /// simulation ; 1 sans difficulté activée.
    pub difficulty: Fixed,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            run_started_frame: 0,
            floor_started_frame: 0,
            floor_index: 0,
            fired: BTreeSet::new(),
            difficulty: fixed_math::FIXED_ONE,
        }
    }
}

impl Clock {
    pub fn run_frames(&self, now: u32) -> u32 {
        now.saturating_sub(self.run_started_frame)
    }

    pub fn floor_frames(&self, now: u32) -> u32 {
        now.saturating_sub(self.floor_started_frame)
    }

    pub fn run_seconds(&self, now: u32) -> Fixed {
        seconds(self.run_frames(now))
    }

    pub fn floor_seconds(&self, now: u32) -> Fixed {
        seconds(self.floor_frames(now))
    }

    /// Entrée dans l'étage `index` à la frame `now` : temps d'étage remis à zéro, ids des
    /// événements de portée `Floor` (`floor_ids`) oubliés.
    pub fn enter_floor<'a>(&mut self, index: u32, now: u32, floor_ids: impl Fn(&str) -> bool + 'a) {
        self.floor_index = index;
        self.floor_started_frame = now;
        self.fired.retain(|id| !floor_ids(id));
    }

    /// Temps écoulé dans une portée, en frames.
    pub fn scope_frames(&self, scope: ClockScope, now: u32) -> u32 {
        match scope {
            ClockScope::Run => self.run_frames(now),
            ClockScope::Floor => self.floor_frames(now),
        }
    }
}

/// Secondes (`Fixed`) d'un nombre de frames.
pub fn seconds(frames: u32) -> Fixed {
    // Quotient puis reste : `Fixed::from_num(frames)` déborderait au-delà de 32 767 frames.
    Fixed::saturating_from_num(frames / FRAMES_PER_SECOND).saturating_add(
        Fixed::from_num(frames % FRAMES_PER_SECOND) / Fixed::from_num(FRAMES_PER_SECOND),
    )
}

/// L'id d'occurrence appartient-il à l'événement `base` (`"renfort"` ou `"renfort#3"`) ?
pub fn belongs_to(occurrence: &str, base: &str) -> bool {
    occurrence == base
        || occurrence
            .strip_prefix(base)
            .is_some_and(|rest| rest.starts_with('#'))
}

/// Ids à déclencher maintenant pour une horloge, dans l'ordre des événements du fichier :
/// toute échéance atteinte (`elapsed >= at`, puis `at + k × repeat`) dont l'id n'est pas dans
/// `fired`. Rattrape plusieurs occurrences d'un coup si plusieurs sont échues.
pub fn due_events(def: &ClockDef, elapsed: u32, fired: &BTreeSet<String>) -> Vec<String> {
    let mut due = Vec::new();
    for event in &def.events {
        let at = event.at.frames();
        if elapsed < at {
            continue;
        }
        match event.repeat.map(ClockTime::frames) {
            None => {
                if !fired.contains(&event.id) {
                    due.push(event.id.clone());
                }
            }
            Some(0) => {
                // Refusé par le lint ; une seule occurrence par sécurité.
                let id = format!("{}#1", event.id);
                if !fired.contains(&id) {
                    due.push(id);
                }
            }
            Some(repeat) => {
                let occurrences = (elapsed - at) / repeat + 1;
                for n in 1..=occurrences {
                    let id = format!("{}#{n}", event.id);
                    if !fired.contains(&id) {
                        due.push(id);
                    }
                }
            }
        }
    }
    due
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arene() -> ClockDef {
        ClockDef {
            scope: ClockScope::Floor,
            events: vec![
                ClockEventDef {
                    id: "tic".into(),
                    at: ClockTime::Seconds(Fixed::from_num(1)),
                    repeat: None,
                },
                ClockEventDef {
                    id: "tac".into(),
                    at: ClockTime::Frames(120),
                    repeat: None,
                },
                ClockEventDef {
                    id: "renfort".into(),
                    at: ClockTime::Seconds(Fixed::from_num(2)),
                    repeat: Some(ClockTime::Seconds(Fixed::from_num(2))),
                },
            ],
        }
    }

    #[test]
    fn echeances_frames_et_secondes() {
        assert_eq!(ClockTime::Seconds(Fixed::from_num(1)).frames(), 60);
        assert_eq!(ClockTime::Seconds(fixed_math::new(1.5)).frames(), 90);
        assert_eq!(ClockTime::Frames(7).frames(), 7);
        let none = BTreeSet::new();
        assert!(due_events(&arene(), 59, &none).is_empty());
        assert_eq!(due_events(&arene(), 60, &none), vec!["tic".to_string()]);
        assert_eq!(
            due_events(&arene(), 120, &none),
            vec!["tic".to_string(), "tac".into(), "renfort#1".into()]
        );
    }

    #[test]
    fn deja_declenches_et_repetitions() {
        let mut fired: BTreeSet<String> = ["tic", "tac", "renfort#1"]
            .into_iter()
            .map(String::from)
            .collect();
        assert!(due_events(&arene(), 200, &fired).is_empty());
        assert_eq!(
            due_events(&arene(), 240, &fired),
            vec!["renfort#2".to_string()]
        );
        // Rattrapage : deux occurrences échues d'un coup.
        fired.remove("renfort#1");
        assert_eq!(
            due_events(&arene(), 240, &fired),
            vec!["renfort#1".to_string(), "renfort#2".into()]
        );
    }

    #[test]
    fn remise_a_zero_de_la_portee_etage() {
        let def = arene();
        let mut clock = Clock {
            fired: ["tic", "tac", "renfort#1", "nuit"]
                .into_iter()
                .map(String::from)
                .collect(),
            ..Default::default()
        };
        clock.enter_floor(1, 500, |id| {
            def.events.iter().any(|event| belongs_to(id, &event.id))
        });
        assert_eq!(clock.floor_index, 1);
        assert_eq!(clock.floor_frames(560), 60);
        assert_eq!(clock.run_frames(560), 560);
        // Seul l'événement d'une autre horloge (portée Run) reste.
        assert_eq!(clock.fired, ["nuit".to_string()].into_iter().collect());
        assert_eq!(
            due_events(&def, clock.floor_frames(560), &clock.fired),
            vec!["tic".to_string()]
        );
    }

    #[test]
    fn horloge_par_defaut_neutre() {
        let clock = Clock::default();
        assert_eq!(clock.difficulty, fixed_math::FIXED_ONE);
        assert!(clock.fired.is_empty());
        // La variante neutre du checksum compare à `Default` : un `Clock` par défaut
        // contribue 0 (voir `utils::rollback::hash_neutral_default`).
        assert_eq!(utils::rollback::hash_neutral_default(&clock), 0);
        assert_eq!(seconds(90), fixed_math::new(1.5));
        // Au-delà de 32 767 frames (≈ 9 min) : pas de débordement.
        assert_eq!(seconds(36000), Fixed::from_num(600));
    }

    #[test]
    fn appartenance_des_occurrences() {
        assert!(belongs_to("renfort", "renfort"));
        assert!(belongs_to("renfort#3", "renfort"));
        assert!(!belongs_to("renforts", "renfort"));
    }
}
