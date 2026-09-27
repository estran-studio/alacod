//! Format RON des scénarios (`tests/scenarios/*.ron`).
//!
//! ```ron
//! Scenario(
//!     frames: 600,
//!     players: [
//!         (inputs: [
//!             (from: 0, to: 120, buttons: [Right, Fire], pan: (100, 0)),
//!         ]),
//!     ],
//!     expect: [
//!         PlayerAlive(handle: 0, at_frame: 300),
//!         KillsAtLeast(kills: 1, at_frame: 600),
//!     ],
//! )
//! ```
//!
//! Les frames d'un segment sont celles où l'input est *lu* : GGRS l'applique après
//! le délai d'input de la session.

use game::character::player::input::{
    BoxInput, InputSegment, ScriptedInputs, INPUT_DASH, INPUT_DOWN, INPUT_INTERACTION,
    INPUT_LEFT, INPUT_MELEE_ATTACK, INPUT_MODIFIER, INPUT_RELOAD, INPUT_RIGHT, INPUT_SPRINT,
    INPUT_SWITCH_WEAPON_MODE, INPUT_UP,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Scenario {
    /// Map LDtk, relative au dossier des assets.
    #[serde(default = "default_map")]
    pub map: String,
    #[serde(default = "default_map_seed")]
    pub map_seed: i32,
    /// Nombre de frames simulées.
    pub frames: u32,
    /// Un script par joueur ; le joueur `i` a le handle GGRS `i`.
    pub players: Vec<PlayerScript>,
    #[serde(default)]
    pub expect: Vec<Expectation>,
}

fn default_map() -> String {
    "exemples/test_map.ldtk".into()
}

fn default_map_seed() -> i32 {
    123456
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PlayerScript {
    #[serde(default)]
    pub inputs: Vec<Segment>,
}

/// Input maintenu sur les frames `from..to`.
#[derive(Debug, Clone, Deserialize)]
pub struct Segment {
    pub from: u32,
    pub to: u32,
    #[serde(default)]
    pub buttons: Vec<Button>,
    /// Visée : vecteur du joueur vers le pointeur, en unités monde.
    #[serde(default)]
    pub pan: (i16, i16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Button {
    Up,
    Down,
    Left,
    Right,
    Fire,
    SwitchWeapon,
    SwitchWeaponMode,
    Reload,
    Sprint,
    Dash,
    Modifier,
    Interaction,
    Melee,
}

/// Vérification faite quand la simulation atteint `at_frame`.
#[derive(Debug, Clone, Deserialize)]
pub enum Expectation {
    PlayerAlive { handle: usize, at_frame: u32 },
    PlayerDead { handle: usize, at_frame: u32 },
    WaveAtLeast { wave: u32, at_frame: u32 },
    KillsAtLeast { kills: u32, at_frame: u32 },
}

impl Expectation {
    pub fn at_frame(&self) -> u32 {
        match self {
            Self::PlayerAlive { at_frame, .. }
            | Self::PlayerDead { at_frame, .. }
            | Self::WaveAtLeast { at_frame, .. }
            | Self::KillsAtLeast { at_frame, .. } => *at_frame,
        }
    }
}

impl Scenario {
    pub fn from_ron(source: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(source)
    }

    pub fn scripted_inputs(&self) -> ScriptedInputs {
        ScriptedInputs {
            players: self
                .players
                .iter()
                .map(|player| player.inputs.iter().map(Segment::to_input_segment).collect())
                .collect(),
        }
    }
}

impl Segment {
    fn to_input_segment(&self) -> InputSegment {
        let mut input = BoxInput {
            pan_x: self.pan.0,
            pan_y: self.pan.1,
            ..Default::default()
        };
        for button in &self.buttons {
            match button {
                Button::Fire => input.fire = true,
                Button::SwitchWeapon => input.switch_weapon = true,
                other => input.buttons |= button_bit(*other),
            }
        }
        InputSegment {
            from: self.from,
            to: self.to,
            input,
        }
    }
}

fn button_bit(button: Button) -> u16 {
    match button {
        Button::Up => INPUT_UP,
        Button::Down => INPUT_DOWN,
        Button::Left => INPUT_LEFT,
        Button::Right => INPUT_RIGHT,
        Button::Reload => INPUT_RELOAD,
        Button::SwitchWeaponMode => INPUT_SWITCH_WEAPON_MODE,
        Button::Sprint => INPUT_SPRINT,
        Button::Dash => INPUT_DASH,
        Button::Modifier => INPUT_MODIFIER,
        Button::Interaction => INPUT_INTERACTION,
        Button::Melee => INPUT_MELEE_ATTACK,
        Button::Fire | Button::SwitchWeapon => 0,
    }
}
