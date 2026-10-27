//! `alacod remote <commande>` : client du contrôle remote d'un jeu lancé avec `ALACOD_REMOTE=1`, `--sandbox`
//! ou `ALACOD_EDITOR=1` (`game::remote`, `docs/conventions.md` §89). Mêmes méthodes BRP `alacod/*` que
//! `scripts/alacod-remote` : un seul protocole (JSON-RPC 2.0 sur HTTP, `127.0.0.1`).

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

use clap::Subcommand;
use serde_json::{json, Map, Value};

pub const DEFAULT_PORT: u16 = 15702;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const STEP_TIMEOUT: Duration = Duration::from_secs(60);

/// Une commande de pilotage. Les boutons : `Up Down Left Right Fire SwitchWeapon SwitchWeaponMode Reload
/// Sprint Dash Modifier Interaction Melee`.
#[derive(Subcommand, Debug)]
pub enum RemoteCommand {
    /// État complet (JSON).
    State,
    /// Résumé : une ligne par joueur, ennemis proches.
    Brief,
    /// Input maintenu (`Up,Fire --pan 100,0`) ; vide : relâcher (en mode éditeur, rend la main au clavier).
    Input {
        #[arg(default_value = "")]
        buttons: String,
        /// Visée `dx,dy`.
        #[arg(long)]
        pan: Option<String>,
        #[arg(long, default_value_t = 0)]
        handle: usize,
    },
    /// Avance de N frames, attend, affiche le résumé.
    Step {
        frames: u32,
        #[arg(long)]
        no_wait: bool,
    },
    Pause,
    Resume,
    /// Capture d'écran (rendu seulement).
    Screenshot {
        path: String,
    },
    /// Écrit la session enregistrée en scénario rejouable.
    Save {
        path: String,
    },
    /// Méthodes du remote (`rpc.discover`) et leurs paramètres.
    Discover,
    /// Relance la partie (mode éditeur ou bac à sable).
    Restart {
        #[arg(long, value_parser = ["Restart", "ToLobby", "EnterRun", "ToHub"])]
        kind: Option<String>,
        #[arg(long, allow_hyphen_values = true)]
        seed: Option<i64>,
    },
    /// Modificateur vivant : `modifier MoveSpeed Mul 0.9`.
    Modifier {
        stat: String,
        #[arg(value_parser = ["Add", "Pct", "Mul", "Set"])]
        op: String,
        #[arg(allow_hyphen_values = true)]
        value: String,
        #[arg(long)]
        handle: Option<usize>,
        #[arg(long)]
        net_id: Option<u64>,
        /// Durée en frames (permanent sinon).
        #[arg(long)]
        until: Option<u32>,
        #[arg(long, default_value = "editor")]
        source: String,
    },
    /// Retire les modificateurs d'une source.
    Unmodify {
        #[arg(long)]
        handle: Option<usize>,
        #[arg(long)]
        net_id: Option<u64>,
        #[arg(long, default_value = "editor")]
        source: String,
    },
    /// Relit le registre de contenu (appliqué au prochain `restart`).
    Reload,
    /// Bac à sable : sans id, liste les profils ; avec un id, charge ce profil (puis restart).
    Profile {
        id: Option<String>,
    },
    /// Donne une arme, un objet ou de l'argent au joueur (`--currency` ajoute, négatif retire).
    Give {
        #[arg(long)]
        handle: Option<usize>,
        #[arg(long)]
        weapon: Option<String>,
        #[arg(long)]
        item: Option<String>,
        #[arg(long, allow_hyphen_values = true, conflicts_with = "set_currency")]
        currency: Option<i64>,
        #[arg(long)]
        set_currency: Option<u32>,
    },
    /// Fait apparaître un personnage : `spawn zombie --ahead`.
    Spawn {
        character: String,
        #[command(flatten)]
        at: Position,
        #[arg(long)]
        team: Option<String>,
        #[arg(long)]
        variant: Option<String>,
    },
    /// Fait apparaître un power-up.
    SpawnPowerup {
        id: String,
        #[command(flatten)]
        at: Position,
    },
    /// Fait apparaître un objet.
    SpawnItem {
        id: String,
        #[command(flatten)]
        at: Position,
    },
}

/// `--at X,Y` ou `--ahead [PX]` (devant le joueur `--handle`, 60 par défaut).
#[derive(clap::Args, Debug)]
pub struct Position {
    #[arg(long, allow_hyphen_values = true)]
    at: Option<String>,
    #[arg(long, num_args = 0..=1, default_missing_value = "60")]
    ahead: Option<f64>,
    #[arg(long, default_value_t = 0)]
    handle: usize,
}

/// Port du remote : `--port`, sinon `ALACOD_REMOTE_PORT`, sinon 15702.
pub fn port_from(arg: Option<u16>) -> Result<u16, String> {
    match (arg, std::env::var("ALACOD_REMOTE_PORT")) {
        (Some(port), _) => Ok(port),
        (None, Ok(value)) => value
            .trim()
            .parse()
            .map_err(|_| format!("ALACOD_REMOTE_PORT = « {value} » n'est pas un port")),
        (None, Err(_)) => Ok(DEFAULT_PORT),
    }
}

/// Client JSON-RPC du remote d'un jeu local.
pub struct Client {
    port: u16,
}

impl Client {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    /// Un appel BRP. `Err` : jeu injoignable, réponse illisible ou erreur JSON-RPC.
    pub fn call(&self, method: &str, params: Option<Value>) -> Result<Value, String> {
        let body =
            json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).to_string();
        let answer = self.post(&body)?;
        let answer: Value = serde_json::from_str(&answer)
            .map_err(|e| format!("{method} : réponse illisible ({e}) : {answer}"))?;
        if let Some(error) = answer.get("error") {
            return Err(format!("erreur {method} : {error}"));
        }
        Ok(answer.get("result").cloned().unwrap_or(Value::Null))
    }

    fn post(&self, body: &str) -> Result<String, String> {
        let addr = ("127.0.0.1", self.port)
            .to_socket_addrs()
            .map_err(|e| e.to_string())?
            .next()
            .ok_or("adresse locale introuvable")?;
        let unreachable = |e: std::io::Error| {
            format!(
                "le jeu ne répond pas sur 127.0.0.1:{} ({e}) : lancé avec `alacod run --sandbox` ou ALACOD_REMOTE=1 ? (`--port` / ALACOD_REMOTE_PORT)",
                self.port
            )
        };
        let mut stream = TcpStream::connect_timeout(&addr, REQUEST_TIMEOUT).map_err(unreachable)?;
        stream
            .set_read_timeout(Some(REQUEST_TIMEOUT))
            .map_err(|e| e.to_string())?;
        stream
            .set_write_timeout(Some(REQUEST_TIMEOUT))
            .map_err(|e| e.to_string())?;
        let request = format!(
            "POST / HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            self.port,
            body.len()
        );
        stream.write_all(request.as_bytes()).map_err(unreachable)?;
        let mut raw = Vec::new();
        stream.read_to_end(&mut raw).map_err(unreachable)?;
        parse_response(&raw)
    }
}

/// Corps d'une réponse HTTP/1.1 (`Content-Length` ou `chunked`) ; erreur si le statut n'est pas 2xx.
fn parse_response(raw: &[u8]) -> Result<String, String> {
    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or("réponse HTTP incomplète")?;
    let head = String::from_utf8_lossy(&raw[..split]).to_string();
    let body = &raw[split + 4..];
    let status = head.lines().next().unwrap_or_default();
    if !status
        .split_whitespace()
        .nth(1)
        .is_some_and(|code| code.starts_with('2'))
    {
        return Err(format!("réponse HTTP : {status}"));
    }
    let chunked = head.lines().any(|l| {
        l.to_ascii_lowercase().starts_with("transfer-encoding:")
            && l.to_ascii_lowercase().contains("chunked")
    });
    if !chunked {
        return Ok(String::from_utf8_lossy(body).into_owned());
    }
    let mut out = Vec::new();
    let mut rest = body;
    loop {
        let eol = rest
            .windows(2)
            .position(|w| w == b"\r\n")
            .ok_or("corps chunked tronqué")?;
        let size = usize::from_str_radix(
            String::from_utf8_lossy(&rest[..eol])
                .split(';')
                .next()
                .unwrap_or("")
                .trim(),
            16,
        )
        .map_err(|_| "taille de chunk illisible")?;
        rest = &rest[eol + 2..];
        if size == 0 {
            break;
        }
        if rest.len() < size + 2 {
            return Err("corps chunked tronqué".into());
        }
        out.extend_from_slice(&rest[..size]);
        rest = &rest[size + 2..];
    }
    Ok(String::from_utf8_lossy(&out).into_owned())
}

// ---------------------------------------------------------------------------------------
// Affichage
// ---------------------------------------------------------------------------------------

/// Valeur JSON comme `print` de Python l'affichait : chaîne sans guillemets, `null` en `None`.
fn plain(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => "None".into(),
        other => other.to_string(),
    }
}

fn num(value: &Value) -> f64 {
    value.as_f64().unwrap_or(0.0)
}

/// Résumé de `alacod/state` : mode éditeur, joueurs et ennemis proches, fenêtres, commandes en attente.
pub fn brief(state: &Value) -> String {
    let mut lines = Vec::new();
    if let Some(editor) = state.get("editor") {
        let step = state
            .pointer("/run/step")
            .map(plain)
            .unwrap_or_else(|| "None".into());
        let sandbox = state.pointer("/sandbox/profile").and_then(Value::as_str);
        lines.push(format!(
            "editor={} online={} check_distance={} run.step={} jeu={} carte={} graine={}{}",
            plain(editor),
            plain(&state["online"]),
            plain(&state["check_distance"]),
            step,
            plain(&state["game"]),
            plain(&state["map"]),
            plain(&state["seed"]),
            sandbox.map(|p| format!(" sandbox={p}")).unwrap_or_default()
        ));
    }
    lines.push(format!(
        "frame {}  pause={}  vague={}",
        plain(&state["frame"]),
        plain(&state["clock"]["paused"]),
        plain(&state["wave"])
    ));
    let enemies = state["enemies"].as_array().cloned().unwrap_or_default();
    for player in state["players"].as_array().into_iter().flatten() {
        let weapon = &player["weapon"];
        lines.push(format!(
            "joueur {} ({},{}) pv {}/{} arme {} {}x{}{}",
            plain(&player["handle"]),
            plain(&player["x"]),
            plain(&player["y"]),
            plain(&player["health"]),
            plain(&player["max_health"]),
            plain(&weapon["name"]),
            plain(&weapon["ammo"]),
            plain(&weapon["reserve"]),
            if weapon["reloading"].as_bool() == Some(true) {
                " recharge"
            } else {
                ""
            }
        ));
        let (px, py) = (num(&player["x"]), num(&player["y"]));
        let mut near: Vec<&Value> = enemies.iter().collect();
        near.sort_by(|a, b| {
            let da = (num(&a["x"]) - px).hypot(num(&a["y"]) - py);
            let db = (num(&b["x"]) - px).hypot(num(&b["y"]) - py);
            da.total_cmp(&db)
        });
        for enemy in near.into_iter().take(5) {
            let (dx, dy) = (num(&enemy["x"]) - px, num(&enemy["y"]) - py);
            lines.push(format!(
                "  ennemi {} {} dx={dx:.0} dy={dy:.0} dist={:.0} pv {} {}",
                plain(&enemy["id"]),
                enemy["config"].as_str().unwrap_or(""),
                dx.hypot(dy),
                plain(&enemy["health"]),
                plain(&enemy["state"])
            ));
        }
    }
    let windows: Vec<String> = state["windows"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|w| plain(&w["health"]))
        .collect();
    lines.push(format!(
        "ennemis {}, fenêtres [{}]",
        enemies.len(),
        windows.join(", ")
    ));
    if let Some(commands) = state["commands"].as_array().filter(|c| !c.is_empty()) {
        let list: Vec<String> = commands
            .iter()
            .map(|c| format!("{}:{}", plain(&c["frame"]), plain(&c["kind"])))
            .collect();
        lines.push(format!("commandes en attente: {}", list.join(", ")));
    }
    lines.join("\n")
}

/// Résultat de `alacod/sandbox_profile` sans id : un profil par ligne, l'actif marqué d'une étoile.
pub fn profiles(result: &Value) -> String {
    let active = result.get("active").and_then(Value::as_str);
    let mut lines = Vec::new();
    for entry in result["profiles"].as_array().into_iter().flatten() {
        let mark = if entry["id"].as_str() == active && active.is_some() {
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
                    .map(|(k, v)| format!("{k}={}", plain(v)))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect();
        let placed: Vec<String> = entry["characters"]
            .as_array()
            .into_iter()
            .flatten()
            .map(plain)
            .collect();
        lines.push(format!(
            "{mark} {}  carte={}  joueurs=[{}]  posés=[{}]",
            plain(&entry["id"]),
            plain(&entry["map"]),
            players.join("; "),
            placed.join(",")
        ));
    }
    if active.is_none() {
        lines.push("(pas en bac à sable : relancer avec `alacod run --sandbox [profil]`)".into());
    }
    lines.join("\n")
}

/// `rpc.discover` : une ligne par méthode, ses paramètres en dessous.
pub fn discover(doc: &Value) -> String {
    let mut methods: Vec<&Value> = doc["methods"].as_array().into_iter().flatten().collect();
    methods.sort_by_key(|m| m["name"].as_str().unwrap_or("").to_string());
    let mut lines = Vec::new();
    for method in methods {
        let summary = method["summary"]
            .as_str()
            .or_else(|| method["description"].as_str())
            .unwrap_or("");
        lines.push(
            format!("{}  {summary}", plain(&method["name"]))
                .trim_end()
                .to_string(),
        );
        for param in method["params"].as_array().into_iter().flatten() {
            let schema = &param["schema"];
            let kind = schema["type"]
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| {
                    schema["$ref"]
                        .as_str()
                        .unwrap_or("")
                        .rsplit('/')
                        .next()
                        .unwrap_or("")
                        .to_string()
                });
            let optional = if param["required"].as_bool() == Some(true) {
                ""
            } else {
                " (optionnel)"
            };
            lines.push(format!("    {}: {kind}{optional}", plain(&param["name"])));
        }
    }
    lines.join("\n")
}

fn facing(name: &str) -> (f64, f64) {
    match name {
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

fn resolve_position(client: &Client, at: &Position) -> Result<(f64, f64), String> {
    match (&at.at, at.ahead) {
        (Some(_), Some(_)) | (None, None) => Err("donner --at X,Y ou --ahead [PX]".into()),
        (Some(at), None) => {
            let (x, y) = at.split_once(',').ok_or("--at attend X,Y")?;
            let parse = |v: &str| {
                v.trim()
                    .parse::<f64>()
                    .map_err(|_| format!("--at : « {v} » n'est pas un nombre"))
            };
            Ok((parse(x)?, parse(y)?))
        }
        (None, Some(distance)) => {
            let state = client.call("alacod/state", None)?;
            let player = state["players"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|p| p["handle"].as_u64() == Some(at.handle as u64))
                .ok_or_else(|| format!("joueur {} absent", at.handle))?;
            let (dx, dy) = facing(player["facing"].as_str().unwrap_or("Right"));
            let norm = dx.hypot(dy);
            Ok((
                num(&player["x"]) + distance * dx / norm,
                num(&player["y"]) + distance * dy / norm,
            ))
        }
    }
}

fn absolute(path: &str) -> Result<String, String> {
    let path = std::path::Path::new(path);
    let full = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    Ok(full.to_string_lossy().into_owned())
}

fn insert<T: Into<Value>>(map: &mut Map<String, Value>, key: &str, value: Option<T>) {
    if let Some(value) = value {
        map.insert(key.to_string(), value.into());
    }
}

/// Résultat d'une commande : le texte à afficher sur stdout, et si la commande a **échoué** malgré une
/// réponse du jeu (code de sortie 1 ; le texte reste sur stdout, comme `scripts/alacod-remote`).
pub struct Reply {
    pub text: String,
    pub failed: bool,
}

/// `alacod/reload_content` : le JSON du jeu va sur stdout, erreurs de contenu comprises, et la commande échoue
/// s'il y en a (même contrat que `scripts/alacod-remote reload`).
fn reload(client: &Client) -> Result<(String, bool), String> {
    let result = client.call("alacod/reload_content", None)?;
    let failed = result["errors"].as_array().is_some_and(|e| !e.is_empty());
    Ok((result.to_string(), failed))
}

/// Exécute une commande. `Err` : message pour stderr (code de sortie 1).
pub fn run(client: &Client, command: RemoteCommand) -> Result<Reply, String> {
    if matches!(command, RemoteCommand::Reload) {
        return reload(client).map(|(text, failed)| Reply { text, failed });
    }
    execute(client, command).map(|text| Reply {
        text,
        failed: false,
    })
}

/// Exécute une commande et rend le texte à afficher. `Err` : message pour stderr (code de sortie 1).
fn execute(client: &Client, command: RemoteCommand) -> Result<String, String> {
    match command {
        RemoteCommand::State => {
            Ok(serde_json::to_string_pretty(&client.call("alacod/state", None)?).unwrap())
        }
        RemoteCommand::Brief => Ok(brief(&client.call("alacod/state", None)?)),
        RemoteCommand::Input {
            buttons,
            pan,
            handle,
        } => {
            let list: Vec<&str> = buttons.split(',').filter(|b| !b.is_empty()).collect();
            let mut params = json!({"handle": handle, "buttons": list});
            if let Some(pan) = pan {
                let parts: Result<Vec<i64>, _> =
                    pan.split(',').map(|v| v.trim().parse::<i64>()).collect();
                params["pan"] =
                    json!(parts.map_err(|_| format!("--pan attend dx,dy entiers : « {pan} »"))?);
            }
            Ok(plain(&client.call("alacod/input", Some(params))?))
        }
        RemoteCommand::Step { frames, no_wait } => {
            let target = client.call("alacod/step", Some(json!({"frames": frames})))?
                ["target_frame"]
                .as_u64()
                .ok_or("alacod/step : target_frame absent")?;
            if no_wait {
                return Ok(String::new());
            }
            let deadline = Instant::now() + STEP_TIMEOUT;
            loop {
                let state = client.call("alacod/state", None)?;
                if state["frame"].as_u64().unwrap_or(0) >= target || Instant::now() >= deadline {
                    return Ok(brief(&state));
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        }
        RemoteCommand::Pause => Ok(plain(&client.call("alacod/pause", None)?)),
        RemoteCommand::Resume => Ok(plain(&client.call("alacod/resume", None)?)),
        RemoteCommand::Screenshot { path } => Ok(plain(
            &client.call("alacod/screenshot", Some(json!({"path": absolute(&path)?})))?,
        )),
        RemoteCommand::Save { path } => Ok(plain(&client.call(
            "alacod/save_recording",
            Some(json!({"path": absolute(&path)?})),
        )?)),
        RemoteCommand::Discover => Ok(discover(&client.call("rpc.discover", None)?)),
        RemoteCommand::Restart { kind, seed } => {
            let mut params = Map::new();
            insert(&mut params, "kind", kind);
            insert(&mut params, "seed", seed);
            Ok(client
                .call("alacod/restart", Some(Value::Object(params)))?
                .to_string())
        }
        RemoteCommand::Modifier {
            stat,
            op,
            value,
            handle,
            net_id,
            until,
            source,
        } => {
            let mut params = Map::new();
            params.insert("stat".into(), stat.into());
            params.insert("op".into(), op.into());
            params.insert("value".into(), value.into());
            params.insert("source".into(), source.into());
            insert(&mut params, "handle", handle);
            insert(&mut params, "net_id", net_id);
            insert(&mut params, "until_frames", until);
            Ok(client
                .call("alacod/set_modifier", Some(Value::Object(params)))?
                .to_string())
        }
        RemoteCommand::Unmodify {
            handle,
            net_id,
            source,
        } => {
            let mut params = Map::new();
            params.insert("source".into(), source.into());
            insert(&mut params, "handle", handle);
            insert(&mut params, "net_id", net_id);
            Ok(client
                .call("alacod/clear_modifiers", Some(Value::Object(params)))?
                .to_string())
        }
        RemoteCommand::Reload => reload(client).map(|(text, _)| text),
        RemoteCommand::Profile { id } => match id {
            Some(id) => Ok(client
                .call("alacod/sandbox_profile", Some(json!({"id": id})))?
                .to_string()),
            None => Ok(profiles(
                &client.call("alacod/sandbox_profile", Some(json!({})))?,
            )),
        },
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
            let mut params = Map::new();
            insert(&mut params, "handle", handle);
            insert(&mut params, "weapon", weapon);
            insert(&mut params, "item", item);
            insert(&mut params, "currency", currency);
            insert(&mut params, "set_currency", set_currency);
            Ok(client
                .call("alacod/give", Some(Value::Object(params)))?
                .to_string())
        }
        RemoteCommand::Spawn {
            character,
            at,
            team,
            variant,
        } => {
            let (x, y) = resolve_position(client, &at)?;
            let mut params = Map::new();
            params.insert("x".into(), x.into());
            params.insert("y".into(), y.into());
            params.insert("character".into(), character.into());
            insert(&mut params, "team", team);
            insert(&mut params, "variant", variant);
            Ok(client
                .call("alacod/spawn_character", Some(Value::Object(params)))?
                .to_string())
        }
        RemoteCommand::SpawnPowerup { id, at } => {
            spawn_pickup(client, "alacod/spawn_powerup", id, &at)
        }
        RemoteCommand::SpawnItem { id, at } => spawn_pickup(client, "alacod/spawn_item", id, &at),
    }
}

fn spawn_pickup(
    client: &Client,
    method: &str,
    id: String,
    at: &Position,
) -> Result<String, String> {
    let (x, y) = resolve_position(client, at)?;
    Ok(client
        .call(method, Some(json!({"x": x, "y": y, "id": id})))?
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_bodies_are_read_with_content_length_or_chunked() {
        let plain_response = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}";
        assert_eq!(parse_response(plain_response).unwrap(), "{}");
        let chunked = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabc\r\n2\r\nde\r\n0\r\n\r\n";
        assert_eq!(parse_response(chunked).unwrap(), "abcde");
        assert!(parse_response(b"HTTP/1.1 500 X\r\n\r\n").is_err());
    }

    #[test]
    fn brief_lists_the_nearest_enemies_first() {
        let state = json!({
            "frame": 12, "clock": {"paused": false}, "wave": null,
            "players": [{"handle": 0, "x": 0.0, "y": 0.0, "health": 5, "max_health": 10,
                         "weapon": {"name": "pistol", "ammo": 3, "reserve": 9, "reloading": true}}],
            "enemies": [
                {"id": 2, "x": 100.0, "y": 0.0, "config": "far", "health": 1, "state": "Chasing"},
                {"id": 1, "x": 3.0, "y": 4.0, "config": "near", "health": 2, "state": "Idle"}
            ],
            "windows": [{"health": 7}], "commands": [{"frame": 20, "kind": "SpawnItem"}]
        });
        let text = brief(&state);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[0], "frame 12  pause=false  vague=None");
        assert_eq!(
            lines[1],
            "joueur 0 (0.0,0.0) pv 5/10 arme pistol 3x9 recharge"
        );
        assert_eq!(lines[2], "  ennemi 1 near dx=3 dy=4 dist=5 pv 2 Idle");
        assert!(lines[3].starts_with("  ennemi 2 far"));
        assert_eq!(lines[4], "ennemis 2, fenêtres [7]");
        assert_eq!(lines[5], "commandes en attente: 20:SpawnItem");
    }
}
