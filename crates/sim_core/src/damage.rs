//! Genres de dégâts et événement de dégât, destinés à circuler dans
//! [`crate::frame_events::FrameEvents<DamageEvent>`].
//!
//! `DamageEvent` identifie source et cible par [`GgrsNetId`], jamais par `Entity`
//! (CLAUDE.md, règle 3 : les `Entity` diffèrent d'un client à l'autre et ne sont pas
//! comparables entre clients).
//!
//! # Pas encore branché
//!
//! Le type est défini ici (T0.2) mais **rien ne l'émet ni ne l'enregistre**.
//! [`add_damage_events`] existe pour que T1.1 (chantier B1, « Équipes et dégâts ») puisse
//! l'appeler quand l'émission et la lecture des dégâts seront branchées. L'appeler
//! aujourd'hui ne casserait rien à la compilation, mais ajouterait
//! `FrameEvents<DamageEvent>` (une ressource vide, jamais écrite) au rollback — et donc au
//! checksum GGRS de chaque frame — ce qui changerait `tests/scenarios/*.trace` sans
//! aucune raison de gameplay. Ne pas l'appeler avant T1.1.

use bevy::app::App;
use bevy_fixed::fixed_math::Fixed;
use serde::{Deserialize, Serialize};
use utils::net_id::GgrsNetId;

use crate::frame_events::FrameEventsAppExt;

/// Genre de dégât. Enum ouvert (`Custom`) pour que le contenu (RON) puisse définir des
/// genres propres à un jeu (ex. `Benit`/`Maudit` pour 1837) sans toucher à l'engine.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DamageKind {
    Physical,
    Fire,
    Ice,
    Poison,
    Electric,
    Explosion,
    Blessed,
    Cursed,
    /// Ignore résistances/armure/invulnérabilité (dégât de test, environnement...).
    True,
    Custom(String),
}

/// Un dégât infligé pendant une frame de simulation. Consommé par les systèmes ordonnés
/// après l'émetteur, dans la même frame (voir [`crate::frame_events::FrameEvents`]).
#[derive(Debug, Clone, Hash)]
pub struct DamageEvent {
    pub source: GgrsNetId,
    pub target: GgrsNetId,
    pub kind: DamageKind,
    pub amount: Fixed,
    pub frame: u32,
}

/// Enregistre `FrameEvents<DamageEvent>` (rollback + trace + vidage en
/// `RollbackSystemSet::FrameStart`, voir [`crate::frame_events`]). **Non appelée** par
/// `sim_core` : c'est à l'appelant (T1.1, chantier B1) de le faire une fois l'émission des
/// dégâts branchée dans `GgrsSchedule`.
pub fn add_damage_events(app: &mut App) -> &mut App {
    app.add_frame_events::<DamageEvent>()
}
