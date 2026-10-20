//! Télémétrie des desyncs, moitié **application** (D69 étape 2, E6) : la simulation (`jjrs::log_ggrs_events`) écrit
//! un message [`DesyncObserved`] au moment où GGRS détecte la divergence et ne connaît ni le crate `telemetry` ni
//! l'horloge ; ce système le transmet (natif : le thread d'envoi de `TelemetryPlugin` ; web : la requête
//! construite depuis `TelemetryConfig`). Le même événement, au même instant, qu'avant la coupe.
//!
//! Chemin final (E9) : `game_app/src/jjrs/telemetry.rs`.

use bevy::prelude::*;

use crate::jjrs::DesyncObserved;

/// Envoie un événement de télémétrie `DESYNC` par message [`DesyncObserved`] lu. Ordonné après
/// `log_ggrs_events` (même update). Sans `TelemetrySender` (natif, `TelemetryPlugin` absent : scénarios, tests),
/// les messages sont lus et ignorés.
pub fn forward_desync_telemetry(
    mut observed: MessageReader<DesyncObserved>,
    #[cfg(not(target_arch = "wasm32"))] sender: Option<Res<::telemetry::TelemetrySender>>,
    #[cfg(target_arch = "wasm32")] config: Res<::telemetry::TelemetryConfig>,
) {
    for desync in observed.read() {
        let event = ::telemetry::TelemetryEvent {
            level: "DESYNC".to_string(),
            message: "Desync detected between local and remote".to_string(),
            frame: Some(desync.frame.try_into().unwrap_or(0)),
            checksum_local: Some(desync.local_checksum),
            checksum_remote: Some(desync.remote_checksum),
            extra: Some(desync.addr_debug.clone()),
            timestamp: chrono::Utc::now().timestamp_micros(),
        };

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(sender) = sender.as_ref() {
            ::telemetry::send_event(sender, event);
        }

        #[cfg(target_arch = "wasm32")]
        ::telemetry::send_event(&config, event);
    }
}
