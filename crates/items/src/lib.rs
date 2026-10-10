//! Objets (M2-T0b, chantier C2, `docs/conventions.md` §36) : contrats sans dépendance au rendu.
//!
//! - [`ItemDef`] : un objet de `items/*.ron` (kind de contenu `Item`) : genre ([`ItemKind`]),
//!   modificateurs de stats, [`effects::Effect`] (réutilisés tels quels), tags, rareté.
//! - [`ActiveCharge`] : comment un objet actif se recharge (salles nettoyées, dégâts infligés,
//!   frames).
//! - [`Inventory`] : composant rollback **neutre** du joueur (posé au premier ramassage, absent
//!   de tout joueur qui n'a rien ramassé : contribution 0 au checksum).
//! - [`ItemPickup`] : objet au sol (rollback, `GgrsNetId`).
//! - [`ItemTable`] : les objets du jeu par identifiant (contenu, hors rollback).
//!
//! Les systèmes (ramassage, charge, usage) vivent dans `game::items`.

use bevy::prelude::*;
use bevy_fixed::fixed_math::Fixed;
use effects::Effect;
use serde::{Deserialize, Serialize};
use sim_core::modifier::{Modifier, ModifierOp, ModifierSource};
use sim_core::stats::StatId;
use std::collections::BTreeMap;

/// Comment un objet actif se recharge. `n` est la charge requise : l'actif est prêt quand sa
/// charge atteint `n`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActiveCharge {
    /// `n` salles nettoyées (`world::RoomChanged` vers `Cleared`, M2-E1).
    Rooms(u32),
    /// `n` points de dégâts infligés par le porteur (`DamageEvent`, arrondi à l'entier inférieur).
    Damage(u32),
    /// `n` frames écoulées.
    Frames(u32),
}

impl ActiveCharge {
    /// Charge requise (toujours > 0 pour un objet valide, voir le lint).
    pub fn required(&self) -> u32 {
        match self {
            Self::Rooms(n) | Self::Damage(n) | Self::Frames(n) => *n,
        }
    }
}

/// Genre d'un objet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemKind {
    /// Effet permanent tant qu'il est tenu (modificateurs, effets).
    Passive,
    /// Un seul emplacement actif ; utilisé par `UseActive`, se recharge selon `charge`.
    Active(ActiveCharge),
    /// Compteur par identifiant (`key`, `blank`, `shell`…), ramassé au contact.
    Consumable,
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// Modificateur d'un objet : un [`Modifier`] sans source ni expiration (la source est
/// `item:<id>`, comme `powerup:<id>`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemModifier {
    pub stat: StatId,
    pub op: ModifierOp,
    pub value: Fixed,
}

/// Source des modificateurs d'un objet.
pub fn item_source(id: &str) -> ModifierSource {
    ModifierSource::Named(format!("item:{id}"))
}

impl ItemModifier {
    /// Le [`Modifier`] permanent correspondant, de source `item:<id>`.
    pub fn to_modifier(&self, id: &str) -> Modifier {
        Modifier {
            stat: self.stat.clone(),
            op: self.op,
            value: self.value,
            source: item_source(id),
            until: None,
        }
    }
}

fn default_pickup_range() -> Fixed {
    Fixed::from_num(24)
}

/// Un objet de `items/<id>.ron` (id = nom de fichier).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemDef {
    pub name: String,
    pub kind: ItemKind,
    /// Modificateurs permanents posés au ramassage d'un passif (retirés s'il est perdu).
    #[serde(default)]
    pub modifiers: Vec<ItemModifier>,
    /// Effets (T1.10). T0b n'exécute que `OnUse` d'un actif (`TimedModifier`, `Modifier`) ;
    /// le reste est refusé par le lint jusqu'aux effets v2.
    #[serde(default)]
    pub effects: Vec<Effect>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub rarity: Rarity,
    /// Portée de ramassage au contact d'un consommable, ou d'interaction (unités monde).
    #[serde(default = "default_pickup_range")]
    pub pickup_range: Fixed,
}

/// Les objets du jeu par identifiant. Hors rollback (donnée de contenu), posée au chargement
/// de la carte depuis le registre (`map_ldtk::loader::item_table`).
#[derive(Resource, Clone, Debug, Default)]
pub struct ItemTable(pub BTreeMap<String, ItemDef>);

/// Emplacement actif d'un joueur : l'objet tenu et sa charge courante (0..=requise).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ActiveSlot {
    pub item: String,
    pub charge: u32,
}

/// Inventaire d'un joueur (rollback, checksum et trace en variante **neutre**). Posé au premier
/// ramassage seulement : un joueur qui n'a jamais rien ramassé n'en porte pas.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Inventory {
    /// Passifs, dans l'ordre de ramassage.
    pub passives: Vec<String>,
    pub active: Option<ActiveSlot>,
    /// Consommables par identifiant (`key`, `blank`, `shell`…).
    pub consumables: BTreeMap<String, u32>,
}

impl Inventory {
    pub fn has(&self, id: &str) -> bool {
        self.passives.iter().any(|p| p == id) || self.active.as_ref().is_some_and(|a| a.item == id)
    }

    pub fn consumable(&self, id: &str) -> u32 {
        self.consumables.get(id).copied().unwrap_or(0)
    }
}

/// Objet au sol (rollback, neutre, `GgrsNetId`). Les passifs et actifs portent en plus un
/// `Interactable` de type `Item` ; un consommable se ramasse au contact.
#[derive(Component, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemPickup {
    pub item_id: String,
}

/// Un objet ramassé (`FrameEvents`, neutre) : moment clé `item_pickup`, récompenses à venir.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ItemPicked {
    pub frame: u32,
    pub player_handle: usize,
    pub item_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_def_ron() {
        let def: ItemDef = ron::from_str(
            r#"(name: "Bottes", kind: Passive,
                modifiers: [(stat: MoveSpeed, op: Pct, value: "0.25")], tags: ["mobilite"])"#,
        )
        .unwrap();
        assert_eq!(def.kind, ItemKind::Passive);
        assert_eq!(def.rarity, Rarity::Common);
        assert_eq!(def.pickup_range, Fixed::from_num(24));
        assert_eq!(def.modifiers[0].to_modifier("bottes").until, None);
        let active: ItemDef = ron::from_str(
            r#"(name: "Fiole", kind: Active(Rooms(2)),
                effects: [(on: OnUse, do: [TimedModifier(stat: Damage, op: Pct, value: "1.0", frames: 300)])])"#,
        )
        .unwrap();
        assert_eq!(active.kind, ItemKind::Active(ActiveCharge::Rooms(2)));
        assert_eq!(ActiveCharge::Rooms(2).required(), 2);
    }

    #[test]
    fn inventaire_par_defaut_vide() {
        let inventory = Inventory::default();
        assert!(!inventory.has("x"));
        assert_eq!(inventory.consumable("key"), 0);
    }
}
