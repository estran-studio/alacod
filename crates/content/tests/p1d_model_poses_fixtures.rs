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
                // Fixtures C-2 : la liste de règles ne porte que les phases à dessiner, pas un `Shoot` ni l'ordre de jeu
                && !m.contains("TakeCover")
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

/// X-6e rouge : un chef à règle `Command` réclame la pose `order`, un boss à `transition > 0` la pose `transition`.
#[test]
fn a_chief_without_order_and_transition_poses_is_refused() {
    let missing = missing_poses("model_poses_chef_missing");
    for field in ["order", "transition"] {
        assert!(
            missing
                .iter()
                .any(|m| m.contains(&format!("poses.{field}`"))),
            "pose `{field}` non réclamée : {missing:#?}"
        );
    }
    assert_eq!(missing.len(), 2, "{missing:#?}");
}

/// X-6e vert : l'ordre et la transition ont leur geste.
#[test]
fn a_chief_with_order_and_transition_poses_is_accepted() {
    assert_eq!(
        unexpected_errors("model_poses_chef_complete"),
        Vec::<String>::new()
    );
}

// C-2 (§ 131) : les phases de behavior (annonce et ruée de `Charge`, recul de `ReturnHome`, repli de `TakeCover`) ont une pose
// optionnelle ; un modèle qui n'en pose aucune est **averti** (jamais une erreur : le contenu existant continue de charger).

fn behavior_warnings(fixture: &str) -> Vec<String> {
    let (registry, manifest, _) = load_and_lint(&fixture_dir(fixture)).unwrap();
    content::warnings(&registry, &manifest)
        .into_iter()
        .filter(|w| w.kind == content::LintWarningKind::MissingBehaviorPose)
        .map(|w| w.message)
        .collect()
}

/// Rouge : un personnage dont les règles ont les quatre phases, dessiné par un modèle qui n'a que l'attente.
#[test]
fn a_model_without_behavior_poses_is_warned_not_rejected() {
    let warned = behavior_warnings("model_behavior_poses_missing");
    for field in ["charge_windup", "charge", "return_home", "take_cover"] {
        assert!(
            warned
                .iter()
                .any(|m| m.contains(&format!("poses.{field}`"))),
            "pose `{field}` non signalée : {warned:#?}"
        );
    }
    assert_eq!(warned.len(), 4, "{warned:#?}");
    assert!(
        !load_and_lint(&fixture_dir("model_behavior_poses_missing"))
            .unwrap()
            .2
            .iter()
            .any(|e| e.kind == LintErrorKind::MissingPose),
        "un avertissement ne devient jamais une erreur"
    );
}

#[test]
fn a_model_with_a_pose_per_behavior_phase_is_not_warned() {
    assert_eq!(
        behavior_warnings("model_behavior_poses_complete"),
        Vec::<String>::new()
    );
    assert_eq!(
        unexpected_errors("model_behavior_poses_complete"),
        Vec::<String>::new()
    );
}

/// Une pose de phase qui nomme un clip absent du manifeste reste une erreur, comme les autres poses.
#[test]
fn an_unknown_behavior_clip_is_an_error() {
    let errors = unexpected_errors("model_behavior_poses_unknown");
    assert!(
        errors
            .iter()
            .any(|m| m.contains("take_cover.still") && m.contains("inconnu")),
        "{errors:#?}"
    );
}
