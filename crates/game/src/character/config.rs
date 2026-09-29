use bevy::{platform::collections::hash_map::HashMap, prelude::*, reflect::TypePath};
use bevy_fixed::fixed_math;
use serde::Deserialize;
use sim_core::tag::{Tag, Tags};
use sim_core::team::Team;
use std::collections::BTreeMap;

use crate::{
    character::enemy::ai::EnemyAiConfigRon, character::movement::MovementConfig,
    collider::ColliderConfig,
};

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

    /// Équipe par défaut d'un personnage spawné via `spawn_enemy` sans `team` explicite sur
    /// l'entité `CharacterSpawn` qui l'a créé (T2.9, testbed). `None` pour tout le contenu
    /// zombies existant (comportement inchangé : `Team::Enemies` en dur, voir
    /// `character::enemy::create::spawn_enemy`).
    #[serde(default)]
    pub team: Option<Team>,

    /// Configuration d'IA (mouvement, portées, obstacles) de ce personnage quand il est
    /// spawné comme ennemi (T2.9, testbed : `dummy`/`target`/`follower`/etc. ont besoin d'un
    /// comportement différent du zombie standard). `None` pour tout le contenu zombies
    /// existant : `spawn_enemy` retombe alors sur `EnemyAiConfig::zombie()` comme avant.
    #[serde(default)]
    pub ai: Option<EnemyAiConfigRon>,

    /// Si vrai, ce personnage reçoit un composant `HitCount` (rollback) à sa création,
    /// incrémenté par le résolveur de dégâts pour chaque coup reçu (T2.9, testbed : la
    /// cible `target`). Faux par défaut : ne s'applique à aucun personnage zombie/joueur
    /// existant.
    #[serde(default)]
    pub counts_hits: bool,
}

#[derive(Component)]
pub struct CharacterConfigHandles {
    pub config: Handle<CharacterConfig>,
}
