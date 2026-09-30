use bevy::{platform::collections::hash_map::HashMap, prelude::*, reflect::TypePath};
use bevy_fixed::fixed_math;
use serde::Deserialize;
use sim_core::stats::StatId;
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

    /// Surcharges de stats (T1.2, chantier B2) : appliquées par-dessus les valeurs de base
    /// dérivées des champs ci-dessus (voir `character::create::create_character`), qui
    /// restent la source de vérité pour un personnage qui ne déclare rien ici — ce champ
    /// vide par défaut ne change donc aucun personnage existant. Bornes validées par
    /// `content::lint` (valeurs >= 0).
    #[serde(default)]
    pub stats: BTreeMap<StatId, fixed_math::Fixed>,

    /// À terre (T1.3, chantier B6) : frames de saignement avant `Death` pour un **joueur**
    /// tombé à terre (`combat::downed::Downed`) sans avoir été réanimé. Défaut 1800 (30 s à
    /// 60 FPS). Sans effet sur un personnage qui ne peut pas tomber à terre (ennemis :
    /// `character::health::rollback_apply_accumulated_damage` ne pose `Downed` que sur un
    /// `Player`). Borné par `content::lint` (> 0).
    #[serde(default = "default_bleedout_frames")]
    pub bleedout_frames: u32,
    /// Frames d'interaction maintenue nécessaires pour réanimer ce personnage une fois à
    /// terre (lues sur la config du joueur **à terre**, pas de celui qui réanime — voir
    /// `interaction::handle_revive_interaction`). Défaut 180 (3 s à 60 FPS). Borné par
    /// `content::lint` (> 0).
    #[serde(default = "default_revive_frames")]
    pub revive_frames: u32,
    /// Multiplicateur de vitesse de déplacement pendant qu'il est à terre : posé comme
    /// modificateur `StatId::MoveSpeed`/`ModifierOp::Mul`, source
    /// `combat::downed::downed_modifier_source()` (T1.2). Défaut 0.3. Borné par
    /// `content::lint` (`0 < downed_speed_mult <= 1`).
    #[serde(default = "default_downed_speed_mult")]
    pub downed_speed_mult: fixed_math::Fixed,

    /// Emplacements d'armes à distance (T2.2, chantier B7 « Munitions typées et inventaire
    /// d'armes ») : au-delà, ramasser une arme remplace l'arme active plutôt que
    /// l'ajouter — voir `interaction::handle_weapon_pickup_interaction`. Défaut 2. Le
    /// personnage joueur `zombies` le porte à 3 explicitement (`starting_weapons` en
    /// déclare trois) ; le testbed garde le défaut. Borné par `content::lint` (`> 0`).
    #[serde(default = "default_weapon_slots")]
    pub weapon_slots: u32,
}

fn default_bleedout_frames() -> u32 {
    1800
}

fn default_revive_frames() -> u32 {
    180
}

fn default_downed_speed_mult() -> fixed_math::Fixed {
    fixed_math::new(0.3)
}

fn default_weapon_slots() -> u32 {
    2
}

#[derive(Component)]
pub struct CharacterConfigHandles {
    pub config: Handle<CharacterConfig>,
}
