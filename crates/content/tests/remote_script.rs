//! `alacod remote` : codes de sortie stables, `--json`, et `alacod remote run <fichier>` (scripts de commandes
//! avec `wait` et `expect`), contre un faux jeu qui parle JSON-RPC sur HTTP (`docs/editeur-vocal.md` §2.5,
//! `docs/conventions.md` §89). Aucun jeu n'est lancé.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};

#[derive(Default)]
struct Game {
    frame: i64,
    enemies: Vec<String>,
    log: Vec<String>,
    paused: bool,
}

/// Faux jeu : `alacod/state` avance de 5 frames à chaque lecture (une partie en marche), `spawn_character` ajoute un
/// ennemi, `restart` est refusé comme le fait la garde `check_distance`.
struct Fake {
    port: u16,
    game: Arc<Mutex<Game>>,
}

impl Fake {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let game = Arc::new(Mutex::new(Game::default()));
        let shared = game.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                serve(stream, &shared);
            }
        });
        Fake { port, game }
    }

    fn set_paused(&self, paused: bool) {
        self.game.lock().unwrap().paused = paused;
    }

    fn paused(&self) -> bool {
        self.game.lock().unwrap().paused
    }

    fn methods(&self) -> Vec<String> {
        self.game.lock().unwrap().log.clone()
    }
}

fn serve(mut stream: TcpStream, game: &Arc<Mutex<Game>>) {
    let mut raw = Vec::new();
    let mut buf = [0u8; 4096];
    let body = loop {
        let n = stream.read(&mut buf).unwrap_or(0);
        raw.extend_from_slice(&buf[..n]);
        let text = String::from_utf8_lossy(&raw).into_owned();
        if let Some(split) = text.find("\r\n\r\n") {
            let length = text
                .lines()
                .find_map(|l| {
                    l.to_lowercase()
                        .strip_prefix("content-length:")
                        .map(|v| v.trim().parse::<usize>().unwrap())
                })
                .unwrap_or(0);
            if text.len() >= split + 4 + length {
                break text[split + 4..split + 4 + length].to_string();
            }
        }
        if n == 0 {
            return;
        }
    };
    let request: Value = serde_json::from_str(&body).unwrap();
    let method = request["method"].as_str().unwrap().to_string();
    let mut game = game.lock().unwrap();
    game.log.push(method.clone());
    let answer = match method.as_str() {
        "alacod/state" => {
            game.frame += 5;
            let enemies: Vec<Value> = game
                .enemies
                .iter()
                .enumerate()
                .map(|(i, config)| {
                    json!({"id": i, "config": config, "x": 480.0, "y": 640.0, "health": 50.0, "state": "Chasing"})
                })
                .collect();
            json!({"result": {
                "frame": game.frame, "clock": {"paused": game.paused}, "wave": 0,
                "players": [{"handle": 0, "x": 100.0, "y": 200.0, "facing": "Right", "health": 90.0,
                    "max_health": 100.0,
                    "weapon": {"name": "pistol", "ammo": 6, "reserve": 8, "reloading": false}}],
                "enemies": enemies, "windows": [], "commands": [],
            }})
        }
        "alacod/spawn_character" => {
            let character = request["params"]["character"].as_str().unwrap().to_string();
            game.enemies.push(character);
            json!({"result": {"frame": game.frame + 1}})
        }
        "alacod/give" => json!({"result": {"ok": true}}),
        "alacod/step" => {
            game.paused = true;
            game.frame += request["params"]["frames"].as_i64().unwrap();
            json!({"result": {"target_frame": game.frame}})
        }
        "alacod/resume" => {
            game.paused = false;
            json!({"result": {"frame": game.frame}})
        }
        "alacod/restart" => {
            json!({"error": {"code": -32602, "message": "refusé : check_distance=2"}})
        }
        other => json!({"error": {"code": -32601, "message": format!("méthode inconnue {other}")}}),
    };
    let body = answer.to_string();
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
}

fn alacod(port: u16, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_alacod"))
        .env("ALACOD_REMOTE_PORT", port.to_string())
        .arg("remote")
        .args(args)
        .output()
        .unwrap()
}

fn script(name: &str, body: &str) -> PathBuf {
    static N: AtomicU32 = AtomicU32::new(0);
    let path = std::env::temp_dir().join(format!(
        "alacod-script-{name}-{}-{}.alacod",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::write(&path, body).unwrap();
    path
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// Un port sur lequel personne n'écoute.
fn closed_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

#[test]
fn les_codes_de_sortie_sont_stables() {
    let fake = Fake::start();
    assert_eq!(alacod(fake.port, &["brief"]).status.code(), Some(0));
    // Erreur du jeu (garde check_distance) : 1, message sur stderr.
    let o = alacod(fake.port, &["restart"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("check_distance=2"), "{}", err(&o));
    // Usage : 2 (clap).
    assert_eq!(alacod(fake.port, &["spawn"]).status.code(), Some(2));
    // Jeu injoignable : 3.
    let o = alacod(closed_port(), &["brief"]);
    assert_eq!(o.status.code(), Some(3));
    assert!(err(&o).contains("le jeu ne répond pas"), "{}", err(&o));
}

#[test]
fn json_donne_une_enveloppe_sur_stdout_succes_comme_echec() {
    let fake = Fake::start();
    let o = alacod(fake.port, &["--json", "state"]);
    assert_eq!(o.status.code(), Some(0));
    let json: Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["result"]["players"][0]["health"], 90.0);

    // `brief` rend l'état complet en JSON (le texte est une vue de l'état).
    let json: Value = serde_json::from_str(&out(&alacod(fake.port, &["brief", "--json"]))).unwrap();
    assert!(json["result"]["frame"].is_number());

    let o = alacod(fake.port, &["--json", "restart"]);
    assert_eq!(o.status.code(), Some(1));
    let json: Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(
        (json["ok"].clone(), json["code"].clone()),
        (json!(false), json!(1))
    );
    assert!(json["error"].as_str().unwrap().contains("check_distance=2"));
    assert_eq!(err(&o), "", "avec --json, rien sur stderr");

    let o = alacod(closed_port(), &["--json", "brief"]);
    assert_eq!(o.status.code(), Some(3));
    let json: Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(json["code"], 3);
}

#[test]
fn un_script_joue_ses_lignes_dans_l_ordre_et_verifie_ses_assertions() {
    let fake = Fake::start();
    let file = script(
        "ok",
        "# monde de test\n\
         give --weapon shotgun          # une arme\n\
         spawn zombie_1 --at 480,640\n\
         spawn zombie_2 --at 500,640\n\
         wait 12 frames\n\
         expect enemies == 2\n\
         expect enemies[zombie_1] == 1\n\
         expect player.health >= 50\n\
         expect frame > 10\n\
         alacod remote brief\n",
    );
    let o = alacod(fake.port, &["run", file.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "{}\n{}", out(&o), err(&o));
    let text = out(&o);
    assert!(text.contains("[2] give --weapon shotgun"), "{text}");
    assert!(text.contains("enemies = 2"), "{text}");
    assert!(text.contains("script ok : 9 lignes"), "{text}");
    let methods = fake.methods();
    assert_eq!(
        &methods[..3],
        [
            "alacod/give",
            "alacod/spawn_character",
            "alacod/spawn_character"
        ]
    );

    // --json : un seul document, une entrée par ligne (sur un jeu neuf : les spawns de la première passe restent).
    let fake = Fake::start();
    let o = alacod(fake.port, &["--json", "run", file.to_str().unwrap()]);
    let json: Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["lines"].as_array().unwrap().len(), 9);
    assert_eq!(json["lines"][0]["command"], "give --weapon shotgun");
}

#[test]
fn une_assertion_fausse_arrete_le_script_avec_le_code_4() {
    let fake = Fake::start();
    let file = script(
        "faux",
        "spawn zombie_1 --at 480,640\n\
         expect enemies == 3\n\
         give --weapon shotgun\n",
    );
    let o = alacod(
        fake.port,
        &["run", "--expect-timeout", "0.2", file.to_str().unwrap()],
    );
    assert_eq!(o.status.code(), Some(4), "{}", out(&o));
    let message = err(&o);
    assert!(
        message.contains("assertion fausse : enemies == 3 ; obtenu 1"),
        "{message}"
    );
    assert!(message.contains("1 non exécutée"), "{message}");
    assert!(
        !fake.methods().contains(&"alacod/give".to_string()),
        "la ligne suivante ne doit pas partir"
    );

    // --keep-going : la ligne suivante part, le code reste celui du premier échec.
    let o = alacod(
        fake.port,
        &[
            "run",
            "--keep-going",
            "--expect-timeout",
            "0.2",
            file.to_str().unwrap(),
        ],
    );
    assert_eq!(o.status.code(), Some(4));
    assert!(fake.methods().contains(&"alacod/give".to_string()));
}

#[test]
fn un_script_mal_forme_est_refuse_avant_la_premiere_commande() {
    let fake = Fake::start();
    let file = script(
        "syntaxe",
        "give --weapon shotgun\nwait vite\nspawn zombie_1 --at 1,2\n",
    );
    let o = alacod(fake.port, &["run", file.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(2));
    assert!(err(&o).contains("ligne 2"), "{}", err(&o));
    assert!(
        fake.methods().is_empty(),
        "aucune commande n'est partie : {:?}",
        fake.methods()
    );

    let o = alacod(fake.port, &["run", "/chemin/qui/n/existe/pas.alacod"]);
    assert_eq!(o.status.code(), Some(1));
}

#[test]
fn une_erreur_du_jeu_dans_un_script_garde_son_code() {
    let fake = Fake::start();
    let file = script("erreur", "restart\nbrief\n");
    let o = alacod(fake.port, &["run", file.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("check_distance=2"), "{}", err(&o));
    let o = alacod(closed_port(), &["run", file.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(3));
}

#[test]
fn le_script_d_exemple_de_zombies_est_syntaxiquement_valide() {
    // Même vérification que `alacod remote run` avant la première commande, sans jeu.
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../games/zombies/scripts/monde_test.alacod");
    let source = std::fs::read_to_string(&path).unwrap();
    let lines = content::script::parse_script(&source).unwrap();
    assert!(lines.len() >= 8, "{} lignes", lines.len());
}

#[test]
fn step_rend_la_partie_comme_elle_etait() {
    // Elle tournait : relancée après le step (jamais figée par oubli).
    let fake = Fake::start();
    let o = alacod(fake.port, &["step", "10"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("partie relancée"), "{}", out(&o));
    assert!(!fake.paused());
    assert!(fake.methods().contains(&"alacod/resume".to_string()));

    // --keep-paused : elle reste en pause.
    let o = alacod(fake.port, &["step", "10", "--keep-paused"]);
    assert!(fake.paused());
    assert!(!out(&o).contains("partie relancée"), "{}", out(&o));

    // Elle était en pause : elle le reste, sans resume.
    let fake = Fake::start();
    fake.set_paused(true);
    let o = alacod(fake.port, &["--json", "step", "10"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(fake.paused());
    assert!(!fake.methods().contains(&"alacod/resume".to_string()));

    // --no-wait rend la main tout de suite : pause (resume ensuite).
    let fake = Fake::start();
    alacod(fake.port, &["step", "10", "--no-wait"]);
    assert!(fake.paused());
}
