//! SB-1 (`docs/conventions.md` §85) : les profils `sandbox/*.ron` livrés par les jeux **se jouent**.
//!
//! Chaque profil devient un `Scenario` (D1 : sous-ensemble de `Scenario`, mêmes types) joué soixante
//! frames en mode `Sandbox` sur l'arène résolue (`SandboxProfile::arena`) : invariants par défaut
//! verts, arme, solde et objets du joueur posés, personnages placés présents. Sans le profil, un
//! placement hors de l'arène ou une référence que le lint ne voit pas échouerait seulement à l'essai.

use bevy::prelude::*;
use game::character::config::CharacterConfigKey;
use game::replay::{Expectation, PlayerScript, Scenario};
use std::collections::BTreeMap;

const FRAMES: u32 = 60;
const CHECK_AT: u32 = FRAMES - 10;

fn scenario_of(
    game: &str,
    profile_id: &str,
) -> (
    Scenario,
    content::Registry,
    content::sandbox::SandboxProfile,
) {
    let (registry, manifest, errors) =
        content::load_and_lint(&scenario::runner::game_dir(game)).expect("manifeste");
    assert!(errors.is_empty(), "{game} : {errors:#?}");
    let profile = registry.sandbox_profiles[profile_id].profile.clone();
    let map = profile.arena(&manifest).expect("arène").to_string();
    let mut scenario = Scenario::from_ron(&format!(
        "Scenario(game: {game:?}, map: {map:?}, frames: {FRAMES}, players: [], mode: Sandbox)"
    ))
    .expect("scénario de base");
    scenario.players = profile
        .players
        .iter()
        .cloned()
        .map(PlayerScript::from)
        .collect();
    if scenario.players.is_empty() {
        scenario.players.push(PlayerScript::default());
    }
    scenario.characters = profile.characters.clone();
    scenario.items = profile.items.clone();
    scenario.chests = profile.chests.clone();
    scenario.shop = profile.shop.clone();
    (scenario, registry, profile)
}

/// Attentes déduites du profil : arme active, solde exact, exemplaires de chaque objet tenu.
fn expectations(
    registry: &content::Registry,
    profile: &content::sandbox::SandboxProfile,
) -> Vec<Expectation> {
    let mut expect = Vec::new();
    for (handle, player) in profile.players.iter().enumerate() {
        if let Some(weapon) = &player.weapon {
            expect.push(Expectation::ActiveWeapon {
                handle,
                weapon: weapon.clone(),
                mode: None,
                at_frame: CHECK_AT,
            });
        }
        if let Some(currency) = player.currency {
            expect.push(Expectation::Currency {
                handle,
                min: Some(currency),
                max: Some(currency),
                at_frame: CHECK_AT,
            });
        }
        let mut held: BTreeMap<&str, u32> = BTreeMap::new();
        for item in &player.items {
            *held.entry(item.as_str()).or_default() += 1;
        }
        for (id, count) in held {
            // Un consommable se compte à part des passifs et de l'actif (`Inventory::consumables`).
            let consumable = matches!(registry.items[id].def.kind, items::ItemKind::Consumable);
            let id = id.to_string();
            expect.push(if consumable {
                Expectation::Consumable {
                    handle,
                    id,
                    count,
                    at_frame: CHECK_AT,
                }
            } else {
                Expectation::ItemCount {
                    handle,
                    id,
                    count,
                    at_frame: CHECK_AT,
                }
            });
        }
    }
    expect
}

fn play_profile(game: &str, profile_id: &str) {
    let (mut scenario, registry, profile) = scenario_of(game, profile_id);
    scenario.expect = expectations(&registry, &profile);
    let outcome = scenario::run(&scenario);
    assert!(
        outcome.failures.is_empty(),
        "{game}/{profile_id} : {:#?}",
        outcome.failures
    );

    let mut app = scenario::runner::run_until(&scenario, CHECK_AT);
    let world = app.world_mut();
    let mut present: BTreeMap<String, usize> = BTreeMap::new();
    for key in world.query::<&CharacterConfigKey>().iter(world) {
        *present.entry(key.0.clone()).or_default() += 1;
    }
    let mut placed: BTreeMap<&str, usize> = BTreeMap::new();
    for character in &profile.characters {
        *placed.entry(character.character.as_str()).or_default() += 1;
        assert!(
            character.at_frame < CHECK_AT,
            "{game}/{profile_id} : placement {character:?}"
        );
    }
    for (id, count) in placed {
        assert!(
            present.get(id).copied().unwrap_or(0) >= count,
            "{game}/{profile_id} : {count} « {id} » posé(s), présents : {present:?}"
        );
    }
}

macro_rules! profile_test {
    ($name:ident, $game:expr, $profile:expr) => {
        #[test]
        fn $name() {
            if map_ldtk::RENDER_ENABLED {
                return;
            }
            play_profile($game, $profile);
        }
    };
}

profile_test!(zombies_arene, "zombies", "arene");
profile_test!(zombies_lourd, "zombies", "lourd");
profile_test!(gungeon_arene, "gungeon", "arene");
profile_test!(gungeon_lourd, "gungeon", "lourd");
profile_test!(isaac_arene, "isaac", "arene");
profile_test!(isaac_lourd, "isaac", "lourd");
profile_test!(throne_arene, "throne", "arene");
profile_test!(throne_lourd, "throne", "lourd");
profile_test!(testbed_arene, "testbed", "arene");
profile_test!(testbed_lourd, "testbed", "lourd");
