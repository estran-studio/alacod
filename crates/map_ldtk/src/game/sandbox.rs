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
    jjrs::local::system_after_map_loaded_local,
    sandbox::{resolve, SandboxRequest, SandboxSession},
    session::{AppState, OnlineState},
    shop::ShopOverrides,
    waves::WaveModeEnabled,
};
use map::generation::config::MapGenerationConfig;

use super::{
    floors::compute_floor_plan,
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
            // `reload` remplace `GameManifest` par le disque (`content_hot_reload`) : le mode `Sandbox` se
            // rétablit dès que le manifeste change, et surtout **avant** `compute_floor_plan` à chaque
            // `GameLoading` (`restart`), son premier lecteur.
            .add_systems(Update, pin_mode.run_if(resource_changed::<GameManifest>))
            // Un rechargement du dernier `Update` de `GameLoading` (après `pin_mode`) précède
            // `OnEnter(GameStarting)`, qui passe avant `Update` : le mode de la `Run` se résout là.
            .add_systems(
                OnEnter(AppState::GameStarting),
                pin_mode.before(system_after_map_loaded_local),
            )
            .add_systems(
                OnEnter(AppState::GameLoading),
                (
                    pin_session.before(compute_floor_plan),
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
    // Rétabli par `pin_mode`/`pin_session` après un rechargement du manifeste.
    force_sandbox_mode(&mut manifest);
    waves.0 = false;
    commands.remove_resource::<game::hub::HubSession>();
    commands.insert_resource(SandboxSession {
        profile: request.0.clone(),
    });
}

/// `entry.mode: Sandbox` (aucune condition de fin, pas de niveaux), quoi que dise le disque. N'écrit que si
/// la valeur diffère : lue par `Deref`, la ressource n'est pas marquée modifiée (`&mut *manifest` le ferait à
/// chaque appel, et `pin_mode` se déclencherait lui-même à chaque frame).
fn force_sandbox_mode(manifest: &mut ResMut<GameManifest>) {
    if manifest.entry.mode != Some(EntryMode::Sandbox) {
        manifest.entry.mode = Some(EntryMode::Sandbox);
    }
}

/// Après un rechargement du manifeste (`alacod/reload_content`, rechargement automatique) : le mode
/// `Sandbox` reprend le dessus.
fn pin_mode(mut manifest: ResMut<GameManifest>) {
    force_sandbox_mode(&mut manifest);
}

/// `OnEnter(GameLoading)`, **avant** `compute_floor_plan` et `setup_generated_map` : à chaque (re)chargement
/// de partie, la session prime sur le disque. Le mode reste `Sandbox` (sinon un `entry.mode: Floors` relu par
/// `reload` ferait charger les niveaux de la run), et la carte est celle du profil **actif tel que le registre
/// le décrit maintenant** (un `map` modifié puis `reload` + `restart` est pris : `Restart` va droit en
/// `GameLoading`, sans repasser par le lobby où tourne [`apply_arena`]). La graine n'est pas touchée
/// (`--seed`, `restart --seed`). Un profil introuvable garde la carte en place (`install_session_profile` le dit).
fn pin_session(
    session: Option<Res<SandboxSession>>,
    registry: Res<Registry>,
    mut manifest: ResMut<GameManifest>,
    mut map: ResMut<LdtkGameMap>,
    config: Option<ResMut<MapGenerationConfig>>,
) {
    let Some(session) = session else {
        return;
    };
    force_sandbox_mode(&mut manifest);
    if let Ok(resolved) = resolve(&registry, &manifest, &session.profile) {
        if map.map_path != resolved.arena {
            map.map_path = resolved.arena.to_string();
        }
        if let Some(mut config) = config {
            if config.map_path != resolved.arena {
                config.map_path = resolved.arena.to_string();
            }
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use content::manifest::{EntryPoint, SandboxEntry};
    use content::registry::SandboxProfileEntry;
    use content::sandbox::SandboxProfile;

    fn manifest(mode: Option<EntryMode>, arena: &str) -> GameManifest {
        GameManifest {
            name: "demo".to_string(),
            content_folders: vec![],
            entry: EntryPoint {
                start_map: "run.ldtk".to_string(),
                default_seed: 1,
                mode,
                floors: None,
                hub: None,
                progression: None,
                clocks: None,
                difficulty: None,
                map: Default::default(),
            },
            generate_template: None,
            mechanics: vec![],
            sandbox: Some(SandboxEntry {
                arena: arena.to_string(),
            }),
        }
    }

    fn registry(map: Option<&str>) -> Registry {
        let mut registry = Registry::default();
        registry.sandbox_profiles.insert(
            "arene".to_string(),
            SandboxProfileEntry {
                id: "arene".to_string(),
                file: "sandbox/arene.ron".into(),
                profile: SandboxProfile {
                    map: map.map(str::to_string),
                    ..Default::default()
                },
            },
        );
        registry
    }

    fn world(manifest: GameManifest, registry: Registry) -> World {
        let mut world = World::new();
        world.insert_resource(manifest);
        world.insert_resource(registry);
        world.insert_resource(SandboxSession {
            profile: "arene".into(),
        });
        world.insert_resource(LdtkGameMap {
            map_path: "ancienne.ldtk".into(),
            seed: 11,
            settings: Default::default(),
        });
        world.insert_resource(MapGenerationConfig {
            map_path: "ancienne.ldtk".into(),
            seed: 7,
            ..Default::default()
        });
        world
    }

    /// Un manifeste relu du disque (`Floors`, arène du manifeste) : `pin_session` rétablit `Sandbox`, pose la
    /// carte du profil **actif** dans les deux ressources de carte, et ne touche à aucune graine.
    #[test]
    fn pin_session_retablit_le_mode_et_la_carte_du_profil_actif() {
        for (profile_map, expected) in [
            (None, "manifeste.ldtk"),
            (Some("profil.ldtk"), "profil.ldtk"),
        ] {
            let mut world = world(
                manifest(Some(EntryMode::Floors), "manifeste.ldtk"),
                registry(profile_map),
            );
            world.run_system_once(pin_session).unwrap();
            assert_eq!(
                world.resource::<GameManifest>().entry.mode,
                Some(EntryMode::Sandbox)
            );
            assert_eq!(world.resource::<LdtkGameMap>().map_path, expected);
            assert_eq!(world.resource::<MapGenerationConfig>().map_path, expected);
            assert_eq!(world.resource::<LdtkGameMap>().seed, 11);
            assert_eq!(world.resource::<MapGenerationConfig>().seed, 7);
        }
    }

    /// Hors bac à sable (pas de `SandboxSession`), ou profil introuvable : rien ne change de carte.
    #[test]
    fn pin_session_ne_touche_pas_a_la_carte_sans_profil() {
        let mut world = world(
            manifest(Some(EntryMode::Floors), "manifeste.ldtk"),
            registry(None),
        );
        world.remove_resource::<SandboxSession>();
        world.run_system_once(pin_session).unwrap();
        assert_eq!(
            world.resource::<GameManifest>().entry.mode,
            Some(EntryMode::Floors)
        );
        assert_eq!(
            world.resource::<MapGenerationConfig>().map_path,
            "ancienne.ldtk"
        );

        world.insert_resource(SandboxSession {
            profile: "absent".into(),
        });
        world.run_system_once(pin_session).unwrap();
        assert_eq!(
            world.resource::<GameManifest>().entry.mode,
            Some(EntryMode::Sandbox)
        );
        assert_eq!(world.resource::<LdtkGameMap>().map_path, "ancienne.ldtk");
    }

    #[test]
    fn pin_mode_retablit_sandbox_sans_marquer_une_ressource_deja_juste() {
        let mut world = world(manifest(None, "a.ldtk"), registry(None));
        world.run_system_once(pin_mode).unwrap();
        assert_eq!(
            world.resource::<GameManifest>().entry.mode,
            Some(EntryMode::Sandbox)
        );
        let tick = world.resource_ref::<GameManifest>().last_changed();
        world.run_system_once(pin_mode).unwrap();
        assert_eq!(world.resource_ref::<GameManifest>().last_changed(), tick);
    }
}
