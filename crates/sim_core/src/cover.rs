//! Couvert (X-6a2, `docs/conventions.md` §118) : un personnage « prop » (M-1b, §112) qui arrête les
//! balles de **toutes** les équipes, quelle que soit l'équipe qu'il porte.
//!
//! Sans ce marquage, une balle ne touche un personnage que si `combat::team::team_allows_hit` le
//! permet : un prop sans équipe reçoit `Team::Enemies` (repli de `CharacterSpawn`), donc les balles
//! ennemies (même camp, tir ami `Never`) le traversent, et un prop `Team::Neutral` arrête les balles
//! mais n'est jamais blessé (impossible à détruire).
//!
//! # Composant statique, hors rollback, absent par défaut
//!
//! `Cover` est posé une fois au spawn du prop (`character::enemy::create::spawn_enemy`), **seulement**
//! si `CharacterConfig::cover` est présent : aucun contenu existant n'en porte, donc aucune trace ne
//! bouge. Comme [`crate::team::Team`] et [`crate::aim::AimMode`], il n'est pas enregistré en rollback
//! (jamais modifié, absent du checksum) et survit à un rollback tant que l'entité n'est détruite que
//! par `despawn_rollback()` (règle 9 de `CLAUDE.md`).

use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

/// Couvert : arrête les balles de toutes les équipes. Forme RON sur un personnage `prop: true` :
/// `cover: Some(Indestructible)` ou `cover: Some(Destructible)`.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Cover {
    /// Arrête toute balle sans jamais être blessé.
    Indestructible,
    /// Arrête toute balle et perd de la santé à chaque impact (`base_health` du personnage, dégât de
    /// la balle) ; cède, comme tout personnage, à santé épuisée.
    Destructible,
}

impl Cover {
    /// Un impact de n'importe quelle équipe blesse-t-il ce couvert ?
    pub fn takes_damage(self) -> bool {
        matches!(self, Cover::Destructible)
    }
}

/// Décor solide (C-3, `docs/conventions.md` §132) : un personnage « prop » dont le collider **bloque
/// les corps** (joueurs : collision dure avec glissement, comme un mur ; ennemis : pas refusé et cases
/// du champ de flux bloquées), en plus de la séparation et du ralentissement qu'un prop imposait déjà.
///
/// Même régime que [`Cover`] : composant statique posé une fois au spawn du prop
/// (`spawn_enemy`), **seulement** si `CharacterConfig::solid` est vrai ; hors rollback, absent du
/// checksum et de la trace (aucune trace existante ne bouge), il survit à un rollback tant que
/// l'entité n'est détruite que par `despawn_rollback()`. Un prop détruit (santé épuisée) cesse de
/// bloquer : l'entité est désactivée, donc absente des requêtes et des grilles de collision, et ses
/// cases quittent le champ de flux à sa prochaine reconstruction.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SolidBody;
