pub mod melee;

pub mod expectations;
use self::melee::MeleeAttackState;
use crate::{
    actors::{
        CursorPosition, DashState, Health, PeerConfig, Player, SprintState, INPUT_DASH,
        INPUT_DROP_WEAPON, INPUT_RELOAD, INPUT_SPRINT, INPUT_SWITCH_WEAPON_MODE,
    },
    collider::{is_colliding, Collider, ColliderShape, CollisionLayer, CollisionSettings, Wall},
    downed::Downed,
    inventory::AmmoReserves,
    projectile::{Projectile, ProjectileSpec, ProjectileTable},
    team::team_allows_hit,
};
use animation::{AnimationStateBundle, FacingDirection};
use bevy::{
    log::{tracing::span, Level},
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_fixed::{fixed_math, rng::RngStreams};
use bevy_ggrs::{GgrsSchedule, PlayerInputs, Rollback};
use ggrs::PlayerHandle;
use serde::{Deserialize, Serialize};
use sim_core::{
    ammo::AmmoType,
    damage::{DamageEvent, DamageKind, FriendlyFire},
    frame_events::{FrameEvents, FrameEventsAppExt},
    interaction::{Interactable, InteractionType},
    kinds::{KindDecl, KindRegistry},
    stats::StatId,
    system_set::RollbackSystemSet,
    tag::{Tag, Tags},
    team::Team,
};
use stats::StatReader;
use std::{collections::BTreeMap, fmt};
use utils::{
    frame::FrameCount,
    net_id::{GgrsNetId, GgrsNetIdFactory},
    order_iter, order_mut_iter,
    rollback::RollbackTraceApp,
};

/// Direction d'une balle de tir simple (toutes les armes sauf `FiringMode::Shotgun`) : `aim_dir`
/// tourné de `(random − ½) × spread`, donc dans `[−spread/2, spread/2]` ; `spread` = 0 → tir
/// exactement dans la direction visée. Un seul tirage RNG par balle (`random`, flux `weapons`).
/// D51 : avant, l'angle valait `(random − ½) × 1` quel que soit `spread` (±0,5 rad pour toutes les
/// armes simples).
pub fn single_shot_direction(
    aim_dir: fixed_math::FixedVec2,
    random: fixed_math::Fixed,
    spread: fixed_math::Fixed,
) -> fixed_math::FixedVec2 {
    // `from_angle(0)` n'est pas l'identité exacte en `Fixed` (cos ≈ 1,00002) : spread 0 = visée
    // exacte.
    if spread == fixed_math::Fixed::ZERO {
        return aim_dir;
    }
    let offset_from_center = random.saturating_sub(fixed_math::FIXED_HALF);
    let angle = offset_from_center.saturating_mul(spread);
    fixed_math::FixedMat2::from_angle(angle).mul_vec2(aim_dir)
}

// COMPONENTS
#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize, PartialEq)]
pub enum FiringMode {
    Automatic {}, // Hold trigger to continuously fire
    Manual {},    // One shot per trigger pull
    Burst {
        pellets_per_shot: u32,
        cooldown_frames: u32,
    }, // Fire a fixed number of shots per trigger pull
    Shotgun {
        pellet_count: u32,
        spread_angle: fixed_math::Fixed,
    },
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize, PartialEq)]
pub enum MagBulletConfig {
    Mag { mag_size: u32, mag_limit: u32 },
    Magless { bullet_limit: u32 },
}

#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize, PartialEq)]
pub enum BulletType {
    Standard {
        damage: fixed_math::Fixed,
        speed: fixed_math::Fixed,
    },
    Explosive {
        damage: fixed_math::Fixed,
        speed: fixed_math::Fixed,
        blast_radius: fixed_math::Fixed,
        explosive_damage_multiplier: fixed_math::Fixed,
    },
    Piercing {
        damage: fixed_math::Fixed,
        speed: fixed_math::Fixed,
        penetration: u8,
    },
}

impl fmt::Display for BulletType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BulletType::Standard { .. } => write!(f, "Standard"),
            BulletType::Explosive { .. } => write!(f, "Explosive"),
            BulletType::Piercing { .. } => write!(f, "Piercing"),
        }
    }
}

#[derive(Component)]
pub struct ExplosiveTag;

#[derive(Component)]
pub struct PiercingTag;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct FiringModeConfig {
    pub firing_rate: fixed_math::Fixed,
    pub firing_mode: FiringMode,
    pub spread: fixed_math::Fixed,
    pub recoil: fixed_math::Fixed,
    pub bullet_type: BulletType,
    pub range: fixed_math::Fixed,

    pub reload_time_seconds: fixed_math::Fixed,
    pub mag: MagBulletConfig,
    /// Projectile composable (T1.1, chantier B5 v1, `crate::projectile`) : modificateurs,
    /// `on_hit`, `on_expire`. Absent (vide) : balle ordinaire, comme avant ce champ.
    #[serde(default, skip_serializing_if = "ProjectileSpec::is_empty")]
    pub projectile: ProjectileSpec,
}

/// Hash et Debug manuels (T1.1) : identiques à ceux que dérivait `FiringModeConfig` avant
/// le champ `projectile` tant qu'il est vide — le contenu existant garde le même checksum
/// (`Weapon` est rollback) et la même ligne dans la trace détaillée (preuve `trace-diff`).
impl std::hash::Hash for FiringModeConfig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.firing_rate.hash(state);
        self.firing_mode.hash(state);
        self.spread.hash(state);
        self.recoil.hash(state);
        self.bullet_type.hash(state);
        self.range.hash(state);
        self.reload_time_seconds.hash(state);
        self.mag.hash(state);
        if !self.projectile.is_empty() {
            self.projectile.hash(state);
        }
    }
}

impl fmt::Debug for FiringModeConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("FiringModeConfig");
        d.field("firing_rate", &self.firing_rate)
            .field("firing_mode", &self.firing_mode)
            .field("spread", &self.spread)
            .field("recoil", &self.recoil)
            .field("bullet_type", &self.bullet_type)
            .field("range", &self.range)
            .field("reload_time_seconds", &self.reload_time_seconds)
            .field("mag", &self.mag);
        if !self.projectile.is_empty() {
            d.field("projectile", &self.projectile);
        }
        d.finish()
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct WeaponConfig {
    pub name: String,
    pub default_firing_mode: String,
    // BTreeMap (pas HashMap) : ce config est intégré au composant `Weapon`, rollback.
    pub firing_modes: BTreeMap<String, FiringModeConfig>,
    /// Politique de tir ami (T1.1, chantier B1). `#[serde(default)]` = `Never` : les armes
    /// existantes ne touchent jamais un allié, comme avant (où la matrice de collision ne
    /// laissait de toute façon jamais une balle atteindre un joueur).
    #[serde(default)]
    pub friendly_fire: FriendlyFire,
    /// Type de munition (T2.2, chantier B7 « Munitions typées et inventaire d'armes »).
    /// Obligatoire (pas de `#[serde(default)]`) : une arme à distance sans `ammo_type` dans
    /// son RON échoue au chargement (`content::lint` rapporte l'erreur RON, fichier +
    /// message). Clé partagée de `combat::inventory::AmmoReserves` : deux armes qui déclarent
    /// le même type puisent dans la même réserve (voir le scénario `ammo_shared_reserve`).
    /// **Exclu du hash manuel ci-dessous** (comme `test`, mais pour une raison différente :
    /// `ammo_type` est une vraie valeur de gameplay, pas une métadonnée d'outillage — il est
    /// exclu uniquement pour que l'ajout de ce champ ne fasse pas dériver le checksum GGRS
    /// des scénarios existants, qui ne changent pas de comportement observable ; la valeur
    /// qui compte pour la simulation vit dans `combat::inventory::AmmoReserves`, déjà
    /// rollback). Voir le rapport de la tâche T2.2, décision « ammo_type hors hash ».
    pub ammo_type: AmmoType,
    /// Gabarit de scénario généré (T2.10, `crates/scenario/src/generate.rs`) : nombre de
    /// coups attendus sur `target` après `frames` images de tir continu. `None` (défaut) :
    /// l'arme obtient quand même un scénario généré, mais avec les invariants seulement (pas
    /// d'attente de coups). Pure métadonnée d'outillage : ne doit **jamais** entrer dans le
    /// hash de `WeaponConfig` (voir l'impl manuelle de `Hash` ci-dessous), qui contribue au
    /// checksum GGRS via `Weapon`/`WeaponInventory` — un `derive(Hash)` ordinaire changerait
    /// le checksum de *toutes* les armes dès que ce champ existe, même à `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test: Option<WeaponTest>,
    /// Projectiles nommés que les patterns de `on_expire` font naître (T1.1, chantier B5
    /// v1, `crate::projectile::ProjectileDef`) : `{ "eclat": (damage: "5", speed: "300",
    /// range: "120") }`. Vide par défaut ; hors hash et hors Debug tant qu'il est vide (voir
    /// `FiringModeConfig`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub projectiles: ProjectileTable,
}

/// Hash manuel : reprend exactement les champs (et l'ordre) que dérivait `WeaponConfig`
/// avant l'ajout de `test`, en excluant `test` — une arme sans `test:` (`None`, tout le
/// contenu existant) produit donc le même hash qu'avant ce champ, et une arme avec `test:`
/// n'en produit pas un différent selon la valeur de `test` (pure métadonnée, voir sa doc).
/// `ammo_type` (T2.2) est exclu pour la même raison pratique (voir sa doc) : les cinq armes
/// du contenu `zombies` avaient déjà des comportements de munition distincts avant ce champ
/// (mag/reload par arme) ; le rendre visible au hash ferait dériver le checksum de tous les
/// scénarios existants dès l'ajout du champ, sans qu'aucune valeur de jeu ne bouge.
impl std::hash::Hash for WeaponConfig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.default_firing_mode.hash(state);
        self.firing_modes.hash(state);
        self.friendly_fire.hash(state);
        if !self.projectiles.is_empty() {
            self.projectiles.hash(state);
        }
    }
}

/// Voir le Hash ci-dessus : même sortie que l'ancien `derive(Debug)` tant que `projectiles`
/// est vide.
impl fmt::Debug for WeaponConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("WeaponConfig");
        d.field("name", &self.name)
            .field("default_firing_mode", &self.default_firing_mode)
            .field("firing_modes", &self.firing_modes)
            .field("friendly_fire", &self.friendly_fire)
            .field("ammo_type", &self.ammo_type)
            .field("test", &self.test);
        if !self.projectiles.is_empty() {
            d.field("projectiles", &self.projectiles);
        }
        d.finish()
    }
}

/// Attentes d'un scénario généré pour cette arme (T2.10). RON : `test: (frames: 240,
/// min_hits: 5, max_hits: 200)` (`max_hits`/`expect` optionnels). Validé par
/// `content::lint` (`frames > 0`, `min_hits <= max_hits`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct WeaponTest {
    /// Nombre de frames du scénario généré, et frame de vérification de `EntityHits`.
    pub frames: u32,
    /// `EntityHits::min` : coups minimum sur `target` à la frame `frames`.
    pub min_hits: u32,
    /// `EntityHits::max`, si présent (sinon aucune borne haute).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_hits: Option<u32>,
    /// Attentes supplémentaires ajoutées telles quelles au scénario généré, en plus de
    /// `EntityHits` (ex. vérifier autre chose que `target`). Vide par défaut.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expect: Vec<expectations::Expectation>,
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize, PartialEq)]
pub struct WeaponSpriteConfig {
    pub name: String,
    pub index: usize,

    pub weapon_offset: fixed_math::FixedVec2,

    pub bullet_offset_left: fixed_math::FixedVec2,
    pub bullet_offset_right: fixed_math::FixedVec2,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WeaponAsset {
    pub config: WeaponConfig,
    pub sprite_config: WeaponSpriteConfig,
}

// Component for a weapon
#[derive(Component, Debug, Clone, Hash)]
pub struct Weapon {
    pub config: WeaponConfig,
    pub sprite_config: WeaponSpriteConfig,
}

impl From<WeaponAsset> for Weapon {
    fn from(value: WeaponAsset) -> Self {
        Self {
            config: value.config,
            sprite_config: value.sprite_config,
        }
    }
}

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct HitMarker {
    pub target: Entity,
    pub damage: fixed_math::Fixed,
}

#[derive(Component)]
pub struct VisualEffectRequest {
    pub effect_type: EffectType,
    pub position: fixed_math::FixedVec2,
    pub scale: fixed_math::Fixed,
}

#[derive(Clone)]
pub enum EffectType {
    BulletHit,
    Explosion,
    Piercing,
}

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct ExplosionMarker {
    pub radius: fixed_math::Fixed,
    pub damage: fixed_math::Fixed,
    pub player_handle: PlayerHandle,
    pub processed: bool, // Flag to ensure one-time processing
}

/// Component to mark an entity as the active weapon
#[derive(Component)]
pub struct ActiveWeapon;

/// Component for bullets
#[derive(Component, Clone, Debug, Hash)]
pub struct Bullet {
    pub velocity: fixed_math::FixedVec2,
    pub bullet_type: BulletType,
    pub damage: fixed_math::Fixed,
    pub range: fixed_math::Fixed,
    pub distance_traveled: fixed_math::Fixed,
    pub player_handle: PlayerHandle,
    pub created_at: u32,
    /// Identité du tireur (T1.1) : `DamageEvent::source` à la collision. Capturée au tir
    /// (pas de requête sur l'entité tireuse au moment de la collision, qui peut arriver
    /// plusieurs frames plus tard).
    pub source: GgrsNetId,
    /// Équipe du tireur au moment du tir (T1.1, `combat::team::team_allows_hit`).
    pub source_team: Team,
    /// Tags du tireur au moment du tir, union `"bullet"` (T1.1, voir
    /// `sim_core::damage::DamageEvent::tags`).
    pub tags: Tags,
    /// Politique de tir ami de l'arme au moment du tir (`WeaponConfig::friendly_fire`).
    pub friendly_fire: FriendlyFire,
}

/// Component to track the player's weapon inventory
#[derive(Component, Debug, Clone, Default)]
pub struct WeaponInventory {
    pub active_weapon_index: usize,
    pub frame_switched: u32,
    pub frame_switched_mode: u32,
    pub weapons: Vec<(Entity, Weapon)>, // Store entity handles and weapon data

    pub reloading_ending_frame: Option<u32>,
}

/// Hash manuel : `weapons` porte des `Entity` (différents d'un client à l'autre) à côté
/// de chaque `Weapon` ; seuls l'index actif, les frames et les `Weapon` (dans l'ordre du
/// `Vec`, déterministe) contribuent au checksum.
impl std::hash::Hash for WeaponInventory {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.active_weapon_index.hash(state);
        self.frame_switched.hash(state);
        self.frame_switched_mode.hash(state);
        self.weapons.len().hash(state);
        for (_entity, weapon) in &self.weapons {
            weapon.hash(state);
        }
        self.reloading_ending_frame.hash(state);
    }
}

impl WeaponInventory {
    pub fn active_weapon(&self) -> &(Entity, Weapon) {
        self.weapons.get(self.active_weapon_index).unwrap()
    }
}

/// État d'un mode de tir d'une arme équipée. **Migration T2.2** : portait jusque-là
/// `mag_quantity: u32` (chargeurs de réserve, décrémenté à chaque rechargement, propre à
/// cette arme). Retiré : la réserve est désormais partagée par type de munition entre toutes
/// les armes d'un joueur (`combat::inventory::AmmoReserves`, clé `WeaponConfig::ammo_type`)
/// plutôt que comptée par arme — voir [`WeaponModeState::can_reload`]/[`WeaponModeState::reload`]
/// et le rapport de la tâche T2.2 pour l'équivalence (même nombre de balles tirables au total
/// qu'avant, avec les types distincts du contenu `zombies` d'aujourd'hui).
#[derive(Reflect, Default, Clone, Debug, Hash)]
pub struct WeaponModeState {
    pub mag_ammo: u32,

    pub burst_shots_left: u32,

    pub mag_size: u32,
    pub burst_cooldown: bool,
}

#[derive(Component, Reflect, Default, Clone, Debug, Hash)]
pub struct WeaponModesState {
    // BTreeMap (pas HashMap) : ce composant est rollback, la clé (nom du mode) doit
    // s'itérer dans un ordre stable entre clients (voir le changement de mode par nom).
    pub modes: BTreeMap<String, WeaponModeState>,
}

// Component to track rollbackable state for weapons
#[derive(Component, Reflect, Default, Clone, Debug, Hash)]
pub struct WeaponState {
    pub last_fire_frame: u32,
    pub is_firing: bool,
    pub active_mode: String,
}

#[derive(Event)]
pub struct FireWeaponEvent {
    pub player_entity: Entity,
}

// ASSETS

#[derive(Asset, TypePath, Serialize, Deserialize)]
pub struct WeaponsConfig(pub HashMap<String, WeaponAsset>);

// UTILITY FUNCTION

impl WeaponModeState {
    pub fn is_mag_full(&self) -> bool {
        self.mag_ammo == self.mag_size
    }

    /// Un rechargement n'est possible que pour une arme à chargeur séparé
    /// (`MagBulletConfig::Mag` ; jamais `Magless` — un fusil à pompe n'a pas de réserve à
    /// puiser, son « rechargement » entre deux tirs n'est qu'un délai de pompe, voir le
    /// commentaire de l'appel dans `weapon_rollback_system`), avec au moins un chargeur
    /// plein (`mag_size` unités) disponible dans `reserve`
    /// (T2.2, `combat::inventory::AmmoReserves::get`), et un chargeur pas déjà plein.
    pub fn can_reload(&self, mag: &MagBulletConfig, reserve: u32) -> bool {
        matches!(mag, MagBulletConfig::Mag { .. })
            && reserve >= self.mag_size
            && !self.is_mag_full()
    }

    /// Termine un rechargement commencé avec [`Self::can_reload`] vrai : remplit le
    /// chargeur. L'appelant (`weapon_rollback_system`) a déjà retiré `mag_size` de la
    /// réserve avant d'appeler cette méthode (T2.2) — reprend le rythme de l'ancien
    /// `mag_quantity` (un chargeur entier par rechargement, jamais un appoint partiel qui
    /// gaspillerait moins de munitions qu'avant), mesuré en munitions plutôt qu'en
    /// chargeurs de réserve.
    pub fn reload(&mut self) {
        self.mag_ammo = self.mag_size;
    }
}

impl WeaponInventory {
    pub fn is_reloading(&self) -> bool {
        self.reloading_ending_frame.is_some()
    }

    pub fn is_reloading_over(&self, current_frame: u32) -> bool {
        self.reloading_ending_frame
            .map_or_else(|| true, |f| current_frame >= f)
    }

    pub fn clear_reloading(&mut self) {
        self.reloading_ending_frame = None;
    }

    pub fn start_reload(
        &mut self,
        current_game_frame: u32,
        reload_time_seconds: fixed_math::Fixed,
    ) {
        self.reloading_ending_frame = {
            if reload_time_seconds <= fixed_math::new(0.0) {
                None
            } else {
                let frames_to_reload =
                    (reload_time_seconds * bevy_fixed::fixed_math::new(60.)).ceil();
                if frames_to_reload == 0 {
                    // Ensure at least one frame for very short reload times
                    Some(current_game_frame + 1)
                } else {
                    Some(current_game_frame + frames_to_reload.to_num::<u32>())
                }
            }
        };
    }
}

// start the reload process

/// Contribution de cette arme à la réserve initiale de son type de munition (T2.2, chantier
/// B7) : `mag_limit × mag_size` de son mode par défaut ; `0` pour une arme magless (pas de
/// chargeur séparé à réapprovisionner, voir la doc de `WeaponModeState::can_reload`).
/// Partagée par `character::player::create::create_player` (somme des armes de départ) et
/// `scenario::runner::apply_player_overrides` (`PlayerScript::weapon`, une seule arme).
/// Capacité (chargeur plein) du mode par défaut de cette arme : `mag_size` (chargeur séparé)
/// ou `bullet_limit` (magless). T2.3 (chantier C5 v1) : arme murale fraîchement achetée,
/// toujours livrée chargeur plein (voir `map_ldtk::game::local::spawn_weapon_locations_when_map_loaded`).
pub fn default_mode_capacity(weapon: &WeaponAsset) -> u32 {
    weapon
        .config
        .firing_modes
        .get(&weapon.config.default_firing_mode)
        .map_or(0, |mode| match mode.mag {
            MagBulletConfig::Mag { mag_size, .. } => mag_size,
            MagBulletConfig::Magless { bullet_limit } => bullet_limit,
        })
}

pub fn default_mode_ammo_contribution(weapon: &WeaponAsset) -> (AmmoType, u32) {
    let amount = weapon
        .config
        .firing_modes
        .get(&weapon.config.default_firing_mode)
        .map_or(0, |mode| match mode.mag {
            MagBulletConfig::Mag {
                mag_size,
                mag_limit,
            } => mag_size * mag_limit,
            MagBulletConfig::Magless { .. } => 0,
        });
    (weapon.config.ammo_type.clone(), amount)
}

// Function to spawn weapon , all weapon should be spawn on the user when they got them
#[allow(clippy::too_many_arguments)]
pub fn spawn_weapon_for_player(
    commands: &mut Commands,

    active: bool,

    player_entity: Entity,
    weapon: WeaponAsset,
    inventory: &mut WeaponInventory,

    id_factory: &mut ResMut<GgrsNetIdFactory>,
    // Munitions à poser dans le chargeur du mode par défaut, au lieu de le remplir à plein
    // (T2.2, chantier B7) : ramasser une arme au sol restaure le `mag_ammo` capturé au
    // moment du dépôt (`WeaponPickup::mag_ammo`), borné à la capacité du mode par défaut.
    // `None` (tous les appels existants avant T2.2) : chargeur plein, comportement inchangé.
    initial_mag_ammo: Option<u32>,
) -> Entity {
    // Entité logique uniquement : le sprite est ajouté par attach_weapon_visuals
    let animation_bundle =
        AnimationStateBundle::new(BTreeMap::from([("body".to_string(), String::new())]));

    let mut weapon_state = WeaponState::default();
    let mut weapon_modes_state = WeaponModesState::default();
    weapon_state.active_mode = weapon.config.default_firing_mode.clone();
    for (k, v) in weapon.config.firing_modes.iter() {
        let mut weapon_mode_state = WeaponModeState::default();
        // Capacité de ce mode : `mag_size` (chargeur séparé) ou `bullet_limit` (magless, pas
        // de chargeur séparé, voir la doc de `WeaponModeState::can_reload`).
        let capacity = match v.mag {
            MagBulletConfig::Mag { mag_size, .. } => {
                weapon_mode_state.mag_size = mag_size;
                mag_size
            }
            MagBulletConfig::Magless { bullet_limit } => bullet_limit,
        };
        // `initial_mag_ammo` (T2.2) ne s'applique qu'au mode par défaut : c'est le seul dont
        // `WeaponPickup` a capturé le `mag_ammo` au moment du dépôt (voir sa doc) ; les
        // autres modes démarrent pleins, comme avant ce champ.
        weapon_mode_state.mag_ammo = if k == &weapon.config.default_firing_mode {
            initial_mag_ammo.map_or(capacity, |ammo| ammo.min(capacity))
        } else {
            capacity
        };

        weapon_modes_state
            .modes
            .insert(k.clone(), weapon_mode_state);
    }

    let weapon: Weapon = weapon.into();

    let transform = Transform::from_translation(
        fixed_math::fixed_to_vec2(weapon.sprite_config.weapon_offset).extend(0.),
    )
    .with_rotation(Quat::IDENTITY);
    let ggrs_transform = fixed_math::FixedTransform3D::from_bevy_transform(&transform);

    let entity = commands
        .spawn((
            transform,
            ggrs_transform,
            weapon_state,
            weapon_modes_state,
            weapon.clone(),
            animation_bundle,
            id_factory.next(weapon.config.name.clone()),
        ))
        .insert(Rollback)
        .id();

    inventory.weapons.push((entity, weapon));

    if active {
        commands
            .entity(entity)
            .insert((ActiveWeapon {}, Visibility::Inherited));
        inventory.active_weapon_index = inventory.weapons.len() - 1;
    } else {
        commands.entity(entity).insert(Visibility::Hidden);
    }

    commands.entity(player_entity).add_child(entity);

    entity
}

/// Arme à distance tombée au sol (T2.2, chantier B7 : lâcher/ramasser). Posé avec
/// `Interactable { interaction_type: InteractionType::Weapon }` et un `Weapon` (réutilisé
/// tel quel pour le sprite en présentation : `attach_weapon_visuals` itère tout `Weapon` sans
/// visuel, qu'il soit équipé ou au sol — pas de système de rendu dédié) sur la même entité
/// rollback. Consommé par `interaction::handle_weapon_pickup_interaction`.
#[derive(Component, Debug, Clone, Hash, Serialize, Deserialize)]
pub struct WeaponPickup {
    /// Id de l'arme dans `weapons.ron` (`WeaponConfig::name`, clé du registre) : relu au
    /// ramassage pour retrouver le `WeaponAsset` complet dans `Assets<WeaponsConfig>`.
    pub weapon_id: String,
    /// Munitions du chargeur du mode par défaut au moment du dépôt (voir
    /// `spawn_weapon_for_player::initial_mag_ammo`) : un seul nombre, pas un par mode — le
    /// mode par défaut est le seul restauré (comportement documenté, voir le rapport de la
    /// tâche T2.2, décision « forme de WeaponPickup »).
    pub mag_ammo: u32,
    /// Prix (T2.3, chantier C5 v1) : `Some(prix)` pour une arme murale (`WeaponLocation`,
    /// `map_ldtk::game::local::spawn_weapon_locations_when_map_loaded`), `None` pour une
    /// arme lâchée par un joueur (`weapons::weapon_drop_system`) ou ramassée sans économie
    /// (T2.2). Lu par `interaction::handle_weapon_pickup_interaction` : une arme murale
    /// n'est jamais consommée au ramassage (elle reste achetable) et exige `Currency >=
    /// price` (ou `refill_price` si le joueur possède déjà cette arme).
    #[serde(default)]
    pub price: Option<u32>,
    /// Anti-rebond (T2.3, chantier C5 v1) : une arme murale n'a pas de cooldown naturel
    /// contrairement à une porte (`Interactable` retiré après ouverture) — sans ceci,
    /// maintenir Interaction facturerait `refill_price` à **chaque frame** après le premier
    /// achat (l'arme devient « déjà possédée » dès la frame suivante). Même mécanisme que
    /// `map::game::entity::map::window::WindowHealth::can_repair_after_frame` : posé après
    /// chaque achat réussi, ignoré (`None`) pour une arme non murale.
    #[serde(default)]
    pub can_buy_after_frame: Option<u32>,
}

/// Délai (frames) avant qu'une arme murale (T2.3) puisse à nouveau être achetée/rechargée
/// après un achat réussi — même ordre de grandeur que
/// `interaction::WindowRepairConfig::repair_cooldown_frames` (défaut 60, 1 s à 60 FPS).
pub const WALL_WEAPON_PURCHASE_COOLDOWN_FRAMES: u32 = 60;

/// Fait tomber une arme au sol (`WeaponPickup`), interactable au ramassage. Portée
/// commune à `weapons::weapon_drop_system` (T2.2, `price: None`) et à T2.3 (armes murales,
/// `map_ldtk::game::local::spawn_weapon_locations_when_map_loaded`, `price: Some(prix)`) :
/// même mécanisme de ramassage (`interaction::handle_weapon_pickup_interaction`) pour les
/// deux, qui lit `WeaponPickup::price` pour distinguer une arme murale (jamais consommée,
/// débite `Currency`) d'une arme au sol ordinaire (consommée, gratuite).
#[allow(clippy::too_many_arguments)]
pub fn spawn_weapon_pickup(
    commands: &mut Commands,
    weapon: Weapon,
    mag_ammo: u32,
    position: fixed_math::FixedVec3,
    price: Option<u32>,
    id_factory: &mut ResMut<GgrsNetIdFactory>,
) -> Entity {
    let weapon_id = weapon.config.name.clone();

    let transform = fixed_math::FixedTransform3D::new(
        position,
        fixed_math::FixedMat3::IDENTITY,
        fixed_math::FixedVec3::ONE,
    );

    let g_id = id_factory.next(format!("weapon_pickup_{weapon_id}"));

    commands
        .spawn((
            transform.to_bevy_transform(),
            transform,
            weapon,
            WeaponPickup {
                weapon_id,
                mag_ammo,
                price,
                can_buy_after_frame: None,
            },
            Interactable {
                interaction_range: fixed_math::new(30.0),
                interaction_type: InteractionType::Weapon,
            },
            g_id,
        ))
        .insert(Rollback)
        .id()
}

/// Une balle à faire apparaître : tout est déjà résolu par l'appelant (position et rotation
/// du point de tir, vitesse, dégât et portée finaux). Voir [`spawn_bullet`].
pub struct BulletSpawn<'a> {
    pub position: fixed_math::FixedVec3,
    pub rotation: fixed_math::FixedMat3,
    pub velocity: fixed_math::FixedVec2,
    pub bullet_type: BulletType,
    /// Dégât final (multiplicateur du tireur déjà appliqué).
    pub damage: fixed_math::Fixed,
    /// Portée finale (multiplicateur du tireur déjà appliqué).
    pub range: fixed_math::Fixed,
    /// Handle GGRS du joueur tireur ; [`NO_PLAYER_HANDLE`] pour un tir d'émetteur ennemi
    /// (jamais écrit dans un log : les logs nomment le tireur par son `GgrsNetId`).
    pub player_handle: PlayerHandle,
    pub created_at: u32,
    pub source: &'a GgrsNetId,
    pub source_team: Team,
    /// Tags du tireur : la balle porte leur union avec `bullet` (voir la doc de
    /// `sim_core::damage::DamageEvent::tags`).
    pub source_tags: &'a Tags,
    pub friendly_fire: FriendlyFire,
    /// Projectile composable (T1.1) ; `None` : balle ordinaire.
    pub projectile: Option<Projectile>,
    /// Préfixe du `GgrsNetId` de la balle (type de balle pour un tir de joueur, id de
    /// projectile pour un projectile né d'un pattern).
    pub id_label: String,
}

/// `player_handle` d'une balle tirée par un émetteur (T1.2, ennemi) : aucun joueur. Seule la
/// copie vers les projectiles nés de cette balle le lit ; jamais écrit dans un log.
pub const NO_PLAYER_HANDLE: PlayerHandle = PlayerHandle::MAX;

/// **Seule** fonction d'apparition d'une balle (T1.2) : tir d'un joueur
/// (`spawn_bullet_rollback`), projectile né d'un pattern
/// (`projectile::spawn_child_projectile`) et tir d'émetteur (`emitter::emitter_system`).
/// Rayon (base 5, 8 pour `Explosive`, × `Size`), sprite (3.5, × `Size` si composable),
/// couleur et marqueurs (`ExplosiveTag`, `PiercingTag`) dérivent du type de balle et du
/// projectile ; le `GgrsNetId` est alloué ici, une fois par balle.
pub fn spawn_bullet(
    commands: &mut Commands,
    collision_settings: &CollisionSettings,
    id_factory: &mut GgrsNetIdFactory,
    spawn: BulletSpawn,
) -> Entity {
    let base_radius = match &spawn.bullet_type {
        BulletType::Explosive { .. } => fixed_math::new(8.0),
        BulletType::Standard { .. } | BulletType::Piercing { .. } => fixed_math::new(5.0),
    };
    let (radius, sprite_size) = match &spawn.projectile {
        Some(projectile) => (
            projectile.radius(base_radius),
            fixed_math::to_f32(projectile.size) * crate::projectile::BASE_SPRITE,
        ),
        None => (base_radius, 3.5),
    };
    let color = match &spawn.bullet_type {
        BulletType::Standard { .. } => Color::BLACK,
        BulletType::Explosive { .. } => Color::WHITE,
        BulletType::Piercing { .. } => Color::BLACK,
    };

    let transform = fixed_math::FixedTransform3D::new(
        spawn.position,
        spawn.rotation,
        fixed_math::FixedVec3::ONE,
    );

    let g_id = id_factory.next(spawn.id_label);

    info!(
        "{} spawn at {} by {}",
        g_id, transform.translation, spawn.source
    );

    let mut tags = spawn.source_tags.clone();
    tags.insert(Tag::new("bullet"));

    let bullet_type = spawn.bullet_type;
    let mut entity_commands = commands.spawn((
        Sprite::from_color(color, Vec2::new(sprite_size, sprite_size)),
        Bullet {
            velocity: spawn.velocity,
            bullet_type,
            damage: spawn.damage,
            range: spawn.range,
            distance_traveled: fixed_math::Fixed::ZERO,
            player_handle: spawn.player_handle,
            created_at: spawn.created_at,
            source: spawn.source.clone(),
            source_team: spawn.source_team,
            tags,
            friendly_fire: spawn.friendly_fire,
        },
        Collider {
            offset: fixed_math::FixedVec3::ZERO,
            shape: ColliderShape::Circle { radius },
        },
        CollisionLayer(collision_settings.bullet_layer),
        transform.to_bevy_transform(),
        transform,
        g_id,
    ));

    match bullet_type {
        BulletType::Explosive { .. } => {
            entity_commands.insert(ExplosiveTag);
        }
        BulletType::Piercing { .. } => {
            entity_commands.insert(PiercingTag);
        }
        _ => {}
    };
    if let Some(projectile) = spawn.projectile {
        entity_commands.insert(projectile);
    }

    entity_commands.insert(Rollback).id()
}

/// Tir d'un joueur : point de tir à la bouche de l'arme (décalage du sprite, rotation de
/// l'arme puis du joueur), puis [`spawn_bullet`].
#[allow(clippy::too_many_arguments)]
fn spawn_bullet_rollback(
    commands: &mut Commands,
    weapon: &Weapon,
    player_transform: &fixed_math::FixedTransform3D,
    weapon_transform: &fixed_math::FixedTransform3D,
    facing_direction: &FacingDirection,
    direction: fixed_math::FixedVec2,
    bullet_type: BulletType,
    range: fixed_math::Fixed,
    player_handle: PlayerHandle,
    current_frame: u32,
    collision_settings: &Res<CollisionSettings>,
    id_factory: &mut ResMut<GgrsNetIdFactory>,
    source: &GgrsNetId,
    source_team: Team,
    source_tags: &Tags,
    friendly_fire: FriendlyFire,
    // Multiplicateur de dégât du porteur (T1.2, stat `Damage`, 1 par défaut).
    damage_mult: fixed_math::Fixed,
    // Multiplicateur de portée du porteur (T1.2, stat `Range`, 1 par défaut).
    range_mult: fixed_math::Fixed,
    // Projectile composable (T1.1) : `Size` agrandit collider et sprite ; `None` (mode de
    // tir sans `projectile:`) : balle ordinaire, inchangée.
    projectile: Option<Projectile>,
) -> Entity {
    let (velocity, damage) = match &bullet_type {
        BulletType::Standard {
            speed,
            damage: damage_bullet,
        }
        | BulletType::Explosive {
            speed,
            damage: damage_bullet,
            ..
        }
        | BulletType::Piercing {
            speed,
            damage: damage_bullet,
            ..
        } => (
            direction * (*speed / fixed_math::Fixed::from_num(60)),
            *damage_bullet,
        ),
    };

    // Stats branchées (T1.2, chantier B2) : sans modificateur actif, `damage_mult`/
    // `range_mult` valent exactement 1 (voir `character::create::create_character`), donc
    // ce produit ne change aucune valeur par rapport à avant ce chantier.
    let damage = damage.saturating_mul(damage_mult);
    let range = range.saturating_mul(range_mult);

    let local_muzzle_offset_v2 = if !facing_direction.should_flip_x() {
        weapon.sprite_config.bullet_offset_right
    } else {
        weapon.sprite_config.bullet_offset_left
    };

    // 1. Muzzle offset in weapon's local 3D space
    let local_muzzle_offset_v3 = fixed_math::FixedVec3 {
        x: local_muzzle_offset_v2.x,
        y: local_muzzle_offset_v2.y,
        z: fixed_math::Fixed::ZERO,
    };

    // 2. Transform muzzle offset by weapon's local rotation (relative to player)
    //    and add weapon's local translation (relative to player)
    //    to get muzzle position in player's local coordinate system.
    let weapon_local_rotation_mat3: fixed_math::FixedMat3 = weapon_transform.rotation.clone();
    let weapon_local_translation_v3: fixed_math::FixedVec3 = weapon_transform.translation;

    let muzzle_pos_in_player_space =
        weapon_local_rotation_mat3.mul_vec3(local_muzzle_offset_v3) + weapon_local_translation_v3;

    // 3. Transform muzzle position from player's local space to world space.
    let player_world_rotation_mat3: fixed_math::FixedMat3 = player_transform.rotation.clone();
    let player_world_translation_v3: fixed_math::FixedVec3 = player_transform.translation;

    let world_firing_position = player_world_rotation_mat3.mul_vec3(muzzle_pos_in_player_space)
        + player_world_translation_v3;

    // 4. Calculate projectile's world rotation.
    // This is player's world rotation combined with weapon's local rotation.
    let projectile_world_rotation =
        player_world_rotation_mat3.mul_mat3(&weapon_local_rotation_mat3); // Ensure mul_mat3 is the correct operation

    let id_label = format!("{}", bullet_type);
    spawn_bullet(
        commands,
        collision_settings,
        id_factory,
        BulletSpawn {
            position: world_firing_position,
            rotation: projectile_world_rotation,
            velocity,
            bullet_type,
            damage,
            range,
            player_handle,
            created_at: current_frame,
            source,
            source_team,
            source_tags,
            friendly_fire,
            projectile,
            id_label,
        },
    )
}

// SYSTEMS

// Rollback system to correctly transform the weapon based on the position
// L'arme active vient de WeaponInventory (rollback), pas du marqueur ActiveWeapon
// qui n'est mis à jour que dans Update pour l'affichage.
pub fn system_weapon_position(
    query: Query<(&WeaponInventory, &CursorPosition), With<Rollback>>,
    mut query_weapon: Query<&mut fixed_math::FixedTransform3D, With<Weapon>>,
) {
    for (inventory, cursor_position) in query.iter() {
        if let Some((active_weapon, _)) = inventory.weapons.get(inventory.active_weapon_index) {
            if let Ok(mut transform) = query_weapon.get_mut(*active_weapon) {
                let cursor_game_world_pos = fixed_math::FixedVec3::new(
                    fixed_math::new(cursor_position.x as f32),
                    fixed_math::new(cursor_position.y as f32),
                    fixed_math::new(0.0),
                );
                let direction_to_target_fixed =
                    (cursor_game_world_pos - transform.translation).normalize_or_zero();
                let angle_radians_fixed = fixed_math::atan2_fixed(
                    direction_to_target_fixed.y,
                    direction_to_target_fixed.x,
                );

                transform.rotation = fixed_math::FixedMat3::from_rotation_z(angle_radians_fixed);
            }
        }
    }
}

/// Lâche l'arme active au sol (T2.2, chantier B7) : `INPUT_DROP_WEAPON`, touche `G`
/// (`Devices`), bouton `DropWeapon` des scénarios. Sans effet si le joueur n'a pas d'arme à
/// distance ou est à terre (T1.3, comme les autres actions volontaires — dash, sprint,
/// interaction). Tourne avant `weapon_rollback_system` (voir `BaseWeaponGamePlugin`) : un
/// lâcher et un tir/rechargement ne se rencontrent jamais à la même frame pour la même arme,
/// la suite de la frame relit un `WeaponInventory` déjà à jour.
pub fn weapon_drop_system(
    mut commands: Commands,
    inputs: Res<PlayerInputs<PeerConfig>>,
    frame: Res<FrameCount>,
    mut inventory_query: Query<
        (
            &mut WeaponInventory,
            &fixed_math::FixedTransform3D,
            &Player,
            Has<Downed>,
        ),
        With<Rollback>,
    >,
    weapon_state_query: Query<(&WeaponState, &WeaponModesState)>,
    mut id_factory: ResMut<GgrsNetIdFactory>,
) {
    let system_span = span!(Level::INFO, "ggrs", f = frame.frame, s = "weapon_drop");
    let _enter = system_span.enter();

    // Ordre par handle (déterministe), comme `weapon_rollback_system` : ce système alloue
    // aussi des `GgrsNetId` (l'entité `WeaponPickup`).
    let mut players: Vec<_> = inventory_query.iter_mut().collect();
    players.sort_by_key(|(_, _, player, _is_downed)| player.handle);

    for (mut inventory, transform, player, is_downed) in players {
        let (input, _status) = inputs[player.handle];
        if input.buttons & INPUT_DROP_WEAPON == 0 {
            continue;
        }
        // À terre (T1.3) : pas d'action volontaire.
        if is_downed {
            continue;
        }
        if inventory.weapons.is_empty() {
            continue;
        }
        // Anti-rebond (même délai et même champ que le changement d'arme,
        // `weapon_rollback_system` : un lâcher change aussi l'arme active) : sans lui, tenir
        // le bouton plusieurs frames (un scénario tient toujours au moins 2-3 frames, voir
        // `Segment`) ferait tomber une arme différente à chaque frame jusqu'à vider
        // l'inventaire entier.
        if inventory.frame_switched + 20 >= frame.frame {
            continue;
        }

        let idx = inventory.active_weapon_index;
        let (weapon_entity, weapon) = inventory.weapons[idx].clone();
        // Munitions du mode actif au moment du dépôt (voir la doc de `WeaponPickup`) :
        // absente (entité déjà despawn) seulement dans un cas qui ne devrait pas arriver
        // (incohérence `WeaponInventory`/entité enfant), 0 par défaut plutôt que paniquer.
        let mag_ammo = weapon_state_query
            .get(weapon_entity)
            .ok()
            .and_then(|(state, modes)| modes.modes.get(&state.active_mode))
            .map_or(0, |mode| mode.mag_ammo);

        info!(
            "player {} drops {} ({} balles)",
            player.handle, weapon.config.name, mag_ammo
        );

        use bevy_ggrs::RollbackDespawnCommandExtension;
        commands.entity(weapon_entity).despawn_rollback();
        inventory.weapons.remove(idx);
        inventory.active_weapon_index = if inventory.weapons.is_empty() {
            0
        } else {
            idx.min(inventory.weapons.len() - 1)
        };
        inventory.frame_switched = frame.frame;
        // Une arme qui tombe emporte son rechargement en cours (l'entité qui rechargeait
        // n'existe plus) ; une seule arme peut être en cours de rechargement à la fois
        // (`WeaponInventory::is_reloading`), donc ceci ne touche jamais une autre arme.
        inventory.clear_reloading();

        spawn_weapon_pickup(
            &mut commands,
            weapon,
            mag_ammo,
            transform.translation,
            None,
            &mut id_factory,
        );
    }
}

// rollback system for weapon action , firing and all
pub fn weapon_rollback_system(
    mut commands: Commands,
    mut rng_streams: ResMut<RngStreams>,
    inputs: Res<PlayerInputs<PeerConfig>>,
    frame: Res<FrameCount>,

    mut inventory_query: Query<(
        Entity,
        &mut WeaponInventory,
        &SprintState,
        &DashState,
        &MeleeAttackState,
        &fixed_math::FixedTransform3D,
        &Player,
        Has<Downed>,
        // T1.3 : `Stun`/`Freeze` (§19) : pas de tir.
        Option<&crate::status::Statuses>,
        // Réserve de munitions partagée par type (T2.2, chantier B7) : consultée/consommée
        // au rechargement, à la place de l'ancien `WeaponModeState::mag_quantity`.
        &mut AmmoReserves,
    )>,
    mut weapon_query: Query<(
        &mut Weapon,
        &mut WeaponState,
        &mut WeaponModesState,
        &fixed_math::FixedTransform3D,
        &ChildOf,
    )>,

    player_query: Query<(
        &fixed_math::FixedTransform3D,
        &FacingDirection,
        &Player,
        &GgrsNetId,
        &Team,
        Option<&Tags>,
    )>,

    collision_settings: Res<CollisionSettings>,

    stats: StatReader,

    mut id_factory: ResMut<GgrsNetIdFactory>,
) {
    let system_span = span!(Level::INFO, "ggrs", f = frame.frame, s = "weapon");
    let _enter = system_span.enter(); // Enter the span

    // Process weapon firing for all players, in handle order: firing consumes RollbackRng
    // (spread) and GgrsNetIds (bullets), so the order must be the same on every client
    let mut players: Vec<_> = inventory_query.iter_mut().collect();
    players.sort_by_key(|(.., player, _is_downed, _statuses, _ammo_reserves)| player.handle);

    for (
        _entity,
        mut inventory,
        sprint_state,
        dash_state,
        melee_attack_state,
        transform,
        player,
        is_downed,
        statuses,
        mut ammo_reserves,
    ) in players
    {
        let (input, _input_status) = inputs[player.handle];
        // T1.3 : étourdi ou gelé : comme à terre, pas de tir
        if crate::status::incapacitated(statuses) {
            continue;
        }

        // Do nothing if no weapons
        if inventory.weapons.is_empty() {
            continue;
        }

        // À terre (T1.3) : pas de tir.
        if is_downed {
            continue;
        }

        // A released trigger is always registered, even while reloading, switching, sprinting
        // or in melee: semi-automatic weapons need a new press after it
        if !input.fire {
            let (active_weapon, _) = inventory.weapons[inventory.active_weapon_index];
            if let Ok((_, mut weapon_state, ..)) = weapon_query.get_mut(active_weapon) {
                weapon_state.is_firing = false;
            }
        }

        // Don't allow weapon firing during melee attacks
        if melee_attack_state.is_attacking {
            continue;
        }

        if sprint_state.is_sprinting
            || dash_state.is_dashing
            || input.buttons & INPUT_SPRINT != 0
            || input.buttons & INPUT_DASH != 0
        {
            continue;
        }

        // Nothing to do for weapon if we are sprinting

        // Get active weapon
        let (weapon_entity, _) = inventory.weapons[inventory.active_weapon_index];

        // Get the entity for the active weapon
        if let Ok((weapon, mut weapon_state, mut weapon_modes_state, weapon_transform, child_of)) =
            weapon_query.get_mut(weapon_entity)
        {
            let active_mode = weapon_state.active_mode.clone();
            let weapon_config = weapon.config.firing_modes.get(&active_mode).unwrap();

            // Stats branchées (T1.2, chantier B2) : la cadence de tir et le temps de
            // rechargement de cette arme sont multipliés par les stats du porteur
            // (`FireRate`/`ReloadSpeed`, 1 par défaut, voir
            // `character::create::create_character`) — sans modificateur actif, produit
            // exact par 1, aucune valeur ne change.
            let shooter = child_of.parent();
            let fire_rate_mult = stats.get(shooter, &StatId::FireRate, fixed_math::FIXED_ONE);
            let reload_speed_mult = stats.get(shooter, &StatId::ReloadSpeed, fixed_math::FIXED_ONE);
            let reload_time_seconds = weapon_config
                .reload_time_seconds
                .saturating_mul(reload_speed_mult);

            // A reload in progress completes on the mode it was started for: switching mode
            // (like switching weapon) is not possible until it is over
            if inventory.is_reloading() {
                if inventory.is_reloading_over(frame.frame) {
                    // Magless (fusil à pompe) : le "rechargement" n'est qu'un délai de pompe
                    // entre deux tirs, jamais un réapprovisionnement — aucune réserve à
                    // puiser (voir la doc de `WeaponModeState::can_reload`), `mag_ammo`
                    // inchangé, comme avant T2.2 (`mag_quantity` valait toujours 0 pour ces
                    // armes, le corps du `if` d'origine ne s'exécutait jamais).
                    if let MagBulletConfig::Mag { .. } = weapon_config.mag {
                        let mode_state = weapon_modes_state.modes.get_mut(&active_mode).unwrap();
                        let needed = mode_state.mag_size;
                        // Puise dans la réserve du type de cette arme (T2.2,
                        // `combat::inventory::AmmoReserves`) : un chargeur entier à la fois,
                        // jamais un appoint partiel (voir la doc de `reload`). Lu puis retiré
                        // séparément (pas juste le résultat de `take`) pour ne jamais
                        // consommer une réserve insuffisante sans remplir le chargeur.
                        if ammo_reserves.get(&weapon.config.ammo_type) >= needed {
                            ammo_reserves.take(&weapon.config.ammo_type, needed);
                            mode_state.reload();
                        }
                    }
                    inventory.clear_reloading();
                } else {
                    continue;
                }
            }

            if input.buttons & INPUT_SWITCH_WEAPON_MODE != 0 {
                // Next mode in name order (any number of modes, same order on every client)
                let mut mode_names: Vec<&String> = weapon_modes_state.modes.keys().collect();
                mode_names.sort();
                let next_mode = mode_names
                    .iter()
                    .position(|name| **name == weapon_state.active_mode)
                    .map(|i| mode_names[(i + 1) % mode_names.len()])
                    .filter(|name| **name != weapon_state.active_mode)
                    .cloned();
                if let Some(new_mode) = next_mode {
                    if inventory.frame_switched_mode + 20 < frame.frame
                        && inventory.frame_switched + 20 < frame.frame
                    {
                        inventory.frame_switched_mode = frame.frame;
                        weapon_state.active_mode = new_mode.clone();

                        continue;
                    }
                }
            }

            let weapon_mode_state = weapon_modes_state.modes.get_mut(&active_mode).unwrap();
            let reserve = ammo_reserves.get(&weapon.config.ammo_type);

            if input.buttons & INPUT_RELOAD != 0
                && weapon_mode_state.can_reload(&weapon_config.mag, reserve)
            {
                inventory.start_reload(frame.frame, reload_time_seconds);
                continue;
            }

            // Handle switching of weapons, will start firing on the next frame
            if input.switch_weapon && !inventory.weapons.is_empty() {
                let new_index = (inventory.active_weapon_index + 1) % inventory.weapons.len();

                if new_index != inventory.active_weapon_index
                    && inventory.frame_switched + 20 < frame.frame
                    && inventory.frame_switched_mode + 20 < frame.frame
                {
                    inventory.active_weapon_index = new_index;
                    inventory.frame_switched = frame.frame;

                    continue;
                }
            }

            // A burst, once started, is fired to the end even if the trigger is released
            let burst_in_progress = matches!(weapon_config.firing_mode, FiringMode::Burst { .. })
                && weapon_mode_state.burst_shots_left > 0;

            if input.fire || burst_in_progress {
                // Calculate fire rate in frames (60 FPS assumed) , need to be configure via ressource instead
                let firing_rate = weapon_config.firing_rate.saturating_mul(fire_rate_mult);
                let frame_per_shot =
                    (bevy_fixed::fixed_math::new(60.) / firing_rate).to_num::<u32>();
                let current_frame = frame.frame;
                let frames_since_last_shot = current_frame - weapon_state.last_fire_frame;

                let (can_fire, empty) = match weapon_config.firing_mode {
                    FiringMode::Automatic { .. } => (
                        frames_since_last_shot >= frame_per_shot,
                        weapon_mode_state.mag_ammo == 0,
                    ),

                    FiringMode::Manual { .. } => (
                        !weapon_state.is_firing && frames_since_last_shot >= frame_per_shot,
                        weapon_mode_state.mag_ammo == 0,
                    ),

                    FiringMode::Burst {
                        pellets_per_shot,
                        cooldown_frames,
                    } => {
                        // The cooldown between bursts ends by itself: a single press then
                        // starts the next burst (it used to be spent lifting the cooldown)
                        if weapon_mode_state.burst_cooldown
                            && frames_since_last_shot >= cooldown_frames
                        {
                            weapon_mode_state.burst_cooldown = false;
                        }
                        if weapon_mode_state.burst_shots_left > 0
                            && frames_since_last_shot >= frame_per_shot
                        {
                            // Continue ongoing burst
                            (true, weapon_mode_state.mag_ammo == 0)
                        } else if weapon_mode_state.burst_shots_left == 0 {
                            if !weapon_state.is_firing
                                && !weapon_mode_state.burst_cooldown
                                && frames_since_last_shot >= cooldown_frames
                            {
                                // Start new burst when trigger is pulled
                                weapon_mode_state.burst_shots_left = pellets_per_shot;
                                (true, weapon_mode_state.mag_ammo == 0)
                            } else if weapon_mode_state.burst_cooldown
                                && frames_since_last_shot >= cooldown_frames
                            {
                                // Reset cooldown
                                weapon_mode_state.burst_cooldown = false;
                                (false, false)
                            } else {
                                (false, false)
                            }
                        } else {
                            (false, false)
                        }
                    }

                    FiringMode::Shotgun { .. } => {
                        if !weapon_state.is_firing && frames_since_last_shot >= frame_per_shot {
                            // Shotgun fires all pellets at once, so we don't need burst_shots_left
                            (true, weapon_mode_state.mag_ammo == 0)
                        } else {
                            (false, false)
                        }
                    }
                };

                if empty {
                    // Empty mag: reload if the reserve has enough for a full mag, otherwise
                    // just a dry click (`reserve` computed above, unchanged since : neither
                    // branch reached this point after consuming it this frame).
                    if weapon_mode_state.can_reload(&weapon_config.mag, reserve) {
                        inventory.start_reload(frame.frame, reload_time_seconds);
                    }
                    continue;
                }

                weapon_state.is_firing = input.fire;

                if can_fire {
                    if let Ok((_, facing_direction, _, shooter_net_id, shooter_team, opt_tags)) =
                        player_query.get(child_of.parent())
                    {
                        let shooter_tags = opt_tags.cloned().unwrap_or_default();
                        // Stats branchées (T1.2) : dégâts et portée de la balle multipliés
                        // par les stats du porteur (`Damage`/`Range`, 1 par défaut).
                        let damage_mult =
                            stats.get(shooter, &StatId::Damage, fixed_math::FIXED_ONE);
                        let range_mult = stats.get(shooter, &StatId::Range, fixed_math::FIXED_ONE);
                        let mut aim_dir = fixed_math::FixedVec2::new(
                            fixed_math::Fixed::from_num(input.pan_x),
                            fixed_math::Fixed::from_num(input.pan_y),
                        );
                        aim_dir.x /= fixed_math::new(127.0);
                        aim_dir.y /= fixed_math::new(127.0);
                        aim_dir = aim_dir.normalize_or_zero();
                        // Projectile composable du mode de tir (T1.1), cloné par balle.
                        let projectile = (!weapon_config.projectile.is_empty()).then(|| {
                            Projectile::new(
                                weapon.config.name.clone(),
                                &weapon_config.projectile,
                                std::sync::Arc::new(weapon.config.projectiles.clone()),
                                damage_mult,
                                0,
                            )
                        });

                        match weapon_config.firing_mode {
                            FiringMode::Shotgun {
                                pellet_count,
                                spread_angle,
                            } => {
                                // Fire multiple pellets in a spread pattern
                                for _ in 0..pellet_count {
                                    // Calculate a random angle within the spread range
                                    let random_fixed_val =
                                        rng_streams.get_mut("weapons").next_fixed();
                                    let offset_from_center =
                                        random_fixed_val.saturating_sub(fixed_math::FIXED_HALF);
                                    let pellet_angle_fixed =
                                        offset_from_center.saturating_mul(spread_angle);

                                    // Create the fixed-point 2D rotation matrix
                                    let fixed_spread_rotation =
                                        fixed_math::FixedMat2::from_angle(pellet_angle_fixed);

                                    // Apply the rotation to the fixed-point aim direction
                                    let direction = fixed_spread_rotation.mul_vec2(aim_dir);

                                    spawn_bullet_rollback(
                                        &mut commands,
                                        &weapon,
                                        transform,
                                        weapon_transform,
                                        facing_direction,
                                        direction,
                                        weapon_config.bullet_type,
                                        weapon_config.range,
                                        player.handle,
                                        frame.frame,
                                        &collision_settings,
                                        &mut id_factory,
                                        shooter_net_id,
                                        *shooter_team,
                                        &shooter_tags,
                                        weapon.config.friendly_fire,
                                        damage_mult,
                                        range_mult,
                                        projectile.clone(),
                                    );
                                }
                                weapon_mode_state.mag_ammo -= 1; // Shotgun uses one ammo for all pellets
                                inventory.start_reload(frame.frame, reload_time_seconds);
                            }
                            _ => {
                                let random_fixed_val = rng_streams.get_mut("weapons").next_fixed();
                                let direction = single_shot_direction(
                                    aim_dir,
                                    random_fixed_val,
                                    weapon_config.spread,
                                );

                                spawn_bullet_rollback(
                                    &mut commands,
                                    &weapon,
                                    transform,
                                    weapon_transform,
                                    facing_direction,
                                    direction,
                                    weapon_config.bullet_type,
                                    weapon_config.range,
                                    player.handle,
                                    frame.frame,
                                    &collision_settings,
                                    &mut id_factory,
                                    shooter_net_id,
                                    *shooter_team,
                                    &shooter_tags,
                                    weapon.config.friendly_fire,
                                    damage_mult,
                                    range_mult,
                                    projectile,
                                );
                                weapon_mode_state.mag_ammo -= 1;

                                if matches!(weapon_config.firing_mode, FiringMode::Burst { .. })
                                    && weapon_mode_state.burst_shots_left > 0
                                {
                                    weapon_mode_state.burst_shots_left -= 1;

                                    // Set cooldown when burst finishes
                                    if weapon_mode_state.burst_shots_left == 0 {
                                        weapon_mode_state.burst_cooldown = true;
                                    }
                                }
                            }
                        }
                        weapon_state.last_fire_frame = frame.frame;
                    }
                }
            } else {
                weapon_state.is_firing = false;
            }
        }
    }
}

pub fn bullet_rollback_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    mut bullet_query: Query<(
        &GgrsNetId,
        Entity,
        &mut fixed_math::FixedTransform3D,
        &mut Bullet,
        Option<&mut Projectile>,
    )>,
) {
    let system_span = span!(Level::INFO, "ggrs", f = frame.frame, s = "bullet_movement");
    let _enter = system_span.enter();

    for (g_id, entity, mut transform, mut bullet, projectile) in order_mut_iter!(bullet_query) {
        // Move bullet based on velocity (fixed timestep)
        let delta = bullet.velocity;

        // Apply movement
        transform.translation.x += delta.x;
        transform.translation.y += delta.y;

        bullet.distance_traveled += delta.length();

        if bullet.distance_traveled >= bullet.range {
            info!(
                "{} despawn after travelleing {}",
                g_id, bullet.distance_traveled
            );
            if let Some(mut projectile) = projectile {
                // Projectile composable (T1.1) : terminé ici, détruit après `on_expire`
                // par `projectile::projectile_expire_system`.
                projectile.ended = true;
            } else {
                use bevy_ggrs::RollbackDespawnCommandExtension;
                commands.entity(entity).despawn_rollback();
            }
        }
    }
}
/// Cible potentielle d'une balle, choisie parmi tous les candidats en collision (triés par
/// `GgrsNetId`, voir [`bullet_rollback_collision_system`]).
enum BulletTarget {
    /// Mur : décidé par `CollisionLayer`/`layer_matrix` (physique, inchangé par T1.1).
    Wall,
    /// Personnage (`Team`) : décidé par `combat::team::team_allows_hit`, pas par
    /// `layer_matrix` (T1.1). `Team::Neutral` arrive ici aussi (elle bloque le tir sans
    /// jamais être blessée, voir `combat::damage::resolve_damage`).
    Character,
}

/// Collision des balles : qui une balle touche et ce que ça déclenche (T1.1, chantier B1).
///
/// La cible géométrique la plus proche (par `GgrsNetId`, déterministe) parmi :
/// - les murs, toujours filtrés par `CollisionLayer`/`layer_matrix` (reste la seule
///   utilité de la matrice de collision pour les balles : les arrêter physiquement) ;
/// - les personnages (`Team`), filtrés par `combat::team::team_allows_hit` (équipe +
///   politique de tir ami de l'arme) — **pas** par `layer_matrix`. Un personnage que la
///   politique bloque (allié, tir ami `Never`) n'est pas un candidat du tout : la balle le
///   traverse comme s'il n'était pas là. `Team::Neutral` est toujours un candidat valide
///   (elle bloque le tir) ; `combat::damage::resolve_damage` décidera ensuite qu'elle ne
///   subit aucun dégât.
///
/// Émet un `DamageEvent` (résolu par `character::health::rollback_resolve_damage_events`,
/// `RollbackSystemSet::CollisionDamage`) au lieu d'écrire `DamageAccumulator` directement.
pub fn bullet_rollback_collision_system(
    frame: Res<FrameCount>,
    mut commands: Commands,
    settings: Res<CollisionSettings>,
    mut damage_events: ResMut<FrameEvents<DamageEvent>>,
    grids: Res<crate::collision_grid::CollisionGrids>,
    bullet_query: Query<
        (
            &GgrsNetId,
            Entity,
            &fixed_math::FixedTransform3D,
            &Bullet,
            &Collider,
            &CollisionLayer,
        ),
        // Les projectiles composables ont leurs propres collisions (T1.1,
        // `projectile::projectile_collision_system`).
        (With<Rollback>, Without<Projectile>),
    >,
    wall_query: Query<
        (
            &fixed_math::FixedTransform3D,
            &Collider,
            &CollisionLayer,
            &GgrsNetId,
        ),
        (With<Wall>, With<Rollback>),
    >,
    target_query: Query<
        (&fixed_math::FixedTransform3D, &Collider, &GgrsNetId, &Team),
        (With<Health>, Without<Bullet>, With<Rollback>),
    >,
) {
    let system_span = span!(
        Level::INFO,
        "ggrs",
        f = frame.frame,
        s = "bullet_collissions"
    );
    let _enter = system_span.enter();

    let mut bullets_to_despawn_set = HashSet::new();

    for (ggrs_net_id, bullet_entity, bullet_transform, bullet, bullet_collider, bullet_layer) in
        order_iter!(bullet_query)
    {
        if bullets_to_despawn_set.contains(&bullet_entity) {
            continue;
        }

        // Phase 1 : tous les candidats en collision géométrique (murs par layer_matrix,
        // personnages par équipe/politique de tir ami). Grille spatiale (T2.1, chantier B4b) :
        // requête sur le segment de déplacement de la frame (AABB englobant l'ancienne et la
        // nouvelle position, `bullet_rollback_system` a déjà appliqué `bullet.velocity` cette
        // frame) — un sur-ensemble sûr pour le broad-phase, jamais plus étroit que la boucle
        // d'avant ; le test précis `is_colliding` qui suit reste sur la position actuelle
        // seule, exactement comme avant (résultat inchangé, seul le nombre de paires testées
        // change).
        let mut candidates: Vec<(GgrsNetId, BulletTarget)> = Vec::new();

        let old_pos = fixed_math::FixedVec3::new(
            bullet_transform.translation.x - bullet.velocity.x,
            bullet_transform.translation.y - bullet.velocity.y,
            bullet_transform.translation.z,
        );
        let swept_aabb = crate::collision_grid::union_aabb(
            crate::collision_grid::collider_aabb(&old_pos, bullet_collider),
            crate::collision_grid::collider_aabb(&bullet_transform.translation, bullet_collider),
        );

        for wall_entry in grids.walls.query_aabb(&swept_aabb) {
            let Ok((target_transform, target_collider, target_layer, wall_net_id)) =
                wall_query.get(wall_entry.entity)
            else {
                continue;
            };
            if !settings.layer_matrix[bullet_layer.0][target_layer.0] {
                continue;
            }
            if is_colliding(
                &bullet_transform.translation,
                bullet_collider,
                &target_transform.translation,
                target_collider,
            ) {
                candidates.push((wall_net_id.clone(), BulletTarget::Wall));
            }
        }

        for char_entry in grids.characters.query_aabb(&swept_aabb) {
            let Ok((target_transform, target_collider, target_net_id, target_team)) =
                target_query.get(char_entry.entity)
            else {
                continue;
            };
            if !team_allows_hit(
                bullet.source_team,
                *target_team,
                bullet.friendly_fire,
                &bullet.tags,
            ) {
                continue;
            }
            if is_colliding(
                &bullet_transform.translation,
                bullet_collider,
                &target_transform.translation,
                target_collider,
            ) {
                candidates.push((target_net_id.clone(), BulletTarget::Character));
            }
        }

        if candidates.is_empty() {
            continue; // No collision for this bullet
        }

        // Sort the collided entities to pick the "first" one deterministically
        candidates.sort_unstable_by_key(|(g_id, _)| g_id.0);
        let (deterministic_target_g_id, target_kind) = &candidates[0];

        info!(
            "bullet {} collissions with {:?}",
            ggrs_net_id, deterministic_target_g_id
        );

        let mut should_bullet_despawn_now = false;
        match target_kind {
            BulletTarget::Character => {
                damage_events.send(DamageEvent {
                    source: bullet.source.clone(),
                    target: deterministic_target_g_id.clone(),
                    kind: DamageKind::Physical,
                    amount: bullet.damage,
                    frame: frame.frame,
                    tags: bullet.tags.clone(),
                    source_team: bullet.source_team,
                    friendly_fire: bullet.friendly_fire,
                });

                match bullet.bullet_type {
                    BulletType::Standard { .. } | BulletType::Explosive { .. } => {
                        should_bullet_despawn_now = true;
                    }
                    BulletType::Piercing { .. } => {
                        // Les balles perforantes continuent à travers les personnages ;
                        // elles ne s'arrêtent que sur un mur (voir plus bas).
                    }
                }
            }
            BulletTarget::Wall => match bullet.bullet_type {
                BulletType::Standard { .. } | BulletType::Explosive { .. } => {
                    should_bullet_despawn_now = true;
                }
                BulletType::Piercing { .. } => {
                    // Piercing bullets despawn on walls
                    should_bullet_despawn_now = true;
                }
            },
        }

        if should_bullet_despawn_now {
            bullets_to_despawn_set.insert(bullet_entity);
            // Since the original code had a `break` here, we effectively stop processing
            // more targets for this bullet after this first deterministic interaction.
        }
    }

    // Deterministic despawning of bullets (already good)
    let mut bullets_to_despawn_vec: Vec<Entity> = bullets_to_despawn_set.into_iter().collect();
    bullets_to_despawn_vec.sort_by_key(|entity| entity.index()); // Or .to_bits()
    use bevy_ggrs::RollbackDespawnCommandExtension;
    for entity in bullets_to_despawn_vec {
        commands.entity(entity).despawn_rollback();
    }
}

// Non rollback system to display the weapon correct sprite
pub fn weapon_inventory_system(
    mut commands: Commands,
    query: Query<(Entity, &WeaponInventory, &MeleeAttackState)>,
    mut weapon_entities: Query<(Entity, &mut Visibility), With<Weapon>>,
) {
    for (_player_entity, inventory, melee_attack_state) in query.iter() {
        if inventory.weapons.is_empty() {
            continue;
        }

        // Update active/inactive weapon visibility
        for (i, (weapon_entity, _)) in inventory.weapons.iter().enumerate() {
            let is_active = i == inventory.active_weapon_index;

            // For simplicity, we're using commands to add/remove components
            // In a real implementation, you might want to use a Visibility component
            if let Ok((_, mut visibility)) = weapon_entities.get_mut(*weapon_entity) {
                if is_active {
                    commands.entity(*weapon_entity).insert(ActiveWeapon);
                    // Hide weapon during melee attacks
                    if melee_attack_state.is_attacking {
                        *visibility = Visibility::Hidden;
                    } else {
                        *visibility = Visibility::Visible;
                    }
                } else {
                    commands.entity(*weapon_entity).remove::<ActiveWeapon>();
                    *visibility = Visibility::Hidden;
                }
            }
        }
    }
}

pub fn weapons_config_update_system(
    _asset_server: Res<AssetServer>,

    weapons_config: Res<Assets<WeaponsConfig>>,

    mut ev_asset: MessageReader<AssetEvent<WeaponsConfig>>,

    mut query_weapons: Query<(&Children, Entity, &mut Weapon)>,
) {
    for event in ev_asset.read() {
        if let AssetEvent::Modified { id } = event {
            if let Some(weapons_config) = weapons_config.get(*id) {
                for (_childs, _entity, mut weapon) in query_weapons.iter_mut() {
                    if let Some(config) = weapons_config.0.get(&weapon.config.name) {
                        weapon.config = config.config.clone();
                        weapon.sprite_config = config.sprite_config.clone();
                    }
                }
            }
        }
    }
}

pub struct BaseWeaponGamePlugin {}

impl Plugin for BaseWeaponGamePlugin {
    fn build(&self, app: &mut App) {
        // Add RON asset plugins for weapons and melee weapons
        app.add_plugins(RonAssetPlugin::<WeaponsConfig>::new(&["ron"]));
        app.add_plugins(RonAssetPlugin::<melee::MeleeWeaponsConfig>::new(&["ron"]));

        // Kinds existants (lus par le lint de contenu, `crates/content`, T1.5). Les noms
        // sont ceux des tables `weapons.ron`/`melee_weapons.ron`, déjà en dur ailleurs
        // dans ce fichier et dans `character/enemy/create.rs` ; T1.5 les lira depuis le
        // registre construit au chargement des RON plutôt que d'une liste Rust.
        app.register_kinds([
            KindDecl::new("weapon", "pistol"),
            KindDecl::new("weapon", "machine_gun"),
            KindDecl::new("weapon", "shotgun"),
            KindDecl::new("weapon", "bare_hands"),
            KindDecl::new("weapon", "zombie_claws"),
            KindDecl::new("weapon", "club"),
            KindDecl::new("weapon", "knife"),
            KindDecl::new("weapon", "sword"),
            KindDecl::new("weapon", "axe"),
        ]);

        // Rollback components for ranged weapons
        app.rollback_and_trace::<WeaponInventory>()
            .rollback_and_trace::<WeaponModesState>()
            .rollback_and_trace::<WeaponState>()
            .rollback_and_trace::<Bullet>()
            .rollback_and_trace::<Weapon>()
            // T2.2, chantier B7 : réserve de munitions par joueur et arme tombée au sol.
            .rollback_and_trace::<AmmoReserves>()
            .rollback_and_trace::<WeaponPickup>()
            // T1.1, chantier B5 v1 : projectiles composables.
            .rollback_and_trace::<Projectile>()
            // T1.2 : émetteurs. Checksum **neutre** : aucune entité du contenu existant n'en
            // porte (voir la doc de `rollback_and_trace_neutral` : un type vide ordinaire
            // déplacerait toutes les traces).
            .rollback_and_trace_neutral::<crate::emitter::Emitter>();
        app.add_frame_events::<crate::projectile::ProjectileHit>();
        // T1.6 : mur touché (file neutre, vide sauf projectile à `on_hit`)
        app.add_frame_events_neutral::<crate::projectile::ProjectileWallHit>();

        // Rollback components for melee weapons
        app.rollback_and_trace::<melee::MeleeWeapon>()
            .rollback_and_trace::<melee::MeleeAttackState>()
            // D38 : type nouveau, sans porteur hors fuite : variante neutre (parité des types
            // vides, CLAUDE.md), sinon toutes les traces bougent dès la frame 0.
            .rollback_and_trace_neutral::<melee::MeleeHold>()
            .rollback_and_trace::<melee::MeleeHitbox>();

        app.add_systems(
            Update,
            (weapon_inventory_system, weapons_config_update_system),
        );

        app.add_systems(
            GgrsSchedule,
            (
                // Ranged weapon systems. `weapon_drop_system` avant tout le reste (T2.2) :
                // un lâcher cette frame doit être vu par `system_weapon_position`/
                // `weapon_rollback_system` (inventaire déjà à jour, arme déjà despawn).
                weapon_drop_system,
                system_weapon_position.after(weapon_drop_system),
                weapon_rollback_system.after(system_weapon_position),
                bullet_rollback_system.after(weapon_rollback_system),
                bullet_rollback_collision_system.after(bullet_rollback_system),
                // Melee weapon systems
                melee::player_melee_attack_system.after(bullet_rollback_collision_system),
                melee::enemy_melee_attack_system.after(melee::player_melee_attack_system),
                melee::update_melee_hitboxes.after(melee::enemy_melee_attack_system),
                melee::melee_hitbox_collision_system.after(melee::update_melee_hitboxes),
                // Émetteurs (T1.2) : après tout tir de joueur et de mêlée de la frame (flux
                // RNG et `GgrsNetId` alloués dans un ordre total).
                crate::emitter::emitter_system.after(melee::melee_hitbox_collision_system),
            )
                .in_set(RollbackSystemSet::Weapon),
        );

        // Projectiles composables (T1.1, voir la doc du module `crate::projectile`).
        app.add_systems(
            GgrsSchedule,
            (
                crate::projectile::projectile_collision_system,
                // Dans `Projectiles` plutôt qu'`Effects` : seul système de ce set à écrire
                // `Modifiers`, sans ambiguïté d'ordre avec les power-ups (`game`).
                crate::projectile::apply_projectile_on_hit_system,
                // T1.3 : ticks de `Burn` et expiration des statuts (§19), avant
                // `CollisionDamage` qui résout leurs dégâts dans la même frame.
                crate::projectile::status_tick_system,
                crate::projectile::projectile_wall_terrain_system,
                crate::projectile::projectile_expire_system,
                crate::projectile::projectile_steering_system,
            )
                .chain()
                .in_set(RollbackSystemSet::Projectiles),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fixed::fixed_math::{new as fx, Fixed, FixedVec2};

    /// D51 : `spread` 0 → la direction visée exactement, quel que soit le tirage.
    #[test]
    fn dispersion_nulle_tir_exact() {
        let aim = FixedVec2::new(fx(1.0), fx(0.0));
        for random in [fx(0.0), fx(0.25), fx(0.5), fx(0.999)] {
            assert_eq!(single_shot_direction(aim, random, Fixed::ZERO), aim);
        }
    }

    /// D51 : l'angle reste dans `[−spread/2, spread/2]` (bornes atteintes aux tirages extrêmes) ;
    /// avant le correctif, il couvrait ±0,5 rad pour toute arme.
    #[test]
    fn dispersion_dans_la_demi_largeur() {
        let aim = FixedVec2::new(fx(1.0), fx(0.0));
        let spread = fx(0.15);
        for random in [fx(0.0), fx(0.1), fx(0.5), fx(0.9), fx(0.999)] {
            let d = single_shot_direction(aim, random, spread);
            let angle = d.y.to_num::<f64>().atan2(d.x.to_num::<f64>());
            assert!(angle.abs() <= 0.075 + 1e-3, "tirage {random} : angle {angle}");
        }
        let d = single_shot_direction(aim, fx(0.0), spread);
        let angle = d.y.to_num::<f64>().atan2(d.x.to_num::<f64>());
        assert!((angle + 0.075).abs() < 2e-3, "tirage 0 : −spread/2, obtenu {angle}");
    }
}
