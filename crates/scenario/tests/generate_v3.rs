//! Fonctions pures du générateur v3 (M3-T13, `docs/conventions.md` §78) : noms de fichiers stables,
//! attentes des gabarits `TearsOnTarget`, `FamiliarHeld`, `ChampionForced`. Rien n'est simulé.

use bevy_fixed::fixed_math::Fixed;
use effects::Action;
use game::gen_test::{ChampionTest, FamiliarTest, ItemTest, TearsHits, TearsTarget, TearsTest};
use game::replay::{EntityRef, Expectation};
use scenario::generate::{
    self, ChampionDeath, ChampionShot, EnemyArena, EnemyProbe, TargetInfo, TearsRun, WeaponArena,
};
use sim_core::stats::StatId;

fn item_def(src: &str) -> items::ItemDef {
    ron::from_str(src).unwrap()
}

fn item_test(frames: u32) -> ItemTest {
    ItemTest {
        frames,
        expect: vec![],
        tears: None,
        familiar: None,
    }
}

fn arena() -> WeaponArena {
    WeaponArena::testbed()
}

fn target() -> TargetInfo {
    TargetInfo {
        net_id: 35,
        x: Fixed::from_num(552),
        y: Fixed::from_num(744),
    }
}

#[test]
fn noms_de_fichiers_stables_et_distincts() {
    assert_eq!(generate::tears_file_name("lentille"), "tears_lentille.ron");
    assert_eq!(generate::familiar_file_name("trio"), "familiar_trio.ron");
    assert_eq!(
        generate::tears_control_file_name("lentille"),
        "tears_lentille_sans_objet.ron"
    );
    assert_eq!(
        generate::familiar_control_file_name("compagnon"),
        "familiar_compagnon_sans_objet.ron"
    );
    assert_eq!(
        generate::champion_file_name("elite_divise", "divise"),
        "champion_elite_divise_divise.ron"
    );
    // Deux variantes d'un même personnage, deux fichiers ; un objet à `tears` et à `familiar`, deux fichiers.
    assert_ne!(
        generate::champion_file_name("c", "a"),
        generate::champion_file_name("c", "b")
    );
    assert_ne!(
        generate::tears_file_name("x"),
        generate::familiar_file_name("x")
    );
    // Aucun nom ne recoupe ceux des gabarits existants (`item_`, `weapon_`, `enemy_`, `room_`, `boss_`).
    for name in [
        generate::tears_file_name("x"),
        generate::familiar_file_name("x"),
        generate::champion_file_name("x", "y"),
    ] {
        assert!(!["item_", "weapon_", "enemy_", "room_", "boss_"]
            .iter()
            .any(|prefix| name.starts_with(prefix)));
    }
}

fn tears(frames: Option<u32>) -> TearsTest {
    TearsTest {
        frames,
        weapon: Some("larme".into()),
        aim: None,
        targets: vec![TearsTarget {
            character: "target".into(),
            dx: -48,
            dy: -18,
        }],
        hits: vec![TearsHits {
            target: 0,
            with_item: 5,
            without_item: 0,
        }],
        expect: vec![Expectation::BulletCount {
            count: 2,
            projectile: Some("larme".into()),
            team: None,
            at_frame: 149,
        }],
        control: vec![Expectation::BulletCount {
            count: 0,
            projectile: Some("larme".into()),
            team: None,
            at_frame: 149,
        }],
    }
}

fn run(control: bool, anchor: Option<TargetInfo>, stats: &[(StatId, f32)]) -> TearsRun<'_> {
    TearsRun {
        stats,
        anchor,
        net_ids: &[35, 37],
        control,
    }
}

#[test]
fn tears_avec_objet_et_temoin_sans_objet() {
    let def = item_def(
        r#"(name: "L", kind: Passive, modifiers: [(stat: ShotSpeed, op: Pct, value: "0.5")])"#,
    );
    let stats = [(StatId::ShotSpeed, 1.5)];
    let with = generate::build_tears_scenario(
        &arena(),
        "lentille",
        &def,
        &item_test(60),
        &tears(Some(150)),
        &run(false, Some(target()), &stats),
    );
    assert_eq!(with.frames, 150, "tears.frames l'emporte sur test.frames");
    assert_eq!(with.players[0].weapon.as_deref(), Some("larme"));
    assert_eq!(with.players[0].items, vec!["lentille".to_string()]);
    assert!(!with.players[0].inputs.is_empty(), "le joueur tire");
    assert!(
        matches!(&with.expect[0], Expectation::HasItem { id, at_frame: 1, .. } if id == "lentille")
    );
    assert!(with.expect.iter().any(
        |e| matches!(e, Expectation::Stat { stat: StatId::ShotSpeed, value, .. } if *value == 1.5)
    ));
    assert!(with.expect.iter().any(|e| matches!(
        e,
        Expectation::EntityHits {
            net_id: 35,
            min: 5,
            max: None,
            at_frame: 150
        }
    )));
    assert!(matches!(
        with.expect.last(),
        Some(Expectation::BulletCount { count: 2, .. })
    ));

    // Le témoin : même tir, même placement, sans l'objet, attentes contraires.
    let without = generate::build_tears_scenario(
        &arena(),
        "lentille",
        &def,
        &item_test(60),
        &tears(Some(150)),
        &run(true, Some(target()), &stats),
    );
    assert!(without.players[0].items.is_empty());
    assert_eq!(without.players[0].inputs, with.players[0].inputs);
    assert_eq!(without.characters, with.characters);
    assert_eq!(without.frames, with.frames);
    assert!(!without
        .expect
        .iter()
        .any(|e| matches!(e, Expectation::HasItem { .. } | Expectation::Stat { .. })));
    assert!(without.expect.iter().any(|e| matches!(
        e,
        Expectation::EntityHits {
            net_id: 35,
            min: 0,
            max: Some(0),
            at_frame: 150
        }
    )));
    assert!(matches!(
        without.expect.last(),
        Some(Expectation::BulletCount { count: 0, .. })
    ));

    // Sans `frames` propre : celle de `test:`.
    assert_eq!(
        generate::build_tears_scenario(
            &arena(),
            "lentille",
            &def,
            &item_test(90),
            &tears(None),
            &run(false, None, &[]),
        )
        .frames,
        90
    );
}

#[test]
fn tears_cibles_posees_relativement_a_l_arene() {
    let def = item_def(r#"(name: "P", kind: Passive)"#);
    let s = generate::build_tears_scenario(
        &arena(),
        "p",
        &def,
        &item_test(100),
        &tears(None),
        &run(false, Some(target()), &[]),
    );
    assert_eq!(s.characters.len(), 1);
    let placed = &s.characters[0];
    assert_eq!(placed.character, "target");
    assert_eq!(placed.x, Fixed::from_num(552 - 48));
    assert_eq!(placed.y, Fixed::from_num(744 - 18));
    let positions = generate::tears_positions(target(), &tears(None).targets);
    assert_eq!(
        positions,
        vec![
            (Fixed::from_num(552), Fixed::from_num(744)),
            (placed.x, placed.y)
        ]
    );
}

#[test]
fn familiar_compte_per_copy_et_temoin_sans_objet() {
    let silent = FamiliarTest::default();
    let s = generate::build_familiar_scenario(
        &arena(),
        "trio",
        &item_test(60),
        &silent,
        3,
        None,
        false,
    );
    assert_eq!(s.frames, 60);
    assert!(
        s.players[0].inputs.is_empty(),
        "min_hits 0 : le joueur ne tire pas"
    );
    assert!(!s
        .expect
        .iter()
        .any(|e| matches!(e, Expectation::EntityHits { .. })));
    let counts: Vec<_> = s
        .expect
        .iter()
        .filter_map(|e| match e {
            Expectation::FamiliarCount {
                count, at_frame, ..
            } => Some((*count, *at_frame)),
            _ => None,
        })
        .collect();
    assert_eq!(counts, vec![(3, generate::FAMILIAR_CHECK_FRAME), (3, 59)]);

    let shooting = FamiliarTest {
        frames: Some(120),
        weapon: Some("larme".into()),
        min_hits: 9,
        expect: vec![],
    };
    let with = generate::build_familiar_scenario(
        &arena(),
        "compagnon",
        &item_test(60),
        &shooting,
        1,
        Some(35),
        false,
    );
    assert_eq!(with.frames, 120);
    assert!(!with.players[0].inputs.is_empty());
    assert!(with.expect.iter().any(|e| matches!(
        e,
        Expectation::EntityHits {
            net_id: 35,
            min: 9,
            max: None,
            at_frame: 120
        }
    )));
    let without = generate::build_familiar_scenario(
        &arena(),
        "compagnon",
        &item_test(60),
        &shooting,
        1,
        Some(35),
        true,
    );
    assert!(without.players[0].items.is_empty());
    assert_eq!(without.players[0].inputs, with.players[0].inputs);
    assert!(without.expect.iter().any(|e| matches!(
        e,
        Expectation::EntityHits {
            net_id: 35,
            min: 0,
            max: Some(8),
            at_frame: 120
        }
    )));
    assert!(without
        .expect
        .iter()
        .any(|e| matches!(e, Expectation::FamiliarCount { count: 0, .. })));
}

fn death() -> ChampionDeath {
    ChampionDeath {
        frame: 99,
        enemies_before: 6,
        enemy_bullets_before: 0,
        enemy_bullets_after: 8,
        items: vec!["boussole".into()],
    }
}

fn explode() -> Action {
    Action::Explode {
        radius: Fixed::from_num(70),
        damage: Fixed::from_num(10),
        destroys: false,
    }
}

fn split(count: u32) -> Action {
    Action::SpawnCharacters {
        id: "petit".into(),
        count,
    }
}

fn pattern() -> Action {
    Action::SpawnPattern {
        pattern: "couronne".into(),
        weapon: "fireball_gun".into(),
    }
}

#[test]
fn mort_attendue_et_chaque_effet_traduit() {
    use game::replay::EntityKind;
    let enemies = |count: u32, at_frame: u32| Expectation::EntityCount {
        kind: EntityKind::Enemy,
        min: Some(count),
        max: Some(count),
        at_frame,
    };
    // Explosion : la mort (6 avant, 5 après) puis le moment clé.
    let expect = generate::champion_death_expectations(&[explode()], false, &death(), 150).unwrap();
    assert_eq!(expect[0], enemies(6, 98));
    assert_eq!(expect[1], enemies(5, 100));
    assert!(
        matches!(&expect[2], Expectation::Event { kind, by_frame: 150, .. } if kind == "explosion")
    );
    assert_eq!(expect.len(), 3);
    // Division : 6 - 1 + 2 + 1 = 8.
    let expect =
        generate::champion_death_expectations(&[split(2), split(1)], false, &death(), 150).unwrap();
    assert_eq!(expect[1], enemies(8, 100));
    // Motif : les balles ennemies mesurées avant et après.
    let expect = generate::champion_death_expectations(&[pattern()], false, &death(), 150).unwrap();
    assert!(expect.iter().any(|e| matches!(
        e,
        Expectation::BulletCount {
            count: 8,
            team: Some(sim_core::team::Team::Enemies),
            at_frame: 100,
            ..
        }
    )));
    assert!(expect.iter().any(|e| matches!(
        e,
        Expectation::BulletCount {
            count: 0,
            at_frame: 98,
            ..
        }
    )));
    // Butin : l'objet ramassé à la dernière frame.
    let expect = generate::champion_death_expectations(&[], true, &death(), 200).unwrap();
    assert!(matches!(
        expect.last(),
        Some(Expectation::HasItem { id, at_frame: 199, .. }) if id == "boussole"
    ));
}

#[test]
fn mort_sans_trace_mesurable_est_refusee() {
    let silent = ChampionDeath {
        enemy_bullets_after: 0,
        items: vec![],
        ..death()
    };
    // Aucun effet de mort traduisible.
    assert!(
        generate::champion_death_expectations(&[], false, &silent, 150)
            .unwrap_err()
            .contains("sans effet de mort traduisible")
    );
    assert!(
        generate::champion_death_expectations(&[Action::GainShield(1)], false, &silent, 150)
            .is_err()
    );
    // Un motif qui n'a produit aucune balle, un butin non ramassé : refusés, jamais ignorés.
    assert!(
        generate::champion_death_expectations(&[pattern()], false, &silent, 150)
            .unwrap_err()
            .contains("aucune balle ennemie")
    );
    assert!(
        generate::champion_death_expectations(&[], true, &silent, 150)
            .unwrap_err()
            .contains("aucun objet")
    );
}

#[test]
fn champion_variante_imposee_et_tir_dans_l_axe() {
    let enemy_arena = EnemyArena {
        game: "testbed".into(),
        map: "testbed/arena.ldtk".into(),
        waves: false,
        mode: None,
    };
    let probe = EnemyProbe {
        player_x: Fixed::from_num(424),
        player_y: Fixed::from_num(696),
    };
    let quiet = ChampionTest {
        variant: "nu".into(),
        kill: false,
        weapon: None,
        expect: vec![],
    };
    let s = generate::build_champion_scenario(
        &enemy_arena,
        "elite",
        150,
        &quiet,
        &probe,
        ChampionShot::None,
        &[],
    );
    assert_eq!(s.characters[0].variant.as_deref(), Some("nu"));
    assert_eq!(
        s.characters[0].x,
        Fixed::from_num(424 + generate::ENEMY_OFFSET_X)
    );
    assert!(s.players[0].inputs.is_empty());
    assert_eq!(
        s.expect.len(),
        1,
        "sans `kill`, ni tir ni effet de mort attendu : {:?}",
        s.expect
    );
    assert!(matches!(
        &s.expect[0],
        Expectation::EnemyVariant { entity: EntityRef::Placed(0), variant: Some(v), at_frame }
            if v == "nu" && *at_frame == generate::ENEMY_AT_FRAME + 1
    ));

    let kill = ChampionTest {
        variant: "riche".into(),
        kill: true,
        weapon: Some("larme".into()),
        expect: vec![],
    };
    let death_expect = generate::champion_death_expectations(&[], true, &death(), 200).unwrap();
    let s = generate::build_champion_scenario(
        &enemy_arena,
        "elite",
        200,
        &kill,
        &probe,
        ChampionShot::UntilDeath {
            death_frame: 99,
            walk: true,
        },
        &death_expect,
    );
    assert_eq!(s.players[0].weapon.as_deref(), Some("larme"));
    // Tire jusqu'à la mort (frame 99 incluse), puis marche vers +x et appuie sur Interaction.
    use game::replay::Button;
    let fire = |segment: &&game::replay::Segment| segment.buttons == vec![Button::Fire];
    assert!(s.players[0]
        .inputs
        .iter()
        .filter(fire)
        .all(|segment| segment.to <= 100));
    assert!(s.players[0]
        .inputs
        .iter()
        .any(|segment| segment.buttons == vec![Button::Right] && segment.from == 100));
    assert!(s.players[0]
        .inputs
        .iter()
        .any(|segment| segment.buttons == vec![Button::Interaction]));
    assert!(s
        .expect
        .iter()
        .any(|e| matches!(e, Expectation::HasItem { .. })));
    // Sonde : tire jusqu'au bout.
    let probe_run = generate::build_champion_scenario(
        &enemy_arena,
        "elite",
        150,
        &kill,
        &probe,
        ChampionShot::ToEnd,
        &[],
    );
    assert!(probe_run.players[0]
        .inputs
        .iter()
        .any(|segment| segment.to > 140));
}
