use bevy::{platform::collections::hash_map::HashMap, prelude::*};
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

/// `test:` d'un personnage (T1.13) : `(frames: 600, still: true, moving: true, expect_still:
/// [...], expect_moving: [...])`. Un gabarit actif sans attente explicite reçoit les attentes
/// par défaut (voir `scenario::generate::enemy_default_expectations`). Validé par
/// `content::lint` (`frames > 0`, au moins un gabarit, attentes seulement d'un gabarit actif).
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct CharacterTest {
    /// Durée de chaque scénario généré.
    pub frames: u32,
    /// Gabarit joueur immobile (défaut vrai).
    #[serde(default = "enabled")]
    pub still: bool,
    /// Gabarit joueur mobile (défaut vrai).
    #[serde(default = "enabled")]
    pub moving: bool,
    #[serde(default)]
    pub expect_still: Vec<combat::weapons::expectations::Expectation>,
    #[serde(default)]
    pub expect_moving: Vec<combat::weapons::expectations::Expectation>,
    /// M2-T17 (§54) : gabarit `BossFight`, pour un personnage qui a `boss:`.
    #[serde(default)]
    pub boss: Option<crate::gen_test::BossFightTest>,
    /// M3-T13 (§78) : gabarits `ChampionForced`, une entrée par variante testée.
    #[serde(default)]
    pub champions: Vec<crate::gen_test::ChampionTest>,
}

fn enabled() -> bool {
    true
}

#[derive(Debug, Deserialize, Clone)]
pub struct CharacterSkin {
    pub layers: HashMap<String, String>,
}

/// Marqueur visuel d'un personnage (D66) : `tint` = couleur RGB (0 à 1) multipliée aux calques,
/// `scale` = facteur appliqué à l'échelle des calques.
#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
pub struct CharacterMarker {
    #[serde(default)]
    pub tint: Option<(f32, f32, f32)>,
    #[serde(default)]
    pub scale: Option<f32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CharacterConfig {
    pub movement: MovementConfig,

    /// Dash nommé (M2-T2, §43) : distance, durée, i-frames, effet de variante, charges. Absent :
    /// le réglage v1 de `movement`.
    #[serde(default)]
    pub dash: Option<crate::character::movement::DashConfig>,

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

    /// Effets v1 du personnage (T1.10, `docs/conventions.md` §27) : `Effects`/`EffectState`
    /// posés au spawn seulement si la liste n'est pas vide (aucun contenu existant n'en a).
    #[serde(default)]
    pub effects: Vec<effects::Effect>,

    /// Boss à phases (M2-T0c, `docs/conventions.md` §37) : `BossPlan`/`BossState` posés au spawn
    /// seulement si présent (aucun contenu existant n'en a).
    #[serde(default)]
    pub boss: Option<behaviors::BossDef>,

    /// Bombe du personnage (M3-T0c, `docs/conventions.md` §62) : sans elle, le bouton `Bomb` est
    /// sans effet (aucun contenu existant n'en a). La pose vient de M3-T4.
    #[serde(default)]
    pub bomb: Option<world::BombDef>,

    /// Visée du personnage (suivi Q17, `docs/conventions.md` §66) : `Free` (défaut, 360°) ou
    /// `FourWay` (haut, bas, gauche, droite : `isaac`). Quantifiée dans `weapon_rollback_system`.
    /// Composant posé seulement pour `FourWay` : aucun contenu existant ne change.
    #[serde(default)]
    pub aim: sim_core::aim::AimMode,

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

    /// Variantes et élites (T1.5, `docs/conventions.md` §25) : table tirée à l'apparition
    /// (`character::variant`). `None` pour tout le contenu existant : aucun tirage.
    #[serde(default)]
    pub variants: Option<crate::character::variant::VariantsConfig>,

    /// Si vrai, ce personnage reçoit un composant `HitCount` à sa création.
    #[serde(default)]
    pub counts_hits: bool,

    /// I-frames après un coup subi (M3-T0b, §61). Absent : comportement historique ; le
    /// résolveur ne pose aucune invulnérabilité supplémentaire.
    #[serde(default)]
    pub hit_iframes: Option<u32>,

    /// Marqueur visuel (D66, présentation seule : aucune simulation ne le lit) : teinte et échelle
    /// des calques, pour distinguer un personnage dont le sprite est provisoire (tourelle à
    /// silhouette de zombie). `None` : sprites inchangés.
    #[serde(default)]
    pub marker: Option<CharacterMarker>,

    /// Scénarios générés de ce personnage (T1.13, `docs/conventions.md` §28, gabarits
    /// `EnemyVsStillPlayer`/`EnemyVsMovingPlayer` de `scenario::generate`). Aucun effet en jeu ;
    /// `None` : pas de scénario généré.
    #[serde(default)]
    pub test: Option<CharacterTest>,

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

    /// Monnaie de départ (T2.3, chantier C5 v1) : valeur initiale de `run::currency::Currency`,
    /// posée à la création (`character::player::create::create_player`). Défaut 500 ; le
    /// contenu `zombies` le déclare explicitement (voir `games/zombies/assets/ZombieShooter/
    /// Sprites/Character/player_config.ron`). Sans effet sur un ennemi (`enemy::create::spawn_enemy`
    /// ne pose jamais `Currency`, voir sa doc).
    #[serde(default = "default_starting_currency")]
    pub starting_currency: u32,

    /// Consommables de départ (M2-T1) : `{"blank": 2}` donne un `items::Inventory` à la création
    /// du joueur. Vide par défaut : le joueur n'a pas d'inventaire avant son premier ramassage
    /// (traces des autres jeux inchangées). Lint : ids d'objets consommables connus.
    #[serde(default)]
    pub starting_consumables: std::collections::BTreeMap<String, u32>,
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

fn default_starting_currency() -> u32 {
    500
}

/// Clé du `CharacterConfig` d'un personnage dans [`CharacterConfigs`] : l'`asset_name_ref`
/// (le `CharacterId` du registre de contenu). Composant non rollback, posé à la création
/// (`create_character`) ; remplace l'ancien `CharacterConfigHandles` (`Handle<CharacterConfig>`).
#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub struct CharacterConfigKey(pub String);

/// Configurations des personnages, **donnée de simulation** (D69 étape 2, E3a) : clés = celles
/// de `GlobalAsset::character_configs` (les `CharacterId` du registre). Remplie hors simulation
/// par `config_asset::sync_character_configs` (`PreUpdate`, avant `RunGgrsSystems`) à partir des
/// assets chargés, et seulement par elle (les surcharges de scénario se posent ensuite, sur cette
/// ressource). Jamais écrite par `GgrsSchedule`, donc ni rollback ni checksum : c'est une donnée
/// statique d'une partie, comme l'était l'`Assets<CharacterConfig>` qu'elle remplace.
#[derive(Resource, Clone, Debug, Default)]
pub struct CharacterConfigs(pub BTreeMap<String, CharacterConfig>);

impl CharacterConfigs {
    /// Configuration d'un personnage, `None` si la clé est inconnue (registre pas encore rempli).
    pub fn get(&self, key: &str) -> Option<&CharacterConfig> {
        self.0.get(key)
    }
}
