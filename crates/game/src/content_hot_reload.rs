//! Rechargement à chaud du registre de contenu, hors partie (T1.5).
//!
//! Quand un fichier de contenu suivi par Bevy change sur disque (feature `native`,
//! `bevy/file_watcher` : voir `games/zombies/Cargo.toml`), et que l'app est dans un état de
//! lobby (`AppState::LobbyLocal`/`LobbyOnline`), on relit `game.ron` et ses dossiers
//! depuis le disque et on relance le lint. Le registre n'est remplacé que si le contenu
//! est valide ; sinon l'ancien registre reste en place et les erreurs sont journalisées.
//! Jamais en partie (`AppState::InGame`) : ce système ne tourne que dans les états de
//! lobby, jamais dans `GgrsSchedule` (les snapshots rollback ne se réécrivent pas à chaud).
//!
//! Idiome repris de `crates/game/src/ui/hud.rs` (`handle_hud_config_changes`) :
//! `MessageReader<AssetEvent<T>>` pour chacun des types de contenu déjà suivis comme
//! assets Bevy (`CharacterConfig`, `WeaponsConfig`, `MeleeWeaponsConfig`, `WaveConfig`).

use bevy::prelude::*;
use std::path::PathBuf;

use crate::character::config::CharacterConfig;
use crate::core::AppState;
use crate::waves::WaveConfig;
use crate::weapons::{melee::MeleeWeaponsConfig, WeaponsConfig};

/// Dossier du jeu (ex. `games/zombies`, posé par `main.rs` depuis
/// `env!("CARGO_MANIFEST_DIR")`) : sert à relire `game.ron` et ses dossiers de contenu.
/// Absente dans les tests de scénario (`crates/scenario`), qui n'ont pas besoin du
/// rechargement à chaud : le système no-op si elle n'est pas posée.
#[derive(Resource, Clone)]
pub struct GameRoot(pub PathBuf);

pub struct ContentHotReloadPlugin;

impl Plugin for ContentHotReloadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, reload_registry_on_change.run_if(in_lobby));
    }
}

fn in_lobby(state: Res<State<AppState>>) -> bool {
    matches!(state.get(), AppState::LobbyLocal | AppState::LobbyOnline)
}

fn reload_registry_on_change(
    game_root: Option<Res<GameRoot>>,
    mut characters: MessageReader<AssetEvent<CharacterConfig>>,
    mut weapons: MessageReader<AssetEvent<WeaponsConfig>>,
    mut melee: MessageReader<AssetEvent<MeleeWeaponsConfig>>,
    mut waves: MessageReader<AssetEvent<WaveConfig>>,
    mut commands: Commands,
) {
    // `|` (pas `||`) : les quatre lecteurs doivent être vidés à chaque appel, même si un
    // des premiers a déjà trouvé un changement (sinon leurs curseurs prennent du retard).
    let changed = characters.read().any(is_modified)
        | weapons.read().any(is_modified)
        | melee.read().any(is_modified)
        | waves.read().any(is_modified);

    if !changed {
        return;
    }

    let Some(game_root) = game_root else {
        return;
    };

    match content::load_and_lint(&game_root.0) {
        Ok((registry, _manifest, errors)) if errors.is_empty() => {
            info!(
                "contenu rechargé ({} personnages, {} armes, {} armes de corps à corps, {} vagues)",
                registry.characters.len(),
                registry.weapons.len(),
                registry.melee_weapons.len(),
                registry.waves.len(),
            );
            commands.insert_resource(registry);
        }
        Ok((_, _, errors)) => {
            for e in &errors {
                error!("contenu invalide, registre non rechargé : {e}");
            }
        }
        Err(e) => error!("game.ron invalide, registre non rechargé : {e}"),
    }
}

fn is_modified<T: Asset>(event: &AssetEvent<T>) -> bool {
    matches!(event, AssetEvent::Modified { .. })
}
