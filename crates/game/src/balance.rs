//! F5 (chantier m0-v11) : équilibrage par nombre de joueurs.
//!
//! Les champs numériques de vagues, prix et santé acceptent une expression
//! [`content::expr::NumOrExpr`] avec la variable `players` (voir `docs/conventions.md`
//! §18). [`resolve_balance_system`] évalue ces champs **une fois**, au démarrage de la
//! partie (`OnEnter(AppState::GameLoading)`), en [`ResolvedBalance`] : des valeurs
//! concrètes, figées pour toute la partie. Les systèmes de simulation (vagues, économie,
//! interactions, création de personnages) lisent `ResolvedBalance`, jamais les assets
//! d'origine — **aucune expression ne vit dans l'état rollback**.
//!
//! Nombre de joueurs : `OnlineState::Online` → `max_player` de la session ggrs (source
//! autoritaire en ligne) ; sinon `PlayersCount` (partie locale ou scénario). Une erreur
//! d'expression (parse, division par zéro, identifiant inconnu, valeur hors domaine) est
//! un **échec du chargement** : `resolve_balance_system` panique avec un message qui nomme
//! le fichier, le champ et l'erreur — jamais de valeur par défaut silencieuse.
//!
//! Le point d'évaluation (`OnEnter(GameLoading)`) est après le chargement des assets et
//! avant le spawn des joueurs/personnages (Update de `GameLoading`) et avant la première
//! vague ; un redémarrage local (`run_state::plan_run_request` → `GameLoading`) re-résout
//! avec le même nombre de joueurs (inchangé).

use bevy::prelude::*;
use bevy_fixed::fixed_math::{self, Fixed};
use content::expr::NumOrExpr;
use sim_core::players::PlayersCount;
use std::collections::{BTreeMap, HashMap};

use crate::character::config::CharacterConfig;
use crate::core::OnlineState;
use crate::economy::{EconomyConfig, PerkModifierDef, PerksConfig};
use crate::global_asset::GlobalAsset;
use crate::jjrs::GggrsSessionConfiguration;
use crate::waves::config::WaveConfig;

/// Palier de vague résolu (`WaveTier` aux valeurs concrètes).
#[derive(Debug, Clone)]
pub struct ResolvedWaveTier {
    pub max_wave: u32,
    pub enemy_probabilities: HashMap<String, u32>,
}

/// `WaveConfig` résolue : mêmes champs, valeurs concrètes, méthodes de calcul inchangées.
#[derive(Debug, Clone)]
pub struct ResolvedWaveConfig {
    pub base_enemies: u32,
    pub enemies_per_wave: u32,
    pub max_random_variance: u32,
    pub min_wave_delay_frames: u32,
    pub grace_period_frames: u32,
    pub max_concurrent_enemies: u32,
    pub spawn_batch_size: u32,
    pub spawn_interval_frames: u32,
    pub min_player_distance: Fixed,
    pub max_player_distance: Fixed,
    pub wave_tiers: Vec<ResolvedWaveTier>,
    pub health_multiplier_per_wave: Fixed,
    pub damage_multiplier_per_wave: Fixed,
    pub max_wave: Option<u32>,
}

impl ResolvedWaveConfig {
    /// Get the wave tier for a given wave number
    pub fn get_tier(&self, wave: u32) -> Option<&ResolvedWaveTier> {
        self.wave_tiers.iter().find(|t| wave <= t.max_wave)
    }

    /// Calculate the total enemy count for a wave
    pub fn calculate_enemy_count(&self, wave: u32, variance: u32) -> u32 {
        let base = self.base_enemies;
        let scaling = wave.saturating_sub(1) * self.enemies_per_wave;
        let clamped_variance = variance.min(self.max_random_variance);
        base + scaling + clamped_variance
    }

    /// Calculate health multiplier for a wave (as Fixed, 1.0 = 100%)
    pub fn calculate_health_multiplier(&self, wave: u32) -> Fixed {
        let wave_bonus = self.health_multiplier_per_wave * Fixed::from_num(wave.saturating_sub(1));
        fixed_math::FIXED_ONE + wave_bonus
    }

    /// Calculate damage multiplier for a wave (as Fixed, 1.0 = 100%)
    pub fn calculate_damage_multiplier(&self, wave: u32) -> Fixed {
        let wave_bonus = self.damage_multiplier_per_wave * Fixed::from_num(wave.saturating_sub(1));
        fixed_math::FIXED_ONE + wave_bonus
    }
}

/// `EconomyConfig` résolue : mêmes champs, valeurs concrètes.
#[derive(Debug, Clone)]
pub struct ResolvedEconomyConfig {
    pub kill_points: u32,
    pub hit_points: u32,
    pub repair_points: u32,
    pub nuke_points: u32,
    pub repair_points_cap_per_wave: Option<u32>,
    pub refill_price_ratio: Fixed,
}

impl ResolvedEconomyConfig {
    /// Prix de recharge d'une arme murale déjà possédée, arrondi à l'unité la plus proche.
    pub fn refill_price(&self, price: u32) -> u32 {
        (Fixed::from_num(price).saturating_mul(self.refill_price_ratio))
            .round()
            .to_num::<u32>()
    }
}

/// Un perk résolu (`PerkDef` aux valeurs concrètes) : `price` en `u32`, modificateurs
/// inchangés (`PerkModifierDef.value` reste un `Fixed` littéral — hors périmètre F5).
#[derive(Debug, Clone)]
pub struct ResolvedPerkDef {
    pub name: String,
    pub price: u32,
    pub modifiers: Vec<PerkModifierDef>,
}

/// Résultat de la résolution F5, inséré par [`resolve_balance_system`] à
/// `OnEnter(AppState::GameLoading)` et lu par les systèmes de simulation. Ressource
/// ordinaire, **hors rollback** (comme `Assets`).
#[derive(Resource, Debug, Clone)]
pub struct ResolvedBalance {
    pub waves: ResolvedWaveConfig,
    pub economy: ResolvedEconomyConfig,
    /// Perks par id (`PerksConfig` garde sa clé), prix résolus.
    pub perks: BTreeMap<String, ResolvedPerkDef>,
    /// Santé max par personnage, clé = `CharacterId` (`asset_name_ref`), résolue pour la
    /// partie. Construit depuis les mêmes `character_configs` que `spawn_enemy`/
    /// `create_player` : toute clé que ceux-ci chercheront est présente.
    pub health_max_by_character: BTreeMap<String, Fixed>,
}

/// Résout un champ en `u32` ou panique avec le contexte (fichier, champ, joueurs, erreur).
fn resolve_u32(value: &NumOrExpr, players: u32, what: &str) -> u32 {
    value.resolve_u32(players).unwrap_or_else(|err| {
        panic!("équilibrage F5 ({players} joueurs) : {what} : {err} — corrigez le contenu RON")
    })
}

/// Résout un champ en `Fixed` ou panique avec le contexte (voir [`resolve_u32`]).
fn resolve_fixed(value: &NumOrExpr, players: u32, what: &str) -> Fixed {
    value.resolve(players).unwrap_or_else(|err| {
        panic!("équilibrage F5 ({players} joueurs) : {what} : {err} — corrigez le contenu RON")
    })
}

pub(crate) fn resolve_waves(config: &WaveConfig, players: u32) -> ResolvedWaveConfig {
    ResolvedWaveConfig {
        base_enemies: resolve_u32(
            &config.base_enemies,
            players,
            "wave_config.ron, champ base_enemies",
        ),
        enemies_per_wave: resolve_u32(
            &config.enemies_per_wave,
            players,
            "wave_config.ron, champ enemies_per_wave",
        ),
        max_random_variance: resolve_u32(
            &config.max_random_variance,
            players,
            "wave_config.ron, champ max_random_variance",
        ),
        min_wave_delay_frames: resolve_u32(
            &config.min_wave_delay_frames,
            players,
            "wave_config.ron, champ min_wave_delay_frames",
        ),
        grace_period_frames: resolve_u32(
            &config.grace_period_frames,
            players,
            "wave_config.ron, champ grace_period_frames",
        ),
        max_concurrent_enemies: resolve_u32(
            &config.max_concurrent_enemies,
            players,
            "wave_config.ron, champ max_concurrent_enemies",
        ),
        spawn_batch_size: resolve_u32(
            &config.spawn_batch_size,
            players,
            "wave_config.ron, champ spawn_batch_size",
        ),
        spawn_interval_frames: resolve_u32(
            &config.spawn_interval_frames,
            players,
            "wave_config.ron, champ spawn_interval_frames",
        ),
        min_player_distance: resolve_fixed(
            &config.min_player_distance,
            players,
            "wave_config.ron, champ min_player_distance",
        ),
        max_player_distance: resolve_fixed(
            &config.max_player_distance,
            players,
            "wave_config.ron, champ max_player_distance",
        ),
        wave_tiers: config
            .wave_tiers
            .iter()
            .enumerate()
            .map(|(i, tier)| ResolvedWaveTier {
                max_wave: resolve_u32(
                    &tier.max_wave,
                    players,
                    &format!("wave_config.ron, palier {i}, champ max_wave"),
                ),
                enemy_probabilities: tier
                    .enemy_probabilities
                    .iter()
                    .map(|(name, weight)| {
                        (
                            name.clone(),
                            resolve_u32(
                                weight,
                                players,
                                &format!("wave_config.ron, palier {i}, poids de « {name} »"),
                            ),
                        )
                    })
                    .collect(),
            })
            .collect(),
        health_multiplier_per_wave: resolve_fixed(
            &config.health_multiplier_per_wave,
            players,
            "wave_config.ron, champ health_multiplier_per_wave",
        ),
        damage_multiplier_per_wave: resolve_fixed(
            &config.damage_multiplier_per_wave,
            players,
            "wave_config.ron, champ damage_multiplier_per_wave",
        ),
        max_wave: config
            .max_wave
            .as_ref()
            .map(|v| resolve_u32(v, players, "wave_config.ron, champ max_wave")),
    }
}

fn resolve_economy(config: &EconomyConfig, players: u32) -> ResolvedEconomyConfig {
    ResolvedEconomyConfig {
        kill_points: resolve_u32(
            &config.kill_points,
            players,
            "economy.ron, champ kill_points",
        ),
        hit_points: resolve_u32(&config.hit_points, players, "economy.ron, champ hit_points"),
        repair_points: resolve_u32(
            &config.repair_points,
            players,
            "economy.ron, champ repair_points",
        ),
        nuke_points: resolve_u32(
            &config.nuke_points,
            players,
            "economy.ron, champ nuke_points",
        ),
        repair_points_cap_per_wave: config
            .repair_points_cap_per_wave
            .as_ref()
            .map(|v| resolve_u32(v, players, "economy.ron, champ repair_points_cap_per_wave")),
        refill_price_ratio: resolve_fixed(
            &config.refill_price_ratio,
            players,
            "economy.ron, champ refill_price_ratio",
        ),
    }
}

fn resolve_perks(config: &PerksConfig, players: u32) -> BTreeMap<String, ResolvedPerkDef> {
    config
        .0
        .iter()
        .map(|(id, def)| {
            (
                id.clone(),
                ResolvedPerkDef {
                    name: def.name.clone(),
                    price: resolve_u32(
                        &def.price,
                        players,
                        &format!("perks.ron, perk « {id} », champ price"),
                    ),
                    modifiers: def.modifiers.clone(),
                },
            )
        })
        .collect()
}

/// Résout tous les champs F5 (vagues, économie, perks, santé des personnages) pour le
/// nombre de joueurs de la partie et insère [`ResolvedBalance`].
///
/// Enregistré sur `OnEnter(AppState::GameLoading)` (voir la doc du module pour la
/// justification du point d'évaluation).
#[allow(clippy::too_many_arguments)]
pub fn resolve_balance_system(
    mut commands: Commands,
    global_assets: Res<GlobalAsset>,
    wave_configs: Res<Assets<WaveConfig>>,
    economy_configs: Res<Assets<EconomyConfig>>,
    perks_configs: Res<Assets<PerksConfig>>,
    character_configs: Res<Assets<CharacterConfig>>,
    online: Res<OnlineState>,
    ggrs_config: Res<GggrsSessionConfiguration>,
    players_count: Res<PlayersCount>,
) {
    let players = match *online {
        OnlineState::Online => ggrs_config.connection.max_player as u32,
        OnlineState::Offline | OnlineState::Unset => players_count.0 as u32,
    };

    let wave_config = global_assets
        .wave_config
        .as_ref()
        .and_then(|h| wave_configs.get(h))
        .cloned()
        .unwrap_or_default();
    let economy_config = global_assets
        .economy_config
        .as_ref()
        .and_then(|h| economy_configs.get(h))
        .cloned()
        .unwrap_or_default();
    let perks_config = global_assets
        .perks_config
        .as_ref()
        .and_then(|h| perks_configs.get(h))
        .cloned()
        .unwrap_or_default();

    let waves = resolve_waves(&wave_config, players);
    let economy = resolve_economy(&economy_config, players);
    let perks = resolve_perks(&perks_config, players);

    let mut health_max_by_character = BTreeMap::new();
    for (id, handle) in global_assets.character_configs.iter() {
        let config = character_configs.get(handle).unwrap_or_else(|| {
            panic!("équilibrage F5 : config du personnage « {id} » pas encore chargée")
        });
        let what = format!("personnage « {id} », champ base_health.max");
        let max = resolve_fixed(&config.base_health.max, players, &what);
        health_max_by_character.insert(id.clone(), max);
    }

    commands.insert_resource(ResolvedBalance {
        waves,
        economy,
        perks,
        health_max_by_character,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// F5 (chantier m0-v11), périmètre A.2 : une expression dépend du nombre de joueurs
    /// et se résout en valeurs concrètes différentes par partie.
    #[test]
    fn expression_resolves_per_player_count() {
        let expr: NumOrExpr = ron::from_str("\"10.0 + (players - 1) * 40.0\"").unwrap();
        assert_eq!(
            resolve_fixed(&expr, 2, "wave_config.ron, champ base_enemies").to_num::<f32>(),
            50.0
        );
        assert_eq!(
            resolve_fixed(&expr, 4, "wave_config.ron, champ base_enemies").to_num::<f32>(),
            130.0
        );
    }

    /// F5 (chantier m0-v11), périmètre A.3 : division par zéro = échec du chargement
    /// (panic nommant le champ), jamais de valeur par défaut silencieuse.
    #[test]
    #[should_panic(expected = "division par zéro")]
    fn division_by_zero_is_a_load_failure() {
        let expr: NumOrExpr = ron::from_str("\"1.0 / 0.0\"").unwrap();
        resolve_fixed(&expr, 2, "wave_config.ron, champ base_enemies");
    }

    /// F5 (chantier m0-v11), périmètre A.3 : identifiant inconnu = échec du chargement
    /// (seul `players` existe dans le contexte).
    #[test]
    #[should_panic(expected = "identifiant inconnu")]
    fn unknown_identifier_is_a_load_failure() {
        let expr: NumOrExpr = ron::from_str("\"1.0 + waves\"").unwrap();
        resolve_fixed(&expr, 2, "wave_config.ron, champ base_enemies");
    }
}
