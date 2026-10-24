//! Profil de bac à sable (`sandbox/<id>.ron`, kind `Sandbox`, `docs/taches/plan-bac-a-sable.md` D1).
//!
//! # Format : un sous-ensemble de `Scenario`, pas un second format
//!
//! Un profil est le **sous-ensemble utile de `game::replay::Scenario`** : mêmes noms de champs, mêmes
//! types. Les types que `content` peut porter (les placements, `ModifierSpec`, `DashConfig`) **vivent
//! ici ou plus bas** (`sim_core::dash`) et `game::replay` les réexporte : il n'y en a qu'une définition.
//! Seul [`SandboxPlayer`] est un type propre, parce que `PlayerScript` (qui porte aussi `inputs` et `bot`,
//! des scripts d'inputs que `content` ne connaît pas) reste dans `game` ; `game::replay` prouve par test
//! (`sandbox_player_est_un_sous_ensemble_de_player_script`) que les deux listes de champs coïncident et
//! convertit par `From<SandboxPlayer> for PlayerScript`.
//!
//! Un profil ne contient ni `inputs`, ni `frames`, ni `expect` : `deny_unknown_fields` refuse ce qui
//! n'appartient pas au sous-ensemble (une faute de frappe ne passe pas en silence).
//!
//! Les `at_frame` des placements sont relatifs au début de la session (frame 0 = ouverture de l'arène).

use bevy_fixed::fixed_math::Fixed;
use serde::{Deserialize, Serialize};
use sim_core::modifier::ModifierOp;
use sim_core::stats::StatId;

/// Profil par défaut d'un jeu : `--sandbox` sans argument charge `sandbox/arene.ron`.
pub const DEFAULT_PROFILE: &str = "arene";

/// Contenu d'un fichier `sandbox/<id>.ron`. L'id du profil est le nom du fichier sans extension.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SandboxProfile {
    /// Carte de l'arène, relative aux assets du jeu (comme `Scenario::map`, ou `cave:<id>`). Absente :
    /// `sandbox.arena` du manifeste, sinon `generate_template.map`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub map: Option<String>,
    /// Un état de départ par joueur ; le joueur `i` a le handle `i`. Un joueur sans entrée démarre
    /// comme dans une partie ordinaire.
    #[serde(default)]
    pub players: Vec<SandboxPlayer>,
    /// Personnages posés (ennemis, alliés, cibles) : `at_frame` relatif au début de la session.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub characters: Vec<CharacterPlacement>,
    /// Objets posés au sol.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ItemPlacement>,
    /// Coffres posés.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chests: Vec<ChestPlacement>,
    /// Présentoirs posés.
    #[serde(default, skip_serializing_if = "ShopSetup::is_empty")]
    pub shop: ShopSetup,
}

/// État de départ d'un joueur : les champs de `game::replay::PlayerScript` **sauf** `inputs` et `bot`
/// (même nom, même type, même sens ; voir le module).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SandboxPlayer {
    /// Tags du personnage.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Tags de dégât contre lesquels le joueur est immunisé.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub immune_to: Vec<String>,
    /// Modificateurs de stats posés une fois le joueur créé.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<ModifierSpec>,
    /// Arme unique (id d'une arme à distance ou de mêlée) au lieu des armes de départ du personnage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weapon: Option<String>,
    /// Solde de départ.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency: Option<u32>,
    /// Mutations (ids du kind `Mutation`) prises avant la première frame.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mutations: Vec<String>,
    /// Dash imposé.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dash: Option<sim_core::dash::DashConfig>,
    /// Objets tenus dès la première frame (ids du kind `Item`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<String>,
    /// Bombe imposée (le stock se donne par `items: ["bomb", ...]`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bomb: Option<world::BombDef>,
    /// Visée imposée.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aim: Option<sim_core::aim::AimMode>,
}

/// Placement scripté d'un personnage (T1.13, `docs/conventions.md` §28) : fait apparaître
/// `character` (id du kind `Character`) à la position `(x, y)` du monde, à la frame `at_frame`
/// exacte, par le même chemin qu'un `CharacterSpawn` de carte (`map_ldtk::game::local::
/// spawn_character` : net id alloué à cette frame, santé × difficulté, variante imposée par
/// `variant` ou tirée). `team` : sinon `CharacterConfig.team`, sinon `Enemies`. Les placements
/// d'une même frame sont appliqués dans l'ordre de déclaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CharacterPlacement {
    pub character: String,
    #[schemars(with = "bevy_fixed::schema::FixedString")]
    pub x: Fixed,
    #[schemars(with = "bevy_fixed::schema::FixedString")]
    pub y: Fixed,
    pub at_frame: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team: Option<sim_core::team::Team>,
}

/// Placement scripté d'un objet (M2-T0b) : fait apparaître l'objet `id` (fichier de `items/`) à
/// la position `(x, y)` du monde, à la frame `at_frame` exacte.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ItemPlacement {
    pub id: String,
    #[schemars(with = "bevy_fixed::schema::FixedString")]
    pub x: Fixed,
    #[schemars(with = "bevy_fixed::schema::FixedString")]
    pub y: Fixed,
    pub at_frame: u32,
}

/// Placement scripté d'un coffre (M2-T5) : `key` est sa clé de contenu (graine du tirage),
/// `table` son pool (`None` : celui de l'étage), `rarity` la rareté minimale de l'objet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ChestPlacement {
    pub key: String,
    #[serde(default)]
    pub table: Option<String>,
    #[serde(default)]
    pub rarity: items::Rarity,
    #[serde(default)]
    pub locked: bool,
    #[schemars(with = "bevy_fixed::schema::FixedString")]
    pub x: Fixed,
    #[schemars(with = "bevy_fixed::schema::FixedString")]
    pub y: Fixed,
    pub at_frame: u32,
}

/// Réglages de boutique d'un scénario (M2-T6).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ShopSetup {
    /// Présentoirs posés à une position et une frame exactes.
    #[serde(default)]
    pub shelves: Vec<ShelfPlacement>,
    /// Force `EconomyConfig::shells_drop` pour ce scénario (preuve du drop sans toucher
    /// l'économie du jeu). `None` : celle du jeu.
    #[serde(default)]
    pub shells_drop_override: Option<(u32, u32)>,
}

impl ShopSetup {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// Placement scripté d'un présentoir : `key` est sa clé de contenu, `item` l'objet, `price` en
/// `shells`. `cost` (M3-T8, §73) : prix de vie maximale ou de consommable (`Price`), exécuté à la
/// place des douilles (`price` vaut alors 0) ; absent : le prix en douilles, comme avant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ShelfPlacement {
    pub key: String,
    pub item: String,
    pub price: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<items::Price>,
    #[schemars(with = "bevy_fixed::schema::FixedString")]
    pub x: Fixed,
    #[schemars(with = "bevy_fixed::schema::FixedString")]
    pub y: Fixed,
    pub at_frame: u32,
}

/// Un modificateur de scénario, posé sur un joueur après sa création
/// (`map_ldtk::game::profile::apply_player_profile`). Format RON :
/// `(stat: MoveSpeed, op: Mul, value: "0.5")`. `until` absent = permanent (dure toute la
/// partie, largement suffisant pour un scénario de test).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ModifierSpec {
    pub stat: StatId,
    pub op: ModifierOp,
    #[schemars(with = "bevy_fixed::schema::FixedString")]
    pub value: Fixed,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<u32>,
}

impl SandboxProfile {
    /// Parse un fichier de profil (RON, `IMPLICIT_SOME` comme le manifeste : `weapon: "pistol"`).
    pub fn parse(text: &str) -> Result<Self, ron::error::SpannedError> {
        ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(text)
    }
}

/// Arène du manifeste (D2, étapes 2 et 3) : `sandbox.arena`, sinon `generate_template.map`.
pub fn manifest_arena(manifest: &crate::manifest::GameManifest) -> Option<&str> {
    manifest
        .sandbox
        .as_ref()
        .map(|sandbox| sandbox.arena.as_str())
        .or_else(|| {
            manifest
                .generate_template
                .as_ref()
                .map(|template| template.map.as_str())
        })
}

impl SandboxProfile {
    /// Carte de l'arène de ce profil (D2) : `map` du profil, sinon celle du manifeste.
    pub fn arena<'a>(&'a self, manifest: &'a crate::manifest::GameManifest) -> Option<&'a str> {
        self.map.as_deref().or_else(|| manifest_arena(manifest))
    }
}
