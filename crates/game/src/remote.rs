//! Contrôle remote du jeu (natif) : un agent ou un script pilote la partie via
//! `bevy_remote` (JSON-RPC 2.0 sur HTTP, `http://127.0.0.1:15702` par défaut).
//!
//! Activé par `ALACOD_REMOTE=1` (port : `ALACOD_REMOTE_PORT`). La partie démarre en
//! pause : la simulation n'avance que sur `alacod/step`, ou en temps réel après
//! `alacod/resume`. Le rendu continue en pause (captures d'écran possibles).
//!
//! Méthodes :
//! - `alacod/state` : état compact (frame, vague, joueurs, ennemis, fenêtres, portes) ;
//! - `alacod/input` `{handle?, buttons: [..], pan: [x, y]}` : input maintenu d'un joueur
//!   jusqu'au prochain appel (`buttons` : noms de `replay::Button`) ;
//! - `alacod/step` `{frames}` : avance de `frames` frames de simulation puis pause ;
//! - `alacod/pause`, `alacod/resume` ;
//! - `alacod/screenshot` `{path}` : capture de la fenêtre (écrite à l'update suivante) ;
//! - `alacod/save_recording` `{path}` : écrit la session jouée en scénario rejouable.
//!
//! Mode éditeur (`ALACOD_EDITOR=1`, K5-1, [`crate::editor_mode`]) : `ALACOD_REMOTE` est implicite, la
//! partie **démarre en marche**, le clavier reste la source d'input et `alacod/input` prend la main
//! pour son handle jusqu'à un `alacod/input` vide (`buttons: []`, `pan` absent). Méthodes d'édition,
//! toutes refusées en ligne ou à `check_distance > 0` (erreur qui nomme la cause) :
//! - `alacod/restart` `{kind?: "Restart"|"ToLobby"|"EnterRun"|"ToHub", seed?}` : pose un `RunRequest`
//!   (`seed` : graine de carte, hors `ToLobby`) ;
//! - `alacod/set_modifier` `{handle?, net_id?, stat, op, value, source?, until_frames?}` et
//!   `alacod/clear_modifiers` `{handle?, net_id?, source?}` : un `Modifier` vivant sur les stats ;
//! - `alacod/reload_content` : relit le registre (voir `content_hot_reload`).

use std::time::Duration;

use bevy::{
    prelude::*,
    remote::{error_codes, http::RemoteHttpPlugin, BrpError, BrpResult, RemotePlugin},
    render::view::screenshot::{save_to_disk, Screenshot},
    time::{TimeSystems, TimeUpdateStrategy},
};
use bevy_fixed::fixed_math::{self, FixedTransform3D};
use map::{
    game::entity::map::{door::DoorComponent, window::WindowHealth},
    generation::config::MapGenerationConfig,
};
use serde::Deserialize;
use serde_json::{json, Value};
use utils::{frame::FrameCount, net_id::GgrsNetId};

use crate::{
    character::{
        config::CharacterConfigKey,
        enemy::{ai::MonsterState, Enemy},
        health::Health,
        player::{
            input::{InputSource, RemoteInputs},
            jjrs::PeerConfig,
            Player,
        },
    },
    content_hot_reload::{reload_registry_now, GameRoot},
    editor_mode::{hot_edit_allowed, EditorMode},
    jjrs::GggrsSessionConfiguration,
    recording::{write_recording, InputRecorder},
    replay::{box_input, Button},
    run_state::RunRequest,
    session::{OnlineState, SIM_FPS},
    waves::WaveState,
    weapons::{WeaponInventory, WeaponModesState, WeaponState},
};

/// Horloge de la simulation pilotée à distance.
#[derive(Resource, Debug)]
pub struct RemoteClock {
    pub paused: bool,
    /// Frames de simulation encore à avancer avant la pause.
    pub pending_frames: u32,
}

pub struct RemoteControlPlugin {
    pub port: u16,
}

impl RemoteControlPlugin {
    /// `Some` si `ALACOD_REMOTE=1` ou `ALACOD_EDITOR=1` (le mode éditeur implique le remote).
    pub fn from_env() -> Option<Self> {
        if !std::env::var("ALACOD_REMOTE").is_ok_and(|v| v == "1") && !EditorMode::enabled_by_env()
        {
            return None;
        }
        let port = std::env::var("ALACOD_REMOTE_PORT")
            .map(|p| p.parse().expect("ALACOD_REMOTE_PORT doit être un port"))
            .unwrap_or(bevy::remote::http::DEFAULT_PORT);
        Some(Self { port })
    }
}

impl Plugin for RemoteControlPlugin {
    fn build(&self, app: &mut App) {
        // `EditorMode` est posée avant ce plugin (`CoreAppPlugin`).
        let editor = app.world().contains_resource::<EditorMode>();
        app.add_plugins(
            RemotePlugin::default()
                .with_method_main("alacod/state", state)
                .with_method_main("alacod/input", input)
                .with_method_main("alacod/step", step)
                .with_method_main("alacod/pause", pause)
                .with_method_main("alacod/resume", resume)
                .with_method_main("alacod/screenshot", screenshot)
                .with_method_main("alacod/save_recording", save_recording)
                .with_method_main("alacod/restart", restart)
                .with_method_main("alacod/set_modifier", set_modifier)
                .with_method_main("alacod/clear_modifiers", clear_modifiers)
                .with_method_main("alacod/reload_content", reload_content),
        )
        .add_plugins(RemoteHttpPlugin::default().with_port(self.port))
        .insert_resource(RemoteClock {
            // Mode éditeur : la partie tourne ; `alacod/pause` et `alacod/step` marchent toujours.
            paused: !editor,
            pending_frames: 0,
        })
        .init_resource::<RemoteInputs>()
        .add_systems(First, drive_clock.before(TimeSystems));
        if !editor {
            // Hors mode éditeur : les inputs sont ceux du remote seul (comportement d'origine).
            app.insert_resource(InputSource::Remote);
        }

        info!(
            "contrôle remote sur http://127.0.0.1:{}{}",
            self.port,
            if editor { " (mode éditeur)" } else { "" }
        );
    }
}

/// En pause, le temps n'avance pas et GGRS ne simule rien. Chaque frame demandée par
/// `alacod/step` avance le temps d'exactement une frame de simulation.
fn drive_clock(
    mut clock: ResMut<RemoteClock>,
    mut strategy: ResMut<TimeUpdateStrategy>,
    session: Option<Res<bevy_ggrs::Session<PeerConfig>>>,
) {
    *strategy = if !clock.paused {
        TimeUpdateStrategy::Automatic
    } else if clock.pending_frames > 0 && session.is_some() {
        clock.pending_frames -= 1;
        TimeUpdateStrategy::ManualDuration(Duration::from_nanos(1_000_000_000 / SIM_FPS))
    } else {
        TimeUpdateStrategy::ManualDuration(Duration::ZERO)
    };
}

fn parse<T: for<'de> Deserialize<'de>>(params: Option<Value>) -> Result<T, BrpError> {
    serde_json::from_value(params.unwrap_or(Value::Null)).map_err(|err| BrpError {
        code: error_codes::INVALID_PARAMS,
        message: err.to_string(),
        data: None,
    })
}

fn internal(message: impl ToString) -> BrpError {
    BrpError {
        code: error_codes::INTERNAL_ERROR,
        message: message.to_string(),
        data: None,
    }
}

fn round(value: fixed_math::Fixed) -> f32 {
    (fixed_math::to_f32(value) * 10.0).round() / 10.0
}

fn state(In(_): In<Option<Value>>, world: &mut World) -> BrpResult {
    let frame = world.resource::<FrameCount>().frame;
    let clock = world.resource::<RemoteClock>();
    let clock = json!({ "paused": clock.paused, "pending_frames": clock.pending_frames });

    let wave = world.get_resource::<WaveState>().map(|w| {
        json!({
            "number": w.current_wave,
            "phase": format!("{:?}", w.phase),
            "to_spawn": w.enemies_to_spawn,
            "killed": w.total_enemies_killed,
        })
    });

    let mut weapons = world.query::<(&WeaponState, &WeaponModesState)>();
    let mut players_query = world.query::<(
        &Player,
        &GgrsNetId,
        &FixedTransform3D,
        &Health,
        Option<&WeaponInventory>,
        Option<&combat::inventory::AmmoReserves>,
    )>();
    let mut players = Vec::new();
    for (player, id, transform, health, inventory, ammo_reserves) in players_query.iter(world) {
        let weapon = inventory.and_then(|inventory| {
            let (entity, weapon) = inventory.weapons.get(inventory.active_weapon_index)?;
            let (state, modes) = weapons.get(world, *entity).ok()?;
            let mode = modes.modes.get(&state.active_mode);
            // Réserve du type de munition de l'arme active (T2.2) : remplace l'ancien
            // `mag_quantity` (chargeurs de réserve propres à l'arme) par la réserve partagée
            // (unités de munition, `combat::inventory::AmmoReserves`).
            let reserve = ammo_reserves.map(|r| r.get(&weapon.config.ammo_type));
            Some(json!({
                "name": weapon.config.name,
                "mode": state.active_mode,
                "ammo": mode.map(|m| m.mag_ammo),
                "reserve": reserve,
                "reloading": inventory.reloading_ending_frame.is_some(),
            }))
        });
        players.push(json!({
            "handle": player.handle,
            "id": id.0,
            "x": round(transform.translation.x),
            "y": round(transform.translation.y),
            "health": round(health.current),
            "max_health": round(health.max),
            "weapon": weapon,
        }));
    }
    players.sort_by_key(|p| p["handle"].as_u64());

    let mut enemies_query = world.query_filtered::<(
        &GgrsNetId,
        &FixedTransform3D,
        &Health,
        Option<&MonsterState>,
        Option<&CharacterConfigKey>,
    ), With<Enemy>>();
    let mut enemies: Vec<Value> = enemies_query
        .iter(world)
        .map(|(id, transform, health, monster, config)| {
            json!({
                "id": id.0,
                "config": config.map(|c| c.0.as_str()),
                "x": round(transform.translation.x),
                "y": round(transform.translation.y),
                "health": round(health.current),
                "state": monster.map(|m| format!("{m:?}")),
            })
        })
        .collect();
    enemies.sort_by_key(|e| e["id"].as_u64());

    let mut windows_query = world.query::<(&GgrsNetId, &FixedTransform3D, &WindowHealth)>();
    let mut windows: Vec<Value> = windows_query
        .iter(world)
        .map(|(id, transform, health)| {
            json!({
                "id": id.0,
                "x": round(transform.translation.x),
                "y": round(transform.translation.y),
                "health": health.current,
                "max_health": health.max,
            })
        })
        .collect();
    windows.sort_by_key(|w| w["id"].as_u64());

    // Porte ouverte = sans collider (une porte non interactive reste fermée)
    let mut doors_query = world.query_filtered::<(
        &GgrsNetId,
        &FixedTransform3D,
        Has<crate::collider::Collider>,
    ), With<DoorComponent>>();
    let mut doors: Vec<Value> = doors_query
        .iter(world)
        .map(|(id, transform, closed)| {
            json!({
                "id": id.0,
                "x": round(transform.translation.x),
                "y": round(transform.translation.y),
                "open": !closed,
            })
        })
        .collect();
    doors.sort_by_key(|d| d["id"].as_u64());

    let editor = world.contains_resource::<EditorMode>();
    let online = matches!(
        world.get_resource::<OnlineState>(),
        Some(OnlineState::Online)
    );
    let check_distance = world
        .get_resource::<GggrsSessionConfiguration>()
        .map(|config| config.connection.check_distance);
    let game = world
        .get_resource::<content::manifest::GameManifest>()
        .map(|manifest| manifest.name.clone())
        .or_else(|| {
            world
                .get_resource::<GameRoot>()
                .and_then(|root| root.0.file_name())
                .map(|name| name.to_string_lossy().into_owned())
        });
    let map = world
        .get_resource::<MapGenerationConfig>()
        .map(|config| config.map_path.clone());
    let seed = world
        .get_resource::<bevy_fixed::rng::RunSeed>()
        .map(|seed| seed.0);
    let run = world
        .get_resource::<run::Run>()
        .map(|run| json!({ "step": format!("{:?}", run.step) }));

    Ok(json!({
        "editor": editor,
        "online": online,
        "check_distance": check_distance,
        "game": game,
        "map": map,
        "seed": seed,
        "run": run,
        "frame": frame,
        "clock": clock,
        "wave": wave,
        "players": players,
        "enemies": enemies,
        "windows": windows,
        "doors": doors,
    }))
}

#[derive(Deserialize)]
struct InputParams {
    #[serde(default)]
    handle: usize,
    #[serde(default)]
    buttons: Vec<Button>,
    #[serde(default)]
    pan: Option<(i16, i16)>,
}

fn input(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let params: InputParams = parse(params)?;
    // Mode éditeur : un input vide (aucun bouton, pas de visée) rend la main au clavier.
    if world.contains_resource::<EditorMode>() && params.buttons.is_empty() && params.pan.is_none()
    {
        world
            .resource_mut::<RemoteInputs>()
            .held
            .remove(&params.handle);
        return Ok(json!({ "handle": params.handle, "input": "released" }));
    }
    let input = box_input(&params.buttons, params.pan.unwrap_or_default());
    world
        .resource_mut::<RemoteInputs>()
        .held
        .insert(params.handle, input);
    Ok(json!({ "handle": params.handle, "input": format!("{input:?}") }))
}

#[derive(Deserialize)]
struct StepParams {
    frames: u32,
}

fn step(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let params: StepParams = parse(params)?;
    let frame = world.resource::<FrameCount>().frame;
    let mut clock = world.resource_mut::<RemoteClock>();
    clock.paused = true;
    clock.pending_frames += params.frames;
    Ok(json!({ "target_frame": frame + clock.pending_frames }))
}

fn pause(In(_): In<Option<Value>>, world: &mut World) -> BrpResult {
    let mut clock = world.resource_mut::<RemoteClock>();
    clock.paused = true;
    clock.pending_frames = 0;
    Ok(json!({ "frame": world.resource::<FrameCount>().frame }))
}

fn resume(In(_): In<Option<Value>>, world: &mut World) -> BrpResult {
    world.resource_mut::<RemoteClock>().paused = false;
    Ok(json!({ "frame": world.resource::<FrameCount>().frame }))
}

#[derive(Deserialize)]
struct PathParams {
    path: String,
}

fn screenshot(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let params: PathParams = parse(params)?;
    world
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(params.path.clone()));
    Ok(json!({ "path": params.path }))
}

fn save_recording(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let params: PathParams = parse(params)?;
    let recorder = world.resource::<InputRecorder>();
    write_recording(
        recorder,
        world.get_resource::<MapGenerationConfig>(),
        std::path::Path::new(&params.path),
    )
    .map_err(internal)?;
    Ok(json!({ "path": params.path, "frames": recorder.frame_count() }))
}

/// Garde de toutes les commandes d'édition (D1) : session locale à `check_distance == 0`.
fn guard_hot_edit(world: &World) -> Result<(), BrpError> {
    let online = matches!(
        world.get_resource::<OnlineState>(),
        Some(OnlineState::Online)
    );
    let check_distance = world
        .get_resource::<GggrsSessionConfiguration>()
        .map(|config| config.connection.check_distance)
        .ok_or_else(|| internal("session inconnue : check_distance illisible"))?;
    hot_edit_allowed(online, check_distance).map_err(|refusal| BrpError {
        code: error_codes::INVALID_REQUEST,
        message: refusal.to_string(),
        data: None,
    })
}

fn invalid_params(message: impl ToString) -> BrpError {
    BrpError {
        code: error_codes::INVALID_PARAMS,
        message: message.to_string(),
        data: None,
    }
}

#[derive(Deserialize)]
struct RestartParams {
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    seed: Option<i64>,
}

/// `alacod/restart` : pose un [`RunRequest`] (consommé par `apply_run_request_system`). `seed`
/// remplace la graine de carte (`MapGenerationConfig::seed`, relue par le `GameLoading` suivant et par
/// la graine de run) : refusée avec `ToLobby`, dont le lobby recalcule la configuration de carte.
fn restart(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let params: RestartParams = parse(params.or(Some(json!({}))))?;
    guard_hot_edit(world)?;
    let request = match params.kind.as_deref().unwrap_or("Restart") {
        "Restart" => RunRequest::Restart,
        "ToLobby" => RunRequest::ToLobby,
        "EnterRun" => RunRequest::EnterRun,
        "ToHub" => RunRequest::ToHub,
        other => {
            return Err(invalid_params(format!(
                "kind inconnu « {other} » (Restart, ToLobby, EnterRun, ToHub)"
            )))
        }
    };
    if let Some(seed) = params.seed {
        if request == RunRequest::ToLobby {
            return Err(invalid_params(
                "seed non pris en charge avec ToLobby (le lobby reconstruit la carte depuis le manifeste ; utiliser --seed ou Restart)",
            ));
        }
        let seed = i32::try_from(seed)
            .map_err(|_| invalid_params("seed hors de l'intervalle i32 de la graine de carte"))?;
        let mut config = world
            .get_resource_mut::<MapGenerationConfig>()
            .ok_or_else(|| internal("MapGenerationConfig absente : aucune carte configurée"))?;
        config.seed = seed;
    }
    world.insert_resource(request);
    Ok(json!({ "request": format!("{request:?}"), "seed": params.seed }))
}

#[derive(Deserialize)]
struct TargetParams {
    #[serde(default)]
    handle: Option<usize>,
    #[serde(default)]
    net_id: Option<u64>,
}

/// Entité visée par un `handle` de joueur (défaut 0) ou un `GgrsNetId`.
fn target_entity(world: &mut World, target: &TargetParams) -> Result<Entity, BrpError> {
    if let Some(net_id) = target.net_id {
        let mut query = world.query::<(&GgrsNetId, Entity)>();
        return query
            .iter(world)
            .find(|(id, _)| id.0 as u64 == net_id)
            .map(|(_, entity)| entity)
            .ok_or_else(|| invalid_params(format!("aucune entité de net_id {net_id}")));
    }
    let handle = target.handle.unwrap_or(0);
    let mut query = world.query::<(&Player, Entity)>();
    query
        .iter(world)
        .find(|(player, _)| player.handle == handle)
        .map(|(_, entity)| entity)
        .ok_or_else(|| invalid_params(format!("aucun joueur de handle {handle}")))
}

/// Source par défaut des modificateurs posés depuis l'éditeur.
const EDITOR_SOURCE: &str = "editor";

/// `StatId` depuis son nom (`MoveSpeed`, `Damage`…) ; `custom:<nom>` pour une stat `Custom`.
pub fn parse_stat(name: &str) -> Result<sim_core::stats::StatId, String> {
    if let Some(custom) = name.strip_prefix("custom:") {
        return Ok(sim_core::stats::StatId::Custom(custom.to_string()));
    }
    serde_json::from_value(Value::String(name.to_string()))
        .map_err(|_| format!("stat inconnue « {name} » (ex. MoveSpeed, MaxHealth, Damage, FireRate, ReloadSpeed ; custom:<nom>)"))
}

#[derive(Deserialize)]
pub struct SetModifierParams {
    #[serde(default)]
    pub handle: Option<usize>,
    #[serde(default)]
    pub net_id: Option<u64>,
    pub stat: String,
    pub op: sim_core::modifier::ModifierOp,
    /// `Fixed` en chaîne (`"0.9"`), comme dans les RON.
    pub value: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub until_frames: Option<u32>,
}

/// Écrit un `Modifier` vivant dans `Modifiers` de la cible (sans garde : voir [`set_modifier`]).
/// Renvoie le nombre de modificateurs de la cible.
pub fn apply_set_modifier(
    world: &mut World,
    params: &SetModifierParams,
) -> Result<usize, BrpError> {
    let stat = parse_stat(&params.stat).map_err(invalid_params)?;
    let value: fixed_math::Fixed = params.value.parse().map_err(|e| {
        invalid_params(format!(
            "value « {} » n'est pas un Fixed : {e}",
            params.value
        ))
    })?;
    let entity = target_entity(
        world,
        &TargetParams {
            handle: params.handle,
            net_id: params.net_id,
        },
    )?;
    let frame = world.resource::<FrameCount>().frame;
    let mut modifiers = world
        .get_mut::<sim_core::modifier::Modifiers>(entity)
        .ok_or_else(|| invalid_params("la cible n'a pas de Modifiers"))?;
    modifiers.push_from(
        sim_core::modifier::ModifierSource::Named(
            params
                .source
                .clone()
                .unwrap_or_else(|| EDITOR_SOURCE.to_string()),
        ),
        stat,
        params.op,
        value,
        params.until_frames.map(|n| frame.saturating_add(n)),
    );
    Ok(modifiers.0.len())
}

fn set_modifier(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let params: SetModifierParams = parse(params)?;
    guard_hot_edit(world)?;
    let count = apply_set_modifier(world, &params)?;
    Ok(json!({
        "stat": params.stat,
        "op": format!("{:?}", params.op),
        "value": params.value,
        "source": params.source.as_deref().unwrap_or(EDITOR_SOURCE),
        "modifiers": count,
    }))
}

#[derive(Deserialize)]
struct ClearModifiersParams {
    #[serde(default)]
    handle: Option<usize>,
    #[serde(default)]
    net_id: Option<u64>,
    #[serde(default)]
    source: Option<String>,
}

/// `alacod/clear_modifiers` : retire les modificateurs de la source (défaut `editor`).
fn clear_modifiers(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let params: ClearModifiersParams = parse(params.or(Some(json!({}))))?;
    guard_hot_edit(world)?;
    let entity = target_entity(
        world,
        &TargetParams {
            handle: params.handle,
            net_id: params.net_id,
        },
    )?;
    let source = sim_core::modifier::ModifierSource::Named(
        params.source.unwrap_or_else(|| EDITOR_SOURCE.to_string()),
    );
    let mut modifiers = world
        .get_mut::<sim_core::modifier::Modifiers>(entity)
        .ok_or_else(|| invalid_params("la cible n'a pas de Modifiers"))?;
    let before = modifiers.0.len();
    modifiers.remove_by_source(&source);
    Ok(json!({ "removed": before - modifiers.0.len(), "modifiers": modifiers.0.len() }))
}

/// `alacod/reload_content` : relit le registre (appliqué au `restart` suivant). Renvoie les
/// compteurs du registre lu et les erreurs de lint (non vide : registre non remplacé).
fn reload_content(In(_): In<Option<Value>>, world: &mut World) -> BrpResult {
    guard_hot_edit(world)?;
    let report = reload_registry_now(world).map_err(internal)?;
    Ok(json!({
        "reloaded": report.reloaded,
        "characters": report.characters,
        "weapons": report.weapons,
        "melee": report.melee,
        "waves": report.waves,
        "errors": report.errors,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::stats::StatId;

    #[test]
    fn parse_stat_reads_names_and_custom() {
        assert_eq!(parse_stat("MoveSpeed"), Ok(StatId::MoveSpeed));
        assert_eq!(
            parse_stat("custom:xx"),
            Ok(StatId::Custom("xx".to_string()))
        );
        let err = parse_stat("Nope").unwrap_err();
        assert!(err.contains("Nope"), "{err}");
    }
}
