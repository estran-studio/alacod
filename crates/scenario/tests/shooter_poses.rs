//! P-1d (`docs/conventions.md` § 122) : les fantassins de la **salle réelle** d'un jeu jouent leur cycle de tir à
//! l'écran. Verdict de William sur 1837 `9e7c6df` (engine `85c86745`) : « je n'ai rien vu comme animation sur les
//! fantassins » ; observé au vrai jeu (`bevy_remote`, `AnimationPlayer` des quatre fantassins) : un seul clip, l'attente,
//! pendant tout le cycle de la simulation, parce que `animate_models` ne lisait l'état de tir que d'une arme de
//! joueur (`WeaponInventory`) et qu'un fantassin n'en a pas.
//!
//! Les tests jouent la salle (`maps/salle.ldtk`, scénario `salle_deux_joueurs`) du **dépôt du jeu** : sa carte, son
//! `models.ron`, ses RON de personnages et de salve, son manifeste `anime.json` ; la simulation est la vraie (règle
//! `Shoot`, `Emitter`, `ranged_cycle`) et le clip joué est lu dans `ModelPlayback`, décidé hors rendu (aucune fenêtre).
//!
//! Un jeu externe n'a pas sa place dans le dépôt de l'engine : ces tests sont ignorés par défaut et se lancent sur une copie du jeu :
//! `ALACOD_P1D_GAME_DIR=/chemin/du/jeu cargo test -p scenario --profile headless --no-default-features \
//!   --test shooter_poses -- --ignored`. `ALACOD_P1D_SCENARIO` (défaut `salle_deux_joueurs`) choisit le scénario du jeu.

use std::collections::BTreeMap;
use std::path::PathBuf;

use bevy::prelude::*;
use game::model3d::playback::ModelPlayback;
use scenario::Scenario;

/// Un échantillon par frame et par personnage : le clip joué et sa position (s).
type Series = Vec<(u32, String, f32)>;

fn scenario_of_the_game() -> Scenario {
    if map_ldtk::RENDER_ENABLED {
        panic!("à lancer sans le rendu (--no-default-features)");
    }
    let dir = PathBuf::from(
        std::env::var("ALACOD_P1D_GAME_DIR").expect("ALACOD_P1D_GAME_DIR : le dossier du jeu à jouer"),
    );
    let name = std::env::var("ALACOD_P1D_SCENARIO").unwrap_or_else(|_| "salle_deux_joueurs".into());
    let text = std::fs::read_to_string(dir.join(format!("tests/scenarios/{name}.ron")))
        .expect("scénario du jeu");
    std::env::set_var("ALACOD_GAME_NAME", game_name(&text));
    std::env::set_var("ALACOD_GAME_DIR", &dir);
    let mut scenario = Scenario::from_ron(&text).expect("scénario");
    scenario.expect.clear();
    scenario
}

/// `game: "<nom>"` du scénario : le nom du jeu, pour `runner::game_dir`.
fn game_name(scenario: &str) -> String {
    let rest = &scenario[scenario.find("game:").expect("`game:` du scénario") + 5..];
    rest.split('"').nth(1).expect("nom du jeu").to_string()
}

fn play_the_room() -> BTreeMap<Entity, Series> {
    let scenario = scenario_of_the_game();
    let mut series: BTreeMap<Entity, Series> = BTreeMap::new();
    scenario::runner::run_visiting(&scenario, scenario.frames, |world, frame| {
        let mut query = world.query::<(Entity, &ModelPlayback)>();
        for (entity, playback) in query.iter(world) {
            if let (Some(clip), Some(time)) = (playback.clip(), playback.time()) {
                series
                    .entry(entity)
                    .or_default()
                    .push((frame, clip.to_string(), time));
            }
        }
    });
    series
}

/// La première plage de frames consécutives où `clip` est le clip joué : (première, dernière).
fn span(series: &Series, clip: &str) -> Option<(u32, u32)> {
    let mut frames = series.iter().filter(|(_, c, _)| c == clip).map(|(f, ..)| *f);
    let first = frames.next()?;
    let mut last = first;
    for frame in frames {
        if frame != last + 1 {
            break;
        }
        last = frame;
    }
    Some((first, last))
}

#[test]
#[ignore = "joue le dépôt d'un jeu externe : ALACOD_P1D_GAME_DIR=<dossier du jeu>"]
fn the_shooters_of_the_real_room_play_their_whole_cycle() {
    let series = play_the_room();
    assert!(!series.is_empty(), "aucun personnage à modèle dans la salle");
    // Les tireurs de la salle : ceux qui épaulent au moins une fois. Il en faut quatre (les fantassins) : sinon la
    // simulation les fait tirer sans qu'aucun geste ne soit joué (le verdict du 16 nov).
    let shooters: Vec<_> = series
        .iter()
        .filter(|(_, s)| s.iter().any(|(_, c, _)| c == "epauler"))
        .collect();
    assert!(
        shooters.len() >= 4,
        "{} tireur(s) épaulent : les quatre fantassins attendus ; clips joués par personnage : {:#?}",
        shooters.len(),
        series
            .iter()
            .map(|(who, s)| {
                let mut seen: Vec<&str> = s.iter().map(|(_, c, _)| c.as_str()).collect();
                seen.dedup();
                (who, seen)
            })
            .collect::<Vec<_>>()
    );
    for (who, series) in shooters {
        let played: Vec<&str> = series.iter().map(|(_, c, _)| c.as_str()).collect();
        // Chaque geste du cycle est joué, dans l'ordre du cycle (banc `armes.js` A:686-735).
        let first = |clip: &str| {
            played
                .iter()
                .position(|c| *c == clip)
                .unwrap_or_else(|| panic!("{who}: `{clip}` n'est jamais joué"))
        };
        let (raise, aim, shot, reload) = (
            first("epauler"),
            first("viser"),
            first("tir"),
            first("rechargement"),
        );
        assert!(
            raise < aim && aim < shot && shot < reload,
            "{who}: ordre du cycle faux (épauler {raise}, viser {aim}, tir {shot}, rechargement {reload})"
        );
        // En joue : le cône dure 60 frames ; le rechargement 180 (3 s) : le clip de 2 s y est étiré, joué une fois.
        let (a0, a1) = span(series, "viser").unwrap();
        assert!(a1 - a0 + 1 >= 55, "{who}: en joue {} frames", a1 - a0 + 1);
        let (r0, r1) = span(series, "rechargement").unwrap();
        let len = r1 - r0 + 1;
        assert!((170..=190).contains(&len), "{who}: rechargement joué {len} frames (180 attendues)");
        let middle = series
            .iter()
            .find(|(f, c, _)| c == "rechargement" && *f == r0 + len / 2)
            .map(|(_, _, seek)| *seek)
            .unwrap();
        assert!(
            (middle - 1.0).abs() < 0.15,
            "{who}: à mi-rechargement le clip de 2 s est à {middle} s (1 s attendue : joué une fois, étiré sur 3 s)"
        );
        let end = series
            .iter()
            .find(|(f, c, _)| c == "rechargement" && *f == r1)
            .map(|(_, _, seek)| *seek)
            .unwrap();
        assert!(end > 1.85, "{who}: le clip finit à {end} s sur 2 s : rechargement incomplet");
    }
}

/// Le contrôle générique : aucun invariant du runner (dont `animation_vivante`) ne tombe sur la salle réelle.
#[test]
#[ignore = "joue le dépôt d'un jeu externe : ALACOD_P1D_GAME_DIR=<dossier du jeu>"]
fn the_real_room_passes_the_animation_invariant() {
    let scenario = scenario_of_the_game();
    let outcome = scenario::runner::run(&scenario);
    let animation: Vec<&String> = outcome
        .failures
        .iter()
        .filter(|f| f.contains("animation_vivante"))
        .collect();
    assert!(animation.is_empty(), "{animation:#?}");
}
