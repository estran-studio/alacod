//! Monnaie de run (T2.3, chantier C5 v1) : points gagnés et dépensés pendant une partie
//! (kills, coups au but, réparations de fenêtre, portes payantes, armes murales, perks).
//!
//! [`Currency`] est un composant rollback posé sur chaque joueur à sa création
//! (`game::character::player::create::create_player`), initialisé à
//! `CharacterConfig::starting_currency` (défaut 500). L'API est saturante : [`Currency::earn`]
//! ne déborde jamais `u32::MAX`, [`Currency::spend`] refuse (renvoie `false`, ne change rien)
//! plutôt que de passer sous zéro — jamais de panique sur une soustraction.
//!
//! [`CurrencyEvent`] est l'unique canal `FrameEvents` de ce crate : émis par `game` (systèmes
//! de points en `RollbackSystemSet::Run`, interactions d'achat en `RollbackSystemSet::Interaction`)
//! à chaque variation de solde, positive (`delta > 0`, gain) ou négative (`delta < 0`, achat).
//! Un événement `delta: 0` signale un achat **refusé** (solde insuffisant) : aucune monnaie ne
//! bouge, mais le HUD/les scénarios (`scenario::events`) ont besoin de savoir qu'une tentative a
//! eu lieu (`GameEvents` : `purchase_refused`). Lu par le HUD (`game::ui::hud`, source
//! `currency`) et par `crates/scenario/src/events.rs` (moments clés `purchase`/`points`).

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use sim_core::frame_events::FrameEventsAppExt;
use utils::rollback::RollbackTraceApp;

use crate::perks::Perks;
use crate::run::Run;

/// Solde de monnaie d'un joueur. Rollback (checksum GGRS, voir [`RunPlugin`]) : toute
/// divergence de solde entre clients est un desync, comme n'importe quel autre état de jeu.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Currency(pub u32);

impl Currency {
    pub fn new(amount: u32) -> Self {
        Self(amount)
    }

    /// Vrai si le solde couvre `amount` (`>=`).
    pub fn can_afford(&self, amount: u32) -> bool {
        self.0 >= amount
    }

    /// Débite `amount` si [`Self::can_afford`] ; sinon ne change rien. Renvoie si le débit a
    /// eu lieu (le seul point de décision « achat accepté/refusé » de ce type : les appelants
    /// n'ont pas à réimplémenter la comparaison).
    pub fn spend(&mut self, amount: u32) -> bool {
        if !self.can_afford(amount) {
            return false;
        }
        self.0 = self.0.saturating_sub(amount);
        true
    }

    /// Crédite `amount`, saturant à `u32::MAX` (jamais de panique/débordement).
    pub fn earn(&mut self, amount: u32) {
        self.0 = self.0.saturating_add(amount);
    }
}

/// Variation de solde d'un joueur pendant une frame de simulation : gain (`delta > 0`,
/// kill/coup/réparation), dépense (`delta < 0`, porte/arme murale/perk) ou refus (`delta ==
/// 0`, solde insuffisant — voir la doc du module). `handle` est le handle GGRS du joueur
/// (`game::character::player::Player::handle`), pas une `Entity` (CLAUDE.md, règle 3).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CurrencyEvent {
    pub handle: usize,
    pub delta: i64,
    pub reason: String,
}

/// Enregistre [`Currency`]/[`Perks`] (composants) et [`Run`] (ressource, T2.4) en rollback
/// (checksum + trace) et `FrameEvents<CurrencyEvent>` (rollback, vidé à chaque frame — voir
/// `sim_core::frame_events`). Ajouté par `game::core::CoreSetupPlugin` : la pose initiale de
/// `Currency`/`Perks` sur un joueur (`character::player::create::create_player`), la
/// création de `Run` (`game::jjrs::{local, p2p}`, au démarrage de session) et leur
/// lecture/écriture (points, achats, fin de partie) vivent dans `game`, qui dépend de ce
/// crate. Contrairement à `Currency`/`Perks` (composants posés une fois par joueur, jamais
/// `init_resource`), `Run` n'a pas de valeur par défaut sensée (pas de graine/mode hors
/// contexte) : rien n'appelle `init_resource::<Run>()`, elle n'existe qu'une fois créée.
pub struct RunPlugin;

impl Plugin for RunPlugin {
    fn build(&self, app: &mut App) {
        app.rollback_and_trace::<Currency>()
            .rollback_and_trace::<Perks>();
        app.add_frame_events::<CurrencyEvent>();
        app.rollback_and_trace_resource::<Run>();
        // T1.8 : état du mode `Floors`, toujours présent (valeur par défaut hors `Floors`).
        // Checksum neutre : la valeur par défaut contribue `0` au checksum GGRS, les traces
        // des autres modes restent identiques (voir la doc de `FloorState`).
        app.init_resource::<crate::floors::FloorState>();
        app.rollback_and_trace_resource_neutral::<crate::floors::FloorState>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_afford_is_inclusive() {
        let wallet = Currency(100);
        assert!(wallet.can_afford(100));
        assert!(wallet.can_afford(50));
        assert!(!wallet.can_afford(101));
    }

    #[test]
    fn spend_debits_when_affordable() {
        let mut wallet = Currency(100);
        assert!(wallet.spend(60));
        assert_eq!(wallet.0, 40);
    }

    #[test]
    fn spend_refuses_and_leaves_balance_unchanged_when_short() {
        let mut wallet = Currency(50);
        assert!(!wallet.spend(51));
        assert_eq!(wallet.0, 50);
    }

    #[test]
    fn earn_saturates_at_max() {
        let mut wallet = Currency(u32::MAX - 1);
        wallet.earn(10);
        assert_eq!(wallet.0, u32::MAX);
    }

    #[test]
    fn spend_zero_always_succeeds() {
        let mut wallet = Currency(0);
        assert!(wallet.spend(0));
        assert_eq!(wallet.0, 0);
    }
}
