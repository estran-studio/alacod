//! Contrat de donnée du dash nommé d'un personnage (`dash: Some(..)` de son RON, M2-T2, §43).
//!
//! Types de **donnée** seulement (aucun composant, aucun système) : ils vivent ici, et non dans
//! `game::character::movement`, pour que `content` (profils de bac à sable, `PlayerScript::dash`
//! et `SandboxPlayer::dash`) et `game` partagent **un seul** format. `game::character::movement`
//! les réexporte : les chemins existants ne bougent pas.

use bevy_fixed::fixed_math;
use serde::{Deserialize, Serialize};

/// Effet de variante d'un dash (M2-T2, §43). Enum fermé, nouveaux effets **en fin d'enum**.
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum DashEffect {
    /// Roulé v1 : on traverse la horde sans ralentir.
    #[default]
    Roll,
    /// Bousculade : repousse les ennemis touchés pendant le dash (`shove_force`).
    Shove,
    /// Sur place : aucune distance, seulement des i-frames (parade sans mouvement).
    InPlace,
}

/// Contre quoi valent les i-frames d'un dash (M2-T2).
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum IframesKind {
    /// V1 : `Health.invulnerable_until_frame`, tout dégât.
    #[default]
    All,
    /// Saut par-dessus les balles : les projectiles traversent, la mêlée et les contacts touchent.
    Projectiles,
}

/// Dash nommé d'un personnage (`dash: Some(..)` dans son RON, M2-T2, §43). Chaque champ absent
/// retombe sur le réglage v1 de `MovementConfig`.
#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DashConfig {
    #[serde(default)]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Option<bevy_fixed::schema::FixedString>")
    )]
    pub distance: Option<fixed_math::Fixed>,
    #[serde(default)]
    pub duration_frames: Option<u32>,
    #[serde(default)]
    pub cooldown_frames: Option<u32>,
    #[serde(default)]
    pub iframes: Option<u32>,
    #[serde(default)]
    pub iframes_kind: IframesKind,
    #[serde(default)]
    pub effect: DashEffect,
    /// Recul (px/s, `Velocity.knockback`) des ennemis bousculés (`Shove`).
    #[serde(default)]
    #[cfg_attr(feature = "schema", schemars(with = "bevy_fixed::schema::FixedString"))]
    pub shove_force: fixed_math::Fixed,
    #[serde(default)]
    pub charges: Option<u32>,
    #[serde(default)]
    pub recharge_frames: Option<u32>,
}
