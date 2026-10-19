//! `TelemetryConfig` (crate `telemetry`, application) posé à partir de `GameArgs` (D69 étape 2, E6).
//!
//! `GameArgsPlugin` (simulation, lu aussi par `scenario`) ne pose plus cette ressource : les scénarios n'ont ni
//! télémétrie ni crate `telemetry` à compiler. `BaseArgsPlugin` ajoute ce plugin juste après `GameArgsPlugin`.
//!
//! Chemin final (E9) : `game_app/src/args/telemetry_config.rs`.

use bevy::prelude::*;

use crate::args::GameArgs;

/// Insère `telemetry::TelemetryConfig` d'après les champs `telemetry*` de [`GameArgs`].
pub struct TelemetryConfigPlugin(pub GameArgs);

impl Plugin for TelemetryConfigPlugin {
    fn build(&self, app: &mut App) {
        let args = self.0.clone();
        app.insert_resource(::telemetry::TelemetryConfig {
            enabled: args.telemetry,
            url: args.telemetry_url,
            auth_token: args.telemetry_auth,
        });
    }
}
