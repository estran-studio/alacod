pub mod loot;
pub use loot::*;
pub mod unlock;
pub use unlock::*;

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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActiveCharge {
    /// `n` salles nettoyées (`world::RoomChanged` vers `Cleared`, M2-E1).
    Rooms(#[cfg_attr(feature = "schema", schemars(extend("x-lint" = "> 0")))] u32),
    /// `n` points de dégâts infligés par le porteur (`DamageEvent`, arrondi à l'entier inférieur).
    Damage(#[cfg_attr(feature = "schema", schemars(extend("x-lint" = "> 0")))] u32),
    /// `n` frames écoulées.
    Frames(#[cfg_attr(feature = "schema", schemars(extend("x-lint" = "> 0")))] u32),
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemModifier {
    pub stat: StatId,
    pub op: ModifierOp,
    #[cfg_attr(feature = "schema", schemars(with = "bevy_fixed::schema::FixedString"))]
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlankSpec {
    #[cfg_attr(feature = "schema", schemars(with = "bevy_fixed::schema::FixedString"))]
    #[cfg_attr(feature = "schema", schemars(extend("x-lint" = "> 0")))]
    pub radius: Fixed,
    #[serde(default)]
    pub iframes: u32,
}

/// Un objet de `items/<id>.ron` (id = nom de fichier).
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemDef {
    pub name: String,
    pub kind: ItemKind,
    /// Modificateurs permanents posés au ramassage d'un passif (retirés s'il est perdu).
    #[serde(default)]
    pub modifiers: Vec<ItemModifier>,
    #[serde(default)]
    pub effects: Vec<Effect>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub rarity: Rarity,
    #[serde(default = "default_pickup_range")]
    #[cfg_attr(feature = "schema", schemars(with = "bevy_fixed::schema::FixedString"))]
    #[cfg_attr(feature = "schema", schemars(extend("x-lint" = "> 0")))]
    pub pickup_range: Fixed,
    #[serde(default)]
    pub blank: Option<BlankSpec>,
    #[serde(default)]
    #[cfg_attr(feature = "schema", schemars(extend("x-lint" = "> 0 (ou absent)")))]
    pub uses_per_floor: Option<u32>,
    #[serde(default)]
    pub cost: Option<(String, u32)>,
    /// M3-T0a (`docs/conventions.md` §60) : modificateurs de projectile ajoutés aux tirs du
    /// porteur (`modifiers`, `on_hit`, `on_expire`), composés avec ceux de l'arme par
    /// `combat::projectile::compose` dans l'ordre de [`projectile_mods`]. `None` : v1 exacte.
    /// Lu au tir par M3-T1 (§66) ; le lint vérifie sa forme (un objet n'a pas de table
    /// `projectiles` : `Split` sans `child`, `on_expire` sans `Spawn`). `ItemDef` est une
    /// définition (ressource [`ItemTable`], hors rollback, sans `Hash`) : le champ n'entre dans
    /// aucun checksum.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projectile: Option<ProjectileMods>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub familiar: Option<FamiliarRef>,
    /// M3-T0b : plafond d'un compteur consommable. Absent = illimité (exécution T3).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("x-lint" = "> 0 (ou absent)")))]
    pub max_stack: Option<u32>,
    /// M3-T2 (§67, Q16) : limites d'effet (`cooldown`, `charges`) partagées entre les exemplaires
    /// du même objet (défaut, comportement d'avant) ou propres à chaque exemplaire.
    #[serde(default, skip_serializing_if = "StackMode::is_shared")]
    pub stack: StackMode,
}

/// Comment deux exemplaires d'un même passif comptent leurs limites d'effet (M3-T2, Q16). Le
/// défaut `Shared` garde la clé `item:<id>:<index>` pour tous les exemplaires (aucune partie
/// existante ne change) ; `Independent` donne au k-ième exemplaire (k >= 1) la clé
/// `item:<id>#k:<index>` ([`item_effect_key`]), donc ses propres charges, cooldown et horloge `Tick`.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StackMode {
    #[default]
    Shared,
    Independent,
}

impl StackMode {
    pub fn is_shared(&self) -> bool {
        matches!(self, StackMode::Shared)
    }
}

/// Prix contractuel d'un présentoir (M3-T0b, §61). L'exécution vient de T8.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Price {
    Currency(u32),
    Consumable(String, u32),
    #[cfg_attr(feature = "schema", schemars(with = "bevy_fixed::schema::FixedString"))]
    MaxHealth(Fixed),
}

impl Price {
    /// Montant strictement positif exigé par le contrat (§61) : un prix nul n'a pas de sens
    /// (un présentoir gratuit est `Shelf.price == 0` sans `cost`).
    fn is_positive(&self) -> bool {
        match self {
            Self::Currency(amount) | Self::Consumable(_, amount) => *amount > 0,
            Self::MaxHealth(amount) => *amount > Fixed::ZERO,
        }
    }

    /// Problèmes de contenu d'un prix : montant nul, consommable inconnu, vie maximale hors
    /// `]0, max_health]`. Vide : prix valide. Pure ; `consumable_exists` dit si l'id est un
    /// objet consommable connu.
    pub fn problems(
        &self,
        consumable_exists: impl Fn(&str) -> bool,
        max_health: Fixed,
    ) -> Vec<String> {
        let mut out = Vec::new();
        if !self.is_positive() {
            out.push(format!("prix {self:?} : le montant doit être > 0"));
        }
        match self {
            Self::Consumable(id, _) if !consumable_exists(id) => {
                out.push(format!("prix {self:?} : consommable « {id} » inconnu"));
            }
            Self::MaxHealth(amount) if *amount > max_health => out.push(format!(
                "prix {self:?} : vie maximale {amount} > {max_health} (hors ]0, max])"
            )),
            _ => {}
        }
        out
    }

    /// Un prix nul n'est jamais payable : `can_pay` et `pay` restent cohérents.
    pub fn can_pay(
        &self,
        currency: u32,
        inventory: &Inventory,
        max_health: Fixed,
        health: Fixed,
    ) -> bool {
        if !self.is_positive() {
            return false;
        }
        match self {
            Self::Currency(amount) => currency >= *amount,
            Self::Consumable(id, amount) => inventory.consumable(id) >= *amount,
            Self::MaxHealth(amount) => *amount <= max_health && health > *amount,
        }
    }

    /// Paie si `can_pay` ; ne panique jamais (`false` sans rien modifier sinon).
    pub fn pay(
        &self,
        currency: &mut u32,
        inventory: &mut Inventory,
        max_health: &mut Fixed,
        health: &mut Fixed,
    ) -> bool {
        if !self.can_pay(*currency, inventory, *max_health, *health) {
            return false;
        }
        match self {
            Self::Currency(amount) => *currency -= *amount,
            Self::Consumable(id, amount) => match inventory.consumables.get_mut(id) {
                Some(count) => *count -= *amount,
                None => return false,
            },
            Self::MaxHealth(amount) => {
                *max_health -= *amount;
                *health -= *amount;
            }
        }
        true
    }
}

/// Le familier que donne un objet (M3-T0d, §63).
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FamiliarRef {
    /// Id d'un personnage (`characters/`) qui porte au moins un behavior de familier.
    pub character: String,
    /// Familiers créés par exemplaire de l'objet tenu (1 par défaut).
    #[serde(default = "one_familiar")]
    #[cfg_attr(feature = "schema", schemars(extend("x-lint" = "> 0")))]
    pub per_copy: u32,
}

fn one_familiar() -> u32 {
    1
}

/// Une synergie (M2-T4, `docs/conventions.md` §44) : un fichier `synergies/<id>.ron` (kind
/// `Synergy`). Active tant que **tous** les objets de `requires` sont tenus (passifs ou actif) ;
/// ses `modifiers` ont la source `synergy:<id>`, ses `effects` la clé `synergy:<id>:<index>`.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SynergyDef {
    pub name: String,
    pub requires: Vec<String>,
    #[serde(default)]
    pub modifiers: Vec<ItemModifier>,
    #[serde(default)]
    pub effects: Vec<Effect>,
    /// M3-T0a : comme [`ItemDef::projectile`], pour une synergie (une synergie peut changer les
    /// larmes). `None` : v1 exacte ; hors rollback et hors hash (définition).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projectile: Option<ProjectileMods>,
}

/// M3-T0a (§60), lu au tir par M3-T1 (§66) : `projectile` des objets tenus par `inventory`, dans
/// l'ordre déterministe de la composition (`combat::projectile::compose`, après ceux de l'arme) :
/// passifs dans l'ordre de ramassage (un passif en double compte deux fois), puis l'actif, puis
/// les synergies actives par id croissant. Les objets sans `projectile` ne donnent rien. Pure.
pub fn projectile_mods<'a>(
    items: &'a ItemTable,
    synergies: &'a SynergyTable,
    inventory: &Inventory,
) -> Vec<&'a ProjectileMods> {
    let item_mods = |id: &str| items.0.get(id).and_then(|def| def.projectile.as_ref());
    let mut out: Vec<&ProjectileMods> = Vec::new();
    out.extend(inventory.passives.iter().filter_map(|id| item_mods(id)));
    if let Some(active) = &inventory.active {
        out.extend(item_mods(&active.item));
    }
    out.extend(
        synergies
            .active(inventory)
            .into_iter()
            .filter_map(|(_, def)| def.projectile.as_ref()),
    );
    out
}

/// Les seules listes de modificateurs de [`projectile_mods`] (même ordre), sans `on_hit` ni
/// `on_expire`.
pub fn projectile_sources<'a>(
    items: &'a ItemTable,
    synergies: &'a SynergyTable,
    inventory: &Inventory,
) -> Vec<&'a [effects::projectile::ProjectileModifier]> {
    projectile_mods(items, synergies, inventory)
        .into_iter()
        .map(|mods| mods.modifiers.as_slice())
        .collect()
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
        self.item.hash(state);
        self.charge.hash(state);
        if let Some(uses) = self.uses {
            uses.hash(state);
        }
    }
}

/// Inventaire d'un joueur (rollback, checksum et trace en variante **neutre**). Posé au premier
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

    /// Nombre d'exemplaires portés, tous types d'inventaire confondus.
    pub fn count(&self, id: &str) -> u32 {
        self.passives
            .iter()
            .filter(|item| item.as_str() == id)
            .count() as u32
            + u32::from(self.active.as_ref().is_some_and(|active| active.item == id))
            + self.consumable(id)
    }
}

/// Clé stable d'un effet porté par un exemplaire. Le premier exemplaire conserve la clé v1 ;
/// les suivants reçoivent un suffixe `#k` (M3-T0b ; branchée par M3-T2 pour `StackMode::Independent`).
pub fn item_effect_key(item_id: &str, copy_index: u32, effect_index: usize) -> String {
    if copy_index == 0 {
        format!("item:{item_id}:{effect_index}")
    } else {
        format!("item:{item_id}#{copy_index}:{effect_index}")
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

    /// M3-T0a : `projectile` absent = v1 exacte ; présent, lu tel quel ; sources de la
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
                parse(
                    r#"(name: "G", kind: Passive, projectile: Some((modifiers: [Size("1.5")])))"#,
                ),
            ),
            (
                "actif".to_string(),
                parse(
                    r#"(name: "A", kind: Active(Frames(10)), projectile: Some((modifiers: [Spectral])))"#,
                ),
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

    /// M3-T1 : `projectile_mods` rend les `ProjectileMods` entiers (`on_hit` et `on_expire` avec les
    /// modificateurs), dans l'ordre de `projectile_sources`.
    #[test]
    fn projectile_mods_porte_on_hit_et_on_expire_dans_le_meme_ordre() {
        let parse = |src: &str| -> ItemDef { ron::from_str(src).unwrap() };
        let table = ItemTable(BTreeMap::from([
            (
                "venin".to_string(),
                parse(
                    r#"(name: "V", kind: Passive, projectile: Some((on_hit: [ApplyStatus(status: "poison", stacks: 1)])))"#,
                ),
            ),
            (
                "mine".to_string(),
                parse(
                    r#"(name: "M", kind: Passive, projectile: Some((on_expire: [DestroyTerrain(radius: "8.0")])))"#,
                ),
            ),
        ]));
        let inventory = Inventory {
            passives: vec!["mine".into(), "venin".into(), "venin".into()],
            active: None,
            consumables: BTreeMap::new(),
        };
        let synergies = SynergyTable::default();
        let mods = projectile_mods(&table, &synergies, &inventory);
        assert_eq!(mods.len(), 3, "un passif en double compte deux fois");
        assert_eq!(mods[0].on_expire.len(), 1);
        assert!(mods[0].on_hit.is_empty());
        assert_eq!(mods[1], mods[2]);
        assert_eq!(mods[1].on_hit.len(), 1);
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

#[cfg(test)]
mod m3_t0b_tests {
    use super::*;

    #[test]
    fn inventory_count_includes_all_copy_kinds() {
        let inventory = Inventory {
            passives: vec!["amulet".into(), "amulet".into()],
            active: Some(ActiveSlot::new("amulet".into())),
            consumables: BTreeMap::from([("amulet".into(), 3)]),
        };
        assert_eq!(inventory.count("amulet"), 6);
        assert_eq!(inventory.count("other"), 0);
    }

    #[test]
    fn item_effect_key_keeps_first_copy_compatibility() {
        assert_eq!(item_effect_key("amulet", 0, 2), "item:amulet:2");
        assert_eq!(item_effect_key("amulet", 1, 2), "item:amulet#1:2");
        assert_eq!(item_effect_key("amulet", 2, 2), "item:amulet#2:2");
    }

    #[test]
    fn prices_pay_currency_consumables_and_non_lethal_max_health() {
        let mut currency = 10;
        let mut inventory = Inventory {
            consumables: BTreeMap::from([("key".into(), 3)]),
            ..Default::default()
        };
        let mut max_health = Fixed::from_num(6);
        let mut health = Fixed::from_num(6);
        assert!(Price::Currency(4).pay(
            &mut currency,
            &mut inventory,
            &mut max_health,
            &mut health
        ));
        assert!(Price::Consumable("key".into(), 2).pay(
            &mut currency,
            &mut inventory,
            &mut max_health,
            &mut health
        ));
        assert_eq!(inventory.consumable("key"), 1);
        assert!(Price::MaxHealth(Fixed::from_num(2)).pay(
            &mut currency,
            &mut inventory,
            &mut max_health,
            &mut health
        ));
        assert_eq!(max_health, Fixed::from_num(4));
        assert_eq!(health, Fixed::from_num(4));
        assert!(
            !Price::MaxHealth(Fixed::from_num(4)).can_pay(currency, &inventory, max_health, health)
        );
    }

    #[test]
    fn zero_prices_are_refused_and_never_panic() {
        let mut currency = 5;
        let mut inventory = Inventory::default();
        let mut max_health = Fixed::from_num(6);
        let mut health = Fixed::from_num(6);
        for price in [
            Price::Consumable("key".into(), 0),
            Price::Currency(0),
            Price::MaxHealth(Fixed::ZERO),
        ] {
            assert!(!price.can_pay(currency, &inventory, max_health, health));
            assert!(!price.pay(&mut currency, &mut inventory, &mut max_health, &mut health));
            assert_eq!(price.problems(|_| true, Fixed::from_num(6)).len(), 1);
        }
        assert_eq!(
            (currency, max_health, health),
            (5, Fixed::from_num(6), Fixed::from_num(6))
        );
        assert!(inventory.consumables.is_empty());
    }

    #[test]
    fn price_problems_report_unknown_consumable_and_out_of_range_health() {
        let max = Fixed::from_num(6);
        assert!(Price::Consumable("key".into(), 1)
            .problems(|id| id == "key", max)
            .is_empty());
        assert_eq!(
            Price::Consumable("ghost".into(), 1)
                .problems(|_| false, max)
                .len(),
            1
        );
        assert_eq!(
            Price::MaxHealth(Fixed::from_num(7))
                .problems(|_| true, max)
                .len(),
            1
        );
        assert_eq!(
            Price::MaxHealth(Fixed::from_num(-1))
                .problems(|_| true, max)
                .len(),
            1
        );
        assert!(Price::MaxHealth(max).problems(|_| true, max).is_empty());
    }
}
