//! D130 (`docs/conventions.md` §92) : un don à chaud (`alacod/give`) est **enregistré** par l'`InputRecorder`
//! (`alacod/save`) et **rejoué** par `Scenario::gifts` dans `GgrsSchedule`, donc en synctest sans désynchronisation.

use game::{
    grant::Gift,
    recording::{InputRecorder, RecordedSettings},
    replay::{Expectation, GiftPlacement, Scenario},
};
use scenario::generate::{self, EnemyArena, EnemyProbe, Template};

const AT: u32 = 20;

/// Scénario du testbed sans placement ni attente : un joueur immobile, assez de frames pour voir un don.
fn base() -> Scenario {
    let character_test = game::character::config::CharacterTest {
        frames: 600,
        still: true,
        moving: true,
        expect_still: vec![],
        expect_moving: vec![],
        boss: None,
        champions: vec![],
    };
    let mut scenario = generate::build_enemy_scenario(
        &EnemyArena {
            game: "testbed".into(),
            map: "testbed/arena.ldtk".into(),
            waves: false,
            mode: None,
        },
        "grunt_plain",
        Template::EnemyVsStillPlayer,
        &character_test,
        &EnemyProbe::default(),
    );
    scenario.characters.clear();
    scenario.frames = 60;
    scenario.expect.clear();
    scenario
}

/// Le don enregistré par l'enregistreur, comme `alacod/give` le fait : `record_gift` puis `to_scenario`.
fn recorded() -> Scenario {
    let mut recorder = InputRecorder::new(RecordedSettings::from_scenario(&base()));
    recorder.record_gift(
        0,
        &Gift {
            weapon: Some("shotgun".into()),
            set_currency: Some(777),
            ..Default::default()
        },
        AT,
    );
    let mut scenario = recorder.to_scenario(None);
    // Aucune frame d'input n'est enregistrée ici (`to_scenario` déduit les joueurs des inputs) : on
    // rétablit les joueurs, la carte et la durée du scénario de base, sinon rien ne joue.
    scenario.players = base().players;
    assert!(
        !scenario.players.is_empty(),
        "le scénario de base a un joueur"
    );
    scenario.map = base().map;
    scenario.frames = 60;
    scenario
}

#[test]
fn le_don_enregistre_est_dans_le_scenario() {
    assert_eq!(
        recorded().gifts,
        [GiftPlacement {
            handle: 0,
            weapon: Some("shotgun".into()),
            item: None,
            currency: None,
            set_currency: Some(777),
            at_frame: AT,
        }]
    );
}

#[test]
fn le_don_rejoue_prend_effet_a_sa_frame_et_survit_au_synctest() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let mut scenario = recorded();
    scenario.expect = vec![
        // Avant sa frame, rien n'a changé ; après, l'arme est active et le solde imposé.
        Expectation::Currency {
            handle: 0,
            min: Some(777),
            max: Some(777),
            at_frame: AT + 5,
        },
        Expectation::ActiveWeapon {
            handle: 0,
            weapon: "shotgun".into(),
            mode: None,
            at_frame: AT + 5,
        },
    ];
    let outcome = scenario::run(&scenario);
    assert!(outcome.failures.is_empty(), "{:#?}", outcome.failures);

    // Avant `at_frame` : le même solde n'est pas encore posé (l'attente échoue, preuve que ce n'est pas le défaut).
    scenario.expect = vec![Expectation::Currency {
        handle: 0,
        min: Some(777),
        max: Some(777),
        at_frame: AT - 5,
    }];
    assert!(!scenario::run(&scenario).failures.is_empty());
}
