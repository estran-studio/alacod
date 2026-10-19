//! États et constantes de session : lus par la simulation, les jeux, `scenario` et `map_ldtk`.
//!
//! Aucune dépendance de présentation : ce fichier est de la simulation (`game_sim::session`).
//! `core.rs` garde les plugins (`CoreSetupPlugin`, `PresentationPlugin`), `GameInfo` et
//! `debug_ui_enabled`.

use bevy::prelude::*;
use bevy_ggrs::GgrsSchedule;
use serde::{Deserialize, Serialize};

// Configuration that is static and bundle with the game
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct CoreSetupConfig {
    pub app_name: String,
    /// Sans fenêtre ni GPU, une frame de simulation par update (voir [`is_headless`]).
    pub headless: bool,
    /// Dossier des assets ; `None` pour le dossier par défaut de bevy.
    pub asset_root: Option<String>,
}

impl CoreSetupConfig {
    /// Config d'un exécutable : le mode headless vient de `ALACOD_HEADLESS`.
    pub fn from_env(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            headless: is_headless(),
            asset_root: None,
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Loading, // Initial loading step for all the required global asset to be resolved
    LobbyLocal,   // Create a local lobby for lan UDP or SyncTest Session
    LobbyOnline,  // Create an online lobby with matchbox
    GameLoading, // After the lobby as agree on the game parameters all required asset are loaded before the game can start
    GameStarting, // To launch the session after the game is loaded
    InGame,      // When the game is played with the active ggrs session from local or online
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum OnlineState {
    #[default]
    Unset, // No ggrs system enable to start a game
    Online,  // For ggrs p2p system to be enable
    Offline, // For ggrs synctest/lan system to be enabe
}

/// Nombre de joueurs de la session pour l'équilibrage (`players` des expressions de
/// `balance::resolve_balance`, `clock::DifficultyConfig`) : `max_player` de la session ggrs en ligne, sinon
/// `PlayersCount` (partie locale ou scénario). Ressource de simulation **écrite par l'application**
/// (`balance_assets::publish_session_players`, `OnEnter(GameLoading)`, avant `resolve_balance_system` et
/// `resolve_clocks_system`) : la simulation ne lit plus ni `OnlineState` ni `GggrsSessionConfiguration`
/// (D69 étape 2, E3c). Hors rollback ; sans valeur par défaut, une résolution lancée sans elle échoue
/// bruyamment au lieu de résoudre pour zéro joueur.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionPlayers(pub u32);

/// Exécuteur monothread pour les schedules du rollback (`ReadInputs`, `AdvanceWorld`,
/// `GgrsSchedule`, `SaveWorld`, `LoadWorld`), dans le jeu comme dans les scénarios.
///
/// La simulation compte des centaines de petits systèmes par frame : l'exécuteur multithread
/// de Bevy y coûte plus (réveils de threads, ≈ 1 500 `futex` par frame) qu'il ne rapporte.
/// Mesuré le 2026-10-15 sur le smoke p2p de `zombies` à deux clients (97 entités), profil
/// `utils::snapshot_profile` : une frame de simulation à 6,8 ms en moyenne (23,6 ms au pire)
/// en multithread, contre 0,15 ms sur la même charge en monothread dans le runner de
/// scénarios. En ligne, un rollback de 12 frames coûte donc 80 ms au lieu de 2. L'ordre
/// d'exécution ne change aucun résultat : `GgrsSchedule` refuse toute ambiguïté d'ordre
/// (`ambiguity_detection: Error`) et les traces de référence le prouvent.
/// `ALACOD_EXECUTOR=multi` rétablit l'exécuteur par défaut (comparaison).
pub fn single_threaded_rollback_schedules(app: &mut App) {
    if std::env::var("ALACOD_EXECUTOR").is_ok_and(|v| v == "multi") {
        return;
    }
    let mut schedules = app
        .world_mut()
        .resource_mut::<bevy::ecs::schedule::Schedules>();
    use bevy::ecs::schedule::ScheduleLabel as _;
    let labels = [
        bevy_ggrs::ReadInputs.intern(),
        bevy_ggrs::AdvanceWorld.intern(),
        GgrsSchedule.intern(),
        bevy_ggrs::SaveWorld.intern(),
        bevy_ggrs::LoadWorld.intern(),
    ];
    for label in labels {
        if let Some(schedule) = schedules.get_mut(label) {
            schedule.set_executor(bevy::ecs::schedule::SingleThreadedExecutor::new());
        }
    }
}

/// Fréquence de simulation GGRS (valeur par défaut de `RollbackFrameRate`).
pub const SIM_FPS: u64 = 60;

/// Mode headless (`ALACOD_HEADLESS=1`) : pas de fenêtre ni de GPU, et le temps
/// avance d'une frame de simulation par update. Pour les tests et les replays.
pub fn is_headless() -> bool {
    std::env::var("ALACOD_HEADLESS").is_ok_and(|v| v == "1")
}
