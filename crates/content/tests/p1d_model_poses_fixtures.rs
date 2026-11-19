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

/// Vert : toutes les poses de l'état sont déclarées et résolues dans le manifeste.
#[test]
fn a_shooter_with_a_pose_per_state_is_accepted() {
    assert_eq!(missing_poses("model_poses_complete"), Vec::<String>::new());
}
