use bevy::prelude::*;
use std::sync::{Arc, Mutex};
use tracing_subscriber::{
    fmt::{self, MakeWriter},
    prelude::*,
    EnvFilter, Registry,
}; // Updated imports
use tracing_web::MakeWebConsoleWriter;
use wasm_bindgen::prelude::*;

#[derive(Resource, Clone, Default)]
pub struct WebLogBuffer(pub Arc<Mutex<String>>);

#[derive(Clone)]
pub struct WebLogWriter {
    buffer: Arc<Mutex<String>>,
}

impl std::io::Write for WebLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Ok(mut buffer_guard) = self.buffer.lock() {
            if let Ok(s) = std::str::from_utf8(buf) {
                buffer_guard.push_str(s);

                const MAX_LOG_BYTES: usize = 5 * 1024 * 1024; // Limite à 5 Mo
                if buffer_guard.len() > MAX_LOG_BYTES {
                    let to_remove = buffer_guard.len() - MAX_LOG_BYTES;
                    buffer_guard.drain(..to_remove);
                }
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for WebLogBuffer {
    type Writer = WebLogWriter;

    fn make_writer(&'a self) -> Self::Writer {
        WebLogWriter {
            buffer: self.0.clone(),
        }
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = download_log_file_js)]
    fn download_log_file_js_binding(filename: &str, content: &str);
}

fn initiate_log_download_from_rust(logs_content: String, filename: &str) {
    if cfg!(target_arch = "wasm32") {
        // SAFETY: Calling JS function, ensure it's correctly defined on JS side.
        #[warn(unused_unsafe)]
        unsafe {
            download_log_file_js_binding(filename, &logs_content);
        }
    } else {
        warn!("Attempting to download log on a non-WASM platform. Ignoring.");
        // For native debugging, you might write to a local file here.
        // For example:
        // if let Err(e) = std::fs::write(filename, logs_content) {
        //     error!("Failed to write log locally: {}", e);
        // } else {
        //     info!("Log saved locally (native) in: {}", filename);
        // }
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct DownloadLogTrigger(pub bool);

pub fn check_for_log_download_request(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut trigger: ResMut<DownloadLogTrigger>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyL) && !trigger.0 {
        info!("Log download requested (L key)!"); // Corrected log message
        trigger.0 = true;
    }
}

pub fn process_log_download_trigger(
    mut trigger: ResMut<DownloadLogTrigger>,
    log_buffer: Res<WebLogBuffer>,
    // Assuming you have a frame counter resource. If not, you might need to adapt the filename.
    // For this example, let's use a placeholder if frame isn't available or simplify.
    // frame_count: Option<Res<crate::frame::FrameCount>>, // Example if it's optional
) {
    if trigger.0 {
        let logs_string;
        {
            // Explicitly unlock and handle potential poison.
            let buffer_guard = log_buffer.0.lock().unwrap_or_else(|poisoned| {
                error!("Log buffer mutex was poisoned! Recovering.");
                poisoned.into_inner()
            });
            logs_string = buffer_guard.clone();
        }

        if !logs_string.is_empty() {
            // Simplified filename if frame count is complex or unavailable for this snippet
            // let current_time = web_time::SystemTime::now() // Requires web_time crate for wasm
            //     .duration_since(web_time::UNIX_EPOCH)
            //     .unwrap_or_default()
            //     .as_secs();
            // let filename = format!("game_log_{}.txt", current_time);
            let filename = "game_log.txt".to_string(); // Simpler filename
                                                       // If you have your frame count:
                                                       // let filename = format!("game_log_frame_{}.txt", frame_count.map_or(0, |f| f.frame));

            initiate_log_download_from_rust(logs_string, &filename);
            info!("Log download initiated to {}", filename);
        } else {
            warn!("Log buffer is empty. No download initiated.");
        }

        trigger.0 = false;
    }
}

fn setup_wasm_logging_subscriber_system(mut commands: Commands) {
    let log_buffer_resource = WebLogBuffer::default();
    commands.insert_resource(log_buffer_resource.clone());

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,wgpu_core=warn,wgpu_hal=warn,naga=warn".into());

    let file_log = fmt::Layer::new()
        .without_time()
        .with_ansi(false)
        .with_writer(log_buffer_resource);

    let console_log = fmt::Layer::new()
        .without_time()
        .with_ansi(false)
        .with_writer(MakeWebConsoleWriter::new());

    let subscriber = Registry::default()
        .with(filter)
        .with(file_log)
        .with(console_log);

    if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
        error!("failed to register custom logger {}", err);
        return;
    }

    info!("WASM logging initialized. Logs buffered and sent to console. Press 'L' to download.");
}

pub struct WasmWebLogPlugin;

impl Plugin for WasmWebLogPlugin {
    fn build(&self, app: &mut App) {
        // Ensure the plugin is added only for the WASM target
        if cfg!(target_arch = "wasm32") {
            app.add_systems(Startup, setup_wasm_logging_subscriber_system)
                .init_resource::<DownloadLogTrigger>()
                .add_systems(
                    Update,
                    (check_for_log_download_request, process_log_download_trigger).chain(),
                );

            info!("WasmWebLogPlugin added.");
        }
    }
}
