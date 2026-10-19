//! Résolution de l'équilibrage à partir des assets chargés (D69 étape 2, E3c) : côté application.
//!
//! [`resolve_balance_system`] lit les enveloppes `Asset` (`waves::assets::WaveConfigAsset`,
//! `economy_assets::EconomyConfigAsset` / `PerksConfigAsset`) et les configurations de personnages, appelle
//! le calcul pur [`resolve_balance`] (`balance.rs`, simulation) et insère [`ResolvedBalance`]. Les mêmes
//! noms qu'avant E3c, au même moment : `OnEnter(AppState::GameLoading)`. [`publish_session_players`]
//! publie, juste avant, le nombre de joueurs de la session que la simulation lit
//! ([`SessionPlayers`], aussi lu par `clock::resolve_clocks_system`).

use bevy::prelude::*;
use sim_core::players::PlayersCount;

use crate::balance::resolve_balance;
use crate::character::config::CharacterConfigs;
use crate::economy_assets::{EconomyConfigAsset, PerksConfigAsset};
use crate::global_asset::GlobalAsset;
use crate::jjrs::GggrsSessionConfiguration;
use crate::session::{OnlineState, SessionPlayers};
use crate::waves::assets::WaveConfigAsset;

/// Nombre de joueurs de la session : `OnlineState::Online` → `max_player` de la session ggrs (source
/// autoritaire en ligne) ; sinon `PlayersCount` (partie locale ou scénario). Ce que
/// `resolve_balance_system` et `resolve_clocks_system` lisaient chacun avant E3c.
pub fn session_players(online: OnlineState, ggrs_max_player: usize, players_count: usize) -> u32 {
    match online {
        OnlineState::Online => ggrs_max_player as u32,
        OnlineState::Offline | OnlineState::Unset => players_count as u32,
    }
}

/// `OnEnter(AppState::GameLoading)`, avant `resolve_balance_system` et `resolve_clocks_system` : écrit
/// [`SessionPlayers`].
pub fn publish_session_players(
    mut commands: Commands,
    online: Res<OnlineState>,
    ggrs_config: Res<GggrsSessionConfiguration>,
    players_count: Res<PlayersCount>,
) {
    commands.insert_resource(SessionPlayers(session_players(
        *online,
        ggrs_config.connection.max_player,
        players_count.0,
    )));
}

/// Résout tous les champs F5 (vagues, économie, perks, santé des personnages) pour le nombre de joueurs de
/// la partie et insère [`ResolvedBalance`].
///
/// Enregistré sur `OnEnter(AppState::GameLoading)` (voir la doc de `balance` pour la justification du
/// point d'évaluation).
pub fn resolve_balance_system(
    mut commands: Commands,
    global_assets: Res<GlobalAsset>,
    wave_configs: Res<Assets<WaveConfigAsset>>,
    economy_configs: Res<Assets<EconomyConfigAsset>>,
    perks_configs: Res<Assets<PerksConfigAsset>>,
    character_configs: Res<CharacterConfigs>,
    players: Res<SessionPlayers>,
) {
    let waves = global_assets
        .wave_config
        .as_ref()
        .and_then(|h| wave_configs.get(h))
        .map(|asset| &asset.0);
    let economy = global_assets
        .economy_config
        .as_ref()
        .and_then(|h| economy_configs.get(h))
        .map(|asset| &asset.0);
    let perks = global_assets
        .perks_config
        .as_ref()
        .and_then(|h| perks_configs.get(h))
        .map(|asset| &asset.0);

    commands.insert_resource(resolve_balance(
        players.0,
        waves,
        economy,
        perks,
        &character_configs,
        global_assets.character_configs.keys(),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::balance::ResolvedBalance;
    use crate::economy::{EconomyConfig, PerksConfig};
    use crate::waves::config::WaveConfig;
    use bevy_fixed::fixed_math::Fixed;
    use std::path::PathBuf;

    #[test]
    fn online_uses_the_ggrs_player_count_otherwise_the_local_one() {
        assert_eq!(session_players(OnlineState::Online, 4, 2), 4);
        assert_eq!(session_players(OnlineState::Offline, 4, 2), 2);
        assert_eq!(session_players(OnlineState::Unset, 4, 2), 2);
    }

    /// Les cinq jeux livrés : `games/<jeu>/assets`.
    fn games() -> Vec<PathBuf> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../games");
        let mut games: Vec<PathBuf> = std::fs::read_dir(&root)
            .unwrap()
            .map(|entry| entry.unwrap().path().join("assets"))
            .filter(|path| path.is_dir())
            .collect();
        games.sort();
        games
    }

    /// Un RON du jeu, lu une fois comme **enveloppe** (le chemin d'après E3c : `Assets<…Asset>`) et une fois
    /// comme **donnée** (le chemin d'avant : la configuration était elle-même l'asset). `None` si le jeu ne
    /// déclare pas ce fichier.
    fn both<E, D>(assets: &std::path::Path, file: &str) -> Option<(E, D)>
    where
        E: serde::de::DeserializeOwned,
        D: serde::de::DeserializeOwned,
    {
        let text = std::fs::read_to_string(assets.join(file)).ok()?;
        let envelope = ron::from_str(&text).unwrap_or_else(|e| panic!("{file} (enveloppe) : {e}"));
        let data = ron::from_str(&text).unwrap_or_else(|e| panic!("{file} (donnée) : {e}"));
        Some((envelope, data))
    }

    /// Deux vrais personnages de `zombies` : `player` (santé littérale) et `zombie_scaled` (santé en expression
    /// de `players`), de quoi rendre `health_max_by_character` non vide.
    fn characters() -> (CharacterConfigs, Vec<String>) {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../games/zombies/assets");
        let mut characters = CharacterConfigs::default();
        for (id, file) in [
            (
                "player",
                "ZombieShooter/Sprites/Character/player_config.ron",
            ),
            (
                "zombie_scaled",
                "ZombieShooter/Sprites/Zombie/zombie_scaled_config.ron",
            ),
        ] {
            let text = std::fs::read_to_string(root.join(file)).unwrap();
            // Comme `RonAssetPlugin` : les fichiers de personnages s'écrivent en `implicit_some`.
            let config = ron::Options::default()
                .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
                .from_str(&text)
                .unwrap_or_else(|e| panic!("{file} : {e}"));
            characters.0.insert(id.to_string(), config);
        }
        let ids = characters.0.keys().cloned().collect();
        (characters, ids)
    }

    /// Parité de `ResolvedBalance`, **champ à champ**, entre le chemin d'avant E3c (la configuration lue
    /// telle quelle) et celui d'après (le contenu de l'enveloppe `Asset`), pour chacun des cinq jeux et 1 à 4
    /// joueurs, sur les vrais `wave_config.ron`, `economy.ron` et `perks.ron` ; les trois champs nouveaux
    /// (`economy.shells_drop`, `economy.shop_prices`, `waves_enabled`) sont comparés à ce que lisaient les
    /// systèmes avant E3c (l'asset d'économie, la présence du handle de vagues).
    #[test]
    fn resolved_balance_is_identical_through_the_envelope_and_the_table() {
        let (characters, ids) = characters();
        let mut seen_waves = 0;
        let mut seen_shells = 0;
        let mut seen_prices = 0;
        for assets in games() {
            let waves: Option<(WaveConfigAsset, WaveConfig)> =
                both(&assets, "waves/wave_config.ron");
            let economy: Option<(EconomyConfigAsset, EconomyConfig)> =
                both(&assets, "economy/economy.ron");
            let perks: Option<(PerksConfigAsset, PerksConfig)> = both(&assets, "economy/perks.ron");
            for players in 1..=4u32 {
                let via_envelope = resolve_balance(
                    players,
                    waves.as_ref().map(|(envelope, _)| &envelope.0),
                    economy.as_ref().map(|(envelope, _)| &envelope.0),
                    perks.as_ref().map(|(envelope, _)| &envelope.0),
                    &characters,
                    ids.iter(),
                );
                let direct = resolve_balance(
                    players,
                    waves.as_ref().map(|(_, data)| data),
                    economy.as_ref().map(|(_, data)| data),
                    perks.as_ref().map(|(_, data)| data),
                    &characters,
                    ids.iter(),
                );
                // Déstructuration exhaustive : un champ de plus à `ResolvedBalance`, `ResolvedWaveConfig` ou
                // `ResolvedEconomyConfig` casse la compilation de ce test tant qu'il n'est pas comparé.
                let ResolvedBalance {
                    waves: w,
                    waves_enabled,
                    economy: e,
                    perks: p,
                    health_max_by_character,
                } = &via_envelope;
                let crate::balance::ResolvedWaveConfig {
                    base_enemies,
                    enemies_per_wave,
                    max_random_variance,
                    min_wave_delay_frames,
                    grace_period_frames,
                    max_concurrent_enemies,
                    spawn_batch_size,
                    spawn_interval_frames,
                    min_player_distance,
                    max_player_distance,
                    wave_tiers,
                    health_multiplier_per_wave,
                    damage_multiplier_per_wave,
                    max_wave,
                } = w;
                let crate::balance::ResolvedEconomyConfig {
                    kill_points,
                    hit_points,
                    repair_points,
                    nuke_points,
                    repair_points_cap_per_wave,
                    refill_price_ratio,
                    shells_drop,
                    shop_prices,
                } = e;
                let d = &direct;
                let what = format!("{} à {players} joueur(s)", assets.display());
                assert_eq!(*base_enemies, d.waves.base_enemies, "base_enemies, {what}");
                assert_eq!(*enemies_per_wave, d.waves.enemies_per_wave, "{what}");
                assert_eq!(*max_random_variance, d.waves.max_random_variance, "{what}");
                assert_eq!(
                    *min_wave_delay_frames, d.waves.min_wave_delay_frames,
                    "{what}"
                );
                assert_eq!(*grace_period_frames, d.waves.grace_period_frames, "{what}");
                assert_eq!(
                    *max_concurrent_enemies, d.waves.max_concurrent_enemies,
                    "{what}"
                );
                assert_eq!(*spawn_batch_size, d.waves.spawn_batch_size, "{what}");
                assert_eq!(
                    *spawn_interval_frames, d.waves.spawn_interval_frames,
                    "{what}"
                );
                assert_eq!(*min_player_distance, d.waves.min_player_distance, "{what}");
                assert_eq!(*max_player_distance, d.waves.max_player_distance, "{what}");
                assert_eq!(wave_tiers, &d.waves.wave_tiers, "wave_tiers, {what}");
                assert_eq!(
                    *health_multiplier_per_wave, d.waves.health_multiplier_per_wave,
                    "{what}"
                );
                assert_eq!(
                    *damage_multiplier_per_wave, d.waves.damage_multiplier_per_wave,
                    "{what}"
                );
                assert_eq!(*max_wave, d.waves.max_wave, "{what}");
                assert_eq!(*kill_points, d.economy.kill_points, "{what}");
                assert_eq!(*hit_points, d.economy.hit_points, "{what}");
                assert_eq!(*repair_points, d.economy.repair_points, "{what}");
                assert_eq!(*nuke_points, d.economy.nuke_points, "{what}");
                assert_eq!(
                    *repair_points_cap_per_wave, d.economy.repair_points_cap_per_wave,
                    "{what}"
                );
                assert_eq!(*refill_price_ratio, d.economy.refill_price_ratio, "{what}");
                assert_eq!(*shells_drop, d.economy.shells_drop, "{what}");
                assert_eq!(shop_prices, &d.economy.shop_prices, "{what}");
                assert_eq!(p, &d.perks, "perks, {what}");
                assert_eq!(
                    health_max_by_character, &d.health_max_by_character,
                    "{what}"
                );
                assert_eq!(*waves_enabled, d.waves_enabled, "{what}");
                assert_eq!(via_envelope, direct, "ResolvedBalance entier, {what}");
                // La santé résolue vaut l'expression du personnage évaluée pour `players` : `zombie_scaled`
                // (`10 + (players - 1) * 40`) en dépend, `player` non.
                for (id, config) in &characters.0 {
                    assert_eq!(
                        health_max_by_character[id],
                        config.base_health.max.resolve(players).unwrap(),
                        "santé de « {id} », {what}"
                    );
                }
                assert_eq!(
                    health_max_by_character["zombie_scaled"],
                    Fixed::from_num(10 + 40 * (players - 1)),
                    "{what}"
                );

                // Ce que lisaient les systèmes avant E3c.
                assert_eq!(
                    *waves_enabled,
                    waves.is_some(),
                    "waves_enabled = `wave_config.is_some()`, {what}"
                );
                let old_economy = economy.as_ref().map(|(_, data)| data);
                assert_eq!(
                    *shells_drop,
                    old_economy.and_then(|economy| economy.shells_drop),
                    "shells_drop, {what}"
                );
                assert_eq!(
                    *shop_prices,
                    old_economy
                        .map(|economy| economy.shop_prices.clone())
                        .unwrap_or_default(),
                    "shop_prices, {what}"
                );
                seen_waves += usize::from(*waves_enabled);
                seen_shells += usize::from(shells_drop.is_some());
                seen_prices += usize::from(!shop_prices.is_empty());
            }
        }
        // Le test ne vaut que s'il rencontre les trois cas : un jeu à vagues, un à drop de `shells`, un à
        // prix de présentoirs (zombies, gungeon…), et au moins un sans vagues.
        assert!(seen_waves > 0 && seen_waves < 4 * games().len(), "vagues");
        assert!(seen_shells > 0, "shells_drop");
        assert!(seen_prices > 0, "shop_prices");
    }

    /// `shells_drop` et `shop_prices` sont des littéraux (pas des `NumOrExpr`) : copiés tels quels quel que soit
    /// `players`, alors que `kill_points` (une expression) suit le nombre de joueurs.
    #[test]
    fn economy_literals_do_not_depend_on_the_player_count() {
        let economy: EconomyConfig = ron::from_str(
            "(kill_points: \"60 + players\", shells_drop: Some((2, 5)), shop_prices: {Rare: 50})",
        )
        .unwrap();
        let characters = CharacterConfigs::default();
        let one = resolve_balance(1, None, Some(&economy), None, &characters, []);
        let four = resolve_balance(4, None, Some(&economy), None, &characters, []);
        assert_eq!(one.economy.kill_points, 61);
        assert_eq!(four.economy.kill_points, 64);
        assert_eq!(one.economy.shells_drop, Some((2, 5)));
        assert_eq!(four.economy.shells_drop, one.economy.shells_drop);
        assert_eq!(four.economy.shop_prices, one.economy.shop_prices);
        assert!(!one.waves_enabled);
    }
}
