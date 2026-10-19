//! Enveloppes `Asset` de l'économie (D69 étape 2, E3c) : côté application.
//!
//! [`EconomyConfig`] et [`PerksConfig`] (`economy.rs`) sont des données de simulation pures
//! (`Deserialize`). Ce fichier les charge comme assets Bevy ; la simulation, elle, ne lit jamais
//! ces assets : [`balance_assets::resolve_balance_system`](crate::balance_assets::resolve_balance_system)
//! les résout une fois à `OnEnter(GameLoading)` en [`ResolvedBalance`](crate::balance::ResolvedBalance)
//! (points, prix des perks, drop de `shells`, prix des présentoirs). `#[serde(transparent)]` :
//! les `.ron` existants se chargent sans modification.

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use serde::Deserialize;

use crate::economy::{EconomyConfig, PerksConfig};

/// Enveloppe `Asset` de `economy/economy.ron` (kind de contenu `Economy`).
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct EconomyConfigAsset(pub EconomyConfig);

/// Enveloppe `Asset` de `economy/perks.ron` (kind de contenu `Perk`).
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct PerksConfigAsset(pub PerksConfig);

/// Chargeurs RON de l'économie, ajoutés juste avant `EconomyPlugin` (simulation) par
/// `CoreSetupPlugin` : l'ordre des types rollback de `EconomyPlugin` ne change pas.
pub struct EconomyAssetsPlugin;

impl Plugin for EconomyAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<EconomyConfigAsset>::new(&["ron"]));
        app.add_plugins(RonAssetPlugin::<PerksConfigAsset>::new(&["ron"]));
    }
}
