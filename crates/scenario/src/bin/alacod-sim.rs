//! `alacod-sim` (T2.11) : joue des centaines de parties bots headless, une par graine, et sort
//! des métriques JSON.
//!
//! ```text
//! alacod-sim --game zombies --bots 4 --map exemples/test_map.ldtk \
//!     --seeds 1..50 --until-wave 10 --max-frames 20000 \
//!     [--floors <séquence>] [--save-scenario <dossier>] [--json <fichier>]
//! ```
//!
//! - `--profiles a,b,...` : un profil par bot ; défaut : `acheteur` pour tous les bots.
//!   Les profils v0 restent disponibles : `fonceur,fonceur,prudent,immobile`.
//! - `--map <fichier.ldtk>` : carte explicite relative aux assets du jeu ; sinon `start_map`.
//! - `--progress` : état de la vague toutes les 1000 frames, hors simulation.
//! - `--log` (D49) : journaux du jeu (`info!` de la simulation : dégâts, mises à terre, butin,
//!   projectiles…) sur stderr, filtrés par `RUST_LOG` (défaut `info`), sans horodatage ni
//!   couleur (comparables d'une exécution à l'autre). Sans `--log`, aucun subscriber : sorties et
//!   JSON inchangés.
//! - `--floors <id>` (T1.8) : mode `Floors` avec la séquence `id` du dossier `Floors` du jeu ;
//!   le JSON rapporte `floor`, le niveau atteint (pas d'arrêt anticipé par niveau : T1.14).
//! - `--shopping` (D62) : les `explorateur` ouvrent les coffres et achètent les présentoirs qu'ils peuvent payer.
//! - `--boss-direct` (M2-T16) : les `explorateur` vont droit au boss (`bots::explorer::ExplorerOptions`).
//! - M3-T14 (`docs/conventions.md` §79) : `--policy <fichier.ron>` (politique de l'`explorateur`, sinon `bot.ron` des
//!   assets du jeu, sinon `Default`), `--devil never|above:<n>` (décision du marché du diable), `--bombs <n>` (chaque bot
//!   porte la bombe d'essai : mèche 60, rayon 56, dégâts 4, tir ami, `n` bombes de départ ; `isaac` n'en donne pas à son
//!   personnage avant M3-T11d), `--pickup-progress` (un ramassage compte comme une progression du soft-lock), `--explored-progress` (entrer dans une case de 32 px jamais visitée aussi : étages de `gungeon`). Le JSON
//!   rapporte `mechanics` (décomptes de moments clés : `mur_secret`, `porte_a_cle`, `bombe`, `coffre`, `achat`, `diable`…).
//! - M2-T16 : le JSON rapporte aussi `outcome` (classement de la graine : `victoire`, `objectif`,
//!   `mort`, `soft-lock`, `plafond`, `desync`) et `rooms` (par étage atteint : salles typées,
//!   salles où un joueur est entré, salles nettoyées) ; un résumé par issue finit la sortie
//!   d'erreur.
//!
//! - `--seeds A..B` : graines `A` à `B` **inclusivement** (`1..50` = 50 graines, la carte est
//!   générée avec `map_seed = graine`).
//! - Un scénario est construit en mémoire par graine (`frames: max_frames`, un `PlayerScript` par
//!   bot, aucun `inputs`) et joué en synctest (comme `make test_scenarios`), avec arrêt anticipé
//!   dès que `--until-wave` est atteinte ou que tous les joueurs sont morts
//!   (`scenario::runner::StopEarly`).
//! - `--save-scenario <dossier>` : écrit `seed_<graine>.ron`, l'enregistrement exact des inputs
//!   envoyés à GGRS (rejouable en `Scripted`, voir `crates/scenario/tests/bots.rs`).
//! - `--json <fichier>` : le JSON (toujours un tableau, une entrée par graine) va dans ce
//!   fichier plutôt que stdout.
//! - Si le plafond est atteint sans objectif ni mort de tous les joueurs, `softlock`
//!   contient les faits observés et deux instantanés (fin et 600 frames auparavant) :
//!   vague, ennemis/cibles/chemins, joueurs/munitions, portes, fenêtres et grille ASCII.
//!   Ce diagnostic ne modifie ni les règles de jeu ni la trace rollback.
//! - Code de sortie 1 si au moins une graine a un desync (mismatch synctest) : un vrai bug de
//!   déterminisme, jamais à masquer (voir la doc de `bots::input`).

use std::path::PathBuf;
use std::time::Instant;

use game::character::player::input::InputSource;
use game::replay::{BotProfile, PlayerScript, Scenario};
use scenario::runner::{self, StopEarly};
use serde::Serialize;

/// Pistes du hub de gungeon (même parcours que `gungeon_full_run.ron` et `p2p-stress-inputs.py --hub-door`) :
/// le marchand (f80), puis la porte (f170) ; le joueur 1 ne fait rien.
const HUB_DOOR_RON: &str = "(stay: false, inputs: [[
    (from: 10, to: 42, buttons: [Right]),
    (from: 42, to: 74, buttons: [Up]),
    (from: 80, to: 82, buttons: [Interaction]),
    (from: 100, to: 138, buttons: [Right]),
    (from: 138, to: 157, buttons: [Down]),
    (from: 170, to: 172, buttons: [Interaction]),
], []])";

#[derive(Debug, Serialize)]
struct SimResult {
    seed: i32,
    wave: u32,
    /// Niveau atteint (mode `Floors`, T1.8 : `FloorState::index`, `0` = premier niveau).
    floor: u32,
    frames: u32,
    deaths: u32,
    kills: u32,
    wall_seconds: f64,
    desync: bool,
    sim_fps: f64,
    /// T1.14 : frame de chaque passage de niveau (mode `Floors`), vide sinon.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    floor_frames: Vec<u32>,
    /// T1.14 : dégâts subis par les joueurs (somme des baisses de santé).
    damage_taken: u32,
    /// D43 : issue de la partie (`"Defeat"`, `"Victory"`…) et sa frame, si elle est terminée.
    #[serde(skip_serializing_if = "Option::is_none")]
    run_end: Option<(String, u32)>,
    /// T1.14 : frames où l'esquive a remplacé le déplacement d'au moins un bot `prudent`.
    #[serde(skip_serializing_if = "is_zero")]
    dodges: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    failures: Vec<String>,
    /// État au plafond (ou au soft-lock `Floors`) et 600 frames auparavant ; absent en cas
    /// d'arrêt volontaire.
    #[serde(skip_serializing_if = "Option::is_none")]
    softlock: Option<scenario::softlock::SoftlockDump>,
    /// Analyse Depart (m1-d36-et-analyse-depart) : portes ouvertes (moments clés `door`).
    doors_opened: u32,
    /// M3-T14 : décomptes des moments clés qui disent quelles mécaniques la partie a rencontrées
    /// (`mur_secret`, `porte_a_cle`, `bombe`, `coffre`, `achat`…), voir [`mechanics`].
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    mechanics: std::collections::BTreeMap<&'static str, u32>,
    /// D88 : personnages à variante (champions, élites) apparus pendant la partie (moments clés
    /// `variant_spawn`) : l'atteignabilité des variantes se lit par graine.
    variants: u32,
    /// État de chaque joueur à l'arrêt de la graine (objectif atteint, fin de partie ou
    /// plafond) : solde, position, présence dans la salle de départ.
    players_end: Vec<PlayerEnd>,
    /// Revue M0 (S5) : une entrée par vague commencée (équilibrage des premières vagues).
    waves: Vec<WaveStat>,
    /// M2-T16 : issue de la graine (voir [`classify_outcome`]).
    outcome: &'static str,
    /// M2-T16 : salles par étage atteint (mode `Floors` avec salles typées), vide sinon.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    rooms: Vec<FloorRooms>,
}

/// M2-T16 : salles typées d'un étage, hors simulation (relu dans `Last` à chaque frame).
#[derive(Serialize, Clone, Debug, Default, PartialEq)]
struct FloorRooms {
    floor: u32,
    /// Salles typées de l'étage.
    total: u32,
    /// Salles où un joueur debout est entré (bornes de la salle).
    visited: u32,
    /// Salles nettoyées (`RoomState::Cleared`).
    cleared: u32,
    /// Salles verrouillantes de l'étage (celles qu'il faut nettoyer).
    locking: u32,
}

/// Accumulateur des salles (hors simulation) : par étage, salles vues et salles visitées.
#[derive(Default)]
struct RoomProbe {
    floors: std::collections::BTreeMap<u32, (FloorRooms, std::collections::BTreeSet<String>)>,
}

/// M2-T16 : classement d'une graine. Ordre : desync, puis issue de la partie (victoire ou
/// défaite = `mort`), objectif atteint (`--until-*`), mort de tous les joueurs, soft-lock
/// `Floors`, plafond de frames.
fn classify_outcome(
    desync: bool,
    run_end: Option<&str>,
    softlock: bool,
    reached: bool,
    all_dead: bool,
) -> &'static str {
    if desync {
        "desync"
    } else if run_end == Some("Victory") {
        "victoire"
    } else if reached {
        "objectif"
    } else if all_dead || run_end.is_some() {
        "mort"
    } else if softlock {
        "soft-lock"
    } else {
        "plafond"
    }
}

/// Revue M0 (S5) : relevé d'une vague, hors simulation (lu dans `Last` à chaque frame).
/// `start` : première frame de la vague (`WaveState::current_wave` change) ; `cleared` :
/// première frame en `WaveComplete`, `None` si la graine s'arrête avant ; `spawning` :
/// première frame d'apparition (hors préparation) ; `damage` : baisses
/// de santé des joueurs pendant la vague ; `downs` : mises à terre ; `deaths` : joueurs
/// disparus (entité détruite) pendant la vague.
#[derive(Serialize, Clone, Debug, Default)]
struct WaveStat {
    wave: u32,
    enemies: u32,
    start: u32,
    spawning: Option<u32>,
    cleared: Option<u32>,
    damage: u32,
    downs: u32,
    deaths: u32,
}

/// Accumulateur du relevé par vague (hors simulation).
#[derive(Default)]
struct WaveProbe {
    last_frame: u32,
    waves: Vec<WaveStat>,
    health: std::collections::BTreeMap<usize, bevy_fixed::fixed_math::Fixed>,
    downed: std::collections::BTreeSet<usize>,
    damage: bevy_fixed::fixed_math::Fixed,
}

/// Hors simulation : relu dans `Last` à chaque frame, la dernière valeur est rapportée.
#[derive(Serialize, Clone, Debug)]
struct PlayerEnd {
    handle: usize,
    currency: u32,
    downed: bool,
    x: f32,
    y: f32,
    /// Dans la salle de départ (`RoomConfig::spawn`) ; `None` hors de toute salle connue.
    in_spawn_room: Option<bool>,
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

/// Condition d'arrêt (T1.14) : `--until-wave <n>` en mode `Waves`, `--until-floor <n>` en mode
/// `Floors` (exige `--floors`). L'un des deux est obligatoire.
fn stop_condition(
    until_wave: Option<&str>,
    until_floor: Option<&str>,
    floors: Option<&str>,
) -> Result<(Option<u32>, Option<u32>), String> {
    let until_wave = until_wave
        .map(|v| v.parse().map_err(|_| "--until-wave : entier".to_string()))
        .transpose()?;
    let until_floor = until_floor
        .map(|v| v.parse().map_err(|_| "--until-floor : entier".to_string()))
        .transpose()?;
    if until_floor.is_some() && floors.is_none() {
        return Err("--until-floor exige --floors <séquence> (mode Floors)".into());
    }
    if until_wave.is_none() && until_floor.is_none() {
        return Err("--until-wave <n> (Waves) ou --until-floor <n> (Floors) obligatoire".into());
    }
    Ok((until_wave, until_floor))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // D49 : `--log` installe le subscriber que le binaire n'avait pas (`RUST_LOG` restait muet).
    if args.iter().any(|arg| arg == "--log") {
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .without_time()
            .with_ansi(false)
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .init();
    }
    let opt = |name: &str| -> Option<String> {
        args.iter().position(|a| a == name).map(|i| {
            args.get(i + 1)
                .unwrap_or_else(|| panic!("{name} demande une valeur"))
                .clone()
        })
    };
    let require = |name: &str| {
        opt(name).unwrap_or_else(|| {
            panic!(
                "usage : alacod-sim --game <jeu> --bots <n> [--profiles <a,b,...>] \
                 --seeds <de>..<à> (--until-wave <n> | --floors <séquence> --until-floor <n>) \
                 --max-frames <n> [--map <fichier.ldtk>] [--save-scenario <dossier>] [--json <fichier>] [--log] ({name} manquant)"
            )
        })
    };

    let game = opt("--game").unwrap_or_else(|| "zombies".into());
    let bots: usize = require("--bots").parse().expect("--bots : entier");
    let profiles: Vec<BotProfile> = opt("--profiles")
        .unwrap_or_else(|| vec!["acheteur"; bots].join(","))
        .split(',')
        .map(|name| {
            BotProfile::parse_name(name.trim()).unwrap_or_else(|| {
                panic!(
                    "--profiles : profil inconnu « {name} » (attendu : immobile, fonceur, prudent, chasseur, acheteur, explorateur)"
                )
            })
        })
        .collect();
    assert_eq!(
        profiles.len(),
        bots,
        "--profiles doit avoir {bots} entrées (--bots {bots}), en a {}",
        profiles.len()
    );

    let (seed_from, seed_to) = parse_seed_range(&require("--seeds"));
    let floors = opt("--floors");
    let (until_wave, until_floor) = stop_condition(
        opt("--until-wave").as_deref(),
        opt("--until-floor").as_deref(),
        floors.as_deref(),
    )
    .unwrap_or_else(|e| panic!("alacod-sim : {e}"));
    // Défaut généreux (20 000 frames ≈ 5.5 min de partie simulée) : `--until-wave` arrête
    // presque toujours la partie bien avant, `--max-frames` n'est qu'un filet de sécurité.
    let max_frames: u32 = opt("--max-frames")
        .map(|v| v.parse().expect("--max-frames : entier"))
        .unwrap_or(20_000);
    let save_scenario_dir = opt("--save-scenario").map(PathBuf::from);
    let json_path = opt("--json").map(PathBuf::from);

    if let Some(dir) = &save_scenario_dir {
        std::fs::create_dir_all(dir).unwrap_or_else(|e| panic!("--save-scenario {dir:?} : {e}"));
    }

    // Carte de départ du jeu (manifeste, T1.5) : la même que ses parties normales, plutôt que le
    // défaut générique de `Scenario` (`exemples/test_map.ldtk`, qui n'est correct que par
    // coïncidence pour `zombies`). `--map` la remplace (chemin relatif aux assets du jeu,
    // comme `start_map`).
    let game_root = runner::game_dir(&game);
    let (_registry, manifest, content_errors) = content::load_and_lint(&game_root)
        .unwrap_or_else(|e| panic!("alacod-sim : « {game} » : game.ron invalide : {e}"));
    assert!(
        content_errors.is_empty(),
        "alacod-sim : « {game} » : contenu invalide :\n{content_errors:#?}"
    );
    let map = opt("--map").unwrap_or_else(|| manifest.entry.start_map.clone());
    // T1.8 : `--floors <id>` impose le mode `Floors` avec cette séquence du dossier `Floors`
    // du jeu (`Scenario::floors`) ; `--map` est alors ignorée. T1.14 : `--until-floor <n>`
    // arrête la graine au n-ième passage de portail (`floor >= n`) ; le JSON rapporte le
    // niveau atteint (`floor`) et la frame de chaque passage (`floor_frames`).

    eprintln!(
        "alacod-sim : {game}, {bots} bots ({}), graines {seed_from}..={seed_to}, jusqu'à la vague {until_wave:?} / au niveau {until_floor:?} ou {max_frames} frames, carte {map}",
        profiles.iter().map(|p| p.name()).collect::<Vec<_>>().join(",")
    );
    // M3-T14 : politique de l'`explorateur` (`docs/conventions.md` §79) : `--policy <fichier.ron>`, sinon
    // `bot.ron` des assets du jeu s'il existe, sinon la politique par défaut ; `--shopping`, `--devil` la
    // surchargent ensuite. `--bombs <n>` : chaque bot porte la bombe d'essai de ces campagnes (`isaac`
    // n'en donne pas à son personnage avant M3-T11d) avec `n` bombes de départ.
    let mut policy: game::replay::ExplorerPolicy = {
        let path = opt("--policy")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(format!("{}/bot.ron", runner::assets_dir(&game))));
        match std::fs::read_to_string(&path) {
            Ok(text) => ron::from_str(&text)
                .unwrap_or_else(|e| panic!("alacod-sim : politique {path:?} invalide : {e}")),
            Err(_) if opt("--policy").is_none() => Default::default(),
            Err(e) => panic!("alacod-sim : --policy {path:?} : {e}"),
        }
    };
    policy.shopping |= args.iter().any(|arg| arg == "--shopping");
    if let Some(devil) = opt("--devil") {
        policy.devil = match devil.split_once(':') {
            None if devil == "never" => game::replay::DevilPolicy::Never,
            Some(("above", n)) => game::replay::DevilPolicy::IfHealthAbove(
                n.parse()
                    .unwrap_or_else(|_| panic!("--devil above:<n> : nombre, pas « {n} »")),
            ),
            _ => panic!("--devil : never ou above:<n> (points de vie)"),
        };
    }
    let bomb_kit: Option<(usize, world::BombDef)> = opt("--bombs").map(|n| {
        (
            n.parse().expect("--bombs : entier"),
            world::BombDef {
                fuse_frames: 60,
                radius: bevy_fixed::fixed_math::Fixed::from_num(56),
                damage: bevy_fixed::fixed_math::Fixed::from_num(4),
                friendly_fire: sim_core::damage::FriendlyFire::Always,
            },
        )
    });
    eprintln!("alacod-sim : politique de l'explorateur {policy:?}, kit de bombes {bomb_kit:?}");

    let mut results = Vec::new();
    let mut any_desync = false;
    let run_started = Instant::now();

    for seed in seed_from..=seed_to {
        let players = profiles
            .iter()
            .map(|profile| PlayerScript {
                inputs: vec![],
                bot: Some(*profile),
                items: bomb_kit
                    .map(|(n, _)| vec!["bomb".to_string(); n])
                    .unwrap_or_default(),
                bomb: bomb_kit.map(|(_, def)| def),
                ..Default::default()
            })
            .collect();
        let mut run_scenario = Scenario {
            game: game.clone(),
            map: map.clone(),
            map_seed: seed,
            frames: max_frames,
            players,
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
            explorer: Some(policy),
            powerup_drop_chance_override: None,
            floors: floors.clone(),
            // T1.10 : progression du manifeste (pas d'option `--progression` en v1).
            progression: None,
            clocks: None,
            difficulty: None,
            characters: vec![],
            mode: None,
            ui: Vec::new(),
        };
        // `ALACOD_SIM_HUB=1` (enregistrement des bots pour le p2p, D85) : la partie passe par le hub
        // du manifeste (le joueur 0 va à la porte, f170) comme une partie en ligne, au lieu de
        // démarrer dans la run ; `--save-scenario` n'enregistre que les frames de la run (les frames
        // du hub, plus courtes, sont écrasées par celles de la run qui repartent de 0).
        if std::env::var("ALACOD_SIM_HUB").is_ok_and(|v| v == "1") {
            run_scenario.hub = Some(
                ron::from_str::<game::replay::HubSetup>(HUB_DOOR_RON)
                    .expect("ALACOD_SIM_HUB : pistes du hub"),
            );
        }

        let stop_early = StopEarly {
            until_wave,
            until_floor,
            stop_when_all_players_dead: true,
            // D43 : une partie terminée (défaite ou victoire) n'est plus jouée jusqu'au plafond
            stop_when_run_ended: true,
            pickups_are_progress: args.iter().any(|arg| arg == "--pickup-progress"),
            explored_is_progress: args.iter().any(|arg| arg == "--explored-progress"),
        };
        let dodges = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let dodges_probe = dodges.clone();
        let players_end = std::sync::Arc::new(std::sync::Mutex::new(Vec::<PlayerEnd>::new()));
        let players_end_probe = players_end.clone();
        let wave_probe = std::sync::Arc::new(std::sync::Mutex::new(WaveProbe::default()));
        let wave_probe_sys = wave_probe.clone();
        let room_probe = std::sync::Arc::new(std::sync::Mutex::new(RoomProbe::default()));
        let room_probe_sys = room_probe.clone();
        let boss_direct = args.iter().any(|arg| arg == "--boss-direct");

        let wall_start = Instant::now();
        // `InputSource::Bot` : aucun joueur scripté ici, contrairement aux scénarios RON qui
        // gardent `Scripted` comme mode de base (voir `runner::build_app`) — mais le résultat
        // serait identique avec `Scripted` : `BotAssignments` couvre tous les joueurs.
        let outcome = runner::run_with_options(
            &run_scenario,
            |app| {
                // Avec le hub (`ALACOD_SIM_HUB`), le mode `Scripted` du runner reste : le hub se joue
                // par ses pistes scriptées, les bots prennent la main à la porte (`swap_hub_inputs`).
                if run_scenario.hub.is_none() {
                    app.insert_resource(InputSource::Bot);
                }
                app.insert_resource(bots::explorer::ExplorerOptions {
                    boss_direct,
                    policy,
                });
                // M2-T16 : salles de l'étage courant, visitées et nettoyées (hors simulation)
                app.add_systems(
                    bevy::prelude::Last,
                    move |rooms: map_ldtk::game::rooms::CurrentRooms,
                          states: Option<bevy::prelude::Res<world::RoomStates>>,
                          table: Option<bevy::prelude::Res<world::RoomKindTable>>,
                          floor: Option<bevy::prelude::Res<run::FloorState>>,
                          frame: Option<bevy::prelude::Res<utils::frame::FrameCount>>,
                          players: bevy::prelude::Query<
                        (
                            &bevy_fixed::fixed_math::FixedTransform3D,
                            bevy::prelude::Has<combat::downed::Downed>,
                        ),
                        bevy::prelude::With<combat::actors::Player>,
                    >| {
                        if frame.is_none_or(|f| f.frame == 0) {
                            return;
                        }
                        let index = floor.map_or(0, |f| f.index);
                        let mut probe = room_probe_sys.lock().unwrap();
                        let (stat, seen) = probe.floors.entry(index).or_default();
                        stat.floor = index;
                        let mut total = 0;
                        let mut cleared = 0;
                        let mut locking = 0;
                        for (level, bounds, kind) in rooms.iter() {
                            total += 1;
                            if table.as_deref().is_some_and(|t| t.locks(&kind.0)) {
                                locking += 1;
                            }
                            if states
                                .as_deref()
                                .is_some_and(|s| s.get(&level.0) == world::RoomState::Cleared)
                            {
                                cleared += 1;
                            }
                            let inside = players.iter().any(|(t, downed)| {
                                !downed && bounds.contains(t.translation.truncate())
                            });
                            if inside {
                                seen.insert(level.0.clone());
                            }
                        }
                        stat.total = total;
                        stat.cleared = cleared;
                        stat.locking = locking;
                        stat.visited = seen.len() as u32;
                    },
                );
                // T1.14 : compteur d'esquives (hors simulation), relu en fin de graine
                app.add_systems(
                    bevy::prelude::Last,
                    move |stats: bevy::prelude::Res<bots::BotStats>| {
                        dodges_probe.store(stats.dodges, std::sync::atomic::Ordering::Relaxed);
                    },
                );
                app.add_systems(
                    bevy::prelude::Last,
                    move |players: bevy::prelude::Query<(
                        &combat::actors::Player,
                        &bevy_fixed::fixed_math::FixedTransform3D,
                        Option<&run::currency::Currency>,
                        bevy::prelude::Has<combat::downed::Downed>,
                    )>,
                          rooms: bevy::prelude::Query<(
                        &map::game::entity::map::room::RoomComponent,
                        &map::game::entity::map::room::RoomBounds,
                    )>| {
                        let mut snapshot: Vec<PlayerEnd> = players
                            .iter()
                            .map(|(player, transform, currency, downed)| {
                                let pos = transform.translation.truncate();
                                PlayerEnd {
                                    handle: player.handle,
                                    currency: currency.map_or(0, |c| c.0),
                                    downed,
                                    x: pos.x.to_num::<f32>(),
                                    y: pos.y.to_num::<f32>(),
                                    in_spawn_room: rooms
                                        .iter()
                                        .find(|(_, bounds)| bounds.contains(pos))
                                        .map(|(room, _)| room.config.spawn),
                                }
                            })
                            .collect();
                        snapshot.sort_by_key(|p| p.handle);
                        *players_end_probe.lock().unwrap() = snapshot;
                    },
                );
                // Revue M0 (S5) : relevé par vague (début, fin, dégâts, mises à terre, morts)
                app.add_systems(
                    bevy::prelude::Last,
                    move |wave: Option<bevy::prelude::Res<game::waves::WaveState>>,
                          frame: Option<bevy::prelude::Res<utils::frame::FrameCount>>,
                          players: bevy::prelude::Query<(
                        &combat::actors::Player,
                        &combat::actors::Health,
                        bevy::prelude::Has<combat::downed::Downed>,
                    )>| {
                        let (Some(wave), Some(frame)) = (wave, frame) else {
                            return;
                        };
                        let mut probe = wave_probe_sys.lock().unwrap();
                        if wave.current_wave == 0 || frame.frame <= probe.last_frame {
                            return;
                        }
                        probe.last_frame = frame.frame;
                        if probe.waves.last().map(|w| w.wave) != Some(wave.current_wave) {
                            probe.damage = bevy_fixed::fixed_math::FIXED_ZERO;
                            probe.waves.push(WaveStat {
                                wave: wave.current_wave,
                                start: frame.frame,
                                ..Default::default()
                            });
                        }
                        let mut seen = std::collections::BTreeSet::new();
                        let mut damage = probe.damage;
                        let mut downs = 0;
                        for (player, health, downed) in players.iter() {
                            seen.insert(player.handle);
                            if let Some(previous) =
                                probe.health.insert(player.handle, health.current)
                            {
                                if health.current < previous {
                                    damage += previous - health.current;
                                }
                            }
                            if downed && probe.downed.insert(player.handle) {
                                downs += 1;
                            } else if !downed {
                                probe.downed.remove(&player.handle);
                            }
                        }
                        let gone: Vec<usize> = probe
                            .health
                            .keys()
                            .copied()
                            .filter(|handle| !seen.contains(handle))
                            .collect();
                        for handle in &gone {
                            probe.health.remove(handle);
                            probe.downed.remove(handle);
                        }
                        probe.damage = damage;
                        let stat = probe.waves.last_mut().unwrap();
                        stat.enemies = wave.wave_enemy_count;
                        stat.damage = damage.to_num::<f64>().round() as u32;
                        stat.downs += downs;
                        stat.deaths += gone.len() as u32;
                        if stat.spawning.is_none() && wave.phase == game::waves::WavePhase::Spawning
                        {
                            stat.spawning = Some(frame.frame);
                        }
                        if stat.cleared.is_none()
                            && wave.phase == game::waves::WavePhase::WaveComplete
                        {
                            stat.cleared = Some(frame.frame);
                        }
                    },
                );
                if args.iter().any(|arg| arg == "--progress") {
                    app.add_systems(bevy::prelude::Last, print_progress);
                }
            },
            Some(stop_early),
        );
        let wall_seconds = wall_start.elapsed().as_secs_f64();

        let desync = outcome
            .failures
            .iter()
            .any(|f| f.contains("synctest mismatch"));
        any_desync |= desync;

        if let Some(dir) = &save_scenario_dir {
            let path = dir.join(format!("seed_{seed}.ron"));
            std::fs::write(&path, outcome.recorded.to_ron() + "\n")
                .unwrap_or_else(|e| panic!("--save-scenario {path:?} : {e}"));
        }

        let deaths = (bots as u32).saturating_sub(outcome.metrics.players_alive);
        let outcome_end = outcome
            .metrics
            .run_end
            .as_ref()
            .map(|(issue, _)| issue.clone());
        let outcome_softlock = outcome.softlock.is_some()
            && outcome.metrics.frames < max_frames
            && outcome.metrics.run_end.is_none();
        let outcome_reached = until_wave.is_some_and(|n| outcome.metrics.final_wave >= n)
            || until_floor.is_some_and(|n| outcome.metrics.final_floor >= n);
        let wave = outcome.metrics.final_wave;
        let frames_reached = outcome.metrics.frames;
        let kills = outcome.metrics.kills;
        let sim_fps = outcome.metrics.sim_fps;
        let desync_flag = if desync { "  DESYNC" } else { "" };
        let floor = outcome.metrics.final_floor;
        let variants_seen = outcome
            .events
            .iter()
            .filter(|e| e.kind == "variant_spawn")
            .count() as u32;
        // D43 : fin de la graine (issue de la partie, soft-lock, ou objectif / plafond)
        let end = match (&outcome.metrics.run_end, &outcome.softlock) {
            (Some((issue, at)), _) => format!("{issue} f{at}"),
            (None, Some(_)) => "soft-lock".to_string(),
            (None, None) => "objectif".to_string(),
        };
        eprintln!(
            "seed {seed:>6} : vague {wave:>2}  niveau {floor:>2}  frames {frames_reached:>6}  morts {deaths}/{bots}  kills {kills:>3}  variantes {variants_seen:>3}  {sim_fps:>6.1} fps  {wall_seconds:>5.2}s  fin {end}{desync_flag}"
        );

        results.push(SimResult {
            seed,
            wave: outcome.metrics.final_wave,
            floor: outcome.metrics.final_floor,
            frames: outcome.metrics.frames,
            deaths,
            kills: outcome.metrics.kills,
            wall_seconds,
            desync,
            sim_fps: outcome.metrics.sim_fps,
            floor_frames: outcome
                .events
                .iter()
                .filter(|e| e.kind == "floor")
                .map(|e| e.frame)
                .collect(),
            damage_taken: outcome.metrics.damage_taken,
            run_end: outcome.metrics.run_end.clone(),
            dodges: dodges.load(std::sync::atomic::Ordering::Relaxed),
            failures: outcome.failures,
            softlock: outcome.softlock,
            doors_opened: outcome.events.iter().filter(|e| e.kind == "door").count() as u32,
            mechanics: mechanics(&outcome.events),
            variants: variants_seen,
            players_end: players_end.lock().unwrap().clone(),
            waves: std::mem::take(&mut wave_probe.lock().unwrap().waves),
            outcome: classify_outcome(
                desync,
                outcome_end.as_deref(),
                outcome_softlock,
                outcome_reached,
                deaths == bots as u32,
            ),
            rooms: std::mem::take(&mut room_probe.lock().unwrap().floors)
                .into_values()
                .map(|(stat, _)| stat)
                .filter(|stat| stat.total > 0)
                .collect(),
        });
    }

    eprintln!(
        "alacod-sim : {} graine(s) en {:.1}s",
        results.len(),
        run_started.elapsed().as_secs_f64()
    );
    // D88 : atteignabilité des variantes (champions) : part des graines qui en rencontrent au moins une.
    let with_variants = results.iter().filter(|r| r.variants > 0).count();
    eprintln!(
        "alacod-sim : variantes rencontrées : {with_variants}/{} graine(s) ({:.0} %), {} au total, {} au plus sur une graine",
        results.len(),
        100.0 * with_variants as f64 / results.len().max(1) as f64,
        results.iter().map(|r| r.variants).sum::<u32>(),
        results.iter().map(|r| r.variants).max().unwrap_or(0)
    );
    // M2-T16 : résumé par issue (graines de chaque classe)
    let mut by_outcome: std::collections::BTreeMap<&str, Vec<i32>> = Default::default();
    for result in &results {
        by_outcome
            .entry(result.outcome)
            .or_default()
            .push(result.seed);
    }
    for (issue, seeds) in &by_outcome {
        let list: Vec<String> = seeds.iter().take(40).map(|s| s.to_string()).collect();
        eprintln!(
            "alacod-sim : {issue} : {} graine(s) [{}{}]",
            seeds.len(),
            list.join(" "),
            if seeds.len() > 40 { " …" } else { "" }
        );
    }

    let json = serde_json::to_string_pretty(&results).expect("sérialisation JSON");
    match &json_path {
        Some(path) => {
            std::fs::write(path, json + "\n").unwrap_or_else(|e| panic!("--json {path:?} : {e}"))
        }
        None => println!("{json}"),
    }

    if any_desync {
        eprintln!(
            "alacod-sim : au moins un desync (mismatch synctest) : bug de déterminisme à corriger, \
             pas à masquer (voir le scénario sauvé avec --save-scenario et la frame en cause)"
        );
        std::process::exit(1);
    }
}

/// M3-T14 : décomptes par mécanique, lus dans les moments clés de la graine (`scenario::events`).
fn mechanics(
    events: &[scenario::events::GameEvent],
) -> std::collections::BTreeMap<&'static str, u32> {
    let mut counts = std::collections::BTreeMap::new();
    for event in events {
        let key = match event.kind {
            "door_lock" if event.label.starts_with("mur secret") => "mur_secret",
            "door_lock" => "porte_a_cle",
            "bomb" => "bombe",
            "explosion" => "explosion",
            "chest" => "coffre",
            "shelf" if event.label.contains("diable") => "diable",
            "shelf" => "achat",
            "item_pickup" => "objet",
            "boss_phase" => "boss_phase",
            _ => continue,
        };
        *counts.entry(key).or_insert(0) += 1;
    }
    counts
}

fn print_progress(
    frame: bevy::prelude::Res<utils::frame::FrameCount>,
    wave: bevy::prelude::Res<game::waves::WaveState>,
    nav: bevy::prelude::Res<bots::navigation::BotNavigation>,
) {
    if frame.frame > 0 && frame.frame % 1000 == 0 {
        eprintln!(
            "progress frame={} wave={} phase={:?} remaining={} kills={} nav_cells={}",
            frame.frame,
            wave.current_wave,
            wave.phase,
            wave.enemies_to_spawn,
            wave.total_enemies_killed,
            nav.field.costs.len()
        );
    }
}

/// `--seeds A..B` : bornes inclusives (`1..50` = 50 graines). `map_seed` est un `i32`
/// (`game::replay::Scenario`), les graines le sont donc aussi.
fn parse_seed_range(spec: &str) -> (i32, i32) {
    let (from, to) = spec
        .split_once("..")
        .unwrap_or_else(|| panic!("--seeds : attendu <de>..<à>, reçu « {spec} »"));
    let from: i32 = from
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("--seeds : borne basse invalide : « {from} »"));
    let to: i32 = to
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("--seeds : borne haute invalide : « {to} »"));
    assert!(
        from <= to,
        "--seeds : {from}..{to} est vide (borne basse > borne haute)"
    );
    (from, to)
}

#[cfg(test)]
mod tests {
    use super::stop_condition;

    #[test]
    fn until_floor_exige_floors() {
        assert!(stop_condition(None, Some("3"), None).is_err());
        assert_eq!(
            stop_condition(None, Some("3"), Some("trois_niveaux")),
            Ok((None, Some(3)))
        );
        assert_eq!(stop_condition(Some("5"), None, None), Ok((Some(5), None)));
        assert!(stop_condition(None, None, None).is_err());
        assert!(stop_condition(Some("x"), None, None).is_err());
    }
}
