use bevy::{platform::collections::hash_map::HashMap, prelude::*, reflect::TypePath};
use bevy_fixed::fixed_math;
use serde::Deserialize;
use sim_core::stats::StatId;
use sim_core::tag::{Tag, Tags};
use std::collections::BTreeMap;

use crate::{character::movement::MovementConfig, collider::ColliderConfig};

use super::health::HealthConfig;

#[derive(Debug, Deserialize, Clone)]
pub struct CharacterSkin {
    pub layers: HashMap<String, String>,
}

#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct CharacterConfig {
    pub movement: MovementConfig,

    pub asset_name_ref: String,

    pub base_health: HealthConfig,

    pub collider: ColliderConfig,

    pub scale: fixed_math::Fixed,

    pub starting_skin: String,
    pub skins: HashMap<String, CharacterSkin>,

    /// Tags du personnage (T1.1, chantier B1 « Équipes et dégâts »), ex. `["cursed"]` ou
    /// `["zombie"]`. Posés en composant `sim_core::tag::Tags` à la création
    /// (`character::create::create_character`) ; union avec le tag de genre d'attaque
    /// (`bullet`/`melee`) dans chaque `DamageEvent` émis par ce personnage (voir
    /// `sim_core::damage::DamageEvent::tags`). Vide par défaut : n'affecte pas les
    /// personnages existants.
    #[serde(default)]
    pub tags: Tags,

    /// Tags de dégât contre lesquels ce personnage est totalement immunisé (ex.
    /// `["bullet"]`). Posés en composant `combat::damage::Defenses` à la création. Vide par
    /// défaut.
    #[serde(default)]
    pub immune_to: Tags,

    /// Multiplicateur de dégât par tag (`{"bullet": "0.5"}` = moitié dégât des sources
    /// taguées `bullet`). Posé en composant `combat::damage::Defenses` à la création. Vide
    /// par défaut (aucune résistance).
    #[serde(default)]
    pub resistances: BTreeMap<Tag, fixed_math::Fixed>,
    /// Armes à distance données au spawn (`WeaponId` de `weapons.ron`), dans l'ordre
    /// déclaré : la première est l'arme active (T1.5, voir
    /// `character/player/create.rs::create_player`). Vide par défaut : seuls les
    /// personnages joueurs en déclarent aujourd'hui (les ennemis n'ont pas d'arme à
    /// distance). Validé par `crates/content::lint` (référence vers une entrée de
    /// `weapons.ron`).
    #[serde(default)]
    pub starting_weapons: Vec<String>,

    /// Surcharges de stats (T1.2, chantier B2) : appliquées par-dessus les valeurs de base
    /// dérivées des champs ci-dessus (voir `character::create::create_character`), qui
    /// restent la source de vérité pour un personnage qui ne déclare rien ici — ce champ
    /// vide par défaut ne change donc aucun personnage existant. Bornes validées par
    /// `content::lint` (valeurs >= 0).
    #[serde(default)]
    pub stats: BTreeMap<StatId, fixed_math::Fixed>,
}

#[derive(Component)]
pub struct CharacterConfigHandles {
    pub config: Handle<CharacterConfig>,
}
