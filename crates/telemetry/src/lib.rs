use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

pub mod panic;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub level: String,
    pub message: String,
    pub frame: Option<u64>,
    pub checksum_local: Option<u128>,
    pub checksum_remote: Option<u128>,
    pub extra: Option<String>,
    pub timestamp: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogPayload {
    pub stream: String,
    pub values: Vec<Vec<serde_json::Value>>,
}

#[derive(Resource, Clone, Debug)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub url: String,
    pub auth_token: String,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: "http://localhost:5080/api/default/default/_json".to_string(),
            auth_token: "".to_string(),
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct TelemetryBuffer {
    pub events: Arc<Mutex<Vec<TelemetryEvent>>>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Resource)]
pub struct TelemetrySender(pub std::sync::mpsc::Sender<TelemetryEvent>);

pub struct TelemetryPlugin;

impl Plugin for TelemetryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TelemetryConfig>()
            .init_resource::<TelemetryBuffer>();

        panic::register_panic_hook(app);

        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Startup, start_telemetry_worker);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn start_telemetry_worker(mut commands: Commands, config: Res<TelemetryConfig>) {
    // Check enabled? Even if disabled, we probably want the channel to exist so system params don't panic?
    // But TelemetrySender is Option<Res> in some places? No, in send_event it's &TelemetrySender.
    // So we MUST insert the resource.

    let (tx, rx) = std::sync::mpsc::channel::<TelemetryEvent>();
    commands.insert_resource(TelemetrySender(tx));

    if config.enabled {
        let config = config.clone();
        std::thread::spawn(move || {
            let mut buffer = Vec::new();
            let mut last_flush = std::time::Instant::now();
            let flush_interval = std::time::Duration::from_secs(1);
            let max_batch_size = 50;

            loop {
                // Determine wait time
                let now = std::time::Instant::now();
                let time_since_flush = now.duration_since(last_flush);
                let timeout = if time_since_flush >= flush_interval {
                    std::time::Duration::from_millis(0) // Flush immediately if overdue
                } else {
                    flush_interval - time_since_flush
                };

                match rx.recv_timeout(timeout) {
                    Ok(event) => {
                        buffer.push(event);
                        if buffer.len() >= max_batch_size {
                            crate::panic::send_log_blocking(&config, &buffer);
                            buffer.clear();
                            last_flush = std::time::Instant::now();
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        // Timeout reached, flush if we have anything
                        if !buffer.is_empty() {
                            crate::panic::send_log_blocking(&config, &buffer);
                            buffer.clear();
                            last_flush = std::time::Instant::now();
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break, // Channel closed
                }
            }
        });
    }
}

pub fn send_event(
    #[cfg(not(target_arch = "wasm32"))] sender: &TelemetrySender,
    #[cfg(target_arch = "wasm32")] config: &TelemetryConfig,
    event: TelemetryEvent,
) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Just push to channel
        let _ = sender.0.send(event);
    }

    #[cfg(target_arch = "wasm32")]
    {
        if !config.enabled {
            return;
        }
        let config = config.clone();
        let event = event.clone();
        wasm_bindgen_futures::spawn_local(async move {
            crate::panic::send_log_wasm(&config, &[event]).await;
        });
    }
}
