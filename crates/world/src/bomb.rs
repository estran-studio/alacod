//! Bombe du joueur (M3-T0c contrat, M3-T4 exécution, `docs/conventions.md` §62 et §69).
//!
//! - [`BombDef`] : définition en données dans le personnage (`bomb: Some((...))`) ; absente, le
//!   bouton `Bomb` (bit `INPUT_BOMB`) n'a aucun effet (aucun contenu existant n'en a).
//! - [`Bomb`] : composant rollback **neutre** de la bombe posée (aucune hors partie à bombes).
//! - [`BombCarrier`] : définition et mémoire du bouton, posée sur les seuls porteurs de bombe.
//! - [`ExplosionRequest`] : demande d'explosion de la frame (bombe à mèche consumée, baril mort,
//!   action `Explode`), exécutée par `game::bombs::explosion_system`.

use bevy::prelude::*;
use bevy_fixed::fixed_math::Fixed;
use serde::{Deserialize, Serialize};
use sim_core::damage::FriendlyFire;
use utils::net_id::GgrsNetId;

/// Définition d'une bombe, lue dans le personnage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BombDef {
    /// Frames entre la pose et l'explosion (> 0).
    #[cfg_attr(feature = "schema", schemars(extend("x-lint" = "> 0")))]
    pub fuse_frames: u32,
    /// Rayon de l'explosion (> 0).
    #[cfg_attr(feature = "schema", schemars(with = "bevy_fixed::schema::FixedString"))]
    #[cfg_attr(feature = "schema", schemars(extend("x-lint" = "> 0")))]
    pub radius: Fixed,
    /// Dégâts aux personnages du rayon (>= 0).
    #[cfg_attr(feature = "schema", schemars(with = "bevy_fixed::schema::FixedString"))]
    #[cfg_attr(feature = "schema", schemars(extend("x-lint" = ">= 0")))]
    pub damage: Fixed,
    /// Tir ami de l'explosion (Q4 : proposition `Always`, comme Isaac, la bombe blesse aussi son
    /// poseur et les autres joueurs). Fin de structure : absent des définitions d'avant.
    #[serde(default = "default_bomb_friendly_fire")]
    pub friendly_fire: FriendlyFire,
}

fn default_bomb_friendly_fire() -> FriendlyFire {
    FriendlyFire::Always
}

/// Une bombe posée. Rollback + trace en variante neutre : portée par aucune entité tant que
/// M3-T4 n'a pas livré la pose.
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Bomb {
    /// Le joueur qui l'a posée.
    pub owner: GgrsNetId,
    /// Frame (absolue) à laquelle elle explose.
    pub fuse_until: u32,
    pub radius: Fixed,
    pub damage: Fixed,
    pub friendly_fire: FriendlyFire,
    /// Équipe du poseur (équipe de la source des dégâts).
    pub team: sim_core::team::Team,
    /// Casse rochers, terrain et murs secrets du rayon (vrai pour une bombe de joueur).
    pub destroys: bool,
}

/// Un joueur qui porte des bombes : sa définition (copie de `CharacterConfig.bomb`, ou imposée par
/// un scénario) et la mémoire du bouton (`held` : front montant, un bouton tenu ne pose qu'une
/// bombe). Rollback **neutre** : posée seulement sur un joueur dont le personnage a `bomb` ;
/// `held` est réécrit à chaque frame, avant tout retour anticipé (piège « donnée dérivée non
/// rollbackée »).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BombCarrier {
    pub def: BombDef,
    pub held: bool,
}

impl BombCarrier {
    pub fn new(def: BombDef) -> Self {
        Self { def, held: false }
    }
}

impl Bomb {
    /// Bombe posée à `frame` par `owner` selon `def`.
    pub fn placed(owner: GgrsNetId, frame: u32, def: &BombDef) -> Self {
        Self {
            owner,
            fuse_until: frame.saturating_add(def.fuse_frames),
            radius: def.radius,
            damage: def.damage,
            friendly_fire: def.friendly_fire,
            team: sim_core::team::Team::Players,
            destroys: true,
        }
    }

    /// La mèche est consumée à `frame`.
    pub fn is_due(&self, frame: u32) -> bool {
        frame >= self.fuse_until
    }
}

/// Le bouton `Bomb` est-il une demande de pose ? Seulement si le personnage a une définition
/// de bombe : sans `bomb`, l'input est ignoré (contrat M3-T0c).
pub fn bomb_requested(buttons: u32, def: Option<&BombDef>, bit: u32) -> bool {
    def.is_some() && buttons & bit != 0
}

/// Demande d'explosion de la frame (`FrameEvents` **transitoire**, hors checksum : un baril émet
/// la sienne dans des parties existantes, une file neutre non vide déplacerait leurs traces).
/// Position en champs séparés (`FixedVec2` n'est pas `Hash`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplosionRequest {
    /// `GgrsNetId.0` de l'objet qui explose (bombe, baril, porteur de l'action) : clé de tri des
    /// explosions d'une frame.
    pub id: usize,
    /// Crédité des dégâts (le poseur, ou le dernier joueur qui a touché le baril).
    pub source: GgrsNetId,
    pub source_team: sim_core::team::Team,
    pub x: Fixed,
    pub y: Fixed,
    pub radius: Fixed,
    pub damage: Fixed,
    /// Casse aussi les rochers et le terrain du rayon, ouvre les murs secrets.
    pub destroys: bool,
    pub friendly_fire: FriendlyFire,
}

/// Une explosion vient d'avoir lieu (`FrameEvents` **transitoire**) : moment clé `explosion` et
/// lecture du mur secret dans la même frame. Position en champs séparés.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exploded {
    pub frame: u32,
    pub x: Fixed,
    pub y: Fixed,
    pub radius: Fixed,
    pub destroys: bool,
}

/// Problèmes d'une définition (lint) ; vide si elle est valide.
pub fn bomb_problems(def: &BombDef) -> Vec<String> {
    let mut out = Vec::new();
    if def.fuse_frames == 0 {
        out.push("bomb : fuse_frames = 0 : doit être > 0".to_string());
    }
    if def.radius <= Fixed::ZERO {
        out.push(format!("bomb : rayon = {} : doit être > 0", def.radius));
    }
    if def.damage < Fixed::ZERO {
        out.push(format!(
            "bomb : dégâts = {} : doivent être >= 0",
            def.damage
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def() -> BombDef {
        BombDef {
            fuse_frames: 90,
            radius: Fixed::from_num(40),
            damage: Fixed::from_num(10),
            friendly_fire: FriendlyFire::Always,
        }
    }

    #[test]
    fn meche_datee_de_la_pose() {
        let bomb = Bomb::placed(GgrsNetId(1, "player".into()), 100, &def());
        assert_eq!(bomb.fuse_until, 190);
        assert!(!bomb.is_due(189));
        assert!(bomb.is_due(190));
    }

    #[test]
    fn sans_definition_le_bouton_est_sans_effet() {
        let bit = 1 << 18;
        assert!(!bomb_requested(bit, None, bit));
        assert!(!bomb_requested(0, Some(&def()), bit));
        assert!(bomb_requested(bit | 1, Some(&def()), bit));
    }

    #[test]
    fn lint_de_la_definition() {
        assert!(bomb_problems(&def()).is_empty());
        let bad = BombDef {
            fuse_frames: 0,
            radius: Fixed::ZERO,
            damage: Fixed::from_num(-1),
            friendly_fire: FriendlyFire::Never,
        };
        assert_eq!(bomb_problems(&bad).len(), 3);
    }
}
