//! Exécution headless d'un scénario, dans le processus courant.

use crate::events::{GameEvent, GameEvents, GameEventsPlugin};
use crate::invariants::InvariantQueries;
use bevy::prelude::*;
use bevy_fixed::fixed_math;
use bevy_ggrs::SyncTestMismatch;
use bots::{BotAssignments, BotsPlugin};
use game::recording::{InputRecorder, RecordedSettings};
use game::{
    args::{GameArgs, GameArgsPlugin},
    character::player::{
        input::{InputSource, ScriptedInputs},
        Player,
    },
    core::CoreSetupPlugin,
    jjrs::PlayerConfig,
    session::CoreSetupConfig,
    state_trace::{StateTraceRecorder, StateTraceRecorderPlugin},
    waves::{WaveDebugEnabled, WaveModeEnabled, WaveState},
};
use map::generation::config::MapGenerationConfig;
use map_ldtk::game::profile::{PlacementErrors, Profile, ProfilePlugin, ScriptedPlacement};
use map_ldtk::{
    game::local::{LdtkGameMap, LdtkLocalGamePlugin},
    plugins::LdtkRoguePlugin,
};
use run::currency::Currency;
use run::{Run, RunStep};
use serde::{Deserialize, Serialize};
use sim_core::modifier::{resolve, Modifiers};
use sim_core::stats::{StatId, Stats};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use utils::frame::FrameCount;

use game::powerups::PowerUpPickup;
use game::powerups_assets::PowerUpsConfigAsset;
use game::replay::{Expectation, RunStepExpectation, Scenario, WaveOverride};
use game::weapons::WeaponRegistry;
use utils::net_id::GgrsNetId;

/// Mismatches de synctest : le premier seulement (GGRS répète ensuite le même à chaque
/// frame et n'avance plus), avec les frames qu'il incrimine.
#[derive(Resource, Default)]
struct SyncTestMismatches {
    messages: Vec<String>,
    frames: Vec<i32>,
}

/// Updates maximum pour charger la map avant la première frame de simulation.
const MAX_LOADING_UPDATES: u32 = 10_000;

/// Frames maximum de la phase du hub d'un scénario qui suit la relance (`HubSetup { stay: false }`).
const HUB_PHASE_MAX_FRAMES: u32 = 2_000;

/// Métriques de performance d'un scénario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    /// Nombre de frames simulées.
    pub frames: u32,
    /// Temps de simulation en secondes (wall clock, seulement la boucle GGRS).
    pub sim_seconds: f64,
    /// Frames simulées par seconde.
    pub sim_fps: f64,
    /// Nombre maximum d'entités rollback à une frame.
    pub entities_max: u32,
    /// Nombre maximum de balles à une frame.
    pub bullets_max: u32,
    /// Nombre maximum d'ennemis à une frame.
    pub enemies_max: u32,
    /// Nombre maximal de joueurs vivants à une frame.
    pub players: u32,
    /// Vague courante à la dernière frame simulée (`WaveState::current_wave`, T2.11 `alacod-sim`).
    #[serde(default)]
    pub final_wave: u32,
    /// Ennemis tués au total à la dernière frame simulée (`WaveState::total_enemies_killed`).
    #[serde(default)]
    pub kills: u32,
    /// Joueurs vivants à la dernière frame simulée (contrairement à `players`, qui est un
    /// maximum sur toute la partie) : `scenario.players.len() - players_alive` = morts.
    #[serde(default)]
    pub players_alive: u32,
    /// Index du niveau à la dernière frame simulée (mode `Floors`, T1.8 :
    /// `run::FloorState::index`, `0` hors `Floors`) : niveaux terminés.
    #[serde(default)]
    pub final_floor: u32,
    /// Destructions de terrain qui ont creusé au moins une case (T1.6, moments clés
    /// `terrain`, une par `world::TerrainDestroyed`) : `bench_cave` en exige au moins 50.
    #[serde(default)]
    pub terrain_destroyed: u32,
    /// Dégâts subis par les joueurs (T1.14) : somme des baisses de santé observées entre deux
    /// updates, arrondie.
    #[serde(default)]
    pub damage_taken: u32,
    /// D43 : issue de la partie si `Run.step` est `Ended` à la dernière frame simulée
    /// (`"Defeat"`, `"Victory"`…), avec sa frame ; absent si la partie est encore en cours.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_end: Option<(String, u32)>,
    /// Coût du rollback (`utils::snapshot_profile`) : `SaveWorld` (checksum + clone des
    /// snapshots), `LoadWorld` (restauration) et `AdvanceWorld` (une frame de simulation),
    /// moyenne et maximum en millisecondes ; octets estimés du plus gros snapshot (porteurs ×
    /// `size_of`, sans le tas) et ses types les plus lourds. En synctest, chaque frame est
    /// sauvegardée puis restaurée `check_distance` fois : `load` est exercé à chaque frame.
    #[serde(default)]
    pub save_ms_avg: f64,
    #[serde(default)]
    pub save_ms_max: f64,
    #[serde(default)]
    pub checksum_ms_avg: f64,
    #[serde(default)]
    pub load_ms_avg: f64,
    #[serde(default)]
    pub load_ms_max: f64,
    #[serde(default)]
    pub advance_ms_avg: f64,
    #[serde(default)]
    pub advance_ms_max: f64,
    #[serde(default)]
    pub snapshot_bytes_max: u64,
    /// `(type, porteurs, octets)` des cinq types les plus lourds du plus gros snapshot.
    #[serde(default)]
    pub snapshot_heaviest: Vec<(String, u32, u64)>,
    /// Pire rollback en ligne : une restauration puis `MAX_PREDICTION_WINDOW` frames
    /// resimulées (`load_ms_avg + 12 × advance_ms_avg`), à comparer aux 16,7 ms d'un tick à
    /// 60 Hz. Plafond optionnel par scénario : `max_rollback_ms` de `tests/budgets.ron`.
    #[serde(default)]
    pub rollback_ms_worst: f64,
}

/// Résultat d'un scénario.
pub struct ScenarioOutcome {
    /// Trace d'état, une ligne par frame (voir `game::state_trace`).
    pub trace: Vec<String>,
    /// Trace détaillée de toutes les frames (hash + détail), si `ALACOD_DUMP_TRACE` était
    /// défini pour ce run (outil de preuve permanent, T1.2, `docs/conventions.md` §10).
    /// `None` sinon. `crates/scenario/tests/scenarios.rs` écrit ces lignes dans
    /// `<dossier>/<scénario>.full` quand elles sont présentes.
    pub full_trace: Option<Vec<String>>,
    /// Attentes non satisfaites, avec leur frame.
    pub failures: Vec<String>,
    /// État en fin de partie, pour écrire ou déboguer un scénario.
    pub summary: String,
    /// Inputs réellement envoyés à GGRS, réenregistrés en scénario.
    pub recorded: Scenario,
    /// Moments clés de la partie.
    pub events: Vec<GameEvent>,
    /// Métriques de performance.
    pub metrics: Metrics,
    /// Compteurs de coups par entité encore présente à la dernière frame (`HitCount`).
    /// Lecture hors simulation, pour le tableau du générateur (une cible dans son arène).
    pub entity_hits: BTreeMap<usize, u32>,
    /// Diagnostic d'un run avec arrêt anticipé qui atteint pourtant son plafond.
    pub softlock: Option<crate::softlock::SoftlockDump>,
}

/// Jeu hors de l'engine (`alacod test` sur un jeu créé par `alacod new`) : `ALACOD_GAME_NAME` et
/// `ALACOD_GAME_DIR` désignent son nom (celui de `game.ron`) et son dossier. Les autres jeux
/// (testbed, jeux de `games/`) se résolvent comme avant.
fn external_game_dir(game: &str) -> Option<std::path::PathBuf> {
    let name = std::env::var("ALACOD_GAME_NAME").ok()?;
    (name == game).then(|| std::env::var("ALACOD_GAME_DIR").ok().map(Into::into))?
}

/// Dossier racine du jeu (`games/<jeu>`), pour `content::load_and_lint` (T1.5).
pub fn game_dir(game: &str) -> std::path::PathBuf {
    external_game_dir(game).unwrap_or_else(|| {
        std::path::PathBuf::from(format!(
            "{}/../../games/{}",
            env!("CARGO_MANIFEST_DIR"),
            game
        ))
    })
}

/// Dossier des assets du jeu.
pub fn assets_dir(game: &str) -> String {
    match external_game_dir(game) {
        Some(dir) => dir.join("assets").to_string_lossy().into_owned(),
        None => format!("{}/../../games/{}/assets", env!("CARGO_MANIFEST_DIR"), game),
    }
}

/// Politique de l'`explorateur` du jeu : `bot.ron` de ses assets (M3-T14, `docs/conventions.md` §79), la politique par défaut
/// s'il n'existe pas. Les campagnes d'`alacod-sim` et les scénarios **générés** (`alacod-gen`) la jouent ; un scénario écrit à la
/// main la pose dans `Scenario::explorer`. Aucun nom de jeu dans le code : tout vient des données du jeu.
pub fn game_bot_policy(game: &str) -> game::replay::ExplorerPolicy {
    let path = format!("{}/bot.ron", assets_dir(game));
    match std::fs::read_to_string(&path) {
        Ok(text) => ron::from_str(&text)
            .unwrap_or_else(|e| panic!("politique de l'explorateur {path} invalide : {e}")),
        Err(_) => Default::default(),
    }
}

/// Configuration de lecture d'un scénario.
pub struct PlayConfig {
    /// Si présent, force la caméra à suivre le joueur avec ce handle GGRS.
    pub follow_handle: Option<usize>,
}

/// Dossier temporaire des profils d'un scénario `profile: true` (vidé à la création).
fn scenario_profile_dir() -> PathBuf {
    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "alacod-scenario-profiles-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// Dossier des profils d'un scénario `profile: true`, avec les profils et achats imposés par
/// `profile_setup` (M2-T12) déjà écrits : la run les lit au lancement, comme le jeu.
fn prepare_profiles(scenario: &Scenario, registry: &content::Registry) -> PathBuf {
    let dir = scenario_profile_dir();
    let Some(setup) = &scenario.profile_setup else {
        return dir;
    };
    let mut profiles: BTreeMap<String, meta::Profile> = BTreeMap::new();
    for seed in &setup.profiles {
        let id = meta::local_id(seed.handle);
        let mut profile = meta::Profile::new(id.clone());
        profile.unlocks.extend(seed.unlocks.iter().cloned());
        profile.stats = seed.stats.clone();
        if seed.essence > 0 {
            profile
                .currencies
                .insert(meta::ESSENCE.into(), seed.essence);
        }
        profiles.insert(id, profile);
    }
    for purchase in &setup.purchases {
        let id = meta::local_id(purchase.handle);
        let profile = profiles
            .entry(id.clone())
            .or_insert_with(|| meta::Profile::new(id.clone()));
        let def = &registry
            .unlocks
            .get(&purchase.unlock)
            .unwrap_or_else(|| panic!("achat : déblocage « {} » inconnu", purchase.unlock))
            .def;
        game::profile::purchase_unlock(profile, &purchase.unlock, def)
            .unwrap_or_else(|e| panic!("achat de « {} » refusé : {e}", purchase.unlock));
    }
    for profile in profiles.values() {
        meta::save(&dir, profile).unwrap_or_else(|e| panic!("profil non écrit : {e}"));
    }
    dir
}

/// Contenu d'un jeu, chargé et linté **une fois par processus** (perf-suite-scenarios) : la
/// suite rejoue 243 scénarios dans le même processus et le chargement (lecture de centaines de
/// `.ron`, lint) coûtait ~0,25 s chacun, ~5 % de la suite. Le contenu ne change pas pendant un
/// test ; seul un chargement sans erreur est mis en cache (une erreur fait échouer le scénario
/// comme avant). Chaque scénario reçoit sa propre copie.
fn load_content(game: &str) -> (content::Registry, content::GameManifest) {
    use std::sync::{Mutex, OnceLock};
    static CACHE: OnceLock<Mutex<BTreeMap<String, (content::Registry, content::GameManifest)>>> =
        OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    if let Some(cached) = cache.lock().unwrap().get(game) {
        return cached.clone();
    }
    let (registry, manifest, content_errors) = content::load_and_lint(&game_dir(game))
        .unwrap_or_else(|e| panic!("scénario « {game} » : game.ron invalide : {e}"));
    assert!(
        content_errors.is_empty(),
        "scénario « {game} » : contenu invalide :\n{content_errors:#?}"
    );
    cache
        .lock()
        .unwrap()
        .insert(game.to_string(), (registry.clone(), manifest.clone()));
    (registry, manifest)
}

/// App de la partie décrite par le scénario (même partie que `zombies`).
/// Avec `headless: false`, la partie est affichée (voir le binaire `play_scenario`).
pub fn build_app(scenario: &Scenario, headless: bool, config: &PlayConfig) -> App {
    let core_plugin = CoreSetupPlugin(CoreSetupConfig {
        app_name: "scenario".into(),
        headless,
        asset_root: Some(assets_dir(&scenario.game)),
        editor: false,
    });

    // Registre de contenu (T1.5) : mêmes règles que `alacod lint`, un contenu invalide
    // fait échouer le test tout de suite plutôt qu'en plein milieu de la simulation.
    let (registry, mut manifest) = load_content(&scenario.game);
    let profile_dir = scenario
        .profile
        .then(|| prepare_profiles(scenario, &registry));

    // Suite de T1.13 : mode de run imposé par le scénario, à la place de `entry.mode`.
    if let Some(mode) = scenario.mode {
        manifest.entry.mode = Some(mode);
    }
    let map_settings = manifest.entry.map.clone();

    let mut app = App::new();
    app.add_plugins(core_plugin.get_default_plugin())
        .add_plugins(GameArgsPlugin(game_args(scenario.players.len())))
        .insert_resource(registry)
        // T2.4, chantier F1 : `game::jjrs::{local, p2p}` résout `RunMode` depuis
        // `GameManifest` (`entry.mode`) au démarrage de session — présente ici comme dans
        // `games/<jeu>/src/main.rs` (voir `game::run_state::resolve_run_mode`).
        .insert_resource(manifest)
        // M2-T0c : un scénario n'écrit aucun profil, sauf `profile: true` (dossier temporaire
        // propre à ce scénario, jamais le profil de l'utilisateur).
        .insert_resource(game::profile::ProfileSettings {
            dir: profile_dir,
        })
        .add_plugins(core_plugin)
        .add_plugins(LdtkRoguePlugin)
        .add_plugins(LdtkLocalGamePlugin(LdtkGameMap {
            map_path: scenario.map.clone(),
            seed: scenario.map_seed,
            settings: map_settings,
        }))
        .insert_resource(WaveModeEnabled(true))
        // Overlay de vagues : du texte d'UI Bevy mesuré à chaque frame (~10 % d'une frame
        // headless), purement visuel ; gardé pour le rendu (vidéos, `play_scenario`).
        .insert_resource(WaveDebugEnabled(!headless))
        // ALACOD_DIAG=1 : trace détaillée, pour nommer le composant qui diverge en synctest
        // ALACOD_DUMP_TRACE=<dossier> : outil de preuve permanent (T1.2), voir
        // `docs/conventions.md` §10 et `crates/scenario/tests/scenarios.rs` (qui écrit
        // `<dossier>/<scénario>.full` depuis `ScenarioOutcome::full_trace`).
        .add_plugins(StateTraceRecorderPlugin {
            full: std::env::var("ALACOD_DIAG").is_ok_and(|v| v == "1"),
            dump: std::env::var("ALACOD_DUMP_TRACE").ok().map(PathBuf::from),
        })
        .add_plugins(GameEventsPlugin)
        // D19 : remplace l'enregistreur vide de `RecordingPlugin` (ajouté par `core_plugin`
        // ci-dessus) par un enregistreur qui connaît les réglages de ce scénario, avant la
        // première frame : `to_scenario` les reporte dans le scénario enregistré.
        .insert_resource(InputRecorder::new(RecordedSettings::from_scenario(scenario)))
        .insert_resource(WeaponOverrides(scenario.weapon_overrides.clone()))
        .add_systems(Update, apply_weapon_overrides)
        .insert_resource(WaveOverrideRes(scenario.wave_overrides.clone()))
        .add_systems(Update, apply_wave_overrides)
        // Power-ups (T2.5, chantier C1 v0) : `drop_chance` forcé (pré-run, comme
        // `apply_wave_overrides`) ; leurs placements scriptés (pendant la simulation) sont
        // dans `map_ldtk::game::profile`, posés plus bas.
        .insert_resource(PowerUpDropChanceOverrideRes(
            scenario.powerup_drop_chance_override,
        ))
        .add_systems(Update, apply_powerup_drop_chance_override)
        // Profil de la partie (placements et réglages des joueurs, `map_ldtk::game::profile`) : mêmes
        // systèmes et même ordre que le bac à sable ; ils vivent dans `GgrsSchedule` (placements) et
        // `Update` (réglages des joueurs).
        .add_plugins(ProfilePlugin(Profile::from_scenario(scenario)))
        // M2-T5 : chance de récompense de salle imposée
        .insert_resource(game::loot::LootOverrides {
            reward_chance: scenario.room_reward_chance_override,
        })
        // M2-T6 : drop de douilles imposé
        .insert_resource(game::shop::ShopOverrides {
            shells_drop: scenario.shop.shells_drop_override,
        })
        .insert_resource(InputSource::Scripted)
        .insert_resource::<ScriptedInputs>(scenario.scripted_inputs())
        // M2-T18 : actions d'UI du scénario (présentation, hors trace) ; absente sans `ui`.
        .insert_resource(game::ui::screens::model::ScreenScript {
            steps: scenario.ui.clone(),
            played_until: None,
        })
        // T2.11 : toujours ajouté (BotAssignments vide ⇒ read_bot_inputs ne fait rien, les
        // scénarios sans bot ne changent pas) ; le mode de base reste Scripted (au-dessus) pour
        // que les joueurs scriptés d'un scénario qui mélange les deux gardent leurs inputs, les
        // joueurs de `bot_assignments` sont remplacés ensuite quel que soit ce mode.
        .add_plugins(BotsPlugin)
        .insert_resource(bot_assignments(scenario))
        .insert_resource(bots::explorer::ExplorerOptions {
            boss_direct: false,
            policy: {
                let mut policy = scenario.explorer.clone().unwrap_or_default();
                policy.shopping |= scenario.shopping;
                policy
            },
        })
        .init_resource::<SyncTestMismatches>()
        .add_observer(|mismatch: On<SyncTestMismatch>, mut log: ResMut<SyncTestMismatches>| {
            let m = mismatch.event();
            if log.messages.is_empty() {
                log.frames = m.mismatched_frames.clone();
                log.messages.push(format!(
                    "frame {}: synctest mismatch (frames {:?}) : l'état rejoué après rollback diffère de l'état sauvegardé",
                    m.current_frame, m.mismatched_frames
                ));
            }
        });

    // Coût du rollback dans les métriques (`Metrics::save_ms_avg`…). Déjà installé par
    // `CoreSetupPlugin` si `ALACOD_SNAPSHOT_PROFILE=1` est dans l'environnement du test.
    if !app.is_plugin_added::<utils::snapshot_profile::SnapshotProfilePlugin>() {
        app.add_plugins(utils::snapshot_profile::SnapshotProfilePlugin);
    }
    // T1.8 : séquence de niveaux imposée par le scénario (mode `Floors`, `Scenario::floors`).
    // T1.9 : horloges et difficulté demandées par le scénario (`game::clock`).
    if let Some(clocks) = &scenario.clocks {
        app.insert_resource(game::clock::ClocksOverride(clocks.clone()));
    }
    if let Some(difficulty) = scenario.difficulty {
        app.insert_resource(game::clock::DifficultyOverride(difficulty));
    }
    if let Some(floors) = &scenario.floors {
        app.insert_resource(game::run_state::FloorsOverride(floors.clone()));
    }
    // M2-T13 : le scénario démarre au hub du manifeste (`entry.hub`).
    if let Some(hub) = &scenario.hub {
        let hub_map = app
            .world()
            .resource::<content::manifest::GameManifest>()
            .entry
            .hub
            .as_ref()
            .map(|hub| hub.map.clone());
        if let Some(hub_map) = hub_map {
            app.insert_resource(game::hub::HubSession {
                hub_map,
                run_map: scenario.map.clone(),
                in_hub: true,
            });
            if hub.stay {
                app.insert_resource(game::hub::HubStay);
            } else {
                // La relance est suivie : `play` et `capture` n'arrêtent la partie qu'une fois la
                // run commencée (`HubPhaseOver`).
                app.init_resource::<HubPhaseOver>()
                    .add_systems(Update, track_hub_phase);
                if !hub.inputs.is_empty() {
                    // m2-full-run : le hub a ses propres pistes, sans bot ; les `players` du
                    // scénario reprennent la main quand la porte relance la session
                    // (`swap_hub_inputs`).
                    app.insert_resource(RunPhaseInputs {
                        scripted: scenario.scripted_inputs(),
                        bots: bot_assignments(scenario),
                    })
                    .insert_resource(scenario.hub_scripted_inputs())
                    .insert_resource(BotAssignments::default())
                    .add_systems(Update, swap_hub_inputs);
                }
            }
        }
    }
    // T1.10 : progression imposée par le scénario.
    if let Some(progression) = &scenario.progression {
        app.insert_resource(game::progression::ProgressionOverride(progression.clone()));
    }

    // Caméra forcée sur un joueur (play_scenario --follow)
    if let Some(handle) = config.follow_handle {
        app.insert_resource(game::ui::hud_model::CameraFollowOverride(handle));
    }

    if headless {
        single_threaded_schedules(&mut app);
    }

    app
}

/// Exécuteur monothread pour tous les schedules d'un scénario headless (perf-suite-scenarios).
///
/// La simulation compte des centaines de petits systèmes par frame : l'exécuteur multithread de
/// Bevy y coûte plus (≈ 1 500 `futex` par frame, réveils de threads) qu'il ne rapporte, et ses
/// threads se disputent les cœurs quand la suite tourne en tranches parallèles. Mesuré sur
/// `idle` : 52 fps sur 4 cœurs, 79 fps sur un seul. `ALACOD_EXECUTOR=multi` rétablit l'exécuteur
/// par défaut. L'ordre d'exécution ne change aucun résultat : `GgrsSchedule` refuse toute
/// ambiguïté d'ordre (`ambiguity_detection: Error`) et les traces de référence le prouvent.
fn single_threaded_schedules(app: &mut App) {
    if std::env::var("ALACOD_EXECUTOR").is_ok_and(|v| v == "multi") {
        return;
    }
    let mut schedules = app
        .world_mut()
        .resource_mut::<bevy::ecs::schedule::Schedules>();
    for (_, schedule) in schedules.iter_mut() {
        schedule.set_executor(bevy::ecs::schedule::SingleThreadedExecutor::new());
    }
}

/// Un profil de bot par joueur (`PlayerScript::bot`), pour `bots::BotsPlugin` (T2.11). Le
/// handle GGRS d'un joueur est son index dans `scenario.players` (voir `ScriptedInputs`,
/// `game_args`, même règle).
fn bot_assignments(scenario: &Scenario) -> BotAssignments {
    let mut assignments = std::collections::BTreeMap::new();
    for (handle, player) in scenario.players.iter().enumerate() {
        if let Some(profile) = player.bot {
            assert!(
                player.inputs.is_empty(),
                "scénario « {} » : joueur {handle} a à la fois des inputs scriptés et un profil de bot ({}) : exclusif (voir PlayerScript::bot)",
                scenario.game,
                profile.name(),
            );
            assignments.insert(handle, profile);
        }
    }
    BotAssignments(assignments)
}

/// Hub suivi (`HubSetup { stay: false }`) : vrai une fois la run commencée (la session relancée
/// repart de la frame 0). `play` et `capture` ne comptent les frames de `Scenario::frames` qu'à
/// partir de là.
#[derive(Resource, Default)]
struct HubPhaseOver(bool);

fn track_hub_phase(
    session: Res<game::hub::HubSession>,
    frame: Res<FrameCount>,
    mut over: ResMut<HubPhaseOver>,
) {
    if !over.0 && !session.in_hub && frame.frame == 0 {
        over.0 = true;
    }
}

/// Inputs de la run d'un scénario qui passe par le hub avec ses propres pistes
/// (`HubSetup::inputs`) : posés quand la porte relance la session.
#[derive(Resource)]
struct RunPhaseInputs {
    scripted: ScriptedInputs,
    bots: BotAssignments,
}

/// Quand `HubSession::in_hub` passe à faux (la sortie du hub est confirmée), les pistes et les
/// bots du scénario remplacent ceux du hub. Les joueurs sont encore ceux du hub ou en cours de
/// recréation : aucune frame de run n'est lue avant cet échange.
fn swap_hub_inputs(
    mut commands: Commands,
    session: Option<Res<game::hub::HubSession>>,
    run_inputs: Option<Res<RunPhaseInputs>>,
) {
    let (Some(session), Some(run_inputs)) = (session, run_inputs) else {
        return;
    };
    if session.in_hub {
        return;
    }
    commands.insert_resource(run_inputs.scripted.clone());
    commands.insert_resource(run_inputs.bots.clone());
    commands.remove_resource::<RunPhaseInputs>();
}

/// Fait avancer le scénario (headless) jusqu'à la frame `frame` et rend l'app, pour
/// inspecter le monde à cet instant (diagnostic).
pub fn run_until(scenario: &Scenario, frame: u32) -> App {
    let config = PlayConfig {
        follow_handle: None,
    };
    let mut app = build_app(scenario, true, &config);
    app.finish();
    app.cleanup();
    for _ in 0..MAX_LOADING_UPDATES + frame {
        app.update();
        if app.world().resource::<FrameCount>().frame >= frame {
            break;
        }
    }
    app
}

/// Joue `scenario` jusqu'à la frame `frames` en appelant `visit(monde, frame)` une fois par frame
/// de simulation atteinte (sonde du générateur, M3-T13 : mesures image par image sans attentes).
pub fn run_visiting<F: FnMut(&mut World, u32)>(scenario: &Scenario, frames: u32, mut visit: F) {
    let config = PlayConfig {
        follow_handle: None,
    };
    let mut app = build_app(scenario, true, &config);
    app.finish();
    app.cleanup();
    let mut last = None;
    for _ in 0..MAX_LOADING_UPDATES + frames {
        app.update();
        let frame = app.world().resource::<FrameCount>().frame;
        if last != Some(frame) {
            last = Some(frame);
            visit(app.world_mut(), frame);
        }
        if frame >= frames {
            break;
        }
    }
}

/// Inventaire du joueur `handle` (copie), pour les sondes du générateur.
pub fn probe_player_inventory(world: &mut World, handle: usize) -> Option<items::Inventory> {
    player_inventory(world, handle).ok().flatten()
}

#[derive(Resource)]
struct WeaponOverrides(Vec<game::replay::WeaponOverride>);

/// Applique les modifications d'armes du scénario sur `WeaponRegistry` dès qu'il est rempli
/// (`game::weapon_assets::sync_weapon_registry`, `PreUpdate` avant `RunGgrsSystems`), avant la
/// création des joueurs (qui copient la config de leurs armes). Une surcharge posée sur le registre
/// n'est jamais réécrasée par la synchro (elle ne recopie une table que vide ou sur `Modified`).
fn apply_weapon_overrides(
    overrides: Res<WeaponOverrides>,
    mut registry: ResMut<WeaponRegistry>,
    mut applied: Local<bool>,
) {
    if *applied || overrides.0.is_empty() {
        return;
    }
    if registry.weapons.is_empty() {
        return; // pas encore rempli par la synchro
    }
    for o in &overrides.0 {
        let weapon = registry
            .weapons
            .get_mut(&o.weapon)
            .unwrap_or_else(|| panic!("weapon_overrides : arme inconnue {}", o.weapon));
        if let Some(friendly_fire) = o.friendly_fire {
            weapon.config.friendly_fire = friendly_fire;
        }
        if let Some(ammo_type) = &o.ammo_type {
            weapon.config.ammo_type = ammo_type.clone();
        }
        for (mode_name, mode) in weapon.config.firing_modes.iter_mut() {
            if o.mode.as_ref().is_some_and(|m| m != mode_name) {
                continue;
            }
            if let game::weapons::MagBulletConfig::Mag {
                mag_size,
                mag_limit,
            } = &mut mode.mag
            {
                if let Some(size) = o.mag_size {
                    *mag_size = size;
                }
                if let Some(limit) = o.mag_limit {
                    *mag_limit = limit;
                }
            }
            if let Some(firing_rate) = o.firing_rate {
                mode.firing_rate = firing_rate;
            }
        }
    }
    *applied = true;
}

#[derive(Resource)]
struct WaveOverrideRes(Option<WaveOverride>);

/// Applique la modification de config des vagues du scénario (T2.1, bench `bench_horde`),
/// même idée que `apply_weapon_overrides` mais sur `waves::config::WaveConfig` : dès que
/// l'asset est chargé, avant que `resolve_balance_system` ne le résolve (`OnEnter(GameLoading)`, E3c : le
/// scénario vise l'**enveloppe** `WaveConfigAsset`, pas `ResolvedBalance`) donc avant la taille de la
/// première vague (que `wave_state_machine_system` lit dans `ResolvedBalance::waves`). Un jeu ne charge
/// qu'une enveloppe de vagues (celle de son dossier `Wave`) : `Assets` la désigne ;
/// sans vagues (aucune enveloppe), la surcharge ne trouve rien, comme avant.
fn apply_wave_overrides(
    overrides: Res<WaveOverrideRes>,
    mut wave_configs: ResMut<Assets<game::waves::assets::WaveConfigAsset>>,
    mut applied: Local<bool>,
) {
    if *applied {
        return;
    }
    let Some(o) = overrides.0.as_ref() else {
        *applied = true;
        return;
    };
    let Some(id) = wave_configs.ids().next() else {
        return;
    };
    let Some(mut asset) = wave_configs.get_mut(id) else {
        return;
    };
    let config = &mut asset.0;
    // F5 (chantier m0-v11) : ces champs sont des `content::expr::NumOrExpr` ; un override
    // de scénario est toujours un entier littéral.
    if let Some(v) = o.max_wave {
        config.max_wave = Some(content::expr::NumOrExpr::Integer(v));
    }
    if let Some(v) = o.min_wave_delay_frames {
        config.min_wave_delay_frames = content::expr::NumOrExpr::Integer(v);
    }
    if let Some(v) = o.base_enemies {
        config.base_enemies = content::expr::NumOrExpr::Integer(v);
    }
    if let Some(v) = o.enemies_per_wave {
        config.enemies_per_wave = content::expr::NumOrExpr::Integer(v);
    }
    if let Some(v) = o.grace_period_frames {
        config.grace_period_frames = content::expr::NumOrExpr::Integer(v);
    }
    if let Some(v) = o.max_concurrent_enemies {
        config.max_concurrent_enemies = content::expr::NumOrExpr::Integer(v);
    }
    if let Some(v) = o.spawn_batch_size {
        config.spawn_batch_size = content::expr::NumOrExpr::Integer(v);
    }
    if let Some(v) = o.spawn_interval_frames {
        config.spawn_interval_frames = content::expr::NumOrExpr::Integer(v);
    }
    *applied = true;
}

#[derive(Resource)]
struct PowerUpDropChanceOverrideRes(Option<fixed_math::Fixed>);

/// Force `PowerUpsConfig::drop_chance` pour ce scénario (T2.5, scénario
/// `powerup_drop_on_kill` : preuve du chemin « drop à la mort » sans dépendre du tirage réel
/// du jeu — voir `game::replay::Scenario::powerup_drop_chance_override`). Vise l'**enveloppe**
/// (`Assets<PowerUpsConfigAsset>`, E3c), pas la table de simulation : même idée que
/// [`apply_wave_overrides`], s'applique dès que l'asset est chargé, bien avant
/// `OnEnter(GameLoading)` qui copie l'asset dans `PowerUpsTable` : la première mort simulée lit donc la
/// valeur forcée. Un jeu ne charge qu'une enveloppe de power-ups : `Assets` la désigne.
fn apply_powerup_drop_chance_override(
    overrides: Res<PowerUpDropChanceOverrideRes>,
    mut powerup_configs: ResMut<Assets<PowerUpsConfigAsset>>,
    mut applied: Local<bool>,
) {
    if *applied {
        return;
    }
    let Some(drop_chance) = overrides.0 else {
        *applied = true;
        return;
    };
    let Some(id) = powerup_configs.ids().next() else {
        return;
    };
    let Some(mut asset) = powerup_configs.get_mut(id) else {
        return;
    };
    let config = &mut asset.0;
    config.drop_chance = drop_chance;
    *applied = true;
}

/// Inventaire (copie) du joueur `handle` : `Ok(None)` s'il n'en porte pas (rien ramassé).
fn player_inventory(world: &mut World, handle: usize) -> Result<Option<items::Inventory>, String> {
    world
        .query::<(&Player, Option<&items::Inventory>)>()
        .iter(world)
        .find(|(player, _)| player.handle == handle)
        .map(|(_, inventory)| inventory.cloned())
        .ok_or_else(|| format!("joueur {handle} absent"))
}

/// Condition d'arrêt anticipé du runner (`alacod-sim`, T2.11) : la simulation s'arrête avant
/// `scenario.frames` dès qu'une condition est atteinte. Contrairement à une simulation qui
/// n'atteint pas `scenario.frames` sans condition d'arrêt (échec, signe habituel d'un problème
/// de chargement), un arrêt anticipé volontaire n'est jamais une failure.
#[derive(Debug, Clone, Copy, Default)]
pub struct StopEarly {
    /// S'arrête dès que `WaveState::current_wave >= until_wave`.
    pub until_wave: Option<u32>,
    /// S'arrête dès que `FloorState::index >= until_floor` (mode `Floors`, T1.14). Active aussi
    /// la détection de soft-lock `Floors` : arrêt avec `softlock` quand ni passage de niveau ni
    /// ennemi en moins depuis [`FLOORS_SOFTLOCK_FRAMES`].
    pub until_floor: Option<u32>,
    /// S'arrête dès qu'aucun joueur n'est vivant (à partir de la première frame simulée).
    pub stop_when_all_players_dead: bool,
    /// D43 : s'arrête dès que la partie est terminée (`Run.step` = `Ended`, défaite ou
    /// victoire) : un dernier joueur à terre en défaite déjà déclarée n'est plus pris pour un
    /// soft-lock pendant son saignement.
    pub stop_when_run_ended: bool,
    /// M3-T14 : un ramassage (objet, consommable, sous) ou une dépense compte comme une progression du
    /// soft-lock `Floors` (`FloorsProgress::stock`). Faux par défaut : les campagnes des autres jeux gardent
    /// leur détecteur. `alacod-sim --pickup-progress` : sans lui, les détours de l'`explorateur` d'`isaac`
    /// (objets et sous aux quatre coins d'un étage nettoyé) dépassent 1 200 frames sans kill sans être bloqués.
    pub pickups_are_progress: bool,
    /// m2-salles-murs-gungeon : entrer dans une case de 32 px jamais visitée du niveau compte comme une progression du
    /// soft-lock `Floors` (`FloorsProgress::explored`) : dans des salles agrandies un étage mesure plus de 2 000 px et la
    /// marche d'une salle de combat à la suivante dépasse [`FLOORS_SOFTLOCK_FRAMES`] sans un coup. Faux par défaut : les
    /// campagnes des autres jeux gardent leur détecteur. `alacod-sim --explored-progress`.
    pub explored_is_progress: bool,
}

/// D44 : état observé pour le soft-lock `Floors`. Il y a progression quand le niveau change, ou
/// qu'un ennemi disparaît, ou que la santé totale des ennemis baisse (dégâts infligés, même
/// sans kill : combat lent contre un boss), ou qu'un joueur change d'état (à terre, relevé,
/// mort), ou, si [`StopEarly::explored_is_progress`], qu'un joueur entre dans une **case de 32 px jamais visitée** de ce niveau
/// (m2-salles-murs-gungeon : sur des salles agrandies, l'étage mesure plus de 2 000 px et la marche d'une
/// salle de combat à la suivante peut dépasser la fenêtre de [`FLOORS_SOFTLOCK_FRAMES`] sans un seul
/// coup ; un joueur qui explore encore n'est pas bloqué, un bot qui tourne en rond l'est : les cases
/// visitées finissent par ne plus grandir).
#[derive(Debug, Clone, PartialEq)]
pub struct FloorsProgress {
    pub floor: u32,
    /// Nombre de cases de 32 px (par niveau) déjà visitées par un joueur (0 si [`StopEarly::explored_is_progress`] est faux).
    pub explored: usize,
    pub enemies: usize,
    pub enemy_health: fixed_math::Fixed,
    /// Par joueur présent (handle), à terre ou non ; un joueur mort disparaît de la liste.
    pub players: Vec<(usize, bool)>,
    /// M3-T14 : empreinte de l'inventaire et du solde des joueurs (0 si [`StopEarly::pickups_are_progress`]
    /// est faux) ; tout changement est une progression.
    pub stock: u64,
}

impl FloorsProgress {
    /// `true` si `now` marque une progression par rapport à `self` (voir le type).
    pub fn progressed(&self, now: &FloorsProgress) -> bool {
        now.floor != self.floor
            || now.explored > self.explored
            || now.enemies < self.enemies
            || now.enemy_health < self.enemy_health
            || now.players != self.players
            || now.stock != self.stock
    }
}

/// Soft-lock `Floors` (T1.14) : frames sans passage de niveau ni ennemi en moins.
pub const FLOORS_SOFTLOCK_FRAMES: u32 = 1200;

/// Joue le scénario jusqu'à `scenario.frames` et vérifie ses attentes, après avoir appliqué
/// `configure` à l'app (pour les tests qui ajoutent un système ou une ressource).
pub fn run_with<F: FnOnce(&mut App)>(scenario: &Scenario, configure: F) -> ScenarioOutcome {
    run_with_options(scenario, configure, None)
}

/// Comme [`run_with`], avec une condition d'arrêt anticipé optionnelle (`alacod-sim`, T2.11).
pub fn run_with_options<F: FnOnce(&mut App)>(
    scenario: &Scenario,
    configure: F,
    stop_early: Option<StopEarly>,
) -> ScenarioOutcome {
    let config = PlayConfig {
        follow_handle: None,
    };
    let mut app = build_app(scenario, true, &config);
    configure(&mut app);
    app.finish();
    app.cleanup();

    // Hub suivi (m2-full-run) : la session démarre au hub, la porte la relance en run.
    let hub_follow = scenario.hub.as_ref().is_some_and(|hub| !hub.stay)
        && app.world().contains_resource::<game::hub::HubSession>();
    let is_hub_expectation = |e: &Expectation| {
        matches!(
            e,
            Expectation::HubExit { .. } | Expectation::HubVisit { .. }
        )
    };
    let mut hub_pending: Vec<&Expectation> = scenario
        .expect
        .iter()
        .filter(|e| hub_follow && is_hub_expectation(e))
        .collect();
    hub_pending.sort_by_key(|e| e.at_frame());

    // Séparer les attentes ponctuelles des attentes continues (NoDamageBetween, et T1.4 :
    // EnemyContactBefore, EnemyNeverInWall)
    let mut pending: Vec<&Expectation> = scenario
        .expect
        .iter()
        .filter(|e| !is_continuous(e) && !(hub_follow && is_hub_expectation(e)))
        .collect();
    pending.sort_by_key(|e| e.at_frame());

    let no_damage_between: Vec<&Expectation> = scenario
        .expect
        .iter()
        .filter(|e| matches!(e, Expectation::NoDamageBetween { .. }))
        .collect();

    // T1.4 : diagnostics de navigation (continus). `contact_met[i]` : la portée de mêlée a
    // été atteinte ; `wall_reported[i]` : un chevauchement de mur a déjà été rapporté.
    let navigation: Vec<&Expectation> = scenario
        .expect
        .iter()
        .filter(|e| {
            matches!(
                e,
                Expectation::EnemyContactBefore { .. } | Expectation::EnemyNeverInWall { .. }
            )
        })
        .collect();
    let mut contact_met = vec![false; navigation.len()];
    let mut wall_reported = vec![false; navigation.len()];

    let mut failures = Vec::new();
    // Suite de T1.13 : `mode: Floors` sans séquence imposée.
    if scenario.mode == Some(content::EntryMode::Floors) && scenario.floors.is_none() {
        failures
            .push("mode: Floors exige floors (id d'une séquence du dossier Floors)".to_string());
    }
    // T1.13 : un placement après la dernière frame ne serait jamais appliqué.
    for (index, placement) in scenario.characters.iter().enumerate() {
        if placement.at_frame >= scenario.frames {
            failures.push(format!(
                "placement {index} (« {} ») : at_frame {} >= frames {}",
                placement.character, placement.at_frame, scenario.frames
            ));
        }
    }
    let mut invariants = InvariantQueries::new(app.world_mut());
    // Dernière santé vue par attente `NoDamageBetween`, clé (handle, from_frame).
    let mut dernieres_santes: BTreeMap<(usize, u32), fixed_math::Fixed> = BTreeMap::new();

    // Métriques : le chrono part au premier update simulé (le chargement de la map n'est pas
    // compté) ; les compteurs d'entités sont lus entre deux updates, jamais dans la simulation.
    let mut sim_start: Option<std::time::Instant> = None;
    let mut sim_elapsed = 0.0f64;
    let mut entities_max = 0u32;
    let mut bullets_max = 0u32;
    let mut enemies_max = 0u32;
    let mut players_count = 0u32;
    let mut q_rollback = app
        .world_mut()
        .query_filtered::<(), With<bevy_ggrs::Rollback>>();
    let mut q_bullets = app
        .world_mut()
        .query_filtered::<(), With<game::weapons::Bullet>>();
    let mut q_enemies = app
        .world_mut()
        .query_filtered::<(), With<game::character::enemy::Enemy>>();
    let mut q_players = app.world_mut().query_filtered::<(), With<Player>>();
    // T1.14 : dégâts subis (somme des baisses de santé des joueurs, lue entre deux updates)
    let mut q_player_health = app
        .world_mut()
        .query::<(&Player, &game::character::health::Health)>();
    let mut player_health: BTreeMap<usize, fixed_math::Fixed> = BTreeMap::new();
    let mut damage_taken = fixed_math::FIXED_ZERO;
    // T1.14 + D44 : progrès en `Floors` pour le soft-lock (état observé, frame du dernier
    // progrès) ; `None` avant la première observation.
    let mut floors_progress: Option<(FloorsProgress, u32)> = None;
    let mut q_enemy_health = app
        .world_mut()
        .query_filtered::<&game::character::health::Health, With<game::character::enemy::Enemy>>();
    let mut q_player_state = app
        .world_mut()
        .query::<(&Player, Has<combat::downed::Downed>)>();
    let mut q_player_stock =
        app.world_mut()
            .query::<(&Player, Option<&items::Inventory>, Option<&run::Currency>)>();
    let mut q_player_cells = app
        .world_mut()
        .query::<(&Player, &bevy_fixed::fixed_math::FixedTransform3D)>();
    let mut explored: BTreeSet<(u32, i32, i32)> = BTreeSet::new();
    let mut floors_softlocked = false;

    // Hub suivi (`HubSetup { stay: false }`) : la phase du hub s'ajoute au budget d'updates.
    let mut hub_phase = hub_follow;
    let max_updates = MAX_LOADING_UPDATES
        + scenario.frames
        + if hub_follow {
            MAX_LOADING_UPDATES + HUB_PHASE_MAX_FRAMES
        } else {
            0
        };
    let mut frame = 0;
    let mut stopped_early = false;
    let mut previous_snapshot = None;
    // ALACOD_PROFILE=1 : temps cumulé par poste du runner (mesure, jamais dans la simulation).
    let profile = std::env::var("ALACOD_PROFILE").is_ok_and(|v| v == "1");
    let mut t_update = 0.0f64;
    let mut t_metrics = 0.0f64;
    let mut t_invariants = 0.0f64;
    let mut t_expect = 0.0f64;
    for _ in 0..max_updates {
        let before = app.world().resource::<FrameCount>().frame;
        if before > 0 && sim_start.is_none() {
            sim_start = Some(std::time::Instant::now());
        }
        let t0 = std::time::Instant::now();
        app.update();
        if profile && before > 0 {
            t_update += t0.elapsed().as_secs_f64();
        }
        frame = app.world().resource::<FrameCount>().frame;
        if hub_phase && frame == 0 && !app.world().resource::<game::hub::HubSession>().in_hub {
            // La porte du hub a relancé la session : la run commence (frames de 0). Le chrono de
            // simulation repart avec elle (le hub n'entre pas dans `sim_fps`).
            hub_phase = false;
            sim_start = None;
            sim_elapsed = 0.0;
        }
        if let Some(start) = sim_start {
            sim_elapsed = start.elapsed().as_secs_f64();
        }

        let t1 = std::time::Instant::now();
        if frame > 0 {
            entities_max = entities_max.max(q_rollback.iter(app.world()).count() as u32);
            bullets_max = bullets_max.max(q_bullets.iter(app.world()).count() as u32);
            enemies_max = enemies_max.max(q_enemies.iter(app.world()).count() as u32);
            players_count = players_count.max(q_players.iter(app.world()).count() as u32);
            for (player, health) in q_player_health.iter(app.world()) {
                if let Some(previous) = player_health.insert(player.handle, health.current) {
                    if health.current < previous {
                        damage_taken += previous - health.current;
                    }
                }
            }

            t_metrics += t1.elapsed().as_secs_f64();
            let t2 = std::time::Instant::now();
            failures.extend(invariants.check(app.world_mut(), &scenario.invariants, frame));
            t_invariants += t2.elapsed().as_secs_f64();

            for expectation in &no_damage_between {
                if let Expectation::NoDamageBetween {
                    handle,
                    from_frame,
                    to_frame,
                } = expectation
                {
                    if frame >= *from_frame && frame <= *to_frame {
                        if let Err(reason) = check_no_damage(
                            app.world_mut(),
                            &mut dernieres_santes,
                            *handle,
                            *from_frame,
                        ) {
                            failures.push(format!("frame {frame}: {expectation:?} : {reason}"));
                        }
                    }
                }
            }

            for (index, expectation) in navigation.iter().enumerate() {
                match expectation {
                    Expectation::EnemyContactBefore { entity, frames } => {
                        if !contact_met[index]
                            && frame <= *frames
                            && enemy_in_melee_range(app.world_mut(), entity)
                        {
                            contact_met[index] = true;
                        }
                    }
                    Expectation::EnemyNeverInWall { entity, from, to } => {
                        if !wall_reported[index]
                            && frame >= *from
                            && frame <= *to
                            && enemy_in_wall(app.world_mut(), entity)
                        {
                            wall_reported[index] = true;
                            failures.push(format!(
                                "frame {frame}: {expectation:?} : le collider de l'ennemi chevauche un mur"
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }

        if profile {
            t_expect += t1.elapsed().as_secs_f64();
        }
        if !app
            .world()
            .resource::<SyncTestMismatches>()
            .messages
            .is_empty()
        {
            break; // la session synctest n'avance plus après un mismatch
        }

        // Attentes du hub (phase hub d'un scénario qui suit la relance) ou de la run : les frames
        // repartent de 0 à la run, chaque liste n'est lue que pendant sa phase.
        let due = if hub_phase {
            &mut hub_pending
        } else {
            &mut pending
        };
        while due.first().is_some_and(|e| e.at_frame() <= frame) {
            let expectation = due.remove(0);
            if let Err(reason) = check(app.world_mut(), expectation) {
                failures.push(format!("frame {frame}: {expectation:?} : {reason}"));
            }
        }

        if frame > 0 {
            if let Some(stop) = stop_early {
                let wave_reached = stop.until_wave.is_some_and(|until_wave| {
                    app.world().resource::<WaveState>().current_wave >= until_wave
                });
                let floor_index = app.world().resource::<run::FloorState>().index;
                let floor_reached = stop.until_floor.is_some_and(|n| floor_index >= n);
                let all_dead =
                    stop.stop_when_all_players_dead && q_players.iter(app.world()).count() == 0;
                let run_ended = stop.stop_when_run_ended
                    && matches!(app.world().resource::<Run>().step, RunStep::Ended { .. });
                if wave_reached || floor_reached || all_dead || run_ended {
                    stopped_early = true;
                } else if stop.until_floor.is_some() {
                    // Soft-lock `Floors` : aucune progression (voir `FloorsProgress`)
                    let world = app.world();
                    let mut players: Vec<(usize, bool)> = q_player_state
                        .iter(world)
                        .map(|(player, downed)| (player.handle, downed))
                        .collect();
                    players.sort_unstable();
                    if stop.explored_is_progress {
                        for (_, transform) in q_player_cells.iter(world) {
                            let at = transform.translation.truncate();
                            explored.insert((
                                floor_index,
                                (at.x.to_num::<i32>()).div_euclid(32),
                                (at.y.to_num::<i32>()).div_euclid(32),
                            ));
                        }
                    }
                    let now = FloorsProgress {
                        floor: floor_index,
                        explored: explored.len(),
                        enemies: q_enemies.iter(world).count(),
                        enemy_health: q_enemy_health
                            .iter(world)
                            .fold(fixed_math::FIXED_ZERO, |sum, h| {
                                sum.saturating_add(h.current)
                            }),
                        players,
                        stock: if stop.pickups_are_progress {
                            // Empreinte déterministe (joueurs triés par handle) de l'inventaire et du solde.
                            let mut held: Vec<(usize, u64)> = q_player_stock
                                .iter(world)
                                .map(|(player, inventory, currency)| {
                                    let mut sum = u64::from(currency.map_or(0, |c| c.0));
                                    if let Some(inventory) = inventory {
                                        sum = sum
                                            .wrapping_mul(1_000_003)
                                            .wrapping_add(inventory.passives.len() as u64);
                                        sum = sum
                                            .wrapping_mul(1_000_003)
                                            .wrapping_add(u64::from(inventory.active.is_some()));
                                        for (id, count) in &inventory.consumables {
                                            sum = sum
                                                .wrapping_mul(1_000_003)
                                                .wrapping_add(u64::from(*count))
                                                .wrapping_add(id.len() as u64);
                                        }
                                    }
                                    (player.handle, sum)
                                })
                                .collect();
                            held.sort_unstable();
                            held.iter().fold(0u64, |acc, (handle, sum)| {
                                acc.wrapping_mul(1_000_003)
                                    .wrapping_add(*handle as u64)
                                    .wrapping_add(sum.wrapping_mul(7919))
                            })
                        } else {
                            0
                        },
                    };
                    match &mut floors_progress {
                        Some((last, since)) if !last.progressed(&now) => {
                            // Pas de progression : on garde l'état de référence, mais une
                            // santé qui remonte (régénération) devient la nouvelle référence
                            last.enemy_health = last.enemy_health.max(now.enemy_health);
                            let stalled = frame.saturating_sub(*since);
                            if stalled == FLOORS_SOFTLOCK_FRAMES / 2 {
                                previous_snapshot =
                                    Some(crate::softlock::snapshot(app.world_mut()));
                            }
                            if stalled >= FLOORS_SOFTLOCK_FRAMES {
                                floors_softlocked = true;
                            }
                        }
                        _ => floors_progress = Some((now, frame)),
                    }
                }
            }
        }

        if stop_early.is_some()
            && frame > 0
            && frame >= scenario.frames.saturating_sub(600)
            && previous_snapshot.is_none()
            && !stopped_early
            && frame < scenario.frames
        {
            previous_snapshot = Some(crate::softlock::snapshot(app.world_mut()));
        }

        if stopped_early || floors_softlocked || (frame >= scenario.frames && !hub_phase) {
            break;
        }
    }

    if profile {
        eprintln!(
            "profil runner : update={t_update:.2}s métriques={t_metrics:.2}s invariants={t_invariants:.2}s \
             hors-update(total)={t_expect:.2}s"
        );
    }
    let sim_fps = if sim_elapsed > 0.0 {
        frame as f64 / sim_elapsed
    } else {
        0.0
    };

    if hub_follow {
        if hub_phase {
            failures.push(format!(
                "la porte du hub n'a jamais relancé la run (frame {frame} au hub)"
            ));
        }
        failures.extend(hub_pending.iter().map(|expectation| {
            format!("{expectation:?} : jamais évaluée, la phase du hub s'est terminée avant")
        }));
        if scenario.expect.iter().any(is_continuous) {
            failures.push(
                "attentes continues (NoDamageBetween, EnemyContactBefore, EnemyNeverInWall) \
                 non prises en charge avec un hub suivi (`hub: Some((stay: false))`)"
                    .into(),
            );
        }
        // `apply_player_overrides` ne s'applique qu'à la première session : les joueurs de la run
        // lancée par la porte n'en recevraient aucun, mieux vaut refuser que mal jouer.
        if scenario
            .players
            .iter()
            .any(|p| !map_ldtk::game::profile::PlayerProfile::from_script(p).is_empty())
        {
            failures.push(
                "réglages de joueur (tags, weapon, currency, items, modifiers, mutations, dash, \
                 bomb, aim) non pris en charge avec un hub suivi (`hub: Some((stay: false))`)"
                    .into(),
            );
        }
    }
    if frame < scenario.frames && !stopped_early && !floors_softlocked {
        failures.push(format!(
            "la simulation n'a atteint que la frame {frame} sur {} (map pas chargée ?)",
            scenario.frames
        ));
    }

    // T1.13 : placements refusés.
    failures.extend(
        app.world()
            .resource::<PlacementErrors>()
            .0
            .values()
            .cloned(),
    );

    // Ajoute les mismatches de synctest aux failures
    let synctest_mismatches = app.world().resource::<SyncTestMismatches>();
    failures.extend(synctest_mismatches.messages.iter().cloned());
    if !synctest_mismatches.frames.is_empty() {
        let report = app
            .world()
            .resource::<StateTraceRecorder>()
            .report_for_frames(&synctest_mismatches.frames);
        failures.push(format!("trace des frames divergentes :\n{report}"));
    }

    for (index, expectation) in navigation.iter().enumerate() {
        if let Expectation::EnemyContactBefore { frames, .. } = expectation {
            if !contact_met[index] {
                failures.push(format!(
                    "frame {frames}: {expectation:?} : jamais à portée de mêlée d'un joueur"
                ));
            }
        }
    }

    let recorder = app.world().resource::<StateTraceRecorder>();
    let trace = recorder
        .lines_until(scenario.frames)
        .map(str::to_string)
        .collect();
    let full_trace = recorder
        .dump_lines_until(scenario.frames)
        .map(|lines| lines.map(str::to_string).collect());

    let summary = summarize(app.world_mut(), frame);
    // Les réglages de simulation (jeu, surcharges, power-ups) sont dans l'enregistreur depuis
    // sa construction (D19, `RecordedSettings::from_scenario` ci-dessus) : plus de recopie ici.
    let recorded = app
        .world()
        .resource::<InputRecorder>()
        .to_scenario(app.world().get_resource::<MapGenerationConfig>());

    let events = app.world().resource::<GameEvents>().events.clone();

    let wave_state = app.world().resource::<WaveState>();
    let final_wave = wave_state.current_wave;
    let kills = wave_state.total_enemies_killed;
    let players_alive = q_players.iter(app.world()).count() as u32;
    let final_floor = app.world().resource::<run::FloorState>().index;
    let profile = app
        .world()
        .resource::<utils::snapshot_profile::SnapshotProfile>();

    // `ALACOD_SYSTEM_PROFILE=1` (diagnostic, M3-T15) : temps par `RollbackSystemSet` de ce scénario.
    if let Some(timings) = app
        .world()
        .get_resource::<utils::system_profile::SetTimings>()
    {
        for row in timings.report() {
            println!("profil sets : {row}");
        }
    }

    let metrics = Metrics {
        frames: frame,
        sim_seconds: sim_elapsed,
        sim_fps,
        entities_max,
        bullets_max,
        enemies_max,
        players: players_count,
        final_wave,
        kills,
        players_alive,
        final_floor,
        terrain_destroyed: events.iter().filter(|e| e.kind == "terrain").count() as u32,
        damage_taken: damage_taken.to_num::<f64>().round() as u32,
        run_end: match &app.world().resource::<Run>().step {
            RunStep::Ended { at_frame, outcome } => Some((format!("{outcome:?}"), *at_frame)),
            _ => None,
        },
        save_ms_avg: profile.save.avg_ms(),
        save_ms_max: profile.save.max_ms(),
        checksum_ms_avg: profile.checksum.avg_ms(),
        load_ms_avg: profile.load.avg_ms(),
        load_ms_max: profile.load.max_ms(),
        advance_ms_avg: profile.advance.avg_ms(),
        advance_ms_max: profile.advance.max_ms(),
        snapshot_bytes_max: profile.snapshot_bytes_max as u64,
        snapshot_heaviest: profile
            .heaviest(5)
            .map(|t| (t.name.to_string(), t.count as u32, t.bytes as u64))
            .collect(),
        rollback_ms_worst: profile.load.avg_ms()
            + game::jjrs::p2p::MAX_PREDICTION_WINDOW as f64 * profile.advance.avg_ms(),
    };

    let world = app.world_mut();
    let entity_hits = world
        .query_filtered::<(&GgrsNetId, &game::character::health::HitCount), With<bevy_ggrs::Rollback>>()
        .iter(world)
        .map(|(net_id, hits)| (net_id.0, hits.0))
        .collect();

    let softlock = if floors_softlocked
        || (stop_early.is_some() && !stopped_early && frame >= scenario.frames)
    {
        Some(crate::softlock::SoftlockDump::new(
            previous_snapshot,
            crate::softlock::snapshot(world),
        ))
    } else {
        None
    };

    ScenarioOutcome {
        softlock,
        entity_hits,
        trace,
        full_trace,
        failures,
        summary,
        recorded,
        events,
        metrics,
    }
}

/// Joue le scénario jusqu'à `scenario.frames` et vérifie ses attentes.
pub fn run(scenario: &Scenario) -> ScenarioOutcome {
    run_with(scenario, |_| {})
}

fn game_args(player_count: usize) -> GameArgs {
    GameArgs {
        seed: None,
        check_distance: game::args::check_distance_from_env(),
        local_port: 0,
        number_player: player_count,
        players: (0..player_count)
            .map(|i| PlayerConfig {
                name: format!("Player {}", i + 1),
                pubkey: "local".into(),
                is_local: true,
            })
            .collect(),
        spectators: vec![],
        matchbox: String::new(),
        allumette: String::new(),
        lobby: String::new(),
        cid: "scenario".into(),
        debug_ai: false,
        sandbox: None,
        telemetry: false,
        telemetry_url: String::new(),
        telemetry_auth: String::new(),
    }
}

fn check(world: &mut World, expectation: &Expectation) -> Result<(), String> {
    match expectation {
        Expectation::PlayerAlive { handle, .. } => {
            if player_alive(world, *handle) {
                Ok(())
            } else {
                Err("joueur mort".into())
            }
        }
        Expectation::PlayerDead { handle, .. } => {
            if player_alive(world, *handle) {
                Err("joueur vivant".into())
            } else {
                Ok(())
            }
        }
        Expectation::WaveAtLeast { wave, .. } => {
            let current = world.resource::<WaveState>().current_wave;
            if current >= *wave {
                Ok(())
            } else {
                Err(format!("vague {current}"))
            }
        }
        Expectation::ActiveWeapon {
            handle,
            weapon,
            mode,
            ..
        } => {
            let Some((name, active_mode, _)) = active_weapon(world, *handle) else {
                return Err("joueur absent ou sans arme".into());
            };
            if name != *weapon || mode.as_ref().is_some_and(|m| *m != active_mode) {
                Err(format!("arme active {name} (mode {active_mode})"))
            } else {
                Ok(())
            }
        }
        Expectation::Ammo { handle, ammo, .. } => {
            let Some((name, mode, current)) = active_weapon(world, *handle) else {
                return Err("joueur absent ou sans arme".into());
            };
            if current == *ammo {
                Ok(())
            } else {
                Err(format!("{current} balles dans {name} ({mode})"))
            }
        }
        Expectation::AmmoReserve {
            handle,
            ammo_type,
            amount,
            ..
        } => {
            let Some(current) = player_ammo_reserve(world, *handle, ammo_type) else {
                return Err("joueur absent".into());
            };
            if current == *amount {
                Ok(())
            } else {
                Err(format!(
                    "réserve {ammo_type:?} = {current} (attendu {amount})"
                ))
            }
        }
        Expectation::WeaponPickups { min, max, .. } => {
            // Armes au sol seulement : une arme murale (T2.3, `price: Some(..)`) est un
            // `WeaponPickup` permanent posé par la carte, pas une arme lâchée par un joueur
            // (T2.6 : `test_map.ldtk` en pose cinq, `drop_pickup_swap` compte de 0 à 1).
            let count = world
                .query::<&game::weapons::WeaponPickup>()
                .iter(world)
                .filter(|pickup| pickup.price.is_none())
                .count() as u32;
            if let Some(min_val) = min {
                if count < *min_val {
                    return Err(format!("{count} armes au sol < min {min_val}"));
                }
            }
            if let Some(max_val) = max {
                if count > *max_val {
                    return Err(format!("{count} armes au sol > max {max_val}"));
                }
            }
            Ok(())
        }
        Expectation::PowerUpPickups { min, max, .. } => {
            let count = world.query::<&PowerUpPickup>().iter(world).count() as u32;
            if let Some(min_val) = min {
                if count < *min_val {
                    return Err(format!("{count} power-ups au sol < min {min_val}"));
                }
            }
            if let Some(max_val) = max {
                if count > *max_val {
                    return Err(format!("{count} power-ups au sol > max {max_val}"));
                }
            }
            Ok(())
        }
        Expectation::BulletsInside {
            x_min,
            x_max,
            y_min,
            y_max,
            ..
        } => {
            let outside: Vec<(f32, f32)> = world
                .query_filtered::<&bevy_fixed::fixed_math::FixedTransform3D, With<game::weapons::Bullet>>()
                .iter(world)
                .map(|t| (t.translation.x.to_num::<f32>(), t.translation.y.to_num::<f32>()))
                .filter(|(x, y)| x < x_min || x > x_max || y < y_min || y > y_max)
                .collect();
            if outside.is_empty() {
                Ok(())
            } else {
                Err(format!("balles hors zone : {outside:?}"))
            }
        }
        Expectation::WindowHealth { window, health, .. } => {
            let current = world
                .query::<(
                    &utils::net_id::GgrsNetId,
                    &map::game::entity::map::window::WindowHealth,
                )>()
                .iter(world)
                .find(|(id, _)| id.0 == *window)
                .map(|(_, h)| h.current);
            match current {
                Some(current) if current == *health => Ok(()),
                Some(current) => Err(format!("santé {current}")),
                None => Err("fenêtre absente".into()),
            }
        }
        Expectation::DoorsOpenAtLeast { doors, .. } => {
            let open = world
                .query_filtered::<Has<game::collider::Collider>, With<map::game::entity::map::door::DoorComponent>>()
                .iter(world)
                .filter(|closed| !closed)
                .count() as u32;
            if open >= *doors {
                Ok(())
            } else {
                Err(format!("{open} portes ouvertes"))
            }
        }
        Expectation::PlayerPosition {
            handle,
            x,
            y,
            tolerance,
            ..
        } => {
            let Some((px, py)) = world
                .query::<(&Player, &bevy_fixed::fixed_math::FixedTransform3D)>()
                .iter(world)
                .find(|(player, _)| player.handle == *handle)
                .map(|(_, t)| {
                    (
                        t.translation.x.to_num::<f32>(),
                        t.translation.y.to_num::<f32>(),
                    )
                })
            else {
                return Err("joueur absent".into());
            };
            if (px - x).abs() <= *tolerance && (py - y).abs() <= *tolerance {
                Ok(())
            } else {
                Err(format!("joueur en ({px:.1}, {py:.1})"))
            }
        }
        Expectation::WindowsBrokenAtLeast { windows, .. } => {
            let broken = windows_broken(world);
            if broken >= *windows {
                Ok(())
            } else {
                Err(format!("{broken} fenêtres cassées"))
            }
        }
        Expectation::KillsAtLeast { kills, .. } => {
            let killed = world.resource::<WaveState>().total_enemies_killed;
            if killed >= *kills {
                Ok(())
            } else {
                Err(format!("{killed} ennemis tués"))
            }
        }
        Expectation::Health {
            handle, min, max, ..
        } => {
            let Some(health_fixed) = player_health(world, *handle) else {
                return Err("joueur absent".into());
            };
            let min_fixed = min.map(fixed_math::Fixed::from_num);
            let max_fixed = max.map(fixed_math::Fixed::from_num);

            if let Some(min_val) = min_fixed {
                if health_fixed < min_val {
                    return Err(format!("santé {} < min {}", health_fixed, min_val));
                }
            }
            if let Some(max_val) = max_fixed {
                if health_fixed > max_val {
                    return Err(format!("santé {} > max {}", health_fixed, max_val));
                }
            }
            Ok(())
        }
        Expectation::EntityHealth {
            net_id, min, max, ..
        } => {
            let Some(health_fixed) = entity_health(world, *net_id) else {
                return Err("entité absente ou sans santé".into());
            };
            let min_fixed = min.map(fixed_math::Fixed::from_num);
            let max_fixed = max.map(fixed_math::Fixed::from_num);

            if let Some(min_val) = min_fixed {
                if health_fixed < min_val {
                    return Err(format!("santé {} < min {}", health_fixed, min_val));
                }
            }
            if let Some(max_val) = max_fixed {
                if health_fixed > max_val {
                    return Err(format!("santé {} > max {}", health_fixed, max_val));
                }
            }
            Ok(())
        }
        Expectation::EntityHits {
            net_id, min, max, ..
        } => {
            let Some(hits) = entity_hit_count(world, *net_id) else {
                return Err("entité absente ou sans compteur de coups (HitCount)".into());
            };
            if hits < *min {
                return Err(format!("{hits} coups reçus < min {min}"));
            }
            if let Some(max_val) = max {
                if hits > *max_val {
                    return Err(format!("{hits} coups reçus > max {max_val}"));
                }
            }
            Ok(())
        }
        Expectation::BulletCount {
            count,
            projectile,
            team,
            ..
        } => {
            let alive = bullet_count(world, projectile.as_deref(), *team);
            if alive == *count {
                Ok(())
            } else {
                Err(format!("{alive} projectiles vivants, {count} attendus"))
            }
        }
        Expectation::HitsAtLeast { entity, hits, .. } => {
            let found = match entity {
                game::replay::EntityRef::NetId(net_id) => entity_hit_count(world, *net_id),
                game::replay::EntityRef::Target => target_hit_count(world),
                placed @ game::replay::EntityRef::Placed(_) => resolve_entity(world, placed)
                    .and_then(|entity| world.get::<GgrsNetId>(entity).map(|id| id.0))
                    .and_then(|net_id| entity_hit_count(world, net_id)),
            };
            let Some(found) = found else {
                return Err("entité absente ou sans compteur de coups (HitCount)".into());
            };
            if found >= *hits {
                Ok(())
            } else {
                Err(format!("{found} coups reçus < {hits}"))
            }
        }
        Expectation::NoDamageBetween { .. }
        | Expectation::EnemyContactBefore { .. }
        | Expectation::EnemyNeverInWall { .. } => {
            // Géré dans la boucle principale, pas dans check()
            Ok(())
        }
        Expectation::EnemyState {
            entity, behavior, ..
        } => match enemy_rule(world, entity) {
            Err(reason) => Err(reason),
            Ok(Some(found)) if found == *behavior => Ok(()),
            Ok(found) => Err(format!(
                "behavior retenu {}, {behavior} attendu",
                found.as_deref().unwrap_or("aucun")
            )),
        },
        Expectation::SquadSize {
            squad, min, max, ..
        } => {
            use game::character::enemy::ai::state::EnemyBehaviors;
            let count = world
                .query::<&EnemyBehaviors>()
                .iter(world)
                .filter(|rules| rules.squad.as_deref() == Some(squad.as_str()))
                .count() as u32;
            if min.is_some_and(|m| count < m) || max.is_some_and(|m| count > m) {
                Err(format!(
                    "{count} ennemi(s) de l'escouade « {squad} » (min {min:?}, max {max:?})"
                ))
            } else {
                Ok(())
            }
        }
        Expectation::EnemyVariant {
            entity, variant, ..
        } => {
            let found = enemy_variant(world, entity)?;
            if found == *variant {
                Ok(())
            } else {
                Err(format!(
                    "variante {}, {} attendue",
                    found.as_deref().unwrap_or("aucune"),
                    variant.as_deref().unwrap_or("aucune")
                ))
            }
        }
        Expectation::EnemyDistance {
            entity,
            target,
            min,
            max,
            ..
        } => {
            let distance = enemy_distance(world, entity, target)?;
            if let Some(min) = min {
                if distance < fixed_math::new(*min) {
                    return Err(format!("distance {distance} < min {min}"));
                }
            }
            if let Some(max) = max {
                if distance > fixed_math::new(*max) {
                    return Err(format!("distance {distance} > max {max}"));
                }
            }
            Ok(())
        }
        Expectation::EntityCount { kind, min, max, .. } => {
            let count = match kind {
                game::replay::EntityKind::Player => world
                    .query_filtered::<(), With<Player>>()
                    .iter(world)
                    .count() as u32,
                game::replay::EntityKind::Enemy => world
                    .query_filtered::<(), With<game::character::enemy::Enemy>>()
                    .iter(world)
                    .count() as u32,
                game::replay::EntityKind::Bullet => world
                    .query_filtered::<(), With<game::weapons::Bullet>>()
                    .iter(world)
                    .count() as u32,
                game::replay::EntityKind::Rollback => world
                    .query_filtered::<(), With<bevy_ggrs::Rollback>>()
                    .iter(world)
                    .count() as u32,
                game::replay::EntityKind::Barrel => world
                    .query_filtered::<(), With<world::BarrelState>>()
                    .iter(world)
                    .count() as u32,
                game::replay::EntityKind::Table => world
                    .query_filtered::<(), With<world::TableState>>()
                    .iter(world)
                    .count() as u32,
                game::replay::EntityKind::Rock => world
                    .query_filtered::<(), With<world::Rock>>()
                    .iter(world)
                    .count() as u32,
            };

            if let Some(min_val) = min {
                if count < *min_val {
                    return Err(format!("{} entités < min {}", count, min_val));
                }
            }
            if let Some(max_val) = max {
                if count > *max_val {
                    return Err(format!("{} entités > max {}", count, max_val));
                }
            }
            Ok(())
        }
        Expectation::Event {
            kind,
            label_contains,
            by_frame: _,
        } => {
            let events = world.resource::<GameEvents>();
            let found = events.events.iter().any(|event| {
                event.kind == kind.as_str()
                    && label_contains
                        .as_ref()
                        .map_or(true, |label| event.label.contains(label))
            });

            if found {
                Ok(())
            } else {
                let label_desc = label_contains
                    .as_ref()
                    .map(|l| format!(", label contient '{}'", l))
                    .unwrap_or_default();
                Err(format!("événement '{}' non trouvé{}", kind, label_desc))
            }
        }
        Expectation::EventAfter {
            kind,
            label_contains,
            after_frame,
            by_frame: _,
        } => {
            let events = world.resource::<GameEvents>();
            let found = events.events.iter().any(|event| {
                event.kind == kind.as_str()
                    && event.frame > *after_frame
                    && label_contains
                        .as_ref()
                        .map_or(true, |label| event.label.contains(label))
            });
            if found {
                Ok(())
            } else {
                let label_desc = label_contains
                    .as_ref()
                    .map(|l| format!(", label contient '{}'", l))
                    .unwrap_or_default();
                Err(format!(
                    "événement '{}' non trouvé après la frame {}{}",
                    kind, after_frame, label_desc
                ))
            }
        }
        // À terre (T1.3, chantier B6). `PlayerDowned` : vérification ponctuelle (comme
        // `PlayerAlive`/`PlayerDead`) — l'entité existe toujours quand un joueur est à
        // terre (contrairement à la mort), seul `Downed` change.
        Expectation::PlayerDowned { handle, .. } => match player_downed(world, *handle) {
            Some(true) => Ok(()),
            Some(false) => Err("joueur pas à terre".into()),
            None => Err("joueur absent".into()),
        },
        // `PlayerRevived`/`Defeat` : vérification cumulative sur l'historique des
        // `GameEvents` (comme `Event`), la transition elle-même (pas un état ponctuel).
        Expectation::PlayerRevived { handle, by_frame } => {
            let label_needle = format!("joueur {handle} ");
            let found = world.resource::<GameEvents>().events.iter().any(|event| {
                event.kind == "revived"
                    && event.frame <= *by_frame
                    && event.label.contains(&label_needle)
            });
            if found {
                Ok(())
            } else {
                Err(format!(
                    "joueur {handle} pas réanimé avant la frame {by_frame}"
                ))
            }
        }
        Expectation::Defeat { by_frame } => {
            let found = world
                .resource::<GameEvents>()
                .events
                .iter()
                .any(|event| event.kind == "defeat" && event.frame <= *by_frame);
            if found {
                Ok(())
            } else {
                Err(format!("pas de défaite avant la frame {by_frame}"))
            }
        }
        // T2.4, chantier F1.
        Expectation::RunState { step, .. } => {
            let actual = &world.resource::<Run>().step;
            let matches = match (step, actual) {
                (RunStepExpectation::Playing, RunStep::Playing { .. }) => true,
                (RunStepExpectation::Ended(expected_outcome), RunStep::Ended { outcome, .. }) => {
                    expected_outcome == outcome
                }
                _ => false,
            };
            if matches {
                Ok(())
            } else {
                Err(format!("Run.step = {actual:?} (attendu {step:?})"))
            }
        }
        Expectation::RunSummary {
            wave_reached_min,
            kills_min,
            floor_reached_min,
            ..
        } => {
            let Some(summary) = world.resource::<Run>().summary else {
                return Err("Run.summary absent (partie pas terminée ?)".into());
            };
            if let Some(min) = wave_reached_min {
                if summary.wave_reached < *min {
                    return Err(format!("wave_reached {} < min {min}", summary.wave_reached));
                }
            }
            if let Some(min) = kills_min {
                if summary.kills < *min {
                    return Err(format!("kills {} < min {min}", summary.kills));
                }
            }
            if let Some(min) = floor_reached_min {
                if summary.floor_reached < *min {
                    return Err(format!(
                        "floor_reached {} < min {min}",
                        summary.floor_reached
                    ));
                }
            }
            Ok(())
        }
        // T1.8 : index du niveau courant du mode `Floors` (ressource rollback `FloorState`).
        Expectation::Clock { id, fired, .. } => {
            let actual = world.resource::<run::Clock>().fired.contains(id);
            if actual == *fired {
                Ok(())
            } else if actual {
                Err(format!("événement d'horloge « {id} » déjà déclenché"))
            } else {
                Err(format!("événement d'horloge « {id} » pas encore déclenché"))
            }
        }
        Expectation::FloorIndex { index, .. } => {
            let actual = world.resource::<run::FloorState>().index;
            if actual != *index {
                return Err(format!("FloorIndex {actual} (attendu {index})"));
            }
            Ok(())
        }
        // T1.6 : case du terrain d'une caverne (ressource rollback `CellGrid`).
        Expectation::CellState {
            x,
            y,
            kind,
            surface,
            ..
        } => {
            if let Some(kind) = kind {
                match world.resource::<world::CellGrid>().get(*x, *y) {
                    Some(actual) if actual == *kind => {}
                    Some(actual) => {
                        return Err(format!("case ({x}, {y}) : {actual:?} (attendu {kind:?})"))
                    }
                    None => return Err(format!("case ({x}, {y}) hors de la grille")),
                }
            }
            if let Some(surface) = surface {
                // T1.7 : nom de la surface (`"aucune"` = case sans surface)
                let actual = world
                    .resource::<world::SurfaceGrid>()
                    .get(*x, *y)
                    .map(|id| {
                        world
                            .get_resource::<world::SurfaceTable>()
                            .and_then(|t| t.0.get(&id).map(|d| d.name.clone()))
                            .unwrap_or_else(|| format!("#{id}"))
                    })
                    .unwrap_or_else(|| "aucune".into());
                if actual != *surface {
                    return Err(format!(
                        "case ({x}, {y}) : surface « {actual} » (attendu « {surface} »)"
                    ));
                }
            }
            if kind.is_none() && surface.is_none() {
                return Err("CellState sans kind ni surface".into());
            }
            Ok(())
        }
        // T2.3, chantier C5 v1 : scénarios `buy_door`/`buy_wall_weapon`/`buy_perk`.
        Expectation::Gauge {
            handle,
            id,
            min,
            max,
            ..
        } => {
            let Some(value) = player_gauge(world, *handle, id) else {
                return Err(format!("joueur absent ou sans jauge « {id} »"));
            };
            if let Some(min_val) = min.map(fixed_math::Fixed::from_num) {
                if value < min_val {
                    return Err(format!("jauge {id} = {value} < min {min_val}"));
                }
            }
            if let Some(max_val) = max.map(fixed_math::Fixed::from_num) {
                if value > max_val {
                    return Err(format!("jauge {id} = {value} > max {max_val}"));
                }
            }
            Ok(())
        }
        Expectation::HasStatus {
            entity,
            status,
            present,
            ..
        } => {
            let target = resolve_entity(world, entity).ok_or("entité absente")?;
            let carried = world
                .get::<combat::status::Statuses>(target)
                .is_some_and(|statuses| statuses.has(status));
            if carried != *present {
                let statuses = world
                    .get::<combat::status::Statuses>(target)
                    .map(|s| s.0.iter().map(|e| e.id.clone()).collect::<Vec<_>>())
                    .unwrap_or_default();
                return Err(format!(
                    "statut « {status} » {} (porte {statuses:?})",
                    if *present {
                        "absent"
                    } else {
                        "encore présent"
                    }
                ));
            }
            Ok(())
        }
        Expectation::StatusStacks {
            entity,
            status,
            stacks,
            ..
        } => {
            let target = resolve_entity(world, entity).ok_or("entité absente")?;
            let found = world
                .get::<combat::status::Statuses>(target)
                .map_or(0, |statuses| statuses.stacks(status));
            if found != *stacks {
                return Err(format!("statut « {status} » : {found} piles ≠ {stacks}"));
            }
            Ok(())
        }
        Expectation::Level { handle, level, .. } => {
            let Some(actual) = world
                .query::<(&Player, &game::progression::Level)>()
                .iter(world)
                .find(|(player, _)| player.handle == *handle)
                .map(|(_, level)| level.0)
            else {
                return Err("joueur absent ou sans niveau (progression inactive)".into());
            };
            if actual != *level {
                return Err(format!("niveau {actual} ≠ {level}"));
            }
            Ok(())
        }
        Expectation::Mutations {
            handle,
            contains,
            count,
            ..
        } => {
            let taken: Vec<String> = world
                .query::<(&Player, &game::progression::Mutations)>()
                .iter(world)
                .find(|(player, _)| player.handle == *handle)
                .map(|(_, m)| m.0.clone())
                .unwrap_or_default();
            let missing: Vec<&String> = contains.iter().filter(|id| !taken.contains(id)).collect();
            if !missing.is_empty() {
                return Err(format!("mutations {taken:?} : manque {missing:?}"));
            }
            if let Some(count) = count {
                if taken.len() != *count as usize {
                    return Err(format!("mutations {taken:?} : {} ≠ {count}", taken.len()));
                }
            }
            Ok(())
        }
        // M2-T0b : inventaire du joueur (`items::Inventory`, absent tant que rien n'est ramassé).
        Expectation::HasItem { handle, id, .. } => {
            let inventory = player_inventory(world, *handle)?;
            if inventory.as_ref().is_some_and(|i| i.has(id)) {
                Ok(())
            } else {
                Err(format!(
                    "joueur {handle} ne tient pas « {id} » ({inventory:?})"
                ))
            }
        }
        // M3-T3 (§68) : points de vie d'âme du joueur (0 sans le composant).
        Expectation::ExtraHealth { handle, n, .. } => {
            let Some(actual) = world
                .query::<(&Player, Option<&combat::actors::ExtraHealth>)>()
                .iter(world)
                .find(|(player, _)| player.handle == *handle)
                .map(|(_, extra)| extra.map_or(fixed_math::Fixed::ZERO, |extra| extra.soul))
            else {
                return Err("joueur absent".into());
            };
            let wanted = fixed_math::Fixed::from_num(*n);
            if actual == wanted {
                Ok(())
            } else {
                Err(format!("{actual} points de vie d'âme (attendu {wanted})"))
            }
        }
        Expectation::ItemCount {
            handle, id, count, ..
        } => {
            let inventory = player_inventory(world, *handle)?;
            let actual = inventory.as_ref().map_or(0, |i| {
                i.passives.iter().filter(|p| *p == id).count()
                    + usize::from(i.active.as_ref().is_some_and(|a| a.item == *id))
            });
            if actual == *count as usize {
                Ok(())
            } else {
                Err(format!(
                    "joueur {handle} : {actual} exemplaire(s) de « {id} » (attendu {count}) ({inventory:?})"
                ))
            }
        }
        // M3-T0d : familiers du joueur (`behaviors::FamiliarOf`).
        Expectation::FamiliarCount { handle, count, .. } => {
            let owner = world
                .query::<(&Player, &GgrsNetId)>()
                .iter(world)
                .find(|(player, _)| player.handle == *handle)
                .map(|(_, net_id)| net_id.clone())
                .ok_or_else(|| format!("joueur {handle} absent"))?;
            let actual = world
                .query::<&behaviors::FamiliarOf>()
                .iter(world)
                .filter(|familiar| familiar.owner == owner)
                .count();
            if actual == *count as usize {
                Ok(())
            } else {
                Err(format!(
                    "joueur {handle} : {actual} familier(s) (attendu {count})"
                ))
            }
        }
        // M3-T6 : distance d'un familier à son porteur ou à un autre familier du même porteur.
        Expectation::FamiliarDistance {
            handle,
            rank,
            to_rank,
            min,
            max,
            ..
        } => {
            let (owner, owner_position) = world
                .query::<(
                    &Player,
                    &GgrsNetId,
                    &bevy_fixed::fixed_math::FixedTransform3D,
                )>()
                .iter(world)
                .find(|(player, ..)| player.handle == *handle)
                .map(|(_, net_id, transform)| (net_id.clone(), transform.translation.truncate()))
                .ok_or_else(|| format!("joueur {handle} absent"))?;
            let position_of = |world: &mut World, wanted: u32| {
                world
                    .query::<(
                        &behaviors::FamiliarOf,
                        &bevy_fixed::fixed_math::FixedTransform3D,
                    )>()
                    .iter(world)
                    .find(|(familiar, _)| familiar.owner == owner && familiar.rank == wanted)
                    .map(|(_, transform)| transform.translation.truncate())
                    .ok_or_else(|| format!("joueur {handle} : pas de familier de rang {wanted}"))
            };
            let familiar = position_of(world, *rank)?;
            let other = match to_rank {
                Some(other) => position_of(world, *other)?,
                None => owner_position,
            };
            let distance = familiar.distance(&other);
            if let Some(min) = min {
                if distance < fixed_math::new(*min) {
                    return Err(format!("distance {distance} < min {min}"));
                }
            }
            if let Some(max) = max {
                if distance > fixed_math::new(*max) {
                    return Err(format!("distance {distance} > max {max}"));
                }
            }
            Ok(())
        }
        Expectation::Shelf {
            nth,
            item,
            price,
            cost,
            sold,
            ..
        } => {
            let mut shelves: Vec<items::Shelf> = world
                .query::<&items::Shelf>()
                .iter(world)
                .cloned()
                .collect();
            shelves.sort_by(|a, b| a.key.cmp(&b.key));
            let Some(shelf) = shelves.get(*nth) else {
                return Err(format!(
                    "présentoir {nth} absent ({} présentoirs)",
                    shelves.len()
                ));
            };
            if item.as_ref().is_some_and(|item| *item != shelf.item_id)
                || price.is_some_and(|price| price != shelf.price)
                || cost
                    .as_ref()
                    .is_some_and(|cost| Some(cost) != shelf.cost.as_ref())
                || sold.is_some_and(|sold| sold != shelf.sold)
            {
                return Err(format!("présentoir {nth} : {shelf:?}"));
            }
            Ok(())
        }
        Expectation::ShelfCount { count, .. } => {
            let actual = world.query::<&items::Shelf>().iter(world).count() as u32;
            if actual == *count {
                Ok(())
            } else {
                Err(format!("{actual} présentoir(s) (attendu {count})"))
            }
        }
        Expectation::ShellPiles { count, .. } => {
            let actual = world.query::<&items::ShellPickup>().iter(world).count() as u32;
            if actual == *count {
                Ok(())
            } else {
                Err(format!("{actual} tas de douilles (attendu {count})"))
            }
        }
        Expectation::ItemCharge { handle, charge, .. } => {
            let inventory = player_inventory(world, *handle)?;
            match inventory.and_then(|i| i.active) {
                Some(slot) if slot.charge == *charge => Ok(()),
                Some(slot) => Err(format!(
                    "charge de « {} » : {} (attendu {charge})",
                    slot.item, slot.charge
                )),
                None => Err(format!("joueur {handle} sans objet actif")),
            }
        }
        Expectation::Consumable {
            handle, id, count, ..
        } => {
            let actual = player_inventory(world, *handle)?.map_or(0, |i| i.consumable(id));
            if actual == *count {
                Ok(())
            } else {
                Err(format!("{actual} « {id} » (attendu {count})"))
            }
        }
        // M2-E1 : état d'une salle typée (ressource rollback `RoomStates`).
        Expectation::RoomState {
            kind,
            nth,
            state,
            doors_closed,
            ..
        } => {
            let mut rooms: Vec<_> = world
                .query::<(
                    &map::game::entity::map::level_id::LevelId,
                    &map::game::entity::map::room::RoomBounds,
                    &world::RoomKind,
                )>()
                .iter(world)
                .filter(|(_, _, k)| &k.0 == kind)
                .map(|(id, bounds, _)| (bounds.position.x, bounds.position.y, id.0.clone()))
                .collect();
            rooms.sort();
            let Some((_, _, id)) = rooms.get(*nth) else {
                return Err(format!(
                    "{} salle(s) de type « {kind} » (index {nth} demandé)",
                    rooms.len()
                ));
            };
            let actual = world.resource::<world::RoomStates>().get(id);
            if actual != *state {
                return Err(format!(
                    "salle {kind}#{nth} : {actual:?} (attendu {state:?})"
                ));
            }
            if let Some(expected) = doors_closed {
                let closed = world
                    .query_filtered::<Has<game::collider::Collider>, With<map::game::entity::map::door::DoorComponent>>()
                    .iter(world)
                    .filter(|closed| *closed)
                    .count() as u32;
                if closed != *expected {
                    return Err(format!("{closed} portes fermées (attendu {expected})"));
                }
            }
            Ok(())
        }
        // M3-T5 : géométrie d'une salle typée (taille, voisinage par un bord commun).
        Expectation::RoomLayout {
            kind,
            nth,
            size,
            touches,
            apart,
            ..
        } => {
            let mut rooms: Vec<_> = world
                .query::<(&map::game::entity::map::room::RoomBounds, &world::RoomKind)>()
                .iter(world)
                .map(|(bounds, k)| {
                    (
                        bounds.position.x.to_num::<i32>(),
                        bounds.position.y.to_num::<i32>(),
                        bounds.size.x.to_num::<i32>(),
                        bounds.size.y.to_num::<i32>(),
                        k.0.clone(),
                    )
                })
                .collect();
            rooms.sort();
            let of_kind: Vec<_> = rooms.iter().filter(|room| &room.4 == kind).collect();
            let Some(&&(x, y, w, h, _)) = of_kind.get(*nth) else {
                return Err(format!(
                    "{} salle(s) de type « {kind} » (index {nth} demandé)",
                    of_kind.len()
                ));
            };
            if let Some((width, height)) = size {
                if (w, h) != (*width as i32, *height as i32) {
                    return Err(format!(
                        "salle {kind}#{nth} : {w} × {h} (attendu {width} × {height})"
                    ));
                }
            }
            // un bord commun de longueur non nulle (salles voisines d'une grille)
            let touching = |other: &&(i32, i32, i32, i32, String)| {
                let (ox, oy, ow, oh, _) = **other;
                let share_x = x < ox + ow && ox < x + w;
                let share_y = y < oy + oh && oy < y + h;
                (share_x && (y + h == oy || oy + oh == y))
                    || (share_y && (x + w == ox || ox + ow == x))
            };
            let neighbours: std::collections::BTreeSet<&str> = rooms
                .iter()
                .filter(|other| (other.0, other.1) != (x, y))
                .filter(touching)
                .map(|other| other.4.as_str())
                .collect();
            for wanted in touches {
                if !neighbours.contains(wanted.as_str()) {
                    return Err(format!(
                        "salle {kind}#{nth} : aucune salle « {wanted} » ne la touche (voisines : {neighbours:?})"
                    ));
                }
            }
            for refused in apart {
                if neighbours.contains(refused.as_str()) {
                    return Err(format!(
                        "salle {kind}#{nth} : touche une salle « {refused} » (voisines : {neighbours:?})"
                    ));
                }
            }
            Ok(())
        }
        // M3-T9 : nombre de salles de la carte (curse `Labyrinth`).
        Expectation::RoomCount { min, max, .. } => {
            let count = world
                .query::<&map::game::entity::map::room::RoomBounds>()
                .iter(world)
                .count() as u32;
            if count >= *min && max.is_none_or(|max| count <= max) {
                Ok(())
            } else {
                Err(format!("{count} salle(s) (attendu entre {min} et {max:?})"))
            }
        }
        // M3-T0c : porte verrouillée d'une salle typée (`world::DoorLocks`, `RoomKindDef::door_lock`).
        Expectation::DoorLocked {
            kind, nth, locked, ..
        } => {
            let mut rooms: Vec<_> = world
                .query::<(
                    &map::game::entity::map::level_id::LevelId,
                    &map::game::entity::map::room::RoomBounds,
                    &world::RoomKind,
                )>()
                .iter(world)
                .filter(|(_, _, k)| &k.0 == kind)
                .map(|(id, bounds, _)| (bounds.position.x, bounds.position.y, id.0.clone()))
                .collect();
            rooms.sort();
            let Some((_, _, id)) = rooms.get(*nth) else {
                return Err(format!(
                    "{} salle(s) de type « {kind} » (index {nth} demandé)",
                    rooms.len()
                ));
            };
            // Verrou de l'étage courant (`lock_from_floor`, M3-T5) ; 0 hors `Floors`.
            let floor = world
                .get_resource::<run::FloorState>()
                .map_or(0, |state| state.index);
            let has_lock = world
                .resource::<world::RoomKindTable>()
                .door_lock_at(kind, floor)
                .is_some();
            let actual = has_lock && !world.resource::<world::DoorLocks>().is_open(id);
            if actual != *locked {
                return Err(format!(
                    "salle {kind}#{nth} : porte verrouillée = {actual} (attendu {locked})"
                ));
            }
            Ok(())
        }
        // M2-T0c : phase d'un boss (`behaviors::BossState`).
        Expectation::BossPhase { entity, phase, .. } => {
            let target = resolve_entity(world, entity).ok_or("entité absente")?;
            let actual = world
                .get::<behaviors::BossState>(target)
                .ok_or("entité sans BossState (pas un boss)")?
                .phase;
            if actual == *phase {
                Ok(())
            } else {
                Err(format!("phase {actual} (attendu {phase})"))
            }
        }
        // M2-T0c : profil de méta-progression (hors simulation).
        Expectation::ProfileHas {
            handle,
            unlock,
            currency,
            achievement,
            lacks_unlocks,
            lacks_achievements,
            ..
        } => {
            let id = meta::local_id(*handle);
            let Some(profile) = world
                .get_resource::<meta::Profiles>()
                .and_then(|profiles| profiles.0.get(&id))
            else {
                return Err(format!(
                    "pas de profil « {id} » (scénario sans `profile: true` ?)"
                ));
            };
            if let Some(unlock) = unlock {
                if !profile.has_unlock(unlock) {
                    return Err(format!("profil « {id} » sans déblocage « {unlock} »"));
                }
            }
            if let Some((currency, min)) = currency {
                let actual = profile.currency(currency);
                if actual < *min {
                    return Err(format!("profil « {id} » : {currency} = {actual} < {min}"));
                }
            }
            if let Some(achievement) = achievement {
                if !profile.achievements.contains(achievement) {
                    return Err(format!(
                        "profil « {id} » sans accomplissement « {achievement} » ({:?})",
                        profile.achievements
                    ));
                }
            }
            if let Some(extra) = lacks_unlocks.iter().find(|u| profile.has_unlock(u)) {
                return Err(format!("profil « {id} » possède le déblocage « {extra} »"));
            }
            if let Some(extra) = lacks_achievements
                .iter()
                .find(|a| profile.achievements.contains(*a))
            {
                return Err(format!(
                    "profil « {id} » possède l'accomplissement « {extra} »"
                ));
            }
            if unlock.is_none()
                && currency.is_none()
                && achievement.is_none()
                && lacks_unlocks.is_empty()
                && lacks_achievements.is_empty()
            {
                return Err("ProfileHas sans condition".into());
            }
            Ok(())
        }
        // M2-T12 : déblocages de la partie (ressource rollback neutre, statique).
        Expectation::RunUnlocks { has, lacks, .. } => {
            let active = world
                .get_resource::<items::RunUnlocks>()
                .cloned()
                .unwrap_or_default();
            if let Some(missing) = has.iter().find(|id| !active.has(id)) {
                return Err(format!("RunUnlocks sans « {missing} » ({:?})", active.0));
            }
            if let Some(extra) = lacks.iter().find(|id| active.has(id)) {
                return Err(format!("RunUnlocks contient « {extra} » ({:?})", active.0));
            }
            Ok(())
        }
        // M3-T0e (§64) : curse de l'étage (ressource rollback neutre `run::RunCurses`).
        Expectation::Curse { floor, id, .. } => {
            let actual = world
                .get_resource::<run::RunCurses>()
                .and_then(|curses| curses.get(*floor).map(str::to_string));
            if actual == *id {
                Ok(())
            } else {
                Err(format!(
                    "curse de l'étage {floor} : {actual:?} (attendu {id:?})"
                ))
            }
        }
        // M3-T0e (§64) : objets sortis des pools à épuisement (`items::PoolState`).
        Expectation::PoolSeen {
            pool, item, seen, ..
        } => {
            let actual = world
                .get_resource::<items::PoolState>()
                .is_some_and(|state| state.has_seen(pool, item));
            if actual == *seen {
                Ok(())
            } else {
                Err(format!(
                    "pool « {pool} » : « {item} » sorti = {actual} (attendu {seen})"
                ))
            }
        }
        // M3-T8 (§73) : objets distincts sortis d'un pool à épuisement.
        Expectation::PoolCount { pool, count, .. } => {
            let actual = world
                .get_resource::<items::PoolState>()
                .map_or(0, |state| state.count(pool));
            if actual == *count {
                Ok(())
            } else {
                Err(format!(
                    "pool « {pool} » : {actual} objet(s) sorti(s) (attendu {count})"
                ))
            }
        }
        // M3-T8 (§73) : objets au sol (butin de boss, objet lâché).
        Expectation::ItemsOnFloor { id, count, .. } => {
            let actual = world
                .query::<&items::ItemPickup>()
                .iter(world)
                .filter(|pickup| pickup.item_id == *id)
                .count() as u32;
            if actual == *count {
                Ok(())
            } else {
                Err(format!("{actual} « {id} » au sol (attendu {count})"))
            }
        }
        // M2-T13 : décisions du hub dans la simulation (`game::hub::HubState`).
        Expectation::HubExit {
            handle,
            exit_frame,
            none,
            ..
        } => {
            let state = world
                .get_resource::<game::hub::HubState>()
                .cloned()
                .unwrap_or_default();
            match (state.exit, *none) {
                (None, true) => Ok(()),
                (Some(exit), true) => Err(format!("sortie du hub décidée : {exit:?}")),
                (None, false) => Err("aucune sortie du hub décidée".into()),
                (Some(exit), false) => {
                    if handle.is_some_and(|handle| handle != exit.handle) {
                        Err(format!(
                            "sortie par le joueur {} (attendu {handle:?})",
                            exit.handle
                        ))
                    } else if exit_frame.is_some_and(|frame| frame != exit.frame) {
                        Err(format!(
                            "sortie à la frame {} (attendu {exit_frame:?})",
                            exit.frame
                        ))
                    } else {
                        Ok(())
                    }
                }
            }
        }
        Expectation::HubVisit {
            handle, service, ..
        } => {
            let state = world
                .get_resource::<game::hub::HubState>()
                .cloned()
                .unwrap_or_default();
            match state.visits.get(handle) {
                Some(visit) if visit.service == *service => Ok(()),
                other => Err(format!(
                    "service du joueur {handle} : {other:?} (attendu « {service} »)"
                )),
            }
        }
        // T1.18 : texte d'une source du HUD (présentation, hors trace).
        Expectation::HudText {
            source,
            contains,
            empty,
            ..
        } => {
            let Some(snapshot) = world.get_resource::<game::ui::hud_model::HudSnapshot>() else {
                return Err("pas de HudSnapshot".into());
            };
            let text = snapshot.texts.get(source).map_or("", String::as_str);
            if *empty {
                return if text.is_empty() {
                    Ok(())
                } else {
                    Err(format!("HUD {source} = « {text} » (attendu vide)"))
                };
            }
            if text.contains(contains.as_str()) {
                Ok(())
            } else {
                Err(format!(
                    "HUD {source} = « {text} » (attendu « {contains} »)"
                ))
            }
        }
        // M2-T18 : texte d'une section d'un écran (pause, inventaire), présentation hors trace.
        Expectation::ScreenText {
            screen,
            section,
            contains,
            closed,
            ..
        } => {
            let Some(screens) = world.get_resource::<game::ui::screens::Screens>() else {
                return Err("pas de Screens".into());
            };
            let open = screens.view.as_ref().map(|view| view.screen.name());
            if *closed {
                return match open {
                    None => Ok(()),
                    Some(open) => Err(format!("écran « {open} » ouvert, attendu fermé")),
                };
            }
            let text = match &screens.view {
                Some(view) if view.screen.name() == screen => view.section_text(section),
                _ => {
                    return Err(format!(
                        "écran « {screen} » fermé (ouvert : {})",
                        open.unwrap_or("aucun")
                    ))
                }
            };
            if text.contains(contains.as_str()) {
                Ok(())
            } else {
                Err(format!(
                    "écran {screen}, section {section} = « {text} » (attendu « {contains} »)"
                ))
            }
        }
        Expectation::Currency {
            handle, min, max, ..
        } => {
            let Some(balance) = player_currency(world, *handle) else {
                return Err("joueur absent".into());
            };
            if let Some(min_val) = min {
                if balance < *min_val {
                    return Err(format!("solde {balance} < min {min_val}"));
                }
            }
            if let Some(max_val) = max {
                if balance > *max_val {
                    return Err(format!("solde {balance} > max {max_val}"));
                }
            }
            Ok(())
        }
        Expectation::Stat {
            handle,
            stat,
            value,
            entity,
            ..
        } => {
            let resolved = match entity {
                Some(entity) => resolve_entity(world, entity)
                    .and_then(|target| entity_stat(world, target, stat)),
                None => player_stat(world, *handle, stat),
            };
            let Some(resolved) = resolved else {
                return Err(format!("joueur ou entité absent, ou sans stat {stat:?}"));
            };
            let expected = fixed_math::Fixed::from_num(*value);
            if resolved == expected {
                Ok(())
            } else {
                Err(format!("stat {stat:?} = {resolved} (attendu {expected})"))
            }
        }
    }
}

/// Solde de monnaie du joueur `handle` (T2.3, chantier C5 v1). `None` si le joueur est absent.
/// Valeur de la jauge `id` du joueur `handle` (T1.10).
fn player_gauge(world: &mut World, handle: usize, id: &str) -> Option<fixed_math::Fixed> {
    world
        .query::<(&Player, &game::effects_runtime::Gauges)>()
        .iter(world)
        .find(|(player, _)| player.handle == handle)
        .and_then(|(_, gauges)| gauges.0.get(id).map(|g| g.value))
}

fn player_currency(world: &mut World, handle: usize) -> Option<u32> {
    world
        .query::<(&Player, &Currency)>()
        .iter(world)
        .find(|(player, _)| player.handle == handle)
        .map(|(_, currency)| currency.0)
}

/// Valeur résolue (base + modificateurs actifs à la frame courante) d'une stat du joueur
/// `handle` (T2.3, chantier C5 v1, scénario `buy_perk`). `None` si le joueur est absent ou
/// n'a pas cette stat — même règle que `stats::StatReader::try_get`, réimplémentée ici en
/// requêtes `World` directes (`check` vérifie des attentes après coup, hors d'un système
/// Bevy où `StatReader`, un `SystemParam`, serait injectable directement).
fn player_stat(world: &mut World, handle: usize, stat: &StatId) -> Option<fixed_math::Fixed> {
    let entity = world
        .query::<(&Player, Entity)>()
        .iter(world)
        .find(|(player, _)| player.handle == handle)
        .map(|(_, entity)| entity)?;
    entity_stat(world, entity, stat)
}

/// Stat résolue d'une entité (base `Stats` + `Modifiers` actifs).
fn entity_stat(world: &mut World, entity: Entity, stat: &StatId) -> Option<fixed_math::Fixed> {
    // M3-T0a : `ShotSpeed`/`ShotSize` ne sont pas posées dans `Stats` (base 1, comme au tir).
    let base = world
        .get::<Stats>(entity)?
        .get(stat)
        .or_else(|| sim_core::stats::default_base(stat))?;
    let frame = world.resource::<FrameCount>().frame;
    Some(match world.get::<Modifiers>(entity) {
        Some(modifiers) => resolve(base, modifiers.iter().filter(|m| &m.stat == stat), frame),
        None => base,
    })
}

/// Le joueur `handle` est-il à terre (`combat::downed::Downed`) ? `None` si absent (mort).
fn player_downed(world: &mut World, handle: usize) -> Option<bool> {
    world
        .query::<(&Player, Has<combat::downed::Downed>)>()
        .iter(world)
        .find(|(player, _)| player.handle == handle)
        .map(|(_, downed)| downed)
}

fn summarize(world: &mut World, frame: u32) -> String {
    let mut alive: Vec<usize> = world
        .query::<&Player>()
        .iter(world)
        .map(|player| player.handle)
        .collect();
    alive.sort();
    let waves = world.resource::<WaveState>();
    format!(
        "frame {frame} : vague {}, {} ennemis tués, joueurs vivants {alive:?}",
        waves.current_wave, waves.total_enemies_killed
    )
}

/// Arme active d'un joueur : nom, mode, munitions du chargeur.
fn active_weapon(world: &mut World, handle: usize) -> Option<(String, String, u32)> {
    use game::weapons::{WeaponInventory, WeaponModesState, WeaponState};
    let (entity, name) = world
        .query::<(&Player, &WeaponInventory)>()
        .iter(world)
        .find(|(player, _)| player.handle == handle)
        .and_then(|(_, inventory)| inventory.weapons.get(inventory.active_weapon_index))
        .map(|(entity, weapon)| (*entity, weapon.config.name.clone()))?;
    let (state, modes) = world
        .query::<(&WeaponState, &WeaponModesState)>()
        .get(world, entity)
        .ok()?;
    let ammo = modes
        .modes
        .get(&state.active_mode)
        .map_or(0, |m| m.mag_ammo);
    Some((name, state.active_mode.clone(), ammo))
}

/// Réserve du joueur `handle` pour `ammo_type` (T2.2, chantier B7,
/// `combat::inventory::AmmoReserves`). `None` si le joueur est absent (0 s'il est présent
/// mais que ce type n'a jamais été crédité, voir `AmmoReserves::get`).
fn player_ammo_reserve(
    world: &mut World,
    handle: usize,
    ammo_type: &sim_core::ammo::AmmoType,
) -> Option<u32> {
    world
        .query::<(&Player, &combat::inventory::AmmoReserves)>()
        .iter(world)
        .find(|(player, _)| player.handle == handle)
        .map(|(_, reserves)| reserves.get(ammo_type))
}

fn windows_broken(world: &mut World) -> u32 {
    world
        .query::<&game::character::enemy::ai::Obstacle>()
        .iter(world)
        .filter(|o| o.breakable && !o.is_intact())
        .count() as u32
}

fn player_alive(world: &mut World, handle: usize) -> bool {
    world
        .query::<&Player>()
        .iter(world)
        .any(|player| player.handle == handle)
}

/// Santé courante du joueur `handle`.
fn player_health(world: &mut World, handle: usize) -> Option<fixed_math::Fixed> {
    use game::character::health::Health;
    world
        .query::<(&Player, &Health)>()
        .iter(world)
        .find(|(player, _)| player.handle == handle)
        .map(|(_, health)| health.current)
}

/// Santé courante d'une entité rollback, par son `GgrsNetId`.
fn entity_health(world: &mut World, net_id: usize) -> Option<fixed_math::Fixed> {
    use bevy_ggrs::Rollback;
    use game::character::health::Health;
    use utils::net_id::GgrsNetId;
    world
        .query_filtered::<(&GgrsNetId, &Health), With<Rollback>>()
        .iter(world)
        .find(|(id, _)| id.0 == net_id)
        .map(|(_, health)| health.current)
}

/// Nombre de coups reçus par une entité rollback (`HitCount`, T2.9), par son `GgrsNetId`.
fn entity_hit_count(world: &mut World, net_id: usize) -> Option<u32> {
    use bevy_ggrs::Rollback;
    use game::character::health::HitCount;
    use utils::net_id::GgrsNetId;
    world
        .query_filtered::<(&GgrsNetId, &HitCount), With<Rollback>>()
        .iter(world)
        .find(|(id, _)| id.0 == net_id)
        .map(|(_, hit_count)| hit_count.0)
}

/// `HitsAtLeast(entity: Target)` (T1.1) : compteur de coups de l'entité qui compte ses coups
/// de plus petit `GgrsNetId` (`target` dans l'arène du testbed).
fn target_hit_count(world: &mut World) -> Option<u32> {
    use bevy_ggrs::Rollback;
    use game::character::health::HitCount;
    use utils::net_id::GgrsNetId;
    world
        .query_filtered::<(&GgrsNetId, &HitCount), With<Rollback>>()
        .iter(world)
        .min_by_key(|(id, _)| id.0)
        .map(|(_, hit_count)| hit_count.0)
}

/// `BulletCount` (T1.1) : balles vivantes, filtrées par id de projectile composable et par
/// équipe du tireur.
fn bullet_count(
    world: &mut World,
    projectile: Option<&str>,
    team: Option<sim_core::team::Team>,
) -> u32 {
    use bevy_ggrs::Rollback;
    use combat::projectile::Projectile;
    use game::weapons::Bullet;
    world
        .query_filtered::<(&Bullet, Option<&Projectile>), With<Rollback>>()
        .iter(world)
        .filter(|(bullet, composable)| {
            projectile.is_none_or(|id| composable.is_some_and(|p| p.id == id))
                && team.is_none_or(|team| bullet.source_team == team)
        })
        .count() as u32
}

/// `NoDamageBetween` : la santé du joueur ne doit pas avoir baissé depuis la frame précédente
/// de l'intervalle (comparaison en `Fixed`, un état par intervalle).
fn check_no_damage(
    world: &mut World,
    dernieres: &mut BTreeMap<(usize, u32), fixed_math::Fixed>,
    handle: usize,
    from_frame: u32,
) -> Result<(), String> {
    let courante = player_health(world, handle).ok_or("joueur absent")?;
    let cle = (handle, from_frame);
    let resultat = match dernieres.get(&cle) {
        Some(&precedente) if courante < precedente => {
            Err(format!("santé passée de {precedente} à {courante}"))
        }
        _ => Ok(()),
    };
    dernieres.insert(cle, courante);
    resultat
}

/// Joue le scénario avec rendu, à vitesse réelle, et quitte à la fin de ses frames.
/// Les attentes ne sont pas vérifiées : c'est un outil de visualisation.
pub fn play(scenario: &Scenario, config: PlayConfig) -> AppExit {
    let frames = scenario.frames;
    let mut app = build_app(scenario, false, &config);
    app.add_systems(
        Update,
        move |frame: Res<FrameCount>,
              hub_over: Option<Res<HubPhaseOver>>,
              mut exit: MessageWriter<AppExit>| {
            if frame.frame >= frames && hub_over.map_or(true, |over| over.0) {
                exit.write(AppExit::Success);
            }
        },
    );
    app.run()
}

/// Capture d'un scénario en images, pour les vidéos.
pub struct CaptureConfig {
    /// Dossier des images (`frame_00000.png`, ...), numérotées par frame de simulation.
    pub dir: std::path::PathBuf,
    /// Une image toutes les `every` frames (2 → vidéo à 30 images/s).
    pub every: u32,
    /// Si présent, force la caméra à suivre le joueur avec ce handle GGRS.
    pub follow_handle: Option<usize>,
    /// Capture viewport, for checking responsive HUD layouts.
    pub size: (u32, u32),
    /// Frames à capturer (sinon : une toutes les `every`). Non vide : la partie s'arrête après la
    /// dernière (`play_scenario --at 42,157`), pour les captures de preuve d'un rapport.
    pub at: Vec<u32>,
    /// Légende ouverte (F7) et sans étiquettes : image de légende en tête des vidéos de revue
    /// (`scripts/scenario-video`, `play_scenario --legend`).
    pub legend: bool,
}

#[derive(Resource)]
struct CaptureState {
    dir: std::path::PathBuf,
    every: u32,
    frames: u32,
    last_captured: Option<u32>,
    updates_after_end: u32,
    /// Image où la caméra rend pendant la capture (taille fixe, indépendante de la fenêtre).
    target: Option<Handle<Image>>,
    size: (u32, u32),
    at: Vec<u32>,
}

/// Taille des images capturées.
pub const CAPTURE_SIZE: (u32, u32) = (960, 540);

/// Joue le scénario avec rendu et capture une image toutes les `every` frames de
/// simulation. Le temps avance d'exactement une frame par update : l'image `n` montre
/// toujours la frame `n`, quelle que soit la vitesse de la machine.
pub fn capture(scenario: &Scenario, config: CaptureConfig) -> AppExit {
    assert!(
        config.size.0 > 0 && config.size.1 > 0,
        "capture size must be positive"
    );
    std::fs::create_dir_all(&config.dir).expect("dossier de capture");

    let play_config = PlayConfig {
        follow_handle: config.follow_handle,
    };
    let mut app = build_app(scenario, false, &play_config);
    // m1-cloture-videos-digest : captures propres, sans overlay de vagues ni texte de debug
    // caméra (présentation seule : la simulation et la trace ne lisent ni l'un ni l'autre).
    app.insert_resource(WaveDebugEnabled(false))
        .insert_resource(game::camera::ui::CameraDebugUiEnabled(false));
    // Fiche m2-lisibilite-gungeon : les captures expliquent ce qu'elles montrent. Étiquettes allumées
    // par défaut (`ALACOD_LABELS=0` les coupe) ; la légende seule pour l'image de tête des vidéos.
    app.insert_resource(game::legibility::LabelsEnabled(
        !config.legend && std::env::var("ALACOD_LABELS").map_or(true, |v| v != "0"),
    ))
    .insert_resource(game::legibility::LegendOpen(config.legend));
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_nanos(1_000_000_000 / game::session::SIM_FPS),
    ))
    .insert_resource(CaptureState {
        dir: config.dir,
        every: config.every.max(1),
        frames: scenario.frames,
        last_captured: None,
        updates_after_end: 0,
        target: None,
        size: config.size,
        at: config.at,
    })
    .add_systems(Startup, configure_capture_window)
    .add_systems(Update, (render_cameras_to_image, capture_frames).chain());
    app.run()
}

/// La fenêtre ne sert pas : cachée, et sans vsync pour capturer aussi vite que possible.
fn configure_capture_window(mut windows: Query<&mut bevy::window::Window>) {
    for mut window in &mut windows {
        window.present_mode = bevy::window::PresentMode::AutoNoVsync;
        window.visible = false;
    }
}

/// Fait rendre les caméras dans une image de taille fixe ([`CAPTURE_SIZE`]) : le cadrage ne
/// dépend pas de la taille de la fenêtre (donnée par le gestionnaire de fenêtres).
fn render_cameras_to_image(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut state: ResMut<CaptureState>,
    cameras: Query<(Entity, &bevy::camera::RenderTarget), With<Camera>>,
) {
    let size = state.size;
    let target = state
        .target
        .get_or_insert_with(|| {
            images.add(Image::new_target_texture(
                size.0,
                size.1,
                bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                None,
            ))
        })
        .clone();
    // bevy_ui ne dessine que sur sa caméra par défaut, qui doit viser la fenêtre principale : une
    // caméra retargetée vers une image n'en est plus une, et l'UI (HUD, prompts, game over)
    // disparaîtrait des captures. On désigne explicitement la première caméra comme caméra d'UI.
    let mut ui_camera_set = false;
    for (camera, current) in &cameras {
        if !matches!(current, bevy::camera::RenderTarget::Image(_)) {
            let mut entity = commands.entity(camera);
            entity.insert(bevy::camera::RenderTarget::Image(target.clone().into()));
            if !ui_camera_set {
                entity.insert(bevy::ui::IsDefaultUiCamera);
                ui_camera_set = true;
            }
        }
    }
}

fn capture_frames(
    mut commands: Commands,
    frame: Res<FrameCount>,
    session: Option<Res<bevy_ggrs::Session<game::character::player::jjrs::PeerConfig>>>,
    mut state: ResMut<CaptureState>,
    events: Res<GameEvents>,
    hub_over: Option<Res<HubPhaseOver>>,
    mut exit: MessageWriter<AppExit>,
) {
    use bevy::render::view::screenshot::{save_to_disk, Screenshot};

    // Rien à capturer avant le début de la partie
    if session.is_none() {
        return;
    }
    let Some(target) = state.target.clone() else {
        return;
    };

    let frame = frame.frame;
    let run_started = hub_over.as_ref().map_or(true, |over| over.0);
    if !run_started {
        // Hub suivi : aucune image tant que la run n'a pas commencé (la vidéo montre la run seule).
        return;
    }
    let last_at = state.at.iter().copied().max();
    if frame >= state.frames || last_at.is_some_and(|last| frame > last) {
        // Laisser le temps aux dernières captures d'être écrites
        state.updates_after_end += 1;
        if state.updates_after_end > 10 {
            // Moments clés à côté des images, pour la page de revue
            let frames = state.frames;
            let events: Vec<&GameEvent> =
                events.events.iter().filter(|e| e.frame < frames).collect();
            let json =
                serde_json::to_string_pretty(&events).expect("sérialisation des moments clés");
            std::fs::write(state.dir.join("events.json"), json).expect("écriture de events.json");
            exit.write(AppExit::Success);
        }
        return;
    }

    let wanted = if state.at.is_empty() {
        frame % state.every == 0
    } else {
        state.at.contains(&frame)
    };
    if wanted && state.last_captured != Some(frame) {
        let path = state.dir.join(format!("frame_{frame:05}.png"));
        commands
            .spawn(Screenshot::image(target))
            .observe(save_to_disk(path));
        state.last_captured = Some(frame);
    }
}

/// Attentes relevées à chaque frame plutôt qu'à `at_frame`.
fn is_continuous(expectation: &Expectation) -> bool {
    matches!(
        expectation,
        Expectation::NoDamageBetween { .. }
            | Expectation::EnemyContactBefore { .. }
            | Expectation::EnemyNeverInWall { .. }
    )
}

/// Entité rollback désignée par un `EntityRef` (`Target` : l'entité qui compte ses coups de
/// plus petit `GgrsNetId`, comme `HitsAtLeast`).
fn resolve_entity(world: &mut World, entity: &game::replay::EntityRef) -> Option<Entity> {
    use bevy_ggrs::Rollback;
    match entity {
        game::replay::EntityRef::NetId(net_id) => world
            .query_filtered::<(Entity, &GgrsNetId), With<Rollback>>()
            .iter(world)
            .find(|(_, id)| id.0 == *net_id)
            .map(|(entity, _)| entity),
        game::replay::EntityRef::Target => world
            .query_filtered::<(Entity, &GgrsNetId), (With<Rollback>, With<game::character::health::HitCount>)>()
            .iter(world)
            .min_by_key(|(_, id)| id.0)
            .map(|(entity, _)| entity),
        game::replay::EntityRef::Placed(index) => world
            .query_filtered::<(Entity, &GgrsNetId, &ScriptedPlacement), With<Rollback>>()
            .iter(world)
            .filter(|(_, _, placement)| placement.0 == *index)
            .min_by_key(|(_, id, _)| id.0)
            .map(|(entity, _, _)| entity),
    }
}

/// `EnemyState` (T1.4) : nom du behavior retenu par l'ennemi
/// (`game::character::enemy::ai::rules::current_rule`).
fn enemy_rule(
    world: &mut World,
    entity: &game::replay::EntityRef,
) -> Result<Option<String>, String> {
    use game::character::enemy::ai::rules::current_rule;
    use game::character::enemy::ai::state::{
        BehaviorRuntime, EnemyBehaviors, EnemyTarget, MonsterState, RangedAttackState,
    };
    let target_entity = resolve_entity(world, entity).ok_or("entité absente")?;
    let mut query = world.query::<(
        &EnemyBehaviors,
        Option<&BehaviorRuntime>,
        &MonsterState,
        &EnemyTarget,
        Option<&RangedAttackState>,
        Has<combat::emitter::Emitter>,
        Option<&behaviors::BossState>,
    )>();
    let (rules, runtime, state, target, ranged, emitting, boss) =
        query
            .get(world, target_entity)
            .map_err(|_| "entité sans règles de comportement (pas un ennemi)".to_string())?;
    let shooting = emitting || ranged.is_some_and(|ranged| ranged.target.is_some());
    Ok(current_rule(
        rules,
        runtime,
        state,
        target,
        shooting,
        boss.map_or(0, |b| b.phase),
    )
    .map(str::to_string))
}

/// `EnemyVariant` (T1.5) : variante du personnage (`None` sans composant `Variant`).
fn enemy_variant(
    world: &mut World,
    entity: &game::replay::EntityRef,
) -> Result<Option<String>, String> {
    use game::character::variant::Variant;
    let target = resolve_entity(world, entity).ok_or("entité absente")?;
    Ok(world
        .query::<&Variant>()
        .get(world, target)
        .ok()
        .map(|variant| variant.0.clone()))
}

/// Position monde d'une entité.
fn entity_position(world: &mut World, entity: Entity) -> Option<fixed_math::FixedVec2> {
    world
        .query::<&fixed_math::FixedTransform3D>()
        .get(world, entity)
        .ok()
        .map(|transform| transform.translation.truncate())
}

/// `EnemyDistance` (T1.4) : distance de l'ennemi à la cible.
fn enemy_distance(
    world: &mut World,
    entity: &game::replay::EntityRef,
    target: &game::replay::DistanceTarget,
) -> Result<fixed_math::Fixed, String> {
    let enemy = resolve_entity(world, entity).ok_or("entité absente")?;
    let enemy_pos = entity_position(world, enemy).ok_or("entité sans position")?;
    let target_pos = match target {
        game::replay::DistanceTarget::Player(handle) => world
            .query::<(&Player, &fixed_math::FixedTransform3D)>()
            .iter(world)
            .find(|(player, _)| player.handle == *handle)
            .map(|(_, transform)| transform.translation.truncate())
            .ok_or_else(|| format!("joueur {handle} absent"))?,
        game::replay::DistanceTarget::Entity(other) => {
            let other = resolve_entity(world, other).ok_or("cible absente")?;
            entity_position(world, other).ok_or("cible sans position")?
        }
    };
    Ok(enemy_pos.distance(&target_pos))
}

/// `EnemyContactBefore` (T1.4) : un joueur est à portée de mêlée (`attack_range`) de
/// l'ennemi.
fn enemy_in_melee_range(world: &mut World, entity: &game::replay::EntityRef) -> bool {
    use game::character::enemy::ai::state::EnemyAiConfig;
    let Some(enemy) = resolve_entity(world, entity) else {
        return false;
    };
    let Some(enemy_pos) = entity_position(world, enemy) else {
        return false;
    };
    let Ok(reach) = world
        .query::<&EnemyAiConfig>()
        .get(world, enemy)
        .map(|config| config.attack_range)
    else {
        return false;
    };
    world
        .query_filtered::<&fixed_math::FixedTransform3D, With<Player>>()
        .iter(world)
        .any(|transform| enemy_pos.distance(&transform.translation.truncate()) <= reach)
}

/// `EnemyNeverInWall` (T1.4) : le collider de l'ennemi chevauche un `Wall` ; M2-T11 : ou les
/// pieds d'un ennemi au sol sont sur une case de fosse (`world::SurfaceGrid`).
fn enemy_in_wall(world: &mut World, entity: &game::replay::EntityRef) -> bool {
    use combat::collider::{is_colliding, Collider, Wall};
    let Some(enemy) = resolve_entity(world, entity) else {
        return false;
    };
    let Ok((enemy_transform, enemy_collider)) = world
        .query::<(&fixed_math::FixedTransform3D, &Collider)>()
        .get(world, enemy)
        .map(|(transform, collider)| (transform.clone(), collider.clone()))
    else {
        return false;
    };
    let grounded = world
        .get::<game::character::enemy::ai::state::EnemyAiConfig>(enemy)
        .is_some_and(|ai| {
            ai.movement_type != game::character::enemy::ai::state::MovementType::Flying
        });
    if grounded {
        if let (Some(grid), Some(table)) = (
            world.get_resource::<world::SurfaceGrid>(),
            world.get_resource::<world::SurfaceTable>(),
        ) {
            let feet = game::character::surface::feet_position(&enemy_transform, &enemy_collider);
            if grid.pit_at(table, feet).is_some() {
                return true;
            }
        }
    }
    world
        .query_filtered::<(&fixed_math::FixedTransform3D, &Collider), With<Wall>>()
        .iter(world)
        .any(|(wall_transform, wall_collider)| {
            is_colliding(
                &enemy_transform.translation,
                &enemy_collider,
                &wall_transform.translation,
                wall_collider,
            )
        })
}

/// D44 : progression du soft-lock `Floors`.
#[cfg(test)]
mod floors_progress_tests {
    use super::*;

    fn state(enemies: usize, health: f32, players: &[(usize, bool)]) -> FloorsProgress {
        FloorsProgress {
            floor: 2,
            explored: 10,
            enemies,
            enemy_health: fixed_math::new(health),
            players: players.to_vec(),
            stock: 0,
        }
    }

    #[test]
    fn degats_sans_kill_et_changements_d_etat_comptent() {
        let base = state(2, 600.0, &[(0, false), (1, false)]);
        // Rien ne change : pas de progression
        assert!(!base.progressed(&base.clone()));
        // Combat lent contre un boss : dégâts sans kill
        assert!(base.progressed(&state(2, 590.0, &[(0, false), (1, false)])));
        // Un joueur tombe à terre, puis meurt (disparaît)
        assert!(base.progressed(&state(2, 600.0, &[(0, true), (1, false)])));
        assert!(base.progressed(&state(2, 600.0, &[(1, false)])));
        // Un ennemi en moins, un niveau de plus
        assert!(base.progressed(&state(1, 600.0, &[(0, false), (1, false)])));
        let next_floor = FloorsProgress {
            floor: 3,
            ..base.clone()
        };
        assert!(base.progressed(&next_floor));
        // Santé des ennemis qui remonte (régénération) : pas une progression
        assert!(!base.progressed(&state(2, 610.0, &[(0, false), (1, false)])));
        // Une case jamais visitée : le joueur explore encore ; la même case revisitée n'est rien
        let explorer = FloorsProgress {
            explored: 11,
            ..base.clone()
        };
        assert!(base.progressed(&explorer));
        assert!(!explorer.progressed(&explorer.clone()));
    }

    /// M3-T14 : l'empreinte de l'inventaire et du solde (`stock`) compte ; à 0 des deux côtés (détecteur
    /// d'avant), rien ne change.
    #[test]
    fn un_ramassage_compte_quand_le_detecteur_le_demande() {
        let base = state(2, 600.0, &[(0, false)]);
        let picked = FloorsProgress {
            stock: 42,
            ..base.clone()
        };
        assert!(base.progressed(&picked));
        assert!(!picked.progressed(&picked.clone()));
        assert!(
            !base.progressed(&base.clone()),
            "sans le drapeau, stock reste 0"
        );
    }
}
