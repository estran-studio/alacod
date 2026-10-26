//! SB-3 (`docs/conventions.md` §87, `docs/taches/plan-bac-a-sable.md` D2/D3/D5) : le mode bac à sable joué
//! **comme un jeu le lance** (`GameArgsPlugin` avec `sandbox: Some(..)` à la place de `--sandbox`, puis le même
//! assemblage que `games/*/src/main.rs`, sans une ligne de bac à sable) : arène du profil, `RunMode::Sandbox`,
//! pas de vagues, profil posé, **`restart` qui rejoue le profil**, changement de profil, don à chaud, et
//! personnage du joueur choisi par le profil (D126).

use bevy::prelude::*;
use game::{
    args::{GameArgs, GameArgsPlugin},
    balance::ResolvedBalance,
    character::{config::CharacterConfigKey, enemy::Enemy, player::Player},
    core::CoreSetupPlugin,
    grant::{give, Gift},
    jjrs::PlayerConfig,
    replay::{PlayerScript, Scenario},
    run_state::RunRequest,
    sandbox::SandboxSession,
    session::{AppState, CoreSetupConfig},
    waves::WaveModeEnabled,
    weapons::WeaponInventory,
};
use map_ldtk::{
    game::{
        local::{LdtkGameMap, LdtkLocalGamePlugin},
        profile::ScriptedPlacement,
    },
    plugins::LdtkRoguePlugin,
};
use run::currency::Currency;
use utils::frame::FrameCount;

/// Les apps de ces tests partagent un processus : le serveur remote du mode éditeur écoute sur un port
/// choisi par le système (`ALACOD_REMOTE_PORT=0`), jamais sur 15702.
fn quiet_remote() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| std::env::set_var("ALACOD_REMOTE_PORT", "0"));
}

/// L'app d'un jeu lancé avec `--sandbox <profile>` : l'assemblage de `games/<jeu>/src/main.rs`.
fn sandbox_app(game: &str, profile: &str) -> App {
    build_app(&scenario::runner::game_dir(game), profile, false)
}

/// Comme [`sandbox_app`] sur le dossier `dir`, avec `GameRoot` (comme les `main.rs` natifs) : le registre et
/// le manifeste se relisent sur le disque (`alacod/reload_content`).
fn sandbox_app_with_root(dir: &std::path::Path, profile: &str) -> App {
    build_app(dir, profile, true)
}

fn build_app(dir: &std::path::Path, profile: &str, game_root: bool) -> App {
    quiet_remote();
    let game = dir.display();
    let (registry, manifest, errors) = content::load_and_lint(dir).expect("manifeste");
    assert!(errors.is_empty(), "{game} : {errors:#?}");
    let wave_mode = !registry.waves.is_empty();
    let core_plugin = CoreSetupPlugin(CoreSetupConfig {
        app_name: "sandbox".into(),
        headless: true,
        asset_root: Some(dir.join("assets").to_string_lossy().into_owned()),
        editor: false,
    });
    let args = GameArgs {
        seed: None,
        local_port: 0,
        number_player: 0,
        players: vec![PlayerConfig {
            name: "Joueur".into(),
            pubkey: "local".into(),
            is_local: true,
        }],
        spectators: vec![],
        matchbox: String::new(),
        allumette: String::new(),
        lobby: String::new(),
        cid: "sandbox".into(),
        debug_ai: false,
        sandbox: Some(profile.to_string()),
        telemetry: false,
        telemetry_url: String::new(),
        telemetry_auth: String::new(),
        check_distance: 0,
    };
    let mut app = App::new();
    if game_root {
        app.insert_resource(game::content_hot_reload::GameRoot(dir.to_path_buf()));
    }
    app.add_plugins(core_plugin.get_default_plugin())
        .add_plugins(GameArgsPlugin(args))
        .insert_resource(registry)
        .insert_resource(manifest.clone())
        .add_plugins(core_plugin)
        .add_plugins(LdtkRoguePlugin)
        .add_plugins(LdtkLocalGamePlugin(LdtkGameMap {
            map_path: manifest.entry.start_map.clone(),
            seed: manifest.entry.default_seed,
            settings: manifest.entry.map.clone(),
        }))
        .insert_resource(WaveModeEnabled(wave_mode));
    app.finish();
    app.cleanup();
    app
}

const MAX_UPDATES: u32 = 4000;

/// Avance jusqu'à ce que la simulation ait atteint `frame` (en jeu).
fn run_to(app: &mut App, frame: u32) {
    for _ in 0..MAX_UPDATES {
        app.update();
        let in_game = *app.world().resource::<State<AppState>>().get() == AppState::InGame;
        if in_game && app.world().resource::<FrameCount>().frame >= frame {
            return;
        }
    }
    panic!("frame {frame} non atteinte en {MAX_UPDATES} updates");
}

/// Demande `request` puis avance jusqu'à ce que la nouvelle partie ait `frame` frames (la précédente est
/// quittée : on attend de repasser par `GameLoading`).
fn restart_and_run_to(app: &mut App, frame: u32) {
    app.world_mut().insert_resource(RunRequest::Restart);
    let mut left = false;
    for _ in 0..MAX_UPDATES {
        app.update();
        let state = app.world().resource::<State<AppState>>().get().clone();
        left |= state != AppState::InGame;
        if left && state == AppState::InGame && app.world().resource::<FrameCount>().frame >= frame
        {
            return;
        }
    }
    panic!("restart : frame {frame} non atteinte");
}

struct Snapshot {
    currency: u32,
    weapons: Vec<String>,
    active_weapon: Option<String>,
    character: String,
    placed: usize,
    enemies: usize,
    position: (f32, f32),
}

fn snapshot(app: &mut App) -> Snapshot {
    let world = app.world_mut();
    let mut players = world.query::<(
        &Player,
        &Currency,
        &WeaponInventory,
        &CharacterConfigKey,
        &bevy_fixed::fixed_math::FixedTransform3D,
    )>();
    let (_, currency, inventory, key, transform) = players
        .iter(world)
        .find(|(player, ..)| player.handle == 0)
        .expect("joueur 0");
    let weapons: Vec<String> = inventory
        .weapons
        .iter()
        .map(|(_, weapon)| weapon.config.name.clone())
        .collect();
    let active_weapon = inventory
        .weapons
        .get(inventory.active_weapon_index)
        .map(|(_, weapon)| weapon.config.name.clone());
    let (currency, character) = (currency.0, key.0.clone());
    let position = (
        bevy_fixed::fixed_math::to_f32(transform.translation.x),
        bevy_fixed::fixed_math::to_f32(transform.translation.y),
    );
    Snapshot {
        position,
        currency,
        weapons,
        active_weapon,
        character,
        placed: world.query::<&ScriptedPlacement>().iter(world).count(),
        enemies: world.query::<&Enemy>().iter(world).count(),
    }
}

fn run_mode(app: &App) -> run::RunMode {
    app.world().resource::<run::Run>().mode.clone()
}

/// Le profil `lourd` de zombies : arme, solde, modificateur, trois zombies posés. `restart` le **rejoue** ;
/// un don à chaud (`alacod/give`) est perdu au `restart` ; un autre profil se charge en changeant la session.
#[test]
fn zombies_lourd_se_pose_se_rejoue_au_restart_et_se_change() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let mut app = sandbox_app("zombies", "lourd");
    // Arène vide (`maps/arene_vide.ldtk`, `alacod new-arena`) assemblée avec la graine `ARENA_SEED` : toujours la
    // même, le joueur 0 en (424, 696) comme dans les scénarios, donc les zombies posés à sa droite le sont à
    // l'intérieur de l'arène.
    run_to(&mut app, 3);
    let start = snapshot(&mut app).position;
    assert!(
        (start.0 - 424.0).abs() < 1.0 && (start.1 - 696.0).abs() < 1.0,
        "{start:?}"
    );
    // Les zombies posés par `lourd` se jettent sur le joueur immobile de l'arène vide (un seul joueur, aucune
    // entrée) : on lit l'état tôt, avant qu'il ne tombe.
    run_to(&mut app, 90);

    // Bac à sable : mode `Sandbox`, aucune vague (`waves_enabled` coupé : zombies en déclare ; l'arène vide n'a
    // de toute façon aucun point d'apparition), l'arène du manifeste, le profil posé.
    assert_eq!(run_mode(&app), run::RunMode::Sandbox);
    assert!(!app.world().resource::<ResolvedBalance>().waves_enabled);
    assert_eq!(
        app.world()
            .resource::<map::generation::config::MapGenerationConfig>()
            .map_path,
        "maps/arene_vide.ldtk"
    );
    let first = snapshot(&mut app);
    assert_eq!(first.currency, 50000);
    assert_eq!(first.weapons, ["machine_gun"]);
    assert_eq!(first.character, "player");
    assert_eq!(first.placed, 3, "trois zombies posés par le profil");
    assert_eq!(first.enemies, 3, "aucun ennemi de vague");

    // Don à chaud : s'ajoute à l'équipement, devient l'arme active.
    let report = give(
        app.world_mut(),
        0,
        &Gift {
            weapon: Some("shotgun".into()),
            currency: Some(25),
            ..Default::default()
        },
    )
    .expect("don");
    assert_eq!(report.currency, Some(50025));
    run_to(&mut app, 110);
    let given = snapshot(&mut app);
    assert_eq!(given.currency, 50025);
    assert_eq!(
        given.weapons,
        ["machine_gun", "shotgun"],
        "ajoutée, pas remplaçante"
    );
    assert_eq!(given.active_weapon.as_deref(), Some("shotgun"));

    // `restart` : le profil se rejoue sur le joueur recréé, le don est perdu, les zombies sont reposés.
    restart_and_run_to(&mut app, 30);
    assert!(app.world().resource::<FrameCount>().frame < 90);
    let again = snapshot(&mut app);
    assert_eq!(again.currency, 50000, "profil rejoué, don perdu");
    assert_eq!(again.weapons, ["machine_gun"]);
    assert_eq!(again.placed, 3);
    assert_eq!(again.enemies, 3);

    // Changer de profil : la session change, `Restart` pose l'autre (`arene` : solde 10000, rien de posé).
    app.world_mut().insert_resource(SandboxSession {
        profile: "arene".into(),
    });
    restart_and_run_to(&mut app, 30);
    let arene = snapshot(&mut app);
    assert_eq!(arene.currency, 10000);
    assert_eq!(arene.placed, 0);
    assert_eq!(arene.enemies, 0);
    assert_ne!(
        arene.weapons,
        ["machine_gun"],
        "armes de départ du personnage"
    );
}

/// Chaque jeu livré s'ouvre en bac à sable sur son profil `arene` : joueur posé, `Sandbox`, pas de vague.
macro_rules! arene_test {
    ($name:ident, $game:expr) => {
        #[test]
        fn $name() {
            if map_ldtk::RENDER_ENABLED {
                return;
            }
            let mut app = sandbox_app($game, "arene");
            run_to(&mut app, 200);
            assert_eq!(run_mode(&app), run::RunMode::Sandbox);
            assert!(!app.world().resource::<ResolvedBalance>().waves_enabled);
            let (_, manifest, _) =
                content::load_and_lint(&scenario::runner::game_dir($game)).unwrap();
            let profile = content::sandbox::DEFAULT_PROFILE;
            let registry = app.world().resource::<content::Registry>();
            let arena = registry.sandbox_profiles[profile]
                .profile
                .arena(&manifest)
                .unwrap();
            assert!(
                arena.ends_with("arene_vide.ldtk"),
                "l'arène vide, pas une carte de jeu : {arena}"
            );
            assert_eq!(
                app.world()
                    .resource::<map::generation::config::MapGenerationConfig>()
                    .map_path,
                arena
            );
            let session = app.world().resource::<SandboxSession>();
            assert_eq!(session.profile, profile);
            let snapshot = snapshot(&mut app);
            assert_eq!(snapshot.character, "player");
            assert_eq!(snapshot.enemies, 0, "arène vide : aucun ennemi");
            assert_eq!(snapshot.placed, 0, "le profil `arene` ne pose rien");
        }
    };
}

arene_test!(zombies_arene_s_ouvre, "zombies");
arene_test!(gungeon_arene_s_ouvre, "gungeon");
arene_test!(isaac_arene_s_ouvre, "isaac");
arene_test!(throne_arene_s_ouvre, "throne");
arene_test!(testbed_arene_s_ouvre, "testbed");

/// D126 : le personnage du joueur est celui du profil (`PlayerScript::character`), posé à la création ; sans
/// champ, le personnage par défaut. Même chemin que le bac à sable (`PlayerProfile::character`).
#[test]
fn le_profil_choisit_le_personnage_du_joueur() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let character_of = |character: Option<&str>| {
        let mut scenario = Scenario::from_ron(
            "Scenario(game: \"zombies\", map: \"exemples/test_map.ldtk\", frames: 20, players: [], mode: Sandbox)",
        )
        .expect("scénario");
        scenario.players.push(PlayerScript {
            character: character.map(str::to_string),
            ..Default::default()
        });
        let mut app = scenario::runner::run_until(&scenario, 5);
        let world = app.world_mut();
        let mut players = world.query::<(
            &Player,
            &CharacterConfigKey,
            &game::character::health::Health,
        )>();
        let (_, key, health) = players.iter(world).next().expect("joueur");
        (key.0.clone(), health.max)
    };
    let (default, default_health) = character_of(None);
    let (chosen, chosen_health) = character_of(Some("zombie_full"));
    assert_eq!(default, "player");
    assert_eq!(chosen, "zombie_full");
    assert_ne!(
        default_health, chosen_health,
        "la santé max est celle du personnage choisi (`ResolvedBalance::health_max_by_character`)"
    );
}

// --- Rechargement du contenu : la session prime sur le disque -----------------------------------

/// Copie récursive d'un dossier de jeu dans un dossier temporaire neuf (le test édite des RON).
fn copy_game(game: &str, tag: &str) -> std::path::PathBuf {
    fn copy(from: &std::path::Path, to: &std::path::Path) {
        std::fs::create_dir_all(to).unwrap();
        for entry in std::fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy(&entry.path(), &target);
            } else {
                std::fs::copy(entry.path(), target).unwrap();
            }
        }
    }
    let dir = std::env::temp_dir().join(format!("alacod-sandbox-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    copy(&scenario::runner::game_dir(game), &dir);
    dir
}

fn floor_plan_present(app: &App) -> bool {
    app.world()
        .contains_resource::<map_ldtk::game::floors::FloorPlan>()
}

fn manifest_mode(app: &App) -> Option<content::EntryMode> {
    app.world().resource::<content::GameManifest>().entry.mode
}

fn map_config(app: &App) -> map::generation::config::MapGenerationConfig {
    app.world()
        .resource::<map::generation::config::MapGenerationConfig>()
        .clone()
}

fn player_position(app: &mut App) -> (f32, f32) {
    snapshot(app).position
}

/// isaac déclare `entry.mode: Floors` : `reload` relit ce mode sur le disque. Un `restart` doit rester en bac à
/// sable (arène, pas les niveaux de la run), et le profil **relu** (modifié sur le disque) doit être posé.
#[test]
fn reload_puis_restart_gardent_le_mode_bac_a_sable() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let dir = copy_game("isaac", "reload-mode");
    let mut app = sandbox_app_with_root(&dir, "arene");
    run_to(&mut app, 30);
    assert_eq!(manifest_mode(&app), Some(content::EntryMode::Sandbox));
    assert!(!floor_plan_present(&app));
    assert_eq!(map_config(&app).map_path, "maps/arene_vide.ldtk");
    assert_eq!(run_mode(&app), run::RunMode::Sandbox);

    // Édition du profil actif sur le disque, puis `alacod/reload_content`.
    let profile = dir.join("assets/sandbox/arene.ron");
    std::fs::write(
        &profile,
        r#"(players: [(currency: 777)], characters: [(character: "cible", x: "624", y: "696", at_frame: 1)])"#,
    )
    .unwrap();
    let report = game::content_hot_reload::reload_registry_now(app.world_mut()).expect("reload");
    assert!(report.reloaded, "{:?}", report.errors);
    assert_eq!(
        app.world().resource::<content::GameManifest>().entry.mode,
        Some(content::EntryMode::Floors),
        "le disque dit `Floors` : c'est ce que `reload` a posé (c'est la session qui doit primer)"
    );

    restart_and_run_to(&mut app, 30);
    assert_eq!(manifest_mode(&app), Some(content::EntryMode::Sandbox));
    assert_eq!(run_mode(&app), run::RunMode::Sandbox);
    assert!(!floor_plan_present(&app), "aucun niveau de la run chargé");
    assert_eq!(map_config(&app).map_path, "maps/arene_vide.ldtk");
    let again = snapshot(&mut app);
    assert_eq!(again.currency, 777, "profil relu du disque");
    assert_eq!(again.placed, 1);
    assert!(
        (again.position.0 - 424.0).abs() < 1.0 && (again.position.1 - 696.0).abs() < 1.0,
        "joueur sur le point de départ de l'arène : {:?}",
        again.position
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// Un `map` ajouté au profil actif, puis `reload` + `restart` (qui va droit en `GameLoading`, sans lobby) :
/// la nouvelle arène est chargée **avant** la pose des placements, et la graine reste celle de la session.
#[test]
fn reload_puis_restart_prennent_l_arene_modifiee_du_profil() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let dir = copy_game("isaac", "reload-arena");
    // Seconde arène : la première avec ses points de départ décalés de 160 unités (10 cases) vers la droite
    // (le joueur y apparaît 160 plus loin).
    let source = std::fs::read_to_string(dir.join("assets/maps/arene_vide.ldtk")).unwrap();
    let mut project: serde_json::Value = serde_json::from_str(&source).unwrap();
    for level in project["levels"].as_array_mut().unwrap() {
        for layer in level["layerInstances"].as_array_mut().into_iter().flatten() {
            for entity in layer["entityInstances"]
                .as_array_mut()
                .into_iter()
                .flatten()
            {
                if entity["__identifier"] == "PlayerSpawn" {
                    entity["px"][0] = (entity["px"][0].as_i64().unwrap() + 160).into();
                    entity["__grid"][0] = (entity["__grid"][0].as_i64().unwrap() + 10).into();
                }
            }
        }
    }
    std::fs::write(
        dir.join("assets/maps/arene_decalee.ldtk"),
        serde_json::to_string(&project).unwrap(),
    )
    .unwrap();

    let mut app = sandbox_app_with_root(&dir, "arene");
    run_to(&mut app, 30);
    assert_eq!(map_config(&app).map_path, "maps/arene_vide.ldtk");
    let before = player_position(&mut app);
    assert!((before.0 - 424.0).abs() < 1.0, "{before:?}");

    // Édition du profil actif sur le disque, puis `reload` et `restart`.
    std::fs::write(
        dir.join("assets/sandbox/arene.ron"),
        r#"(map: "maps/arene_decalee.ldtk", characters: [(character: "cible", x: "944", y: "696", at_frame: 1)])"#,
    )
    .unwrap();
    let report = game::content_hot_reload::reload_registry_now(app.world_mut()).expect("reload");
    assert!(report.reloaded, "{:?}", report.errors);
    let seed = map_config(&app).seed;
    restart_and_run_to(&mut app, 30);

    let config = map_config(&app);
    assert_eq!(config.map_path, "maps/arene_decalee.ldtk");
    // (Une arène assemblée par graine change avec elle : la graine reste celle d'avant le `restart`, ce
    // qui permet de comparer les positions. Le test unitaire `pin_session_…` prouve une graine de 7 intacte.)
    assert_eq!(config.seed, seed, "la graine n'est pas écrasée");
    assert!(!floor_plan_present(&app));
    let after = player_position(&mut app);
    assert!(
        (after.0 - (before.0 + 160.0)).abs() < 1.0 && (after.1 - before.1).abs() < 1.0,
        "le joueur apparaît sur la nouvelle arène : avant {before:?}, après {after:?}"
    );
    assert_eq!(
        snapshot(&mut app).placed,
        1,
        "placement posé sur la nouvelle carte"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// Manifeste rechargé **après** l'`Update` qui demande `GameStarting` : `OnEnter(GameStarting)` passe avant
/// `Update` et résout le mode de la `Run` depuis le manifeste du disque (`Floors` pour isaac). La session doit primer.
#[test]
fn un_rechargement_avant_game_starting_ne_change_pas_le_mode_de_la_run() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let dir = copy_game("isaac", "reload-starting");
    let mut app = sandbox_app_with_root(&dir, "arene");
    run_to(&mut app, 5);
    app.world_mut().insert_resource(RunRequest::Restart);
    let mut injected = false;
    for _ in 0..MAX_UPDATES {
        app.update();
        let state = app.world().resource::<State<AppState>>().get().clone();
        // `GameStarting` est demandé (état suivant), pas encore entré : on remet le disque ici.
        let next = app.world().resource::<NextState<AppState>>();
        if !injected && matches!(next, NextState::Pending(AppState::GameStarting)) {
            let report =
                game::content_hot_reload::reload_registry_now(app.world_mut()).expect("reload");
            assert!(report.reloaded, "{:?}", report.errors);
            assert_eq!(manifest_mode(&app), Some(content::EntryMode::Floors));
            injected = true;
        }
        if injected && state == AppState::InGame {
            break;
        }
    }
    assert!(injected, "GameStarting jamais demandé");
    assert_eq!(run_mode(&app), run::RunMode::Sandbox);
    assert!(!floor_plan_present(&app));
    let _ = std::fs::remove_dir_all(&dir);
}
