//! Pont entre le registre de contenu (`content::registry::Registry`, T1.5) et les
//! `Handle<...>` Bevy : ressource **d'application** (D69 étape 2, E3d), jamais lue par la simulation.
//!
//! Deux familles de handles, toutes deux tenues par cette ressource pour que les assets restent chargés :
//! - **présentation** : feuilles de sprite, configurations d'animation et effet de coup de mêlée
//!   (`spritesheets`, `animations`, `slash_effect_*`), lus par `character::visuals`, `ui::weapon_visuals` et
//!   `world_sprites` ;
//! - **sources de synchro** : les enveloppes `Asset` des configurations (personnages, armes, vagues, économie,
//!   perks, power-ups). La simulation ne les voit pas : elle lit les ressources de donnée que l'application en tire
//!   (`CharacterConfigs`, `WeaponRegistry`, `ResolvedBalance`, `PowerUpsTable`). Leurs lecteurs sont les systèmes de
//!   synchro et de résolution (`character::config_asset`, `weapon_assets`, `balance_assets`, `powerups_assets`).
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
    character::{config::CharacterConfigs, config_asset::CharacterConfigAsset},
    economy_assets::{EconomyConfigAsset, PerksConfigAsset},
    powerups_assets::PowerUpsConfigAsset,
    session::{AppState, OnlineState},
    waves::assets::WaveConfigAsset,
    weapon_assets::{MeleeWeaponsConfig, WeaponsConfig},
    weapons::WeaponRegistry,
};

/// D3 : id de la feuille de l'effet de coup de mêlée (`ui::weapon_visuals::
/// spawn_slash_effects`) dans la table `SpriteSheet`, calque `body`. Absente : pas d'effet
/// affiché.
pub const SLASH_EFFECT_SPRITE_ID: &str = "slash";

#[derive(Resource)]
pub struct GlobalAsset {
    // Présentation.
    pub spritesheets: HashMap<String, HashMap<String, Handle<SpriteSheetConfig>>>,
    pub animations: HashMap<String, Handle<AnimationMapConfig>>,
    /// Effet de coup de mêlée (D3 : entrée [`SLASH_EFFECT_SPRITE_ID`] de la table
    /// `SpriteSheet`, `None` si le jeu ne la déclare pas).
    pub slash_effect_spritesheet: Option<Handle<SpriteSheetConfig>>,
    pub slash_effect_animation: Option<Handle<AnimationMapConfig>>,

    // Sources de synchro (enveloppes, application).
    /// Source de la synchro de `CharacterConfigs` (`character::config_asset::sync_character_configs`).
    pub character_configs: HashMap<String, Handle<CharacterConfigAsset>>,
    /// Source de la synchro de `WeaponRegistry` (`weapon_assets::sync_weapon_registry`).
    pub weapons: Handle<WeaponsConfig>,
    pub melee_weapons: Handle<MeleeWeaponsConfig>,
    /// Vagues : `Some` seulement si le jeu déclare un dossier de contenu `Wave`. Source de
    /// `balance_assets::resolve_balance_system` (`ResolvedBalance::waves_enabled`).
    pub wave_config: Option<Handle<WaveConfigAsset>>,
    /// Économie de run (T2.3) : `Some` seulement si le jeu déclare un dossier de contenu `Economy`/`Perk`
    /// (comme `wave_config` pour `Wave`). Source de `resolve_balance_system`, qui résout `EconomyConfig::default()`
    /// et aucun perk quand elle est absente.
    pub economy_config: Option<Handle<EconomyConfigAsset>>,
    pub perks_config: Option<Handle<PerksConfigAsset>>,
    /// Power-ups (T2.5) : `Some` seulement si le jeu déclare un dossier de contenu `PowerUp`. Source de
    /// `powerups_assets::resolve_powerups_table_system` : absente, `PowerUpsTable` reste vide et aucun
    /// power-up ne tombe ni ne peut être ramassé.
    pub powerups_config: Option<Handle<PowerUpsConfigAsset>>,
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

/// Un registre est publié quand l'asset qui le nourrit est lisible et, s'il n'est pas vide, que le registre
/// l'est aussi (`sync_weapon_registry` ne copie jamais une table vide). `asset_is_empty` : `None` tant que
/// l'asset n'est pas lisible, sinon « sa table est vide ».
fn table_published(asset_is_empty: Option<bool>, registry_is_empty: bool) -> bool {
    match asset_is_empty {
        None => false,
        Some(true) => true,
        Some(false) => !registry_is_empty,
    }
}

/// `CharacterConfigs` contient chaque personnage déclaré et `WeaponRegistry` ses deux tables.
fn registries_published(
    global: &GlobalAsset,
    characters: &CharacterConfigs,
    weapon_registry: &WeaponRegistry,
    weapons_asset_is_empty: Option<bool>,
    melee_asset_is_empty: Option<bool>,
) -> bool {
    global
        .character_configs
        .keys()
        .all(|key| characters.0.contains_key(key))
        && table_published(weapons_asset_is_empty, weapon_registry.weapons.is_empty())
        && table_published(melee_asset_is_empty, weapon_registry.melee.is_empty())
}

pub fn loading_asset_system(
    mut app_state: ResMut<NextState<AppState>>,
    online: Res<OnlineState>,
    global_assets: Res<GlobalAsset>,
    asset_server: Res<AssetServer>,
    characters: Res<CharacterConfigs>,
    weapon_registry: Res<WeaponRegistry>,
    weapon_tables: Res<Assets<WeaponsConfig>>,
    melee_tables: Res<Assets<MeleeWeaponsConfig>>,
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

    // E3d : la simulation lit les registres (`CharacterConfigs`, `WeaponRegistry`), jamais les assets. Leurs
    // systèmes de synchro tournent en `PreUpdate` : on ne quitte `Loading` qu'une fois la publication constatée,
    // pour que la première partie ne démarre jamais sur un registre vide (les replis des lecteurs le masqueraient).
    if !registries_published(
        &global_assets,
        &characters,
        &weapon_registry,
        weapon_tables
            .get(&global_assets.weapons)
            .map(|t| t.0.is_empty()),
        melee_tables
            .get(&global_assets.melee_weapons)
            .map(|t| t.0.is_empty()),
    ) {
        return;
    }

    if matches!(*online, OnlineState::Online) {
        app_state.set(AppState::LobbyOnline);
    } else {
        app_state.set(AppState::LobbyLocal);
    }
    info!("loading of asset is done , now entering lobby");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::config::CharacterConfig;

    /// Le garde de sortie de `Loading` ne bloque jamais sur une table légitimement vide (la synchro ne copie
    /// pas une table vide) mais attend une table lisible et pas encore copiée.
    #[test]
    fn a_table_is_published_once_readable_and_copied() {
        assert!(!table_published(None, true), "asset pas encore lisible");
        assert!(
            !table_published(Some(false), true),
            "lisible, pas encore copiée"
        );
        assert!(table_published(Some(false), false), "copiée");
        assert!(
            table_published(Some(true), true),
            "fichier vide : rien à copier"
        );
    }

    /// Un vrai personnage (`CharacterConfig` n'a pas de valeur par défaut) : le joueur de `zombies`.
    fn player_config() -> CharacterConfig {
        let file = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../games/zombies/assets/ZombieShooter/Sprites/Character/player_config.ron");
        let text = std::fs::read_to_string(&file).unwrap();
        // Comme `RonAssetPlugin` : les fichiers de personnages s'écrivent en `implicit_some`.
        ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(&text)
            .unwrap_or_else(|e| panic!("{} : {e}", file.display()))
    }

    fn global_with_character(key: &str) -> GlobalAsset {
        GlobalAsset {
            spritesheets: Default::default(),
            animations: Default::default(),
            character_configs: [(key.to_string(), Handle::default())].into_iter().collect(),
            weapons: Default::default(),
            melee_weapons: Default::default(),
            slash_effect_spritesheet: None,
            slash_effect_animation: None,
            wave_config: None,
            economy_config: None,
            perks_config: None,
            powerups_config: None,
        }
    }

    /// `Loading` ne se quitte qu'avec chaque personnage déclaré dans `CharacterConfigs`.
    #[test]
    fn loading_waits_for_every_declared_character() {
        let global = global_with_character("player");
        let registry = WeaponRegistry::default();
        let mut characters = CharacterConfigs::default();
        assert!(!registries_published(
            &global,
            &characters,
            &registry,
            Some(true),
            Some(true)
        ));
        characters.0.insert("other".to_string(), player_config());
        assert!(
            !registries_published(&global, &characters, &registry, Some(true), Some(true)),
            "un autre personnage ne publie pas celui qui est déclaré"
        );
        characters.0.insert("player".to_string(), player_config());
        assert!(registries_published(
            &global,
            &characters,
            &registry,
            Some(true),
            Some(true)
        ));
    }
}
