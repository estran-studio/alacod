//! Enveloppe `Asset` des power-ups et publication de [`PowerUpsTable`] (D69 étape 2, E3c) : côté application.
//!
//! [`PowerUpsConfig`] (`powerups.rs`) est une donnée de simulation pure (`Deserialize`). Ce fichier la
//! charge comme asset Bevy (`#[serde(transparent)]` : les `items/powerups.ron` existants se chargent sans
//! modification) et la copie dans [`PowerUpsTable`], que lisent `powerup_pickup_detect_system`,
//! `apply_powerup_actions_system` et `loot_drop_on_death_system` (avant E3c : `Assets<PowerUpsConfig>`, relu à
//! chaque frame).
//!
//! # Moment du remplissage
//!
//! [`resolve_powerups_table_system`] tourne à `OnEnter(AppState::GameLoading)`, **après**
//! `balance_assets::resolve_balance_system` : les assets sont chargés (`loading_asset_system` ne quitte
//! `Loading` qu'une fois tous chargés), la table est pleine avant la première frame simulée et avant tout
//! spawn, et une surcharge de scénario posée sur l'**asset** (`apply_powerup_drop_chance_override`, en
//! `Update` dès que l'asset est chargé, donc bien avant `GameLoading`) est déjà dedans. Un redémarrage local
//! repasse par `GameLoading` et recopie l'asset. Un rechargement à chaud pendant la partie ne change plus
//! la table (comme `ResolvedBalance`).

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use serde::Deserialize;

use crate::global_asset::GlobalAsset;
use crate::powerups::{PowerUpsConfig, PowerUpsTable};

/// Enveloppe `Asset` de `items/powerups.ron` (kind de contenu `PowerUp`).
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct PowerUpsConfigAsset(pub PowerUpsConfig);

/// Chargeur RON des power-ups, ajouté juste avant `PowerUpsPlugin` (simulation) par `CoreSetupPlugin` :
/// la position de `PowerUpsPlugin` dans `build` ne change pas.
pub struct PowerUpsAssetsPlugin;

impl Plugin for PowerUpsAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<PowerUpsConfigAsset>::new(&["ron"]));
    }
}

/// Copie le contenu de l'asset dans [`PowerUpsTable`] (`None` si le jeu ne déclare aucun dossier `PowerUp`).
pub fn resolve_powerups_table_system(
    global: Res<GlobalAsset>,
    assets: Res<Assets<PowerUpsConfigAsset>>,
    mut table: ResMut<PowerUpsTable>,
) {
    table.0 = global
        .powerups_config
        .as_ref()
        .and_then(|handle| assets.get(handle))
        .map(|asset| asset.0.clone());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::powerups::PowerUpDef;
    use std::collections::BTreeMap;

    /// Parité de la table avec la configuration lue directement (le chemin d'avant E3c), champ à champ, sur les
    /// `items/powerups.ron` des jeux livrés, en passant par l'enveloppe, `Assets` et le système de publication.
    #[test]
    fn table_is_the_envelope_content_field_by_field() {
        let games = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../games");
        let mut seen = 0;
        for entry in std::fs::read_dir(games).unwrap() {
            let file = entry.unwrap().path().join("assets/items/powerups.ron");
            let Ok(text) = std::fs::read_to_string(&file) else {
                continue;
            };
            let envelope: PowerUpsConfigAsset = ron::from_str(&text).unwrap();
            let mut assets = Assets::<PowerUpsConfigAsset>::default();
            let handle = assets.add(envelope.clone());
            let mut world = World::new();
            world.insert_resource(assets);
            world.insert_resource(PowerUpsTable::default());
            world.insert_resource(global_with(Some(handle)));
            bevy::ecs::system::RunSystemOnce::run_system_once(
                &mut world,
                resolve_powerups_table_system,
            )
            .unwrap();
            let table = world
                .resource::<PowerUpsTable>()
                .0
                .as_ref()
                .expect("table remplie");
            // Le chemin d'avant E3c : le fichier lu directement comme `PowerUpsConfig` (l'asset était la
            // configuration elle-même).
            let direct: PowerUpsConfig = ron::from_str(&text).unwrap();
            let PowerUpsConfig {
                drop_chance,
                powerups,
            } = &direct;
            assert_eq!(table.drop_chance, *drop_chance, "{}", file.display());
            let same = |a: &BTreeMap<String, PowerUpDef>, b: &BTreeMap<String, PowerUpDef>| {
                a.keys().eq(b.keys())
                    && a.iter().all(|(id, x)| {
                        let y = &b[id];
                        let PowerUpDef {
                            name,
                            weight,
                            pickup_range,
                            lifetime_frames,
                            actions,
                        } = x;
                        *name == y.name
                            && *weight == y.weight
                            && *pickup_range == y.pickup_range
                            && *lifetime_frames == y.lifetime_frames
                            && format!("{actions:?}") == format!("{:?}", y.actions)
                    })
            };
            assert!(same(&table.powerups, powerups), "{}", file.display());
            seen += 1;
        }
        assert!(seen >= 2, "au moins deux jeux déclarent des power-ups");
    }

    /// Sans handle (jeu sans dossier `PowerUp`) : table vide, et une table d'une partie précédente est effacée.
    #[test]
    fn game_without_powerups_clears_the_table() {
        let mut world = World::new();
        world.insert_resource(Assets::<PowerUpsConfigAsset>::default());
        world.insert_resource(PowerUpsTable(Some(PowerUpsConfig {
            drop_chance: bevy_fixed::fixed_math::FIXED_ONE,
            powerups: BTreeMap::new(),
        })));
        world.insert_resource(global_with(None));
        bevy::ecs::system::RunSystemOnce::run_system_once(
            &mut world,
            resolve_powerups_table_system,
        )
        .unwrap();
        assert!(world.resource::<PowerUpsTable>().0.is_none());
    }

    fn global_with(powerups_config: Option<Handle<PowerUpsConfigAsset>>) -> GlobalAsset {
        GlobalAsset {
            spritesheets: Default::default(),
            animations: Default::default(),
            character_configs: Default::default(),
            weapons: Default::default(),
            melee_weapons: Default::default(),
            slash_effect_spritesheet: Default::default(),
            slash_effect_animation: Default::default(),
            wave_config: None,
            economy_config: None,
            perks_config: None,
            powerups_config,
        }
    }
}
