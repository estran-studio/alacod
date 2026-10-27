//! Entrée unique d'un jeu (X-1, D-e, `docs/conventions.md` §91) : le `main.rs` d'un jeu déclare son
//! nom, son dossier et ses plugins ; [`run`] fait le reste. Un seul chemin sert :
//!
//! - la **partie** (`<jeu>`, `--sandbox [profil]`, `--players`, p2p…) ;
//! - le **runner de scénarios** : `<jeu> scenarios gen|play …` joue et génère les scénarios du jeu,
//!   plugin compris, avec leurs traces (`scenario::cli`, le code d'`alacod-gen`) ;
//! - le **lint** du jeu : `<jeu> lint [--json]`, règles Rust du plugin comprises ;
//! - les bots (`BotsPlugin` du runner) : ils jouent le jeu qui porte le plugin.
//!
//! `run` installe le plugin (`game::extension::install`) **avant** de charger le contenu : le lint
//! compare ce que `game.ron` déclare à ce que le code fournit, `CoreSimPlugin` pose les types rollback
//! du plugin avant sa clôture. Voir `game::extension` pour le contrat d'un plugin.

use std::sync::Arc;

use bevy::prelude::*;
use game::{
    args::BaseArgsPlugin,
    core::CoreSetupPlugin,
    extension::{self, GamePlugin},
    session::CoreSetupConfig,
    waves::WaveModeEnabled,
};
use map_ldtk::{
    game::local::{LdtkGameMap, LdtkLocalGamePlugin},
    plugins::LdtkRoguePlugin,
};

#[cfg(not(target_arch = "wasm32"))]
mod native;

/// D'où vient le contenu du jeu.
enum Source {
    /// Dossier du jeu (contient `assets/game.ron`) : natif.
    #[cfg(not(target_arch = "wasm32"))]
    Dir(std::path::PathBuf),
    /// Contenu embarqué dans le binaire web et version du build (chemin HTTP des assets).
    #[cfg(target_arch = "wasm32")]
    Embedded {
        files: content::files::EmbeddedFiles,
        app_version: &'static str,
    },
}

/// Ce qu'un jeu déclare : son nom, d'où vient son contenu, ses plugins.
pub struct GameSpec {
    name: String,
    source: Source,
    plugins: Vec<Arc<dyn GamePlugin>>,
}

impl GameSpec {
    /// Jeu natif. `manifest_dir` : `env!("CARGO_MANIFEST_DIR")` du binaire du jeu, capturé à la
    /// compilation (fiable même si le binaire est lancé hors de `cargo run`, `docs/conventions.md` §3).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(name: &str, manifest_dir: &str) -> Self {
        Self {
            name: name.to_string(),
            source: Source::Dir(manifest_dir.into()),
            plugins: Vec::new(),
        }
    }

    /// Jeu web : contenu embarqué par `scripts/embed-game-content.rs`, `app_version` =
    /// `env!("APP_VERSION")` (les assets binaires sont servis sous `/builds/<version>/<jeu>/assets`).
    #[cfg(target_arch = "wasm32")]
    pub fn embedded(
        name: &str,
        files: content::files::EmbeddedFiles,
        app_version: &'static str,
    ) -> Self {
        Self {
            name: name.to_string(),
            source: Source::Embedded { files, app_version },
            plugins: Vec::new(),
        }
    }

    /// Ajoute un plugin de jeu (`game::extension::GamePlugin`). L'ordre est l'ordre de `build`.
    pub fn plugin(mut self, plugin: impl GamePlugin) -> Self {
        self.plugins.push(Arc::new(plugin));
        self
    }
}

/// Lance le jeu. Retourne l'`AppExit` : en headless, un desync p2p sort en code 3
/// (`SESSION_LOST_EXIT_CODE`), une trace écrite en 0.
pub fn run(spec: GameSpec) -> AppExit {
    extension::install(spec.plugins.clone());

    #[cfg(not(target_arch = "wasm32"))]
    if let Some(code) = native::tooling_mode(&spec) {
        return code;
    }

    let name = spec.name.clone();
    // Manifeste, registre et lint (T1.5) : refuse de démarrer sur contenu invalide, exactement comme
    // `alacod lint <jeu>` (même code, `content::load_and_lint`), règles du plugin en plus.
    #[cfg(not(target_arch = "wasm32"))]
    let Source::Dir(game_dir) = &spec.source;
    #[cfg(not(target_arch = "wasm32"))]
    let loaded = content::load_and_lint(game_dir);
    #[cfg(target_arch = "wasm32")]
    let Source::Embedded { files, app_version } = &spec.source;
    #[cfg(target_arch = "wasm32")]
    let loaded = content::load_embedded(files);
    let (registry, manifest, errors) = loaded.unwrap_or_else(|e| {
        eprintln!("{name} : {e}");
        std::process::exit(1);
    });
    if !errors.is_empty() {
        for e in &errors {
            eprintln!("{e}");
        }
        eprintln!(
            "{name} : {} erreur(s) de contenu, arrêt (voir `alacod lint {name}`)",
            errors.len()
        );
        std::process::exit(1);
    }
    // Mode vagues activé si (et seulement si) le jeu déclare un dossier de contenu `Wave`.
    let wave_mode = !registry.waves.is_empty();

    let mut game_config = CoreSetupConfig::from_env(name.as_str());
    // Assets du jeu : chemin explicite (dossier du manifeste capturé à la compilation), pour que le
    // binaire lancé hors de `cargo run` (CI de nuit, p2p headless) les trouve aussi.
    #[cfg(not(target_arch = "wasm32"))]
    {
        game_config.asset_root = Some(game_dir.join("assets").to_string_lossy().into_owned());
    }
    #[cfg(target_arch = "wasm32")]
    {
        game_config.asset_root = Some(format!("/builds/{app_version}/{name}/assets"));
    }

    let core_plugin = CoreSetupPlugin(game_config);

    let mut app = App::new();
    #[cfg(not(target_arch = "wasm32"))]
    app.insert_resource(game::content_hot_reload::GameRoot(game_dir.clone()));
    app
        // Configure the bevy default plugins from our core_plugin configuration
        .add_plugins(core_plugin.get_default_plugin())
        // Load default arguments from cli or query params (MUST be before core_plugin for --debug-ai to work)
        .add_plugins(BaseArgsPlugin)
        .insert_resource(registry)
        // T2.4, chantier F1 : `game::jjrs::{local, p2p}` résout `RunMode` depuis `entry.mode` au
        // démarrage de session (voir `game::run_state::resolve_run_mode`).
        .insert_resource(manifest.clone())
        // Core systems and components (les `build` des plugins du jeu : avant la clôture rollback)
        .add_plugins(core_plugin)
        // Plugins for rogue like map with ldtk
        .add_plugins(LdtkRoguePlugin)
        // Map, seed and player spawning (shared with the scenario tests). Point d'entrée du manifeste
        // (`game.ron`, T1.5) plutôt qu'en dur.
        .add_plugins(LdtkLocalGamePlugin(LdtkGameMap {
            map_path: manifest.entry.start_map.clone(),
            seed: manifest.entry.default_seed,
            settings: manifest.entry.map.clone(),
        }))
        .insert_resource(WaveModeEnabled(wave_mode));
    // M2-T13 : la partie démarre au hub quand le manifeste en déclare un (`entry.hub`) ; la porte
    // lance la run (`entry.start_map`, mode `Floors` du manifeste).
    // `ALACOD_SKIP_HUB=1` (natif, preuves p2p) : la partie démarre directement dans la run, comme
    // `alacod-sim` et les scénarios sans champ `hub` ; les inputs enregistrés d'un scénario de
    // bots s'y rejouent à l'identique (la graine de run d'une partie en ligne passée par le hub est
    // dérivée de celle du hub, §33, celle d'un scénario non).
    let skip_hub = std::env::var("ALACOD_SKIP_HUB").is_ok_and(|v| v == "1");
    if let Some(hub) = manifest.entry.hub.as_ref().filter(|_| !skip_hub) {
        app.insert_resource(game::hub::HubSession {
            hub_map: hub.map.clone(),
            run_map: manifest.entry.start_map.clone(),
            in_hub: true,
        });
    }
    // Reste de la partie jouée propre au jeu (ressources, systèmes `Update`, présentation).
    extension::assemble_plugins(&mut app);
    // `--seed <n>` (graine de carte explicite, preuves p2p) : `map_ldtk::game::local::apply_launch_seed`.
    app.run()
}
