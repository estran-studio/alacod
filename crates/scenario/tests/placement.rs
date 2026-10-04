//! Placement scripté de personnages (T1.13, `Scenario::characters`, `docs/conventions.md` §28)
//! et gabarits ennemis du générateur (`scenario::generate`, construits en mémoire : rien n'est
//! écrit sous `tests/scenarios/generated/`).

use bevy::prelude::*;
use bevy_fixed::fixed_math::Fixed;
use game::character::config::CharacterTest;
use game::character::enemy::Enemy;
use game::character::variant::Variant;
use game::replay::{CharacterPlacement, EntityRef, Expectation, Scenario};
use scenario::generate::{self, EnemyArena, EnemyProbe, Template, ENEMY_AT_FRAME};
use utils::net_id::GgrsNetId;

const AT: u32 = 5;

fn testbed_arena() -> EnemyArena {
    EnemyArena {
        game: "testbed".into(),
        map: "testbed/arena.ldtk".into(),
        waves: false,
    }
}

fn test_600() -> CharacterTest {
    CharacterTest {
        frames: 600,
        still: true,
        moving: true,
        expect_still: vec![],
        expect_moving: vec![],
    }
}

/// Scénario du testbed (gabarit immobile de `grunt_plain`) dont les placements sont remplacés
/// par deux personnages à la frame [`AT`] : `grunt_plain` puis `grunt` (variante `blinde`
/// imposée).
fn two_placements() -> Scenario {
    let mut scenario = generate::build_enemy_scenario(
        &testbed_arena(),
        "grunt_plain",
        Template::EnemyVsStillPlayer,
        &test_600(),
        &EnemyProbe::default(),
    );
    let probe = generate::discover_enemy_probe(&scenario).expect("joueur 0 dans l'arène");
    let at = |character: &str, dy: i32, variant: Option<&str>| CharacterPlacement {
        character: character.into(),
        x: probe.player_x + Fixed::from_num(200),
        y: probe.player_y + Fixed::from_num(dy),
        at_frame: AT,
        variant: variant.map(String::from),
        team: None,
    };
    scenario.characters = vec![at("grunt_plain", 0, None), at("grunt", 40, Some("blinde"))];
    scenario.frames = 60;
    scenario.expect = vec![];
    scenario
}

/// `(net id, variante)` des ennemis vivants, triés par net id.
fn enemies(app: &mut App) -> Vec<(usize, Option<String>)> {
    let world = app.world_mut();
    let mut found: Vec<_> = world
        .query_filtered::<(&GgrsNetId, Option<&Variant>), With<Enemy>>()
        .iter(world)
        .map(|(id, variant)| (id.0, variant.map(|v| v.0.clone())))
        .collect();
    found.sort();
    found
}

#[test]
fn placement_a_la_frame_exacte_dans_l_ordre_de_declaration() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let scenario = two_placements();
    let before = enemies(&mut scenario::runner::run_until(&scenario, AT));
    let after = enemies(&mut scenario::runner::run_until(&scenario, AT + 1));
    assert_eq!(
        after.len(),
        before.len() + 2,
        "avant {before:?}, après {after:?}"
    );
    let new: Vec<_> = after.iter().filter(|e| !before.contains(e)).collect();
    // Ordre de déclaration : `grunt_plain` (sans variante) reçoit le plus petit net id, puis
    // `grunt`, variante imposée `blinde`.
    assert_eq!(new[0].1, None);
    assert_eq!(new[1].1.as_deref(), Some("blinde"));
    assert!(new[0].0 < new[1].0);
}

#[test]
fn entity_ref_placed_et_variante_imposee() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let mut scenario = two_placements();
    scenario.expect = vec![
        Expectation::EnemyVariant {
            entity: EntityRef::Placed(0),
            variant: None,
            at_frame: AT + 2,
        },
        Expectation::EnemyVariant {
            entity: EntityRef::Placed(1),
            variant: Some("blinde".into()),
            at_frame: AT + 2,
        },
    ];
    let outcome = scenario::run(&scenario);
    assert!(outcome.failures.is_empty(), "{:#?}", outcome.failures);

    // Avant sa frame, le personnage placé n'existe pas : l'attente échoue.
    scenario.expect = vec![Expectation::EnemyVariant {
        entity: EntityRef::Placed(1),
        variant: Some("blinde".into()),
        at_frame: AT - 2,
    }];
    let outcome = scenario::run(&scenario);
    assert!(!outcome.failures.is_empty());
}

#[test]
fn gabarits_attentes_par_defaut_et_explicites() {
    let probe = EnemyProbe {
        player_x: Fixed::from_num(100),
        player_y: Fixed::from_num(-50),
    };
    let still = generate::build_enemy_scenario(
        &testbed_arena(),
        "grunt",
        Template::EnemyVsStillPlayer,
        &test_600(),
        &probe,
    );
    assert_eq!(still.frames, 600);
    assert!(still.players[0].inputs.is_empty());
    assert_eq!(still.players[0].weapon, None);
    assert_eq!(still.characters.len(), 1);
    assert_eq!(still.characters[0].character, "grunt");
    assert_eq!(still.characters[0].x, Fixed::from_num(300));
    assert_eq!(still.characters[0].y, Fixed::from_num(-50));
    assert_eq!(still.characters[0].at_frame, ENEMY_AT_FRAME);
    assert_eq!(
        still.expect,
        vec![
            Expectation::Event {
                kind: "hit".into(),
                label_contains: Some("joueur 0 touché".into()),
                by_frame: 600,
            },
            Expectation::EnemyNeverInWall {
                entity: EntityRef::Placed(0),
                from: ENEMY_AT_FRAME + 1,
                to: 600,
            },
        ]
    );
    assert_eq!(still.wave_overrides, None);

    let moving = generate::build_enemy_scenario(
        &testbed_arena(),
        "grunt",
        Template::EnemyVsMovingPlayer,
        &test_600(),
        &probe,
    );
    assert!(matches!(
        moving.expect[0],
        Expectation::PlayerAlive {
            handle: 0,
            at_frame: 600
        }
    ));
    // Carré : 90 frames par côté, de la frame 10 à la dernière.
    let inputs = &moving.players[0].inputs;
    assert_eq!((inputs[0].from, inputs[0].to), (10, 100));
    assert_eq!(inputs.last().unwrap().to, 600);

    // Attentes explicites : remplacent les attentes par défaut du gabarit concerné seulement ;
    // mode `Waves` : période de grâce au-delà de la durée.
    let mut test = test_600();
    test.expect_moving = vec![Expectation::PlayerDead {
        handle: 0,
        at_frame: 600,
    }];
    let arena = EnemyArena {
        waves: true,
        ..testbed_arena()
    };
    let moving = generate::build_enemy_scenario(
        &arena,
        "grunt",
        Template::EnemyVsMovingPlayer,
        &test,
        &probe,
    );
    assert_eq!(moving.expect, test.expect_moving);
    let still = generate::build_enemy_scenario(
        &arena,
        "grunt",
        Template::EnemyVsStillPlayer,
        &test,
        &probe,
    );
    assert_eq!(still.expect.len(), 2);
    assert_eq!(still.wave_overrides.unwrap().grace_period_frames, Some(601));
}

#[test]
fn placements_invalides_en_echec() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let mut scenario = two_placements();
    scenario.characters[0].character = "absent".into();
    scenario.characters[1].variant = Some("doree".into());
    scenario.characters.push(CharacterPlacement {
        at_frame: scenario.frames,
        ..scenario.characters[1].clone()
    });
    let outcome = scenario::run(&scenario);
    let has = |needle: &str| outcome.failures.iter().any(|f| f.contains(needle));
    assert!(
        has("personnage inconnu « absent »"),
        "{:#?}",
        outcome.failures
    );
    assert!(
        has("variante « doree » inconnue"),
        "{:#?}",
        outcome.failures
    );
    assert!(has("at_frame 60 >= frames 60"), "{:#?}", outcome.failures);
}
