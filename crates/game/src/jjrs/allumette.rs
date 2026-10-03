//! Client HTTP allumette (D12, m0-v9) : authentification par challenge Ed25519,
//! découverte ou création du lobby `zombies`, serveurs ICE, puis WebSocket de
//! signalisation avec le JWT dans le chemin (`ws(s)://hôte/<JWT>`, plus de nom de salle).
//!
//! Hors simulation : ce module ne tourne qu'en phase de lobby (`start_allumette_flow`,
//! chaîné avant `start_matchbox_socket` en `OnEnter(LobbyOnline)`), et rien de ce qu'il
//! fait n'entre dans le rollback GGRS — aucune trace ne doit bouger.
//!
//! Contrat exact du serveur : `docs/taches/m0-v9-allumette-client.md`, vérifié sur
//! `../allumette/src/{lib,auth,lobby,topology}.rs` :
//! - `POST /auth/challenge` → `{"challenge"}` (32 caractères alphanumériques, expire
//!   après 60 s, usage unique, consommé par le login) ;
//! - `POST /auth/login` `{"public_key_b64","username","challenge","signature_b64"}` →
//!   `{"token"}` (JWT HS256, `sub` = clé publique) ; la signature est une signature
//!   Ed25519 `verify_strict` des octets UTF-8 du challenge tels quels, encodages en
//!   base64 STANDARD ;
//! - `GET/POST /lobbies` ; `POST /lobbies/{id}/join` ; le créateur est ajouté d'office ;
//! - `GET /ice-servers` → `[{"urls", "username"?, "credential"?}]` ;
//! - la topologie marque le lobby `InProgress` dès que son propriétaire connecte le
//!   WebSocket (et interdit les nouvelles arrivées) : le créateur attend donc
//!   `player_count == --number-player` avant d'ouvrir le sien ; les rejoigneurs ouvrent
//!   le leur dès que le join a réussi. L'appartenance au lobby survit à une déconnexion.

use bevy::prelude::*;
use ed25519_dalek::Signer;
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::json;

use crate::jjrs::GggrsSessionConfiguration;

/// `game_id` des lobbies du jeu zombies (découverte automatique).
pub const GAME_ID: &str = "zombies";

/// Délai entre deux sondages du lobby par le créateur.
const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);
/// Nombre maximal de sondages du créateur (~ 1 min) avant abandon.
const MAX_POLLS: u32 = 60;
/// Timeout de chaque requête HTTP (un serveur muet ne doit pas bloquer le lobby).
const HTTP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// Configuration du socket allumette : remplie par [`connect`] pendant le flux HTTP,
/// consommée par `start_matchbox_socket` (`jjrs/p2p.rs`) à l'ouverture du socket.
#[derive(Resource, Clone, Debug)]
pub struct AllumetteConfig {
    /// URL complète du WebSocket de signalisation, JWT dans le chemin.
    pub ws_url: String,
    /// Réponse de `GET /ice-servers`. Le builder matchbox n'accepte qu'un seul
    /// `RtcIceServerConfig` : `start_matchbox_socket` utilise la première entrée.
    pub ice_servers: Vec<IceServer>,
}

/// Entrée de la réponse `GET /ice-servers`.
#[derive(Clone, Debug, Deserialize)]
pub struct IceServer {
    pub urls: Vec<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub credential: Option<String>,
}

#[derive(Deserialize)]
struct ChallengeResponse {
    challenge: String,
}

#[derive(Deserialize)]
struct LoginResponse {
    token: String,
}

/// Réponse `GET /lobbies` / `POST /lobbies`, limitée aux champs utiles — les champs
/// ignorés (`is_owner`, `players`, `is_private`, `is_whitelisted`) sont omis du DTO.
#[derive(Clone, Debug, Deserialize)]
pub struct LobbyResponse {
    pub id: String,
    pub game_id: String,
    pub player_count: usize,
    pub status: LobbyStatus,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum LobbyStatus {
    Waiting,
    InProgress,
}

/// Erreurs du flux allumette (HTTP, encodages, attente du lobby).
#[derive(Debug, thiserror::Error)]
pub enum AllumetteError {
    #[error("allumette {url} : HTTP {status} ({body})")]
    Http {
        url: String,
        status: u16,
        body: String,
    },
    #[error("allumette {url} : transport ({source})")]
    Transport { url: String, source: ureq::Error },
    #[error("allumette {url} : réponse JSON illisible ({source})")]
    Json { url: String, source: std::io::Error },
    #[error("entropie système indisponible pour la paire Ed25519 ({source})")]
    Entropy { source: getrandom::Error },
    #[error("URL de base allumette invalide : {0:?} (attendu http(s)://hôte[:port])")]
    BadBaseUrl(String),
    #[error(
        "le lobby {lobby_id} n'a pas atteint {expected} joueur(s) après {seconds} s (toujours {got})"
    )]
    LobbyNotFull {
        lobby_id: String,
        expected: usize,
        got: usize,
        seconds: u64,
    },
}

/// Déroule le flux complet et produit la configuration du socket :
/// paire Ed25519 éphémère → challenge → login → lobby (rejoindre `lobby_arg` s'il est
/// fourni, sinon découverte puis création) → attente du complet si créateur →
/// `/ice-servers` → URL `ws(s)://hôte/<JWT>`.
pub fn connect(
    base_url: &str,
    lobby_arg: Option<&str>,
    number_player: usize,
    username: &str,
) -> Result<AllumetteConfig, AllumetteError> {
    let base = base_url.trim_end_matches('/');

    // Paire Ed25519 éphémère : une par partie, pas de persistance en v1.
    let mut seed = [0u8; 32];
    getrandom::getrandom(&mut seed).map_err(|e| AllumetteError::Entropy { source: e })?;
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&seed);
    let public_key_b64 = encode_standard(signing_key.verifying_key().to_bytes());

    // Challenge → signature → login. Le message signé est la chaîne challenge telle
    // quelle (ses octets UTF-8, sans rien d'autre).
    let challenge: ChallengeResponse =
        http_post_json(&format!("{base}/auth/challenge"), None, None)?;
    let signature_b64 =
        encode_standard(signing_key.sign(challenge.challenge.as_bytes()).to_bytes());
    let login: LoginResponse = http_post_json(
        &format!("{base}/auth/login"),
        Some(json!({
            "public_key_b64": public_key_b64,
            "username": username,
            "challenge": challenge.challenge,
            "signature_b64": signature_b64,
        })),
        None,
    )?;
    let token = login.token;

    // Lobby : `--lobby <uuid>` = rejoindre ce lobby précis ; sinon découverte
    // automatique (liste, filtre game_id + Waiting, join) puis création. Deux clients
    // démarrés en même temps peuvent chacun créer (course connue, limite v1) : les
    // recettes démarrent les clients en décalé.
    let lobby_id = match lobby_arg {
        Some(id) => {
            http_post_no_body(&format!("{base}/lobbies/{id}/join"), Some(&token))?;
            id.to_string()
        }
        None => {
            let lobbies: Vec<LobbyResponse> =
                http_get_json(&format!("{base}/lobbies"), Some(&token))?;
            match select_waiting_lobby(&lobbies, GAME_ID) {
                Some(id) => {
                    http_post_no_body(&format!("{base}/lobbies/{id}/join"), Some(&token))?;
                    id
                }
                None => {
                    let created: LobbyResponse = http_post_json(
                        &format!("{base}/lobbies"),
                        Some(json!({ "is_private": false, "game_id": GAME_ID })),
                        Some(&token),
                    )?;
                    // Le créateur attend le complet avant d'ouvrir le WebSocket : la
                    // topologie marque le lobby InProgress dès que le propriétaire
                    // connecte, ce qui bloquerait les rejoigneurs.
                    wait_until_full(base, &token, &created.id, number_player)?;
                    created.id
                }
            }
        }
    };

    // Serveurs ICE de l'API (remplace le STUN en dur).
    let ice_servers: Vec<IceServer> = http_get_json(&format!("{base}/ice-servers"), Some(&token))?;

    // WebSocket : le JWT dans le chemin, plus de nom de salle.
    let ws_url = http_to_ws(base, &token)?;

    info!("allumette : lobby {lobby_id} prêt, WebSocket {ws_url}");
    Ok(AllumetteConfig {
        ws_url,
        ice_servers,
    })
}

/// Attend que `player_count` du lobby atteigne `number_player` (sondage de
/// `GET /lobbies` toutes les secondes, ~ 1 min au plus).
fn wait_until_full(
    base: &str,
    token: &str,
    lobby_id: &str,
    number_player: usize,
) -> Result<(), AllumetteError> {
    let mut got = 0;
    for attempt in 0..MAX_POLLS {
        let lobbies: Vec<LobbyResponse> = http_get_json(&format!("{base}/lobbies"), Some(token))?;
        if let Some(lobby) = lobbies.iter().find(|lobby| lobby.id == lobby_id) {
            got = lobby.player_count;
            if got >= number_player {
                info!("allumette : lobby {lobby_id} complet ({got}/{number_player})");
                return Ok(());
            }
            if attempt % 5 == 0 {
                info!("allumette : lobby {lobby_id} {got}/{number_player} joueurs, attente…");
            }
        }
        std::thread::sleep(POLL_INTERVAL);
    }
    Err(AllumetteError::LobbyNotFull {
        lobby_id: lobby_id.to_string(),
        expected: number_player,
        got,
        seconds: u64::from(MAX_POLLS),
    })
}

/// Dérive l'URL du WebSocket depuis l'URL HTTP de base : `http`→`ws`, `https`→`wss`,
/// chemin = `/` + JWT.
pub fn http_to_ws(base_url: &str, token: &str) -> Result<String, AllumetteError> {
    let (ws_scheme, host) = if let Some(host) = base_url.strip_prefix("http://") {
        ("ws://", host)
    } else if let Some(host) = base_url.strip_prefix("https://") {
        ("wss://", host)
    } else {
        return Err(AllumetteError::BadBaseUrl(base_url.to_string()));
    };
    Ok(format!("{ws_scheme}{}/{token}", host.trim_end_matches('/')))
}

/// Découverte automatique : le premier lobby `Waiting` du jeu demandé. Retourne son id.
pub fn select_waiting_lobby(lobbies: &[LobbyResponse], game_id: &str) -> Option<String> {
    lobbies
        .iter()
        .find(|lobby| lobby.game_id == game_id && lobby.status == LobbyStatus::Waiting)
        .map(|lobby| lobby.id.clone())
}

/// Base64 STANDARD (le serveur utilise base64 0.21, même alphabet).
fn encode_standard(bytes: impl AsRef<[u8]>) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    STANDARD.encode(bytes)
}

fn http_get_json<T: DeserializeOwned>(url: &str, token: Option<&str>) -> Result<T, AllumetteError> {
    let mut request = ureq::get(url).timeout(HTTP_TIMEOUT);
    if let Some(token) = token {
        request = request.set("Authorization", &format!("Bearer {token}"));
    }
    let response = request.call().map_err(|e| transport_error(url, e))?;
    response.into_json::<T>().map_err(|e| AllumetteError::Json {
        url: url.to_string(),
        source: e,
    })
}

fn http_post_json<T: DeserializeOwned>(
    url: &str,
    body: Option<serde_json::Value>,
    token: Option<&str>,
) -> Result<T, AllumetteError> {
    let mut request = ureq::post(url).timeout(HTTP_TIMEOUT);
    if let Some(token) = token {
        request = request.set("Authorization", &format!("Bearer {token}"));
    }
    let response = match body {
        Some(body) => request.send_json(body),
        None => request.call(),
    }
    .map_err(|e| transport_error(url, e))?;
    response.into_json::<T>().map_err(|e| AllumetteError::Json {
        url: url.to_string(),
        source: e,
    })
}

/// POST sans corps attendu en réponse (`POST /lobbies/{id}/join` répond un 200 vide).
fn http_post_no_body(url: &str, token: Option<&str>) -> Result<(), AllumetteError> {
    let mut request = ureq::post(url).timeout(HTTP_TIMEOUT);
    if let Some(token) = token {
        request = request.set("Authorization", &format!("Bearer {token}"));
    }
    let response = request.call().map_err(|e| transport_error(url, e))?;
    // Drain le corps (vide pour join) pour libérer la connexion ; le statut 2xx a
    // déjà été vérifié par ureq.
    let _ = response.into_string();
    Ok(())
}

fn transport_error(url: &str, error: ureq::Error) -> AllumetteError {
    match error {
        ureq::Error::Status(status, response) => AllumetteError::Http {
            url: url.to_string(),
            status,
            body: response.into_string().unwrap_or_default(),
        },
        error => AllumetteError::Transport {
            url: url.to_string(),
            source: error,
        },
    }
}

/// Flux HTTP allumette, exécuté en `OnEnter(LobbyOnline)` avant `start_matchbox_socket`
/// (chaînés dans `core.rs`). Bloquant : phase de lobby, hors simulation. En cas d'échec
/// le processus s'arrête : pas de socket sans JWT valide, et un binaire muet qui attend
/// pour rien serait pire qu'un échec net.
pub fn start_allumette_flow(mut commands: Commands, ggrs_config: Res<GggrsSessionConfiguration>) {
    if ggrs_config.allumette_url.is_empty() {
        return;
    }
    let username = ggrs_config
        .players
        .iter()
        .find(|player| player.is_local)
        .map(|player| player.name.clone())
        .unwrap_or_else(|| ggrs_config.cid.clone());
    let lobby_arg = if ggrs_config.lobby.is_empty() {
        None
    } else {
        Some(ggrs_config.lobby.as_str())
    };
    match connect(
        &ggrs_config.allumette_url,
        lobby_arg,
        ggrs_config.connection.max_player,
        &username,
    ) {
        Ok(config) => {
            commands.insert_resource(config);
        }
        Err(e) => {
            error!("allumette : {e}");
            eprintln!("allumette : {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Signer;

    #[test]
    fn http_to_ws_derive() {
        assert_eq!(
            http_to_ws("http://127.0.0.1:3536", "jwt").unwrap(),
            "ws://127.0.0.1:3536/jwt"
        );
        assert_eq!(
            http_to_ws("https://sign.example.com:8443", "tok").unwrap(),
            "wss://sign.example.com:8443/tok"
        );
        // Slash final toléré, sans doublon.
        assert_eq!(
            http_to_ws("http://127.0.0.1:3536/", "tok").unwrap(),
            "ws://127.0.0.1:3536/tok"
        );
        // Seuls http(s) sont acceptés.
        assert!(http_to_ws("ws://127.0.0.1:3536", "tok").is_err());
        assert!(http_to_ws("127.0.0.1:3536", "tok").is_err());
    }

    #[test]
    fn lobby_response_from_server_fixture() {
        // Fixture au format exact du serveur (lobby.rs `LobbyResponse`), y compris les
        // champs que le DTO du client ignore (`is_owner`, `players`, `is_private`,
        // `is_whitelisted`).
        let json = r#"[
            {
                "id": "3b5f8f42-7c1a-4e9d-b2f3-1a0b2c3d4e5f",
                "game_id": "zombies",
                "is_owner": false,
                "player_count": 1,
                "players": [],
                "status": "Waiting",
                "is_private": false,
                "is_whitelisted": true
            },
            {
                "id": "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
                "game_id": "zombies",
                "is_owner": true,
                "player_count": 2,
                "players": [
                    {"publicKey": "abc", "is_you": true},
                    {"publicKey": "def", "is_you": false}
                ],
                "status": "InProgress",
                "is_private": false,
                "is_whitelisted": true
            },
            {
                "id": "11111111-2222-3333-4444-555555555555",
                "game_id": "autre-jeu",
                "is_owner": false,
                "player_count": 1,
                "players": [],
                "status": "Waiting",
                "is_private": false,
                "is_whitelisted": true
            }
        ]"#;
        let lobbies: Vec<LobbyResponse> = serde_json::from_str(json).unwrap();
        assert_eq!(lobbies.len(), 3);
        assert_eq!(lobbies[0].status, LobbyStatus::Waiting);
        assert_eq!(lobbies[1].status, LobbyStatus::InProgress);
        assert_eq!(lobbies[1].player_count, 2);
    }

    #[test]
    fn select_waiting_lobby_filters_game_and_status() {
        let lobbies = vec![
            LobbyResponse {
                id: "in-progress".into(),
                game_id: GAME_ID.into(),
                player_count: 2,
                status: LobbyStatus::InProgress,
            },
            LobbyResponse {
                id: "autre-jeu".into(),
                game_id: "autre-jeu".into(),
                player_count: 1,
                status: LobbyStatus::Waiting,
            },
            LobbyResponse {
                id: "la-bonne".into(),
                game_id: GAME_ID.into(),
                player_count: 1,
                status: LobbyStatus::Waiting,
            },
        ];
        assert_eq!(
            select_waiting_lobby(&lobbies, GAME_ID),
            Some("la-bonne".into())
        );
        assert_eq!(select_waiting_lobby(&lobbies, "inconnu"), None);
        assert_eq!(select_waiting_lobby(&lobbies[..1], GAME_ID), None);
    }

    #[test]
    fn challenge_signature_roundtrip() {
        // Signe le challenge avec une paire éphémère puis vérifie avec `verify_strict`,
        // exactement comme le serveur (`auth.rs` `verify_signature`) : la signature
        // porte sur la chaîne challenge telle quelle, et rien d'autre n'est accepté.
        let mut seed = [0u8; 32];
        getrandom::getrandom(&mut seed).unwrap();
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&seed);
        let challenge = "AbCdEfGh1234567890AbCdEfGh1234567890";
        let signature = signing_key.sign(challenge.as_bytes());
        signing_key
            .verifying_key()
            .verify_strict(challenge.as_bytes(), &signature)
            .unwrap();
        // Un autre message ne vérifie pas (ni contenu voisin, ni préfixe).
        assert!(signing_key
            .verifying_key()
            .verify_strict(b"AbCdEfGh1234567890AbCdEfGh1234567890x", &signature)
            .is_err());
    }
}
