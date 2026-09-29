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
        enemy::{ai::MonsterState, Enemy},
        health::Health,
        player::{
            input::{InputSource, RemoteInputs},
            jjrs::PeerConfig,
            Player,
        },
    },
    core::SIM_FPS,
    recording::{write_recording, InputRecorder},
    replay::{box_input, Button},
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
    /// `Some` si `ALACOD_REMOTE=1`.
    pub fn from_env() -> Option<Self> {
        if !std::env::var("ALACOD_REMOTE").is_ok_and(|v| v == "1") {
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
        app.add_plugins(
            RemotePlugin::default()
                .with_method_main("alacod/state", state)
                .with_method_main("alacod/input", input)
                .with_method_main("alacod/step", step)
                .with_method_main("alacod/pause", pause)
                .with_method_main("alacod/resume", resume)
                .with_method_main("alacod/screenshot", screenshot)
                .with_method_main("alacod/save_recording", save_recording),
        )
        .add_plugins(RemoteHttpPlugin::default().with_port(self.port))
        .insert_resource(RemoteClock {
            paused: true,
            pending_frames: 0,
        })
        .insert_resource(InputSource::Remote)
        .init_resource::<RemoteInputs>()
        .add_systems(First, drive_clock.before(TimeSystems));

        info!("contrôle remote sur http://127.0.0.1:{}", self.port);
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
    )>();
    let mut players = Vec::new();
    for (player, id, transform, health, inventory) in players_query.iter(world) {
        let weapon = inventory.and_then(|inventory| {
            let (entity, weapon) = inventory.weapons.get(inventory.active_weapon_index)?;
            let (state, modes) = weapons.get(world, *entity).ok()?;
            let mode = modes.modes.get(&state.active_mode);
            Some(json!({
                "name": weapon.config.name,
                "mode": state.active_mode,
                "ammo": mode.map(|m| m.mag_ammo),
                "mags": mode.map(|m| m.mag_quantity),
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
    ), With<Enemy>>();
    let mut enemies: Vec<Value> = enemies_query
        .iter(world)
        .map(|(id, transform, health, monster)| {
            json!({
                "id": id.0,
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

    Ok(json!({
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
    pan: (i16, i16),
}

fn input(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let params: InputParams = parse(params)?;
    let input = box_input(&params.buttons, params.pan);
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
