//! Enveloppe `Asset` de la configuration des vagues (D69 étape 2, E3c) : côté application.
//!
//! [`WaveConfig`] (`waves/config.rs`) est une donnée de simulation pure (`Deserialize`). Ce fichier la
//! charge comme asset Bevy (`#[serde(transparent)]` : les `waves/wave_config.ron` existants se chargent sans
//! modification). La simulation ne lit jamais cet asset : `balance_assets::resolve_balance_system` le résout
//! une fois à `OnEnter(GameLoading)` en `ResolvedBalance::waves` (et `waves_enabled`).

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use serde::Deserialize;

use super::config::WaveConfig;

/// Enveloppe `Asset` de `waves/wave_config.ron` (kind de contenu `Wave`).
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct WaveConfigAsset(pub WaveConfig);

/// Chargeur RON des vagues, ajouté juste avant `WaveSystemPlugin` (simulation) par `CoreSetupPlugin`.
pub struct WaveAssetsPlugin;

impl Plugin for WaveAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<WaveConfigAsset>::new(&["ron"]));
    }
}
