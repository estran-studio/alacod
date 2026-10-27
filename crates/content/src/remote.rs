//! `alacod remote …` : client du contrôle remote d'un jeu lancé en mode éditeur ou en bac à sable
//! (`docs/editeur-vocal.md`, `docs/conventions.md` §90). Port de l'ancien `scripts/alacod-remote` (Python,
//! supprimé) : JSON-RPC sur HTTP vers `127.0.0.1:<port>` (`ALACOD_REMOTE_PORT`, 15702 par défaut), sans
//! dépendance (un `TcpStream`), pour que la commande `alacod` soit le seul point d'entrée.
//!
//! Les sorties de lecture (`brief`, `profile`, `discover`) sont du texte ; les écritures affichent la
//! réponse JSON du jeu. `--json` rend toujours une enveloppe JSON sur stdout (`{"ok": true, "result": …}` ou
//! `{"ok": false, "error": …, "code": N}`). Codes de sortie stables : 0 réussi, 1 erreur du jeu ou de la commande
//! (garde `check_distance`, en ligne, id inconnu…, message sur stderr), 2 usage, 3 jeu injoignable, 4 assertion
//! fausse ou attente expirée (`alacod remote run`, `script.rs`).

use clap::Subcommand;
use serde_json::{json, Map, Value};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

/// Port du remote : `ALACOD_REMOTE_PORT` ou 15702.
pub fn port() -> u16 {
    std::env::var("ALACOD_REMOTE_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(15702)
}

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Un appel JSON-RPC (`result`, ou `Err` avec le message du jeu ou de la connexion).
pub fn call(method: &str, params: Option<Value>) -> Result<Value, String> {
    call_on(port(), method, params)
}

pub fn call_on(port: u16, method: &str, params: Option<Value>) -> Result<Value, String> {
    let body = json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).to_string();
    let answer = http_post(port, &body)?;
    let answer: Value = serde_json::from_str(&answer)
        .map_err(|e| format!("{method} : réponse illisible ({e}) : {answer}"))?;
    if let Some(error) = answer.get("error") {
        return Err(format!("erreur {method} : {error}"));
    }
    Ok(answer.get("result").cloned().unwrap_or(Value::Null))
}

fn http_post(port: u16, body: &str) -> Result<String, String> {
    let address = ("127.0.0.1", port)
        .to_socket_addrs()
        .map_err(|e| e.to_string())?
        .next()
        .ok_or("adresse locale introuvable")?;
    let unreachable = |e: std::io::Error| {
        format!(
            "{UNREACHABLE_MARK} sur 127.0.0.1:{port} ({e}) : lancer `alacod sandbox <jeu>` \
             (ALACOD_REMOTE_PORT pour un autre port)"
        )
    };
    let mut stream = TcpStream::connect_timeout(&address, REQUEST_TIMEOUT).map_err(unreachable)?;
    stream.set_read_timeout(Some(REQUEST_TIMEOUT)).ok();
    stream.set_write_timeout(Some(REQUEST_TIMEOUT)).ok();
    let request = format!(
        "POST / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|e| e.to_string())?;
    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .map_err(|e| format!("lecture de la réponse : {e}"))?;
    parse_http_response(&raw)
}

/// Corps d'une réponse HTTP/1.1 (`Content-Length`, `chunked` ou jusqu'à la fermeture) ; `Err` si le statut
/// n'est pas 2xx.
fn parse_http_response(raw: &[u8]) -> Result<String, String> {
    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or("réponse HTTP sans en-têtes")?;
    let head = String::from_utf8_lossy(&raw[..split]).to_string();
    let body = &raw[split + 4..];
    let status_line = head.lines().next().unwrap_or_default();
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("statut HTTP illisible : {status_line}"))?;
    let chunked = head
        .lines()
        .any(|l| l.to_ascii_lowercase().starts_with("transfer-encoding:") && l.contains("chunked"));
    let bytes = if chunked {
        let mut out = Vec::new();
        let mut rest = body;
        loop {
            let eol = rest
                .windows(2)
                .position(|w| w == b"\r\n")
                .ok_or("réponse chunked tronquée")?;
            let size = usize::from_str_radix(
                String::from_utf8_lossy(&rest[..eol])
                    .split(';')
                    .next()
                    .unwrap_or("0")
                    .trim(),
                16,
            )
            .map_err(|e| format!("taille de bloc illisible : {e}"))?;
            rest = &rest[eol + 2..];
            if size == 0 {
                break;
            }
            if rest.len() < size {
                return Err("réponse chunked tronquée".to_string());
            }
            out.extend_from_slice(&rest[..size]);
            rest = rest.get(size + 2..).unwrap_or_default();
        }
        out
    } else {
        body.to_vec()
    };
    let text = String::from_utf8_lossy(&bytes).to_string();
    if !(200..300).contains(&status) {
        return Err(format!("HTTP {status} : {text}"));
    }
    Ok(text)
}

// ---------------------------------------------------------------------------------------
// Mise en forme (texte)
// ---------------------------------------------------------------------------------------

/// Valeur JSON comme `print` la montrait en Python : `True`, `None`, chaînes sans guillemets.
pub(crate) fn show(value: &Value) -> String {
    match value {
        Value::Null => "None".to_string(),
        Value::Bool(true) => "True".to_string(),
        Value::Bool(false) => "False".to_string(),
        Value::String(s) => s.clone(),
        Value::Array(items) => format!(
            "[{}]",
            items.iter().map(show).collect::<Vec<_>>().join(", ")
        ),
        other => other.to_string(),
    }
}

fn num(value: &Value, key: &str) -> f64 {
    value.get(key).and_then(Value::as_f64).unwrap_or_default()
}

/// Résumé d'une ligne par joueur et ses cinq ennemis les plus proches (`alacod/state`).
pub fn brief(state: &Value) -> String {
    let mut lines = Vec::new();
    if state.get("editor").is_some() {
        let run = state["run"].get("step").map_or("None".into(), show);
        let sandbox = state["sandbox"].get("profile").and_then(Value::as_str);
        let mut line = format!(
            "editor={} online={} check_distance={} run.step={} jeu={} carte={} graine={}",
            show(&state["editor"]),
            show(&state["online"]),
            show(&state["check_distance"]),
            run,
            show(&state["game"]),
            show(&state["map"]),
            show(&state["seed"]),
        );
        if let Some(profile) = sandbox {
            line.push_str(&format!(" sandbox={profile}"));
        }
        lines.push(line);
    }
    lines.push(format!(
        "frame {}  pause={}  vague={}",
        show(&state["frame"]),
        show(&state["clock"]["paused"]),
        show(&state["wave"])
    ));
    let enemies = state["enemies"].as_array().cloned().unwrap_or_default();
    for player in state["players"].as_array().into_iter().flatten() {
        let weapon = &player["weapon"];
        lines.push(format!(
            "joueur {} ({},{}) pv {}/{} arme {} {}x{}{}",
            show(&player["handle"]),
            show(&player["x"]),
            show(&player["y"]),
            show(&player["health"]),
            show(&player["max_health"]),
            show(&weapon["name"]),
            show(&weapon["ammo"]),
            show(&weapon["reserve"]),
            if weapon["reloading"].as_bool().unwrap_or(false) {
                " recharge"
            } else {
                ""
            }
        ));
        let (px, py) = (num(player, "x"), num(player, "y"));
        let mut near: Vec<&Value> = enemies.iter().collect();
        near.sort_by(|a, b| {
            let da = (num(a, "x") - px).hypot(num(a, "y") - py);
            let db = (num(b, "x") - px).hypot(num(b, "y") - py);
            da.total_cmp(&db)
        });
        for enemy in near.into_iter().take(5) {
            let (dx, dy) = (num(enemy, "x") - px, num(enemy, "y") - py);
            let config = enemy
                .get("config")
                .filter(|c| !c.is_null())
                .map(show)
                .unwrap_or_default();
            lines.push(format!(
                "  ennemi {} {} dx={dx:.0} dy={dy:.0} dist={:.0} pv {} {}",
                show(&enemy["id"]),
                config,
                dx.hypot(dy),
                show(&enemy["health"]),
                show(&enemy["state"])
            ));
        }
    }
    let windows: Vec<String> = state["windows"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|w| show(&w["health"]))
        .collect();
    lines.push(format!(
        "ennemis {}, fenêtres [{}]",
        enemies.len(),
        windows.join(", ")
    ));
    if let Some(commands) = state["commands"].as_array().filter(|c| !c.is_empty()) {
        let pending: Vec<String> = commands
            .iter()
            .map(|c| format!("{}:{}", show(&c["frame"]), show(&c["kind"])))
            .collect();
        lines.push(format!("commandes en attente: {}", pending.join(", ")));
    }
    lines.join("\n")
}

/// Les profils du bac à sable : l'actif marqué d'une étoile, leur arène, leur joueur, leurs personnages posés.
pub fn profiles(result: &Value) -> String {
    let mut lines = Vec::new();
    let active = result.get("active").and_then(Value::as_str);
    for entry in result["profiles"].as_array().into_iter().flatten() {
        let mark = if entry["id"].as_str() == active {
            "*"
        } else {
            " "
        };
        let players: Vec<String> = entry["players"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|player| {
                player
                    .as_object()
                    .into_iter()
                    .flatten()
                    .filter(|(_, v)| !v.is_null() && v.as_array().is_none_or(|a| !a.is_empty()))
                    .map(|(k, v)| format!("{k}={}", show(v)))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect();
        let placed: Vec<String> = entry["characters"]
            .as_array()
            .into_iter()
            .flatten()
            .map(show)
            .collect();
        lines.push(format!(
            "{mark} {}  carte={}  joueurs=[{}]  posés=[{}]",
            show(&entry["id"]),
            show(&entry["map"]),
            players.join("; "),
            placed.join(",")
        ));
    }
    if active.is_none() {
        lines.push(
            "(pas en bac à sable : relancer avec `alacod sandbox <jeu> [--profile P]`)".to_string(),
        );
    }
    lines.join("\n")
}

/// `rpc.discover` : une ligne par méthode (nom, résumé), ses paramètres en dessous.
pub fn discover(doc: &Value) -> String {
    let mut methods: Vec<&Value> = doc["methods"].as_array().into_iter().flatten().collect();
    methods.sort_by_key(|m| m["name"].as_str().unwrap_or_default());
    let mut lines = Vec::new();
    for method in methods {
        let summary = method
            .get("summary")
            .or_else(|| method.get("description"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        lines.push(
            format!("{}  {summary}", method["name"].as_str().unwrap_or_default())
                .trim_end()
                .to_string(),
        );
        for param in method["params"].as_array().into_iter().flatten() {
            let schema = &param["schema"];
            let kind = schema
                .get("type")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| {
                    schema["$ref"]
                        .as_str()
                        .unwrap_or_default()
                        .rsplit('/')
                        .next()
                        .unwrap_or_default()
                        .to_string()
                });
            let optional = if param["required"].as_bool().unwrap_or(false) {
                ""
            } else {
                " (optionnel)"
            };
            lines.push(format!(
                "    {}: {kind}{optional}",
                param["name"].as_str().unwrap_or_default()
            ));
        }
    }
    lines.join("\n")
}

/// Direction de visée (`facing`) → vecteur unitaire non normalisé.
fn facing_vector(facing: &str) -> (f64, f64) {
    match facing {
        "UpRight" => (1.0, 1.0),
        "Up" => (0.0, 1.0),
        "UpLeft" => (-1.0, 1.0),
        "Left" => (-1.0, 0.0),
        "DownLeft" => (-1.0, -1.0),
        "Down" => (0.0, -1.0),
        "DownRight" => (1.0, -1.0),
        _ => (1.0, 0.0),
    }
}

/// `--at X,Y` tel quel ; `--ahead [PX]` : la position du joueur `handle` avancée de `PX` dans son `facing`.
pub fn resolve_position(
    at: Option<&str>,
    ahead: Option<f64>,
    handle: i64,
    state: impl FnOnce() -> Result<Value, String>,
) -> Result<(f64, f64), String> {
    match (at, ahead) {
        (Some(_), Some(_)) | (None, None) => Err("donner --at X,Y ou --ahead [PX]".to_string()),
        (Some(at), None) => {
            let (x, y) = at
                .split_once(',')
                .ok_or_else(|| format!("--at « {at} » : attendu X,Y"))?;
            let parse = |v: &str| {
                v.trim()
                    .parse::<f64>()
                    .map_err(|_| format!("--at « {at} » : nombre illisible « {v} »"))
            };
            Ok((parse(x)?, parse(y)?))
        }
        (None, Some(distance)) => {
            let state = state()?;
            let player = state["players"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|p| p["handle"].as_i64() == Some(handle))
                .ok_or_else(|| format!("joueur {handle} absent"))?;
            let (dx, dy) = facing_vector(player["facing"].as_str().unwrap_or("Right"));
            let norm = dx.hypot(dy);
            Ok((
                num(player, "x") + distance * dx / norm,
                num(player, "y") + distance * dy / norm,
            ))
        }
    }
}

// ---------------------------------------------------------------------------------------
// Commandes
// ---------------------------------------------------------------------------------------

/// `alacod remote <commande>` : pilote le jeu en cours (bac à sable ou mode éditeur) sans relancer le binaire.
#[derive(Subcommand, Debug, Clone)]
pub enum RemoteCommand {
    /// État complet (JSON).
    State,
    /// Résumé : une ligne par joueur, ennemis proches, commandes en attente.
    Brief,
    /// Input maintenu : `input Up,Fire --pan 100,0 [--handle 0]` ; vide = relâcher (rend la main au clavier en
    /// mode éditeur). Boutons : Up Down Left Right Fire SwitchWeapon SwitchWeaponMode Reload Sprint Dash
    /// Modifier Interaction Melee.
    Input {
        #[arg(default_value = "")]
        buttons: String,
        /// Visée `dx,dy`.
        #[arg(long)]
        pan: Option<String>,
        #[arg(long, default_value_t = 0)]
        handle: i64,
    },
    /// Avance de N frames, attend, affiche le résumé, puis REND LA PARTIE comme elle était : relancée si elle
    /// tournait avant (mode éditeur, bac à sable), en pause sinon.
    Step {
        frames: i64,
        /// Rend la main tout de suite : la partie reste en pause (`resume` ensuite).
        #[arg(long)]
        no_wait: bool,
        /// Laisse la partie en pause après le step, même si elle tournait avant.
        #[arg(long)]
        keep_paused: bool,
    },
    Pause,
    Resume,
    /// Capture d'écran PNG de la fenêtre du jeu (rendu seulement). La capture est écrite après la réponse du
    /// jeu : la commande attend que le fichier soit (ré)écrit et que sa taille ne bouge plus entre deux relevés
    /// (code 1 au-delà de `--wait`).
    Screenshot {
        path: String,
        /// Secondes d'attente du fichier complet ; `0` rend la main dès la réponse du jeu (fichier pas encore écrit).
        #[arg(long, default_value_t = 5.0)]
        wait: f64,
    },
    /// Écrit la session jouée en scénario rejouable (`tests/scenarios/ma_session.ron`).
    Save {
        path: String,
    },
    /// Méthodes du remote (`rpc.discover`, OpenRPC) et leurs paramètres.
    Discover,
    /// Relance la partie (Restart par défaut) : le profil du bac à sable est reposé.
    Restart {
        #[arg(long, value_parser = ["Restart", "ToLobby", "EnterRun", "ToHub"])]
        kind: Option<String>,
        #[arg(long)]
        seed: Option<i64>,
    },
    /// Modificateur vivant : `modifier --handle 0 MoveSpeed Mul 0.9` (OP : Add|Pct|Mul|Set).
    Modifier {
        stat: String,
        #[arg(value_parser = ["Add", "Pct", "Mul", "Set"])]
        op: String,
        value: String,
        #[arg(long)]
        handle: Option<i64>,
        #[arg(long)]
        net_id: Option<i64>,
        /// Durée en frames (permanent sinon).
        #[arg(long)]
        until: Option<i64>,
        #[arg(long, default_value = "editor")]
        source: String,
    },
    /// Retire les modificateurs d'une source.
    Unmodify {
        #[arg(long)]
        handle: Option<i64>,
        #[arg(long)]
        net_id: Option<i64>,
        #[arg(long, default_value = "editor")]
        source: String,
    },
    /// Relit le registre de contenu (appliqué au prochain restart).
    Reload,
    /// Profils du bac à sable : sans id, les liste ; avec un id, charge ce profil (arène du profil + restart).
    Profile {
        id: Option<String>,
    },
    /// Donne une arme, un objet ou de l'argent au joueur en cours de partie.
    Give {
        #[arg(long)]
        handle: Option<i64>,
        #[arg(long)]
        weapon: Option<String>,
        #[arg(long)]
        item: Option<String>,
        /// Montant ajouté au solde (négatif : retiré).
        #[arg(long, allow_negative_numbers = true, conflicts_with = "set_currency")]
        currency: Option<i64>,
        /// Solde imposé.
        #[arg(long, allow_negative_numbers = true)]
        set_currency: Option<i64>,
    },
    /// Fait apparaître un personnage : `spawn zombie_1 --ahead` (60 px devant le joueur) ou `--at X,Y`.
    Spawn {
        character: String,
        #[arg(long, allow_hyphen_values = true)]
        at: Option<String>,
        /// Devant le joueur, à PX pixels (60 par défaut).
        #[arg(long, num_args = 0..=1, default_missing_value = "60")]
        ahead: Option<f64>,
        #[arg(long, default_value_t = 0)]
        handle: i64,
        #[arg(long)]
        team: Option<String>,
        #[arg(long)]
        variant: Option<String>,
    },
    /// Fait apparaître un power-up.
    SpawnPowerup {
        id: String,
        #[arg(long, allow_hyphen_values = true)]
        at: Option<String>,
        #[arg(long, num_args = 0..=1, default_missing_value = "60")]
        ahead: Option<f64>,
        #[arg(long, default_value_t = 0)]
        handle: i64,
    },
    /// Fait apparaître un objet au sol.
    SpawnItem {
        id: String,
        #[arg(long, allow_hyphen_values = true)]
        at: Option<String>,
        #[arg(long, num_args = 0..=1, default_missing_value = "60")]
        ahead: Option<f64>,
        #[arg(long, default_value_t = 0)]
        handle: i64,
    },
    /// Joue un fichier texte de commandes, une par ligne (`spawn …`, `give …`, `step 30`, `wait 60 frames`,
    /// `expect enemies == 3`) : un monde de test écrit une fois, rejoué à volonté. S'arrête à la première ligne en
    /// échec (`--keep-going` : continue) ; code de sortie 4 si une assertion est fausse.
    Run {
        /// Fichier de commandes (`#` ou `//` : commentaire ; `alacod remote` devant une ligne est toléré).
        file: String,
        /// Continue après une ligne en échec (le code de sortie reste celui du premier échec).
        #[arg(long)]
        keep_going: bool,
        /// Secondes pendant lesquelles un `expect` est réévalué avant d'échouer (le jeu agit à la frame suivante).
        #[arg(long, default_value_t = 2.0)]
        expect_timeout: f64,
    },
}

fn absolute(path: &str) -> String {
    let path = std::path::Path::new(path);
    if path.is_absolute() {
        path.display().to_string()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path).display().to_string())
            .unwrap_or_else(|_| path.display().to_string())
    }
}

/// Date de modification et taille d'un fichier, `None` s'il n'existe pas.
fn file_stamp(path: &str) -> Option<(std::time::SystemTime, u64)> {
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.modified().ok()?, meta.len()))
}

/// Attend que `path` soit écrit après `before` (absent avant, ou date changée) et que sa taille, non nulle,
/// soit la même sur deux relevés successifs : la capture de bevy est écrite par un observateur, après la réponse.
fn wait_written(
    path: &str,
    before: Option<(std::time::SystemTime, u64)>,
    timeout: Duration,
) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    let mut last: Option<u64> = None;
    loop {
        let now = file_stamp(path);
        match now {
            Some((modified, len)) if len > 0 && before.map_or(true, |(old, _)| old != modified) => {
                if last == Some(len) {
                    return Ok(());
                }
                last = Some(len);
            }
            _ => last = None,
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "capture {path} : fichier pas écrit (ou pas complet) après {:.1} s (--wait pour attendre plus)",
                timeout.as_secs_f64()
            ));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn object(pairs: Vec<(&str, Option<Value>)>) -> Value {
    let mut map = Map::new();
    for (key, value) in pairs {
        if let Some(value) = value {
            map.insert(key.to_string(), value);
        }
    }
    Value::Object(map)
}

/// Code de sortie d'une erreur d'usage (ligne de commande ou script mal formés : même code que `clap`).
pub const EXIT_USAGE: u8 = 2;
/// Code de sortie quand le jeu ne répond pas (pas lancé, mauvais port).
pub const EXIT_UNREACHABLE: u8 = 3;
/// Code de sortie d'une assertion (`expect`) fausse ou d'une attente (`wait`) expirée.
pub const EXIT_ASSERT: u8 = 4;

/// Fragment du message quand le jeu ne répond pas (sert aussi à classer l'erreur : code 3).
pub(crate) const UNREACHABLE_MARK: &str = "le jeu ne répond pas";
/// Début du message d'une assertion fausse (code 4).
pub(crate) const ASSERT_MARK: &str = "assertion fausse";

/// Un échec de commande : le message et le code de sortie du processus (1 : erreur du jeu ou de la commande,
/// 3 : jeu injoignable, 4 : assertion fausse ; 2 est celui de l'usage).
#[derive(Debug, Clone)]
pub struct Failure {
    pub code: u8,
    pub message: String,
    /// Réponse structurée du jeu à conserver sur stdout même quand la commande échoue.
    pub reply: Option<Value>,
}

impl From<String> for Failure {
    fn from(message: String) -> Self {
        let code = if message.contains(UNREACHABLE_MARK) {
            EXIT_UNREACHABLE
        } else if message.starts_with(ASSERT_MARK) {
            EXIT_ASSERT
        } else {
            1
        };
        Failure {
            code,
            message,
            reply: None,
        }
    }
}

impl From<&str> for Failure {
    fn from(message: &str) -> Self {
        message.to_string().into()
    }
}

/// Le résultat d'une commande : ce que le mode texte affiche et la valeur JSON que `--json` rend.
pub type Output = (String, Value);

fn plain(value: Value) -> Output {
    (value.to_string(), value)
}

/// `alacod remote <commande>` de bout en bout : affiche (texte, ou une enveloppe JSON `{"ok", "result" | "error",
/// "code"}` avec `--json`) et rend le code de sortie du processus : 0 réussi, 1 erreur du jeu ou de la commande,
/// 3 jeu injoignable, 4 assertion fausse (`expect`, `wait`) ; 2 est l'usage (`clap`).
pub fn run_cli(command: &RemoteCommand, json: bool, port_override: Option<u16>) -> u8 {
    let port = port_override.unwrap_or_else(port);
    if let RemoteCommand::Run {
        file,
        keep_going,
        expect_timeout,
    } = command
    {
        return crate::script::run_file_cli(port, file, *keep_going, *expect_timeout, json);
    }
    match execute(port, command) {
        Ok((text, value)) => {
            if json {
                println!("{}", json!({"ok": true, "result": value}));
            } else if !text.is_empty() {
                println!("{text}");
            }
            0
        }
        Err(failure) => {
            if json {
                println!(
                    "{}",
                    json!({"ok": false, "error": failure.reply.unwrap_or(Value::String(failure.message)), "code": failure.code})
                );
            } else if let Some(reply) = failure.reply {
                println!("{reply}");
            } else {
                eprintln!("{}", failure.message);
            }
            failure.code
        }
    }
}

/// Exécute une commande sur le jeu qui écoute sur `port`.
pub fn execute(port: u16, command: &RemoteCommand) -> Result<Output, Failure> {
    let call = |method: &str, params: Option<Value>| call_on(port, method, params);
    let state = || call("alacod/state", None);
    match command {
        RemoteCommand::State => {
            let state = state()?;
            Ok((
                serde_json::to_string_pretty(&state).unwrap_or_default(),
                state,
            ))
        }
        RemoteCommand::Brief => {
            let state = state()?;
            Ok((brief(&state), state))
        }
        RemoteCommand::Input {
            buttons,
            pan,
            handle,
        } => {
            let list: Vec<&str> = buttons.split(',').filter(|b| !b.is_empty()).collect();
            let mut params = json!({"handle": handle, "buttons": list});
            if let Some(pan) = pan {
                let values: Result<Vec<i64>, _> =
                    pan.split(',').map(|v| v.trim().parse::<i64>()).collect();
                let values =
                    values.map_err(|_| format!("--pan « {pan} » : attendu dx,dy entiers"))?;
                params["pan"] = json!(values);
            }
            Ok(plain(call("alacod/input", Some(params))?))
        }
        RemoteCommand::Step {
            frames,
            no_wait,
            keep_paused,
        } => {
            // Le jeu se met en pause à chaque `step` : on note s'il tournait pour le relancer après.
            let was_running = !state()?["clock"]["paused"].as_bool().unwrap_or(false);
            let started = call("alacod/step", Some(json!({"frames": frames})))?;
            let target = started["target_frame"]
                .as_i64()
                .ok_or("alacod/step : target_frame absent")?;
            if *no_wait {
                // Sans attendre, on ne sait pas quand relancer : la partie reste en pause.
                return Ok((String::new(), started));
            }
            let deadline = Instant::now() + Duration::from_secs(60);
            let reached = loop {
                match state() {
                    Ok(now)
                        if now["frame"].as_i64().unwrap_or_default() >= target
                            || Instant::now() >= deadline =>
                    {
                        break Ok(now)
                    }
                    Ok(_) => std::thread::sleep(Duration::from_millis(20)),
                    Err(e) => break Err(e),
                }
            };
            let resume = was_running && !*keep_paused;
            if resume {
                // Même après une erreur : ne jamais laisser la partie de William figée.
                call("alacod/resume", None)?;
            }
            let mut now = reached?;
            let mut text = brief(&now);
            if resume {
                text.push_str("\npartie relancée (elle tournait avant le step ; --keep-paused la laisse en pause)");
                now["resumed"] = json!(true);
            }
            Ok((text, now))
        }
        RemoteCommand::Pause => Ok(plain(call("alacod/pause", None)?)),
        RemoteCommand::Resume => Ok(plain(call("alacod/resume", None)?)),
        RemoteCommand::Screenshot { path, wait } => {
            let path = absolute(path);
            let before = file_stamp(&path);
            let value = call("alacod/screenshot", Some(json!({"path": path})))?;
            if *wait > 0.0 {
                wait_written(&path, before, Duration::from_secs_f64(*wait))?;
            }
            Ok(plain(value))
        }
        RemoteCommand::Save { path } => Ok(plain(call(
            "alacod/save_recording",
            Some(json!({"path": absolute(path)})),
        )?)),
        RemoteCommand::Discover => {
            let doc = call("rpc.discover", None)?;
            Ok((discover(&doc), doc))
        }
        RemoteCommand::Restart { kind, seed } => Ok(plain(call(
            "alacod/restart",
            Some(object(vec![
                ("kind", kind.as_ref().map(|k| json!(k))),
                ("seed", seed.map(|s| json!(s))),
            ])),
        )?)),
        RemoteCommand::Modifier {
            stat,
            op,
            value,
            handle,
            net_id,
            until,
            source,
        } => Ok(plain(call(
            "alacod/set_modifier",
            Some(object(vec![
                ("stat", Some(json!(stat))),
                ("op", Some(json!(op))),
                ("value", Some(json!(value))),
                ("source", Some(json!(source))),
                ("handle", handle.map(|h| json!(h))),
                ("net_id", net_id.map(|n| json!(n))),
                ("until_frames", until.map(|u| json!(u))),
            ])),
        )?)),
        RemoteCommand::Unmodify {
            handle,
            net_id,
            source,
        } => Ok(plain(call(
            "alacod/clear_modifiers",
            Some(object(vec![
                ("source", Some(json!(source))),
                ("handle", handle.map(|h| json!(h))),
                ("net_id", net_id.map(|n| json!(n))),
            ])),
        )?)),
        RemoteCommand::Reload => {
            let result = call("alacod/reload_content", None)?;
            let failed = result["errors"].as_array().is_some_and(|e| !e.is_empty());
            if failed {
                return Err(Failure {
                    code: 1,
                    message: "alacod/reload_content : erreurs de lint".into(),
                    reply: Some(result),
                });
            }
            Ok(plain(result))
        }
        RemoteCommand::Profile { id } => {
            let params = match id {
                Some(id) => json!({"id": id}),
                None => json!({}),
            };
            let result = call("alacod/sandbox_profile", Some(params))?;
            Ok(if id.is_some() {
                plain(result)
            } else {
                (profiles(&result), result)
            })
        }
        RemoteCommand::Give {
            handle,
            weapon,
            item,
            currency,
            set_currency,
        } => {
            if weapon.is_none() && item.is_none() && currency.is_none() && set_currency.is_none() {
                return Err("donner --weapon, --item, --currency ou --set-currency".into());
            }
            Ok(plain(call(
                "alacod/give",
                Some(object(vec![
                    ("handle", handle.map(|h| json!(h))),
                    ("weapon", weapon.as_ref().map(|w| json!(w))),
                    ("item", item.as_ref().map(|i| json!(i))),
                    ("currency", currency.map(|c| json!(c))),
                    ("set_currency", set_currency.map(|c| json!(c))),
                ])),
            )?))
        }
        RemoteCommand::Spawn {
            character,
            at,
            ahead,
            handle,
            team,
            variant,
        } => {
            let (x, y) = resolve_position(at.as_deref(), *ahead, *handle, state)?;
            Ok(plain(call(
                "alacod/spawn_character",
                Some(object(vec![
                    ("x", Some(json!(x))),
                    ("y", Some(json!(y))),
                    ("character", Some(json!(character))),
                    ("team", team.as_ref().map(|t| json!(t))),
                    ("variant", variant.as_ref().map(|v| json!(v))),
                ])),
            )?))
        }
        RemoteCommand::SpawnPowerup {
            id,
            at,
            ahead,
            handle,
        } => {
            let (x, y) = resolve_position(at.as_deref(), *ahead, *handle, state)?;
            Ok(plain(call(
                "alacod/spawn_powerup",
                Some(json!({"x": x, "y": y, "id": id})),
            )?))
        }
        RemoteCommand::SpawnItem {
            id,
            at,
            ahead,
            handle,
        } => {
            let (x, y) = resolve_position(at.as_deref(), *ahead, *handle, state)?;
            Ok(plain(call(
                "alacod/spawn_item",
                Some(json!({"x": x, "y": y, "id": id})),
            )?))
        }
        RemoteCommand::Run {
            file,
            keep_going,
            expect_timeout,
        } => crate::script::run_file(port, file, *keep_going, *expect_timeout),
    }
}

/// Attend que le remote réponde (`rpc.discover`), au plus `timeout` ; `alive` dit si le processus du jeu
/// tourne encore (rend la main dès qu'il est mort : inutile d'attendre).
pub fn wait_ready(
    port: u16,
    timeout: Duration,
    mut alive: impl FnMut() -> bool,
) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    loop {
        if call_on(port, "rpc.discover", None).is_ok() {
            return Ok(());
        }
        if !alive() {
            return Err("le jeu s'est arrêté avant d'ouvrir le remote".to_string());
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "le remote ne répond pas après {} s",
                timeout.as_secs()
            ));
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reponse_http_a_longueur_fixe_et_chunked() {
        let fixed = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}";
        assert_eq!(parse_http_response(fixed).unwrap(), "{}");
        let chunked =
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\n{\"a\"\r\n3\r\n:1}\r\n0\r\n\r\n";
        assert_eq!(parse_http_response(chunked).unwrap(), "{\"a\":1}");
        assert!(parse_http_response(b"HTTP/1.1 500 X\r\n\r\nboom")
            .unwrap_err()
            .contains("500"));
    }

    #[test]
    fn brief_reprend_le_format_du_script() {
        let state = json!({
            "editor": true, "online": false, "check_distance": 0, "run": {"step": "InGame"},
            "game": "zombies", "map": "maps/arene_vide.ldtk", "seed": 123456,
            "sandbox": {"profile": "arene"},
            "frame": 120, "clock": {"paused": false}, "wave": 0,
            "players": [{"handle": 0, "x": 424.0, "y": 696.0, "health": 100.0, "max_health": 100.0,
                "weapon": {"name": "pistol", "ammo": 8, "reserve": 64, "reloading": false}}],
            "enemies": [{"id": "7", "config": "zombie_1", "x": 484.0, "y": 696.0, "health": 50.0, "state": "Chasing"}],
            "windows": [], "commands": [{"frame": 130, "kind": "Spawn"}],
        });
        let text = brief(&state);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(
            lines[0],
            "editor=True online=False check_distance=0 run.step=InGame jeu=zombies carte=maps/arene_vide.ldtk graine=123456 sandbox=arene"
        );
        assert_eq!(lines[1], "frame 120  pause=False  vague=0");
        assert_eq!(
            lines[2],
            "joueur 0 (424.0,696.0) pv 100.0/100.0 arme pistol 8x64"
        );
        assert_eq!(
            lines[3],
            "  ennemi 7 zombie_1 dx=60 dy=0 dist=60 pv 50.0 Chasing"
        );
        assert_eq!(lines[4], "ennemis 1, fenêtres []");
        assert_eq!(lines[5], "commandes en attente: 130:Spawn");
    }

    #[test]
    fn position_devant_le_joueur_suit_sa_direction() {
        let state =
            || Ok(json!({"players": [{"handle": 0, "x": 100.0, "y": 200.0, "facing": "Up"}]}));
        assert_eq!(
            resolve_position(None, Some(60.0), 0, state).unwrap(),
            (100.0, 260.0)
        );
        assert_eq!(
            resolve_position(Some("420,300"), None, 0, || Err("jamais".into())).unwrap(),
            (420.0, 300.0)
        );
        assert!(resolve_position(None, None, 0, state).is_err());
        assert!(resolve_position(Some("1,2"), Some(5.0), 0, state).is_err());
        assert!(resolve_position(None, Some(60.0), 3, state)
            .unwrap_err()
            .contains("absent"));
    }
}
