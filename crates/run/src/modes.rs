//! Règles propres à chaque [`RunMode`] (T2.4, chantier F1) : condition de victoire et
//! résumé de fin de partie. La défaite (tous les joueurs à terre ou morts, T1.3, chantier
//! B6) est universelle, quel que soit le mode — elle reste hors de ce trait, posée
//! directement dans `Run.step` par `game::character::health::rollback_check_defeat`.
//!
//! Ce crate reste minimal (voir `lib.rs`) : il ne dépend pas de `game`, donc pas de
//! `WaveState`/`WaveConfig` directement. [`RunContext`] porte les seules valeurs dont une
//! règle de mode a besoin, construites par l'appelant (`game::run`) à partir de l'état de
//! simulation réel.

use crate::run::{RunEnd, RunMode, RunSummary};

/// Contexte générique transmis à [`RunModeRules`], construit par `game::run` à chaque
/// frame à partir de l'état de simulation (`WaveState`, soldes des joueurs...).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RunContext {
    /// Frame courante de la simulation.
    pub frame: u32,
    /// Vague courante (mode `Waves` ; `0` si sans objet pour le mode actif).
    pub current_wave: u32,
    /// Vague de victoire déclarée par le RON (`waves::config::WaveConfig::max_wave`), si
    /// le jeu en déclare une. `None` : pas de condition de victoire par les vagues (partie
    /// infinie jusqu'à la défaite, comportement d'avant ce chantier).
    pub max_wave: Option<u32>,
    /// Ennemis tués au total (mode `Waves` ; `WaveState::total_enemies_killed`).
    pub kills: u32,
    /// Somme des soldes (`run::currency::Currency`) de tous les joueurs à cet instant.
    pub points_total: u32,
}

/// Règles de fin de partie propres à un mode. Implémenté pour [`RunMode`] directement (un
/// seul type porte déjà la distinction `Waves`/`Sandbox`, voir sa doc) plutôt que des types
/// marqueurs séparés par mode : pas de donnée ni de comportement qui justifierait un type à
/// part pour l'instant.
pub trait RunModeRules {
    /// `Some(RunEnd::Victory)` si la condition de victoire de ce mode est atteinte ce
    /// frame ; `None` tant que la partie continue (toujours `None` pour un mode sans
    /// condition de victoire, ex. `Sandbox`). N'est appelé que si `Run.step` est
    /// `Playing` (voir `game::run::check_run_victory_system`) : ne décide jamais de
    /// défaite, seulement de victoire.
    fn check_victory(&self, ctx: &RunContext) -> Option<RunEnd>;

    /// Résumé à poser dans `Run.summary` une fois `outcome` connu (défaite T1.3, victoire
    /// de `check_victory`, ou abandon `RunRequest::ToLobby`).
    fn summarize(&self, ctx: &RunContext, outcome: RunEnd) -> RunSummary;
}

impl RunModeRules for RunMode {
    fn check_victory(&self, ctx: &RunContext) -> Option<RunEnd> {
        match self {
            // Victoire si le RON déclare une vague de fin (`max_wave`) et qu'elle est
            // atteinte ; `max_wave: None` (défaut) : jamais de victoire, comme avant ce
            // chantier (voir la doc de `WaveConfig::max_wave`).
            RunMode::Waves { .. } => {
                let max_wave = ctx.max_wave?;
                (max_wave > 0 && ctx.current_wave >= max_wave).then_some(RunEnd::Victory)
            }
            RunMode::Sandbox => None,
        }
    }

    fn summarize(&self, ctx: &RunContext, outcome: RunEnd) -> RunSummary {
        RunSummary {
            wave_reached: ctx.current_wave,
            kills: ctx.kills,
            points_total: ctx.points_total,
            frames: ctx.frame,
            outcome,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn waves() -> RunMode {
        RunMode::Waves {
            config: "wave_config".to_string(),
        }
    }

    #[test]
    fn waves_never_wins_without_max_wave() {
        let ctx = RunContext {
            current_wave: 999,
            max_wave: None,
            ..Default::default()
        };
        assert_eq!(waves().check_victory(&ctx), None);
    }

    #[test]
    fn waves_wins_once_max_wave_reached() {
        let ctx = RunContext {
            current_wave: 9,
            max_wave: Some(10),
            ..Default::default()
        };
        assert_eq!(waves().check_victory(&ctx), None);

        let ctx = RunContext {
            current_wave: 10,
            max_wave: Some(10),
            ..Default::default()
        };
        assert_eq!(waves().check_victory(&ctx), Some(RunEnd::Victory));
    }

    #[test]
    fn sandbox_never_wins() {
        let ctx = RunContext {
            current_wave: 1_000_000,
            max_wave: Some(1),
            ..Default::default()
        };
        assert_eq!(RunMode::Sandbox.check_victory(&ctx), None);
    }

    #[test]
    fn summarize_carries_context_and_outcome() {
        let ctx = RunContext {
            frame: 1500,
            current_wave: 4,
            max_wave: None,
            kills: 37,
            points_total: 900,
        };
        let summary = waves().summarize(&ctx, RunEnd::Defeat);
        assert_eq!(summary.wave_reached, 4);
        assert_eq!(summary.kills, 37);
        assert_eq!(summary.points_total, 900);
        assert_eq!(summary.frames, 1500);
        assert_eq!(summary.outcome, RunEnd::Defeat);
    }
}
