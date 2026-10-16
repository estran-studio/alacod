//! Fonctions pures du générateur v3 (M3-T13, `docs/conventions.md` §78) : noms de fichiers stables,
//! attentes des gabarits `TearsOnTarget`, `FamiliarHeld`, `ChampionForced`. Rien n'est simulé.

use bevy_fixed::fixed_math::Fixed;
use effects::Action;
use game::gen_test::{ChampionTest, FamiliarTest, ItemTest, TearsTest};
use game::replay::{EntityRef, Expectation};
use scenario::generate::{self, EnemyArena, EnemyProbe, TargetInfo, WeaponArena};
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

#[test]
fn tears_cible_de_l_arene() {
    let def = item_def(
        r#"(name: "L", kind: Passive, modifiers: [(stat: ShotSpeed, op: Pct, value: "0.5")])"#,
    );
    let tears = TearsTest {
        frames: Some(150),
        weapon: Some("larme".into()),
        target: None,
        min_hits: 2,
        expect: vec![Expectation::BulletCount {
            count: 0,
            projectile: None,
            team: None,
            at_frame: 149,
        }],
    };
    let stats = [(StatId::ShotSpeed, 1.5)];
    let s = generate::build_tears_scenario(
        &arena(),
        "lentille",
        &def,
        &item_test(60),
        &tears,
        &stats,
        Some(target()),
    );
    assert_eq!(s.frames, 150, "tears.frames l'emporte sur test.frames");
    assert_eq!(s.players[0].weapon.as_deref(), Some("larme"));
    assert_eq!(s.players[0].items, vec!["lentille".to_string()]);
    assert!(!s.players[0].inputs.is_empty(), "le joueur tire");
    assert!(s.characters.is_empty());
    assert!(
        matches!(&s.expect[0], Expectation::HasItem { id, at_frame: 1, .. } if id == "lentille")
    );
    assert!(s.expect.iter().any(
        |e| matches!(e, Expectation::Stat { stat: StatId::ShotSpeed, value, .. } if *value == 1.5)
    ));
    assert!(s.expect.iter().any(|e| matches!(
        e,
        Expectation::EntityHits {
            net_id: 35,
            min: 2,
            max: None,
            at_frame: 150
        }
    )));
    assert!(matches!(
        s.expect.last(),
        Some(Expectation::BulletCount { at_frame: 149, .. })
    ));
    // Sans `frames` propre : celle de `test:`.
    let tears = TearsTest {
        frames: None,
        expect: vec![],
        ..tears
    };
    assert_eq!(
        generate::build_tears_scenario(
            &arena(),
            "lentille",
            &def,
            &item_test(90),
            &tears,
            &[],
            None
        )
        .frames,
        90
    );
}

#[test]
fn tears_cible_de_remplacement_sur_la_ligne_de_tir() {
    let def = item_def(r#"(name: "P", kind: Passive)"#);
    let tears = TearsTest {
        frames: None,
        weapon: None,
        target: Some("target".into()),
        min_hits: 1,
        expect: vec![],
    };
    let s = generate::build_tears_scenario(
        &arena(),
        "p",
        &def,
        &item_test(100),
        &tears,
        &[],
        Some(target()),
    );
    assert_eq!(s.characters.len(), 1);
    let placed = &s.characters[0];
    assert_eq!(placed.character, "target");
    // 48 unités vers le joueur en x, 18 en y : sur la droite de visée (128, 48).
    assert_eq!(placed.x, Fixed::from_num(552 - 48));
    assert_eq!(placed.y, Fixed::from_num(744 - 18));
    assert!(s.expect.iter().any(|e| matches!(
        e,
        Expectation::HitsAtLeast {
            entity: EntityRef::Placed(0),
            hits: 1,
            at_frame: 100
        }
    )));
    assert!(!s
        .expect
        .iter()
        .any(|e| matches!(e, Expectation::EntityHits { .. })));
}

#[test]
fn familiar_compte_per_copy_et_coups_seulement_si_demandes() {
    let silent = FamiliarTest::default();
    let s = generate::build_familiar_scenario(&arena(), "trio", &item_test(60), &silent, 3, None);
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
        min_hits: 2,
        expect: vec![],
    };
    let s = generate::build_familiar_scenario(
        &arena(),
        "compagnon",
        &item_test(60),
        &shooting,
        1,
        Some(35),
    );
    assert_eq!(s.frames, 120);
    assert!(!s.players[0].inputs.is_empty());
    assert!(s.expect.iter().any(|e| matches!(
        e,
        Expectation::EntityHits {
            net_id: 35,
            min: 2,
            at_frame: 120,
            ..
        }
    )));
}

#[test]
fn effets_de_mort_attendus() {
    use game::replay::EntityKind;
    let explode = Action::Explode {
        radius: Fixed::from_num(70),
        damage: Fixed::from_num(10),
        destroys: false,
    };
    let split = Action::SpawnCharacters {
        id: "petit".into(),
        count: 3,
    };
    assert!(generate::champion_death_expectations(&[], 100).is_empty());
    assert!(generate::champion_death_expectations(&[Action::GainShield(1)], 100).is_empty());
    let both = generate::champion_death_expectations(&[explode, split.clone(), split], 100);
    assert!(
        matches!(&both[0], Expectation::Event { kind, by_frame: 100, .. } if kind == "explosion")
    );
    assert!(matches!(
        &both[1],
        Expectation::EntityCount {
            kind: EntityKind::Enemy,
            min: Some(6),
            max: None,
            at_frame: 100
        }
    ));
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
    let split = Action::SpawnCharacters {
        id: "petit".into(),
        count: 2,
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
        &[split.clone()],
        &probe,
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
        variant: "divise".into(),
        kill: true,
        weapon: Some("larme".into()),
        expect: vec![],
    };
    let s = generate::build_champion_scenario(&enemy_arena, "elite", 150, &kill, &[split], &probe);
    assert_eq!(s.players[0].weapon.as_deref(), Some("larme"));
    assert!(s.players[0]
        .inputs
        .iter()
        .all(|segment| segment.pan == (127, 0)));
    assert!(s
        .expect
        .iter()
        .any(|e| matches!(e, Expectation::EntityCount { min: Some(2), .. })));
}
