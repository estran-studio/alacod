//! Genres de dégâts et événement de dégât, destinés à circuler dans
//! [`crate::frame_events::FrameEvents<DamageEvent>`].
//!
//! `DamageEvent` identifie source et cible par [`GgrsNetId`], jamais par `Entity`
//! (CLAUDE.md, règle 3 : les `Entity` diffèrent d'un client à l'autre et ne sont pas
//! comparables entre clients).
//!
//! # Branché depuis T1.1
//!
//! [`add_damage_events`] est appelé par `game::character::BaseCharacterGamePlugin` (T1.1,
//! chantier B1, « Équipes et dégâts »). Les trois émetteurs (collision de balles
//! `weapons::bullet_rollback_collision_system`, collision de mêlée
//! `weapons::melee::melee_hitbox_collision_system`, attaque d'ennemi
//! `character::enemy::ai::behavior::enemy_attack_damage_translate_system`) construisent un
//! `DamageEvent` ; un seul système (`character::health::rollback_resolve_damage_events`,
//! `RollbackSystemSet::CollisionDamage`) les lit dans l'ordre d'émission et applique les
//! règles de `combat::damage::resolve_damage` (équipe, tir ami, tags, résistances,
//! immunités, invulnérabilité).

use bevy::app::App;
use bevy_fixed::fixed_math::Fixed;
use serde::{Deserialize, Serialize};
use utils::net_id::GgrsNetId;

use crate::frame_events::FrameEventsAppExt;
use crate::tag::Tags;
use crate::team::Team;

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

/// Politique de tir ami d'une arme ou d'une attaque (`WeaponConfig`/`MeleeWeaponConfig`/
/// `EnemyAiConfig`, RON, `#[serde(default)]` = [`FriendlyFire::Never`]). Contrat partagé
/// (comme [`DamageKind`]) : les règles qui l'interprètent (même équipe, polarité `cursed`)
/// vivent dans `combat::team::team_allows_hit`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum FriendlyFire {
    /// Un allié (même bord : `Players` et `Allies` ensemble, ou deux `Enemies`) n'est
    /// jamais touché.
    #[default]
    Never,
    /// Un allié est toujours touché, comme un adversaire.
    Always,
    /// Un allié n'est touché que si la source porte le tag `cursed` (polarité maudite).
    Cursed,
}

/// Un dégât infligé pendant une frame de simulation. Consommé par les systèmes ordonnés
/// après l'émetteur, dans la même frame (voir [`crate::frame_events::FrameEvents`]).
///
/// `tags`, `source_team`, `friendly_fire` (T1.1) : snapshot, au moment de l'émission, de ce
/// qu'il faut pour résoudre le coup (`combat::damage::resolve_damage`) sans avoir à
/// requêter à nouveau l'entité source (qui peut avoir disparu, ou juste pour éviter une
/// requête ECS supplémentaire au moment de la résolution). `tags` est l'union des tags du
/// personnage source (`CharacterConfig::tags`, ex. `cursed`) et d'un tag de genre d'attaque
/// posé par l'émetteur (`bullet`, `melee`, plus `zombie` pour une griffe de zombie puisque
/// ce tag vient du personnage) : il sert à la fois la politique `Cursed` et les
/// résistances/immunités par tag de la cible.
#[derive(Debug, Clone, Hash)]
pub struct DamageEvent {
    pub source: GgrsNetId,
    pub target: GgrsNetId,
    pub kind: DamageKind,
    pub amount: Fixed,
    pub frame: u32,
    pub tags: Tags,
    pub source_team: Team,
    pub friendly_fire: FriendlyFire,
}

/// Enregistre `FrameEvents<DamageEvent>` (rollback + trace + vidage en
/// `RollbackSystemSet::FrameStart`, voir [`crate::frame_events`]). **Non appelée** par
/// `sim_core` : c'est à l'appelant (T1.1, chantier B1) de le faire une fois l'émission des
/// dégâts branchée dans `GgrsSchedule`.
pub fn add_damage_events(app: &mut App) -> &mut App {
    app.add_frame_events::<DamageEvent>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn friendly_fire_round_trips_through_ron() {
        for policy in [
            FriendlyFire::Never,
            FriendlyFire::Always,
            FriendlyFire::Cursed,
        ] {
            let ron = ron::to_string(&policy).expect("sérialisation RON");
            let back: FriendlyFire = ron::from_str(&ron).expect("désérialisation RON");
            assert_eq!(policy, back);
        }
    }

    #[test]
    fn friendly_fire_defaults_to_never() {
        assert_eq!(FriendlyFire::default(), FriendlyFire::Never);
    }
}
