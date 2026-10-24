//! Bac à sable (`--sandbox [profil]`, `docs/taches/plan-bac-a-sable.md` D2/D3/D4, `docs/conventions.md` §87) :
//! l'arène du jeu, sans vagues ni étages, avec un profil de départ, en mode éditeur.
//!
//! Posé par [`super::local::LdtkLocalGamePlugin`] **quand** `game::sandbox::SandboxRequest` existe (posée par
//! `BaseArgsPlugin` si la ligne de commande a `--sandbox`) : aucune ligne dans le `main.rs` d'un jeu. Le mode
//! éditeur (`remote`, `check_distance` 0, clavier) est allumé par `CoreSetupPlugin` à partir de la même
//! ressource.
//!
//! # Ce que fait le plugin
//!
//! `Startup` ([`launch_sandbox`]) : valide le profil et l'arène (`game::sandbox::resolve` ; erreur claire et
//! sortie 1 sinon), pose l'arène dans `LdtkGameMap` (et la graine `ARENA_SEED`, ou celle de `--seed`), force `RunMode::Sandbox` (`entry.mode` du manifeste),
//! retire un `HubSession` posé par le `main` du jeu et coupe `WaveModeEnabled`, puis pose
//! `SandboxSession`.
//!
//! `OnEnter(GameLoading)` ([`install_session_profile`], à **chaque** entrée, donc après un `restart`) : lit le
//! profil actif de `SandboxSession` dans le registre (rechargé à chaud : un RON modifié se prend au `restart`),
//! le pose par [`Profile::install`] (placements, réglages des joueurs, `ProfileApplied` remis à zéro) et pose
//! `ShopOverrides` (`shop.shells_drop_override` du profil). [`disable_waves`] remet `ResolvedBalance::waves_enabled`
//! à faux **après** `resolve_balance_system`, qui le recalcule à chaque entrée.
//!
//! La mise en place elle-même (armes, objets, argent, modificateurs, placements) est celle du runner de
//! scénarios, [`super::profile`] : un seul code.
//!
//! # `restart`
//!
//! `RunRequest::Restart` (`alacod/restart`) retourne en `GameLoading` : `FrameCount` repart de 0, les joueurs,
//! ennemis et objets de la partie disparaissent, le joueur est recréé (avec le `character` du profil) et le
//! profil se repose (réglages des joueurs, placements `at_frame`). Ce que le joueur a reçu par `alacod/give`
//! ou `alacod/set_modifier`, et ce qu'il a fait apparaître par `alacod/spawn_*`, est perdu.
//!
//! Rien ici ne tourne dans `GgrsSchedule`.

use bevy::prelude::*;
use content::{manifest::GameManifest, registry::Registry, EntryMode};
use game::{
    args::LaunchSeed,
    balance::ResolvedBalance,
    balance_assets::resolve_balance_system,
    sandbox::{resolve, SandboxRequest, SandboxSession},
    session::{AppState, OnlineState},
    shop::ShopOverrides,
    waves::WaveModeEnabled,
};

use super::{
    local::{configure_map, LdtkGameMap},
    profile::{Profile, ProfilePlugin},
};

pub struct SandboxPlugin;

impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        // Profil vide au départ : `install_session_profile` pose le vrai à l'entrée de `GameLoading`.
        app.add_plugins(ProfilePlugin(Profile::default()))
            .add_systems(Startup, launch_sandbox)
            // L'arène du profil actif, avant que le lobby ne fabrique la configuration de carte.
            .add_systems(
                OnEnter(AppState::LobbyLocal),
                apply_arena.before(configure_map),
            )
            .add_systems(
                OnEnter(AppState::GameLoading),
                (
                    install_session_profile,
                    disable_waves.after(resolve_balance_system),
                ),
            );
    }
}

/// Erreur de lancement : message sur la sortie d'erreur et code 1, comme un contenu invalide dans les
/// `main.rs` des jeux.
fn refuse(message: &str) -> ! {
    eprintln!("{message}");
    std::process::exit(1);
}

#[allow(clippy::too_many_arguments)]
fn launch_sandbox(
    mut commands: Commands,
    request: Res<SandboxRequest>,
    registry: Res<Registry>,
    mut manifest: ResMut<GameManifest>,
    mut map: ResMut<LdtkGameMap>,
    mut waves: ResMut<WaveModeEnabled>,
    online: Res<OnlineState>,
    launch: Option<Res<LaunchSeed>>,
) {
    if *online == OnlineState::Online {
        refuse("bac à sable : session locale seulement (retirer --matchbox / --allumette)");
    }
    let arena = match resolve(&registry, &manifest, &request.0) {
        Ok(resolved) => resolved.arena.to_string(),
        Err(message) => refuse(&message),
    };
    info!(
        "bac à sable : jeu « {} », profil « {} », arène {arena}",
        manifest.name, request.0
    );
    map.map_path = arena;
    // Une arène assemblée par graine doit être la même à chaque lancement (les placements des profils en
    // dépendent) : `--seed` si donné, sinon `ARENA_SEED`. `LaunchSeed` empêche un jeu de tirer la sienne.
    let seed = launch
        .and_then(|launch| launch.0)
        .unwrap_or(game::sandbox::ARENA_SEED);
    map.seed = seed;
    commands.insert_resource(LaunchSeed(Some(seed)));
    // Aucune condition de victoire, pas de niveaux ; pas de hub (la porte du hub lancerait une run).
    manifest.entry.mode = Some(EntryMode::Sandbox);
    waves.0 = false;
    commands.remove_resource::<game::hub::HubSession>();
    commands.insert_resource(SandboxSession {
        profile: request.0.clone(),
    });
}

/// L'arène du profil actif devient la carte de la partie (`LdtkGameMap`, relue par `configure_map`). Rejoue
/// le choix de [`launch_sandbox`] quand un `ToLobby` repasse par le lobby avec un autre profil.
fn apply_arena(
    session: Option<Res<SandboxSession>>,
    registry: Res<Registry>,
    manifest: Res<GameManifest>,
    mut map: ResMut<LdtkGameMap>,
) {
    let Some(session) = session else {
        return;
    };
    match resolve(&registry, &manifest, &session.profile) {
        Ok(resolved) => map.map_path = resolved.arena.to_string(),
        Err(message) => error!("{message}"),
    }
}

/// Pose le profil actif ; un profil devenu introuvable (registre rechargé sans lui) se dit, et la partie
/// continue sans profil plutôt que de planter.
fn install_session_profile(world: &mut World) {
    let Some(session) = world.get_resource::<SandboxSession>() else {
        return;
    };
    let id = session.profile.clone();
    let mut profile = {
        let registry = world.resource::<Registry>();
        let manifest = world.resource::<GameManifest>();
        match resolve(registry, manifest, &id) {
            Ok(resolved) => (
                Profile::from_sandbox(resolved.profile),
                resolved.profile.shop.shells_drop_override,
            ),
            Err(message) => {
                error!("{message} : partie sans profil");
                (Profile::default(), None)
            }
        }
    };
    // Un profil qui décrit plus de joueurs que la session n'est jamais « complet » : on ne garde que les
    // joueurs qui existent (`apply_player_overrides` attend `players.len()` joueurs).
    if let Some(count) = world.get_resource::<sim_core::players::PlayersCount>() {
        profile.0.players.truncate(count.0);
    }
    profile.0.install(world);
    world.insert_resource(ShopOverrides {
        shells_drop: profile.1,
    });
    info!("bac à sable : profil « {id} » posé");
}

/// `resolve_balance_system` recalcule `waves_enabled` (le jeu déclare-t-il un dossier `Wave` ?) à chaque
/// entrée de `GameLoading` : le bac à sable le remet à faux, `wave_spawning_system` ne fait alors rien.
fn disable_waves(balance: Option<ResMut<ResolvedBalance>>) {
    if let Some(mut balance) = balance {
        balance.waves_enabled = false;
    }
}
