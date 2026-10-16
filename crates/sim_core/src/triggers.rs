//! Événements de joueur lus par les déclencheurs d'effets v2 (M2-T3, `docs/conventions.md` §42).
//!
//! Émis par les systèmes qui possèdent le fait (dash, rechargement) dans des
//! `FrameEvents` **transitoires** (`add_frame_events_transient` : hors checksum et hors snapshot,
//! vidés au `FrameStart`), lus dans la même frame par
//! `game::effects_runtime::apply_triggered_effects_system` (ordonné après l'émetteur).
//! Le joueur est désigné par son handle GGRS.

/// Un joueur commence un dash (`OnDodge`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DashStarted {
    pub frame: u32,
    pub player_handle: usize,
}

/// Un joueur finit de recharger son arme (`OnReload`) : le chargeur vient d'être rempli.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ReloadFinished {
    pub frame: u32,
    pub player_handle: usize,
}

/// Un joueur vient de tirer avec son arme active (M3-T6, `MirrorShot`, §71) : émis par
/// `combat::weapons::weapon_rollback_system` à chaque tir **réellement parti** (cadence respectée,
/// chargeur non vide), jamais pour une gâchette tenue que la cadence ou le chargeur refuse. Lu dans
/// la même frame par `game::familiars` (après l'émetteur).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WeaponFired {
    pub frame: u32,
    pub player_handle: usize,
}
