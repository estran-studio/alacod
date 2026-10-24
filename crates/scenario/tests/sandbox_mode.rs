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
    quiet_remote();
    let (registry, manifest, errors) =
        content::load_and_lint(&scenario::runner::game_dir(game)).expect("manifeste");
    assert!(errors.is_empty(), "{game} : {errors:#?}");
    let wave_mode = !registry.waves.is_empty();
    let core_plugin = CoreSetupPlugin(CoreSetupConfig {
        app_name: "sandbox".into(),
        headless: true,
        asset_root: Some(scenario::runner::assets_dir(game)),
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
    // Arène assemblée par graine : toujours la même (`ARENA_SEED`), le joueur 0 en (424, 696) comme dans les
    // scénarios, donc les zombies posés à sa droite le sont à l'intérieur de l'arène.
    run_to(&mut app, 3);
    let start = snapshot(&mut app).position;
    assert!(
        (start.0 - 424.0).abs() < 1.0 && (start.1 - 696.0).abs() < 1.0,
        "{start:?}"
    );
    run_to(&mut app, 700);

    // Bac à sable : mode `Sandbox`, aucune vague (zombies en déclare : sans la coupure, la vague 1 apparaît avant
    // la frame 700, voir l'essai headless du rapport), l'arène du manifeste, le profil posé.
    assert_eq!(run_mode(&app), run::RunMode::Sandbox);
    assert!(!app.world().resource::<ResolvedBalance>().waves_enabled);
    assert_eq!(
        app.world()
            .resource::<map::generation::config::MapGenerationConfig>()
            .map_path,
        "exemples/test_map.ldtk"
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
    run_to(&mut app, 720);
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
    assert!(app.world().resource::<FrameCount>().frame < 700);
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
