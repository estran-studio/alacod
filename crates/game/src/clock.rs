//! Horloges et difficulté en simulation (T1.9, chantier F2, `docs/conventions.md` §23).
//!
//! L'état rollback est `run::Clock` (checksum neutre) ; ce module construit, au lancement de
//! la partie et **seulement si elles sont activées**, les horloges ([`ClockSchedule`]) et la
//! difficulté ([`DifficultyConfig`]) depuis le registre de contenu (ressources hors rollback,
//! identiques sur tous les clients), et fait avancer l'horloge ([`clock_system`]).
//!
//! Activation : champ `clocks`/`difficulty` d'un scénario ([`ClocksOverride`],
//! [`DifficultyOverride`], posés par le runner) ou `entry.clocks`/`entry.difficulty` du
//! manifeste pour une partie jouée. Sans activation : aucune des deux ressources, `Clock`
//! reste par défaut, aucun événement — les traces des parties existantes ne bougent pas.

use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_fixed::fixed_math::{self, Fixed};
use content::{
    expr::{difficulty_context, Expr},
    manifest::GameManifest,
    registry::{ClockScopeEntry, ClockTimeEntry, Registry},
};
use run::{
    clock::{belongs_to, due_events, ClockDef, ClockEventDef, ClockScope, ClockTime},
    Clock, ClockFired, FloorEntered, FloorState,
};
use sim_core::{frame_events::FrameEvents, players::PlayersCount};
use utils::frame::FrameCount;

use crate::{core::OnlineState, jjrs::GggrsSessionConfiguration};

/// Horloges demandées par un scénario (`Scenario::clocks`), posé par le runner.
#[derive(Resource, Clone, Debug)]
pub struct ClocksOverride(pub Vec<String>);

/// Difficulté demandée par un scénario (`Scenario::difficulty`), posé par le runner.
#[derive(Resource, Clone, Copy, Debug)]
pub struct DifficultyOverride(pub bool);

/// Horloges actives de la partie, par id (ordre des ids). Absente sans activation.
#[derive(Resource, Clone, Debug, Default)]
pub struct ClockSchedule {
    pub clocks: Vec<(String, ClockDef)>,
}

impl ClockSchedule {
    /// L'id d'occurrence appartient-il à un événement d'une horloge de portée `Floor` ?
    pub fn is_floor_event(&self, occurrence: &str) -> bool {
        self.clocks.iter().any(|(_, def)| {
            def.scope == ClockScope::Floor
                && def
                    .events
                    .iter()
                    .any(|event| belongs_to(occurrence, &event.id))
        })
    }
}

/// Difficulté de la partie : l'expression (asset immuable, jamais dans l'état rollback) et le
/// nombre de joueurs. Absente sans activation.
#[derive(Resource, Clone, Debug)]
pub struct DifficultyConfig {
    pub expr: Expr,
    pub players: u32,
}

impl DifficultyConfig {
    /// Valeur pour un étage et des temps donnés (frames). Une erreur d'évaluation (refusée par
    /// le lint) donne 1, jamais une panique.
    pub fn evaluate(&self, floor: u32, run_frames: u32, floor_frames: u32) -> Fixed {
        let ctx = difficulty_context(self.players, floor, run_frames, floor_frames);
        match self.expr.try_eval_num(&ctx) {
            Ok(value) => value,
            Err(error) => {
                warn!("difficulté : évaluation impossible ({error:?}), 1 par défaut");
                fixed_math::FIXED_ONE
            }
        }
    }
}

fn clock_time(time: ClockTimeEntry) -> ClockTime {
    match time {
        ClockTimeEntry::Frames(frames) => ClockTime::Frames(frames),
        ClockTimeEntry::Seconds(seconds) => ClockTime::Seconds(seconds.get()),
    }
}

/// `OnEnter(AppState::GameLoading)` : construit les horloges et la difficulté actives (voir
/// la doc du module), ou retire les ressources d'une partie précédente.
#[allow(clippy::too_many_arguments)]
pub fn resolve_clocks_system(
    mut commands: Commands,
    registry: Option<Res<Registry>>,
    manifest: Option<Res<GameManifest>>,
    clocks_override: Option<Res<ClocksOverride>>,
    difficulty_override: Option<Res<DifficultyOverride>>,
    online: Res<OnlineState>,
    ggrs_config: Res<GggrsSessionConfiguration>,
    players_count: Res<PlayersCount>,
) {
    commands.remove_resource::<ClockSchedule>();
    commands.remove_resource::<DifficultyConfig>();
    let Some(registry) = registry else {
        return;
    };
    let wanted_clocks = clocks_override
        .map(|o| o.0.clone())
        .or_else(|| manifest.as_ref().and_then(|m| m.entry.clocks.clone()))
        .unwrap_or_default();
    let mut clocks: Vec<(String, ClockDef)> = wanted_clocks
        .iter()
        .filter_map(|id| {
            let entry = registry
                .clocks
                .get(&content::registry::ClockId::from(id.clone()));
            if entry.is_none() {
                warn!("horloge « {id} » inconnue (voir `alacod lint`), ignorée");
            }
            entry.map(|entry| {
                let def = ClockDef {
                    scope: match entry.scope {
                        ClockScopeEntry::Run => ClockScope::Run,
                        ClockScopeEntry::Floor => ClockScope::Floor,
                    },
                    events: entry
                        .events
                        .iter()
                        .map(|event| ClockEventDef {
                            id: event.id.clone(),
                            at: clock_time(event.at),
                            repeat: event.repeat.map(clock_time),
                        })
                        .collect(),
                };
                (id.clone(), def)
            })
        })
        .collect();
    clocks.sort_by(|a, b| a.0.cmp(&b.0));
    if !clocks.is_empty() {
        commands.insert_resource(ClockSchedule { clocks });
    }

    let wants_difficulty = difficulty_override
        .map(|o| o.0)
        .or_else(|| manifest.as_ref().and_then(|m| m.entry.difficulty))
        .unwrap_or(false);
    if wants_difficulty {
        match registry
            .difficulty
            .as_ref()
            .map(|entry| Expr::parse(&entry.value))
        {
            Some(Ok(expr)) => {
                let players = match *online {
                    OnlineState::Online => ggrs_config.connection.max_player as u32,
                    OnlineState::Offline | OnlineState::Unset => players_count.0 as u32,
                };
                commands.insert_resource(DifficultyConfig { expr, players });
            }
            Some(Err(error)) => warn!("difficulté invalide ({error:?}, voir `alacod lint`)"),
            None => warn!("difficulté demandée sans fichier de kind Difficulty"),
        }
    }
}

/// Lecture de la difficulté par les apparitions d'ennemis (vagues, spawners, cartes) :
/// multiplicateur 1 (exact) sans difficulté activée.
#[derive(SystemParam)]
pub struct DifficultyReader<'w> {
    pub clock: Res<'w, Clock>,
    pub config: Option<Res<'w, DifficultyConfig>>,
    pub frame: Res<'w, FrameCount>,
}

impl DifficultyReader<'_> {
    /// Difficulté pour l'étage `floor` **maintenant**, évaluée à neuf (les personnages d'un
    /// nouvel étage apparaissent dans la frame du passage, avant la réévaluation de
    /// [`clock_system`]) : temps d'étage 0 pour un étage pas encore vu par l'horloge.
    pub fn at_floor(&self, floor: u32) -> Fixed {
        let Some(config) = &self.config else {
            return fixed_math::FIXED_ONE;
        };
        let now = self.frame.frame;
        let floor_frames = if floor == self.clock.floor_index {
            self.clock.floor_frames(now)
        } else {
            0
        };
        config.evaluate(floor, self.clock.run_frames(now), floor_frames)
    }

    /// Difficulté pour l'étage courant de l'horloge.
    pub fn current(&self) -> Fixed {
        self.at_floor(self.clock.floor_index)
    }
}

/// Applique un multiplicateur de difficulté à une quantité (santé, dégâts) : **aucun calcul**
/// quand il vaut exactement 1 (sans difficulté activée), la valeur reste identique au bit près.
pub fn scale(value: Fixed, difficulty: Fixed) -> Fixed {
    if difficulty == fixed_math::FIXED_ONE {
        value
    } else {
        value.saturating_mul(difficulty)
    }
}

/// Fait avancer l'horloge d'une frame (`RollbackSystemSet::FrameCounter`, avant
/// l'incrément de frame, donc après le passage d'étage du set `Run`). Sans activation : rien.
/// Ordre : entrée d'étage (temps d'étage remis à zéro, événements de portée `Floor` oubliés,
/// `FloorEntered`), événements échus par horloge (ordre des ids) puis par événement (ordre du
/// fichier), difficulté réévaluée chaque seconde et à chaque entrée d'étage.
#[allow(clippy::too_many_arguments)]
pub fn clock_system(
    frame: Res<FrameCount>,
    mut clock: ResMut<Clock>,
    floor_state: Res<FloorState>,
    schedule: Option<Res<ClockSchedule>>,
    difficulty: Option<Res<DifficultyConfig>>,
    mut fired_events: ResMut<FrameEvents<ClockFired>>,
    mut floor_events: ResMut<FrameEvents<FloorEntered>>,
) {
    if schedule.is_none() && difficulty.is_none() {
        return;
    }
    let now = frame.frame;
    let entered = floor_state.index != clock.floor_index;
    if entered {
        let schedule = schedule.as_deref();
        clock.enter_floor(floor_state.index, now, |id| {
            schedule.is_some_and(|schedule| schedule.is_floor_event(id))
        });
        floor_events.send(FloorEntered {
            index: floor_state.index,
        });
        info!(
            "ggrs{{f={} clock floor_entered index={}}}",
            now, floor_state.index
        );
    }
    if let Some(schedule) = &schedule {
        for (_name, def) in &schedule.clocks {
            let elapsed = clock.scope_frames(def.scope, now);
            for id in due_events(def, elapsed, &clock.fired) {
                info!("ggrs{{f={} clock id={}}}", now, id);
                clock.fired.insert(id.clone());
                fired_events.send(ClockFired { id });
            }
        }
    }
    if let Some(config) = &difficulty {
        if entered || now % run::clock::FRAMES_PER_SECOND == 0 {
            let value = config.evaluate(
                clock.floor_index,
                clock.run_frames(now),
                clock.floor_frames(now),
            );
            if value != clock.difficulty {
                info!("ggrs{{f={} clock difficulty={}}}", now, value);
            }
            clock.difficulty = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Critère de la fiche : dégâts (ou santé) × difficulté 1,0 identiques **au bit près** à
    /// l'absence de multiplication — prouvé sur des valeurs de tout l'intervalle, y compris
    /// celles où `saturating_mul` aurait pu arrondir.
    #[test]
    fn difficulte_un_identique_au_bit_pres() {
        let samples = [
            Fixed::ZERO,
            Fixed::from_bits(1),
            Fixed::from_bits(-1),
            fixed_math::new(2.5),
            fixed_math::new(10.0),
            fixed_math::new(0.333),
            fixed_math::new(1234.5678),
            Fixed::MAX,
            Fixed::MIN,
        ];
        for value in samples {
            assert_eq!(
                scale(value, fixed_math::FIXED_ONE).to_bits(),
                value.to_bits(),
                "{value}"
            );
            // Et même le produit brut par 1 est exact (la garde n'est qu'une ceinture).
            assert_eq!(
                value.saturating_mul(fixed_math::FIXED_ONE).to_bits(),
                value.to_bits()
            );
        }
        assert_eq!(
            scale(fixed_math::new(10.0), fixed_math::new(1.5)),
            fixed_math::new(15.0)
        );
    }

    #[test]
    fn evaluation_de_la_difficulte() {
        let config = DifficultyConfig {
            expr: Expr::parse("1 + floor * 0.5 + floor_minutes * 0.5").unwrap(),
            players: 1,
        };
        assert_eq!(config.evaluate(0, 0, 0), fixed_math::FIXED_ONE);
        assert_eq!(config.evaluate(1, 600, 0), fixed_math::new(1.5));
        assert_eq!(config.evaluate(0, 7200, 7200), fixed_math::new(2.0));
        let broken = DifficultyConfig {
            expr: Expr::parse("1 + inconnu").unwrap(),
            players: 1,
        };
        assert_eq!(broken.evaluate(0, 0, 0), fixed_math::FIXED_ONE);
    }

    #[test]
    fn evenements_de_portee_etage() {
        let schedule = ClockSchedule {
            clocks: vec![
                (
                    "arene".into(),
                    ClockDef {
                        scope: ClockScope::Floor,
                        events: vec![ClockEventDef {
                            id: "renfort".into(),
                            at: ClockTime::Frames(60),
                            repeat: Some(ClockTime::Frames(60)),
                        }],
                    },
                ),
                (
                    "run".into(),
                    ClockDef {
                        scope: ClockScope::Run,
                        events: vec![ClockEventDef {
                            id: "minuit".into(),
                            at: ClockTime::Frames(600),
                            repeat: None,
                        }],
                    },
                ),
            ],
        };
        assert!(schedule.is_floor_event("renfort#2"));
        assert!(!schedule.is_floor_event("minuit"));
    }
}
