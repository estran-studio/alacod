//! Diagnostic de la manette (C-7), **application** : présentation et journal, hors `GgrsSchedule`, rien
//! de rollback, aucune liaison ni aucun seuil d'input touché (`control.rs`, `input_devices.rs`).
//!
//! - `ALACOD_INPUT_DEBUG=1` : à la connexion d'une manette, nom, identifiants et source du mapping ; ensuite
//!   chaque axe et bouton **bruts** (événements `RawGamepadEvent`, avant les liaisons) dont la valeur change d'au
//!   moins [`NOISE_THRESHOLD`] depuis la dernière valeur journalisée (pas une ligne par frame).
//! - Avertissement nommé, une seule fois par manette (toujours actif, sans la variable) : une manette connectée
//!   qui n'a émis aucun événement brut pendant [`SILENT_FRAMES`] frames de jeu (un joueur local existe) est
//!   sans doute tenue par un autre programme (Parsec, Steam Input). Une manette posée, que personne ne touche,
//!   est indiscernable : le message le dit.

use std::collections::{BTreeMap, BTreeSet};

use bevy::input::gamepad::{Gamepad, RawGamepadEvent};
use bevy::prelude::*;

use crate::character::player::LocalPlayer;

/// Variation minimale (depuis la dernière valeur journalisée) d'un axe ou d'un bouton pour une ligne de journal.
pub const NOISE_THRESHOLD: f32 = 0.05;

/// Frames de rendu de jeu (un joueur local présent) sans le moindre événement brut avant l'avertissement
/// (≈ 30 s à 60 Hz).
pub const SILENT_FRAMES: u32 = 1800;

/// Variable d'environnement qui allume le journal des événements bruts.
pub const INPUT_DEBUG_ENV: &str = "ALACOD_INPUT_DEBUG";

/// État du diagnostic, application seule (jamais rollback, jamais lu par la simulation).
#[derive(Resource, Debug)]
pub struct GamepadDiagnostics {
    /// `ALACOD_INPUT_DEBUG` : journal des connexions et des événements bruts.
    pub debug: bool,
    /// Frames de jeu muettes avant l'avertissement ([`SILENT_FRAMES`], réglable pour un test).
    pub silent_frames: u32,
    /// Frames de jeu écoulées sans événement, par manette connectée.
    silent: BTreeMap<Entity, u32>,
    /// Manettes qui ont émis au moins un événement brut : jamais averties.
    spoke: BTreeSet<Entity>,
    /// Manettes déjà averties (une seule fois chacune).
    pub warned: BTreeSet<Entity>,
    /// Dernière valeur journalisée par manette et entrée brute.
    last: BTreeMap<(Entity, String), f32>,
}

impl GamepadDiagnostics {
    pub fn new(debug: bool) -> Self {
        Self {
            debug,
            silent_frames: SILENT_FRAMES,
            silent: BTreeMap::new(),
            spoke: BTreeSet::new(),
            warned: BTreeSet::new(),
            last: BTreeMap::new(),
        }
    }

    /// Lit `ALACOD_INPUT_DEBUG` (`1`, `true`, `on`) ; absent côté navigateur (pas d'environnement).
    pub fn from_env() -> Self {
        let debug = std::env::var(INPUT_DEBUG_ENV)
            .map(|v| matches!(v.trim(), "1" | "true" | "on"))
            .unwrap_or(false);
        Self::new(debug)
    }
}

/// Motif du GUID SDL d'une manette : bevy ne transmet que fabricant et produit (le bus et la version, `????`,
/// manquent). Format SDL : bus(2) crc(2) fabricant(2) 0000 produit(2) 0000 version(2) 0000, en octets
/// petit-boutiste. Ex. `045e:0b13` donne `????0000 5e04 0000 130b 0000 ????0000`.
pub fn guid_pattern(vendor: Option<u16>, product: Option<u16>) -> String {
    let le = |v: Option<u16>| match v {
        Some(v) => format!("{:02x}{:02x}", v & 0xff, v >> 8),
        None => "????".to_string(),
    };
    format!("????0000{}0000{}0000????0000", le(vendor), le(product))
}

/// Source du mapping des boutons et des axes, d'après la variable `SDL_GAMECONTROLLERCONFIG` (lue par gilrs,
/// prioritaire sur sa base embarquée).
pub fn mapping_source(env: Option<&str>) -> &'static str {
    if cfg!(target_arch = "wasm32") {
        "API Gamepad du navigateur (mapping « standard » du navigateur, ou brut si le navigateur n'en donne pas)"
    } else if env.is_some_and(|v| !v.trim().is_empty()) {
        "variable d'environnement SDL_GAMECONTROLLERCONFIG (prioritaire sur la base SDL embarquée par gilrs)"
    } else {
        "base SDL embarquée par gilrs, ligne choisie par le GUID ; sans ligne pour ce GUID, mapping par défaut de gilrs (bevy ne dit pas lequel)"
    }
}

/// Connexion d'une manette : une ligne de journal (nom, fabricant:produit, motif de GUID, source du mapping),
/// seulement avec `ALACOD_INPUT_DEBUG`.
pub fn log_gamepad_connections(
    diag: Res<GamepadDiagnostics>,
    added: Query<(&Name, &Gamepad), Added<Gamepad>>,
) {
    if !diag.debug {
        return;
    }
    let env = std::env::var("SDL_GAMECONTROLLERCONFIG").ok();
    for (name, pad) in &added {
        let (vendor, product) = (pad.vendor_id(), pad.product_id());
        let id = |v: Option<u16>| v.map_or("?".to_string(), |v| format!("{v:04x}"));
        info!(
            "manette connectée : nom=\"{name}\" id={}:{} guid={} mapping={}",
            id(vendor),
            id(product),
            guid_pattern(vendor, product),
            mapping_source(env.as_deref())
        );
    }
}

/// Événements bruts (avant les liaisons de leafwing) : journal des changements au-delà du bruit
/// (`ALACOD_INPUT_DEBUG`) et fin du silence d'une manette (toujours).
pub fn track_raw_gamepad_events(
    mut diag: ResMut<GamepadDiagnostics>,
    mut events: MessageReader<RawGamepadEvent>,
    names: Query<&Name, With<Gamepad>>,
) {
    for event in events.read() {
        let (pad, input, value) = match event {
            RawGamepadEvent::Button(e) => (e.gamepad, format!("bouton {:?}", e.button), e.value),
            RawGamepadEvent::Axis(e) => (e.gamepad, format!("axe {:?}", e.axis), e.value),
            RawGamepadEvent::Connection(_) => continue,
        };
        diag.silent.remove(&pad);
        diag.spoke.insert(pad);
        if !diag.debug {
            continue;
        }
        let last = diag.last.entry((pad, input.clone())).or_insert(0.0);
        if (value - *last).abs() >= NOISE_THRESHOLD {
            *last = value;
            let name = names.get(pad).map_or("?", |n| n.as_str());
            info!("manette \"{name}\" : {input} = {value:.3}");
        }
    }
}

/// Pendant qu'un joueur local existe, compte les frames des manettes connectées qui n'ont rien émis ; à
/// [`GamepadDiagnostics::silent_frames`], un `warn!` nommé, une fois par manette. Oublie les manettes déconnectées.
pub fn warn_silent_gamepads(
    mut diag: ResMut<GamepadDiagnostics>,
    pads: Query<(Entity, &Name), With<Gamepad>>,
    players: Query<(), With<LocalPlayer>>,
    mut gone: RemovedComponents<Gamepad>,
) {
    for pad in gone.read() {
        diag.silent.remove(&pad);
        diag.spoke.remove(&pad);
        diag.warned.remove(&pad);
        diag.last.retain(|(p, _), _| *p != pad);
    }
    if players.is_empty() {
        return;
    }
    let limit = diag.silent_frames;
    for (pad, name) in &pads {
        if diag.warned.contains(&pad) || diag.spoke.contains(&pad) {
            continue;
        }
        let frames = diag.silent.entry(pad).or_insert(0);
        *frames += 1;
        if *frames < limit {
            continue;
        }
        diag.silent.remove(&pad);
        diag.warned.insert(pad);
        warn!(
            "manette \"{name}\" connectée mais muette : aucun axe ni bouton reçu depuis {limit} frames de jeu. \
             Causes probables : la manette est tenue par un autre programme (Parsec, Steam Input, \
             un autre jeu) ; fermez-le et reconnectez la manette. Si personne ne touche la manette, ignorez ce \
             message. ALACOD_INPUT_DEBUG=1 journalise ce que le jeu reçoit."
        );
    }
}

/// Installe le diagnostic (`CharacterAppPlugin`, hors headless) : `Update`, jamais `GgrsSchedule`.
pub struct GamepadDiagnosticsPlugin;

impl Plugin for GamepadDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GamepadDiagnostics::from_env());
        app.add_systems(
            Update,
            (
                log_gamepad_connections,
                track_raw_gamepad_events,
                warn_silent_gamepads.after(track_raw_gamepad_events),
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_motif_du_guid_suit_le_format_sdl() {
        // GUID réel de la manette Xbox Bluetooth : 050000005e040000130b000015050000.
        assert_eq!(
            guid_pattern(Some(0x045e), Some(0x0b13)),
            "????00005e040000130b0000????0000"
        );
        assert_eq!(guid_pattern(None, None), "????0000????0000????0000????0000");
    }
}
