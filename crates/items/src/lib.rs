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

pub mod loot;
pub use loot::*;

use bevy::prelude::*;
use bevy_fixed::fixed_math::Fixed;
use effects::projectile::ProjectileMods;
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

/// Effet d'un consommable `blank` (M2-T1, `docs/conventions.md` §41) : l'appui sur `Blank`
/// consomme un exemplaire, efface les balles ennemies dans `radius` (unités monde) autour du
/// joueur et le rend invulnérable `iframes` frames (0 : aucune).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlankSpec {
    pub radius: Fixed,
    #[serde(default)]
    pub iframes: u32,
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
    /// M2-T1 : le consommable est un **blank** (voir [`BlankSpec`]) ; le blank utilisé est le
    /// premier objet de la table (ordre des ids) qui porte cette donnée et que le joueur possède.
    #[serde(default)]
    pub blank: Option<BlankSpec>,
    /// M2-T4 : un actif utilisable au plus `n` fois par étage du mode `Floors` (hors `Floors`,
    /// par partie) ; `None` : sans limite (v1 exact).
    #[serde(default)]
    pub uses_per_floor: Option<u32>,
    /// M2-T4 : un actif qui consomme `n` exemplaires du consommable `item` à l'usage ; sans le
    /// stock, l'usage est refusé (la charge reste). `None` : gratuit (v1 exact).
    #[serde(default)]
    pub cost: Option<(String, u32)>,
    /// M4-T0a (`docs/conventions.md` §60) : modificateurs de projectile ajoutés aux tirs du
    /// porteur (`modifiers`, `on_hit`, `on_expire`), composés avec ceux de l'arme par
    /// `combat::projectile::compose` dans l'ordre de [`projectile_sources`]. `None` : v1 exacte.
    /// Contrat : le tir ne le lit pas encore (M4-T1) ; le lint refuse un objet qui le porte
    /// (`Unsupported`). `ItemDef` est une définition (ressource [`ItemTable`], hors rollback,
    /// sans `Hash`) : le champ n'entre dans aucun checksum.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projectile: Option<ProjectileMods>,
}

/// Une synergie (M2-T4, `docs/conventions.md` §44) : un fichier `synergies/<id>.ron` (kind
/// `Synergy`). Active tant que **tous** les objets de `requires` sont tenus (passifs ou actif) ;
/// ses `modifiers` ont la source `synergy:<id>`, ses `effects` la clé `synergy:<id>:<index>`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SynergyDef {
    pub name: String,
    pub requires: Vec<String>,
    #[serde(default)]
    pub modifiers: Vec<ItemModifier>,
    #[serde(default)]
    pub effects: Vec<Effect>,
    /// M4-T0a : comme [`ItemDef::projectile`], pour une synergie (une synergie peut changer les
    /// larmes). `None` : v1 exacte ; hors rollback et hors hash (définition).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projectile: Option<ProjectileMods>,
}

/// M4-T0a (§60) : listes de modificateurs de projectile des objets tenus par `inventory`, dans
/// l'ordre déterministe de la composition (`combat::projectile::compose`, après ceux de l'arme) :
/// passifs dans l'ordre de ramassage (un passif en double compte deux fois), puis l'actif, puis
/// les synergies actives par id croissant. Les objets sans `projectile` ne donnent rien. Pure :
/// M4-T1 l'appellera au tir.
pub fn projectile_sources<'a>(
    items: &'a ItemTable,
    synergies: &'a SynergyTable,
    inventory: &Inventory,
) -> Vec<&'a [effects::projectile::ProjectileModifier]> {
    let item_mods = |id: &str| {
        items
            .0
            .get(id)
            .and_then(|def| def.projectile.as_ref())
            .map(|mods| mods.modifiers.as_slice())
    };
    let mut out: Vec<&[effects::projectile::ProjectileModifier]> = Vec::new();
    out.extend(inventory.passives.iter().filter_map(|id| item_mods(id)));
    if let Some(active) = &inventory.active {
        out.extend(item_mods(&active.item));
    }
    out.extend(
        synergies
            .active(inventory)
            .into_iter()
            .filter_map(|(_, def)| def.projectile.as_ref())
            .map(|mods| mods.modifiers.as_slice()),
    );
    out
}

/// Source des modificateurs d'une synergie.
pub fn synergy_source(id: &str) -> ModifierSource {
    ModifierSource::Named(format!("synergy:{id}"))
}

/// Les synergies du jeu par identifiant (hors rollback, comme [`ItemTable`]).
#[derive(Resource, Clone, Debug, Default)]
pub struct SynergyTable(pub BTreeMap<String, SynergyDef>);

impl SynergyTable {
    /// Les synergies actives pour `inventory`, par identifiant croissant : tous les objets
    /// requis sont tenus.
    pub fn active<'a>(&'a self, inventory: &Inventory) -> Vec<(&'a str, &'a SynergyDef)> {
        self.0
            .iter()
            .filter(|(_, def)| {
                !def.requires.is_empty() && def.requires.iter().all(|id| inventory.has(id))
            })
            .map(|(id, def)| (id.as_str(), def))
            .collect()
    }
}

/// Les objets du jeu par identifiant. Hors rollback (donnée de contenu), posée au chargement
/// de la carte depuis le registre (`map_ldtk::loader::item_table`).
#[derive(Resource, Clone, Debug, Default)]
pub struct ItemTable(pub BTreeMap<String, ItemDef>);

/// Emplacement actif d'un joueur : l'objet tenu et sa charge courante (0..=requise).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveSlot {
    pub item: String,
    pub charge: u32,
    /// M2-T4 : `(étage, usages)` : usages de l'actif à cet étage (`ItemDef::uses_per_floor`).
    /// `None` tant que la limite n'a pas servi.
    pub uses: Option<(u32, u32)>,
}

impl ActiveSlot {
    pub fn new(item: String) -> Self {
        Self {
            item,
            charge: 0,
            uses: None,
        }
    }

    /// Usages à l'étage `floor` (ceux d'un autre étage ne comptent plus).
    pub fn uses_on(&self, floor: u32) -> u32 {
        match self.uses {
            Some((f, n)) if f == floor => n,
            _ => 0,
        }
    }
}

/// `uses` n'entre dans le hash que s'il est posé : le hash d'un actif v1 est celui d'avant
/// M2-T4 (traces inchangées, `docs/conventions.md` §44).
impl std::hash::Hash for ActiveSlot {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use std::hash::Hash as _;
        self.item.hash(state);
        self.charge.hash(state);
        if let Some(uses) = self.uses {
            uses.hash(state);
        }
    }
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

/// Un blank utilisé (`FrameEvents`, neutre) : lu par la présentation (onde) et, plus tard, par
/// `OnBlank` des effets (M2-T3).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlankUsed {
    pub frame: u32,
    pub player_handle: usize,
    pub x: Fixed,
    pub y: Fixed,
    pub radius: Fixed,
    /// Balles ennemies effacées.
    pub cleared: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_of<T: std::hash::Hash>(value: &T) -> u64 {
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    /// M4-T0a : `projectile` absent = v1 exacte ; présent, lu tel quel ; sources de la
    /// composition dans l'ordre passifs (ramassage, doublons compris), actif, synergies par id.
    #[test]
    fn projectile_d_objet_et_ordre_des_sources() {
        use effects::projectile::ProjectileModifier::*;
        let plain: ItemDef = ron::from_str(r#"(name: "Rien", kind: Passive)"#).unwrap();
        assert_eq!(plain.projectile, None);
        let parse = |src: &str| -> ItemDef { ron::from_str(src).unwrap() };
        let table = ItemTable(BTreeMap::from([
            (
                "rebond".to_string(),
                parse(r#"(name: "R", kind: Passive, projectile: Some((modifiers: [Bounce(1)])))"#),
            ),
            (
                "grosse".to_string(),
                parse(r#"(name: "G", kind: Passive, projectile: Some((modifiers: [Size("1.5")])))"#),
            ),
            (
                "actif".to_string(),
                parse(r#"(name: "A", kind: Active(Frames(10)), projectile: Some((modifiers: [Spectral])))"#),
            ),
            ("rien".to_string(), plain),
        ]));
        let synergy: SynergyDef = ron::from_str(
            r#"(name: "S", requires: ["rebond", "grosse"], projectile: Some((modifiers: [Pierce(2)])))"#,
        )
        .unwrap();
        let synergies = SynergyTable(BTreeMap::from([("s".to_string(), synergy)]));
        let inventory = Inventory {
            passives: vec![
                "grosse".into(),
                "rien".into(),
                "rebond".into(),
                "grosse".into(),
            ],
            active: Some(ActiveSlot::new("actif".into())),
            consumables: BTreeMap::new(),
        };
        let sources = projectile_sources(&table, &synergies, &inventory);
        assert_eq!(
            sources,
            vec![
                &[Size(Fixed::from_num(1.5))][..],
                &[Bounce(1)][..],
                &[Size(Fixed::from_num(1.5))][..],
                &[Spectral][..],
                &[Pierce(2)][..],
            ]
        );
        assert!(projectile_sources(&table, &synergies, &Inventory::default()).is_empty());
    }

    #[test]
    fn actif_sans_usage_garde_le_hash_d_avant_m2_t4() {
        // Équivalent de l'ancien `#[derive(Hash)]` sur `{ item, charge }`
        #[derive(Hash)]
        struct Before {
            item: String,
            charge: u32,
        }
        let slot = ActiveSlot::new("fiole".into());
        let before = Before {
            item: "fiole".into(),
            charge: 0,
        };
        assert_eq!(hash_of(&slot), hash_of(&before));
        let mut used = slot.clone();
        used.uses = Some((0, 1));
        assert_ne!(hash_of(&used), hash_of(&before));
    }

    #[test]
    fn usages_d_un_autre_etage_ne_comptent_plus() {
        let mut slot = ActiveSlot::new("eclat".into());
        assert_eq!(slot.uses_on(0), 0);
        slot.uses = Some((0, 2));
        assert_eq!(slot.uses_on(0), 2);
        assert_eq!(slot.uses_on(1), 0);
    }

    fn synergies() -> SynergyTable {
        let ron = |requires: &str| -> SynergyDef {
            ron::from_str(&format!(r#"(name: "S", requires: {requires})"#)).unwrap()
        };
        SynergyTable(
            [
                ("b_paire".to_string(), ron(r#"["lame", "manche"]"#)),
                ("a_trio".to_string(), ron(r#"["lame", "manche", "braise"]"#)),
                ("c_actif".to_string(), ron(r#"["lame", "braise"]"#)),
            ]
            .into(),
        )
    }

    fn inventory(passives: &[&str], active: Option<&str>) -> Inventory {
        Inventory {
            passives: passives.iter().map(|p| p.to_string()).collect(),
            active: active.map(|a| ActiveSlot::new(a.into())),
            consumables: BTreeMap::new(),
        }
    }

    #[test]
    fn synergies_actives_par_id_quand_tout_est_tenu() {
        let table = synergies();
        let ids = |inv: &Inventory| {
            table
                .active(inv)
                .into_iter()
                .map(|(id, _)| id)
                .collect::<Vec<_>>()
        };
        assert!(ids(&inventory(&["lame"], None)).is_empty());
        assert_eq!(ids(&inventory(&["lame", "manche"], None)), vec!["b_paire"]);
        // L'actif compte comme un objet tenu
        assert_eq!(ids(&inventory(&["lame"], Some("braise"))), vec!["c_actif"]);
        assert_eq!(
            ids(&inventory(&["lame", "manche"], Some("braise"))),
            vec!["a_trio", "b_paire", "c_actif"]
        );
        // L'actif remplacé retire les synergies qui en dépendaient
        assert_eq!(
            ids(&inventory(&["lame", "manche"], Some("fiole"))),
            vec!["b_paire"]
        );
    }

    #[test]
    fn synergie_sans_objet_requis_n_est_jamais_active() {
        let table = SynergyTable(
            [(
                "vide".to_string(),
                ron::from_str::<SynergyDef>(r#"(name: "V", requires: [])"#).unwrap(),
            )]
            .into(),
        );
        assert!(table.active(&inventory(&["lame"], None)).is_empty());
    }

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
