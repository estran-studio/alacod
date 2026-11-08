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

/// Profil de vitesse d'un dash (P-1, `docs/conventions.md` §110).
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum DashProfile {
    /// V1 : vitesse qui décroît linéairement jusqu'à celle de la course (`combat::actors::dash_step`).
    #[default]
    Linear,
    /// Élan puis freinage : la part `u` du dash couvre `1 − (1 − u)²` de la distance (vitesse qui
    /// part au double de la moyenne et tombe à zéro).
    EaseOut,
}

/// Jugement de l'appui d'un dash contre le tempo d'une horloge (P-1, §110) : « parfait » ou
/// « bien » donnent un dash plus long et plus protégé ; hors du temps, le dash est normal (jamais
/// de pénalité). L'appui est jugé à sa frame, même s'il attend la fin du dash en cours.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DashOnBeat {
    /// Id de l'horloge (kind `Clock`) qui porte le `tempo` ; elle doit être activée
    /// (`entry.clocks` ou `Scenario::clocks`), sinon aucun appui n'est jugé.
    pub clock: String,
    /// Fenêtre « parfait » : écart au temps le plus proche, en millisecondes (borne comprise).
    pub perfect_ms: u32,
    /// Fenêtre « bien », en millisecondes (borne comprise, `>= perfect_ms`).
    pub good_ms: u32,
    /// Multiplicateur des deux fenêtres quand la session compte au moins deux joueurs (la
    /// fenêtre élargie en coop). Absent : 1.
    #[serde(default)]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Option<bevy_fixed::schema::FixedString>")
    )]
    pub coop_window_mult: Option<fixed_math::Fixed>,
    /// Part de distance ajoutée par un « parfait » (la moitié pour un « bien ») : `0.3` = +30 %.
    #[serde(default)]
    #[cfg_attr(feature = "schema", schemars(with = "bevy_fixed::schema::FixedString"))]
    pub distance_bonus: fixed_math::Fixed,
    /// Part de la durée du dash ajoutée aux i-frames par un « parfait » (la moitié pour un
    /// « bien ») : `0.25` = +25 % de la durée.
    #[serde(default)]
    #[cfg_attr(feature = "schema", schemars(with = "bevy_fixed::schema::FixedString"))]
    pub iframes_bonus: fixed_math::Fixed,
    /// Plafond des i-frames, en part de la durée du dash (`0.95`). Absent : la durée entière.
    #[serde(default)]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Option<bevy_fixed::schema::FixedString>")
    )]
    pub iframes_cap: Option<fixed_math::Fixed>,
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
    /// P-1 (§110) : profil de vitesse (défaut : linéaire, la v1).
    #[serde(default)]
    pub profile: DashProfile,
    /// P-1 : sans direction tenue, le dash se fait sur place (i-frames et durée, aucune distance).
    /// Faux (défaut) : sans direction, il part vers la visée puis le regard (v1).
    #[serde(default)]
    pub in_place_without_direction: bool,
    /// P-1 : un appui pendant le dash en lance un nouveau dès que `cut_from` frames du dash en
    /// cours ont été jouées (le reste du dash est coupé). Absent : on attend la fin et le cooldown.
    #[serde(default)]
    pub cut_from: Option<u32>,
    /// P-1 : frames après la fin d'un dash pendant lesquelles le suivant continue l'enchaînement
    /// (compteur `combat::actors::DashCadence::chain`). Absent : pas de compteur.
    #[serde(default)]
    pub chain_window_frames: Option<u32>,
    /// P-1 : jugement de l'appui contre un tempo et bonus d'un appui sur le temps.
    #[serde(default)]
    pub on_beat: Option<DashOnBeat>,
}

impl DashConfig {
    /// Le personnage a-t-il besoin de l'état de cadence (`DashCadence`) ? Seulement si un champ
    /// P-1 qui en lit ou en écrit est posé : sans eux, rien n'est ajouté (traces inchangées).
    pub fn needs_cadence(&self) -> bool {
        self.on_beat.is_some() || self.chain_window_frames.is_some()
    }
}
