//! D69 étape 2, E3c (`docs/taches/rapports/dette-d69-e3c-balance.md`) : `ResolvedBalance`, `PowerUpsTable` et
//! `SessionPlayers`, les données d'équilibrage et de power-ups lues par la simulation, sont **pleins et justes au
//! premier tick de `GgrsSchedule`**.
//!
//! Les lecteurs ont des replis qui masquent une table vide : `powerup_pickup_detect_system` et
//! `loot_drop_on_death_system` reviennent sans rien faire sans table, la machine à vagues reste inerte si
//! `waves_enabled` est faux, `shell_drop_on_death_system` ne lâche rien sans `shells_drop`. Un scénario qui ne
//! regarde que les résultats ne verrait pas une table jamais remplie : ces tests regardent les ressources elles-mêmes.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use game::balance::ResolvedBalance;
use game::global_asset::GlobalAsset;
use game::powerups::PowerUpsTable;
use game::powerups_assets::PowerUpsConfigAsset;
use game::session::SessionPlayers;
use scenario::Scenario;

fn load(ron: &str) -> Scenario {
    let path = format!(
        "{}/../../tests/scenarios/{ron}.ron",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path} : {e}"));
    let mut scenario = Scenario::from_ron(&text).unwrap_or_else(|e| panic!("{ron} : {e}"));
    scenario.frames = 3;
    scenario.expect.clear();
    scenario
}

/// Ce que le premier tick voit, avec ce que contiennent les **fichiers** (les enveloppes `Asset`, avant
/// surcharge de scénario) pour comparer.
#[derive(Default)]
struct FirstTick {
    seen: bool,
    players: Option<u32>,
    balance: Option<ResolvedBalance>,
    table_drop_chance: Option<String>,
    table_ids: Option<Vec<String>>,
    /// `drop_chance` de l'asset `items/powerups.ron` **au premier tick** (surcharge de scénario comprise),
    /// `None` sans fichier.
    asset_drop_chance: Option<String>,
}

fn first_tick(ron: &str) -> FirstTick {
    let scenario = load(ron);
    let probe = Arc::new(Mutex::new(FirstTick::default()));
    let sink = probe.clone();
    scenario::runner::run_with(&scenario, move |app| {
        app.add_systems(
            bevy_ggrs::GgrsSchedule,
            (move |session: Option<Res<SessionPlayers>>,
                   balance: Option<Res<ResolvedBalance>>,
                   table: Res<PowerUpsTable>,
                   global: Res<GlobalAsset>,
                   powerups: Res<Assets<PowerUpsConfigAsset>>| {
                let mut probe = sink.lock().expect("sonde non empoisonnée");
                if probe.seen {
                    return;
                }
                probe.seen = true;
                probe.players = session.map(|session| session.0);
                probe.balance = balance.map(|balance| balance.clone());
                probe.table_drop_chance = table
                    .0
                    .as_ref()
                    .map(|config| config.drop_chance.to_string());
                probe.table_ids = table
                    .0
                    .as_ref()
                    .map(|config| config.powerups.keys().cloned().collect());
                probe.asset_drop_chance = global
                    .powerups_config
                    .as_ref()
                    .and_then(|handle| powerups.get(handle))
                    .map(|asset| asset.0.drop_chance.to_string());
            })
            .in_set(game::system_set::RollbackSystemSet::FrameStart),
        );
    });
    Arc::try_unwrap(probe)
        .ok()
        .expect("sonde partagée")
        .into_inner()
        .expect("sonde non empoisonnée")
}

/// Zombies à quatre joueurs : `SessionPlayers` vaut le nombre de joueurs de la session et pilote la résolution
/// (`zombie_scaled` : `10 + (players - 1) * 40` = 130), les vagues sont actives, la table de power-ups est celle
/// du fichier et les points d'économie sont résolus.
#[test]
fn zombies_four_players_resolve_for_four() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let probe = first_tick("four_players_idle");
    assert!(probe.seen, "aucun tick simulé");
    assert_eq!(probe.players, Some(4), "SessionPlayers");
    let balance = probe
        .balance
        .expect("ResolvedBalance posée avant le premier tick");
    assert_eq!(
        balance.health_max_by_character["zombie_scaled"].to_string(),
        "130"
    );
    assert_eq!(balance.health_max_by_character["player"].to_string(), "100");
    assert!(balance.waves_enabled, "zombies déclare un dossier `Wave`");
    assert_eq!(balance.waves.base_enemies, 3);
    assert_eq!(balance.economy.kill_points, 60);
    assert!(balance.perks.contains_key("juggernog"), "perks résolus");
    assert_eq!(balance.economy.shells_drop, None);
    let ids = probe
        .table_ids
        .expect("PowerUpsTable remplie avant le premier tick");
    assert!(ids.iter().any(|id| id == "insta_kill"), "{ids:?}");
    assert_eq!(
        probe.table_drop_chance, probe.asset_drop_chance,
        "la table est le contenu de l'asset (aucune surcharge ici)"
    );
}

/// À deux joueurs la même expression donne 50 : le nombre de joueurs vient bien de la session.
#[test]
fn zombies_two_players_resolve_for_two() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let probe = first_tick("two_players_shooting");
    assert_eq!(probe.players, Some(2), "SessionPlayers");
    let balance = probe.balance.expect("ResolvedBalance");
    assert_eq!(
        balance.health_max_by_character["zombie_scaled"].to_string(),
        "50"
    );
}

/// Gungeon : pas de vagues (`waves_enabled` faux : la machine reste inerte), `shells_drop` et prix de présentoirs
/// de `economy.ron`.
#[test]
fn gungeon_has_no_waves_but_shells_and_shop_prices() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let probe = first_tick("bench_floor");
    let balance = probe.balance.expect("ResolvedBalance");
    assert!(!balance.waves_enabled, "gungeon ne déclare pas de vagues");
    assert_eq!(balance.economy.shells_drop, Some((2, 5)));
    assert_eq!(
        balance
            .economy
            .shop_prices
            .values()
            .copied()
            .collect::<Vec<_>>(),
        vec![15, 30, 50],
        "Common, Uncommon, Rare"
    );
    assert!(probe.table_ids.is_some(), "gungeon déclare des power-ups");
}

/// Un jeu sans `items/powerups.ron` : table vide (`None`), pas une table par défaut.
#[test]
fn a_game_without_powerups_has_no_table() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let probe = first_tick("achievement_unlocks");
    assert!(probe.seen);
    assert!(probe.asset_drop_chance.is_none(), "isaac sans power-ups");
    assert!(probe.table_ids.is_none(), "PowerUpsTable vide");
}

/// Les surcharges de scénario visent l'**enveloppe** avant la résolution : la valeur forcée est celle de la
/// table et de `ResolvedBalance`. Chaque surcharge est comparée au scénario voisin du même jeu qui n'en a pas
/// (la surcharge écrit dans l'asset lui-même : il ne porte plus la valeur du fichier au premier tick).
#[test]
fn scenario_overrides_reach_the_resolved_tables() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    // Testbed : `powerup_drop_chance_override: "1.0"` contre `aim_four_way` (`drop_chance` du fichier).
    let plain = first_tick("aim_four_way");
    assert_eq!(
        plain.table_drop_chance.as_deref(),
        Some("0"),
        "fichier du testbed"
    );
    let probe = first_tick("powerup_drop_on_kill");
    assert_eq!(
        probe.table_drop_chance.as_deref(),
        Some("1"),
        "surcharge du drop dans la table"
    );

    // Zombies : `wave_overrides: (base_enemies: Some(60), …)` contre `four_players_idle` (fichier : 3).
    let plain = first_tick("four_players_idle")
        .balance
        .expect("ResolvedBalance");
    assert_eq!(plain.waves.base_enemies, 3, "fichier de vagues de zombies");
    let balance = first_tick("bench_horde").balance.expect("ResolvedBalance");
    assert_eq!(balance.waves.base_enemies, 60, "surcharge de vagues");
    assert_eq!(balance.waves.max_concurrent_enemies, 80);
}
