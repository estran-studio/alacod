//! Registre typé : charge les dossiers de contenu déclarés par le manifeste
//! (`manifest.rs`) en tables indexées par des identifiants typés (newtypes sur chaîne,
//! `Ord`, `BTreeMap` partout : CLAUDE.md, règle de déterminisme 5). C'est la source de
//! vérité des ids de contenu (`docs/plan-engine.md` §4.2) ; les `Handle<...>` Bevy
//! (`crates/game/src/global_asset.rs`) restent le support de rendu/animation.
//!
//! Le chargement est "best effort" : une entrée invalide (RON cassé, id dupliqué) est
//! rapportée comme erreur et ignorée plutôt que d'interrompre tout le chargement, pour que
//! `alacod lint` et le rechargement à chaud (`crates/game/src/content_hot_reload.rs`)
//! rapportent plusieurs problèmes en un seul passage.

use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use bevy::prelude::Resource;
use bevy_fixed::fixed_math::Fixed;
use sim_core::ammo::AmmoType;
use sim_core::damage::FriendlyFire;
use sim_core::kinds::{KindDecl, Kinds};
use sim_core::modifier::ModifierOp;
use sim_core::stats::StatId;

use crate::expr::NumOrExpr;
use crate::files::ContentFiles;
use crate::lint::{LintError, LintErrorKind};
use crate::manifest::{ContentFolderDecl, GameManifest, MANIFEST_FILE_NAME};
use crate::value::FixedField;

// ---------------------------------------------------------------------------------------
// Identifiants typés
// ---------------------------------------------------------------------------------------

macro_rules! string_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub String);

        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }
    };
}

string_id!(
    /// Identifiant d'un personnage (joueur ou ennemi : les deux sont un `CharacterConfig`,
    /// voir `crates/game/src/character/config.rs`). Dérivé du champ `asset_name_ref` de
    /// chaque fichier, pas du nom de fichier : les fichiers actuels sous
    /// `ZombieShooter/Sprites/**` ne sont pas nommés d'après leur id (voir
    /// `games/zombies/assets/game.ron`).
    CharacterId
);
string_id!(
    /// Identifiant d'une entrée de `weapons.ron` (arme à distance).
    WeaponId
);
string_id!(
    /// Identifiant d'une entrée de `melee_weapons.ron` (arme de corps à corps).
    MeleeWeaponId
);
string_id!(
    /// Référence à un `CharacterId` dans un rôle d'ennemi (ex. `wave_config.ron`,
    /// `enemy_probabilities`). Même espace de noms que `CharacterId` aujourd'hui : il
    /// n'existe pas encore de dossier `enemies/*.ron` séparé de `characters/*.ron` avec un
    /// schéma propre (voir la doc du module et le rapport de la tâche T1.5).
    EnemyId
);
string_id!(
    /// Identifiant d'une configuration de vagues (nom de fichier, sans extension).
    WaveConfigId
);
string_id!(
    /// Identifiant d'une carte LDtk (nom de fichier, sans extension).
    MapId
);
string_id!(
    /// Identifiant d'une séquence de niveaux du mode `Floors` (T1.8, kind `Floors`) : nom de
    /// fichier sans extension (même règle que [`WaveConfigId`]).
    FloorsConfigId
);
string_id!(
    /// Identifiant d'une caverne (T1.6, kind `Cave`) : nom de fichier sans extension, désigné
    /// comme carte par `cave:<id>` (voir [`cave_designation`]).
    CaveId
);
string_id!(
    /// Identifiant d'une surface (T1.7, kind `Surface`) : nom de fichier sans extension.
    SurfaceName
);
string_id!(
    /// Identifiant d'une progression (T1.10, kind `Progression`) : nom de fichier sans
    /// extension (`progression/<id>.ron`).
    ProgressionId
);
string_id!(
    /// Identifiant d'un statut (T1.3, kind `Status`) : nom de fichier sans extension
    /// (`statuses/<id>.ron`), référencé par `ApplyStatus { status, .. }`.
    StatusId
);
string_id!(
    /// Identifiant d'une mutation (T1.10, kind `Mutation`) : nom de fichier sans extension
    /// (`mutations/<id>.ron`).
    MutationId
);
string_id!(
    /// Identifiant d'un pattern nommé (T1.2, kind `Pattern`) : nom de fichier sans extension
    /// (`patterns/<nom>.ron`), référencé par `ranged.pattern` d'un personnage et par
    /// `Named("<nom>")` dans un pattern.
    PatternId
);
string_id!(
    /// Identifiant d'une horloge (T1.9, kind `Clock`) : nom de fichier sans extension
    /// (`clocks/<nom>.ron`), demandé par `entry.clocks` ou le champ `clocks` d'un scénario.
    ClockId
);
string_id!(
    /// Identifiant du fichier d'économie (T2.3, chantier C5 v1). Nom de fichier, sans
    /// extension (même règle que [`WaveConfigId`]) : un seul fichier par jeu en pratique,
    /// pas imposé par le chargement (comme `Wave`).
    EconomyId
);
string_id!(
    /// Identifiant d'un perk de `economy/perks.ron` (T2.3, chantier C5 v1) : clé de la table.
    PerkId
);
string_id!(
    /// Identifiant d'un power-up de `items/powerups.ron` (T2.5, chantier C1 v0) : clé de
    /// la table (`insta_kill`, `double_points`, `max_ammo`, `carpenter`, `nuke`...).
    PowerUpId
);

string_id!(
    /// D3 : identifiant d'une entrée de la table des feuilles de sprites (kind `SpriteSheet`,
    /// `sprites/sprites.ron`) : le nom que le contenu cite — `asset_name_ref` d'un
    /// personnage, `sprite_config.name` d'une arme à distance, `slash` pour l'effet de mêlée.
    SpriteSheetId
);

/// Dérive le même id qu'au chargement (`load_maps`) à partir d'un chemin quelconque
/// (utilisé pour valider `entry.start_map`, qui n'est pas forcément le même chemin exact
/// que celui listé par un dossier `Map`, mais désigne le même fichier).
pub fn map_id_from_path(path: &str) -> MapId {
    let stem = Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(path);
    MapId::from(stem.to_string())
}

/// Préfixe qui désigne une caverne là où une carte LDtk est attendue (`entry.start_map`,
/// `Scenario.map`, `--map`, `levels` d'une séquence `Floors`) : `cave:<id>` (T1.6,
/// `docs/conventions.md` §21).
pub const CAVE_PREFIX: &str = "cave:";

/// Nom du gabarit LDtk d'un dossier `Cave` (définitions de couches et d'entités, niveau
/// réécrit en mémoire par la génération).
pub const CAVE_TEMPLATE_FILE: &str = "gabarit.ldtk";

/// `Some(id)` si `map` désigne une caverne (`cave:<id>`).
pub fn cave_designation(map: &str) -> Option<CaveId> {
    map.strip_prefix(CAVE_PREFIX)
        .map(|id| CaveId::from(id.to_string()))
}

// ---------------------------------------------------------------------------------------
// Entrées
// ---------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct CharacterEntry {
    pub id: CharacterId,
    /// Chemin relatif à `assets/` : message d'erreur et `AssetServer::load`.
    pub file: PathBuf,
    pub asset_name_ref: String,
    /// F5 (chantier m0-v11) : littéral ou expression `players` (`crate::expr::NumOrExpr`).
    pub base_health_max: NumOrExpr,
    pub max_speed: FixedField,
    pub starting_skin: String,
    pub skins: BTreeSet<String>,
    /// `WeaponId` donnés au spawn, **dans l'ordre déclaré** (`starting_weapons` du RON) :
    /// l'ordre compte (le premier est l'arme active), voir
    /// `crates/game/src/character/player/create.rs`.
    pub starting_weapons: Vec<WeaponId>,
    /// Surcharges de stats (T1.2, chantier B2), pour la règle « valeurs >= 0 ».
    pub stats: BTreeMap<StatId, FixedField>,
    /// À terre (T1.3, chantier B6), pour la règle « > 0 ».
    pub bleedout_frames: u32,
    /// À terre (T1.3, chantier B6), pour la règle « > 0 ».
    pub revive_frames: u32,
    /// À terre (T1.3, chantier B6), pour la règle « 0 < downed_speed_mult <= 1 ».
    pub downed_speed_mult: FixedField,
    /// Emplacements d'armes à distance (T2.2, chantier B7), pour la règle « > 0 ».
    pub weapon_slots: u32,
    /// T1.10 : effets v1 du personnage (`docs/conventions.md` §27).
    pub effects: Vec<effects::Effect>,
    /// T1.4 : règles de comportement (`ai.behaviors`, `None` : liste par défaut).
    pub behaviors: Option<Vec<BehaviorEntry>>,
    /// T1.4 : tags ignorés par le ciblage (`ai.targeting: Some(Nearest(ignore: [...]))`).
    pub ignore_tags: Vec<String>,
    /// Tags du personnage (`tags`), source des tags connus du jeu (règle `ignore`).
    pub tags: Vec<String>,
    /// T1.5 : `ai` présent (personnage IA : la règle `MoveSpeed` → `EnemyMoveSpeed`).
    pub has_ai: bool,
    /// T1.5 : table de variantes (`variants`), dans l'ordre du fichier (doublons gardés pour
    /// la règle « nom en double »).
    pub variants: Option<VariantsEntry>,
    /// T1.13 : `test:` (scénarios générés), pour `lint_character_test`.
    pub test: Option<CharacterTestEntry>,
    /// T1.13 : `counts_hits`, pour la cible de `generate_template`.
    pub counts_hits: bool,
    /// D41 : étendue du corps **en jeu** (collider × `scale`, comme `create_character`) : la plus
    /// grande distance du centre du personnage à un bord de son collider, en px. 0 sans collider.
    pub body_extent: Fixed,
    /// D48 : corps en jeu de plus de 20 px de large ou de haut, le gabarit `Large` de la
    /// navigation (`game::…::navigation::AgentSize::of`, `SMALL_AGENT_MAX`).
    pub body_large: bool,
}

/// T1.13 : mirroir de `game::character::config::CharacterTest` (attentes comptées).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterTestEntry {
    pub frames: u32,
    pub still: bool,
    pub moving: bool,
    pub expect_still: usize,
    pub expect_moving: usize,
}

/// T1.5 : mirroir de `game::character::variant::VariantsConfig`.
#[derive(Debug, Clone)]
pub struct VariantsEntry {
    pub chance: FixedField,
    pub table: Vec<(String, VariantEntry)>,
}

/// T1.5 : mirroir de `game::character::variant::VariantDef`.
#[derive(Debug, Clone, Deserialize)]
pub struct VariantEntry {
    #[serde(default = "default_variant_weight")]
    pub weight: u32,
    #[serde(default)]
    pub modifiers: Vec<VariantModifierEntry>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub skin: Option<String>,
}

fn default_variant_weight() -> u32 {
    1
}

/// T1.5 : un modificateur de variante (format §9 ; une stat inconnue échoue au chargement).
#[derive(Debug, Clone, Deserialize)]
pub struct VariantModifierEntry {
    pub stat: StatId,
    pub op: ModifierOp,
    pub value: FixedField,
}

/// T1.4 : mirroir de `behaviors::Behavior` (`docs/conventions.md` §22) ; les `Fixed` passent
/// par [`FixedField`].
#[derive(Debug, Clone, Deserialize)]
pub enum BehaviorEntry {
    Chase {
        profile: String,
    },
    KeepDistance {
        min: FixedField,
        max: FixedField,
    },
    Strafe,
    Charge {
        telegraph: u32,
    },
    Shoot {
        weapon: String,
        pattern: String,
        range: FixedField,
        cooldown_frames: u32,
    },
    Melee(String),
    Flee,
    Wander,
}

/// T1.4 : mirroir de `behaviors::Targeting`.
#[derive(Debug, Clone, Deserialize)]
pub enum TargetingEntry {
    Nearest {
        #[serde(default)]
        ignore: Vec<String>,
    },
}

/// T1.9 : horloge (`clocks/<nom>.ron`, kind `Clock`), mirroir de `run::clock::ClockDef`.
#[derive(Debug, Clone)]
pub struct ClockEntry {
    pub id: ClockId,
    pub file: PathBuf,
    pub scope: ClockScopeEntry,
    pub events: Vec<ClockEventEntry>,
}

/// T1.9 : mirroir de `run::clock::ClockFileSchema`.
#[derive(Debug, Clone, Deserialize)]
struct ClockFileSchema {
    scope: ClockScopeEntry,
    events: Vec<ClockEventEntry>,
}

/// T1.9 : mirroir de `run::clock::ClockScope`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ClockScopeEntry {
    Run,
    Floor,
}

/// T1.9 : un événement d'horloge (mirroir de `run::clock::ClockEventDef`).
#[derive(Debug, Clone, Deserialize)]
pub struct ClockEventEntry {
    pub id: String,
    pub at: ClockTimeEntry,
    #[serde(default)]
    pub repeat: Option<ClockTimeEntry>,
}

/// T1.9 : mirroir de `run::clock::ClockTime`.
#[derive(Debug, Clone, Copy, Deserialize)]
pub enum ClockTimeEntry {
    Frames(u32),
    Seconds(FixedField),
}

impl ClockTimeEntry {
    /// Durée en frames (60 par seconde, arrondie à la frame inférieure, négatif → 0).
    pub fn frames(self) -> u32 {
        match self {
            ClockTimeEntry::Frames(frames) => frames,
            ClockTimeEntry::Seconds(seconds) => seconds
                .get()
                .saturating_mul(Fixed::from_num(60))
                .max(Fixed::ZERO)
                .to_num::<u32>(),
        }
    }
}

/// T1.9 : difficulté (`difficulty.ron`, kind `Difficulty`) : une expression (`value`).
#[derive(Debug, Clone)]
pub struct DifficultyEntry {
    pub file: PathBuf,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
struct DifficultyFileSchema {
    value: String,
}

/// T1.2 : pattern nommé (`patterns/<nom>.ron`, kind `Pattern`).
#[derive(Debug, Clone)]
pub struct PatternFileEntry {
    pub id: PatternId,
    pub file: PathBuf,
    pub pattern: PatternEntry,
}

#[derive(Debug, Clone)]
pub struct WeaponEntry {
    pub id: WeaponId,
    pub file: PathBuf,
    /// Cadence de tir par mode (`firing_modes`), pour la règle « cadence > 0 ».
    pub firing_rates: BTreeMap<String, FixedField>,
    /// Dispersion par mode (`spread`, radians, pleine largeur), pour la règle « 0 <= spread <= π »
    /// (D51). Absent de la RON (fixtures) : 0.
    pub spreads: BTreeMap<String, FixedField>,
    /// Gabarit de scénario généré (T2.10, champ `test:` de `WeaponConfig`), pour le lint
    /// (`frames > 0`, `min_hits <= max_hits`). `None` : pas de `test:`, aucune règle à
    /// vérifier (l'arme obtient quand même un scénario généré, invariants seulement).
    pub test: Option<WeaponTestRange>,
    /// Type de munition (T2.2), pour la règle « `Custom` au nom non vide » (T2.8).
    pub ammo_type: AmmoType,
    /// Sons référencés par `audio_config` (T2.8), pour la règle « fichier présent sous
    /// `assets/` ». Dans l'ordre (mode, champ) : déterministe pour les messages.
    pub sounds: Vec<SoundRef>,
    /// D3 : `sprite_config.name`, id de la table `SpriteSheet` (`None` si absent ou vide).
    pub sprite: Option<String>,
    /// T1.1 (B5 v1) : `projectile:` de chaque mode de tir (vide = balle ordinaire).
    pub mode_projectiles: BTreeMap<String, ProjectileSpecEntry>,
    /// T1.1 (B5 v1) : table `projectiles` de l'arme, cible des patterns de `on_expire`.
    pub projectiles: BTreeMap<String, ProjectileDefEntry>,
}

/// T1.1 (B5 v1) : mirroirs de `combat::projectile` pour le lint (`content` ne dépend pas de
/// `combat`). Les champs `Fixed` passent par [`FixedField`] (littéral nu refusé).
#[derive(Debug, Clone, Deserialize)]
pub enum ProjectileModifierEntry {
    Bounce(u32),
    Pierce(u32),
    Size(FixedField),
    Lifetime(u32),
    Homing(FixedField),
    Gravity(FixedField),
}

impl ProjectileModifierEntry {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Bounce(_) => "Bounce",
            Self::Pierce(_) => "Pierce",
            Self::Size(_) => "Size",
            Self::Lifetime(_) => "Lifetime",
            Self::Homing(_) => "Homing",
            Self::Gravity(_) => "Gravity",
        }
    }
}

/// Mirroir de `combat::projectile::Pattern`.
#[derive(Debug, Clone, Deserialize)]
pub enum PatternEntry {
    Aimed {
        count: u32,
        spread: FixedField,
        projectile: String,
    },
    Spread {
        count: u32,
        spread: FixedField,
        projectile: String,
    },
    Ring {
        count: u32,
        speed: FixedField,
        projectile: String,
        every: u32,
    },
    Sequence(Vec<PatternEntry>),
    Telegraph(u32),
    Wait(u32),
    /// T1.2 : émetteurs seulement (aléatoire, flux `patterns`).
    Scatter {
        count: u32,
        spread: FixedField,
        projectile: String,
    },
    /// T1.2 : pattern nommé du kind `Pattern`.
    Named(String),
}

/// Mirroir de `combat::projectile::ExpireAction`.
#[derive(Debug, Clone, Deserialize)]
pub enum ExpireActionEntry {
    Spawn(PatternEntry),
    /// T1.6 : creuse le terrain d'une caverne au point de fin du projectile.
    DestroyTerrain {
        radius: FixedField,
    },
}

/// Mirroir de `combat::projectile::ProjectileSpec`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProjectileSpecEntry {
    #[serde(default)]
    pub modifiers: Vec<ProjectileModifierEntry>,
    #[serde(default)]
    pub on_hit: Vec<effects::Action>,
    #[serde(default)]
    pub on_expire: Vec<ExpireActionEntry>,
}

/// Mirroir de `combat::projectile::ProjectileDef`.
#[derive(Debug, Clone, Deserialize)]
pub struct ProjectileDefEntry {
    pub damage: FixedField,
    pub speed: FixedField,
    pub range: FixedField,
    #[serde(default)]
    pub modifiers: Vec<ProjectileModifierEntry>,
    #[serde(default)]
    pub on_hit: Vec<effects::Action>,
    #[serde(default)]
    pub on_expire: Vec<ExpireActionEntry>,
}

impl ProjectileDefEntry {
    pub fn spec(&self) -> ProjectileSpecEntry {
        ProjectileSpecEntry {
            modifiers: self.modifiers.clone(),
            on_hit: self.on_hit.clone(),
            on_expire: self.on_expire.clone(),
        }
    }
}

/// D3 : une entrée de la table des feuilles de sprites (kind `SpriteSheet`). Les chemins
/// sont relatifs à `assets/`, comme partout dans le contenu.
#[derive(Debug, Clone)]
pub struct SpriteSheetEntry {
    pub id: SpriteSheetId,
    pub file: PathBuf,
    /// Configuration d'animation (`animation::AnimationMapConfig`).
    pub animation: String,
    /// Calque (`body`, `shadow`, `hair`…) -> feuille (`animation::SpriteSheetConfig`).
    pub layers: BTreeMap<String, String>,
    /// Calque -> image (`path` de la feuille), pour les feuilles lisibles : le lint vérifie
    /// que l'image existe. Une feuille illisible n'a pas d'entrée ici (le lint rapporte déjà
    /// la feuille elle-même).
    pub images: BTreeMap<String, String>,
}

/// Un chemin de son lu dans le contenu (T2.8) : `field` est le chemin du champ RON
/// (`audio_config.modes.default.reloading`), `path` le fichier, relatif à `assets/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundRef {
    pub field: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct MeleeWeaponEntry {
    pub id: MeleeWeaponId,
    pub file: PathBuf,
    pub damage: FixedField,
    /// Voir `WeaponEntry::test`.
    pub test: Option<WeaponTestRange>,
}

/// Mirroir minimal de `game::weapons::WeaponTest` (`content` ne dépend pas de `game`, voir
/// le module) : juste assez pour que `lint.rs` valide les plages (`frames > 0`, `min_hits
/// <= max_hits`). Le générateur (`crates/scenario/src/generate.rs`, qui dépend à la fois de
/// `content` et de `game`) relit lui-même le RON de l'arme avec le vrai type pour construire
/// le scénario (attentes `expect:` libres comprises, que ce mirroir ignore).
#[derive(Debug, Clone, Copy)]
pub struct WeaponTestRange {
    pub frames: u32,
    pub min_hits: u32,
    pub max_hits: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct WaveConfigEntry {
    pub id: WaveConfigId,
    pub file: PathBuf,
    /// Noms référencés par `wave_tiers[].enemy_probabilities`, union de tous les paliers.
    pub enemy_refs: BTreeSet<EnemyId>,
}

/// Séquence de niveaux du mode `Floors` (T1.8, `docs/conventions.md` §17), un fichier RON
/// par séquence (`floors/<id>.ron`) : `(levels: ["testbed/floor_a.ldtk", ...])`. Les chemins
/// sont relatifs à `assets/`, comme `entry.start_map`. Lint (`lint::lint_floors`) : liste
/// non vide, chaque niveau désigne une carte chargée (kind `Map`).
#[derive(Debug, Clone)]
pub struct FloorsEntry {
    pub id: FloorsConfigId,
    pub file: PathBuf,
    /// Cartes LDtk des niveaux, dans l'ordre de jeu.
    pub levels: Vec<String>,
}

/// Caverne générée (T1.6, `docs/conventions.md` §21), un fichier RON par caverne
/// (`caves/<id>.ron`, champs de [`world::CaveConfig`]). Le dossier contient aussi le gabarit
/// LDtk [`CAVE_TEMPLATE_FILE`] ; `template` est son chemin relatif à `assets/`.
#[derive(Debug, Clone)]
pub struct CaveEntry {
    pub id: CaveId,
    pub file: PathBuf,
    pub config: world::CaveConfig,
    pub template: String,
}

/// Surface (T1.7, `docs/conventions.md` §26), un fichier RON par surface (`surfaces/<id>.ron`) :
/// `(intgrid_value: 1, tags: ["eau"], move_speed: "0.5", acceleration: Some("1.0"))`. Facteurs
/// abstraits (`Mul`) traduits vers la stat du personnage par `world::surface`.
#[derive(Debug, Clone)]
pub struct SurfaceEntry {
    pub id: SurfaceName,
    pub file: PathBuf,
    pub intgrid_value: u8,
    pub tags: Vec<String>,
    pub move_speed: Fixed,
    pub acceleration: Fixed,
}

#[derive(Debug, Clone, Deserialize)]
struct SurfaceFileSchema {
    intgrid_value: u8,
    tags: Vec<String>,
    move_speed: FixedField,
    #[serde(default)]
    acceleration: Option<FixedField>,
}

/// Progression (T1.10, chantier C4 v1, `docs/conventions.md` §27), `progression/<id>.ron` : jauge
/// des joueurs (rads), seuils de niveau, choix de mutation, pool d'armes par niveau.
#[derive(Debug, Clone)]
pub struct ProgressionEntry {
    pub id: ProgressionId,
    pub file: PathBuf,
    /// Id de la jauge posée sur chaque joueur (`"rads"`).
    pub gauge: String,
    /// Ajouté à la jauge du tueur à chaque ennemi tué par un joueur.
    pub per_kill: Fixed,
    /// Seuils de la jauge, croissants : niveau = nombre de seuils atteints.
    pub levels: Vec<Fixed>,
    /// Nombre de mutations proposées à chaque niveau.
    pub choices: u32,
    /// Frames avant que la première option soit prise d'office.
    pub choice_frames: u32,
    /// Pool de mutations ; vide = toutes les mutations du jeu.
    pub mutations: Vec<String>,
    pub weapon_pool: Vec<WeaponPoolEntry>,
    /// Chance (`[0, 1]`) qu'un ennemi tué laisse tomber une arme du pool ; 0 par défaut.
    pub weapon_drop_chance: Fixed,
}

/// Armes débloquées à partir de `level` (niveau max des joueurs).
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponPoolEntry {
    pub level: u32,
    pub weapons: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ProgressionFileSchema {
    gauge: String,
    per_kill: FixedField,
    levels: Vec<FixedField>,
    choices: u32,
    choice_frames: u32,
    #[serde(default)]
    mutations: Vec<String>,
    #[serde(default)]
    weapon_pool: Vec<WeaponPoolEntry>,
    #[serde(default)]
    weapon_drop_chance: Option<FixedField>,
}

/// Genre d'un statut (miroir de `combat::status::StatusDef`, T1.3 ; `content` ne dépend pas
/// de `combat`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum StatusKindEntry {
    Burn,
    Slow,
    Stun,
    Freeze,
}

/// Statut (T1.3, `docs/conventions.md` §19), `statuses/<id>.ron` :
/// `(kind: Burn, frames: 120, damage: "4", period: 30)`, `(kind: Slow, frames: 120, factor: "0.5")`.
#[derive(Debug, Clone)]
pub struct StatusContentEntry {
    pub id: StatusId,
    pub file: PathBuf,
    pub kind: StatusKindEntry,
    pub frames: u32,
    pub damage: Fixed,
    pub period: u32,
    pub factor: Fixed,
}

#[derive(Debug, Clone, Deserialize)]
struct StatusFileSchema {
    kind: StatusKindEntry,
    frames: u32,
    #[serde(default)]
    damage: Option<FixedField>,
    #[serde(default)]
    period: Option<u32>,
    #[serde(default)]
    factor: Option<FixedField>,
}

/// Mutation (T1.10, chantier C4 v1), `mutations/<id>.ron` : `( name, weight, tags, max_stacks,
/// effects )`. Ses effets sont ajoutés aux `Effects` du joueur qui la choisit.
#[derive(Debug, Clone)]
pub struct MutationEntry {
    pub id: MutationId,
    pub file: PathBuf,
    pub name: String,
    pub weight: u32,
    pub tags: Vec<String>,
    pub max_stacks: u32,
    pub effects: Vec<effects::Effect>,
}

#[derive(Debug, Clone, Deserialize)]
struct MutationFileSchema {
    name: String,
    #[serde(default = "one")]
    weight: u32,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default = "one")]
    max_stacks: u32,
    effects: Vec<effects::Effect>,
}

fn one() -> u32 {
    1
}

#[derive(Debug, Clone, Deserialize)]
struct FloorsFileSchema {
    levels: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MapEntry {
    pub id: MapId,
    pub file: PathBuf,
    /// T1.5 : `CharacterSpawn` qui imposent une variante (`(personnage, variante)`, champs LDtk
    /// `character`/`variant` des niveaux internes du `.ldtk`), pour le lint des références.
    pub forced_variants: Vec<(String, String)>,
    /// T1.12 : personnage de chaque `CharacterSpawn` des niveaux internes (champ `character`),
    /// pour la règle « personnage connu ».
    pub spawned_characters: Vec<String>,
    /// M2-E1 : `(niveau, room_kind)` de chaque niveau LDtk interne dont le champ de niveau
    /// `room_kind` est rempli (gabarits de salles typées, `docs/conventions.md` §35), pour le
    /// lint « type de salle connu ».
    pub room_kinds: Vec<(String, String)>,
}

/// M2-E1 : champ de niveau `room_kind` (non vide) de chaque niveau d'un `.ldtk`, avec
/// l'identifiant du niveau, dans l'ordre du fichier ; vide si la carte est illisible en JSON.
fn ldtk_room_kinds(text: &str) -> Vec<(String, String)> {
    let Ok(json) = serde_json::from_str::<serde_json::Value>(text) else {
        return Vec::new();
    };
    json["levels"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|level| {
            let kind = level["fieldInstances"]
                .as_array()?
                .iter()
                .find(|f| f["__identifier"] == "room_kind")?["__value"]
                .as_str()
                .filter(|value| !value.is_empty())?;
            Some((
                level["identifier"].as_str().unwrap_or_default().to_string(),
                kind.to_string(),
            ))
        })
        .collect()
}

/// Type de salle (M2-E1, `docs/conventions.md` §35), un fichier RON par type
/// (`rooms/<id>.ron`, champs de [`world::RoomKindDef`]) : `(locks: true)`.
#[derive(Debug, Clone)]
pub struct RoomEntry {
    pub id: String,
    pub file: PathBuf,
    pub def: world::RoomKindDef,
}

/// T1.12 : champ `character` de chaque `CharacterSpawn` d'un `.ldtk` (niveaux internes), dans
/// l'ordre du fichier ; vide si la carte est illisible en JSON.
fn ldtk_character_spawns(text: &str) -> Vec<String> {
    let Ok(json) = serde_json::from_str::<serde_json::Value>(text) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for level in json["levels"].as_array().into_iter().flatten() {
        for layer in level["layerInstances"].as_array().into_iter().flatten() {
            for entity in layer["entityInstances"].as_array().into_iter().flatten() {
                if entity["__identifier"] != "CharacterSpawn" {
                    continue;
                }
                let character = entity["fieldInstances"]
                    .as_array()
                    .and_then(|fields| fields.iter().find(|f| f["__identifier"] == "character"))
                    .and_then(|f| f["__value"].as_str())
                    .unwrap_or_default();
                out.push(character.to_string());
            }
        }
    }
    out
}

/// T1.5 : `CharacterSpawn` à champ `variant` rempli dans un `.ldtk` (niveaux internes). Une
/// carte illisible en JSON ne donne rien (le chargement LDtk du jeu la rapportera).
fn ldtk_forced_variants(text: &str) -> Vec<(String, String)> {
    let Ok(json) = serde_json::from_str::<serde_json::Value>(text) else {
        return Vec::new();
    };
    let field = |entity: &serde_json::Value, name: &str| -> Option<String> {
        entity["fieldInstances"]
            .as_array()?
            .iter()
            .find(|f| f["__identifier"] == name)?["__value"]
            .as_str()
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    };
    let mut out = Vec::new();
    for level in json["levels"].as_array().into_iter().flatten() {
        for layer in level["layerInstances"].as_array().into_iter().flatten() {
            for entity in layer["entityInstances"].as_array().into_iter().flatten() {
                if entity["__identifier"] != "CharacterSpawn" {
                    continue;
                }
                if let Some(variant) = field(entity, "variant") {
                    out.push((field(entity, "character").unwrap_or_default(), variant));
                }
            }
        }
    }
    out
}

/// T2.3, chantier C5 v1 : `games/<jeu>/assets/economy/economy.ron`. Lint (T2.8,
/// `lint::lint_economy`) : `refill_price_ratio` dans `[0, 1]` ; les compteurs de points
/// n'ont pas de plage interdite. F5 (chantier m0-v11) : champs numériques en
/// `NumOrExpr` (littéral ou expression `players`, résolus par `game::balance`).
#[derive(Debug, Clone)]
pub struct EconomyEntry {
    pub id: EconomyId,
    pub file: PathBuf,
    pub kill_points: NumOrExpr,
    pub hit_points: NumOrExpr,
    pub repair_points: NumOrExpr,
    pub nuke_points: NumOrExpr,
    pub repair_points_cap_per_wave: Option<NumOrExpr>,
    pub refill_price_ratio: NumOrExpr,
}

/// T2.3, chantier C5 v1 : une entrée de `games/<jeu>/assets/economy/perks.ron`. Un `StatId`
/// inconnu dans `modifiers[].stat` échoue déjà au chargement RON (`StatId` n'a pas de
/// variante fourre-tout implicite, voir `PerkModifierSchema`), rapporté comme erreur de
/// parse avec le nom du champ (T2.8).
#[derive(Debug, Clone)]
pub struct PerkEntry {
    pub id: PerkId,
    pub file: PathBuf,
    /// F5 (chantier m0-v11) : littéral ou expression `players` (`crate::expr::NumOrExpr`).
    pub price: NumOrExpr,
    /// T2.8 : pour les règles « au moins un modificateur » et « `Mul` > 0 ».
    pub modifiers: Vec<PerkModifierEntry>,
}

/// Un modificateur de perk (T2.8), mirroir de `game::economy::PerkModifierDef`.
#[derive(Debug, Clone)]
pub struct PerkModifierEntry {
    pub stat: StatId,
    pub op: ModifierOp,
    pub value: FixedField,
}

/// T2.5, chantier C1 v0 : une entrée de `games/<jeu>/assets/items/powerups.ron`. Une
/// référence de `StatId` inconnue dans `actions[].stat` échoue déjà au chargement RON
/// (`effects::Action` est le type réel, pas un mirroir — voir la doc de
/// [`PowerUpEntrySchema`]), rapportée comme n'importe quelle autre erreur de parse.
#[derive(Debug, Clone)]
pub struct PowerUpEntry {
    pub id: PowerUpId,
    pub file: PathBuf,
    /// Poids relatif de tirage parmi les power-ups (T2.5) quand un drop a lieu. Pour la
    /// règle « chance de drop » (probabilité globale qu'un drop ait lieu du tout, pas
    /// laquelle), voir [`Registry::powerup_drop_chance`].
    pub weight: u32,
    /// T2.8 : portée de ramassage, pour la règle « > 0 ».
    pub pickup_range: FixedField,
    /// T2.8 : durée de vie au sol, pour la règle « > 0 ».
    pub lifetime_frames: u32,
    /// T2.8 : pour les règles « au moins une action », « `frames` > 0 », « `factor` > 0 ».
    pub actions: Vec<effects::Action>,
}

/// Valeur racine de `items/powerups.ron` (T2.5) avec son fichier, pour un message de lint
/// précis (voir [`Registry::powerup_drop_chance`]).
#[derive(Debug, Clone)]
pub struct PowerUpDropChanceEntry {
    pub drop_chance: FixedField,
    pub file: PathBuf,
}

/// Paramètres de présentation de `ui/feedback.ron`, lus par le lint hors simulation.
#[derive(Debug, Clone)]
pub struct FeedbackEntry {
    pub file: PathBuf,
    /// Même type que l'asset du jeu (`game::feedback::FeedbackConfig`), T1.17.
    pub settings: crate::feedback::FeedbackSettings,
}

/// Registre de contenu d'un jeu, chargé depuis son manifeste (`GameManifest`). Voir le
/// module pour les garanties (BTreeMap partout, chargement "best effort").
#[derive(Resource, Debug, Clone, Default)]
pub struct Registry {
    /// Optional browser inventory; native registries retain filesystem validation.
    pub embedded_files: Option<crate::files::EmbeddedFiles>,
    pub game_dir: PathBuf,
    pub characters: BTreeMap<CharacterId, CharacterEntry>,
    pub weapons: BTreeMap<WeaponId, WeaponEntry>,
    pub melee_weapons: BTreeMap<MeleeWeaponId, MeleeWeaponEntry>,
    pub waves: BTreeMap<WaveConfigId, WaveConfigEntry>,
    pub maps: BTreeMap<MapId, MapEntry>,
    /// T1.8 : séquences de niveaux du mode `Floors` (kind `Floors`).
    pub floors: BTreeMap<FloorsConfigId, FloorsEntry>,
    /// T1.6 : cavernes générées (kind `Cave`).
    pub caves: BTreeMap<CaveId, CaveEntry>,
    /// M2-E1 : types de salles (kind `Room`).
    pub rooms: BTreeMap<String, RoomEntry>,
    /// T1.7 : surfaces (kind `Surface`).
    pub surfaces: BTreeMap<SurfaceName, SurfaceEntry>,
    /// T1.2 : patterns nommés (kind `Pattern`).
    pub patterns: BTreeMap<PatternId, PatternFileEntry>,
    /// T1.10 : progressions (kind `Progression`) ; la partie en joue au plus une, nommée par
    /// `entry.progression` ou `Scenario::progression`.
    pub progression: BTreeMap<ProgressionId, ProgressionEntry>,
    /// T1.10 : mutations (kind `Mutation`).
    pub mutations: BTreeMap<MutationId, MutationEntry>,
    /// T1.3 : statuts (kind `Status`).
    pub statuses: BTreeMap<StatusId, StatusContentEntry>,
    /// T1.9 : horloges (kind `Clock`).
    pub clocks: BTreeMap<ClockId, ClockEntry>,
    /// T1.9 : difficulté (kind `Difficulty`, un fichier ; le dernier chargé gagne).
    pub difficulty: Option<DifficultyEntry>,
    /// T2.3, chantier C5 v1.
    pub economy: BTreeMap<EconomyId, EconomyEntry>,
    /// T2.3, chantier C5 v1.
    pub perks: BTreeMap<PerkId, PerkEntry>,
    /// T2.5, chantier C1 v0 : une entrée par power-up de `items/powerups.ron`.
    pub powerups: BTreeMap<PowerUpId, PowerUpEntry>,
    /// T2.5, chantier C1 v0 : probabilité (Fixed `[0, 1]`) qu'un ennemi tué laisse tomber
    /// un power-up, lue à la racine de `items/powerups.ron` (`PowerUpsFileSchema::
    /// drop_chance`) — indépendante du choix du power-up (voir [`PowerUpEntry::weight`]).
    /// `None` si aucun dossier de contenu `PowerUp` n'est déclaré par le jeu. Plusieurs
    /// fichiers `PowerUp` chargés (pas le cas en pratique, un seul par jeu comme
    /// `Economy`) : le dernier traité gagne silencieusement, comme `wave_config`/
    /// `economy_config` dans `game::global_asset` (`.values().next()`).
    pub powerup_drop_chance: Option<PowerUpDropChanceEntry>,
    /// Fichiers `Ui`/`Camera` validés (RON syntaxiquement correct, feedback typé). Pas de table typée par
    /// id : rien ne les référence par id aujourd'hui (décision T1.5, voir le rapport de la
    /// tâche).
    pub ui_files: Vec<PathBuf>,
    /// Réglages typés du feedback (T3.4) parmi les fichiers Ui.
    pub feedback: Vec<FeedbackEntry>,
    /// T1.16 : écrans de mutation (`ui/mutation_screen.ron`) parmi les fichiers Ui.
    pub mutation_screens: Vec<(PathBuf, crate::ui::MutationScreenLayout)>,
    /// T1.18 : sources des widgets de chaque `ui/hud.ron`.
    pub hud_sources: Vec<(PathBuf, Vec<String>)>,
    pub camera_files: Vec<PathBuf>,
    /// D3 : feuilles de sprites par id (kind `SpriteSheet`), source des sprites chargés par
    /// `game::global_asset` (avant D3 : une table de chemins écrite dans le code).
    pub sprite_sheets: BTreeMap<SpriteSheetId, SpriteSheetEntry>,
}

/// Les "kinds" de dossier de contenu que cette version de `content` sait charger. Exposé
/// via `sim_core::kinds::Kinds` (constante ici plutôt que construite depuis une `App`
/// Bevy : voir le rapport de la tâche T1.5, décision « liste des kinds pour la CLI »).
pub const KNOWN_KIND_NAMES: &[&str] = &[
    "Character",
    "Weapon",
    "MeleeWeapon",
    "Wave",
    "Map",
    "Ui",
    "Camera",
    "Economy",
    "Perk",
    "PowerUp",
    "SpriteSheet",
    "Floors",
    "Cave",
    "Room",
    "Surface",
    "Pattern",
    "Progression",
    "Mutation",
    "Status",
    "Clock",
    "Difficulty",
];

pub fn known_content_kinds() -> Kinds {
    let mut kinds = Kinds::default();
    for name in KNOWN_KIND_NAMES {
        kinds.register(KindDecl::new("content_folder", *name));
    }
    kinds
}

impl Registry {
    /// Charge tous les dossiers déclarés par `manifest` (chemins relatifs à
    /// `game_dir/assets/`). Retourne un registre "best effort" (les entrées valides sont
    /// gardées même si d'autres échouent) et la liste des erreurs de chargement (RON
    /// cassé, id dupliqué, kind de dossier inconnu). Les erreurs *sémantiques*
    /// (références, plages de valeurs) sont du ressort de `lint::run`, appelé séparément
    /// sur le résultat.
    pub fn build(game_dir: &Path, manifest: &GameManifest) -> (Registry, Vec<LintError>) {
        Self::build_with_files(
            game_dir,
            manifest,
            ContentFiles::native(GameManifest::assets_dir(game_dir)),
            None,
        )
    }

    pub fn build_embedded(
        manifest: &GameManifest,
        files: crate::files::EmbeddedFiles,
    ) -> (Registry, Vec<LintError>) {
        Self::build_with_files(
            Path::new(""),
            manifest,
            ContentFiles::embedded(files),
            Some(files),
        )
    }

    pub fn asset_exists(&self, path: impl AsRef<Path>) -> bool {
        match self.embedded_files {
            Some(files) => ContentFiles::embedded(files).contains(path),
            None => ContentFiles::native(GameManifest::assets_dir(&self.game_dir)).contains(path),
        }
    }

    fn build_with_files(
        game_dir: &Path,
        manifest: &GameManifest,
        assets_dir: ContentFiles,
        embedded_files: Option<crate::files::EmbeddedFiles>,
    ) -> (Registry, Vec<LintError>) {
        let mut registry = Registry {
            game_dir: game_dir.to_path_buf(),
            embedded_files,
            ..Default::default()
        };
        let mut errors = Vec::new();
        let kinds = known_content_kinds();

        for decl in &manifest.content_folders {
            if !kinds.has("content_folder", &decl.kind) {
                errors.push(LintError {
                    kind: LintErrorKind::UnknownKind,
                    file: MANIFEST_FILE_NAME.to_string(),
                    message: format!(
                        "content_folders : kind « {} » inconnu pour le dossier « {} » (connus : {})",
                        decl.kind,
                        decl.path,
                        KNOWN_KIND_NAMES.join(", ")
                    ),
                });
                continue;
            }

            match decl.kind.as_str() {
                "Character" => load_characters(&assets_dir, decl, &mut registry, &mut errors),
                "Weapon" => load_weapons(&assets_dir, decl, &mut registry, &mut errors),
                "MeleeWeapon" => load_melee_weapons(&assets_dir, decl, &mut registry, &mut errors),
                "Wave" => load_waves(&assets_dir, decl, &mut registry, &mut errors),
                "Map" => load_maps(&assets_dir, decl, &mut registry, &mut errors),
                "Ui" => load_ui(&assets_dir, decl, &mut registry, &mut errors),
                "Camera" => {
                    load_generic_ron(&assets_dir, decl, &mut registry.camera_files, &mut errors)
                }
                "Economy" => load_economy(&assets_dir, decl, &mut registry, &mut errors),
                "Perk" => load_perks(&assets_dir, decl, &mut registry, &mut errors),
                "PowerUp" => load_powerups(&assets_dir, decl, &mut registry, &mut errors),
                "SpriteSheet" => load_sprite_sheets(&assets_dir, decl, &mut registry, &mut errors),
                "Floors" => load_floors(&assets_dir, decl, &mut registry, &mut errors),
                "Cave" => load_caves(&assets_dir, decl, &mut registry, &mut errors),
                "Room" => load_rooms(&assets_dir, decl, &mut registry, &mut errors),
                "Surface" => load_surfaces(&assets_dir, decl, &mut registry, &mut errors),
                "Pattern" => load_patterns(&assets_dir, decl, &mut registry, &mut errors),
                "Progression" => load_progression(&assets_dir, decl, &mut registry, &mut errors),
                "Mutation" => load_mutations(&assets_dir, decl, &mut registry, &mut errors),
                "Status" => load_statuses(&assets_dir, decl, &mut registry, &mut errors),
                "Clock" => load_clocks(&assets_dir, decl, &mut registry, &mut errors),
                "Difficulty" => load_difficulty(&assets_dir, decl, &mut registry, &mut errors),
                _ => unreachable!("filtré par `kinds.has` ci-dessus"),
            }
        }

        apply_cave_spawn_clearance(&mut registry);
        (registry, errors)
    }
}

/// Dégagement (en cases) qu'exige un corps d'étendue `extent` px autour d'un point
/// d'apparition (D41) : le centre d'une case est à 8 px de son bord, chaque case de dégagement
/// ajoute 16 px ; au moins 1 (`world::is_open`).
pub fn spawn_clearance_for(extent: Fixed) -> u32 {
    let half_cell = Fixed::from_num(world::CELL_SIZE / 2);
    let cell = Fixed::from_num(world::CELL_SIZE);
    let mut clearance = 1u32;
    while half_cell + cell * Fixed::from_num(clearance) < extent {
        clearance += 1;
    }
    clearance
}

/// D41 : `CaveConfig::spawn_clearance` de chaque caverne = le dégagement du plus grand corps en
/// jeu parmi ses `characters` (au moins celui déclaré ; un id inconnu est rapporté par le lint).
/// Après le chargement de tous les kinds : l'ordre des dossiers du manifeste n'importe pas.
fn apply_cave_spawn_clearance(registry: &mut Registry) {
    for cave in registry.caves.values_mut() {
        let needed = cave
            .config
            .characters
            .iter()
            .filter_map(|id| registry.characters.get(&CharacterId::from(id.as_str())))
            .map(|character| spawn_clearance_for(character.body_extent))
            .max()
            .unwrap_or(1);
        cave.config.spawn_clearance = cave.config.spawn_clearance.max(needed);
        // D48 : gabarit de navigation du plus grand corps (points atteints par son champ).
        cave.config.nav_large |= cave
            .config
            .characters
            .iter()
            .filter_map(|id| registry.characters.get(&CharacterId::from(id.as_str())))
            .any(|character| character.body_large);
    }
}

// ---------------------------------------------------------------------------------------
// Découverte de fichiers
// ---------------------------------------------------------------------------------------

/// Résout une déclaration de contenu (`decl.path`, relatif à `assets/`) en fichiers : un
/// fichier unique tel quel, ou un dossier scanné (non récursif, trié) pour l'extension
/// `ext`. Non récursif : certains dossiers de contenu actuels mélangent plusieurs kinds
/// (ex. `ZombieShooter/Sprites/Character/` contient aussi bien des personnages que des
/// feuilles de sprite) ; le manifeste déclare alors des fichiers individuels plutôt que le
/// dossier entier (voir `games/zombies/assets/game.ron`).
fn discover_files(
    assets_dir: &ContentFiles,
    decl: &ContentFolderDecl,
    ext: &str,
) -> Result<Vec<PathBuf>, LintError> {
    assets_dir
        .discover(Path::new(&decl.path), ext)
        .map_err(|e| LintError {
            kind: LintErrorKind::Parse,
            file: decl.path.clone(),
            message: format!("fichier ou dossier illisible (content_folders) : {e}"),
        })
}

fn read_file(assets_dir: &ContentFiles, rel: &Path) -> Result<String, LintError> {
    assets_dir.read(rel).map_err(|e| LintError {
        kind: LintErrorKind::Parse,
        file: rel.display().to_string(),
        message: format!("lecture impossible : {e}"),
    })
}

// ---------------------------------------------------------------------------------------
// Schémas de lint (mirroir minimal des types réels de `crates/game`, voir `value.rs`)
// ---------------------------------------------------------------------------------------
//
// Ces structs ne portent que les champs utiles au lint (id, références, plages). Le RON
// est self-describing (comme JSON) : un champ non déclaré ici est silencieusement ignoré
// par serde (pas de `deny_unknown_fields`), donc un fichier réel plus riche (collider,
// sprite_config, audio_config...) se charge sans erreur.

#[derive(Deserialize)]
struct CharacterFileSchema {
    movement: MovementSchema,
    asset_name_ref: String,
    base_health: HealthSchema,
    starting_skin: String,
    #[serde(default)]
    skins: BTreeMap<String, serde::de::IgnoredAny>,
    #[serde(default)]
    starting_weapons: Vec<String>,
    /// T1.2, chantier B2 : surcharges de stats. `StatId` vient directement de `sim_core`
    /// (pas de mirroir : c'est déjà le type réel, `content` en dépend déjà).
    #[serde(default)]
    stats: BTreeMap<StatId, FixedField>,
    /// T1.3, chantier B6 : voir `game::character::config::CharacterConfig::bleedout_frames`.
    #[serde(default = "default_bleedout_frames")]
    bleedout_frames: u32,
    #[serde(default = "default_revive_frames")]
    revive_frames: u32,
    #[serde(default = "default_downed_speed_mult")]
    downed_speed_mult: FixedField,
    /// T2.2, chantier B7 : voir `game::character::config::CharacterConfig::weapon_slots`.
    #[serde(default = "default_weapon_slots")]
    weapon_slots: u32,
    /// T1.4 : seuls `ai.behaviors` et `ai.targeting` sont lus.
    #[serde(default)]
    ai: Option<AiSchema>,
    /// T1.10 : effets v1.
    #[serde(default)]
    effects: Vec<effects::Effect>,
    #[serde(default)]
    tags: Vec<String>,
    /// T1.5 : variantes et élites.
    #[serde(default)]
    variants: Option<VariantsSchema>,
    /// T1.13 : scénarios générés du personnage.
    #[serde(default)]
    test: Option<CharacterTestSchema>,
    /// T1.13 : cible de `generate_template` (règle « `counts_hits` »).
    #[serde(default)]
    counts_hits: bool,
    /// D41 : corps (dégagement des points d'apparition des cavernes). Écrit sans `Some(...)`
    /// dans les RON de personnages (`collider: (...)`, comme `CharacterConfig`).
    #[serde(default, deserialize_with = "present")]
    collider: Option<ColliderSchema>,
    #[serde(default = "default_scale")]
    scale: FixedField,
}

/// Champ optionnel écrit nu dans le RON (`champ: (...)` et non `champ: Some((...))`).
fn present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

fn default_scale() -> FixedField {
    FixedField(Fixed::ONE)
}

/// Mirroir RON de `game::collider::Collider` (seules les dimensions servent).
#[derive(Deserialize)]
struct ColliderSchema {
    shape: ColliderShapeSchema,
    #[serde(default)]
    offset: OffsetSchema,
}

#[derive(Deserialize)]
enum ColliderShapeSchema {
    Rectangle {
        width: FixedField,
        height: FixedField,
    },
    Circle {
        radius: FixedField,
    },
}

#[derive(Deserialize)]
struct OffsetSchema {
    x: FixedField,
    y: FixedField,
    #[serde(rename = "z", default)]
    _z: serde::de::IgnoredAny,
}

impl Default for OffsetSchema {
    fn default() -> Self {
        Self {
            x: FixedField(Fixed::ZERO),
            y: FixedField(Fixed::ZERO),
            _z: serde::de::IgnoredAny,
        }
    }
}

impl ColliderSchema {
    /// Plus grande distance du centre à un bord, offset compris, à l'échelle `scale`.
    fn extent(&self, scale: Fixed) -> Fixed {
        let (half_w, half_h) = match &self.shape {
            ColliderShapeSchema::Rectangle { width, height } => {
                (width.0 / Fixed::from_num(2), height.0 / Fixed::from_num(2))
            }
            ColliderShapeSchema::Circle { radius } => (radius.0, radius.0),
        };
        let (ox, oy) = (self.offset.x.0.abs(), self.offset.y.0.abs());
        (half_w + ox).max(half_h + oy).saturating_mul(scale)
    }

    /// D48 : gabarit de navigation grand, la règle de `AgentSize::of` (`game`) : largeur ou
    /// hauteur du collider à l'échelle `scale` au-delà de [`SMALL_AGENT_MAX`] px (l'offset ne
    /// change pas la largeur).
    fn is_large(&self, scale: Fixed) -> bool {
        let (w, h) = match &self.shape {
            ColliderShapeSchema::Rectangle { width, height } => (width.0, height.0),
            ColliderShapeSchema::Circle { radius } => {
                (radius.0 * Fixed::from_num(2), radius.0 * Fixed::from_num(2))
            }
        };
        let max = Fixed::from_num(SMALL_AGENT_MAX);
        w.saturating_mul(scale) > max || h.saturating_mul(scale) > max
    }
}

/// Largeur maximale (px) d'un corps du gabarit de navigation petit : `SMALL_AGENT_MAX` de
/// `game::character::enemy::ai::navigation` (`content` ne dépend pas de `game`).
pub const SMALL_AGENT_MAX: i32 = 20;

/// Mirroir RON de `game::character::config::CharacterTest` (comme `WeaponTestSchema`) : les
/// attentes ne sont que comptées (`content` ne type pas `Expectation`).
#[derive(Deserialize)]
struct CharacterTestSchema {
    frames: u32,
    #[serde(default = "default_true")]
    still: bool,
    #[serde(default = "default_true")]
    moving: bool,
    #[serde(default)]
    expect_still: Vec<serde::de::IgnoredAny>,
    #[serde(default)]
    expect_moving: Vec<serde::de::IgnoredAny>,
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
struct VariantsSchema {
    #[serde(default = "default_variant_chance")]
    chance: FixedField,
    table: KeyedEntries<VariantEntry>,
}

fn default_variant_chance() -> FixedField {
    FixedField(Fixed::from_num(1))
}

#[derive(Deserialize)]
struct AiSchema {
    #[serde(default)]
    behaviors: Option<Vec<BehaviorEntry>>,
    #[serde(default)]
    targeting: Option<TargetingEntry>,
}

fn default_bleedout_frames() -> u32 {
    1800
}

fn default_revive_frames() -> u32 {
    180
}

fn default_downed_speed_mult() -> FixedField {
    FixedField(Fixed::from_num(0.3))
}

fn default_weapon_slots() -> u32 {
    2
}

#[derive(Deserialize)]
struct MovementSchema {
    max_speed: FixedField,
}

#[derive(Deserialize)]
struct HealthSchema {
    /// F5 (chantier m0-v11) : littéral ou expression `players` (`crate::expr::NumOrExpr`).
    max: NumOrExpr,
}

#[derive(Deserialize)]
struct WeaponsFileSchema(KeyedEntries<WeaponEntrySchema>);

#[derive(Deserialize)]
struct WeaponEntrySchema {
    config: WeaponConfigSchema,
    /// T2.8 : sons de l'arme. Lu par aucun type de `game` aujourd'hui (les sons joués
    /// viennent de `ui/feedback.ron`), mais validé pour qu'un chemin cassé ne reste pas
    /// invisible jusqu'au jour où il sera branché.
    /// Pas d'`Option` : en RON, un `Option` exige `Some(...)` ; absent = aucun son.
    #[serde(default)]
    audio_config: WeaponAudioSchema,
    /// D3 : seul `name` est lu (référence vers la table `SpriteSheet`). Pas d'`Option`
    /// (voir `audio_config`) : absent = nom vide = pas de sprite.
    #[serde(default)]
    sprite_config: WeaponSpriteSchema,
}

#[derive(Deserialize, Default)]
struct WeaponSpriteSchema {
    #[serde(default)]
    name: String,
}

/// D3 : `sprites/sprites.ron`, table `{ "id": (animation: "...", layers: { ... }) }`, lue en
/// liste pour rapporter un id répété (comme `weapons.ron`, voir [`KeyedEntries`]).
#[derive(Deserialize)]
struct SpriteSheetsFileSchema(KeyedEntries<SpriteSheetEntrySchema>);

#[derive(Deserialize)]
struct SpriteSheetEntrySchema {
    animation: String,
    layers: BTreeMap<String, String>,
}

/// D3 : le seul champ d'une feuille (`animation::SpriteSheetConfig`) que le lint lit.
#[derive(Deserialize)]
struct SheetImageSchema {
    path: String,
}

#[derive(Deserialize, Default)]
struct WeaponAudioSchema {
    #[serde(default)]
    modes: BTreeMap<String, WeaponAudioModeSchema>,
}

/// `String` (pas `Option<String>`, pour la même raison que `audio_config`) : un champ absent
/// vaut `""`, ignoré par le lint.
#[derive(Deserialize)]
struct WeaponAudioModeSchema {
    #[serde(default)]
    reloading: String,
    #[serde(default)]
    firing: String,
}

#[derive(Deserialize)]
struct WeaponConfigSchema {
    #[serde(default)]
    firing_modes: BTreeMap<String, FiringModeSchema>,
    #[serde(default)]
    test: Option<WeaponTestSchema>,
    /// Type de munition (T2.2, chantier B7). **Pas de `#[serde(default)]`** : le type réel
    /// (`game::weapons::WeaponConfig::ammo_type`) ne l'a pas non plus — une arme à distance
    /// sans `ammo_type` dans son RON échoue déjà à charger comme le type réel, ce mirroir se
    /// contente de reproduire la même erreur ici (`LintErrorKind::Parse`, fichier + message
    /// RON) plutôt que de la laisser silencieusement absente d'un `WeaponEntry` construit
    /// avec des valeurs par défaut. T2.8 : une variante inconnue est rapportée avec le nom
    /// du champ (`de_ammo_type`), un `Custom("")` par `lint::lint_weapons`.
    #[serde(deserialize_with = "de_ammo_type")]
    ammo_type: AmmoType,
    /// T2.8 : mirroité seulement pour que le lint voie une valeur inconnue (enum fermé
    /// `FriendlyFire`, `#[serde(default)]` = `Never` comme le type réel) ; aucune règle
    /// sémantique dessus.
    #[serde(default, deserialize_with = "de_friendly_fire")]
    #[allow(dead_code)]
    friendly_fire: FriendlyFire,
    /// T1.1 : table `projectiles` (voir `WeaponEntry::projectiles`).
    #[serde(default)]
    projectiles: BTreeMap<String, ProjectileDefEntry>,
}

fn zero_spread() -> FixedField {
    FixedField(Fixed::ZERO)
}

#[derive(Deserialize)]
struct FiringModeSchema {
    firing_rate: FixedField,
    /// D51 : voir `WeaponEntry::spreads` (absent : 0, aucune règle violée).
    #[serde(default = "zero_spread")]
    spread: FixedField,
    /// T1.1 : voir `WeaponEntry::mode_projectiles`.
    #[serde(default)]
    projectile: ProjectileSpecEntry,
}

/// Mirroir RON de `game::weapons::WeaponTest` (voir `WeaponTestRange`) : ignore `expect`,
/// que `content` ne peut pas typer (`Expectation` vit dans `game`, `content` n'en dépend
/// pas) — silencieusement, comme tout champ non déclaré ici (voir la doc du module).
#[derive(Deserialize)]
struct WeaponTestSchema {
    frames: u32,
    min_hits: u32,
    #[serde(default)]
    max_hits: Option<u32>,
}

impl From<&WeaponTestSchema> for WeaponTestRange {
    fn from(schema: &WeaponTestSchema) -> Self {
        WeaponTestRange {
            frames: schema.frames,
            min_hits: schema.min_hits,
            max_hits: schema.max_hits,
        }
    }
}

#[derive(Deserialize)]
struct MeleeWeaponsFileSchema(KeyedEntries<MeleeWeaponEntrySchema>);

#[derive(Deserialize)]
struct MeleeWeaponEntrySchema {
    config: MeleeWeaponConfigSchema,
}

#[derive(Deserialize)]
struct MeleeWeaponConfigSchema {
    damage: FixedField,
    #[serde(default)]
    test: Option<WeaponTestSchema>,
    /// T2.8 : voir `WeaponConfigSchema::friendly_fire`.
    #[serde(default, deserialize_with = "de_friendly_fire")]
    #[allow(dead_code)]
    friendly_fire: FriendlyFire,
}

#[derive(Deserialize)]
struct WaveConfigFileSchema {
    #[serde(default)]
    wave_tiers: Vec<WaveTierSchema>,
}

#[derive(Deserialize)]
struct WaveTierSchema {
    #[serde(default)]
    enemy_probabilities: BTreeMap<String, u32>,
}

/// Mirroir de `game::economy::EconomyConfig` (T2.3, chantier C5 v1). Défauts identiques
/// (voir leur doc respective) : un `economy.ron` qui ne déclare qu'un sous-ensemble des
/// champs se comporte pareil ici et à l'exécution. F5 (chantier m0-v11) : `NumOrExpr`.
#[derive(Deserialize)]
struct EconomyFileSchema {
    #[serde(default = "default_kill_points")]
    kill_points: NumOrExpr,
    #[serde(default = "default_hit_points")]
    hit_points: NumOrExpr,
    #[serde(default = "default_repair_points")]
    repair_points: NumOrExpr,
    #[serde(default = "default_nuke_points")]
    nuke_points: NumOrExpr,
    #[serde(default)]
    repair_points_cap_per_wave: Option<NumOrExpr>,
    #[serde(default = "default_refill_price_ratio")]
    refill_price_ratio: NumOrExpr,
}

fn default_kill_points() -> NumOrExpr {
    NumOrExpr::Integer(60)
}

fn default_hit_points() -> NumOrExpr {
    NumOrExpr::Integer(10)
}

fn default_repair_points() -> NumOrExpr {
    NumOrExpr::Integer(10)
}

fn default_nuke_points() -> NumOrExpr {
    NumOrExpr::Integer(400)
}

fn default_refill_price_ratio() -> NumOrExpr {
    NumOrExpr::Literal(Fixed::from_num(0.5))
}

/// Mirroir de `game::economy::PerksConfig` (T2.3, chantier C5 v1) : juste assez pour le lint
/// (`price > 0`, au moins un modificateur, `Mul` > 0, voir `lint::lint_perks`). La table est
/// lue en liste ([`KeyedEntries`], T2.8) : un id écrit deux fois dans le fichier est
/// rapporté `DuplicateId` au lieu d'être écrasé en silence.
#[derive(Deserialize)]
struct PerksFileSchema(KeyedEntries<PerkEntrySchema>);

#[derive(Deserialize)]
struct PerkEntrySchema {
    /// F5 (chantier m0-v11) : littéral ou expression `players` (`crate::expr::NumOrExpr`).
    price: NumOrExpr,
    #[serde(default)]
    modifiers: Vec<PerkModifierSchema>,
}

/// Mirroir de `game::economy::PerkModifierDef` (`value` en [`FixedField`] : même règle de
/// chaîne que le type réel `Fixed`, message plus clair pour un littéral nu).
#[derive(Deserialize)]
struct PerkModifierSchema {
    #[serde(deserialize_with = "de_stat")]
    stat: StatId,
    op: ModifierOp,
    value: FixedField,
}

/// Mirroir de `game::powerups::PowerUpsConfig` (T2.5, chantier C1 v0) : `drop_chance`
/// (racine du fichier, probabilité qu'un ennemi tué laisse tomber un power-up) et la table
/// nommée des power-ups eux-mêmes.
#[derive(Deserialize)]
struct PowerUpsFileSchema {
    drop_chance: FixedField,
    powerups: KeyedEntries<PowerUpEntrySchema>,
}

/// `actions` réutilise le type réel `effects::Action` (pas un mirroir) : comme `StatId`
/// pour `PerkModifierSchema` (voir sa doc), `content` dépend déjà de `effects` (crate de
/// vocabulaire, sans dépendance vers `content`/`game` — aucun cycle). Une référence de
/// `StatId` inconnue dans une `TimedModifier` échoue donc au chargement RON exactement
/// comme ailleurs dans le projet, sans règle de lint dédiée (voir la doc de
/// [`PowerUpEntry`]).
///
/// `pickup_range`/`lifetime_frames` (T2.8) : obligatoires, comme dans le type réel
/// `game::powerups::PowerUpDef` (un fichier qui les omet échoue au chargement du jeu ; le
/// lint rapporte la même erreur plutôt que de laisser passer le fichier).
#[derive(Deserialize)]
struct PowerUpEntrySchema {
    weight: u32,
    pickup_range: FixedField,
    lifetime_frames: u32,
    #[serde(default)]
    actions: Vec<effects::Action>,
}

// ---------------------------------------------------------------------------------------
// Aides de désérialisation (T2.8)
// ---------------------------------------------------------------------------------------

/// Désérialise `T` et préfixe une erreur par le nom du champ RON. Une variante inconnue
/// d'un enum (`ammo_type: Laser`) donne sinon un message RON qui nomme le type
/// (`AmmoType`) mais pas le champ ; avec ce préfixe, le message de lint nomme les deux.
fn with_field_name<'de, D, T>(deserializer: D, field: &str) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map_err(|e| de::Error::custom(format!("champ {field} : {e}")))
}

fn de_ammo_type<'de, D: Deserializer<'de>>(d: D) -> Result<AmmoType, D::Error> {
    with_field_name(d, "ammo_type")
}

fn de_friendly_fire<'de, D: Deserializer<'de>>(d: D) -> Result<FriendlyFire, D::Error> {
    with_field_name(d, "friendly_fire")
}

fn de_stat<'de, D: Deserializer<'de>>(d: D) -> Result<StatId, D::Error> {
    with_field_name(d, "stat")
}

/// Table RON `{ "id": valeur, ... }` lue en liste, dans l'ordre du fichier, **doublons
/// compris** (T2.8). Une `BTreeMap` (comme `serde` le fait pour toute map) garderait la
/// dernière valeur d'une clé répétée sans rien dire ; le chargeur, qui insère les entrées
/// une à une dans le registre, rapporte ainsi le doublon comme `DuplicateId`.
struct KeyedEntries<V>(Vec<(String, V)>);

impl<'de, V: Deserialize<'de>> Deserialize<'de> for KeyedEntries<V> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct EntriesVisitor<V>(PhantomData<V>);

        impl<'de, V: Deserialize<'de>> Visitor<'de> for EntriesVisitor<V> {
            type Value = KeyedEntries<V>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "une table {{ \"id\": (...), ... }}")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut entries = Vec::new();
                while let Some(entry) = map.next_entry::<String, V>()? {
                    entries.push(entry);
                }
                Ok(KeyedEntries(entries))
            }
        }

        deserializer.deserialize_map(EntriesVisitor(PhantomData))
    }
}

// ---------------------------------------------------------------------------------------
// Chargeurs par kind
// ---------------------------------------------------------------------------------------

fn load_characters(
    assets_dir: &ContentFiles,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    let files = match discover_files(assets_dir, decl, "ron") {
        Ok(f) => f,
        Err(e) => {
            errors.push(e);
            return;
        }
    };

    for rel in files {
        let text = match read_file(assets_dir, &rel) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        let parsed: CharacterFileSchema = match ron::from_str(&text) {
            Ok(p) => p,
            Err(e) => {
                errors.push(LintError {
                    kind: LintErrorKind::Parse,
                    file: rel.display().to_string(),
                    message: format!("erreur RON : {e}"),
                });
                continue;
            }
        };

        let id = CharacterId::from(parsed.asset_name_ref.clone());
        if let Some(existing) = registry.characters.get(&id) {
            errors.push(LintError {
                kind: LintErrorKind::DuplicateId,
                file: rel.display().to_string(),
                message: format!(
                    "id de personnage « {id} » (champ asset_name_ref) déjà défini dans {}",
                    existing.file.display()
                ),
            });
            continue;
        }

        registry.characters.insert(
            id.clone(),
            CharacterEntry {
                id,
                file: rel,
                asset_name_ref: parsed.asset_name_ref,
                base_health_max: parsed.base_health.max,
                max_speed: parsed.movement.max_speed,
                starting_skin: parsed.starting_skin,
                skins: parsed.skins.into_keys().collect(),
                starting_weapons: parsed
                    .starting_weapons
                    .into_iter()
                    .map(WeaponId::from)
                    .collect(),
                stats: parsed.stats,
                bleedout_frames: parsed.bleedout_frames,
                revive_frames: parsed.revive_frames,
                downed_speed_mult: parsed.downed_speed_mult,
                weapon_slots: parsed.weapon_slots,
                effects: parsed.effects,
                ignore_tags: parsed
                    .ai
                    .as_ref()
                    .and_then(|ai| ai.targeting.as_ref())
                    .map(|TargetingEntry::Nearest { ignore }| ignore.clone())
                    .unwrap_or_default(),
                has_ai: parsed.ai.is_some(),
                behaviors: parsed.ai.and_then(|ai| ai.behaviors),
                tags: parsed.tags,
                variants: parsed.variants.map(|variants| VariantsEntry {
                    chance: variants.chance,
                    table: variants.table.0,
                }),
                test: parsed.test.map(|test| CharacterTestEntry {
                    frames: test.frames,
                    still: test.still,
                    moving: test.moving,
                    expect_still: test.expect_still.len(),
                    expect_moving: test.expect_moving.len(),
                }),
                counts_hits: parsed.counts_hits,
                body_extent: parsed
                    .collider
                    .as_ref()
                    .map(|collider| collider.extent(parsed.scale.0))
                    .unwrap_or(Fixed::ZERO),
                body_large: parsed
                    .collider
                    .as_ref()
                    .is_some_and(|collider| collider.is_large(parsed.scale.0)),
            },
        );
    }
}

fn load_weapons(
    assets_dir: &ContentFiles,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    let files = match discover_files(assets_dir, decl, "ron") {
        Ok(f) => f,
        Err(e) => {
            errors.push(e);
            return;
        }
    };

    for rel in files {
        let text = match read_file(assets_dir, &rel) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        let parsed: WeaponsFileSchema = match ron::from_str(&text) {
            Ok(p) => p,
            Err(e) => {
                errors.push(LintError {
                    kind: LintErrorKind::Parse,
                    file: rel.display().to_string(),
                    message: format!("erreur RON : {e}"),
                });
                continue;
            }
        };

        for (name, entry) in parsed.0 .0 {
            let id = WeaponId::from(name);
            if let Some(existing) = registry.weapons.get(&id) {
                errors.push(LintError {
                    kind: LintErrorKind::DuplicateId,
                    file: rel.display().to_string(),
                    message: format!(
                        "id d'arme « {id} » déjà défini dans {}",
                        existing.file.display()
                    ),
                });
                continue;
            }
            let test = entry.config.test.as_ref().map(WeaponTestRange::from);
            let mut firing_rates = BTreeMap::new();
            let mut spreads = BTreeMap::new();
            let mut mode_projectiles = BTreeMap::new();
            for (mode, cfg) in entry.config.firing_modes {
                firing_rates.insert(mode.clone(), cfg.firing_rate);
                spreads.insert(mode.clone(), cfg.spread);
                mode_projectiles.insert(mode, cfg.projectile);
            }
            let mut sounds = Vec::new();
            for (mode, audio) in entry.audio_config.modes {
                for (name, path) in [("reloading", audio.reloading), ("firing", audio.firing)] {
                    if !path.is_empty() {
                        sounds.push(SoundRef {
                            field: format!("audio_config.modes.{mode}.{name}"),
                            path,
                        });
                    }
                }
            }
            registry.weapons.insert(
                id.clone(),
                WeaponEntry {
                    id,
                    file: rel.clone(),
                    firing_rates,
                    spreads,
                    test,
                    ammo_type: entry.config.ammo_type,
                    sounds,
                    sprite: Some(entry.sprite_config.name).filter(|name| !name.is_empty()),
                    mode_projectiles,
                    projectiles: entry.config.projectiles,
                },
            );
        }
    }
}

fn load_melee_weapons(
    assets_dir: &ContentFiles,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    let files = match discover_files(assets_dir, decl, "ron") {
        Ok(f) => f,
        Err(e) => {
            errors.push(e);
            return;
        }
    };

    for rel in files {
        let text = match read_file(assets_dir, &rel) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        let parsed: MeleeWeaponsFileSchema = match ron::from_str(&text) {
            Ok(p) => p,
            Err(e) => {
                errors.push(LintError {
                    kind: LintErrorKind::Parse,
                    file: rel.display().to_string(),
                    message: format!("erreur RON : {e}"),
                });
                continue;
            }
        };

        for (name, entry) in parsed.0 .0 {
            let id = MeleeWeaponId::from(name);
            if let Some(existing) = registry.melee_weapons.get(&id) {
                errors.push(LintError {
                    kind: LintErrorKind::DuplicateId,
                    file: rel.display().to_string(),
                    message: format!(
                        "id d'arme de corps à corps « {id} » déjà défini dans {}",
                        existing.file.display()
                    ),
                });
                continue;
            }
            let test = entry.config.test.as_ref().map(WeaponTestRange::from);
            registry.melee_weapons.insert(
                id.clone(),
                MeleeWeaponEntry {
                    id,
                    file: rel.clone(),
                    damage: entry.config.damage,
                    test,
                },
            );
        }
    }
}

fn load_waves(
    assets_dir: &ContentFiles,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    let files = match discover_files(assets_dir, decl, "ron") {
        Ok(f) => f,
        Err(e) => {
            errors.push(e);
            return;
        }
    };

    for rel in files {
        let text = match read_file(assets_dir, &rel) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        let parsed: WaveConfigFileSchema = match ron::from_str(&text) {
            Ok(p) => p,
            Err(e) => {
                errors.push(LintError {
                    kind: LintErrorKind::Parse,
                    file: rel.display().to_string(),
                    message: format!("erreur RON : {e}"),
                });
                continue;
            }
        };

        let id = WaveConfigId::from(
            rel.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string(),
        );
        if let Some(existing) = registry.waves.get(&id) {
            errors.push(LintError {
                kind: LintErrorKind::DuplicateId,
                file: rel.display().to_string(),
                message: format!(
                    "id de vagues « {id} » déjà défini dans {}",
                    existing.file.display()
                ),
            });
            continue;
        }

        let mut enemy_refs = BTreeSet::new();
        for tier in &parsed.wave_tiers {
            for name in tier.enemy_probabilities.keys() {
                enemy_refs.insert(EnemyId::from(name.clone()));
            }
        }

        registry.waves.insert(
            id.clone(),
            WaveConfigEntry {
                id,
                file: rel,
                enemy_refs,
            },
        );
    }
}

/// T1.8 : mêmes règles que [`load_waves`] (id = nom de fichier sans extension).
fn load_clocks(
    assets_dir: &ContentFiles,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    let files = match discover_files(assets_dir, decl, "ron") {
        Ok(f) => f,
        Err(e) => {
            errors.push(e);
            return;
        }
    };
    for rel in files {
        let text = match read_file(assets_dir, &rel) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        let parsed: ClockFileSchema = match ron::from_str(&text) {
            Ok(p) => p,
            Err(e) => {
                errors.push(LintError {
                    kind: LintErrorKind::Parse,
                    file: rel.display().to_string(),
                    message: format!("erreur RON : {e}"),
                });
                continue;
            }
        };
        let id = ClockId::from(
            rel.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string(),
        );
        if let Some(existing) = registry.clocks.get(&id) {
            errors.push(LintError {
                kind: LintErrorKind::DuplicateId,
                file: rel.display().to_string(),
                message: format!(
                    "id d'horloge « {id} » déjà défini dans {}",
                    existing.file.display()
                ),
            });
            continue;
        }
        registry.clocks.insert(
            id.clone(),
            ClockEntry {
                id,
                file: rel,
                scope: parsed.scope,
                events: parsed.events,
            },
        );
    }
}

fn load_difficulty(
    assets_dir: &ContentFiles,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    let files = match discover_files(assets_dir, decl, "ron") {
        Ok(f) => f,
        Err(e) => {
            errors.push(e);
            return;
        }
    };
    for rel in files {
        let text = match read_file(assets_dir, &rel) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        match ron::from_str::<DifficultyFileSchema>(&text) {
            Ok(parsed) => {
                registry.difficulty = Some(DifficultyEntry {
                    file: rel,
                    value: parsed.value,
                })
            }
            Err(e) => errors.push(LintError {
                kind: LintErrorKind::Parse,
                file: rel.display().to_string(),
                message: format!("erreur RON : {e}"),
            }),
        }
    }
}

fn load_patterns(
    assets_dir: &ContentFiles,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    let files = match discover_files(assets_dir, decl, "ron") {
        Ok(f) => f,
        Err(e) => {
            errors.push(e);
            return;
        }
    };

    for rel in files {
        let text = match read_file(assets_dir, &rel) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        let pattern: PatternEntry = match ron::from_str(&text) {
            Ok(p) => p,
            Err(e) => {
                errors.push(LintError {
                    kind: LintErrorKind::Parse,
                    file: rel.display().to_string(),
                    message: format!("erreur RON : {e}"),
                });
                continue;
            }
        };
        let id = PatternId::from(
            rel.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string(),
        );
        if let Some(existing) = registry.patterns.get(&id) {
            errors.push(LintError {
                kind: LintErrorKind::DuplicateId,
                file: rel.display().to_string(),
                message: format!(
                    "id de pattern « {id} » déjà défini dans {}",
                    existing.file.display()
                ),
            });
            continue;
        }
        registry.patterns.insert(
            id.clone(),
            PatternFileEntry {
                id,
                file: rel,
                pattern,
            },
        );
    }
}

fn load_floors(
    assets_dir: &ContentFiles,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    let files = match discover_files(assets_dir, decl, "ron") {
        Ok(f) => f,
        Err(e) => {
            errors.push(e);
            return;
        }
    };

    for rel in files {
        let text = match read_file(assets_dir, &rel) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        let parsed: FloorsFileSchema = match ron::from_str(&text) {
            Ok(p) => p,
            Err(e) => {
                errors.push(LintError {
                    kind: LintErrorKind::Parse,
                    file: rel.display().to_string(),
                    message: format!("erreur RON : {e}"),
                });
                continue;
            }
        };
        let id = FloorsConfigId::from(
            rel.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string(),
        );
        if let Some(existing) = registry.floors.get(&id) {
            errors.push(LintError {
                kind: LintErrorKind::DuplicateId,
                file: rel.display().to_string(),
                message: format!(
                    "id de séquence de niveaux « {id} » déjà défini dans {}",
                    existing.file.display()
                ),
            });
            continue;
        }
        registry.floors.insert(
            id.clone(),
            FloorsEntry {
                id,
                file: rel,
                levels: parsed.levels,
            },
        );
    }
}

/// T1.6 : mêmes règles que [`load_waves`] (id = nom de fichier sans extension) ; le gabarit
/// est cherché dans le dossier déclaré (`<path>/gabarit.ldtk`).
fn load_caves(
    assets_dir: &ContentFiles,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    let files = match discover_files(assets_dir, decl, "ron") {
        Ok(f) => f,
        Err(e) => {
            errors.push(e);
            return;
        }
    };
    let template = format!("{}/{CAVE_TEMPLATE_FILE}", decl.path.trim_end_matches('/'));

    for rel in files {
        let text = match read_file(assets_dir, &rel) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        let config: world::CaveConfig = match ron::from_str(&text) {
            Ok(p) => p,
            Err(e) => {
                errors.push(LintError {
                    kind: LintErrorKind::Parse,
                    file: rel.display().to_string(),
                    message: format!("erreur RON : {e}"),
                });
                continue;
            }
        };
        let id = CaveId::from(
            rel.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string(),
        );
        if let Some(existing) = registry.caves.get(&id) {
            errors.push(LintError {
                kind: LintErrorKind::DuplicateId,
                file: rel.display().to_string(),
                message: format!(
                    "id de caverne « {id} » déjà défini dans {}",
                    existing.file.display()
                ),
            });
            continue;
        }
        registry.caves.insert(
            id.clone(),
            CaveEntry {
                id,
                file: rel,
                config,
                template: template.clone(),
            },
        );
    }
}

/// T1.7 : mêmes règles que [`load_waves`] (id = nom de fichier sans extension).
/// M2-E1 : `rooms/<id>.ron` (kind `Room`), un type de salle par fichier ; id = nom de fichier.
fn load_rooms(
    assets_dir: &ContentFiles,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    for (id, rel, def) in load_ron_files::<world::RoomKindDef>(assets_dir, decl, errors) {
        if let Some(existing) = registry.rooms.get(&id) {
            errors.push(LintError {
                kind: LintErrorKind::DuplicateId,
                file: rel.display().to_string(),
                message: format!(
                    "id de type de salle « {id} » déjà défini dans {}",
                    existing.file.display()
                ),
            });
            continue;
        }
        registry
            .rooms
            .insert(id.clone(), RoomEntry { id, file: rel, def });
    }
}

fn load_surfaces(
    assets_dir: &ContentFiles,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    let files = match discover_files(assets_dir, decl, "ron") {
        Ok(f) => f,
        Err(e) => {
            errors.push(e);
            return;
        }
    };
    for rel in files {
        let text = match read_file(assets_dir, &rel) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        let parsed: SurfaceFileSchema = match ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(&text)
        {
            Ok(p) => p,
            Err(e) => {
                errors.push(LintError {
                    kind: LintErrorKind::Parse,
                    file: rel.display().to_string(),
                    message: format!("erreur RON : {e}"),
                });
                continue;
            }
        };
        let id = SurfaceName::from(
            rel.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string(),
        );
        if let Some(existing) = registry.surfaces.get(&id) {
            errors.push(LintError {
                kind: LintErrorKind::DuplicateId,
                file: rel.display().to_string(),
                message: format!(
                    "id de surface « {id} » déjà défini dans {}",
                    existing.file.display()
                ),
            });
            continue;
        }
        registry.surfaces.insert(
            id.clone(),
            SurfaceEntry {
                id,
                file: rel,
                intgrid_value: parsed.intgrid_value,
                tags: parsed.tags,
                move_speed: parsed.move_speed.get(),
                acceleration: parsed.acceleration.map_or(Fixed::ONE, |a| a.get()),
            },
        );
    }
}

/// Fichiers RON d'un dossier de contenu, parsés en `T` (`implicit_some`), avec leur id (nom de
/// fichier sans extension) ; erreurs de lecture et de RON poussées dans `errors`.
fn load_ron_files<T: serde::de::DeserializeOwned>(
    assets_dir: &ContentFiles,
    decl: &ContentFolderDecl,
    errors: &mut Vec<LintError>,
) -> Vec<(String, PathBuf, T)> {
    let files = match discover_files(assets_dir, decl, "ron") {
        Ok(f) => f,
        Err(e) => {
            errors.push(e);
            return Vec::new();
        }
    };
    let mut parsed_files = Vec::new();
    for rel in files {
        let text = match read_file(assets_dir, &rel) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        match ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str::<T>(&text)
        {
            Ok(parsed) => {
                let id = rel
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string();
                parsed_files.push((id, rel, parsed));
            }
            Err(e) => errors.push(LintError {
                kind: LintErrorKind::Parse,
                file: rel.display().to_string(),
                message: format!("erreur RON : {e}"),
            }),
        }
    }
    parsed_files
}

/// T1.10 : mêmes règles que [`load_waves`] (id = nom de fichier sans extension).
fn load_progression(
    assets_dir: &ContentFiles,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    for (id, rel, parsed) in load_ron_files::<ProgressionFileSchema>(assets_dir, decl, errors) {
        let id = ProgressionId::from(id);
        if let Some(existing) = registry.progression.get(&id) {
            errors.push(LintError {
                kind: LintErrorKind::DuplicateId,
                file: rel.display().to_string(),
                message: format!(
                    "id de progression « {id} » déjà défini dans {}",
                    existing.file.display()
                ),
            });
            continue;
        }
        registry.progression.insert(
            id.clone(),
            ProgressionEntry {
                id,
                file: rel,
                gauge: parsed.gauge,
                per_kill: parsed.per_kill.get(),
                levels: parsed.levels.iter().map(|l| l.get()).collect(),
                choices: parsed.choices,
                choice_frames: parsed.choice_frames,
                mutations: parsed.mutations,
                weapon_pool: parsed.weapon_pool,
                weapon_drop_chance: parsed.weapon_drop_chance.map_or(Fixed::ZERO, |c| c.get()),
            },
        );
    }
}

/// T1.3 : mêmes règles que [`load_waves`] (id = nom de fichier sans extension).
fn load_statuses(
    assets_dir: &ContentFiles,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    for (id, rel, parsed) in load_ron_files::<StatusFileSchema>(assets_dir, decl, errors) {
        let id = StatusId::from(id);
        if let Some(existing) = registry.statuses.get(&id) {
            errors.push(LintError {
                kind: LintErrorKind::DuplicateId,
                file: rel.display().to_string(),
                message: format!(
                    "id de statut « {id} » déjà défini dans {}",
                    existing.file.display()
                ),
            });
            continue;
        }
        registry.statuses.insert(
            id.clone(),
            StatusContentEntry {
                id,
                file: rel,
                kind: parsed.kind,
                frames: parsed.frames,
                damage: parsed.damage.map_or(Fixed::ZERO, |d| d.get()),
                period: parsed.period.unwrap_or(0),
                factor: parsed.factor.map_or(Fixed::ONE, |f| f.get()),
            },
        );
    }
}

/// T1.10 : mêmes règles que [`load_waves`] (id = nom de fichier sans extension).
fn load_mutations(
    assets_dir: &ContentFiles,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    for (id, rel, parsed) in load_ron_files::<MutationFileSchema>(assets_dir, decl, errors) {
        let id = MutationId::from(id);
        if let Some(existing) = registry.mutations.get(&id) {
            errors.push(LintError {
                kind: LintErrorKind::DuplicateId,
                file: rel.display().to_string(),
                message: format!(
                    "id de mutation « {id} » déjà défini dans {}",
                    existing.file.display()
                ),
            });
            continue;
        }
        registry.mutations.insert(
            id.clone(),
            MutationEntry {
                id,
                file: rel,
                name: parsed.name,
                weight: parsed.weight,
                tags: parsed.tags,
                max_stacks: parsed.max_stacks,
                effects: parsed.effects,
            },
        );
    }
}

/// T2.3, chantier C5 v1 : mêmes règles que [`load_waves`] (id = nom de fichier sans
/// extension, un fichier par jeu en pratique mais pas imposé).
fn load_economy(
    assets_dir: &ContentFiles,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    let files = match discover_files(assets_dir, decl, "ron") {
        Ok(f) => f,
        Err(e) => {
            errors.push(e);
            return;
        }
    };

    for rel in files {
        let text = match read_file(assets_dir, &rel) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        let parsed: EconomyFileSchema = match ron::from_str(&text) {
            Ok(p) => p,
            Err(e) => {
                errors.push(LintError {
                    kind: LintErrorKind::Parse,
                    file: rel.display().to_string(),
                    message: format!("erreur RON : {e}"),
                });
                continue;
            }
        };

        let id = EconomyId::from(
            rel.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string(),
        );
        if let Some(existing) = registry.economy.get(&id) {
            errors.push(LintError {
                kind: LintErrorKind::DuplicateId,
                file: rel.display().to_string(),
                message: format!(
                    "id d'économie « {id} » déjà défini dans {}",
                    existing.file.display()
                ),
            });
            continue;
        }

        registry.economy.insert(
            id.clone(),
            EconomyEntry {
                id,
                file: rel,
                kill_points: parsed.kill_points,
                hit_points: parsed.hit_points,
                repair_points: parsed.repair_points,
                nuke_points: parsed.nuke_points,
                repair_points_cap_per_wave: parsed.repair_points_cap_per_wave,
                refill_price_ratio: parsed.refill_price_ratio,
            },
        );
    }
}

/// T2.3, chantier C5 v1 : table de perks (même forme que [`load_weapons`], une entrée par
/// clé du fichier).
fn load_perks(
    assets_dir: &ContentFiles,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    let files = match discover_files(assets_dir, decl, "ron") {
        Ok(f) => f,
        Err(e) => {
            errors.push(e);
            return;
        }
    };

    for rel in files {
        let text = match read_file(assets_dir, &rel) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        let parsed: PerksFileSchema = match ron::from_str(&text) {
            Ok(p) => p,
            Err(e) => {
                errors.push(LintError {
                    kind: LintErrorKind::Parse,
                    file: rel.display().to_string(),
                    message: format!("erreur RON : {e}"),
                });
                continue;
            }
        };

        for (name, entry) in parsed.0 .0 {
            let id = PerkId::from(name);
            if let Some(existing) = registry.perks.get(&id) {
                errors.push(LintError {
                    kind: LintErrorKind::DuplicateId,
                    file: rel.display().to_string(),
                    message: format!(
                        "id de perk « {id} » déjà défini dans {}",
                        existing.file.display()
                    ),
                });
                continue;
            }
            let modifiers = entry
                .modifiers
                .into_iter()
                .map(|m| PerkModifierEntry {
                    stat: m.stat,
                    op: m.op,
                    value: m.value,
                })
                .collect();
            registry.perks.insert(
                id.clone(),
                PerkEntry {
                    id,
                    file: rel.clone(),
                    price: entry.price,
                    modifiers,
                },
            );
        }
    }
}

/// D3 : table des feuilles de sprites (kind `SpriteSheet`). Un id répété (dans un fichier
/// ou entre deux fichiers) est un `DuplicateId`, la première entrée est gardée. L'image de
/// chaque feuille lisible est relevée pour le lint (fichier présent sous `assets/`).
fn load_sprite_sheets(
    assets_dir: &ContentFiles,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    let files = match discover_files(assets_dir, decl, "ron") {
        Ok(f) => f,
        Err(e) => {
            errors.push(e);
            return;
        }
    };

    for rel in files {
        let text = match read_file(assets_dir, &rel) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        let parsed: SpriteSheetsFileSchema = match ron::from_str(&text) {
            Ok(p) => p,
            Err(e) => {
                errors.push(LintError {
                    kind: LintErrorKind::Parse,
                    file: rel.display().to_string(),
                    message: format!("erreur RON : {e}"),
                });
                continue;
            }
        };

        for (name, entry) in parsed.0 .0 {
            let id = SpriteSheetId::from(name);
            if let Some(existing) = registry.sprite_sheets.get(&id) {
                errors.push(LintError {
                    kind: LintErrorKind::DuplicateId,
                    file: rel.display().to_string(),
                    message: format!(
                        "id de feuille de sprites « {id} » déjà défini dans {}",
                        existing.file.display()
                    ),
                });
                continue;
            }
            let images = entry
                .layers
                .iter()
                .filter_map(|(layer, sheet)| {
                    let text = assets_dir.read(Path::new(sheet)).ok()?;
                    let sheet: SheetImageSchema = ron::from_str(&text).ok()?;
                    Some((layer.clone(), sheet.path))
                })
                .collect();
            registry.sprite_sheets.insert(
                id.clone(),
                SpriteSheetEntry {
                    id,
                    file: rel.clone(),
                    animation: entry.animation,
                    layers: entry.layers,
                    images,
                },
            );
        }
    }
}

/// T2.5, chantier C1 v0 : `drop_chance` (racine) et la table des power-ups (une entrée par
/// clé, même forme que [`load_perks`]) d'un seul fichier `items/powerups.ron`. Un dossier
/// `PowerUp` qui contient plusieurs fichiers (pas le cas des jeux réels, seulement la
/// fixture `powerup_duplicate_id`, voir `tests/fixtures/`) : chaque fichier contribue ses
/// propres entrées à `registry.powerups` (id dupliqué entre fichiers = erreur, comme
/// `load_perks`), et `registry.powerup_drop_chance` retient le dernier fichier traité (voir
/// sa doc).
fn load_powerups(
    assets_dir: &ContentFiles,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    let files = match discover_files(assets_dir, decl, "ron") {
        Ok(f) => f,
        Err(e) => {
            errors.push(e);
            return;
        }
    };

    for rel in files {
        let text = match read_file(assets_dir, &rel) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        let parsed: PowerUpsFileSchema = match ron::from_str(&text) {
            Ok(p) => p,
            Err(e) => {
                errors.push(LintError {
                    kind: LintErrorKind::Parse,
                    file: rel.display().to_string(),
                    message: format!("erreur RON : {e}"),
                });
                continue;
            }
        };

        registry.powerup_drop_chance = Some(PowerUpDropChanceEntry {
            drop_chance: parsed.drop_chance,
            file: rel.clone(),
        });

        for (name, entry) in parsed.powerups.0 {
            let id = PowerUpId::from(name);
            if let Some(existing) = registry.powerups.get(&id) {
                errors.push(LintError {
                    kind: LintErrorKind::DuplicateId,
                    file: rel.display().to_string(),
                    message: format!(
                        "id de power-up « {id} » déjà défini dans {}",
                        existing.file.display()
                    ),
                });
                continue;
            }
            registry.powerups.insert(
                id.clone(),
                PowerUpEntry {
                    id,
                    file: rel.clone(),
                    weight: entry.weight,
                    pickup_range: entry.pickup_range,
                    lifetime_frames: entry.lifetime_frames,
                    actions: entry.actions,
                },
            );
        }
    }
}

fn load_maps(
    assets_dir: &ContentFiles,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    let files = match discover_files(assets_dir, decl, "ldtk") {
        Ok(f) => f,
        Err(e) => {
            errors.push(e);
            return;
        }
    };

    for rel in files {
        let id = map_id_from_path(rel.to_str().unwrap_or_default());
        if let Some(existing) = registry.maps.get(&id) {
            errors.push(LintError {
                kind: LintErrorKind::DuplicateId,
                file: rel.display().to_string(),
                message: format!(
                    "id de carte « {id} » déjà défini dans {}",
                    existing.file.display()
                ),
            });
            continue;
        }
        let text = read_file(assets_dir, &rel).ok();
        let forced_variants = text
            .as_deref()
            .map(ldtk_forced_variants)
            .unwrap_or_default();
        let spawned_characters = text
            .as_deref()
            .map(ldtk_character_spawns)
            .unwrap_or_default();
        let room_kinds = text.as_deref().map(ldtk_room_kinds).unwrap_or_default();
        registry.maps.insert(
            id.clone(),
            MapEntry {
                id,
                file: rel,
                forced_variants,
                spawned_characters,
                room_kinds,
            },
        );
    }
}

/// Les réglages `feedback.ron` déclarés comme Ui ont un schéma typé (T3.4), comme
/// `mutation_screen.ron` (T1.16, [`crate::ui::MutationScreenLayout`]).
/// Les autres fichiers Ui gardent la validation syntaxique.
fn load_ui(
    assets_dir: &ContentFiles,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    let files = match discover_files(assets_dir, decl, "ron") {
        Ok(files) => files,
        Err(error) => {
            errors.push(error);
            return;
        }
    };
    for rel in files {
        let text = match read_file(assets_dir, &rel) {
            Ok(text) => text,
            Err(error) => {
                errors.push(error);
                continue;
            }
        };
        let parsed = if rel.file_name().and_then(|name| name.to_str()) == Some("feedback.ron") {
            ron::from_str::<crate::feedback::FeedbackSettings>(&text).map(|settings| {
                registry.feedback.push(FeedbackEntry {
                    file: rel.clone(),
                    settings,
                });
            })
        } else if rel.file_name().and_then(|name| name.to_str()) == Some(crate::ui::HUD_FILE_NAME) {
            ron::from_str::<crate::ui::HudFileSchema>(&text).map(|hud| {
                let sources = hud
                    .widgets
                    .iter()
                    .map(|w| w.kind.source().to_string())
                    .collect();
                registry.hud_sources.push((rel.clone(), sources));
            })
        } else if rel.file_name().and_then(|name| name.to_str())
            == Some(crate::ui::MUTATION_SCREEN_FILE_NAME)
        {
            ron::from_str::<crate::ui::MutationScreenLayout>(&text).map(|layout| {
                registry.mutation_screens.push((rel.clone(), layout));
            })
        } else {
            ron::from_str::<ron::Value>(&text).map(|_| ())
        };
        match parsed {
            Ok(()) => registry.ui_files.push(rel),
            Err(error) => errors.push(LintError {
                kind: LintErrorKind::Parse,
                file: rel.display().to_string(),
                message: format!("erreur RON : {error}"),
            }),
        }
    }
}

/// `Camera` : seule la validité syntaxique RON est vérifiée (pas de schéma typé,
/// ces fichiers ne sont référencés par id par aucun autre contenu aujourd'hui).
fn load_generic_ron(
    assets_dir: &ContentFiles,
    decl: &ContentFolderDecl,
    into: &mut Vec<PathBuf>,
    errors: &mut Vec<LintError>,
) {
    let files = match discover_files(assets_dir, decl, "ron") {
        Ok(f) => f,
        Err(e) => {
            errors.push(e);
            return;
        }
    };

    for rel in files {
        let text = match read_file(assets_dir, &rel) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        match ron::from_str::<ron::Value>(&text) {
            Ok(_) => into.push(rel),
            Err(e) => errors.push(LintError {
                kind: LintErrorKind::Parse,
                file: rel.display().to_string(),
                message: format!("erreur RON : {e}"),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_kinds_cover_dispatch_table() {
        let kinds = known_content_kinds();
        for name in KNOWN_KIND_NAMES {
            assert!(kinds.has("content_folder", name));
        }
        assert!(!kinds.has("content_folder", "Npc"));
    }

    #[test]
    fn map_id_from_path_strips_dir_and_extension() {
        assert_eq!(
            map_id_from_path("exemples/test_map.ldtk").as_str(),
            "test_map"
        );
        assert_eq!(
            map_id_from_path("testbed/testbed_empty.ldtk").as_str(),
            "testbed_empty"
        );
    }
}

/// D41 : dégagement des points d'apparition selon le corps en jeu.
#[cfg(test)]
mod spawn_clearance_tests {
    use super::*;

    fn extent_of(ron_collider: &str, scale: f64) -> Fixed {
        let collider: ColliderSchema = ron::from_str(ron_collider).unwrap();
        collider.extent(Fixed::from_num(scale))
    }

    #[test]
    fn etendue_en_jeu_avec_offset_et_echelle() {
        let corps = r#"(shape: Rectangle(width: "20.", height: "20."), offset: (x: "0.0", y: "-6.0", z: "0.0"))"#;
        // Zombie / ennemi de 20 px : 10 + 6.
        assert_eq!(extent_of(corps, 1.0), Fixed::from_num(16));
        // Boss : 20 × 1.4 = 28 px en jeu, 16 × 1.4 = 22.4.
        assert!((extent_of(corps, 1.4).to_num::<f64>() - 22.4).abs() < 0.01);
        let gros = r#"(shape: Rectangle(width: "28.", height: "28."), offset: (x: "0.0", y: "-6.0", z: "0.0"))"#;
        // 28 × 1.4 = 39 px en jeu : (14 + 6) × 1.4 = 28.
        assert!((extent_of(gros, 1.4).to_num::<f64>() - 28.0).abs() < 0.01);
    }

    #[test]
    fn degagement_selon_l_etendue() {
        assert_eq!(spawn_clearance_for(Fixed::ZERO), 1);
        assert_eq!(spawn_clearance_for(Fixed::from_num(16)), 1);
        assert_eq!(spawn_clearance_for(Fixed::from_num(24)), 1);
        assert_eq!(spawn_clearance_for(Fixed::from_num(22.4)), 1);
        assert_eq!(spawn_clearance_for(Fixed::from_num(28)), 2);
        assert_eq!(spawn_clearance_for(Fixed::from_num(40)), 2);
        assert_eq!(spawn_clearance_for(Fixed::from_num(41)), 3);
    }
}
