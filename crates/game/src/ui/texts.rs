//! Textes visibles du moteur (S1-4, `docs/taches/polissage-textes-fr.md`, `docs/conventions.md` § 100).
//!
//! Un seul endroit pour les mots affichés par les écrans communs à tous les jeux : lobby, états
//! réseau, perte de session (déconnexion, perte de la partie commune), fin de partie, pause,
//! inventaire, marchand. Français, pas de système d'i18n. Présentation seule : aucune de ces
//! chaînes n'entre dans la simulation, un hash ou une trace ; les journaux (`info!`, `error!`)
//! gardent leur anglais, les scripts p2p les lisent.
//!
//! Vocabulaire : « salon » (lobby), « pair » (autre joueur en ligne), « partie commune »
//! (simulation partagée), « tentative ».

// --- Lobby ----------------------------------------------------------------------------------

/// Titre de l'écran d'attente du salon en ligne.
pub const LOBBY_WAITING_FOR_PLAYERS: &str = "En attente des joueurs…";
/// Bouton de l'écran d'attente du salon local.
pub const NEW_GAME: &str = "Nouvelle partie";
/// Aide de la manette sur l'écran d'attente du salon local.
pub const NEW_GAME_PAD_HINT: &str = "A (manette) : nouvelle partie";
/// État d'un joueur dans le salon : prêt (joueur local connecté).
pub const PLAYER_READY: &str = "Prêt";
/// État d'un joueur dans le salon : connexion du joueur local en cours.
pub const PLAYER_CONNECTING: &str = "Connexion…";
/// État d'un joueur dans le salon : pair distant connecté.
pub const PLAYER_CONNECTED: &str = "Connecté";
/// État d'un joueur dans le salon : pair distant attendu.
pub const PLAYER_WAITING: &str = "En attente…";

/// Nom d'un joueur du salon (numéro à partir de 1).
pub fn player_name(number: usize) -> String {
    format!("Joueur {number}")
}

/// Nom d'un spectateur du salon (numéro à partir de 1).
pub fn spectator_name(number: usize) -> String {
    format!("Spectateur {number}")
}

// --- Fin de partie (résumé du salon local) ---------------------------------------------------

pub const OUTCOME_DEFEAT: &str = "Défaite";
pub const OUTCOME_VICTORY: &str = "Victoire";
pub const OUTCOME_ABANDON: &str = "Partie abandonnée";

// --- États réseau du salon (ligne d'état, `jjrs::p2p_socket::connect_status_text`) -----------

/// Un pair se fait attendre (`secs` secondes).
pub fn peer_connecting(secs: f32) -> String {
    format!("Connexion au pair… {secs:.0} s")
}

/// Tentative abandonnée, une autre commence.
pub fn peer_retry(attempt: u32, of: u32) -> String {
    format!("Connexion au pair échouée : nouvelle tentative ({attempt}/{of})")
}

/// Toutes les tentatives ont échoué : retour au salon.
pub fn peer_failed_back_to_lobby(attempts: u32) -> String {
    let word = if attempts > 1 {
        "tentatives"
    } else {
        "tentative"
    };
    format!("Connexion au pair échouée après {attempts} {word} : retour au salon")
}

/// Salle pleine, retour au salon.
pub fn room_full_back_to_lobby(peers: usize, max: usize) -> String {
    format!("Salle pleine ({peers} pairs pour {max} places) : retour au salon")
}

/// Salle pleine, attente d'une place.
pub fn room_full_waiting(peers: usize, max: usize) -> String {
    format!("Salle pleine ({peers} pairs pour {max} places) : en attente d'une place")
}

// --- Perte de session -----------------------------------------------------------------------

/// Titre de l'écran : un pair a quitté la partie.
pub const SESSION_LOST_TITLE: &str = "PARTIE INTERROMPUE";
/// Titre de l'écran : la partie commune est perdue (états divergents).
pub const DESYNC_TITLE: &str = "PARTIE COMMUNE PERDUE";
/// Bouton de l'écran de perte de session.
pub const BACK_TO_LOBBY: &str = "Retour au salon";

/// Message : `who` s'est déconnecté.
pub fn peer_disconnected(who: &str) -> String {
    format!("{who} s'est déconnecté")
}

/// Message : la partie commune est perdue à l'image `frame`, détectée avec `who`.
pub fn desync_message(frame: i32, who: &str) -> String {
    format!(
        "La partie commune est perdue : l'état de jeu diffère de celui de {who} (image {frame})."
    )
}

// --- Pause, inventaire, marchand ------------------------------------------------------------

pub const PAUSE_TITLE: &str = "Pause";
pub const PAUSE_RESUME: &str = "Reprendre";
pub const PAUSE_RESTART: &str = "Recommencer";
pub const PAUSE_LOBBY: &str = "Salon";
pub const PAUSE_SETTINGS: &str = "Réglages";
pub const PAUSE_KEYS: &str = "Touches";
pub const PAUSE_HINT: &str = "↑ ↓ choisir · Entrée / A valider · Échap / B reprendre";
pub const INVENTORY_TITLE: &str = "Inventaire";
pub const INVENTORY_HINT: &str = "↑ ↓ parcourir · I / Échap / B fermer";
pub const MERCHANT_TITLE: &str = "Marchand";
pub const MERCHANT_HINT: &str = "↑ ↓ choisir · Entrée / A acheter · Échap / B fermer";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn etats_reseau_en_francais() {
        assert_eq!(peer_connecting(7.4), "Connexion au pair… 7 s");
        assert_eq!(
            peer_retry(2, 2),
            "Connexion au pair échouée : nouvelle tentative (2/2)"
        );
        assert_eq!(
            peer_failed_back_to_lobby(1),
            "Connexion au pair échouée après 1 tentative : retour au salon"
        );
        assert_eq!(
            peer_failed_back_to_lobby(2),
            "Connexion au pair échouée après 2 tentatives : retour au salon"
        );
        assert_eq!(
            room_full_back_to_lobby(3, 2),
            "Salle pleine (3 pairs pour 2 places) : retour au salon"
        );
        assert_eq!(
            room_full_waiting(3, 2),
            "Salle pleine (3 pairs pour 2 places) : en attente d'une place"
        );
    }

    #[test]
    fn perte_de_session() {
        assert_eq!(peer_disconnected("Joueur 2"), "Joueur 2 s'est déconnecté");
        let message = desync_message(120, "Joueur 2");
        assert!(
            message.starts_with("La partie commune est perdue"),
            "{message}"
        );
        assert!(message.contains("Joueur 2") && message.contains("120"));
        assert_eq!(player_name(1), "Joueur 1");
        assert_eq!(spectator_name(2), "Spectateur 2");
    }

    /// Aucun mot anglais des écrans d'avant ne doit revenir dans les constantes.
    #[test]
    fn pas_d_anglais_dans_les_ecrans() {
        let all = [
            LOBBY_WAITING_FOR_PLAYERS,
            NEW_GAME,
            NEW_GAME_PAD_HINT,
            PLAYER_READY,
            PLAYER_CONNECTING,
            PLAYER_CONNECTED,
            PLAYER_WAITING,
            SESSION_LOST_TITLE,
            DESYNC_TITLE,
            BACK_TO_LOBBY,
            PAUSE_LOBBY,
        ];
        let forbidden = [
            "Waiting",
            "Ready",
            "Connecting",
            "Connected",
            "Lobby",
            "lobby",
            "Menu",
            "DESYNC",
            "DISCONNECTED",
            "Frame",
            "Player",
        ];
        for text in all {
            for word in forbidden {
                assert!(!text.contains(word), "« {text} » contient « {word} »");
            }
        }
    }
}
