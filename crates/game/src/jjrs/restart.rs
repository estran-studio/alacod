//! Restart en ligne (D14, `docs/conventions.md` §33) : « Rejouer » à la fin d'une partie p2p
//! relance une partie avec les mêmes pairs, sans saisie de lobby et **sans message réseau**.
//!
//! Les deux clients ont joué les mêmes parties en ligne : leur compteur [`OnlineGames`]
//! (parties en ligne depuis la dernière entrée « normale » dans `LobbyOnline`, hors rollback)
//! est identique. « Rejouer » pose [`OnlineRestart`] puis passe en `LobbyOnline` :
//! - le socket matchbox de la partie est fermé (`OnExit(InGame)`) ; le nouveau s'ouvre sur la
//!   salle [`restart_room`] `{lobby}-r{n}` (jamais la salle d'origine : pas de pair fantôme) ;
//! - `wait_for_players` attend les pairs comme au premier lancement ; au bout de
//!   [`RESTART_TIMEOUT_SECS`] sans eux (l'autre a choisi « Lobby » ou est parti), retour à la
//!   salle `{lobby}` d'origine, comme un « Lobby » ;
//! - la session GGRS est recréée ; graine [`restart_seed`] (`seed ^ fnv1a("restart") ^ n`).
//!
//! Un client qui a joué en local avant de se connecter n'a pas compté ces parties : seules
//! les parties en ligne comptent, et le compteur repart de 0 à chaque entrée dans
//! `LobbyOnline` sans [`OnlineRestart`] (saisie, « Lobby », délai écoulé).
//!
//! Allumette (`--allumette`) : non couvert en v1 (l'API joint un lobby par `game_id`, pas par
//! nom) ; « Rejouer » y reste redirigé vers le lobby.

use bevy::prelude::*;
use bevy_fixed::rng::fnv1a;
use bevy_matchbox::MatchboxSocket;

use crate::core::OnlineState;

/// Délai d'attente des pairs dans la salle de restart avant le retour au lobby.
pub const RESTART_TIMEOUT_SECS: f32 = 30.0;

/// Parties en ligne jouées depuis la dernière entrée normale dans `LobbyOnline`.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct OnlineGames(pub u32);

/// Restart en ligne en cours : partie `game` (= `OnlineGames` à la fin de la précédente).
/// Retiré quand la session est recréée ou au délai écoulé.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct OnlineRestart {
    pub game: u32,
    /// Temps réel (`Time<Real>::elapsed_secs`) de l'ouverture de la salle de restart.
    pub since: Option<f32>,
}

/// Salle matchbox du restart `game` (≥ 1).
pub fn restart_room(lobby: &str, game: u32) -> String {
    format!("{lobby}-r{game}")
}

/// Graine de run de la partie relancée `game` (≥ 1) depuis la graine de carte.
pub fn restart_seed(seed: u32, game: u32) -> u32 {
    let mixed = u64::from(seed) ^ fnv1a(b"restart") ^ u64::from(game);
    (mixed ^ (mixed >> 32)) as u32
}

/// Le délai d'attente des pairs est écoulé.
pub fn restart_timed_out(since: Option<f32>, now: f32) -> bool {
    since.is_some_and(|since| now - since >= RESTART_TIMEOUT_SECS)
}

/// `OnEnter(LobbyOnline)` : sans restart en cours, le compteur repart de 0.
pub fn reset_online_games(restart: Option<Res<OnlineRestart>>, mut games: ResMut<OnlineGames>) {
    if restart.is_none() {
        games.0 = 0;
    }
}

/// `OnEnter(InGame)` : une partie en ligne de plus.
pub fn count_online_game(online: Res<OnlineState>, mut games: ResMut<OnlineGames>) {
    if matches!(*online, OnlineState::Online) {
        games.0 += 1;
    }
}

/// `OnExit(InGame)` : ferme le socket matchbox de la partie (le pair resté n'y voit plus
/// l'autre ; le prochain `LobbyOnline` en rouvre un).
pub fn drop_matchbox_socket(mut commands: Commands, socket: Option<Res<MatchboxSocket>>) {
    if socket.is_some() {
        commands.remove_resource::<MatchboxSocket>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn salle_et_graine() {
        assert_eq!(restart_room("test-42", 1), "test-42-r1");
        assert_eq!(restart_room("test-42", 3), "test-42-r3");
        assert_eq!(restart_seed(1234, 1), restart_seed(1234, 1), "pure");
        assert_ne!(restart_seed(1234, 1), restart_seed(1234, 2));
        assert_ne!(restart_seed(1234, 1), 1234);
        assert_ne!(restart_seed(1234, 1), restart_seed(1235, 1));
    }

    #[test]
    fn delai() {
        assert!(!restart_timed_out(None, 100.0));
        assert!(!restart_timed_out(Some(10.0), 39.9));
        assert!(restart_timed_out(Some(10.0), 40.0));
    }
}
