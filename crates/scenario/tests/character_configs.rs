//! D69 étape 2, E3a (`docs/taches/rapports/dette-d69-e3a-character-configs.md`) : `CharacterConfigs`,
//! la configuration des personnages lue par la simulation, est **remplie avant le premier tick de
//! `GgrsSchedule`** et suit le rechargement d'un asset.
//!
//! Les lecteurs de la simulation ont des replis silencieux (`apply_inputs` : `continue` ; `move_enemies` :
//! vitesse de repli ; `health` : défauts) qui masqueraient un registre vide : ces tests regardent donc
//! la ressource elle-même, à chaque tick, pour chaque personnage créé (joueurs au chargement de la
//! carte, spawns de carte, vagues, divisions de champions, jeux zombies, testbed et isaac).

use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use game::character::config::{CharacterConfigKey, CharacterConfigs};
use game::character::config_asset::CharacterConfigAsset;
use game::character::enemy::Enemy;
use game::global_asset::GlobalAsset;
use scenario::Scenario;
use utils::net_id::GgrsNetId;

#[derive(Default)]
struct Probe {
    /// Ticks simulés vus par la sonde.
    ticks: u32,
    /// Personnages (net id) vus au moins une fois.
    seen: std::collections::BTreeSet<usize>,
    /// Ennemis (`Enemy`) vus : net id → rang du premier tick où ils sont vus.
    enemies: std::collections::BTreeMap<usize, u32>,
    /// `(len de CharacterConfigs, len de GlobalAsset::character_configs)` au premier tick.
    first_tick: Option<(usize, usize)>,
    /// Un personnage sans entrée dans `CharacterConfigs` : `frame net_id clé`.
    violations: Vec<String>,
}

/// Joue `ron` (chemin relatif à `tests/scenarios/`) en observant `CharacterConfigs` à chaque tick de
/// `GgrsSchedule`, avant tout autre système de la frame.
fn observe(ron: &str, frames: u32) -> Probe {
    let path = format!(
        "{}/../../tests/scenarios/{ron}.ron",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path} : {e}"));
    let mut scenario = Scenario::from_ron(&text).unwrap_or_else(|e| panic!("{ron} : {e}"));
    scenario.frames = frames;
    scenario.expect.clear();
    let probe = Arc::new(Mutex::new(Probe::default()));
    let sink = probe.clone();
    scenario::runner::run_with(&scenario, move |app| {
        app.add_systems(
            bevy_ggrs::GgrsSchedule,
            (move |frame: Res<utils::frame::FrameCount>,
                   configs: Res<CharacterConfigs>,
                   global: Res<GlobalAsset>,
                   characters: Query<(&GgrsNetId, &CharacterConfigKey, Has<Enemy>)>| {
                let mut probe = sink.lock().unwrap();
                probe.ticks += 1;
                if probe.first_tick.is_none() {
                    probe.first_tick = Some((configs.0.len(), global.character_configs.len()));
                }
                for (net_id, key, is_enemy) in &characters {
                    probe.seen.insert(net_id.0);
                    if is_enemy {
                        let tick = probe.ticks - 1;
                        probe.enemies.entry(net_id.0).or_insert(tick);
                    }
                    if configs.get(&key.0).is_none() {
                        probe
                            .violations
                            .push(format!("f{} #{} « {} »", frame.frame, net_id.0, key.0));
                    }
                }
            })
            .in_set(game::system_set::RollbackSystemSet::FrameStart),
        );
    });
    Arc::try_unwrap(probe)
        .ok()
        .expect("sonde partagée")
        .into_inner()
        .unwrap()
}

/// `enemies_in_simulation` : un ennemi au moins est **créé par la simulation** (vague, division de champion :
/// vu pour la première fois après le premier tick) ; sinon un ennemi au moins existe.
fn check(ron: &str, frames: u32, enemies_in_simulation: bool) {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let probe = observe(ron, frames);
    assert!(probe.ticks > 0, "{ron} : aucun tick simulé");
    let (filled, expected) = probe.first_tick.expect("premier tick");
    assert!(expected > 0, "{ron} : GlobalAsset sans personnage");
    assert_eq!(
        filled, expected,
        "{ron} : CharacterConfigs rempli à {filled}/{expected} au premier tick"
    );
    assert!(!probe.seen.is_empty(), "{ron} : aucun personnage observé");
    assert!(!probe.enemies.is_empty(), "{ron} : aucun ennemi observé");
    if enemies_in_simulation {
        assert!(
            probe.enemies.values().any(|&tick| tick > 0),
            "{ron} : aucun ennemi créé par la simulation"
        );
    }
    assert!(
        probe.violations.is_empty(),
        "{ron} : personnage sans entrée dans CharacterConfigs : {:?}",
        probe.violations
    );
}

/// Zombies : joueurs créés au chargement de la carte, ennemis de vague créés par `GgrsSchedule`
/// (délai de grâce de 180 frames : 400 frames pour voir le premier spawn).
#[test]
fn zombies_players_and_waves_see_their_config() {
    check("four_players_idle", 400, true);
}

/// Testbed : `CharacterSpawn` de carte (créés avant le premier tick) et placements.
#[test]
fn testbed_map_spawns_see_their_config() {
    check("hit_iframes", 60, false);
}

/// Isaac : jeu à objets, autre contenu, autre `GlobalAsset`.
#[test]
fn isaac_sees_its_config() {
    check("hit_iframes_isaac", 60, false);
}

/// Division de champion : `spawn_enemy` depuis `GgrsSchedule` (`champion_death_system`).
#[test]
fn champion_split_spawns_see_their_config() {
    check("champion_splits", 400, true);
}

/// Rechargement à chaud (chemin `AssetEvent::Modified`, le même que celui du `file_watcher`) : modifier
/// l'asset met `CharacterConfigs` à jour, et un asset **inchangé** n'écrase pas une surcharge posée sur
/// la ressource (les surcharges de scénario se posent après la synchro).
#[test]
fn asset_modification_refreshes_registry_but_overrides_survive() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let path = format!(
        "{}/../../tests/scenarios/four_players_idle.ron",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(&path).unwrap();
    let mut scenario = Scenario::from_ron(&text).unwrap();
    scenario.frames = 90;
    scenario.expect.clear();

    // (surcharge, valeur d'asset modifiée) → `weapon_slots` de « player » vu à chaque phase.
    const OVERRIDE: u32 = 7;
    const RELOADED: u32 = 9;
    let seen = Arc::new(Mutex::new(Vec::<(u32, u32)>::new()));
    let sink = seen.clone();
    // Vue de la simulation : `weapon_slots` de « player » à chaque exécution de `GgrsSchedule`, et rang de la
    // première exécution qui suit la modification de l'asset. Elle doit déjà voir la valeur rechargée : la
    // synchro tourne avant `RunGgrsSystems`, dans le même `PreUpdate` que la lecture de l'événement
    // (même si la session resimule d'abord une frame ancienne).
    let sim = Arc::new(Mutex::new(Vec::<u32>::new()));
    let sim_sink = sim.clone();
    let sim_at_update = sim.clone();
    let modified_at = Arc::new(Mutex::new(None::<usize>));
    let modified_sink = modified_at.clone();
    scenario::runner::run_with(&scenario, move |app| {
        app.add_systems(
            bevy_ggrs::GgrsSchedule,
            (move |configs: Res<CharacterConfigs>| {
                if let Some(config) = configs.get("player") {
                    sim_sink.lock().unwrap().push(config.weapon_slots);
                }
            })
            .in_set(game::system_set::RollbackSystemSet::FrameStart),
        );
        app.add_systems(
            Update,
            move |frame: Res<utils::frame::FrameCount>,
                  mut configs: ResMut<CharacterConfigs>,
                  mut assets: ResMut<Assets<CharacterConfigAsset>>,
                  global: Res<GlobalAsset>| {
                let Some(player) = configs.0.get("player").map(|c| c.weapon_slots) else {
                    return;
                };
                let handle = global.character_configs.get("player").unwrap().clone();
                match frame.frame {
                    // Surcharge posée sur la ressource : l'asset est intact.
                    10 => configs.0.get_mut("player").unwrap().weapon_slots = OVERRIDE,
                    // Modification de l'asset : `AssetEvent::Modified` à la frame suivante.
                    30 => {
                        assets.get_mut(&handle).unwrap().0.weapon_slots = RELOADED;
                        let ticks_so_far = sim_at_update.lock().unwrap().len();
                        modified_sink.lock().unwrap().get_or_insert(ticks_so_far);
                    }
                    _ => {}
                }
                sink.lock().unwrap().push((frame.frame, player));
            },
        );
    });
    let seen = seen.lock().unwrap();
    let at = |frame: u32| {
        seen.iter()
            .rev()
            .find(|(f, _)| *f <= frame)
            .map(|(_, v)| *v)
            .unwrap_or_else(|| panic!("aucune mesure avant la frame {frame}"))
    };
    assert_eq!(
        at(9),
        3,
        "valeur du fichier avant surcharge (`weapon_slots: 3`)"
    );
    assert_eq!(
        at(29),
        OVERRIDE,
        "la surcharge survit (aucun événement Modified)"
    );
    assert_eq!(
        at(80),
        RELOADED,
        "l'asset modifié recopié dans CharacterConfigs"
    );
    let sim = sim.lock().unwrap();
    let first_after = modified_at
        .lock()
        .unwrap()
        .expect("modification de l'asset jouée");
    assert_eq!(
        sim.get(first_after).copied(),
        Some(RELOADED),
        "la première exécution de GgrsSchedule après la modification voit déjà la valeur rechargée"
    );
    assert_eq!(
        sim.get(first_after - 1).copied(),
        Some(OVERRIDE),
        "et la précédente voyait encore la surcharge"
    );
}

/// Diagnostic (ignoré, `cargo test -p scenario --test character_configs fill_timeline -- --ignored --nocapture`) :
/// frame d'app (nombre d'`Update` depuis le début) du remplissage de `CharacterConfigs`, de chaque
/// changement d'état de l'application, du premier personnage créé et du premier tick de `GgrsSchedule`.
#[test]
#[ignore]
fn fill_timeline() {
    use std::sync::atomic::{AtomicU32, Ordering};
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let path = format!(
        "{}/../../tests/scenarios/four_players_idle.ron",
        env!("CARGO_MANIFEST_DIR")
    );
    let mut scenario = Scenario::from_ron(&std::fs::read_to_string(&path).unwrap()).unwrap();
    scenario.frames = 5;
    scenario.expect.clear();
    let tick = Arc::new(AtomicU32::new(0));
    let log = Arc::new(Mutex::new(Vec::<String>::new()));
    let (t, l) = (tick.clone(), log.clone());
    scenario::runner::run_with(&scenario, move |app| {
        let t1 = t.clone();
        app.add_systems(First, move || {
            t1.fetch_add(1, Ordering::Relaxed);
        });
        let (t2, l2) = (t.clone(), l.clone());
        app.add_systems(
            PreUpdate,
            (move |configs: Res<CharacterConfigs>, mut done: Local<bool>| {
                if !*done && !configs.0.is_empty() {
                    *done = true;
                    l2.lock().unwrap().push(format!(
                        "frame d'app {:>4} : CharacterConfigs rempli ({} entrées)",
                        t2.load(Ordering::Relaxed),
                        configs.0.len()
                    ));
                }
            })
            .after(game::character::config_asset::sync_character_configs),
        );
        let (t3, l3) = (t.clone(), l.clone());
        app.add_systems(
            Update,
            move |state: Res<State<game::session::AppState>>,
                  mut last: Local<Option<game::session::AppState>>| {
                if last.as_ref() != Some(state.get()) {
                    *last = Some(state.get().clone());
                    l3.lock().unwrap().push(format!(
                        "frame d'app {:>4} : AppState = {:?}",
                        t3.load(Ordering::Relaxed),
                        state.get()
                    ));
                }
            },
        );
        let (t4, l4) = (t.clone(), l.clone());
        app.add_systems(
            PostUpdate,
            move |added: Query<&GgrsNetId, Added<CharacterConfigKey>>, mut done: Local<bool>| {
                if !*done && !added.is_empty() {
                    *done = true;
                    l4.lock().unwrap().push(format!(
                        "frame d'app {:>4} : premier personnage créé ({} entité(s))",
                        t4.load(Ordering::Relaxed),
                        added.iter().count()
                    ));
                }
            },
        );
        let (t5, l5) = (t.clone(), l.clone());
        app.add_systems(
            bevy_ggrs::GgrsSchedule,
            (move |mut done: Local<bool>| {
                if !*done {
                    *done = true;
                    l5.lock().unwrap().push(format!(
                        "frame d'app {:>4} : premier tick de GgrsSchedule",
                        t5.load(Ordering::Relaxed)
                    ));
                }
            })
            .in_set(game::system_set::RollbackSystemSet::FrameStart),
        );
    });
    for line in log.lock().unwrap().iter() {
        println!("{line}");
    }
}
