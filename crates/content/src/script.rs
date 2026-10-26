//! `alacod remote run <fichier>` : un monde de test écrit en texte (`docs/editeur-vocal.md` §2.5, `docs/conventions.md` §89).
//!
//! Un fichier de commandes, **une par ligne**, dans la syntaxe de `alacod remote` (sans le préfixe, toléré s'il est
//! là) plus trois verbes de script :
//!
//! ```text
//! # commentaire (ou //)
//! restart                              # une commande `alacod remote` : restart, give, spawn, modifier, step…
//! give --weapon shotgun
//! spawn zombie_1 --at 480,640
//! wait 30 frames                       # avance de 30 frames (partie en marche : attend ; en pause : `step`)
//! sleep 0.5                            # secondes (ou `500ms`)
//! expect enemies == 1                  # assertion sur `alacod/state`, réévaluée jusqu'à 2 s (--expect-timeout)
//! expect player.health >= 50 within 5s
//! ```
//!
//! `expect <mesure> <op> <valeur>` : `==`, `!=`, `<`, `<=`, `>`, `>=`. La mesure est un chemin dans le JSON de
//! `alacod/state` (`frame`, `wave`, `clock.paused`…) avec trois commodités : une liste vaut sa **longueur**
//! (`enemies`, `players`, `windows`), `enemies[zombie_1]` compte les ennemis de ce nom (ou de cet état :
//! `enemies[Chasing]`), `player.<champ>` (ou `players[N].<champ>`, N = handle) lit un joueur
//! (`player.health`, `player.weapon.ammo`, `player.x`).
//!
//! Le fichier entier est lu et vérifié **avant** la première commande : une ligne mal formée ne laisse pas un monde
//! à moitié posé. L'exécution s'arrête à la première ligne en échec (`--keep-going` : elle continue).

use std::time::{Duration, Instant};

use clap::Parser;
use serde_json::{json, Value};

use crate::remote::{
    self, call_on, execute, show, Failure, Output, RemoteCommand, ASSERT_MARK, EXIT_ASSERT,
    EXIT_USAGE,
};

/// Un pas du script.
#[derive(Debug, Clone)]
pub enum Step {
    Remote(RemoteCommand),
    Wait(i64),
    Sleep(Duration),
    Expect(Expectation),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl Op {
    fn symbol(self) -> &'static str {
        match self {
            Op::Eq => "==",
            Op::Ne => "!=",
            Op::Lt => "<",
            Op::Le => "<=",
            Op::Gt => ">",
            Op::Ge => ">=",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Expectation {
    pub metric: String,
    pub op: Op,
    pub value: String,
    pub within: Option<Duration>,
}

/// Une ligne du fichier, avec son numéro (à partir de 1).
#[derive(Debug, Clone)]
pub struct Line {
    pub number: usize,
    pub text: String,
    pub step: Step,
}

#[derive(Debug, Clone)]
pub struct ScriptOptions {
    pub keep_going: bool,
    /// Durée pendant laquelle un `expect` est réévalué (le jeu applique une commande à la frame suivante).
    pub expect_timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct LineReport {
    pub number: usize,
    pub text: String,
    pub ok: bool,
    pub output: String,
    pub result: Value,
    pub error: Option<String>,
    pub code: u8,
}

#[derive(Debug, Clone)]
pub struct Report {
    pub lines: Vec<LineReport>,
    /// Lignes non exécutées (arrêt à la première erreur).
    pub skipped: usize,
    /// 0 si tout a réussi, sinon le code de la première ligne en échec.
    pub code: u8,
}

impl Report {
    pub fn failed(&self) -> usize {
        self.lines.iter().filter(|l| !l.ok).count()
    }
}

// ---------------------------------------------------------------------------------------
// Lecture du fichier
// ---------------------------------------------------------------------------------------

/// Mots d'une ligne : guillemets simples ou doubles, `#` et `//` en début de mot commencent un commentaire.
fn split_words(line: &str) -> Result<Vec<String>, String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut in_word = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '#' if !in_word => break,
            '/' if !in_word && chars.peek() == Some(&'/') => break,
            c if c.is_whitespace() => {
                if in_word {
                    words.push(std::mem::take(&mut current));
                    in_word = false;
                }
            }
            '"' | '\'' => {
                in_word = true;
                loop {
                    match chars.next() {
                        None => return Err("guillemet non fermé".to_string()),
                        Some(q) if q == c => break,
                        Some('\\') if c == '"' => match chars.next() {
                            Some(n @ ('"' | '\\')) => current.push(n),
                            Some(n) => {
                                current.push('\\');
                                current.push(n);
                            }
                            None => return Err("guillemet non fermé".to_string()),
                        },
                        Some(other) => current.push(other),
                    }
                }
            }
            other => {
                in_word = true;
                current.push(other);
            }
        }
    }
    if in_word {
        words.push(current);
    }
    Ok(words)
}

#[derive(Parser)]
#[command(no_binary_name = true)]
struct LineCli {
    #[command(subcommand)]
    command: RemoteCommand,
}

fn parse_duration(word: &str) -> Result<Duration, String> {
    let (number, scale) = if let Some(ms) = word.strip_suffix("ms") {
        (ms, 0.001)
    } else {
        (word.strip_suffix('s').unwrap_or(word), 1.0)
    };
    let seconds: f64 = number
        .parse()
        .map_err(|_| format!("durée « {word} » : nombre de secondes attendu (`0.5`, `2s`, `500ms`)"))?;
    if !(seconds >= 0.0 && seconds.is_finite()) {
        return Err(format!("durée « {word} » : positive attendue"));
    }
    Ok(Duration::from_secs_f64(seconds * scale))
}

fn parse_op(text: &str) -> Option<(usize, usize, Op)> {
    let bytes = text.as_bytes();
    for i in 0..bytes.len() {
        let two = text.get(i..i + 2);
        let found = match two {
            Some(">=") => Some((2, Op::Ge)),
            Some("<=") => Some((2, Op::Le)),
            Some("==") => Some((2, Op::Eq)),
            Some("!=") => Some((2, Op::Ne)),
            _ => match bytes[i] {
                b'>' => Some((1, Op::Gt)),
                b'<' => Some((1, Op::Lt)),
                b'=' => Some((1, Op::Eq)),
                _ => None,
            },
        };
        if let Some((len, op)) = found {
            return Some((i, i + len, op));
        }
    }
    None
}

fn parse_expect(rest: &str) -> Result<Expectation, String> {
    // Commentaire de fin de ligne.
    let rest = [" #", " //"]
        .iter()
        .filter_map(|m| rest.find(m))
        .min()
        .map_or(rest, |at| &rest[..at]);
    let (rest, within) = match rest.rfind(" within ") {
        Some(at) => (
            &rest[..at],
            Some(parse_duration(rest[at + " within ".len()..].trim())?),
        ),
        None => (rest, None),
    };
    let (start, end, op) = parse_op(rest)
        .ok_or("expect : opérateur attendu (==, !=, <, <=, >, >=), ex. `expect enemies == 3`")?;
    let metric = rest[..start].trim().to_string();
    let value = rest[end..]
        .trim()
        .trim_matches(|c: char| c == '"' || c == '\'')
        .to_string();
    if metric.is_empty() || value.is_empty() {
        return Err("expect : `expect <mesure> <op> <valeur>` attendu, ex. `expect enemies == 3`".to_string());
    }
    Ok(Expectation {
        metric,
        op,
        value,
        within,
    })
}

fn parse_line(number: usize, raw: &str) -> Result<Option<Line>, String> {
    let words = split_words(raw)?;
    let mut words: &[String] = &words;
    // `alacod remote spawn …` collé tel quel.
    if words.first().map(String::as_str) == Some("alacod") {
        words = &words[1..];
    }
    if words.first().map(String::as_str) == Some("remote") {
        words = &words[1..];
    }
    let Some(verb) = words.first() else {
        return Ok(None);
    };
    let step = match verb.as_str() {
        "wait" => {
            let [_, count, unit @ ..] = words else {
                return Err("wait : `wait <N> [frames]` attendu".to_string());
            };
            if !matches!(unit, [] | [_]) || unit.first().is_some_and(|u| !matches!(u.as_str(), "frames" | "frame" | "f")) {
                return Err("wait : `wait <N> [frames]` attendu (pour des secondes : `sleep`)".to_string());
            }
            let frames: i64 = count
                .parse()
                .map_err(|_| format!("wait : « {count} » n'est pas un nombre de frames"))?;
            if frames <= 0 {
                return Err("wait : un nombre de frames positif est attendu".to_string());
            }
            Step::Wait(frames)
        }
        "sleep" => {
            let [_, duration] = words else {
                return Err("sleep : `sleep <secondes>` attendu".to_string());
            };
            Step::Sleep(parse_duration(duration)?)
        }
        "expect" => {
            let after = raw
                .trim_start()
                .strip_prefix("alacod")
                .map_or(raw.trim_start(), str::trim_start);
            let after = after.strip_prefix("remote").map_or(after, str::trim_start);
            let after = after.strip_prefix("expect").unwrap_or(after);
            Step::Expect(parse_expect(after)?)
        }
        "run" => return Err("run : un script n'en appelle pas un autre".to_string()),
        _ => {
            let parsed = LineCli::try_parse_from(words)
                .map_err(|e| e.to_string().trim_end().to_string())?;
            Step::Remote(parsed.command)
        }
    };
    Ok(Some(Line {
        number,
        text: words_text(raw),
        step,
    }))
}

/// La ligne sans son commentaire de fin, pour l'affichage.
fn words_text(raw: &str) -> String {
    raw.split(" #").next().unwrap_or(raw).trim().to_string()
}

/// Vérifie tout le fichier. L'erreur dit la ligne : `ligne 7 : …`.
pub fn parse_script(source: &str) -> Result<Vec<Line>, String> {
    let mut lines = Vec::new();
    for (index, raw) in source.lines().enumerate() {
        match parse_line(index + 1, raw) {
            Ok(Some(line)) => lines.push(line),
            Ok(None) => {}
            Err(e) => return Err(format!("ligne {} : {e}", index + 1)),
        }
    }
    Ok(lines)
}

// ---------------------------------------------------------------------------------------
// Mesures et comparaisons
// ---------------------------------------------------------------------------------------

/// Lit `lhs` dans l'état de la partie (voir l'en-tête du module).
pub fn metric(state: &Value, lhs: &str) -> Result<Value, String> {
    let lhs = if lhs == "player" || lhs.starts_with("player.") || lhs.starts_with("player[") {
        if lhs.starts_with("player[") {
            lhs.replacen("player[", "players[", 1)
        } else {
            lhs.replacen("player", "players[0]", 1)
        }
    } else {
        lhs.to_string()
    };
    let mut current = state.clone();
    for segment in lhs.split('.') {
        let (name, selector) = match segment.split_once('[') {
            Some((name, rest)) => (
                name,
                Some(
                    rest.strip_suffix(']')
                        .ok_or_else(|| format!("« {segment} » : `]` manquant"))?,
                ),
            ),
            None => (segment, None),
        };
        current = match &current {
            Value::Object(map) => map.get(name).cloned().ok_or_else(|| {
                format!(
                    "mesure « {lhs} » : « {name} » inconnu ; disponibles : {}",
                    map.keys().cloned().collect::<Vec<_>>().join(", ")
                )
            })?,
            Value::Array(items) => name
                .parse::<usize>()
                .ok()
                .and_then(|i| items.get(i).cloned())
                .ok_or_else(|| format!("mesure « {lhs} » : « {name} » n'est pas un indice de liste"))?,
            _ => return Err(format!("mesure « {lhs} » : « {name} » : pas de champ dans une valeur simple")),
        };
        if let Some(selector) = selector {
            let Value::Array(items) = &current else {
                return Err(format!("mesure « {lhs} » : « {name} » n'est pas une liste"));
            };
            current = match selector.parse::<i64>() {
                Ok(n) if name == "players" => items
                    .iter()
                    .find(|p| p["handle"].as_i64() == Some(n))
                    .cloned()
                    .ok_or_else(|| format!("mesure « {lhs} » : pas de joueur de handle {n}"))?,
                Ok(n) => usize::try_from(n)
                    .ok()
                    .and_then(|i| items.get(i).cloned())
                    .ok_or_else(|| format!("mesure « {lhs} » : « {name}[{n}] » hors de la liste"))?,
                Err(_) => json!(items
                    .iter()
                    .filter(|it| {
                        ["config", "id", "kind", "name", "state"]
                            .iter()
                            .any(|key| it.get(*key).is_some_and(|v| show(v) == selector))
                    })
                    .count()),
            };
        }
    }
    match current {
        Value::Array(items) => Ok(json!(items.len())),
        Value::Object(map) => Err(format!(
            "mesure « {lhs} » : un objet, choisir un champ ({})",
            map.keys().cloned().collect::<Vec<_>>().join(", ")
        )),
        other => Ok(other),
    }
}

/// `actual op value` ; les nombres se comparent comme des nombres, les booléens et les textes par égalité.
pub fn compare(actual: &Value, op: Op, value: &str) -> Result<bool, String> {
    if let (Some(a), Ok(b)) = (actual.as_f64(), value.parse::<f64>()) {
        return Ok(match op {
            Op::Eq => (a - b).abs() < 1e-6,
            Op::Ne => (a - b).abs() >= 1e-6,
            Op::Lt => a < b,
            Op::Le => a <= b,
            Op::Gt => a > b,
            Op::Ge => a >= b,
        });
    }
    let equal = if let (Some(a), "true" | "false") = (actual.as_bool(), value) {
        a == (value == "true")
    } else {
        show(actual) == value || actual.as_str() == Some(value)
    };
    match op {
        Op::Eq => Ok(equal),
        Op::Ne => Ok(!equal),
        _ => Err(format!(
            "`{}` ne compare que des nombres ; la mesure (« {} ») ou la valeur (« {value} ») n'en est pas un",
            op.symbol(),
            show(actual)
        )),
    }
}

// ---------------------------------------------------------------------------------------
// Exécution
// ---------------------------------------------------------------------------------------

fn frame_of(state: &Value) -> i64 {
    state["frame"].as_i64().unwrap_or_default()
}

fn paused(state: &Value) -> bool {
    state["clock"]["paused"].as_bool().unwrap_or(false)
}

fn run_expect(port: u16, expect: &Expectation, default: Duration) -> Result<Output, Failure> {
    let deadline = Instant::now() + expect.within.unwrap_or(default);
    loop {
        let state = call_on(port, "alacod/state", None)?;
        let outcome = metric(&state, &expect.metric).and_then(|actual| {
            compare(&actual, expect.op, &expect.value).map(|holds| (holds, actual))
        });
        let last = match outcome {
            Ok((true, actual)) => {
                let text = format!("{} = {}", expect.metric, show(&actual));
                return Ok((text, json!({"metric": expect.metric, "actual": actual})));
            }
            Ok((false, actual)) => format!("obtenu {}", show(&actual)),
            Err(e) => e,
        };
        if Instant::now() >= deadline || paused(&state) {
            return Err(Failure {
                code: EXIT_ASSERT,
                message: format!(
                    "{ASSERT_MARK} : {} {} {} ; {last}",
                    expect.metric,
                    expect.op.symbol(),
                    expect.value
                ),
            });
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn run_wait(port: u16, frames: i64) -> Result<Output, Failure> {
    let first = call_on(port, "alacod/state", None)?;
    if paused(&first) {
        return execute(
            port,
            &RemoteCommand::Step {
                frames,
                no_wait: false,
                keep_paused: false,
            },
        );
    }
    // Les frames s'additionnent d'un relevé au suivant ; un retour en arrière (restart) repart de la nouvelle frame.
    let mut last = frame_of(&first);
    let mut elapsed = 0i64;
    let deadline = Instant::now() + Duration::from_secs_f64(10.0 + frames as f64 / 60.0 * 4.0);
    loop {
        std::thread::sleep(Duration::from_millis(10));
        match call_on(port, "alacod/state", None) {
            Ok(state) => {
                let now = frame_of(&state);
                if now >= last {
                    elapsed += now - last;
                }
                last = now;
                if elapsed >= frames {
                    return Ok((format!("frame {now}"), json!({"frame": now, "waited": elapsed})));
                }
            }
            Err(e) if e.contains(remote::UNREACHABLE_MARK) => return Err(e.into()),
            Err(_) => {}
        }
        if Instant::now() >= deadline {
            return Err(Failure {
                code: EXIT_ASSERT,
                message: format!(
                    "attente expirée : {frames} frames non écoulées (jeu en pause ou arrêté ?), {elapsed} vues"
                ),
            });
        }
    }
}

fn run_step(port: u16, step: &Step, options: &ScriptOptions) -> Result<Output, Failure> {
    match step {
        Step::Remote(command) => execute(port, command),
        Step::Wait(frames) => run_wait(port, *frames),
        Step::Sleep(duration) => {
            std::thread::sleep(*duration);
            Ok((String::new(), json!({"slept": duration.as_secs_f64()})))
        }
        Step::Expect(expect) => run_expect(port, expect, options.expect_timeout),
    }
}

/// Joue les lignes sur le jeu qui écoute sur `port` ; `on_line` voit chaque résultat dès qu'il est connu.
pub fn run(
    port: u16,
    lines: &[Line],
    options: &ScriptOptions,
    on_line: &mut dyn FnMut(&LineReport),
) -> Report {
    let mut report = Report {
        lines: Vec::new(),
        skipped: 0,
        code: 0,
    };
    for (index, line) in lines.iter().enumerate() {
        let line_report = match run_step(port, &line.step, options) {
            Ok((output, result)) => LineReport {
                number: line.number,
                text: line.text.clone(),
                ok: true,
                output,
                result,
                error: None,
                code: 0,
            },
            Err(failure) => LineReport {
                number: line.number,
                text: line.text.clone(),
                ok: false,
                output: String::new(),
                result: Value::Null,
                error: Some(failure.message),
                code: failure.code,
            },
        };
        on_line(&line_report);
        let failed = !line_report.ok;
        if failed && report.code == 0 {
            report.code = line_report.code;
        }
        report.lines.push(line_report);
        if failed && !options.keep_going {
            report.skipped = lines.len() - index - 1;
            break;
        }
    }
    report
}

fn options(keep_going: bool, expect_timeout: f64) -> Result<ScriptOptions, String> {
    if !(expect_timeout >= 0.0 && expect_timeout.is_finite()) {
        return Err("--expect-timeout : un nombre de secondes positif est attendu".to_string());
    }
    Ok(ScriptOptions {
        keep_going,
        expect_timeout: Duration::from_secs_f64(expect_timeout),
    })
}

fn report_json(report: &Report) -> Value {
    json!({
        "ok": report.code == 0,
        "code": report.code,
        "failed": report.failed(),
        "skipped": report.skipped,
        "lines": report.lines.iter().map(|l| json!({
            "line": l.number,
            "command": l.text,
            "ok": l.ok,
            "result": l.result,
            "error": l.error,
            "code": l.code,
        })).collect::<Vec<_>>(),
    })
}

/// `RemoteCommand::Run` dans `execute` : joue le fichier sans rien afficher. Le résultat en dit le bilan ; un échec
/// rend le code de la première ligne en échec.
pub fn run_file(
    port: u16,
    file: &str,
    keep_going: bool,
    expect_timeout: f64,
) -> Result<Output, Failure> {
    let usage = |message: String| Failure {
        code: EXIT_USAGE,
        message,
    };
    let source = std::fs::read_to_string(file).map_err(|e| usage(format!("{file} : {e}")))?;
    let lines = parse_script(&source).map_err(|e| usage(format!("{file}, {e}")))?;
    let options = options(keep_going, expect_timeout).map_err(usage)?;
    let report = run(port, &lines, &options, &mut |_| {});
    let value = report_json(&report);
    if report.code == 0 {
        Ok((format!("{} lignes exécutées", report.lines.len()), value))
    } else {
        let first = report.lines.iter().find(|l| !l.ok);
        Err(Failure {
            code: report.code,
            message: first.map_or_else(String::new, |l| {
                format!(
                    "{file}:{} `{}` : {}",
                    l.number,
                    l.text,
                    l.error.clone().unwrap_or_default()
                )
            }),
        })
    }
}

/// `alacod remote run` : joue le fichier en affichant chaque ligne au fur et à mesure (texte), ou un seul document
/// JSON à la fin (`--json`). Rend le code de sortie.
pub fn run_file_cli(port: u16, file: &str, keep_going: bool, expect_timeout: f64, json: bool) -> u8 {
    let fail = |message: String, code: u8| -> u8 {
        if json {
            println!("{}", json!({"ok": false, "error": message, "code": code}));
        } else {
            eprintln!("{message}");
        }
        code
    };
    let source = match std::fs::read_to_string(file) {
        Ok(source) => source,
        Err(e) => return fail(format!("{file} : {e}"), 1),
    };
    let lines = match parse_script(&source) {
        Ok(lines) => lines,
        Err(e) => return fail(format!("{file}, {e}"), EXIT_USAGE),
    };
    let options = match options(keep_going, expect_timeout) {
        Ok(options) => options,
        Err(e) => return fail(e, EXIT_USAGE),
    };
    let total = lines.len();
    let report = run(port, &lines, &options, &mut |l| {
        if json {
            return;
        }
        println!("[{}] {}", l.number, l.text);
        for out in l.output.lines() {
            println!("      {out}");
        }
        if let Some(error) = &l.error {
            eprintln!("      ÉCHEC (code {}) : {error}", l.code);
        }
    });
    if json {
        println!("{}", report_json(&report));
    } else if report.code == 0 {
        println!("script ok : {total} lignes");
    } else {
        eprintln!(
            "script en échec : {} ligne(s) en échec, {} non exécutée(s)",
            report.failed(),
            report.skipped
        );
    }
    report.code
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> Value {
        json!({
            "frame": 120, "wave": 2, "clock": {"paused": false},
            "players": [
                {"handle": 0, "x": 424.0, "y": 696.0, "health": 80.0, "weapon": {"name": "pistol", "ammo": 6}},
                {"handle": 3, "x": 10.0, "y": 20.0, "health": 40.0},
            ],
            "enemies": [
                {"id": "7", "config": "zombie_1", "state": "Chasing"},
                {"id": "8", "config": "zombie_1", "state": "Idle"},
                {"id": "9", "config": "zombie_2", "state": "Chasing"},
            ],
            "windows": [],
        })
    }

    #[test]
    fn les_mesures_se_lisent_dans_l_etat() {
        let s = state();
        assert_eq!(metric(&s, "enemies").unwrap(), json!(3));
        assert_eq!(metric(&s, "windows").unwrap(), json!(0));
        assert_eq!(metric(&s, "enemies[zombie_1]").unwrap(), json!(2));
        assert_eq!(metric(&s, "enemies[Chasing]").unwrap(), json!(2));
        assert_eq!(metric(&s, "frame").unwrap(), json!(120));
        assert_eq!(metric(&s, "player.health").unwrap(), json!(80.0));
        assert_eq!(metric(&s, "player.weapon.ammo").unwrap(), json!(6));
        assert_eq!(metric(&s, "player[3].health").unwrap(), json!(40.0));
        assert_eq!(metric(&s, "clock.paused").unwrap(), json!(false));
        assert!(metric(&s, "enemis").unwrap_err().contains("enemies"));
        assert!(metric(&s, "player[9].health").unwrap_err().contains("handle 9"));
        assert!(metric(&s, "clock").unwrap_err().contains("paused"));
    }

    #[test]
    fn les_comparaisons_distinguent_nombres_booleens_et_textes() {
        assert!(compare(&json!(3), Op::Eq, "3").unwrap());
        assert!(compare(&json!(3.0), Op::Eq, "3").unwrap());
        assert!(compare(&json!(80.0), Op::Ge, "50").unwrap());
        assert!(!compare(&json!(80.0), Op::Lt, "50").unwrap());
        assert!(compare(&json!(false), Op::Eq, "false").unwrap());
        assert!(compare(&json!("InGame"), Op::Eq, "InGame").unwrap());
        assert!(compare(&json!("InGame"), Op::Ne, "Lobby").unwrap());
        assert!(compare(&json!("InGame"), Op::Gt, "3").is_err());
    }

    #[test]
    fn un_script_se_lit_ligne_par_ligne() {
        let source = "\
# monde de test
restart
give --weapon shotgun   # le fusil
alacod remote spawn zombie_1 --at 480,640
wait 30 frames
sleep 500ms
expect enemies == 1
expect player.health >= 50 within 5s
expect   enemies[zombie_1]=2   # deux

step 3
";
        let lines = parse_script(source).unwrap();
        let kinds: Vec<&str> = lines
            .iter()
            .map(|l| match &l.step {
                Step::Remote(_) => "remote",
                Step::Wait(_) => "wait",
                Step::Sleep(_) => "sleep",
                Step::Expect(_) => "expect",
            })
            .collect();
        assert_eq!(
            kinds,
            ["remote", "remote", "remote", "wait", "sleep", "expect", "expect", "expect", "remote"]
        );
        assert_eq!(lines[1].text, "give --weapon shotgun");
        assert_eq!(lines[1].number, 3);
        let Step::Expect(e) = &lines[6].step else {
            panic!("expect")
        };
        assert_eq!(e.metric, "player.health");
        assert_eq!(e.op, Op::Ge);
        assert_eq!(e.value, "50");
        assert_eq!(e.within, Some(Duration::from_secs(5)));
        let Step::Expect(e) = &lines[7].step else {
            panic!("expect")
        };
        assert_eq!((e.metric.as_str(), e.op, e.value.as_str()), ("enemies[zombie_1]", Op::Eq, "2"));
    }

    #[test]
    fn une_ligne_fausse_est_refusee_avec_son_numero() {
        let cases = [
            ("restart\nspawn", "ligne 2"),
            ("wait vite", "ligne 1"),
            ("wait 0 frames", "positif"),
            ("sleep", "sleep"),
            ("expect enemies", "opérateur"),
            ("expect == 3", "expect"),
            ("run autre.alacod", "un script n'en appelle pas un autre"),
            ("brief\nteleport 1 2", "ligne 2"),
            ("give --weapon 'ouvert", "guillemet"),
        ];
        for (source, needle) in cases {
            let e = parse_script(source).unwrap_err();
            assert!(e.contains(needle), "{source:?} → {e}");
        }
    }
}
