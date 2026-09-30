//! Acceptation du générateur (T2.10, `docs/taches.md` §T2.10) : un objet sans `test:` passe
//! (couvert par `tests/scenarios/generated/zombies/weapon_{axe,bare_hands,knife,zombie_claws}.ron`,
//! joués par `crates/scenario/tests/scenarios.rs` comme n'importe quel scénario, invariants
//! seulement) ; un `test:` délibérément faux fait échouer le scénario généré, avec le message
//! attendu — vérifié ici sans jamais écrire dans `tests/scenarios/generated/` (scénario
//! construit en mémoire par [`scenario::generate::build_scenario`], la même fonction pure
//! qu'utilise `alacod-gen` avant d'écrire un fichier).

use game::weapons::WeaponTest;
use scenario::generate::{self, WeaponKind};

#[test]
fn false_weapon_test_fails_with_expected_message() {
    if map_ldtk::RENDER_ENABLED {
        return; // voir la même garde dans `tests/scenarios.rs`
    }

    // Sonde (test: None) : seulement pour retrouver le net_id de `target` dans le testbed,
    // indépendamment du `test:` réel de "pistol" dans le contenu de `games/zombies` (ce test
    // ne dépend pas de ce contenu, voir la doc du module).
    let probe = generate::build_scenario("pistol", WeaponKind::Ranged, None, 0);
    let target_net_id =
        generate::discover_target_net_id(&probe).expect("target introuvable dans le testbed");

    // `WeaponTest` fabriqué, jamais lu depuis un fichier `weapons.ron` : un `min_hits`
    // délibérément impossible à atteindre.
    let fake_test = WeaponTest {
        frames: 200,
        min_hits: 10_000,
        max_hits: None,
        expect: Vec::new(),
    };
    let scenario = generate::build_scenario(
        "pistol",
        WeaponKind::Ranged,
        Some(&fake_test),
        target_net_id,
    );

    let outcome = scenario::run(&scenario);

    assert!(
        !outcome.failures.is_empty(),
        "un test: délibérément faux (min_hits: 10 000, portée d'un pistolet sur 200 frames) \
         devrait faire échouer le scénario généré, mais il a réussi"
    );
    assert!(
        outcome
            .failures
            .iter()
            .any(|f| f.contains("coups reçus") && f.contains("< min 10000")),
        "message de failure inattendu (attendu une mention de « coups reçus ... < min 10000 ») : {:#?}",
        outcome.failures
    );

    // Ce test ne construit le scénario qu'en mémoire : rien n'a été écrit sous
    // `tests/scenarios/generated/` par cet appel (`alacod-gen`, pas `generate::build_scenario`,
    // est seul responsable d'écrire des fichiers).
    let would_be_written = std::path::PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/scenarios/generated"
    ))
    .join("zombies")
    .join("weapon_pistol.ron");
    if let Ok(existing) = std::fs::read_to_string(&would_be_written) {
        assert!(
            !existing.contains("10000") && !existing.contains("10_000"),
            "le `test:` fabriqué par ce test ne doit jamais atteindre le fichier généré par `alacod-gen`"
        );
    }
}
