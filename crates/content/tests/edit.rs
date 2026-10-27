//! `alacod get` / `set` / `undo` (`docs/editeur-vocal.md` §2, `docs/conventions.md` §90) : un jeu de travail
//! (copie du gabarit `templates/starter`, qui passe le lint) dans un dépôt git temporaire. Jamais l'engine.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU32, Ordering};

const PLAYER: &str = "assets/ZombieShooter/Sprites/Character/player_config.ron";
const CAMERA: &str = "assets/camera.ron";

/// Jeu de travail unique, supprimé à la fin du test.
struct Work {
    root: PathBuf,
}

impl Work {
    fn new(label: &str) -> Self {
        static N: AtomicU32 = AtomicU32::new(0);
        let root = std::env::temp_dir()
            .join(format!(
                "alacod-edit-{label}-{}-{}",
                std::process::id(),
                N.fetch_add(1, Ordering::SeqCst)
            ))
            .join("jeu");
        std::fs::create_dir_all(&root).unwrap();
        let root = root.canonicalize().unwrap();
        copy_dir(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/starter"),
            &root,
        );
        let work = Self { root };
        work.git(&["init", "-q"]);
        work.git(&["config", "user.name", "Test"]);
        work.git(&["config", "user.email", "test@example.com"]);
        work.git(&["config", "commit.gpgsign", "false"]);
        work.git(&["add", "-A"]);
        work.git(&["commit", "-q", "-m", "départ"]);
        work
    }

    fn git(&self, args: &[&str]) -> String {
        let out = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?} : {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim_end().to_string()
    }

    fn alacod(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_alacod"))
            .current_dir(&self.root)
            .args(args)
            .output()
            .unwrap()
    }

    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.root.join(rel)).unwrap()
    }

    fn write(&self, rel: &str, text: &str) {
        std::fs::write(self.root.join(rel), text).unwrap();
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.root.parent().unwrap());
    }
}

fn copy_dir(from: &Path, to: &Path) {
    for entry in std::fs::read_dir(from).unwrap().flatten() {
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            std::fs::create_dir_all(&target).unwrap();
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// Les lignes qui diffèrent entre deux textes de même longueur.
fn changed_lines(before: &str, after: &str) -> Vec<(String, String)> {
    assert_eq!(before.lines().count(), after.lines().count());
    before
        .lines()
        .zip(after.lines())
        .filter(|(a, b)| a != b)
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}

#[test]
fn get_lit_une_valeur_un_bloc_et_refuse_ce_qui_n_existe_pas() {
    let w = Work::new("get");
    let o = w.alacod(&["get", "Character", "player", "movement.max_speed"]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o).trim(), "\"150.0\"");

    // Dossier du jeu en premier argument, id d'arme, chemin à travers table et variante.
    let o = w.alacod(&[
        "get",
        w.root.to_str().unwrap(),
        "Weapon",
        "pistol",
        "config.firing_modes.default.bullet_type.Standard.damage",
    ]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o).trim(), "\"15.0\"");

    // Un `Some(..)` se traverse ; un bloc rend son texte et ses réglages.
    let o = w.alacod(&["get", "Character", "player", "base_health.regen_rate"]);
    assert_eq!(out(&o).trim(), "\"10.0\"");
    let o = w.alacod(&["get", "Character", "player", "base_health"]);
    assert!(
        out(&o).contains("réglages : base_health.max, base_health.regen_rate"),
        "{}",
        out(&o)
    );

    let o = w.alacod(&["get", "--json", "Camera", "camera", "lerp_speed"]);
    let json: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(json["value"], "12.0");
    assert_eq!(json["file"], "camera.ron");
    assert_eq!(json["scalar"], true);

    let o = w.alacod(&["get", "Character", "player", "movement.vitesse"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("movement.max_speed"), "{}", err(&o));

    let o = w.alacod(&["get", "Character", "plyer", "movement.max_speed"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("le plus proche : player"), "{}", err(&o));

    let o = w.alacod(&["get", "Personnage", "player", "x"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("kinds du jeu"), "{}", err(&o));
}

#[test]
fn set_ne_change_que_la_valeur_et_dit_quand_elle_prend_effet() {
    let w = Work::new("set");
    let before = w.read(PLAYER);
    let o = w.alacod(&["set", "Character", "player", "movement.max_speed", "120"]);
    assert!(o.status.success(), "{}", err(&o));
    let line = out(&o);
    assert!(line.contains("150.0 -> 120.0"), "{line}");
    assert!(line.contains("effet : au prochain spawn"), "{line}");
    assert_eq!(line.lines().count(), 1, "une seule ligne : {line}");

    // Un Fixed reste une chaîne `"120.0"` ; rien d'autre ne bouge (commentaires, indentation, lignes vides).
    let after = w.read(PLAYER);
    let diff = changed_lines(&before, &after);
    assert_eq!(diff.len(), 1, "{diff:?}");
    assert_eq!(diff[0].0.trim(), "max_speed: \"150.0\",");
    assert_eq!(diff[0].1.trim(), "max_speed: \"120.0\",");
    assert_eq!(
        before.replacen("max_speed: \"150.0\"", "max_speed: \"120.0\"", 1),
        after
    );

    // Un entier reste un entier, `Some(..)` se règle par sa valeur, un commentaire de fin de ligne survit.
    let before = after;
    let o = w.alacod(&[
        "set",
        "Character",
        "player",
        "movement.dash_cooldown_frames",
        "30",
    ]);
    assert!(o.status.success(), "{}", err(&o));
    let o = w.alacod(&[
        "set",
        "Character",
        "player",
        "base_health.regen_rate",
        "12.5",
    ]);
    assert!(o.status.success(), "{}", err(&o));
    let o = w.alacod(&[
        "set",
        "Character",
        "player",
        "base_health.regen_delay_frames",
        "120",
    ]);
    assert!(o.status.success(), "{}", err(&o));
    let after = w.read(PLAYER);
    let diff = changed_lines(&before, &after);
    assert_eq!(diff.len(), 3, "{diff:?}");
    assert!(
        diff[0]
            .1
            .contains("dash_cooldown_frames: 30,      // 0,4 s"),
        "{}",
        diff[0].1
    );
    assert!(
        diff[1]
            .1
            .contains("regen_rate: Some(\"12.5\"),  // 1 HP per second"),
        "{}",
        diff[1].1
    );
    assert!(
        diff[2]
            .1
            .contains("regen_delay_frames: Some(120),  // 3 seconds"),
        "{}",
        diff[2].1
    );

    // Une valeur déjà posée ne réécrit rien.
    let o = w.alacod(&["set", "Character", "player", "movement.max_speed", "120.0"]);
    assert!(out(&o).contains("rien changé"), "{}", out(&o));
    assert_eq!(w.read(PLAYER), after);

    // Valeur négative acceptée comme valeur.
    let o = w.alacod(&["set", "Camera", "camera", "lerp_speed", "-5"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(
        w.read(CAMERA).contains("lerp_speed: -5.0,"),
        "{}",
        w.read(CAMERA)
    );
}

#[test]
fn set_relatif_garde_le_type_et_montre_l_ancienne_et_la_nouvelle_valeur() {
    let w = Work::new("relatif");
    let set = |path: &str, value: &str| w.alacod(&["set", "Character", "player", path, value]);

    // Facteur sur un `Fixed` : reste une chaîne avec sa virgule, sans bruit de flottant.
    let o = set("movement.max_speed", "x1.1");
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).contains("150.0 -> 165.0"), "{}", out(&o));
    assert!(w.read(PLAYER).contains("max_speed: \"165.0\","));
    let o = set("movement.max_speed", "*0.9");
    assert!(out(&o).contains("165.0 -> 148.5"), "{}", out(&o));
    // Pourcentages (signe obligatoire, `-` en tête accepté comme valeur), puis ajout.
    let o = set("movement.max_speed", "-10%");
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).contains("148.5 -> 133.65"), "{}", out(&o));
    let o = set("movement.max_speed", "+=6.35");
    assert!(out(&o).contains("133.65 -> 140.0"), "{}", out(&o));
    let o = set("movement.max_speed", "-=40");
    assert!(out(&o).contains("140.0 -> 100.0"), "{}", out(&o));
    assert!(w.read(PLAYER).contains("max_speed: \"100.0\","));

    // Entier : reste un entier ; un résultat fractionnaire est refusé avec les deux entiers voisins.
    let o = set("movement.dash_cooldown_frames", "+50%");
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).contains("24 -> 36"), "{}", out(&o));
    assert!(w
        .read(PLAYER)
        .contains("dash_cooldown_frames: 36,      // 0,4 s"));
    let before = w.read(PLAYER);
    let o = set("movement.dash_cooldown_frames", "x0.9");
    assert_eq!(o.status.code(), Some(1));
    assert!(
        err(&o).contains("32.4") && err(&o).contains("donner 32 ou 33"),
        "{}",
        err(&o)
    );
    assert_eq!(w.read(PLAYER), before);
    let o = set("base_health.regen_delay_frames", "-=30");
    assert!(out(&o).contains("180 -> 150"), "{}", out(&o));

    // Sur autre chose qu'un nombre : refusé, rien d'écrit.
    let before = w.read(PLAYER);
    let o = set("asset_name_ref", "x0.9");
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("valeur relative"), "{}", err(&o));
    assert_eq!(w.read(PLAYER), before);

    // Nombre non chaîne ; `-5` et `+5` restent absolus.
    let o = w.alacod(&["set", "Camera", "camera", "lerp_speed", "+=0.5"]);
    assert!(out(&o).contains("12.0 -> 12.5"), "{}", out(&o));
    let o = w.alacod(&["set", "Camera", "camera", "lerp_speed", "+5"]);
    assert!(out(&o).contains("12.5 -> 5.0 ("), "{}", out(&o));
    let o = w.alacod(&["set", "Camera", "camera", "lerp_speed", "-5", "--json"]);
    let json: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(
        (json["old"].as_str(), json["new"].as_str()),
        (Some("5.0"), Some("-5.0"))
    );
}

#[test]
fn set_refuse_les_valeurs_de_mauvais_type_sans_rien_ecrire() {
    let w = Work::new("types");
    let before = w.read(PLAYER);
    let o = w.alacod(&[
        "set",
        "Character",
        "player",
        "movement.dash_cooldown_frames",
        "30.5",
    ]);
    assert!(err(&o).contains("entier attendu"), "{}", err(&o));
    // Une valeur de `Fixed` non numérique est une chaîne : le lint tranche, le fichier est restauré.
    let o = w.alacod(&["set", "Character", "player", "movement.max_speed", "vite"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("restauré"), "{}", err(&o));
    // Un bloc n'est pas une valeur : les réglages possibles sont listés.
    let o = w.alacod(&["set", "Character", "player", "movement", "3"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(
        err(&o).contains("movement.max_speed") && err(&o).contains("--raw"),
        "{}",
        err(&o)
    );
    assert_eq!(w.read(PLAYER), before);
}

#[test]
fn le_lint_rouge_restaure_le_fichier() {
    let w = Work::new("lint");
    let before = w.read(PLAYER);
    let o = w.alacod(&["set", "Character", "player", "starting_skin", "9"]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    let message = err(&o);
    assert!(
        message.contains("lint rouge") && message.contains("starting_skin"),
        "{message}"
    );
    assert!(message.contains("player_config.ron restauré"), "{message}");
    assert_eq!(
        w.read(PLAYER),
        before,
        "le fichier doit être restauré octet pour octet"
    );
    assert_eq!(w.git(&["status", "--porcelain"]), "");

    // Avec --json, l'erreur est sur stdout.
    let o = w.alacod(&["set", "--json", "Character", "player", "starting_skin", "9"]);
    let json: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(json["ok"], false);
    assert_eq!(o.status.code(), Some(1));
}

#[test]
fn des_erreurs_de_lint_deja_presentes_ne_bloquent_pas_un_reglage_sain() {
    let w = Work::new("preexistant");
    // Le contenu est déjà rouge avant le réglage (commit de départ compris).
    let broken = w
        .read(PLAYER)
        .replacen("starting_skin: \"1\"", "starting_skin: \"9\"", 1);
    w.write(PLAYER, &broken);
    let o = w.alacod(&["set", "Camera", "camera", "lerp_speed", "10"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(err(&o).contains("déjà présente"), "{}", err(&o));
    assert!(w.read(CAMERA).contains("lerp_speed: 10.0,"));
}

#[test]
fn les_chemins_inconnus_ou_ambigus_listent_les_candidats() {
    let w = Work::new("candidats");
    let o = w.alacod(&["set", "Character", "player", "movement.max_spead", "1"]);
    assert_eq!(o.status.code(), Some(1));
    let message = err(&o);
    assert!(
        message.contains("introuvable") && message.contains("movement.max_speed"),
        "{message}"
    );

    // Deux modes de tir portent la même variante dans une liste : l'indice tranche.
    w.write(
        "assets/economy/economy.ron",
        "(\n    kill_points: 60,\n    hit_points: 10,\n)\n",
    );
    let o = w.alacod(&["get", "Economy", "economy", "kill_points"]);
    assert_eq!(out(&o).trim(), "60");

    // Un id présent dans deux fichiers du même kind est ambigu.
    let player = w.read(PLAYER);
    w.write("assets/characters/double.ron", &player);
    let o = w.alacod(&["get", "Character", "player", "movement.max_speed"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("ambigu"), "{}", err(&o));
}

#[test]
fn raw_remplace_le_noeud_tel_quel() {
    let w = Work::new("raw");
    let o = w.alacod(&[
        "set",
        "--raw",
        "Character",
        "player",
        "base_health.regen_rate",
        "None",
    ]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(w.read(PLAYER).contains("regen_rate: None,"));
    // Sans --raw, `None` n'a pas de type à reprendre.
    let o = w.alacod(&["set", "Character", "player", "base_health.regen_rate", "5"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("--raw"), "{}", err(&o));
    let o = w.alacod(&[
        "set",
        "--raw",
        "Character",
        "player",
        "base_health.regen_rate",
        "Some(\"5.0\")",
    ]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(w.read(PLAYER).contains("regen_rate: Some(\"5.0\"),"));
    // Un RON mal formé est refusé avant toute écriture.
    let before = w.read(PLAYER);
    let o = w.alacod(&[
        "set",
        "--raw",
        "Character",
        "player",
        "base_health.regen_rate",
        "Some((",
    ]);
    assert_eq!(o.status.code(), Some(1));
    assert_eq!(w.read(PLAYER), before);
}

#[test]
fn commit_ne_prend_que_le_fichier_touche() {
    let w = Work::new("commit");
    // Du travail en cours : un autre fichier modifié, et un fichier nouveau indexé.
    w.write(CAMERA, &w.read(CAMERA).replacen("12.0", "13.0", 1));
    w.write("assets/notes.txt", "à moi");
    w.git(&["add", "assets/notes.txt"]);

    let o = w.alacod(&[
        "set",
        "--commit",
        "Character",
        "player",
        "movement.max_speed",
        "120",
    ]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).contains("; commit "), "{}", out(&o));

    assert_eq!(
        w.git(&["log", "-1", "--format=%s"]),
        "editor: Character player movement.max_speed 150.0 -> 120.0"
    );
    assert_eq!(
        w.git(&["show", "--name-only", "--format=", "HEAD"]),
        "assets/ZombieShooter/Sprites/Character/player_config.ron"
    );
    // Le reste est exactement comme avant : modifié non indexé, indexé non commité.
    let status = w.git(&["status", "--porcelain"]);
    assert!(status.contains(" M assets/camera.ron"), "{status}");
    assert!(status.contains("A  assets/notes.txt"), "{status}");
    assert_eq!(status.lines().count(), 2, "{status}");
}

#[test]
fn commit_refuse_un_fichier_deja_modifie() {
    let w = Work::new("commit-sale");
    w.write(
        PLAYER,
        &w.read(PLAYER)
            .replacen("// Double speed", "// Double vitesse", 1),
    );
    let dirty = w.read(PLAYER);
    let head = w.git(&["rev-parse", "HEAD"]);
    let o = w.alacod(&[
        "set",
        "--commit",
        "Character",
        "player",
        "movement.max_speed",
        "120",
    ]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("changements non commités"), "{}", err(&o));
    assert_eq!(w.read(PLAYER), dirty, "rien n'est écrit");
    assert_eq!(w.git(&["rev-parse", "HEAD"]), head);
}

#[test]
fn undo_annule_un_commit_editor_et_lui_seul() {
    let w = Work::new("undo");
    let original = w.read(PLAYER);
    let o = w.alacod(&[
        "set",
        "--commit",
        "Character",
        "player",
        "movement.max_speed",
        "100",
    ]);
    assert!(o.status.success(), "{}", err(&o));
    assert_ne!(w.read(PLAYER), original);

    let o = w.alacod(&["undo"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(
        out(&o).contains("annulé : editor: Character player movement.max_speed 150.0 -> 100.0"),
        "{}",
        out(&o)
    );
    assert!(out(&o).contains("lint : aucune erreur"), "{}", out(&o));
    assert_eq!(w.read(PLAYER), original);
    assert!(w.git(&["log", "-1", "--format=%s"]).starts_with("Revert"));

    // Le dernier commit n'est plus un `editor:` : rien à annuler, rien de touché.
    let head = w.git(&["rev-parse", "HEAD"]);
    let o = w.alacod(&["undo"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("pas un commit `editor:`"), "{}", err(&o));
    assert_eq!(w.git(&["rev-parse", "HEAD"]), head);

    // Un commit ordinaire n'est jamais annulé non plus.
    w.write("assets/notes.txt", "x");
    w.git(&["add", "assets/notes.txt"]);
    w.git(&["commit", "-q", "-m", "travail de William"]);
    let o = w.alacod(&["undo"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(w.root.join("assets/notes.txt").exists());
}

#[test]
fn undo_garde_le_travail_en_cours_des_autres_fichiers() {
    let w = Work::new("undo-sale");
    let o = w.alacod(&[
        "set",
        "--commit",
        "Character",
        "player",
        "movement.max_speed",
        "100",
    ]);
    assert!(o.status.success(), "{}", err(&o));
    w.write(CAMERA, &w.read(CAMERA).replacen("12.0", "13.0", 1));
    w.write("assets/notes.txt", "à moi");
    w.git(&["add", "assets/notes.txt"]);
    let o = w.alacod(&["undo"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(w.read(PLAYER).contains("max_speed: \"150.0\""));
    assert!(w.read(CAMERA).contains("13.0"));
    assert!(w.root.join("assets/notes.txt").exists());
}

#[test]
fn undo_refuse_si_le_fichier_du_commit_a_des_changements_non_commites() {
    let w = Work::new("undo-fichier-sale");
    let o = w.alacod(&[
        "set",
        "--commit",
        "Character",
        "player",
        "movement.max_speed",
        "100",
    ]);
    assert!(o.status.success(), "{}", err(&o));
    w.write(
        PLAYER,
        &w.read(PLAYER)
            .replacen("// Double speed", "// Double vitesse", 1),
    );
    let dirty = w.read(PLAYER);
    let head = w.git(&["rev-parse", "HEAD"]);
    let o = w.alacod(&["undo"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("changements non commités"), "{}", err(&o));
    assert_eq!(w.read(PLAYER), dirty);
    assert_eq!(w.git(&["rev-parse", "HEAD"]), head);
}
