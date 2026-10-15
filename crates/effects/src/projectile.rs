//! Contrats de projectiles (B5) : modificateurs, patterns, actions de fin et leur spec.
//!
//! Définis par T1.0a/T1.1/T1.2/M2-T1 dans `combat::projectile`, **déplacés ici par M3-T0a**
//! (`docs/conventions.md` §60) pour qu'un objet (`items::ItemDef::projectile`, crate de
//! simulation sans rendu) puisse les porter sans dépendre de `combat` (crate encore en dette
//! D69). `combat::projectile` les réexporte : chemins, `Hash`, `Debug` et RON inchangés (un
//! `derive` ne dépend pas du crate qui définit le type), aucune trace ne bouge. L'exécution
//! (`PatternLibrary`, `Projectile`, systèmes) reste dans `combat`.

use bevy_fixed::fixed_math::Fixed;
use serde::{Deserialize, Serialize};

use crate::Action;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub enum ProjectileModifier {
    Bounce(u32),
    Pierce(u32),
    Size(Fixed),
    Lifetime(u32),
    Homing(Fixed),
    Gravity(Fixed),
    // M3-T0a (`docs/conventions.md` §60) : contrats, **en fin d'enum** (le `derive(Hash)` hache
    // l'index de la variante : les six d'avant gardent le leur, le hash des armes existantes ne
    // bouge pas). Acceptés par le lint dans le RON (formes, plages) mais une arme qui les porte
    // est refusée (`Unsupported`) tant que M3-T1 ne les exécute pas.
    /// Traverse les murs et les obstacles (pas les personnages).
    Spectral,
    /// À l'impact sur un personnage, `count` projectiles `child` en éventail de `spread`
    /// radians : `child` est un id de la table `projectiles` de l'arme, ou, absent, le
    /// projectile par défaut du porteur (Q3).
    Split {
        count: u32,
        spread: Fixed,
        #[serde(default)]
        child: Option<String>,
    },
    /// Tourne autour du tireur à `radius` unités, un tour en `frames_per_turn` frames, au lieu
    /// d'avancer.
    Orbit {
        radius: Fixed,
        frames_per_turn: u32,
    },
}

impl ProjectileModifier {
    /// Nom de la variante (kind `projectile_modifier`, messages du lint).
    pub fn name(&self) -> &'static str {
        match self {
            Self::Bounce(_) => "Bounce",
            Self::Pierce(_) => "Pierce",
            Self::Size(_) => "Size",
            Self::Lifetime(_) => "Lifetime",
            Self::Homing(_) => "Homing",
            Self::Gravity(_) => "Gravity",
            Self::Spectral => "Spectral",
            Self::Split { .. } => "Split",
            Self::Orbit { .. } => "Orbit",
        }
    }

    /// Contrat déclaré mais pas encore exécuté au tir (M3-T1) : le lint refuse une arme qui le
    /// porte (`Unsupported`).
    pub fn is_contract_only(&self) -> bool {
        matches!(
            self,
            Self::Spectral | Self::Split { .. } | Self::Orbit { .. }
        )
    }
}

/// Les projectiles sont référencés par leur id de contenu. Les angles sont en radians,
/// les vitesses en unités/seconde, les durées en frames. Sequence conserve l'ordre RON.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub enum Pattern {
    Aimed {
        count: u32,
        spread: Fixed,
        projectile: String,
    },
    Spread {
        count: u32,
        spread: Fixed,
        projectile: String,
    },
    Ring {
        count: u32,
        speed: Fixed,
        projectile: String,
        every: u32,
    },
    Sequence(Vec<Pattern>),
    Telegraph(u32),
    Wait(u32),
    // T1.2 : variantes ajoutées **en fin d'enum** — le `derive(Hash)` hache l'index de la
    // variante, les variantes existantes gardent le leur (hash des armes et projectiles
    // existants inchangé).
    /// `count` tirs à des angles tirés au hasard dans `±spread/2` autour de la visée (flux
    /// RNG `"patterns"`, émetteurs seulement : refusé en `on_expire` par le lint).
    Scatter {
        count: u32,
        spread: Fixed,
        projectile: String,
    },
    /// Pattern nommé du contenu (`patterns/<nom>.ron`, kind `Pattern`), résolu par
    /// [`PatternLibrary`] (nom inconnu : erreur de lint, jamais de panique en jeu).
    Named(String),
    // M2-T1 : encore en fin d'enum (hash des variantes précédentes inchangé).
    /// Éventail de `count` tirs répartis exactement sur `spread` radians (bornes comprises,
    /// sans hasard, à la différence de `Scatter`), centré sur la visée ; `speed` > 0 remplace
    /// celle de la définition du projectile.
    Fan {
        count: u32,
        spread: Fixed,
        speed: Fixed,
        projectile: String,
    },
    /// Spirale (émetteurs seulement) : `shots` salves espacées de `every` frames, chacune de
    /// `count` bras régulièrement répartis sur le cercle ; la salve `k` tourne de `k * step`
    /// radians par rapport à la visée. `speed` > 0 remplace celle de la définition.
    Spiral {
        count: u32,
        step: Fixed,
        shots: u32,
        every: u32,
        speed: Fixed,
        projectile: String,
    },
    /// Salves (émetteurs seulement) : `count` salves de `per_burst` tirs en éventail de
    /// `spread` radians centré sur la visée, espacées de `interval` frames.
    Burst {
        count: u32,
        per_burst: u32,
        interval: u32,
        spread: Fixed,
        projectile: String,
    },
}

impl Pattern {
    /// Vrai si le pattern (ou un de ses enfants) contient une étape temporelle ou aléatoire
    /// (`Telegraph`, `Wait`, `Scatter`, `Ring.every > 0` n'en est pas une : ignoré en
    /// `on_expire`). `Named` n'est pas résolu ici.
    pub fn has_emitter_only_step(&self) -> bool {
        match self {
            Pattern::Telegraph(_)
            | Pattern::Wait(_)
            | Pattern::Scatter { .. }
            | Pattern::Spiral { .. }
            | Pattern::Burst { .. } => true,
            Pattern::Sequence(children) => children.iter().any(Pattern::has_emitter_only_step),
            _ => false,
        }
    }
}

/// Action déclenchée à la fin d'un projectile (`on_expire`). `Hash` manuel (voir plus bas).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ExpireAction {
    /// Fait naître le pattern au point de fin du projectile. Seuls les patterns instantanés
    /// sont joués (`Aimed`, `Spread`, `Ring` une seule fois, `Sequence` de ceux-ci) ;
    /// `Telegraph`/`Wait` appartiennent aux émetteurs (T1.2) et sont refusés par le lint.
    Spawn(Pattern),
    /// Creuse le terrain d'une caverne au point de fin (T1.6, `effects::Action::DestroyTerrain`
    /// : `Rock` à moins de `radius` → `Floor`).
    DestroyTerrain { radius: Fixed },
}

/// `#[derive(Hash)]` d'un enum à **une seule** variante n'écrit pas le discriminant : tant que
/// `Spawn` était seule, son hash était celui du pattern. Ajouter une variante dérivée ferait
/// écrire le discriminant et déplacerait le checksum de toute arme à `on_expire` (config
/// hachée dès la frame 0, trace `weapon_grenade`). `Spawn` garde donc exactement son hash
/// historique ; `DestroyTerrain` écrit un marqueur puis son rayon.
impl std::hash::Hash for ExpireAction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            ExpireAction::Spawn(pattern) => pattern.hash(state),
            ExpireAction::DestroyTerrain { radius } => {
                "DestroyTerrain".hash(state);
                radius.hash(state);
            }
        }
    }
}

/// Comportement composable d'un projectile, tel que déclaré dans un mode de tir. Vide (tous
/// les champs absents) : balle ordinaire, sans `combat::projectile::Projectile`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectileSpec {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<ProjectileModifier>,
    /// Actions posées sur chaque personnage touché (`effects::Action` à modificateur :
    /// `TimedModifier`, `CurrencyMultiplier` ; les autres sont refusées par le lint).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub on_hit: Vec<Action>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub on_expire: Vec<ExpireAction>,
}

impl ProjectileSpec {
    pub fn is_empty(&self) -> bool {
        self.modifiers.is_empty() && self.on_hit.is_empty() && self.on_expire.is_empty()
    }
}

/// M3-T0a : modificateurs de projectile **portés par un objet** (`items::ItemDef::projectile`,
/// `items::SynergyDef::projectile`), ajoutés aux tirs du porteur. Même forme que la spec d'un
/// mode de tir (`modifiers`, `on_hit`, `on_expire`) ; composés avec ceux de l'arme par
/// `combat::projectile::compose` (M3-T1 les lira au tir).
pub type ProjectileMods = ProjectileSpec;
