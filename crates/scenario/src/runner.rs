//! Exécution headless d'un scénario, dans le processus courant.

use crate::events::{GameEvent, GameEvents, GameEventsPlugin};
use crate::invariants::InvariantQueries;
use bevy::prelude::*;
use bevy_fixed::fixed_math;
use bevy_ggrs::SyncTestMismatch;
use bots::{BotAssignments, BotsPlugin};
use combat::damage::Defenses;
use combat::inventory::AmmoReserves;
use game::recording::InputRecorder;
use game::{
    args::{GameArgs, GameArgsPlugin},
    character::player::{
        input::{InputSource, ScriptedInputs},
        Player,
    },
    core::{CoreSetupConfig, CoreSetupPlugin},
    jjrs::PlayerConfig,
    state_trace::{StateTraceRecorder, StateTraceRecorderPlugin},
    waves::{WaveDebugEnabled, WaveModeEnabled, WaveState},
};
use map::generation::config::MapGenerationConfig;
use map_ldtk::{
    game::local::{LdtkGameMap, LdtkLocalGamePlugin},
    plugins::LdtkRoguePlugin,
};
use run::currency::Currency;
use serde::{Deserialize, Serialize};
use sim_core::modifier::{resolve, Modifier, ModifierSource, Modifiers};
use sim_core::stats::{StatId, Stats};
use sim_core::tag::Tags;
use std::collections::BTreeMap;
use std::path::PathBuf;
use utils::frame::FrameCount;

use game::global_asset::GlobalAsset;
use game::replay::{Expectation, ModifierSpec, Scenario, WaveOverride};
use game::weapons::melee::{self, MeleeWeapon, MeleeWeaponsConfig};
use game::weapons::{spawn_weapon_for_player, Weapon, WeaponInventory, WeaponsConfig};
use utils::net_id::GgrsNetIdFactory;

/// Mismatches de synctest : le premier seulement (GGRS répète ensuite le même à chaque
/// frame et n'avance plus), avec les frames qu'il incrimine.
#[derive(Resource, Default)]
struct SyncTestMismatches {
    messages: Vec<String>,
    frames: Vec<i32>,
}

/// Updates maximum pour charger la map avant la première frame de simulation.
const MAX_LOADING_UPDATES: u32 = 10_000;

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
}

/// Résultat d'un scénario.
pub struct ScenarioOutcome {
    /// Trace d'état, une ligne par frame (voir `game::state_trace`).
    pub trace: Vec<String>,
    /// Trace détaillée de toutes les frames (hash + détail), si `ALACOD_DUMP_TRACE` était
    /// défini pour ce run (outil de preuve permanent, T1.2, `docs/conventions.md` §8).
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
}

/// Dossier racine du jeu (`games/<jeu>`), pour `content::load_and_lint` (T1.5).
pub fn game_dir(game: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(format!(
        "{}/../../games/{}",
        env!("CARGO_MANIFEST_DIR"),
        game
    ))
}

/// Dossier des assets du jeu.
pub fn assets_dir(game: &str) -> String {
    format!("{}/../../games/{}/assets", env!("CARGO_MANIFEST_DIR"), game)
}

/// Configuration de lecture d'un scénario.
pub struct PlayConfig {
    /// Si présent, force la caméra à suivre le joueur avec ce handle GGRS.
    pub follow_handle: Option<usize>,
}

/// App de la partie décrite par le scénario (même partie que `zombies`).
/// Avec `headless: false`, la partie est affichée (voir le binaire `play_scenario`).
pub fn build_app(scenario: &Scenario, headless: bool, config: &PlayConfig) -> App {
    let core_plugin = CoreSetupPlugin(CoreSetupConfig {
        app_name: "scenario".into(),
        headless,
        asset_root: Some(assets_dir(&scenario.game)),
    });

    // Registre de contenu (T1.5) : mêmes règles que `alacod lint`, un contenu invalide
    // fait échouer le test tout de suite plutôt qu'en plein milieu de la simulation.
    let game_root = game_dir(&scenario.game);
    let (registry, _manifest, content_errors) = content::load_and_lint(&game_root)
        .unwrap_or_else(|e| panic!("scénario « {} » : game.ron invalide : {e}", scenario.game));
    assert!(
        content_errors.is_empty(),
        "scénario « {} » : contenu invalide :\n{:#?}",
        scenario.game,
        content_errors
    );

    let mut app = App::new();
    app.add_plugins(core_plugin.get_default_plugin())
        .add_plugins(GameArgsPlugin(game_args(scenario.players.len())))
        .insert_resource(registry)
        .add_plugins(core_plugin)
        .add_plugins(LdtkRoguePlugin)
        .add_plugins(LdtkLocalGamePlugin(LdtkGameMap {
            map_path: scenario.map.clone(),
            seed: scenario.map_seed,
        }))
        .insert_resource(WaveModeEnabled(true))
        .insert_resource(WaveDebugEnabled(true))
        // ALACOD_DIAG=1 : trace détaillée, pour nommer le composant qui diverge en synctest
        // ALACOD_DUMP_TRACE=<dossier> : outil de preuve permanent (T1.2), voir
        // `docs/conventions.md` §8 et `crates/scenario/tests/scenarios.rs` (qui écrit
        // `<dossier>/<scénario>.full` depuis `ScenarioOutcome::full_trace`).
        .add_plugins(StateTraceRecorderPlugin {
            full: std::env::var("ALACOD_DIAG").is_ok_and(|v| v == "1"),
            dump: std::env::var("ALACOD_DUMP_TRACE").ok().map(PathBuf::from),
        })
        .add_plugins(GameEventsPlugin)
        .insert_resource(WeaponOverrides(scenario.weapon_overrides.clone()))
        .add_systems(Update, apply_weapon_overrides)
        .insert_resource(WaveOverrideRes(scenario.wave_overrides.clone()))
        .add_systems(Update, apply_wave_overrides)
        .insert_resource(PlayerOverrides(
            scenario
                .players
                .iter()
                .map(|p| PlayerOverride {
                    tags: p.tags.clone(),
                    immune_to: p.immune_to.clone(),
                    modifiers: p.modifiers.clone(),
                    weapon: p.weapon.clone(),
                    currency: p.currency,
                })
                .collect(),
        ))
        // `.before(weapon_inventory_system)` : quand `weapon` despawn/recrée les armes du
        // joueur (voir la doc de `apply_player_overrides`), `weapon_inventory_system` (Update,
        // `BaseWeaponGamePlugin`) ne doit jamais lire l'ancien `WeaponInventory` (armes déjà
        // despawnées) dans la même frame — Bevy insère un `ApplyDeferred` entre deux systèmes
        // ordonnés (`auto_insert_apply_deferred`, par défaut), ce qui suffit ici : pas besoin
        // d'ordonner aussi par rapport à `spawn_players_when_map_loaded` (voir la doc de la
        // fonction, inchangé : elle attend déjà que `Player` existe).
        .add_systems(
            Update,
            apply_player_overrides.before(game::weapons::weapon_inventory_system),
        )
        .insert_resource(InputSource::Scripted)
        .insert_resource::<ScriptedInputs>(scenario.scripted_inputs())
        // T2.11 : toujours ajouté (BotAssignments vide ⇒ read_bot_inputs ne fait rien, les
        // scénarios sans bot ne changent pas) ; le mode de base reste Scripted (au-dessus) pour
        // que les joueurs scriptés d'un scénario qui mélange les deux gardent leurs inputs, les
        // joueurs de `bot_assignments` sont remplacés ensuite quel que soit ce mode.
        .add_plugins(BotsPlugin)
        .insert_resource(bot_assignments(scenario))
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

    // Caméra forcée sur un joueur (play_scenario --follow)
    if let Some(handle) = config.follow_handle {
        app.insert_resource(game::camera::CameraFollowOverride(handle));
    }

    app
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

#[derive(Resource)]
struct WeaponOverrides(Vec<game::replay::WeaponOverride>);

/// Applique les modifications d'armes du scénario dès que la config est chargée, avant la
/// création des joueurs (qui copient la config de leurs armes).
fn apply_weapon_overrides(
    overrides: Res<WeaponOverrides>,
    global_assets: Option<Res<game::global_asset::GlobalAsset>>,
    mut weapons: ResMut<Assets<game::weapons::WeaponsConfig>>,
    mut applied: Local<bool>,
) {
    if *applied || overrides.0.is_empty() {
        return;
    }
    let Some(mut config) = global_assets.and_then(|g| weapons.get_mut(&g.weapons)) else {
        return;
    };
    for o in &overrides.0 {
        let weapon = config
            .0
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
/// l'asset est chargé, avant que `wave_state_machine_system` ne calcule la taille de la
/// première vague (qui lit cette config).
fn apply_wave_overrides(
    overrides: Res<WaveOverrideRes>,
    global_assets: Option<Res<game::global_asset::GlobalAsset>>,
    mut wave_configs: ResMut<Assets<game::waves::config::WaveConfig>>,
    mut applied: Local<bool>,
) {
    if *applied {
        return;
    }
    let Some(o) = overrides.0.as_ref() else {
        *applied = true;
        return;
    };
    let Some(handle) = global_assets.and_then(|g| g.wave_config.clone()) else {
        return;
    };
    let Some(mut config) = wave_configs.get_mut(&handle) else {
        return;
    };
    if let Some(v) = o.base_enemies {
        config.base_enemies = v;
    }
    if let Some(v) = o.enemies_per_wave {
        config.enemies_per_wave = v;
    }
    if let Some(v) = o.grace_period_frames {
        config.grace_period_frames = v;
    }
    if let Some(v) = o.max_concurrent_enemies {
        config.max_concurrent_enemies = v;
    }
    if let Some(v) = o.spawn_batch_size {
        config.spawn_batch_size = v;
    }
    if let Some(v) = o.spawn_interval_frames {
        config.spawn_interval_frames = v;
    }
    *applied = true;
}

/// Condition d'arrêt anticipé du runner (`alacod-sim`, T2.11) : la simulation s'arrête avant
/// `scenario.frames` dès qu'une condition est atteinte. Contrairement à une simulation qui
/// n'atteint pas `scenario.frames` sans condition d'arrêt (échec, signe habituel d'un problème
/// de chargement), un arrêt anticipé volontaire n'est jamais une failure.
#[derive(Debug, Clone, Copy, Default)]
pub struct StopEarly {
    /// S'arrête dès que `WaveState::current_wave >= until_wave`.
    pub until_wave: Option<u32>,
    /// S'arrête dès qu'aucun joueur n'est vivant (à partir de la première frame simulée).
    pub stop_when_all_players_dead: bool,
}

/// Tags, immunités, modificateurs de stats et choix d'arme par joueur (T1.1 chantier B1,
/// T1.2 chantier B2, T2.10 générateur) — voir `game::replay::PlayerScript::{tags,
/// immune_to, modifiers, weapon}`.
struct PlayerOverride {
    tags: Vec<String>,
    immune_to: Vec<String>,
    modifiers: Vec<ModifierSpec>,
    weapon: Option<String>,
    /// T2.3, chantier C5 v1 (scénarios d'achat) : voir `game::replay::PlayerScript::currency`.
    currency: Option<u32>,
}

#[derive(Resource)]
struct PlayerOverrides(Vec<PlayerOverride>);

/// Pose les `tags`/`immune_to`/`modifiers`/`weapon` d'un scénario sur les joueurs une fois
/// créés (composants `Tags`/`Defenses`/`Modifiers`/`WeaponInventory`). Contrairement à
/// `apply_weapon_overrides` (qui doit s'appliquer *avant* la création des joueurs, qui
/// copient la config de leurs armes), celui-ci s'applique *après* : il attend que les
/// entités `Player` existent, puis pose les composants une seule fois (`Local<bool>`).
/// `Tags`/`Defenses` sont hors rollback (voir leur doc) : les poser ici, avant la première
/// frame simulée, suffit — ils ne sont jamais mutés ensuite. `Modifiers`/`WeaponInventory`
/// sont en rollback, mais poser leur valeur initiale avant la première frame simulée est
/// tout aussi correct : elle est identique sur tous les clients (même scénario), donc fait
/// partie de l'état de départ comme les autres — `weapon` n'ajoute donc rien de nouveau au
/// checksum GGRS, ce n'est qu'un choix d'inventaire à la création (voir la doc de
/// `PlayerScript::weapon`).
///
/// `weapon` : le joueur apparaît avec **cette seule arme** (à distance ou de mêlée, id du
/// registre), à la place de la totalité de son équipement par défaut — exclusivité complète
/// (les armes à distance par défaut du personnage *et* son arme de mêlée par défaut,
/// `bare_hands`, sont retirées, même si `weapon` désigne une arme à distance). Décision
/// (rapport T2.10) : plus simple et plus sûr pour un scénario généré isolé par arme (aucun
/// risque qu'un coup de mêlée ou un tir parasite d'une arme de départ non retirée fausse
/// `EntityHits`) que de ne retirer que la catégorie choisie. Cherche `weapon_id` d'abord
/// dans `weapons.ron` (arme à distance), puis dans `melee_weapons.ron` (arme de mêlée) ;
/// panique si absent des deux (même style que `apply_weapon_overrides`, faute de contenu
/// plutôt que silencieuse).
#[allow(clippy::too_many_arguments)]
fn apply_player_overrides(
    overrides: Res<PlayerOverrides>,
    mut commands: Commands,
    players: Query<(Entity, &Player, &Children)>,
    ranged_children: Query<(), With<Weapon>>,
    melee_children: Query<(), With<MeleeWeapon>>,
    global_assets: Option<Res<GlobalAsset>>,
    weapons_asset: Res<Assets<WeaponsConfig>>,
    melee_weapons_asset: Res<Assets<MeleeWeaponsConfig>>,
    mut id_factory: ResMut<GgrsNetIdFactory>,
    mut applied: Local<bool>,
) {
    if *applied {
        return;
    }
    if overrides.0.iter().all(|o| {
        o.tags.is_empty()
            && o.immune_to.is_empty()
            && o.modifiers.is_empty()
            && o.weapon.is_none()
            && o.currency.is_none()
    }) {
        *applied = true;
        return;
    }
    if players.iter().count() < overrides.0.len() {
        return; // les joueurs ne sont pas encore tous créés
    }
    let Some(global_assets) = global_assets else {
        return; // arme choisie : attend GlobalAsset (comme apply_weapon_overrides)
    };
    for (entity, player, children) in players.iter() {
        let Some(over) = overrides.0.get(player.handle) else {
            continue;
        };
        if !over.tags.is_empty() {
            commands
                .entity(entity)
                .insert(Tags::parse(over.tags.clone()));
        }
        if !over.immune_to.is_empty() {
            commands.entity(entity).insert(Defenses {
                immune_to: Tags::parse(over.immune_to.clone()),
                resistances: Default::default(),
            });
        }
        if !over.modifiers.is_empty() {
            let mut built = Modifiers::default();
            for (i, spec) in over.modifiers.iter().enumerate() {
                built.push(Modifier {
                    stat: spec.stat.clone(),
                    op: spec.op,
                    value: spec.value,
                    // Source stable par joueur/index : suffit pour un scénario (jamais
                    // retiré par `remove_by_source`), et distingue les modificateurs entre
                    // eux si un futur scénario en pose plusieurs sur le même joueur.
                    source: ModifierSource::Named(format!("scenario_player_{}_{i}", player.handle)),
                    until: spec.until,
                });
            }
            commands.entity(entity).insert(built);
        }
        if let Some(weapon_id) = &over.weapon {
            // Exclusivité complète (voir la doc de la fonction) : retire toute arme (à
            // distance ou de mêlée) déjà posée par `create_player` avant d'en spawner une
            // seule.
            for child in children.iter() {
                if ranged_children.get(child).is_ok() || melee_children.get(child).is_ok() {
                    commands.entity(child).despawn();
                }
            }
            let mut inventory = WeaponInventory::default();
            // Réserve de munitions (T2.2) recalculée pour cette seule arme — pas la somme
            // des armes de départ du personnage (`create_player`), qui n'existent plus dans
            // l'inventaire une fois l'exclusivité appliquée ci-dessus. Vide si `weapon_id`
            // est une arme de mêlée (pas de munitions).
            let mut ammo_reserves = AmmoReserves::new();
            if let Some(weapon_asset) = weapons_asset
                .get(&global_assets.weapons)
                .and_then(|config| config.0.get(weapon_id))
            {
                let (ammo_type, amount) =
                    game::weapons::default_mode_ammo_contribution(weapon_asset);
                ammo_reserves.add(ammo_type, amount);
                spawn_weapon_for_player(
                    &mut commands,
                    true,
                    entity,
                    weapon_asset.clone(),
                    &mut inventory,
                    &mut id_factory,
                    None,
                );
            } else if let Some(melee_asset) = melee_weapons_asset
                .get(&global_assets.melee_weapons)
                .and_then(|config| config.0.get(weapon_id))
            {
                melee::spawn_melee_weapon_for_character(
                    &mut commands,
                    entity,
                    melee_asset.clone(),
                    &mut id_factory,
                );
            } else {
                panic!(
                    "PlayerScript::weapon : arme inconnue « {weapon_id} » pour le joueur {} (ni dans weapons.ron ni dans melee_weapons.ron)",
                    player.handle
                );
            }
            // Remplace l'inventaire par défaut (potentiellement plusieurs armes à distance)
            // par celui-ci (une seule arme, ou aucune si `weapon_id` est une arme de mêlée).
            commands.entity(entity).insert(inventory);
            commands.entity(entity).insert(ammo_reserves);
        }
        // T2.3, chantier C5 v1 (scénarios d'achat `buy_door`/`buy_wall_weapon`/`buy_perk`) :
        // remplace le `Currency` posé par `create_player`
        // (`CharacterConfig::starting_currency`), avant la première frame simulée — même
        // raisonnement que `weapon` ci-dessus (voir sa doc).
        if let Some(amount) = over.currency {
            commands.entity(entity).insert(Currency::new(amount));
        }
    }
    *applied = true;
}

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

    // Séparer les attentes ponctuelles des attentes continues (NoDamageBetween)
    let mut pending: Vec<&Expectation> = scenario
        .expect
        .iter()
        .filter(|e| !matches!(e, Expectation::NoDamageBetween { .. }))
        .collect();
    pending.sort_by_key(|e| e.at_frame());

    let no_damage_between: Vec<&Expectation> = scenario
        .expect
        .iter()
        .filter(|e| matches!(e, Expectation::NoDamageBetween { .. }))
        .collect();

    let mut failures = Vec::new();
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

    let max_updates = MAX_LOADING_UPDATES + scenario.frames;
    let mut frame = 0;
    let mut stopped_early = false;
    for _ in 0..max_updates {
        let before = app.world().resource::<FrameCount>().frame;
        if before > 0 && sim_start.is_none() {
            sim_start = Some(std::time::Instant::now());
        }
        app.update();
        frame = app.world().resource::<FrameCount>().frame;
        if let Some(start) = sim_start {
            sim_elapsed = start.elapsed().as_secs_f64();
        }

        if frame > 0 {
            entities_max = entities_max.max(q_rollback.iter(app.world()).count() as u32);
            bullets_max = bullets_max.max(q_bullets.iter(app.world()).count() as u32);
            enemies_max = enemies_max.max(q_enemies.iter(app.world()).count() as u32);
            players_count = players_count.max(q_players.iter(app.world()).count() as u32);

            failures.extend(invariants.check(app.world_mut(), &scenario.invariants, frame));

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
        }

        if !app
            .world()
            .resource::<SyncTestMismatches>()
            .messages
            .is_empty()
        {
            break; // la session synctest n'avance plus après un mismatch
        }

        while pending.first().is_some_and(|e| e.at_frame() <= frame) {
            let expectation = pending.remove(0);
            if let Err(reason) = check(app.world_mut(), expectation) {
                failures.push(format!("frame {frame}: {expectation:?} : {reason}"));
            }
        }

        if frame > 0 {
            if let Some(stop) = stop_early {
                let wave_reached = stop.until_wave.is_some_and(|until_wave| {
                    app.world().resource::<WaveState>().current_wave >= until_wave
                });
                let all_dead =
                    stop.stop_when_all_players_dead && q_players.iter(app.world()).count() == 0;
                if wave_reached || all_dead {
                    stopped_early = true;
                }
            }
        }

        if stopped_early || frame >= scenario.frames {
            break;
        }
    }

    let sim_fps = if sim_elapsed > 0.0 {
        frame as f64 / sim_elapsed
    } else {
        0.0
    };

    if frame < scenario.frames && !stopped_early {
        failures.push(format!(
            "la simulation n'a atteint que la frame {frame} sur {} (map pas chargée ?)",
            scenario.frames
        ));
    }

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

    let recorder = app.world().resource::<StateTraceRecorder>();
    let trace = recorder
        .lines_until(scenario.frames)
        .map(str::to_string)
        .collect();
    let full_trace = recorder
        .dump_lines_until(scenario.frames)
        .map(|lines| lines.map(str::to_string).collect());

    let summary = summarize(app.world_mut(), frame);
    let recorded = app
        .world()
        .resource::<InputRecorder>()
        .to_scenario(app.world().get_resource::<MapGenerationConfig>());

    let events = app.world().resource::<GameEvents>().events.clone();

    let wave_state = app.world().resource::<WaveState>();
    let final_wave = wave_state.current_wave;
    let kills = wave_state.total_enemies_killed;
    let players_alive = q_players.iter(app.world()).count() as u32;

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
    };

    ScenarioOutcome {
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
        lobby: String::new(),
        cid: "scenario".into(),
        debug_ai: false,
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
        Expectation::NoDamageBetween { .. } => {
            // Géré dans la boucle principale, pas dans check()
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
        // T2.3, chantier C5 v1 : scénarios `buy_door`/`buy_wall_weapon`/`buy_perk`.
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
            ..
        } => {
            let Some(resolved) = player_stat(world, *handle, stat) else {
                return Err(format!("joueur absent ou sans stat {stat:?}"));
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
    let base = world.get::<Stats>(entity)?.get(stat)?;
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
        move |frame: Res<FrameCount>, mut exit: MessageWriter<AppExit>| {
            if frame.frame >= frames {
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
}

/// Taille des images capturées.
pub const CAPTURE_SIZE: (u32, u32) = (960, 540);

/// Joue le scénario avec rendu et capture une image toutes les `every` frames de
/// simulation. Le temps avance d'exactement une frame par update : l'image `n` montre
/// toujours la frame `n`, quelle que soit la vitesse de la machine.
pub fn capture(scenario: &Scenario, config: CaptureConfig) -> AppExit {
    std::fs::create_dir_all(&config.dir).expect("dossier de capture");

    let play_config = PlayConfig {
        follow_handle: config.follow_handle,
    };
    let mut app = build_app(scenario, false, &play_config);
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_nanos(1_000_000_000 / game::core::SIM_FPS),
    ))
    .insert_resource(CaptureState {
        dir: config.dir,
        every: config.every.max(1),
        frames: scenario.frames,
        last_captured: None,
        updates_after_end: 0,
        target: None,
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
    let target = state
        .target
        .get_or_insert_with(|| {
            images.add(Image::new_target_texture(
                CAPTURE_SIZE.0,
                CAPTURE_SIZE.1,
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
    if frame >= state.frames {
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

    if frame % state.every == 0 && state.last_captured != Some(frame) {
        let path = state.dir.join(format!("frame_{frame:05}.png"));
        commands
            .spawn(Screenshot::image(target))
            .observe(save_to_disk(path));
        state.last_captured = Some(frame);
    }
}
