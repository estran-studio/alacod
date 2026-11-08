//! Contrat de donnée du « feu follet » d'un personnage (`wisp: Some(..)` de son RON, M-5,
//! `docs/conventions.md` §113).
//!
//! Un joueur qui tombe à terre (`combat::downed::Downed`, coop seulement) reste, dans ce mode,
//! une **lumière** qui erre : il se déplace, ne frappe pas, ne prend pas de dégâts (l'état à terre
//! l'interdit déjà), peut **leurrer** les ennemis et se relève soit par un coéquipier (interaction
//! tenue, `revive_frames` du personnage), soit à un point de reprise (`combat::downed::RespawnPoint`).
//! Types de **donnée** seulement : ils vivent ici pour que `content` (lint, schéma) et `game`
//! partagent **un seul** format. Aucun nombre n'a de défaut codé dans le moteur sauf ceux, neutres,
//! de [`WispDef::default`] qui ne s'appliquent qu'à un champ omis du RON.

use bevy_fixed::fixed_math::{self, Fixed};
use serde::{Deserialize, Serialize};

/// Comment un feu follet détourne les ennemis (champ `lure` de [`WispDef`]). Enum fermé, nouvelles
/// variantes **en fin d'enum**. `range` est une portée de **chemin** en unités monde (la même que
/// les cases du champ de flux : 16 unités par case, coût 10 par case orthogonale).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum WispLure {
    /// **Attire** : dans `range`, le feu follet est une cible comme un joueur debout ; chaque
    /// ennemi va vers la plus proche (par le chemin) des cibles.
    Attract {
        #[cfg_attr(feature = "schema", schemars(with = "bevy_fixed::schema::FixedString"))]
        #[cfg_attr(feature = "schema", schemars(extend("x-lint" = "> 0")))]
        range: Fixed,
    },
    /// **Égare** : dans `range`, les ennemis ne vont que vers le feu follet, même quand un joueur
    /// debout est plus près ; hors de `range`, ils font comme sans lui.
    Mislead {
        #[cfg_attr(feature = "schema", schemars(with = "bevy_fixed::schema::FixedString"))]
        #[cfg_attr(feature = "schema", schemars(extend("x-lint" = "> 0")))]
        range: Fixed,
    },
}

impl WispLure {
    /// Portée de chemin, en unités monde.
    pub fn range(&self) -> Fixed {
        match self {
            Self::Attract { range } | Self::Mislead { range } => *range,
        }
    }
}

/// Le mode « feu follet » d'un personnage joueur. Absent du RON : `Downed` seul, comme avant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct WispDef {
    /// Multiplicateur de la vitesse de déplacement (stat `MoveSpeed`, `Mul`) ; remplace
    /// `downed_speed_mult` du personnage tant qu'il est feu follet. Dans `]0, 4]`.
    #[serde(default = "default_speed_mult")]
    #[cfg_attr(feature = "schema", schemars(with = "bevy_fixed::schema::FixedString"))]
    #[cfg_attr(feature = "schema", schemars(extend("x-lint" = "]0, 4]")))]
    pub speed_mult: Fixed,
    /// Leurre des ennemis. `None` : les ennemis l'ignorent, comme un joueur à terre ordinaire.
    #[serde(default)]
    pub lure: Option<WispLure>,
    /// Frames avant la mort par saignement. `None` : le feu follet dure jusqu'à sa relève ;
    /// `Some(n)` remplace `CharacterConfig::bleedout_frames` (`> 0`).
    #[serde(default)]
    #[cfg_attr(feature = "schema", schemars(extend("x-lint" = "> 0")))]
    pub bleedout_frames: Option<u32>,
    /// Frames passées **dans la portée d'un point de reprise** pour se relever sans coéquipier
    /// (`> 0`). `None` : seule la relève par un coéquipier existe.
    #[serde(default)]
    #[cfg_attr(feature = "schema", schemars(extend("x-lint" = "> 0")))]
    pub respawn_frames: Option<u32>,
    /// Fraction de `Health.max` rendue à la reprise, dans `]0, 1]`.
    #[serde(default = "default_respawn_health")]
    #[cfg_attr(feature = "schema", schemars(with = "bevy_fixed::schema::FixedString"))]
    #[cfg_attr(feature = "schema", schemars(extend("x-lint" = "]0, 1]")))]
    pub respawn_health: Fixed,
}

fn default_speed_mult() -> Fixed {
    fixed_math::new(1.0)
}

fn default_respawn_health() -> Fixed {
    fixed_math::new(0.5)
}

impl Default for WispDef {
    fn default() -> Self {
        Self {
            speed_mult: default_speed_mult(),
            lure: None,
            bleedout_frames: None,
            respawn_frames: None,
            respawn_health: default_respawn_health(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_champ_omis_prend_son_defaut_neutre() {
        let def: WispDef = ron::from_str("(lure: Some(Attract(range: \"96.0\")))").unwrap();
        assert_eq!(def.speed_mult, fixed_math::new(1.0));
        assert_eq!(def.bleedout_frames, None);
        assert_eq!(def.respawn_frames, None);
        assert_eq!(def.respawn_health, fixed_math::new(0.5));
        assert_eq!(def.lure.unwrap().range(), fixed_math::new(96.0));
    }
}
