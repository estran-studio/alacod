//! D69 étape 2, E3b (`docs/taches/rapports/dette-d69-e3b-weapon-registry.md`) : `WeaponRegistry`, les
//! armes lues par la simulation, est **plein avant le premier tick de `GgrsSchedule`**, porte les
//! surcharges de scénario (`weapon_overrides`) et suit le rechargement d'un asset.
//!
//! Les créateurs (`create_player`, `spawn_enemy`) ignorent avec un `warn!` une arme absente du
//! registre : un registre vide donnerait des joueurs sans arme sans faire échouer un scénario. Ces tests
//! regardent donc le registre lui-même et l'équipement créé, au premier tick.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use game::global_asset::GlobalAsset;
use game::weapon_assets::{MeleeWeaponsConfig, WeaponsConfig};
use game::weapons::melee::MeleeWeapon;
use game::weapons::{MagBulletConfig, Weapon, WeaponInventory, WeaponRegistry};
use scenario::Scenario;

fn load(ron: &str, frames: u32) -> Scenario {
    let path = format!(
        "{}/../../tests/scenarios/{ron}.ron",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path} : {e}"));
    let mut scenario = Scenario::from_ron(&text).unwrap_or_else(|e| panic!("{ron} : {e}"));
    scenario.frames = frames;
    scenario.expect.clear();
    scenario
}

/// `mag_limit` du mode par défaut d'une arme.
fn mag_limit(config: &game::weapons::WeaponConfig) -> u32 {
    match &config.firing_modes[&config.default_firing_mode].mag {
        MagBulletConfig::Mag { mag_limit, .. } => *mag_limit,
        other => panic!("arme sans chargeur : {:?}", std::mem::discriminant(other)),
    }
}

#[derive(Default)]
struct FirstTick {
    seen: bool,
    /// `(armes, mêlée)` du registre ; `(armes, mêlée)` de l'asset de `GlobalAsset`.
    registry: (usize, usize),
    assets: (usize, usize),
    /// Armes à distance des inventaires (hors armes murales et butin au sol, qui portent aussi `Weapon`).
    ranged: Vec<String>,
    melee: usize,
    /// `mag_limit` du pistolet : dans le registre, dans l'asset, sur les armes portées.
    pistol_registry: Option<u32>,
    pistol_asset: Option<u32>,
    pistol_carried: Vec<u32>,
}

/// Joue `ron` en relevant, **à la première exécution de `GgrsSchedule`**, le registre, les tables
/// d'assets de `GlobalAsset` et les armes déjà créées.
fn first_tick(ron: &str, frames: u32) -> FirstTick {
    let scenario = load(ron, frames);
    let probe = Arc::new(Mutex::new(FirstTick::default()));
    let sink = probe.clone();
    scenario::runner::run_with(&scenario, move |app| {
        app.add_systems(
            bevy_ggrs::GgrsSchedule,
            (move |registry: Res<WeaponRegistry>,
                   global: Res<GlobalAsset>,
                   weapons: Res<Assets<WeaponsConfig>>,
                   melee: Res<Assets<MeleeWeaponsConfig>>,
                   ranged: Query<&Weapon>,
                   melee_weapons: Query<(), With<MeleeWeapon>>,
                   inventories: Query<&WeaponInventory>| {
                let mut probe = sink.lock().unwrap();
                if probe.seen {
                    return;
                }
                probe.seen = true;
                let weapon_table = weapons.get(&global.weapons).expect("weapons.ron chargé");
                let melee_table = melee
                    .get(&global.melee_weapons)
                    .expect("melee_weapons.ron chargé");
                probe.registry = (registry.weapons.len(), registry.melee.len());
                probe.assets = (weapon_table.0.len(), melee_table.0.len());
                probe.ranged = inventories
                    .iter()
                    .flat_map(|inventory| inventory.weapons.iter())
                    .map(|(_, weapon)| weapon.config.name.clone())
                    .collect();
                probe.melee = melee_weapons.iter().count();
                probe.pistol_registry = registry
                    .weapons
                    .get("pistol")
                    .map(|asset| mag_limit(&asset.config));
                probe.pistol_asset = weapon_table
                    .0
                    .get("pistol")
                    .map(|asset| mag_limit(&asset.config));
                probe.pistol_carried = inventories
                    .iter()
                    .flat_map(|inventory| inventory.weapons.iter())
                    .filter(|(_, weapon)| weapon.config.name == "pistol")
                    .map(|(_, weapon)| mag_limit(&weapon.config))
                    .collect();
                // Chaque entrée d'inventaire pointe vers une arme portée qui existe.
                for inventory in &inventories {
                    for (entity, weapon) in &inventory.weapons {
                        assert!(
                            ranged.get(*entity).is_ok(),
                            "arme « {} » de l'inventaire sans entité",
                            weapon.config.name
                        );
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

fn check_full(ron: &str, frames: u32, game: &str) -> FirstTick {
    let probe = first_tick(ron, frames);
    assert!(probe.seen, "{game} : aucun tick simulé");
    assert!(
        probe.assets.0 > 0 && probe.assets.1 > 0,
        "{game} : tables d'assets vides"
    );
    assert_eq!(
        probe.registry, probe.assets,
        "{game} : WeaponRegistry rempli à {:?} (armes, mêlée) au premier tick, assets {:?}",
        probe.registry, probe.assets
    );
    probe
}

/// Zombies : les joueurs naissent au chargement de la carte avec leurs armes de départ
/// (`starting_weapons`) et `bare_hands` ; les créateurs lisent le registre avant le premier tick.
#[test]
fn zombies_registry_is_full_and_players_are_armed_at_first_tick() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let probe = check_full("four_players_idle", 30, "zombies");
    for weapon in ["machine_gun", "pistol", "shotgun"] {
        assert_eq!(
            probe.ranged.iter().filter(|name| *name == weapon).count(),
            4,
            "zombies : chaque joueur porte « {weapon} » au premier tick ({:?})",
            probe.ranged
        );
    }
    assert!(
        probe.melee >= 4,
        "zombies : `bare_hands` de chaque joueur au premier tick ({} armes de mêlée)",
        probe.melee
    );
}

/// Testbed : autre contenu d'armes (`CharacterSpawn` de carte armés par `spawn_enemy`).
#[test]
fn testbed_registry_is_full_at_first_tick() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let probe = check_full("hit_iframes", 30, "testbed");
    assert!(
        !probe.ranged.is_empty() || probe.melee > 0,
        "testbed : personnages sans arme"
    );
}

/// Isaac : jeu à objets, autre table d'armes.
#[test]
fn isaac_registry_is_full_at_first_tick() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let probe = check_full("hit_iframes_isaac", 30, "isaac");
    assert!(
        !probe.ranged.is_empty(),
        "isaac : joueur sans arme au premier tick"
    );
}

/// `PlayerScript::weapon` (`map_ldtk::game::profile::apply_player_overrides`, lit le registre) : le joueur ne porte que
/// l'arme demandée (arme à distance puis arme de mêlée), pour deux jeux.
#[test]
fn player_weapon_override_reads_the_registry() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let probe = first_tick("generated/zombies/weapon_pistol", 60);
    assert!(probe.seen);
    assert_eq!(probe.registry.0, probe.assets.0, "zombies : registre plein");
    // L'exclusivité s'applique avant le premier tick simulé : un seul pistolet, aucune autre arme.
    assert_eq!(probe.ranged, vec!["pistol".to_string()], "arme du joueur");
    assert_eq!(probe.melee, 0, "exclusivité : `bare_hands` retiré");

    let probe = first_tick("generated/zombies/weapon_sword", 60);
    assert!(
        probe.ranged.is_empty(),
        "arme de mêlée : aucune arme à distance"
    );
    assert_eq!(probe.melee, 1, "l'épée, lue dans `WeaponRegistry::melee`");
}

/// `weapon_overrides` (`apply_weapon_overrides`) s'applique au **registre**, avant la création des
/// joueurs : le pistolet porté a `mag_limit: 1` (scénario `ammo_burst`), alors que l'asset du
/// fichier, lui, n'a pas bougé.
#[test]
fn scenario_weapon_overrides_apply_to_the_registry() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let probe = first_tick("ammo_burst", 30);
    assert!(probe.seen);
    let file_value = probe.pistol_asset.expect("pistolet dans weapons.ron");
    assert_ne!(
        file_value, 1,
        "le test suppose un `mag_limit` du fichier ≠ 1"
    );
    assert_eq!(
        probe.pistol_registry,
        Some(1),
        "WeaponRegistry porte la surcharge `mag_limit: 1`"
    );
    assert!(!probe.pistol_carried.is_empty(), "le pistolet est porté");
    assert!(
        probe.pistol_carried.iter().all(|limit| *limit == 1),
        "les pistolets créés copient le registre surchargé : {:?}",
        probe.pistol_carried
    );
}

/// Rechargement à chaud (chemin `AssetEvent::Modified`, le même que celui du `file_watcher`) :
/// modifier l'asset met `WeaponRegistry` à jour et la première exécution de `GgrsSchedule` qui suit
/// voit déjà la valeur rechargée ; un asset **inchangé** n'écrase pas une surcharge posée sur le
/// registre.
#[test]
fn asset_modification_refreshes_registry_but_overrides_survive() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let scenario = load("four_players_idle", 90);

    const OVERRIDE: u32 = 41;
    const RELOADED: u32 = 43;
    // `(frame d'app, mag_limit du pistolet du registre)` à chaque `Update`.
    let seen = Arc::new(Mutex::new(Vec::<(u32, u32)>::new()));
    let sink = seen.clone();
    // `mag_limit` du registre à chaque exécution de `GgrsSchedule`, et rang de la première qui suit la
    // modification de l'asset.
    let sim = Arc::new(Mutex::new(Vec::<u32>::new()));
    let sim_sink = sim.clone();
    let sim_at_update = sim.clone();
    let modified_at = Arc::new(Mutex::new(None::<usize>));
    let modified_sink = modified_at.clone();
    scenario::runner::run_with(&scenario, move |app| {
        app.add_systems(
            bevy_ggrs::GgrsSchedule,
            (move |registry: Res<WeaponRegistry>| {
                if let Some(asset) = registry.weapons.get("pistol") {
                    sim_sink.lock().unwrap().push(mag_limit(&asset.config));
                }
            })
            .in_set(game::system_set::RollbackSystemSet::FrameStart),
        );
        app.add_systems(
            Update,
            move |frame: Res<utils::frame::FrameCount>,
                  mut registry: ResMut<WeaponRegistry>,
                  mut assets: ResMut<Assets<WeaponsConfig>>,
                  global: Res<GlobalAsset>| {
                let Some(pistol) = registry.weapons.get_mut("pistol") else {
                    return;
                };
                let limit = mag_limit(&pistol.config);
                let set_limit = |config: &mut game::weapons::WeaponConfig, value: u32| {
                    for mode in config.firing_modes.values_mut() {
                        if let MagBulletConfig::Mag { mag_limit, .. } = &mut mode.mag {
                            *mag_limit = value;
                        }
                    }
                };
                match frame.frame {
                    // Surcharge posée sur le registre : l'asset est intact.
                    10 => set_limit(&mut pistol.config, OVERRIDE),
                    // Modification de l'asset : `AssetEvent::Modified` à la frame suivante.
                    30 => {
                        let mut asset = assets.get_mut(&global.weapons).unwrap();
                        set_limit(&mut asset.0.get_mut("pistol").unwrap().config, RELOADED);
                        let ticks_so_far = sim_at_update.lock().unwrap().len();
                        modified_sink.lock().unwrap().get_or_insert(ticks_so_far);
                    }
                    _ => {}
                }
                sink.lock().unwrap().push((frame.frame, limit));
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
    let file_value = at(9);
    assert_ne!(file_value, OVERRIDE);
    assert_ne!(file_value, RELOADED);
    assert_eq!(
        at(29),
        OVERRIDE,
        "la surcharge survit (aucun événement Modified)"
    );
    assert_eq!(
        at(75),
        RELOADED,
        "l'asset modifié recopié dans WeaponRegistry"
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
