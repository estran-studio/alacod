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
        powerup_drop_chance_override: None,
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
