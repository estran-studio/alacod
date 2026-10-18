//! Enveloppe `Asset` de [`CharacterConfig`] et synchro du registre de simulation (D69 étape 2, E3a).
//!
//! `CharacterConfig` est une donnée de simulation pure (`Deserialize`) ; ce fichier, côté application,
//! la charge comme asset Bevy et publie [`CharacterConfigs`] que lit la simulation (`apply_inputs`,
//! `rollback_resolve_damage_events`, `rollback_apply_accumulated_damage`, `move_enemies`, les créateurs
//! de personnages…). `#[serde(transparent)]` : les `.ron` existants se chargent sans modification.
//!
//! # Moment du remplissage (décision D2 du plan, `docs/taches/rapports/dette-d69-e3a-character-configs.md`)
//!
//! [`sync_character_configs`] tourne en `PreUpdate`, **avant** `RunGgrsSystems` (donc avant tout tick
//! de `GgrsSchedule`), à chaque frame, et ne copie que ce qui manque ou a changé :
//!
//! - **entrée absente** de [`CharacterConfigs`] dont l'asset est lisible : copiée. C'est le chemin du
//!   chargement initial ; il ne dépend d'aucun événement, donc d'aucun décalage de frame entre
//!   l'insertion de l'asset et son `AssetEvent` ;
//! - **`AssetEvent::Modified`** (rechargement à chaud, `file_watcher` natif) : l'entrée de l'asset
//!   modifié est recopiée (une surcharge de scénario posée sur cette entrée est alors remplacée). Les
//!   événements `Added`/`LoadedWithDependencies` ne déclenchent **rien** : une surcharge de scénario posée
//!   sur la ressource après le remplissage n'est jamais écrasée par eux.
//!
//! `loading_asset_system` ne quitte `AppState::Loading` qu'une fois tous les assets chargés, et le
//! remplissage a lieu au `PreUpdate` suivant au plus tard, avant le `StateTransition` qui peut entrer
//! dans le lobby puis `GameLoading` : la ressource est pleine avant le moindre spawn de personnage et
//! avant `resolve_balance_system` (`OnEnter(GameLoading)`).

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_ggrs::RunGgrsSystems;
use serde::Deserialize;

use super::config::{CharacterConfig, CharacterConfigs};
use crate::global_asset::GlobalAsset;

/// Enveloppe `Asset` d'un fichier de personnage (`assets/characters/*.ron`).
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct CharacterConfigAsset(pub CharacterConfig);

pub struct CharacterConfigAssetPlugin;

impl Plugin for CharacterConfigAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<CharacterConfigAsset>::new(&["ron"]));
        app.add_systems(PreUpdate, sync_character_configs.before(RunGgrsSystems));
    }
}

/// Publie les `CharacterConfig` chargés dans [`CharacterConfigs`] (voir la doc du module).
pub fn sync_character_configs(
    global: Option<Res<GlobalAsset>>,
    assets: Res<Assets<CharacterConfigAsset>>,
    mut events: MessageReader<AssetEvent<CharacterConfigAsset>>,
    mut configs: ResMut<CharacterConfigs>,
) {
    // Toujours vidé, même sans `GlobalAsset` : le curseur ne prend pas de retard.
    let modified: Vec<AssetId<CharacterConfigAsset>> = events
        .read()
        .filter_map(|event| match event {
            AssetEvent::Modified { id } => Some(*id),
            _ => None,
        })
        .collect();
    let Some(global) = global else {
        return;
    };
    for (key, handle) in &global.character_configs {
        let Some(asset) = assets.get(handle) else {
            continue;
        };
        let is_modified = modified.contains(&handle.id());
        if !is_modified && configs.0.contains_key(key) {
            continue;
        }
        if is_modified {
            info!("personnage « {key} » rechargé : CharacterConfigs mis à jour");
        } else {
            debug!("personnage « {key} » : CharacterConfigs rempli");
        }
        configs.0.insert(key.clone(), asset.0.clone());
    }
}
