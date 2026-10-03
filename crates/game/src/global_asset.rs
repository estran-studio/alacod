//! Pont entre le registre de contenu (`content::registry::Registry`, T1.5) et les
//! `Handle<...>` Bevy utilisés par le rendu et l'animation.
//!
//! Le registre est la **source de vérité des ids** (quels personnages, quelles armes...
//! existent pour ce jeu) : `character_configs`, `weapons` et `melee_weapons` sont chargés
//! d'après ses entrées. D3 : les feuilles de sprite et configs d'animation aussi, d'après la
//! table du kind `SpriteSheet` (`sprites/sprites.ron` des jeux) : un id par entrée (le
//! `asset_name_ref` d'un personnage, le `sprite_config.name` d'une arme, ou
//! [`SLASH_EFFECT_SPRITE_ID`]), avec sa configuration d'animation et ses calques. Avant D3,
//! ces chemins étaient écrits ici en dur ; un jeu ne charge plus que ce qu'il déclare.

use animation::{AnimationMapConfig, SpriteSheetConfig};
use bevy::{platform::collections::hash_map::HashMap, prelude::*};
use content::registry::Registry;

use crate::{
    character::config::CharacterConfig,
    core::{AppState, OnlineState},
    economy::{EconomyConfig, PerksConfig},
    powerups::PowerUpsConfig,
    waves::WaveConfig,
    weapons::{melee::MeleeWeaponsConfig, WeaponsConfig},
};

/// D3 : id de la feuille de l'effet de coup de mêlée (`ui::weapon_visuals::
/// spawn_slash_effects`) dans la table `SpriteSheet`, calque `body`. Absente : pas d'effet
/// affiché.
pub const SLASH_EFFECT_SPRITE_ID: &str = "slash";

#[derive(Resource)]
pub struct GlobalAsset {
    pub spritesheets: HashMap<String, HashMap<String, Handle<SpriteSheetConfig>>>,
    pub animations: HashMap<String, Handle<AnimationMapConfig>>,
    pub character_configs: HashMap<String, Handle<CharacterConfig>>,
    pub weapons: Handle<WeaponsConfig>,
    pub melee_weapons: Handle<MeleeWeaponsConfig>,

    /// Effet de coup de mêlée (D3 : entrée [`SLASH_EFFECT_SPRITE_ID`] de la table
    /// `SpriteSheet`, `None` si le jeu ne la déclare pas).
    pub slash_effect_spritesheet: Option<Handle<SpriteSheetConfig>>,
    pub slash_effect_animation: Option<Handle<AnimationMapConfig>>,

    // Wave spawning config (optional - only loaded when the game declares a `Wave` folder)
    pub wave_config: Option<Handle<WaveConfig>>,

    /// Économie de run (T2.3, chantier C5 v1) : `Some` seulement si le jeu déclare un dossier
    /// de contenu `Economy`/`Perk` (comme `wave_config` pour `Wave`). Absent : les points et
    /// achats retombent sur `EconomyConfig::default()` (voir `economy::award_points_system`),
    /// et les perks n'ont aucune entrée connue (`interaction::handle_perk_purchase_interaction`
    /// refuse tout achat).
    pub economy_config: Option<Handle<EconomyConfig>>,
    pub perks_config: Option<Handle<PerksConfig>>,
    /// Power-ups (T2.5, chantier C1 v0) : `Some` seulement si le jeu déclare un dossier de
    /// contenu `PowerUp`, comme `wave_config`/`economy_config` ci-dessus. Absent :
    /// `powerups::loot_drop_on_death_system`/`powerup_pickup_detect_system` ne font rien
    /// (retour anticipé), aucun power-up ne tombe ni ne peut être ramassé.
    pub powerups_config: Option<Handle<PowerUpsConfig>>,
}

impl GlobalAsset {
    pub fn create(asset_server: &AssetServer, registry: &Registry) -> Self {
        // D3 : une entrée par id de la table `SpriteSheet` (calques + animation). Les clés
        // sont celles que cherchent `character::visuals` (`asset_name_ref`) et
        // `ui::weapon_visuals` (`sprite_config.name`), comme la table en dur d'avant D3.
        let mut spritesheets: HashMap<String, HashMap<String, Handle<SpriteSheetConfig>>> =
            HashMap::default();
        let mut animations: HashMap<String, Handle<AnimationMapConfig>> = HashMap::default();
        for entry in registry.sprite_sheets.values() {
            let layers = entry
                .layers
                .iter()
                .map(|(layer, path)| (layer.clone(), asset_server.load(path.clone())))
                .collect();
            spritesheets.insert(entry.id.as_str().to_string(), layers);
            animations.insert(
                entry.id.as_str().to_string(),
                asset_server.load(entry.animation.clone()),
            );
        }
        let slash_effect_spritesheet = spritesheets
            .get(SLASH_EFFECT_SPRITE_ID)
            .and_then(|layers| layers.get("body"))
            .cloned();
        let slash_effect_animation = animations.get(SLASH_EFFECT_SPRITE_ID).cloned();

        // character_configs : la clé est le `CharacterId` du registre (== `asset_name_ref`
        // aujourd'hui), le chemin vient de l'entrée du registre (source de vérité, T1.5).
        let character_configs = registry
            .characters
            .values()
            .map(|entry| {
                (
                    entry.id.as_str().to_string(),
                    asset_server.load(path_to_asset_string(&entry.file)),
                )
            })
            .collect();

        let weapons = registry
            .weapons
            .values()
            .next()
            .map(|entry| asset_server.load(path_to_asset_string(&entry.file)))
            .unwrap_or_else(|| {
                panic!("game.ron ne déclare aucun dossier de contenu `Weapon` (`content_folders`)")
            });
        let melee_weapons = registry
            .melee_weapons
            .values()
            .next()
            .map(|entry| asset_server.load(path_to_asset_string(&entry.file)))
            .unwrap_or_else(|| {
                panic!(
                    "game.ron ne déclare aucun dossier de contenu `MeleeWeapon` (`content_folders`)"
                )
            });
        let wave_config = registry
            .waves
            .values()
            .next()
            .map(|entry| asset_server.load(path_to_asset_string(&entry.file)));

        // Économie de run (T2.3) : `Some` seulement si `game.ron` déclare un dossier
        // `Economy`/`Perk`, comme `wave_config` ci-dessus.
        let economy_config = registry
            .economy
            .values()
            .next()
            .map(|entry| asset_server.load(path_to_asset_string(&entry.file)));
        // `registry.perks` a une entrée par perk (pas par fichier) : toutes celles d'un
        // même `perks.ron` partagent le même `file`, `.next()` suffit pour le retrouver
        // (comme `weapons`/`melee_weapons` ci-dessus, une seule table par jeu).
        let perks_config = registry
            .perks
            .values()
            .next()
            .map(|entry| asset_server.load(path_to_asset_string(&entry.file)));

        // Power-ups (T2.5) : `registry.powerups` a une entrée par power-up (pas par
        // fichier), comme `perks` ci-dessus — `.next()` suffit pour retrouver le fichier
        // unique du jeu.
        let powerups_config = registry
            .powerups
            .values()
            .next()
            .map(|entry| asset_server.load(path_to_asset_string(&entry.file)));

        Self {
            spritesheets,
            animations,
            character_configs,

            weapons,
            melee_weapons,

            // Effet de coup de mêlée (D3 : table `SpriteSheet`).
            slash_effect_spritesheet,
            slash_effect_animation,

            // Wave spawning config : seulement si le jeu déclare un dossier `Wave`.
            wave_config,

            // Économie de run (T2.3) : seulement si le jeu déclare `Economy`/`Perk`.
            economy_config,
            perks_config,

            // Power-ups (T2.5) : seulement si le jeu déclare `PowerUp`.
            powerups_config,
        }
    }
}

/// `AssetServer::load` prend un chemin relatif à `assets/`, comme une chaîne (pas de
/// séparateur Windows possible ici : le projet ne cible que Linux/wasm, voir
/// `crates/utils/src/test/mod.rs`).
fn path_to_asset_string(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub fn add_global_asset(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    registry: Res<Registry>,
) {
    let global_asset = GlobalAsset::create(&asset_server, &registry);

    commands.insert_resource(global_asset);
}

pub fn loading_asset_system(
    mut app_state: ResMut<NextState<AppState>>,
    online: Res<OnlineState>,
    global_assets: Res<GlobalAsset>,
    asset_server: Res<AssetServer>,
) {
    for (_, v) in global_assets.spritesheets.iter() {
        for (_, handle) in v.iter() {
            if !asset_server.load_state(handle).is_loaded() {
                return;
            }
        }
    }

    for (_, handle) in global_assets.animations.iter() {
        if !asset_server.load_state(handle).is_loaded() {
            return;
        }
    }

    for (_, handle) in global_assets.character_configs.iter() {
        if !asset_server.load_state(handle).is_loaded() {
            return;
        }
    }

    if !asset_server.load_state(&global_assets.weapons).is_loaded() {
        return;
    }
    if !asset_server
        .load_state(&global_assets.melee_weapons)
        .is_loaded()
    {
        return;
    }

    // Effet de coup de mêlée : seulement s'il est déclaré (D3). Ses handles sont aussi dans
    // `spritesheets`/`animations` ci-dessus, déjà attendus ; vérifiés ici pour le cas où
    // l'entrée changerait de forme.
    if let Some(handle) = &global_assets.slash_effect_spritesheet {
        if !asset_server.load_state(handle).is_loaded() {
            return;
        }
    }
    if let Some(handle) = &global_assets.slash_effect_animation {
        if !asset_server.load_state(handle).is_loaded() {
            return;
        }
    }

    // Check wave config (if loaded)
    if let Some(wave_config) = &global_assets.wave_config {
        if !asset_server.load_state(wave_config).is_loaded() {
            return;
        }
    }

    // Économie de run (T2.3) : mêmes règles que `wave_config` ci-dessus.
    if let Some(economy_config) = &global_assets.economy_config {
        if !asset_server.load_state(economy_config).is_loaded() {
            return;
        }
    }
    if let Some(perks_config) = &global_assets.perks_config {
        if !asset_server.load_state(perks_config).is_loaded() {
            return;
        }
    }
    // Power-ups (T2.5) : mêmes règles que `wave_config` ci-dessus.
    if let Some(powerups_config) = &global_assets.powerups_config {
        if !asset_server.load_state(powerups_config).is_loaded() {
            return;
        }
    }

    if matches!(*online, OnlineState::Online) {
        app_state.set(AppState::LobbyOnline);
    } else {
        app_state.set(AppState::LobbyLocal);
    }
    info!("loading of asset is done , now entering lobby");
}
