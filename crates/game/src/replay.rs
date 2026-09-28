//! Format RON des scénarios et replays (`tests/scenarios/*.ron`, enregistrements).
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
//!
//! Joué par `crates/scenario` ; écrit par l'enregistrement (`crate::recording`).

use crate::character::player::input::{
    BoxInput, InputSegment, ScriptedInputs, INPUT_DASH, INPUT_DOWN, INPUT_INTERACTION,
    INPUT_FORCE_CRASH, INPUT_LEFT, INPUT_MELEE_ATTACK, INPUT_MODIFIER, INPUT_RELOAD, INPUT_RIGHT, INPUT_SPRINT,
    INPUT_SWITCH_WEAPON_MODE, INPUT_UP,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expect: Vec<Expectation>,
    /// Modifications de la config des armes pour ce scénario (ex. moins de chargeurs pour
    /// tester leur épuisement en quelques secondes).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub weapon_overrides: Vec<WeaponOverride>,
}

/// Remplace des valeurs de chargeur d'une arme, pour tous ses modes ou un seul.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponOverride {
    pub weapon: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mag_size: Option<u32>,
    /// Chargeurs de réserve.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mag_limit: Option<u32>,
}

fn default_map() -> String {
    "exemples/test_map.ldtk".into()
}

fn default_map_seed() -> i32 {
    123456
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerScript {
    #[serde(default)]
    pub inputs: Vec<Segment>,
}

/// Input maintenu sur les frames `from..to`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub from: u32,
    pub to: u32,
    #[serde(default)]
    pub buttons: Vec<Button>,
    /// Visée : vecteur du joueur vers le pointeur, en unités monde.
    #[serde(default)]
    pub pan: (i16, i16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Touche de debug qui provoque un crash volontaire
    ForceCrash,
}

/// Vérification faite quand la simulation atteint `at_frame`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expectation {
    PlayerAlive { handle: usize, at_frame: u32 },
    PlayerDead { handle: usize, at_frame: u32 },
    WaveAtLeast { wave: u32, at_frame: u32 },
    KillsAtLeast { kills: u32, at_frame: u32 },
    /// Fenêtres détruites (obstacles cassables qui ne bloquent plus).
    WindowsBrokenAtLeast { windows: u32, at_frame: u32 },
    /// Arme active du joueur (et son mode de tir, si précisé).
    ActiveWeapon {
        handle: usize,
        weapon: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mode: Option<String>,
        at_frame: u32,
    },
    /// Munitions exactes dans le chargeur de l'arme active.
    Ammo { handle: usize, ammo: u32, at_frame: u32 },
    /// Portes ouvertes (sans collider).
    DoorsOpenAtLeast { doors: u32, at_frame: u32 },
    /// Santé exacte d'une fenêtre (par son GgrsNetId).
    WindowHealth { window: usize, health: u8, at_frame: u32 },
    /// Toutes les balles en vol sont dans la zone (ex. elles ne traversent pas un mur).
    BulletsInside { x_min: f32, x_max: f32, y_min: f32, y_max: f32, at_frame: u32 },
    /// Position du joueur, à `tolerance` unités près sur chaque axe.
    PlayerPosition { handle: usize, x: f32, y: f32, tolerance: f32, at_frame: u32 },
}

impl Expectation {
    pub fn at_frame(&self) -> u32 {
        match self {
            Self::PlayerAlive { at_frame, .. }
            | Self::PlayerDead { at_frame, .. }
            | Self::WaveAtLeast { at_frame, .. }
            | Self::KillsAtLeast { at_frame, .. }
            | Self::WindowsBrokenAtLeast { at_frame, .. }
            | Self::ActiveWeapon { at_frame, .. }
            | Self::Ammo { at_frame, .. }
            | Self::DoorsOpenAtLeast { at_frame, .. }
            | Self::WindowHealth { at_frame, .. }
            | Self::BulletsInside { at_frame, .. }
            | Self::PlayerPosition { at_frame, .. } => *at_frame,
        }
    }
}

impl Scenario {
    /// Lecture avec l'extension RON `implicit_some` : un champ optionnel s'écrit
    /// `mode: "default"` plutôt que `mode: Some("default")`.
    pub fn from_ron(source: &str) -> Result<Self, ron::error::SpannedError> {
        ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(source)
    }

    pub fn to_ron(&self) -> String {
        let config = ron::ser::PrettyConfig::new()
            .struct_names(true)
            .depth_limit(4)
            .indentor("    ".to_string())
            .extensions(ron::extensions::Extensions::IMPLICIT_SOME);
        ron::ser::to_string_pretty(self, config).expect("sérialisation du scénario")
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
    /// Segment qui rejoue `input` sur les frames `from..to`.
    pub fn from_input(from: u32, to: u32, input: &BoxInput) -> Self {
        let mut buttons: Vec<Button> = ALL_BUTTONS
            .iter()
            .copied()
            .filter(|b| button_bit(*b) != 0 && input.buttons & button_bit(*b) != 0)
            .collect();
        if input.fire {
            buttons.push(Button::Fire);
        }
        if input.switch_weapon {
            buttons.push(Button::SwitchWeapon);
        }
        Self {
            from,
            to,
            buttons,
            pan: (input.pan_x, input.pan_y),
        }
    }

    fn to_input_segment(&self) -> InputSegment {
        InputSegment {
            from: self.from,
            to: self.to,
            input: box_input(&self.buttons, self.pan),
        }
    }
}

/// Input GGRS correspondant à des boutons enfoncés et une visée.
pub fn box_input(buttons: &[Button], pan: (i16, i16)) -> BoxInput {
    let mut input = BoxInput {
        pan_x: pan.0,
        pan_y: pan.1,
        ..Default::default()
    };
    for button in buttons {
        match button {
            Button::Fire => input.fire = true,
            Button::SwitchWeapon => input.switch_weapon = true,
            other => input.buttons |= button_bit(*other),
        }
    }
    input
}

const ALL_BUTTONS: [Button; 14] = [
    Button::Up,
    Button::Down,
    Button::Left,
    Button::Right,
    Button::Fire,
    Button::SwitchWeapon,
    Button::SwitchWeaponMode,
    Button::Reload,
    Button::Sprint,
    Button::Dash,
    Button::Modifier,
    Button::Interaction,
    Button::Melee,
    Button::ForceCrash,
];

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
        Button::ForceCrash => INPUT_FORCE_CRASH,
        Button::Fire | Button::SwitchWeapon => 0,
    }
}
