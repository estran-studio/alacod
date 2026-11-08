//! Bots (T2.11) : un run piloté par `InputSource::Bot` doit être rejouable — un run sauvé en
//! scénario (`--save-scenario` d'`alacod-sim`, ou tout enregistrement) puis rejoué en
//! `Scripted` (le mode d'un scénario ordinaire) donne exactement la même trace. Voir
//! `crates/game/src/recording.rs` (`InputRecorder::to_scenario`, qui ne recopie jamais `bot:` —
//! l'enregistrement capture le `BoxInput` déjà décidé, pas le profil).

use game::replay::{BotProfile, PlayerScript, Scenario};
use scenario::run;

fn two_fonceurs(frames: u32) -> Scenario {
    Scenario {
        game: "zombies".into(),
        map: "exemples/test_map.ldtk".into(),
        map_seed: 123456,
        frames,
        players: vec![
            PlayerScript {
                inputs: vec![],
                bot: Some(BotProfile::Fonceur),
                ..Default::default()
            },
            PlayerScript {
                inputs: vec![],
                bot: Some(BotProfile::Fonceur),
                ..Default::default()
            },
        ],
        expect: vec![],
        weapon_overrides: vec![],
        wave_overrides: None,
        invariants: Default::default(),
        powerups: vec![],
        items: vec![],
        chests: vec![],
        room_reward_chance_override: None,
        shop: Default::default(),
        profile: false,
        profile_setup: None,
        hub: None,
        shopping: false,
        explorer: None,
        powerup_drop_chance_override: None,
        floors: None,
        progression: None,
        clocks: None,
        difficulty: None,
        characters: vec![],
        mode: None,
        ui: Vec::new(),
    }
}

/// Un run court (2 bots, 300 frames, comme demandé par la tâche T2.11) rejoué depuis son propre
/// enregistrement donne la même trace : ce que l'enregistrement capture (le `BoxInput` déjà
/// décidé par `bots::decide`, pas le profil) suffit à reproduire la partie à l'identique.
#[test]
fn bot_run_replays_identically_once_recorded() {
    if map_ldtk::RENDER_ENABLED {
        eprintln!(
            "test ignoré : compilé avec le rendu des tilemaps (utiliser --no-default-features)"
        );
        return;
    }

    let scenario = two_fonceurs(300);
    let original = run(&scenario);
    assert!(
        original.failures.is_empty(),
        "le run original a des failures : {:?}",
        original.failures
    );

    // Le RON enregistré n'a pas de `bot:` (voir la doc du module) : il rejoue en `Scripted`,
    // exactement comme un scénario ordinaire (`build_app` pose toujours `InputSource::Scripted`
    // comme mode de base).
    let recorded_ron = original.recorded.to_ron();
    assert!(
        !recorded_ron.contains("bot:"),
        "l'enregistrement d'un run de bots ne doit pas contenir `bot:` : {recorded_ron}"
    );

    let mut recorded = Scenario::from_ron(&recorded_ron).expect("scénario enregistré invalide");
    // La trace s'arrête une frame avant la fin (l'état final n'est sauvegardé qu'à l'avance
    // suivante, voir `recording_replays_identically` dans `tests/scenarios.rs`) : rejouer le
    // même nombre de frames que l'original.
    recorded.frames = scenario.frames;
    let replayed = run(&recorded);

    assert_eq!(
        original.trace, replayed.trace,
        "le replay de l'enregistrement d'un run de bots diverge de l'original"
    );
}

/// Le cache physique v1 ne doit pas ajouter d'état rollback caché : le replay scripté
/// reproduit les décisions de navigation, de tir et d'interaction au bit près.
#[test]
fn hunter_run_replays_identically_once_recorded() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let mut scenario = two_fonceurs(600);
    scenario.players[0].bot = Some(BotProfile::Chasseur);
    scenario.players[1].bot = Some(BotProfile::Acheteur);
    let original = run(&scenario);
    assert!(original.failures.is_empty(), "{:?}", original.failures);
    let mut recorded = Scenario::from_ron(&original.recorded.to_ron()).unwrap();
    recorded.frames = scenario.frames;
    let replayed = run(&recorded);
    assert!(replayed.failures.is_empty(), "{:?}", replayed.failures);
    assert_eq!(original.trace, replayed.trace);
}

/// D68 : à terre, l'explorateur attend son équipier au lieu de continuer à fuir ou
/// à poursuivre le portail, ce qui ferait sortir sa surface de la portée de réanimation.
#[test]
fn downed_explorer_waits_for_revive() {
    let mut scenario =
        Scenario::from_ron(include_str!("../../../tests/scenarios/downed_revive.ron"))
            .expect("scénario de réanimation valide");
    scenario.players[1].bot = Some(BotProfile::Explorateur);
    let outcome = run(&scenario);
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
}

/// D68 : les morts de l'arène sont posées en fin de frame ; leur butin doit être identique
/// quand la frame suivante est rejouée depuis un snapshot portant déjà `Death`.
#[test]
fn arena_death_loot_survives_rollback() {
    let mut scenario = two_fonceurs(1100);
    scenario.game = "gungeon".into();
    scenario.map = "maps/etage_1.ldtk".into();
    scenario.map_seed = 2;
    scenario.floors = Some("etages_deux".into());
    for player in &mut scenario.players {
        player.bot = Some(BotProfile::Explorateur);
    }
    let outcome = run(&scenario);
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
}

/// D62 : armes que l'`explorateur` a tenues pendant une partie `etages_deux` de gungeon, avec
/// `shopping` (coffres ouverts, présentoirs achetés ; le combat de `prudent` ramasse l'arme qui
/// tombe). Une graine par appel.
fn shopped_weapons(seed: i32, frames: u32) -> std::collections::BTreeSet<String> {
    use bevy::prelude::*;
    use std::sync::{Arc, Mutex};
    let mut scenario = two_fonceurs(frames);
    scenario.players.truncate(1);
    scenario.game = "gungeon".into();
    scenario.map = "maps/etage_1.ldtk".into();
    scenario.map_seed = seed;
    scenario.floors = Some("etages_deux".into());
    scenario.players[0].bot = Some(BotProfile::Explorateur);
    let held = Arc::new(Mutex::new(std::collections::BTreeSet::new()));
    let probe = held.clone();
    let outcome = scenario::runner::run_with_options(
        &scenario,
        move |app| {
            app.insert_resource(bots::explorer::ExplorerOptions {
                boss_direct: false,
                policy: game::replay::ExplorerPolicy {
                    shopping: true,
                    ..scenario::runner::game_bot_policy("gungeon")
                },
            });
            app.add_systems(
                Last,
                move |inventories: Query<
                    &game::weapons::WeaponInventory,
                    With<game::character::player::Player>,
                >,
                      weapons: Query<&game::weapons::Weapon>| {
                    let mut held = probe.lock().unwrap();
                    for inventory in &inventories {
                        for (entity, _) in &inventory.weapons {
                            if let Ok(weapon) = weapons.get(*entity) {
                                held.insert(weapon.config.name.clone());
                            }
                        }
                    }
                },
            );
        },
        None,
    );
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    let held = held.lock().unwrap().clone();
    held
}

/// D62 : une partie ordinaire (`etages_deux`, grammaire `etage`) donne au moins une arme autre
/// que le pistolet de départ à l'explorateur qui achète et ouvre (atteignabilité, règle du
/// 2026-10-14).
#[test]
fn explorer_finds_weapons_on_ordinary_floors() {
    let seeds: Vec<i32> = std::env::var("ALACOD_SHOP_SEEDS")
        .ok()
        .map(|v| v.split(',').filter_map(|s| s.parse().ok()).collect())
        // M2-T14d : graine 41 (la grammaire `etage` compte sept types exigés, la graine 3 d'avant tire un
        // autre étage où la boutique et les coffres sont hors de portée en 1 000 frames ; 41, 13, 29 et 54 passent).
        .unwrap_or_else(|| vec![41]);
    let frames: u32 = std::env::var("ALACOD_SHOP_FRAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1000);
    for seed in seeds {
        let held = shopped_weapons(seed, frames);
        println!("graine {seed} : armes tenues {held:?}");
        assert!(
            held.iter().any(|name| name != "pistolet"),
            "graine {seed} : aucune arme obtenue ({held:?})"
        );
    }
}
