//! P-1d (`docs/conventions.md` § 122) : `alacod lint` exige une pose de `models.ron` pour chaque état que la simulation
//! d'un personnage à modèle peut prendre. Un tireur posté (règle `Shoot`, sans arme dans la simulation) qui n'a que
//! l'attente reste figé à l'écran : c'est le défaut observé sur les fantassins de 1837 le 16 novembre.

use content::{load_and_lint, LintErrorKind};
use std::path::PathBuf;

fn fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures")).join(name)
}

fn missing_poses(fixture: &str) -> Vec<String> {
    let (_, _, errors) = load_and_lint(&fixture_dir(fixture)).unwrap();
    errors
        .into_iter()
        .filter(|e| e.kind == LintErrorKind::MissingPose)
        .map(|e| e.message)
        .collect()
}

/// Rouge : le tireur n'a que `idle` ; chaque état de sa salve réclame sa pose.
#[test]
fn a_shooter_with_only_an_idle_pose_misses_every_state_of_its_cycle() {
    let missing = missing_poses("model_poses_missing");
    for field in ["shoulder", "aim", "shoot", "reload", "hit"] {
        assert!(
            missing
                .iter()
                .any(|m| m.contains(&format!("poses.{field}`"))),
            "pose `{field}` non réclamée : {missing:#?}"
        );
    }
    assert_eq!(missing.len(), 5, "{missing:#?}");
}

/// Toutes les erreurs du lint, sauf celles qu'un jeu minimal sans carte ni fichier d'armes porte par construction (l'arme
/// `absente` de la règle `Shoot` et des `weapons` de models.ron est inconnue, `start_map` ne charge rien) : les tests verts
/// échouent sur **toute** autre erreur, pas seulement sur `MissingPose`.
fn unexpected_errors(fixture: &str) -> Vec<String> {
    let (_, _, errors) = load_and_lint(&fixture_dir(fixture)).unwrap();
    errors
        .into_iter()
        .map(|e| e.message)
        .filter(|m| {
            !m.contains("arme inconnue « absente »")
                && !m.contains("weapons.absente : arme inconnue")
                && !m.contains("entry.start_map")
        })
        .collect()
}

/// Vert : toutes les poses de l'état sont déclarées et résolues dans le manifeste.
#[test]
fn a_shooter_with_a_pose_per_state_is_accepted() {
    assert_eq!(
        unexpected_errors("model_poses_complete"),
        Vec::<String>::new()
    );
}

/// Vert (revue P-1d, 3) : les gestes déclarés sur l'arme de la règle `Shoot` (`weapons` de models.ron) comptent, comme dans
/// `decide_poses`, qui résout les poses de l'arme par-dessus celles du modèle.
#[test]
fn a_shooter_whose_gestures_are_on_its_weapon_is_accepted() {
    assert_eq!(
        unexpected_errors("model_poses_on_weapon"),
        Vec::<String>::new()
    );
}

/// X-6d (§ 126) rouge : un agrippeur (règle `Grab`) qui n'a que l'attente ; le bond, l'accroche et l'étourdissement réclament
/// chacun leur pose (`leap`, `ride`, `stunned`).
#[test]
fn a_grabber_with_only_an_idle_pose_misses_its_leap_ride_and_daze() {
    let missing = missing_poses("model_poses_grab_missing");
    for field in ["leap", "ride", "stunned"] {
        assert!(
            missing
                .iter()
                .any(|m| m.contains(&format!("poses.{field}`"))),
            "pose `{field}` non réclamée : {missing:#?}"
        );
    }
    assert_eq!(missing.len(), 3, "{missing:#?}");
}

/// X-6d (§ 126) vert : les trois poses déclarées et résolues dans le manifeste.
#[test]
fn a_grabber_with_a_pose_per_state_is_accepted() {
    assert_eq!(
        unexpected_errors("model_poses_grab_complete"),
        Vec::<String>::new()
    );
}

/// X-6d (§ 126, suivi de revue) rouge : le `Grab` d'une phase de boss réclame les mêmes poses que celui du personnage.
#[test]
fn a_boss_phase_grab_misses_its_poses() {
    let missing = missing_poses("model_poses_grab_boss_missing");
    for field in ["leap", "ride", "stunned"] {
        assert!(
            missing
                .iter()
                .any(|m| m.contains(&format!("poses.{field}`"))),
            "pose `{field}` non réclamée : {missing:#?}"
        );
    }
}
