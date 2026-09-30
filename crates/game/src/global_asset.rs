//! Pont entre le registre de contenu (`content::registry::Registry`, T1.5) et les
//! `Handle<...>` Bevy utilisés par le rendu et l'animation.
//!
//! Le registre est la **source de vérité des ids** (quels personnages, quelles armes...
//! existent pour ce jeu) : `character_configs`, `weapons` et `melee_weapons` sont chargés
//! d'après ses entrées. Les feuilles de sprite et configs d'animation par `asset_name_ref`
//! (`spritesheets`, `animations`) restent une table en dur ici : leur chemin ne vient
//! d'aucun RON aujourd'hui (voir `docs/plan-engine.md` §5 A5, « Pipeline sprites », hors
//! périmètre de T1.5) ; seul le *sous-ensemble effectivement utilisé* par ce jeu (d'après
//! le registre) est chargé, pour qu'un jeu qui ne déclare pas de personnage "zombie_full"
//! (ex. `games/testbed`) n'exige plus ses sprites.

use animation::{AnimationMapConfig, SpriteSheetConfig};
use bevy::{platform::collections::hash_map::HashMap, prelude::*};
use content::registry::Registry;
use std::collections::BTreeSet;
use utils::bmap;

use crate::{
    character::config::CharacterConfig,
    core::{AppState, OnlineState},
    economy::{EconomyConfig, PerksConfig},
    powerups::PowerUpsConfig,
    waves::WaveConfig,
    weapons::{melee::MeleeWeaponsConfig, WeaponsConfig},
};

const PLAYER_SPRITESHEET_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/player_sheet.ron";
const PLAYER_SHIRT_SPRITESHEET_CONFIG_PATH: &str =
    "ZombieShooter/Sprites/Character/shirt_1_sheet.ron";
const PLAYER_HAIR_SPRITESHEET_CONFIG_PATH: &str =
    "ZombieShooter/Sprites/Character/hair_1_sheet.ron";
const PLAYER_ANIMATIONS_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/player_animation.ron";

#[derive(Resource)]
pub struct GlobalAsset {
    pub spritesheets: HashMap<String, HashMap<String, Handle<SpriteSheetConfig>>>,
    pub animations: HashMap<String, Handle<AnimationMapConfig>>,
    pub character_configs: HashMap<String, Handle<CharacterConfig>>,
    pub weapons: Handle<WeaponsConfig>,
    pub melee_weapons: Handle<MeleeWeaponsConfig>,

    // Visual effects
    pub slash_effect_spritesheet: Handle<SpriteSheetConfig>,
    pub slash_effect_animation: Handle<AnimationMapConfig>,

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
        // asset_name_ref réellement déclarés par ce jeu (`characters/*.ron` via le
        // registre) : pilote quelles feuilles de sprite sont chargées ci-dessous.
        let used_refs: BTreeSet<&str> = registry
            .characters
            .values()
            .map(|entry| entry.asset_name_ref.as_str())
            .collect();

        let mut spritesheets: HashMap<String, HashMap<String, Handle<SpriteSheetConfig>>> =
            HashMap::default();
        let mut animations: HashMap<String, Handle<AnimationMapConfig>> = HashMap::default();

        if used_refs.contains("player") {
            spritesheets.insert(
                "player".to_string(),
                bmap!(
                    "body" => asset_server.load(PLAYER_SPRITESHEET_CONFIG_PATH),
                    "shirt" => asset_server.load(PLAYER_SHIRT_SPRITESHEET_CONFIG_PATH),
                    "hair" => asset_server.load(PLAYER_HAIR_SPRITESHEET_CONFIG_PATH),
                    "shadow" => asset_server.load("ZombieShooter/Sprites/Character/shadow_sheet.ron")
                ),
            );
            animations.insert(
                "player".to_string(),
                asset_server.load(PLAYER_ANIMATIONS_CONFIG_PATH),
            );

            // Armes à distance : sprites du joueur qui les porte, pas d'un personnage
            // particulier ; chargées avec "player" tant qu'il n'y a qu'un seul jeu de
            // sprites d'armes (voir la note du module sur le pipeline sprites, A5).
            spritesheets.insert(
                "shotgun".to_string(),
                bmap!("body" => asset_server.load("ZombieShooter/Sprites/Character/shotgun_sheet.ron")),
            );
            spritesheets.insert(
                "pistol".to_string(),
                bmap!("body" => asset_server.load("ZombieShooter/Sprites/Character/pistol_sheet.ron")),
            );
            spritesheets.insert(
                "machine_gun".to_string(),
                bmap!("body" => asset_server.load("ZombieShooter/Sprites/Character/machine_gun_sheet.ron")),
            );
            animations.insert(
                "shotgun".to_string(),
                asset_server.load(PLAYER_ANIMATIONS_CONFIG_PATH),
            );
            animations.insert(
                "pistol".to_string(),
                asset_server.load(PLAYER_ANIMATIONS_CONFIG_PATH),
            );
            animations.insert(
                "machine_gun".to_string(),
                asset_server.load(PLAYER_ANIMATIONS_CONFIG_PATH),
            );
        }

        if used_refs.contains("zombie_1") {
            spritesheets.insert(
                "zombie_1".to_string(),
                bmap!(
                    "body" => asset_server.load("ZombieShooter/Sprites/Zombie/zombie_sheet.ron"),
                    "shadow" => asset_server.load("ZombieShooter/Sprites/Character/shadow_sheet.ron")
                ),
            );
            animations.insert(
                "zombie_1".to_string(),
                asset_server.load("ZombieShooter/Sprites/Zombie/zombie_animation.ron"),
            );
        }

        if used_refs.contains("zombie_2") {
            spritesheets.insert(
                "zombie_2".to_string(),
                bmap!(
                    "body" => asset_server.load("ZombieShooter/Sprites/Zombie/zombie_hard_sheet.ron"),
                    "shadow" => asset_server.load("ZombieShooter/Sprites/Character/shadow_sheet.ron")
                ),
            );
            // Le zombie "hard" réutilise l'animation du zombie standard (aucun fichier
            // `zombie_hard_animation.ron` n'existe) : comportement inchangé par T1.5.
            animations.insert(
                "zombie_2".to_string(),
                asset_server.load("ZombieShooter/Sprites/Zombie/zombie_animation.ron"),
            );
        }

        if used_refs.contains("zombie_full") {
            spritesheets.insert(
                "zombie_full".to_string(),
                bmap!(
                    "body" => asset_server.load("ZombieShooter/Sprites/Zombie/zombie_full_sheet.ron"),
                    "shadow" => asset_server.load("ZombieShooter/Sprites/Character/shadow_sheet.ron")
                ),
            );
            animations.insert(
                "zombie_full".to_string(),
                asset_server.load("ZombieShooter/Sprites/Zombie/zombie_full_animation.ron"),
            );
        }

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

            // Visual effects
            slash_effect_spritesheet: asset_server
                .load("ZombieShooter/Sprites/Character/slash_sheet.ron"),
            slash_effect_animation: asset_server
                .load("ZombieShooter/Sprites/Character/slash_animation.ron"),

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

    // Check visual effects
    if !asset_server
        .load_state(&global_assets.slash_effect_spritesheet)
        .is_loaded()
    {
        return;
    }
    if !asset_server
        .load_state(&global_assets.slash_effect_animation)
        .is_loaded()
    {
        return;
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
