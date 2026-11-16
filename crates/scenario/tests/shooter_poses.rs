//! P-1d (`docs/conventions.md` § 122) : les fantassins de la **salle réelle** d'un jeu jouent leur cycle de tir à
//! l'écran. Verdict de William sur 1837 `9e7c6df` (engine `85c86745`) : « je n'ai rien vu comme animation sur les
//! fantassins » ; observé au vrai jeu (`bevy_remote`, `AnimationPlayer` des quatre fantassins) : un seul clip, l'attente,
//! pendant tout le cycle de la simulation, parce que `animate_models` ne lisait l'état de tir que d'une arme de
//! joueur (`WeaponInventory`) et qu'un fantassin n'en a pas.
//!
//! Le test joue la salle (`maps/salle.ldtk`, scénario `salle_deux_joueurs`) du **dépôt du jeu** : sa carte, son
//! `models.ron`, ses RON de personnages et de salve, son manifeste `anime.json` ; la simulation est la vraie
//! (règle `Shoot`, `Emitter`, `ranged_cycle`). La présentation est la vraie `animate_models`, sur des bancs sans rendu
//! (`ModelRig::headless`), et le clip joué est lu sur l'`AnimationPlayer` (le même que celui qu'on voit à l'écran).
//!
//! Un jeu externe n'a pas sa place dans le dépôt de l'engine : le test est ignoré par défaut et se lance sur une copie du jeu :
//! `ALACOD_P1D_GAME_DIR=/chemin/du/jeu cargo test -p scenario --profile headless --no-default-features \
//!   --test shooter_poses -- --ignored --exact the_shooters_of_the_real_room_play_their_whole_cycle`.
//! `ALACOD_P1D_SCENARIO` (défaut `salle_deux_joueurs`) choisit le scénario du jeu.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use bevy::animation::graph::AnimationNodeIndex;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use combat::actors::Enemy;
use content::Registry;
use game::character::config::CharacterConfigKey;
use game::model3d::{animate::animate_models, ModelRig};
use scenario::Scenario;

/// Un échantillon par frame et par tireur : le clip de plus fort poids et sa position (s).
type Series = Vec<(u32, String, f32)>;

fn play_the_room() -> BTreeMap<Entity, Series> {
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
    let frames = scenario.frames;

    let mut nodes: BTreeMap<Entity, BTreeMap<AnimationNodeIndex, String>> = BTreeMap::new();
    let mut series: BTreeMap<Entity, Series> = BTreeMap::new();
    scenario::runner::run_visiting(&scenario, frames, |world, frame| {
        rig_new_shooters(world, &mut nodes);
        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f64(1.0 / 60.0));
        world.run_system_once(animate_models).unwrap();
        let mut rigs = world.query::<(Entity, &ModelRig)>();
        let rigs: Vec<(Entity, Entity)> = rigs
            .iter(world)
            .filter_map(|(e, r)| Some((e, r.ready.as_ref()?.player)))
            .collect();
        for (shooter, player) in rigs {
            let Some(player) = world.get::<AnimationPlayer>(player) else {
                continue;
            };
            let Some((node, active)) = player
                .playing_animations()
                .max_by(|a, b| a.1.weight().total_cmp(&b.1.weight()))
            else {
                continue;
            };
            let clip = nodes[&shooter][node].clone();
            series
                .entry(shooter)
                .or_default()
                .push((frame, clip, active.seek_time()));
        }
    });
    series
}

/// `game: "<nom>"` du scénario : le nom du jeu, pour `runner::game_dir`.
fn game_name(scenario: &str) -> String {
    let rest = &scenario[scenario.find("game:").expect("`game:` du scénario") + 5..];
    rest.split('"').nth(1).expect("nom du jeu").to_string()
}

/// Pose un banc sans rendu sur chaque ennemi dont le personnage a un modèle (une fois).
fn rig_new_shooters(
    world: &mut World,
    nodes: &mut BTreeMap<Entity, BTreeMap<AnimationNodeIndex, String>>,
) {
    let mut query =
        world.query_filtered::<(Entity, &CharacterConfigKey), (With<Enemy>, Without<ModelRig>)>();
    let enemies: Vec<(Entity, String)> = query
        .iter(world)
        .map(|(e, key)| (e, key.0.clone()))
        .collect();
    for (enemy, model) in enemies {
        let clips: BTreeMap<String, (AnimationNodeIndex, f32)> = {
            let registry = world.resource::<Registry>();
            let Some((_, manifest)) = registry.models.as_ref().and_then(|t| t.get(&model)) else {
                continue;
            };
            manifest
                .clips
                .keys()
                .enumerate()
                .map(|(i, clip)| {
                    let duration = manifest.duration(clip).unwrap_or(1.0);
                    (clip.clone(), (AnimationNodeIndex::new(i + 1), duration))
                })
                .collect()
        };
        nodes.insert(
            enemy,
            clips
                .iter()
                .map(|(clip, (node, _))| (*node, clip.clone()))
                .collect(),
        );
        let root = world.spawn(Transform::default()).id();
        let player = world.spawn(AnimationPlayer::default()).id();
        world
            .entity_mut(enemy)
            .insert(ModelRig::headless(&model, root, player, clips));
    }
}

/// Les frames (consécutives) où `clip` est le clip joué, première et dernière.
fn span(series: &Series, clip: &str) -> Option<(u32, u32)> {
    let frames: Vec<u32> = series
        .iter()
        .filter(|(_, c, _)| c == clip)
        .map(|(f, ..)| *f)
        .collect();
    Some((*frames.first()?, *frames.last()?))
}

#[test]
#[ignore = "joue le dépôt d'un jeu externe : ALACOD_P1D_GAME_DIR=<dossier du jeu>"]
fn the_shooters_of_the_real_room_play_their_whole_cycle() {
    let series = play_the_room();
    assert!(!series.is_empty(), "aucun ennemi à modèle dans la salle");
    for (shooter, series) in &series {
        let played: Vec<&str> = series.iter().map(|(_, c, _)| c.as_str()).collect();
        // Chaque geste du cycle est joué, dans l'ordre du cycle (banc `armes.js` A:686-735).
        let first = |clip: &str| {
            played
                .iter()
                .position(|c| *c == clip)
                .unwrap_or_else(|| panic!("{shooter}: `{clip}` n'est jamais joué (clips joués : {:?})", {
                    let mut seen: Vec<&str> = played.clone();
                    seen.dedup();
                    seen
                }))
        };
        let (raise, aim, shot, reload) = (
            first("epauler"),
            first("viser"),
            first("tir"),
            first("rechargement"),
        );
        assert!(
            raise < aim && aim < shot && shot < reload,
            "{shooter}: ordre du cycle faux (épauler {raise}, viser {aim}, tir {shot}, rechargement {reload})"
        );
        // En joue : le cône dure 60 frames ; le rechargement 180 (3 s) : le clip de 2 s y est étiré, joué une fois.
        let (a0, a1) = span(series, "viser").unwrap();
        assert!(a1 - a0 + 1 >= 55, "{shooter}: en joue {} frames", a1 - a0 + 1);
        let (r0, r1) = span(series, "rechargement").unwrap();
        let len = r1 - r0 + 1;
        assert!((170..=190).contains(&len), "{shooter}: rechargement joué {len} frames (180 attendues)");
        let middle = series
            .iter()
            .find(|(f, c, _)| c == "rechargement" && *f == r0 + len / 2)
            .map(|(_, _, seek)| *seek)
            .unwrap();
        assert!(
            (middle - 1.0).abs() < 0.15,
            "{shooter}: à mi-rechargement le clip de 2 s est à {middle} s (1 s attendue : joué une fois, étiré sur 3 s)"
        );
        let end = series
            .iter()
            .rev()
            .find(|(_, c, _)| c == "rechargement")
            .map(|(_, _, seek)| *seek)
            .unwrap();
        assert!(end > 1.85, "{shooter}: le clip finit à {end} s sur 2 s : rechargement incomplet");
    }
}
