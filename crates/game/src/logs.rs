use bevy::prelude::*;
use chrono::Local; // Import chrono
use std::fs; // For directory creation
use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[cfg(not(target_arch = "wasm32"))]
// Function to set up logging and return the guard
pub fn setup_logging(suffix: Option<String>) -> Result<WorkerGuard, Box<dyn std::error::Error>> {
    let log_directory = Path::new("logs");

    // Create the log directory if it doesn't exist
    if !log_directory.exists() {
        fs::create_dir_all(log_directory)?;
    }

    // Generate a filename with the current timestamp
    let suffix = suffix.map_or_else(
        || Local::now().format("%Y-%m-%d_%H-%M-%S").to_string(),
        |v| v,
    );

    let log_filename = format!("game_run_{}.log", suffix);
    let log_file_path = log_directory.join(log_filename);

    // Use tracing_appender::file::File::create for a single, non-rolling file
    // For more advanced scenarios (like putting it in a directory with a prefix but still non-rolling for *this specific file*),
    // you can use tracing_appender::rolling::never(log_directory, log_filename);
    // However, for a direct path, direct file creation is simpler.
    let file_appender = match std::fs::File::create(&log_file_path) {
        Ok(file) => file,
        Err(e) => {
            // Fallback to stderr if file creation fails
            eprintln!("Failed to create log file {:?}: {}", log_file_path, e);
            // As a simple fallback, we won't log to file in this error case.
            // A more robust solution might try a default name or panic.
            let (non_blocking_writer, guard) = tracing_appender::non_blocking(std::io::stderr());
            let subscriber = tracing_subscriber::registry()
                .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
                .with(fmt::Layer::new().with_writer(non_blocking_writer));
            tracing::subscriber::set_global_default(subscriber)?;
            return Ok(guard);
        }
    };

    let (non_blocking_writer, guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,wgpu_core=warn,wgpu_hal=warn,naga=warn".into()),
        )
        .with(
            fmt::Layer::new()
                .with_writer(non_blocking_writer) // Log to our timestamped file
                .without_time()
                .with_ansi(false), // ANSI colors don't make sense in a file
        )
        .with(
            fmt::Layer::new().with_writer(std::io::stdout), // Also log to console
        );

    tracing::subscriber::set_global_default(subscriber)?;

    info!("Logging initialized. Log file: {:?}", log_file_path); // Log the path for confirmation

    Ok(guard)
}

#[derive(Resource)]
pub struct LogGuard {
    pub guard: tracing_appender::non_blocking::WorkerGuard,
}

pub struct NativeLogPlugin(pub String);

impl Plugin for NativeLogPlugin {
    fn build(&self, app: &mut App) {
        if cfg!(not(target_arch = "wasm32")) {
            let guard = setup_logging(Some(self.0.clone())).unwrap();
            app.insert_resource(LogGuard { guard });
        }
    }
}
