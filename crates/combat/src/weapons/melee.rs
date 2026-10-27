use crate::downed::Downed;
use crate::team::team_allows_hit;
use bevy::{
    log::{tracing::span, Level},
    prelude::*,
};
use bevy_fixed::fixed_math;
use bevy_ggrs::Rollback;
use ggrs::PlayerHandle;
use serde::{Deserialize, Serialize};
use sim_core::animation_state::FacingDirection;
use sim_core::damage::{DamageEvent, DamageKind, FriendlyFire};
use sim_core::tag::{Tag, Tags};
use sim_core::team::Team;
use utils::{
    net_id::{GgrsNetId, GgrsNetIdFactory},
    order_iter, order_mut_iter,
};

use crate::{
    actors::{Enemy, Health, PeerConfig, Player, Velocity, INPUT_MELEE_ATTACK},
    collider::{is_colliding, Collider, ColliderShape, CollisionLayer, CollisionSettings},
};
use sim_core::frame_events::FrameEvents;
use utils::frame::FrameCount;

// MELEE ATTACK PATTERN
#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize, PartialEq)]
pub enum MeleeAttackPattern {
    // Single quick strike
    SingleStrike,
    // Sweeping arc attack (hits multiple targets in front)
    Sweep { arc_angle: fixed_math::Fixed },
    // Thrust forward (longer range, narrower)
    Thrust,
    // Combo chain (multiple strikes in sequence)
    Combo { strikes_in_combo: u32 },
}

/// Parade et renvoi de projectiles (X-3, verbes D1) : paramètres du verbe, en frames
/// entières et `Fixed`, jamais de `f32`.
#[derive(Debug, Clone, Hash, Serialize, Deserialize, PartialEq)]
pub struct MeleeReflect {
    /// Fenêtre parfaite : un projectile touché au plus `perfect_frames` après le début
    /// du coup (création de la hitbox, `MeleeHitbox::created_frame`, celle-ci comprise) est
    /// `Perfect`, sinon `Good` (`sim_core::timing::grade_timing`, repère = début du
    /// coup, `good` = toute la durée de la hitbox). `0` : seul un contact à la frame
    /// du départ est parfait.
    pub perfect_frames: u32,
    /// Multiplicateur des dégâts du projectile renvoyé sur un `Perfect` (`1` : dégâts
    /// conservés comme sur un `Good`).
    pub perfect_damage_mult: fixed_math::Fixed,
}

/// Parade de mêlée contre mêlée (X-3, verbes D1) : la hitbox d'un coup adverse qui touche la
/// hitbox de cette arme est **annulée** (aucun dégât, aucun recul de sa part). Même fenêtre de
/// jugement que le renvoi : un contact au plus `perfect_frames` après le début du coup du pareur
/// est `Perfect`, sinon `Good` (`sim_core::timing`, repère = frame de création de la hitbox).
/// Frames entières et `Fixed`, jamais de `f32`.
#[derive(Debug, Clone, Hash, Serialize, Deserialize, PartialEq)]
pub struct MeleeParry {
    /// Fenêtre parfaite, en frames depuis le début du coup du pareur (`0` : seul un contact à la
    /// frame du départ est parfait).
    pub perfect_frames: u32,
    /// Recul infligé à l'attaquant sur un `Perfect`, le long de l'axe pareur → attaquant
    /// (`Velocity::knockback`, comme le recul d'un coup). `0` : aucun recul.
    pub perfect_knockback: fixed_math::Fixed,
}

// MELEE WEAPON CONFIG
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct MeleeWeaponConfig {
    pub name: String,
    pub damage: fixed_math::Fixed,
    pub range: fixed_math::Fixed,
    pub attack_pattern: MeleeAttackPattern,
    pub attack_duration_frames: u32,
    pub cooldown_frames: u32,
    pub knockback_force: fixed_math::Fixed,
    pub stamina_cost: fixed_math::Fixed,
    /// Politique de tir ami (T1.1, chantier B1). `#[serde(default)]` = `Never`.
    #[serde(default)]
    pub friendly_fire: FriendlyFire,
    /// Gabarit de scénario généré (T2.10). Même rôle que `WeaponConfig::test` (voir sa
    /// doc) : jamais dans le hash (impl manuelle ci-dessous), pure métadonnée d'outillage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test: Option<super::WeaponTest>,
    /// Parade et renvoi (X-3, verbes D1) : la hitbox de mêlée renvoie les projectiles
    /// hostiles qu'elle touche (changement d'équipe, direction du regard, dégâts
    /// conservés, ou multipliés par `perfect_damage_mult` dans la fenêtre parfaite).
    /// `None` (défaut) : pas de parade, comportement d'avant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reflect: Option<MeleeReflect>,
    /// Parade de mêlée (X-3) : annule les coups de mêlée adverses que la hitbox de l'arme
    /// touche (`melee_parry_system`). `None` (défaut) : pas de parade de mêlée.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parry: Option<MeleeParry>,
}

/// Hash manuel : voir `WeaponConfig::hash` (même raison, même méthode) — reprend l'ordre des
/// champs que dérivait `MeleeWeaponConfig` avant l'ajout de `test`, en l'excluant.
/// X-3 : `reflect` n'est haché que s'il est présent (`None`, tout le contenu d'avant :
/// hash inchangé, aucune trace ne bouge).
impl std::hash::Hash for MeleeWeaponConfig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.damage.hash(state);
        self.range.hash(state);
        self.attack_pattern.hash(state);
        self.attack_duration_frames.hash(state);
        self.cooldown_frames.hash(state);
        self.knockback_force.hash(state);
        self.stamina_cost.hash(state);
        self.friendly_fire.hash(state);
        if let Some(reflect) = &self.reflect {
            reflect.hash(state);
        }
        // `parry` suit la même règle (absent : rien d'écrit).
        if let Some(parry) = &self.parry {
            parry.hash(state);
        }
    }
}

/// Debug manuel : identique au `derive(Debug)` d'avant X-3 tant que `reflect` est absent
/// (même ligne dans la trace détaillée) ; `reflect` n'est écrit que s'il est présent.
impl std::fmt::Debug for MeleeWeaponConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("MeleeWeaponConfig");
        d.field("name", &self.name)
            .field("damage", &self.damage)
            .field("range", &self.range)
            .field("attack_pattern", &self.attack_pattern)
            .field("attack_duration_frames", &self.attack_duration_frames)
            .field("cooldown_frames", &self.cooldown_frames)
            .field("knockback_force", &self.knockback_force)
            .field("stamina_cost", &self.stamina_cost)
            .field("friendly_fire", &self.friendly_fire)
            .field("test", &self.test);
        if let Some(reflect) = &self.reflect {
            d.field("reflect", reflect);
        }
        if let Some(parry) = &self.parry {
            d.field("parry", parry);
        }
        d.finish()
    }
}

// MELEE WEAPON SPRITE CONFIG
#[derive(Debug, Clone, Hash, Serialize, Deserialize, PartialEq)]
pub struct MeleeWeaponSpriteConfig {
    pub name: String,
    pub index: usize,
    pub weapon_offset: fixed_math::FixedVec2,
}

// MELEE WEAPON ASSET
#[derive(Serialize, Deserialize, Clone)]
pub struct MeleeWeaponAsset {
    pub config: MeleeWeaponConfig,
    pub sprite_config: MeleeWeaponSpriteConfig,
}

// MELEE WEAPON COMPONENT
#[derive(Component, Debug, Clone, Hash)]
pub struct MeleeWeapon {
    pub config: MeleeWeaponConfig,
    pub sprite_config: MeleeWeaponSpriteConfig,
}

impl From<MeleeWeaponAsset> for MeleeWeapon {
    fn from(value: MeleeWeaponAsset) -> Self {
        Self {
            config: value.config,
            sprite_config: value.sprite_config,
        }
    }
}

/// D38 : l'ennemi ne lance pas de nouvelle attaque au corps à corps (il finit celle en
/// cours). Posé par `game` tant que la règle retenue de l'ennemi est `Flee` (un ennemi qui fuit
/// n'attaque plus dès qu'un joueur entre à portée), retiré ensuite. Rollback et tracé, en
/// variante **neutre** (`rollback_and_trace_neutral`) : type nouveau, absent de tout ennemi qui
/// ne fuit pas ; enregistré en variante normale, il déplacerait toutes les traces (parité des
/// types vides, CLAUDE.md).
#[derive(Component, Reflect, Default, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct MeleeHold;

/// D29 : multiplicateur de dégâts de la vague où l'ennemi est apparu
/// (`WaveState::current_damage_multiplier`, valeur de `wave_config.ron`). Posé au spawn par
/// `waves::wave_spawning_system` seulement quand il diffère de 1 (vague 1 : absent, donc rien ne
/// change), lu par la mêlée et l'attaque directe de l'ennemi. Rollback et tracé, variante
/// **neutre** (type nouveau sans porteur à la frame 0, parité des types vides, CLAUDE.md).
#[derive(Component, Default, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct WaveDamageScale(pub fixed_math::Fixed);

// MELEE ATTACK STATE
#[derive(Component, Reflect, Default, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct MeleeAttackState {
    pub is_attacking: bool,
    pub attack_started_frame: Option<u32>,
    pub attack_ended_frame: Option<u32>,
    pub last_attack_frame: u32,
    pub combo_count: u32,
    pub entities_hit_this_attack: Vec<GgrsNetId>,
}

impl MeleeAttackState {
    pub fn can_attack(&self, current_frame: u32, cooldown_frames: u32) -> bool {
        !self.is_attacking && (current_frame - self.last_attack_frame >= cooldown_frames)
    }

    pub fn start_attack(&mut self, current_frame: u32) {
        self.is_attacking = true;
        self.attack_started_frame = Some(current_frame);
        self.attack_ended_frame = None;
        self.entities_hit_this_attack.clear();
    }

    pub fn end_attack(&mut self, current_frame: u32) {
        self.is_attacking = false;
        self.attack_ended_frame = Some(current_frame);
        self.last_attack_frame = current_frame;
        self.combo_count = 0;
        self.entities_hit_this_attack.clear();
    }

    pub fn has_hit_entity(&self, net_id: &GgrsNetId) -> bool {
        self.entities_hit_this_attack.contains(net_id)
    }

    pub fn add_hit_entity(&mut self, net_id: GgrsNetId) {
        if !self.entities_hit_this_attack.contains(&net_id) {
            self.entities_hit_this_attack.push(net_id);
        }
    }
}

// MELEE ATTACK HITBOX
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct MeleeHitbox {
    pub damage: fixed_math::Fixed,
    pub knockback_force: fixed_math::Fixed,
    pub owner_entity: Entity,
    pub owner_net_id: GgrsNetId,
    pub owner_handle: Option<PlayerHandle>,
    pub created_frame: u32,
    pub duration_frames: u32,
    /// Direction de l'attaquant au moment du coup (orientation de l'effet visuel)
    pub facing: FacingDirection,
    /// Équipe de l'attaquant au moment du coup (T1.1, `combat::team::team_allows_hit`).
    pub owner_team: Team,
    /// Tags de l'attaquant au moment du coup, union `"melee"` (T1.1, voir
    /// `sim_core::damage::DamageEvent::tags`).
    pub tags: Tags,
    /// Politique de tir ami de l'arme au moment du coup (`MeleeWeaponConfig::friendly_fire`).
    pub friendly_fire: FriendlyFire,
}

/// Hash manuel : exclut `owner_entity` (`Entity`, différent d'un client à l'autre) au
/// profit de `owner_net_id`, déjà présent sur le composant.
impl std::hash::Hash for MeleeHitbox {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.damage.hash(state);
        self.knockback_force.hash(state);
        self.owner_net_id.hash(state);
        self.owner_handle.hash(state);
        self.created_frame.hash(state);
        self.duration_frames.hash(state);
        self.facing.hash(state);
        self.owner_team.hash(state);
        self.tags.hash(state);
        self.friendly_fire.hash(state);
    }
}

// SPAWN MELEE WEAPON
pub fn spawn_melee_weapon_for_character(
    commands: &mut Commands,
    character_entity: Entity,
    weapon: MeleeWeaponAsset,
    id_factory: &mut ResMut<GgrsNetIdFactory>,
) -> Entity {
    // For now, melee weapons might not have visible sprites or use simple sprite representations
    // This can be expanded later with actual weapon sprites

    let weapon_component: MeleeWeapon = weapon.into();

    let transform = Transform::from_translation(
        fixed_math::fixed_to_vec2(weapon_component.sprite_config.weapon_offset).extend(0.),
    )
    .with_rotation(Quat::IDENTITY);
    let ggrs_transform = fixed_math::FixedTransform3D::from_bevy_transform(&transform);

    let entity = commands
        .spawn((
            transform,
            ggrs_transform,
            weapon_component.clone(),
            id_factory.next(weapon_component.config.name.clone()),
        ))
        .insert(Rollback)
        .id();

    commands.entity(character_entity).add_child(entity);

    entity
}

// SYSTEM: MELEE ATTACK HITBOX SPAWNING
#[allow(clippy::too_many_arguments)]
pub fn spawn_melee_hitbox(
    commands: &mut Commands,
    attacker_entity: Entity,
    attacker_net_id: &GgrsNetId,
    attacker_transform: &fixed_math::FixedTransform3D,
    facing_direction: &FacingDirection,
    melee_weapon: &MeleeWeapon,
    current_frame: u32,
    collision_settings: &Res<CollisionSettings>,
    owner_handle: Option<PlayerHandle>,
    id_factory: &mut ResMut<GgrsNetIdFactory>,
    owner_team: Team,
    owner_tags: &Tags,
) -> Entity {
    let config = &melee_weapon.config;

    // Calculate hitbox position based on facing direction and range
    // Use the direction's vector to determine offset
    let direction_vec = facing_direction.to_vector();
    let direction_fixed = fixed_math::FixedVec2::new(
        fixed_math::Fixed::from_num(direction_vec.x),
        fixed_math::Fixed::from_num(direction_vec.y),
    );

    let offset = direction_fixed * config.range * fixed_math::FIXED_HALF;
    let hitbox_position = fixed_math::FixedVec3::new(
        attacker_transform.translation.x + offset.x,
        attacker_transform.translation.y + offset.y,
        attacker_transform.translation.z,
    );

    let hitbox_transform = fixed_math::FixedTransform3D::new(
        hitbox_position,
        attacker_transform.rotation.clone(),
        fixed_math::FixedVec3::ONE,
    );

    // Determine hitbox shape based on attack pattern
    let hitbox_collider = match config.attack_pattern {
        MeleeAttackPattern::SingleStrike | MeleeAttackPattern::Combo { .. } => Collider {
            shape: ColliderShape::Circle {
                radius: config.range * fixed_math::new(0.6),
            },
            offset: fixed_math::FixedVec3::ZERO,
        },
        MeleeAttackPattern::Sweep { .. } => Collider {
            shape: ColliderShape::Circle {
                radius: config.range * fixed_math::new(0.8),
            },
            offset: fixed_math::FixedVec3::ZERO,
        },
        MeleeAttackPattern::Thrust => Collider {
            shape: ColliderShape::Rectangle {
                width: config.range,
                height: config.range * fixed_math::new(0.5),
            },
            offset: fixed_math::FixedVec3::ZERO,
        },
    };

    let g_id = id_factory.next(format!("melee_hitbox_{}", config.name));

    info!(
        "{} melee hitbox spawned at {} by {:?}",
        g_id, hitbox_position, owner_handle
    );

    let layer = if owner_handle.is_some() {
        collision_settings.bullet_layer // Reuse bullet layer for player melee attacks
    } else {
        collision_settings.enemy_layer // Enemy melee attacks
    };

    // Tags du dégât (T1.1) : tags de l'attaquant union le genre d'attaque `melee` (voir la
    // doc de `sim_core::damage::DamageEvent::tags`).
    let mut tags = owner_tags.clone();
    tags.insert(Tag::new("melee"));

    let hitbox_entity = commands
        .spawn((
            MeleeHitbox {
                damage: config.damage,
                knockback_force: config.knockback_force,
                owner_entity: attacker_entity,
                owner_net_id: attacker_net_id.clone(),
                owner_handle,
                created_frame: current_frame,
                duration_frames: config.attack_duration_frames,
                facing: *facing_direction,
                owner_team,
                tags,
                friendly_fire: config.friendly_fire,
            },
            hitbox_collider,
            CollisionLayer(layer),
            hitbox_transform.to_bevy_transform(),
            hitbox_transform.clone(),
            g_id,
        ))
        .insert(Rollback)
        .id();

    hitbox_entity
}

// SYSTEM: UPDATE MELEE HITBOXES (despawn when duration expires)
pub fn update_melee_hitboxes(
    mut commands: Commands,
    frame: Res<FrameCount>,
    hitbox_query: Query<(&GgrsNetId, Entity, &MeleeHitbox), With<Rollback>>,
) {
    let system_span = span!(
        Level::INFO,
        "ggrs",
        f = frame.frame,
        s = "melee_hitbox_update"
    );
    let _enter = system_span.enter();

    for (g_id, entity, hitbox) in order_iter!(hitbox_query) {
        let frames_alive = frame.frame - hitbox.created_frame;

        if frames_alive >= hitbox.duration_frames {
            info!(
                "{} melee hitbox despawned after {} frames",
                g_id, frames_alive
            );
            use bevy_ggrs::RollbackDespawnCommandExtension;
            commands.entity(entity).despawn_rollback();
        }
    }
}

// SYSTEM: MELEE HITBOX COLLISION DETECTION
/// Collision des hitbox de mêlée (joueur ou ennemi, même système — T1.1, chantier B1).
///
/// Chaque cible en collision géométrique est filtrée par équipe et politique de tir ami
/// (`combat::team::team_allows_hit`), **pas** par `CollisionLayer`/`layer_matrix` : une
/// cible que la politique bloque (allié, tir ami `Never`) n'est pas touchée du tout (ni
/// dégât, ni recul), comme si la hitbox ne la voyait pas. Émet un `DamageEvent` (résolu par
/// `character::health::rollback_resolve_damage_events`) au lieu d'écrire
/// `DamageAccumulator` directement ; le recul (`Velocity.knockback`) reste appliqué ici,
/// inconditionnellement une fois la cible acceptée (T1.1 ne conditionne pas le recul aux
/// résistances/immunités, qui ne concernent que le *montant* du dégât). Une cible
/// invulnérable (i-frames du dash, `Health::is_invulnerable_at`) n'est pas acceptée : ni
/// dégât ni recul, et pas marquée touchée, donc une hitbox encore active à la fin des
/// i-frames la touche.
pub fn melee_hitbox_collision_system(
    frame: Res<FrameCount>,
    mut damage_events: ResMut<FrameEvents<DamageEvent>>,
    grids: Res<crate::collision_grid::CollisionGrids>,
    hitbox_query: Query<
        (
            &GgrsNetId,
            &fixed_math::FixedTransform3D,
            &MeleeHitbox,
            &Collider,
        ),
        With<Rollback>,
    >,
    mut target_query: Query<
        (
            &GgrsNetId,
            Entity,
            &fixed_math::FixedTransform3D,
            &Collider,
            &Team,
            Option<&mut Velocity>,
            Option<&Health>,
        ),
        (Without<MeleeHitbox>, With<Rollback>),
    >,
    mut attacker_query: Query<
        (
            &GgrsNetId,
            &mut MeleeAttackState,
            &fixed_math::FixedTransform3D,
        ),
        With<Rollback>,
    >,
) {
    let system_span = span!(Level::INFO, "ggrs", f = frame.frame, s = "melee_collisions");
    let _enter = system_span.enter();

    for (hitbox_g_id, hitbox_transform, hitbox, hitbox_collider) in order_iter!(hitbox_query) {
        // Find the attacker by their GgrsNetId
        let mut attacker_data = None;
        for (attacker_net_id, attack_state, attacker_transform) in order_mut_iter!(attacker_query) {
            if attacker_net_id == &hitbox.owner_net_id {
                attacker_data = Some((attack_state, attacker_transform));
                break;
            }
        }

        let Some((mut attacker_state, attacker_transform)) = attacker_data else {
            continue;
        };

        // Grille spatiale (T2.1, chantier B4b) : candidats triés par `net_id` (comme
        // `order_mut_iter!` avant), donc les `damage_events.send` ci-dessous gardent le même
        // ordre de file qu'avant — seul le nombre de candidats testés change (la boucle ne
        // parcourt plus toutes les cibles du monde, seulement celles proches de la hitbox).
        let hitbox_aabb =
            crate::collision_grid::collider_aabb(&hitbox_transform.translation, hitbox_collider);

        for target_entry in grids.characters.query_aabb(&hitbox_aabb) {
            let Ok((
                target_g_id,
                target_entity,
                target_transform,
                target_collider,
                target_team,
                opt_velocity_mut,
                opt_health,
            )) = target_query.get_mut(target_entry.entity)
            else {
                continue;
            };

            // Skip if this is the attacker
            if target_entity == hitbox.owner_entity {
                continue;
            }

            // Skip if already hit this entity in this attack
            if attacker_state.has_hit_entity(target_g_id) {
                continue;
            }

            // Équipe + politique de tir ami (remplace layer_matrix et le test
            // owner_handle/Player/Enemy d'avant T1.1).
            if !team_allows_hit(
                hitbox.owner_team,
                *target_team,
                hitbox.friendly_fire,
                &hitbox.tags,
            ) {
                continue;
            }

            // I-frames du dash : la hitbox ne voit pas la cible
            if opt_health.is_some_and(|health| health.is_invulnerable_at(frame.frame)) {
                continue;
            }

            // Check collision
            if is_colliding(
                &hitbox_transform.translation,
                hitbox_collider,
                &target_transform.translation,
                target_collider,
            ) {
                info!(
                    "Melee hitbox {} hit target {} for {} damage",
                    hitbox_g_id, target_g_id, hitbox.damage
                );

                damage_events.send(DamageEvent {
                    source: hitbox.owner_net_id.clone(),
                    target: target_g_id.clone(),
                    kind: DamageKind::Physical,
                    amount: hitbox.damage,
                    frame: frame.frame,
                    tags: hitbox.tags.clone(),
                    source_team: hitbox.owner_team,
                    friendly_fire: hitbox.friendly_fire,
                });

                // Apply knockback
                if let Some(mut velocity) = opt_velocity_mut {
                    // Calculate direction from attacker to target
                    let attacker_pos = attacker_transform.translation.truncate();
                    let target_pos = target_transform.translation.truncate();
                    let knockback_direction = (target_pos - attacker_pos).normalize_or_zero();

                    // Apply knockback force to knockback field
                    let knockback_velocity = knockback_direction * hitbox.knockback_force;
                    velocity.knockback = velocity.knockback + knockback_velocity;

                    info!(
                        "Applied knockback force {} in direction {:?} to target {} (knockback field)",
                        hitbox.knockback_force, knockback_direction, target_g_id
                    );
                }

                // Mark entity as hit
                attacker_state.add_hit_entity(target_g_id.clone());
            }
        }
    }
}

// SYSTEM: PLAYER MELEE ATTACK HANDLING
pub fn player_melee_attack_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    inputs: Res<bevy_ggrs::PlayerInputs<PeerConfig>>,
    collision_settings: Res<CollisionSettings>,
    mut id_factory: ResMut<GgrsNetIdFactory>,
    mut player_query: Query<
        (
            &GgrsNetId,
            Entity,
            &Player,
            &fixed_math::FixedTransform3D,
            &FacingDirection,
            &Children,
            &mut MeleeAttackState,
            &Team,
            Option<&Tags>,
            Has<Downed>,
            // T1.3 : `Stun`/`Freeze` (§19) : pas de mêlée.
            Option<&crate::status::Statuses>,
        ),
        With<Rollback>,
    >,
    melee_weapon_query: Query<&MeleeWeapon, With<Rollback>>,
) {
    let system_span = span!(
        Level::INFO,
        "ggrs",
        f = frame.frame,
        s = "player_melee_attack"
    );
    let _enter = system_span.enter();

    for (
        net_id,
        entity,
        player,
        transform,
        facing_direction,
        children,
        mut attack_state,
        team,
        opt_tags,
        is_downed,
        statuses,
    ) in order_mut_iter!(player_query)
    {
        let (input, _status) = inputs[player.handle];

        // Check if melee attack button is pressed — à terre (T1.3) : pas de mêlée ; étourdi
        // ou gelé (statuts, T1.3 aussi) : pas de mêlée.
        let wants_melee_attack = !is_downed
            && !crate::status::incapacitated(statuses)
            && (input.buttons & INPUT_MELEE_ATTACK != 0);

        // Find melee weapon in children
        let mut melee_weapon_opt: Option<&MeleeWeapon> = None;
        for child in children.iter() {
            if let Ok(weapon) = melee_weapon_query.get(child) {
                melee_weapon_opt = Some(weapon);
                break;
            }
        }

        if let Some(melee_weapon) = melee_weapon_opt {
            let config = &melee_weapon.config;

            // Update attack state
            if attack_state.is_attacking {
                // Check if attack duration has expired
                if let Some(start_frame) = attack_state.attack_started_frame {
                    let frames_since_start = frame.frame - start_frame;
                    if frames_since_start >= config.attack_duration_frames {
                        attack_state.end_attack(frame.frame);
                    }
                }
            } else if wants_melee_attack
                && attack_state.can_attack(frame.frame, config.cooldown_frames)
            {
                // Start new attack
                attack_state.start_attack(frame.frame);

                // Spawn hitbox
                spawn_melee_hitbox(
                    &mut commands,
                    entity,
                    net_id,
                    transform,
                    facing_direction,
                    melee_weapon,
                    frame.frame,
                    &collision_settings,
                    Some(player.handle),
                    &mut id_factory,
                    *team,
                    opt_tags.unwrap_or(&Tags::default()),
                );

                info!(
                    "Player {} started melee attack with {} at frame {}",
                    player.handle, config.name, frame.frame
                );
            }
        }
    }
}

// SYSTEM: ENEMY MELEE ATTACK HANDLING
pub fn enemy_melee_attack_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    collision_settings: Res<CollisionSettings>,
    mut id_factory: ResMut<GgrsNetIdFactory>,
    mut enemy_query: Query<
        (
            &GgrsNetId,
            Entity,
            &fixed_math::FixedTransform3D,
            &FacingDirection,
            &Children,
            &mut MeleeAttackState,
            &Team,
            Option<&Tags>,
            Has<MeleeHold>,
            Option<&WaveDamageScale>,
            // Suivi Q12 : un ennemi charmé ne griffe aucun joueur (état dérivé du statut, sans composant à
            // lever : `Statuses` est retiré dès qu'il est vide).
            Option<&crate::status::Statuses>,
        ),
        (With<Enemy>, With<Rollback>),
    >,
    player_query: Query<&fixed_math::FixedTransform3D, (With<Player>, Without<Enemy>)>,
    melee_weapon_query: Query<&MeleeWeapon, With<Rollback>>,
) {
    let system_span = span!(
        Level::INFO,
        "ggrs",
        f = frame.frame,
        s = "enemy_melee_attack"
    );
    let _enter = system_span.enter();

    for (
        net_id,
        entity,
        transform,
        facing_direction,
        children,
        mut attack_state,
        team,
        opt_tags,
        held,
        wave_scale,
        statuses,
    ) in order_mut_iter!(enemy_query)
    {
        let held = held || crate::status::charmed(statuses);
        // Find melee weapon in children
        let mut melee_weapon_opt: Option<&MeleeWeapon> = None;
        for child in children.iter() {
            if let Ok(weapon) = melee_weapon_query.get(child) {
                melee_weapon_opt = Some(weapon);
                break;
            }
        }

        if let Some(melee_weapon) = melee_weapon_opt {
            let config = &melee_weapon.config;
            let enemy_pos = transform.translation.truncate();

            // Update attack state
            if attack_state.is_attacking {
                // Check if attack duration has expired
                if let Some(start_frame) = attack_state.attack_started_frame {
                    let frames_since_start = frame.frame - start_frame;
                    if frames_since_start >= config.attack_duration_frames {
                        attack_state.end_attack(frame.frame);
                    }
                }
            } else if !held && attack_state.can_attack(frame.frame, config.cooldown_frames) {
                // Check if any player is in range
                let mut player_in_range = false;

                for player_transform in player_query.iter() {
                    let player_pos = player_transform.translation.truncate();
                    let distance = enemy_pos.distance(&player_pos);

                    // Attack if player is within range
                    if distance <= config.range * fixed_math::new(1.2) {
                        player_in_range = true;
                        break;
                    }
                }

                if player_in_range {
                    // Start new attack
                    attack_state.start_attack(frame.frame);

                    // Spawn hitbox
                    let hitbox = spawn_melee_hitbox(
                        &mut commands,
                        entity,
                        net_id,
                        transform,
                        facing_direction,
                        melee_weapon,
                        frame.frame,
                        &collision_settings,
                        None, // No player handle for enemies
                        &mut id_factory,
                        *team,
                        opt_tags.unwrap_or(&Tags::default()),
                    );
                    // D29 : dégâts de la vague (absent = vague 1 = aucun calcul).
                    if let Some(scale) = wave_scale {
                        let damage = config.damage.saturating_mul(scale.0);
                        commands.queue(move |world: &mut World| {
                            if let Some(mut hitbox) = world.get_mut::<MeleeHitbox>(hitbox) {
                                hitbox.damage = damage;
                            }
                        });
                    }

                    info!(
                        "Enemy {} started melee attack with {} at frame {}",
                        net_id, config.name, frame.frame
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------------------
// Parade et renvoi de projectiles (X-3, verbes D1)
// ---------------------------------------------------------------------------------------

use std::collections::{BTreeMap, BTreeSet};

use sim_core::timing::{grade_timing, TimingGrade};

/// Projectile renvoyé par une hitbox de mêlée, émis pour la preuve (moments clés,
/// scénarios) et la composition (un système peut lire la note). File **neutre**
/// (`add_frame_events_neutral`) : vide dans tout le contenu d'avant, aucune trace ne bouge.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ProjectileReflected {
    /// `GgrsNetId` de la balle renvoyée.
    pub bullet: GgrsNetId,
    /// `GgrsNetId` du pareur (nouveau tireur).
    pub owner: GgrsNetId,
    /// Note de la fenêtre de jugement (`sim_core::timing`, repère = début du coup).
    pub grade: TimingGrade,
    pub frame: u32,
}

/// Coup de mêlée adverse annulé par une parade (`melee_parry_system`). Même règle que
/// [`ProjectileReflected`] : file **neutre**, vide dans tout le contenu d'avant.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct MeleeParried {
    /// `GgrsNetId` du porteur de l'arme qui pare.
    pub parrier: GgrsNetId,
    /// `GgrsNetId` de l'attaquant dont le coup est annulé.
    pub attacker: GgrsNetId,
    /// `GgrsNetId` de la hitbox annulée.
    pub hitbox: GgrsNetId,
    /// Note de la fenêtre de jugement (repère = création de la hitbox du pareur).
    pub grade: TimingGrade,
    pub frame: u32,
}

/// Direction de regard en vecteur `Fixed` unitaire (ou nul, jamais : les huit
/// directions sont non nulles). Pure, déterministe, sans `f32`.
pub fn facing_to_fixed(direction: FacingDirection) -> fixed_math::FixedVec2 {
    let (x, y) = match direction {
        FacingDirection::Right => (1, 0),
        FacingDirection::UpRight => (1, 1),
        FacingDirection::Up => (0, 1),
        FacingDirection::UpLeft => (-1, 1),
        FacingDirection::Left => (-1, 0),
        FacingDirection::DownLeft => (-1, -1),
        FacingDirection::Down => (0, -1),
        FacingDirection::DownRight => (1, -1),
    };
    fixed_math::FixedVec2::new(
        fixed_math::Fixed::from_num(x),
        fixed_math::Fixed::from_num(y),
    )
    .normalize_or_zero()
}

/// Arme de mêlée du porteur (`MeleeWeapon` enfant), la première des enfants.
fn melee_weapon_of<'a>(
    children: &Children,
    weapon_query: &'a Query<&MeleeWeapon>,
) -> Option<&'a MeleeWeapon> {
    children
        .iter()
        .find_map(|child| weapon_query.get(child).ok())
}

/// Parade de mêlée contre mêlée (X-3, verbes D1, D149).
///
/// Pour chaque hitbox active dont l'arme porte `parry` (ordre `GgrsNetId`), toute hitbox de
/// mêlée **hostile** à son porteur (`team_allows_hit` : jamais sa propre équipe ni un allié
/// protégé du tir ami) qui la touche géométriquement est **annulée** (`despawn_rollback`) avant
/// de pouvoir blesser. Les contacts sont tous décidés sur le même état (une copie triée des
/// hitbox actives de la frame), puis appliqués : deux armes qui se parent mutuellement
/// s'annulent toutes les deux, quel que soit l'ordre ; une hitbox touchée par plusieurs pareurs
/// est créditée au plus petit `GgrsNetId`. Note `Perfect` (contact au plus `perfect_frames`
/// après la création de la hitbox du pareur) : l'attaquant reçoit en plus
/// `perfect_knockback` (axe pareur → attaquant, `Velocity::knockback`). Émet
/// `FrameEvents<MeleeParried>`.
///
/// Placement : après la création des hitbox de la frame et l'expiration des anciennes
/// (`update_melee_hitboxes`), **avant** le renvoi de projectiles (une hitbox annulée ne renvoie
/// plus rien) et leurs collisions (`melee_hitbox_collision_system`, qui blesse : une hitbox
/// annulée n'y est plus vue).
pub fn melee_parry_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    // Absente hors jeu complet (enregistrée par `game::core_sim`) : la parade agit, sans événement.
    mut parried_events: Option<ResMut<FrameEvents<MeleeParried>>>,
    hitbox_query: Query<
        (
            &GgrsNetId,
            Entity,
            &fixed_math::FixedTransform3D,
            &MeleeHitbox,
            &Collider,
        ),
        With<Rollback>,
    >,
    owner_query: Query<(&GgrsNetId, &Children), With<Rollback>>,
    weapon_query: Query<&MeleeWeapon>,
    mut character_query: Query<
        (
            &GgrsNetId,
            &fixed_math::FixedTransform3D,
            Option<&mut Velocity>,
        ),
        (With<Rollback>, Without<MeleeHitbox>),
    >,
) {
    let system_span = span!(Level::INFO, "ggrs", f = frame.frame, s = "melee_parry");
    let _enter = system_span.enter();
    let now = frame.frame;

    // Arme de parade par porteur (une seule fois par porteur), seulement si une hitbox est active.
    let mut parry_of: BTreeMap<usize, Option<MeleeParry>> = BTreeMap::new();
    let hitboxes = order_iter!(hitbox_query);
    if hitboxes.len() < 2 {
        return;
    }
    for (_, _, _, hitbox, _) in &hitboxes {
        parry_of.entry(hitbox.owner_net_id.0).or_insert_with(|| {
            owner_query
                .iter()
                .find(|(id, _)| id.0 == hitbox.owner_net_id.0)
                .and_then(|(_, children)| melee_weapon_of(children, &weapon_query))
                .and_then(|weapon| weapon.config.parry.clone())
        });
    }
    if parry_of.values().all(Option::is_none) {
        return;
    }

    // Contacts décidés sur l'état de la frame : hitbox annulée (indice) -> pareur (indice).
    let mut cancelled: BTreeMap<usize, usize> = BTreeMap::new();
    for (i, (_, _, parrier_transform, parrier, parrier_collider)) in hitboxes.iter().enumerate() {
        if parry_of
            .get(&parrier.owner_net_id.0)
            .is_none_or(Option::is_none)
        {
            continue;
        }
        for (j, (_, _, other_transform, other, other_collider)) in hitboxes.iter().enumerate() {
            if i == j || other.owner_net_id.0 == parrier.owner_net_id.0 {
                continue;
            }
            if cancelled.contains_key(&j) {
                continue;
            }
            if !team_allows_hit(
                other.owner_team,
                parrier.owner_team,
                other.friendly_fire,
                &other.tags,
            ) {
                continue;
            }
            if is_colliding(
                &parrier_transform.translation,
                parrier_collider,
                &other_transform.translation,
                other_collider,
            ) {
                cancelled.insert(j, i);
            }
        }
    }
    if cancelled.is_empty() {
        return;
    }

    let positions: BTreeMap<usize, fixed_math::FixedVec2> = character_query
        .iter()
        .map(|(id, transform, _)| (id.0, transform.translation.truncate()))
        .collect();
    use bevy_ggrs::RollbackDespawnCommandExtension;
    for (j, i) in cancelled {
        let (hitbox_id, hitbox_entity, _, other, _) = &hitboxes[j];
        let (_, _, _, parrier, _) = &hitboxes[i];
        let Some(Some(parry)) = parry_of.get(&parrier.owner_net_id.0) else {
            continue;
        };
        let grade = grade_timing(now, parrier.created_frame, parry.perfect_frames, u32::MAX);
        commands.entity(*hitbox_entity).despawn_rollback();
        if grade == TimingGrade::Perfect && parry.perfect_knockback > fixed_math::Fixed::ZERO {
            if let (Some(from), Some(to)) = (
                positions.get(&parrier.owner_net_id.0),
                positions.get(&other.owner_net_id.0),
            ) {
                let direction = (*to - *from).normalize_or_zero();
                let push = direction * parry.perfect_knockback;
                for (id, _, velocity) in character_query.iter_mut() {
                    if id.0 == other.owner_net_id.0 {
                        if let Some(mut velocity) = velocity {
                            velocity.knockback = velocity.knockback + push;
                        }
                        break;
                    }
                }
            }
        }
        info!(
            "ggrs{{f={} parry hitbox={} parrier={} attacker={} grade={:?}}}",
            now, hitbox_id, parrier.owner_net_id, other.owner_net_id, grade
        );
        if let Some(events) = parried_events.as_mut() {
            events.send(MeleeParried {
                parrier: parrier.owner_net_id.clone(),
                attacker: other.owner_net_id.clone(),
                hitbox: (*hitbox_id).clone(),
                grade,
                frame: now,
            });
        }
    }
}

/// Renvoi des projectiles par les hitbox de mêlée dont l'arme porte `reflect`.
///
/// Pour chaque hitbox active (ordre `GgrsNetId`), si l'arme du porteur déclare `reflect` :
/// tout projectile hostile au porteur (même filtre que s'il allait le toucher :
/// `team_allows_hit`, jamais sa propre équipe) en contact géométrique avec la hitbox
/// est renvoyé : `source`/`source_team` deviennent le pareur et son équipe (la balle
/// touche désormais ses anciens alliés), la vitesse garde son module et prend la
/// direction du regard, les personnages déjà traversés sont oubliés (le projectile peut
/// toucher son ancien camp). Note (`Perfect`/`Good`, `sim_core::timing`) mesurée depuis la
/// création de la hitbox (le début du coup) ; sur `Perfect`, les dégâts sont multipliés par
/// `perfect_damage_mult`. Une balle n'est renvoyée qu'une fois par frame (la hitbox de
/// plus petit `GgrsNetId` l'emporte) ; une balle terminée (`Projectile::ended`) ne l'est jamais.
///
/// Placement (`RollbackSystemSet::Weapon`) : **après** la création des hitbox de la frame
/// (`player_melee_attack_system`, `enemy_melee_attack_system` : un coup lancé cette frame
/// pare dès la frame 0, note `Perfect` à `perfect_frames = 0`) et leur expiration
/// (`update_melee_hitboxes` : une hitbox arrivée au bout de sa durée ne renvoie plus rien),
/// après la parade de mêlée, et **avant** les collisions de balles (la balle renvoyée ne
/// touche pas le pareur la même frame) : une balle ordinaire comme composable suit ensuite son
/// chemin normal, vers son ancien camp. Ordre `GgrsNetId` partout (hitbox, balles, porteurs) ;
/// aucun RNG, aucun `f32`.
#[allow(clippy::too_many_arguments)]
pub fn melee_reflect_system(
    frame: Res<FrameCount>,
    // Absente hors jeu complet (enregistrée par `game::core_sim`) : la parade agit, sans événement.
    mut reflected_events: Option<ResMut<FrameEvents<ProjectileReflected>>>,
    hitbox_query: Query<
        (
            &GgrsNetId,
            &fixed_math::FixedTransform3D,
            &MeleeHitbox,
            &Collider,
        ),
        With<Rollback>,
    >,
    mut bullet_query: Query<
        (
            &GgrsNetId,
            &fixed_math::FixedTransform3D,
            &mut super::Bullet,
            Option<&mut crate::projectile::Projectile>,
            &Collider,
        ),
        With<Rollback>,
    >,
    owner_query: Query<
        (
            &GgrsNetId,
            &FacingDirection,
            Option<&Player>,
            &Team,
            &Children,
        ),
        With<Rollback>,
    >,
    weapon_query: Query<&MeleeWeapon>,
) {
    let system_span = span!(Level::INFO, "ggrs", f = frame.frame, s = "melee_reflect");
    let _enter = system_span.enter();
    let now = frame.frame;
    let mut reflected_this_frame: BTreeSet<usize> = BTreeSet::new();

    for (_hitbox_id, hitbox_transform, hitbox, hitbox_collider) in order_iter!(hitbox_query) {
        let mut owner_data = None;
        for (owner_net_id, facing, player, team, children) in order_iter!(owner_query) {
            if owner_net_id == &hitbox.owner_net_id {
                owner_data = Some((owner_net_id, facing, player, team, children));
                break;
            }
        }
        let Some((owner_net_id, facing, player, owner_team, children)) = owner_data else {
            continue;
        };
        let Some(reflect) = melee_weapon_of(children, &weapon_query)
            .and_then(|weapon| weapon.config.reflect.clone())
        else {
            continue;
        };
        let started = hitbox.created_frame;
        let direction = facing_to_fixed(*facing);

        for (bullet_id, bullet_transform, mut bullet, mut projectile, bullet_collider) in
            order_mut_iter!(bullet_query)
        {
            if reflected_this_frame.contains(&bullet_id.0) {
                continue;
            }
            if bullet.source_team == *owner_team
                || !team_allows_hit(
                    bullet.source_team,
                    *owner_team,
                    bullet.friendly_fire,
                    &bullet.tags,
                )
                || !projectile.as_ref().is_none_or(|p| !p.ended)
                || !is_colliding(
                    &hitbox_transform.translation,
                    hitbox_collider,
                    &bullet_transform.translation,
                    bullet_collider,
                )
            {
                continue;
            }
            reflected_this_frame.insert(bullet_id.0);
            let grade = grade_timing(now, started, reflect.perfect_frames, u32::MAX);
            let (speed, _) = bullet.velocity.length_and_direction();
            bullet.velocity = fixed_math::FixedVec2::new(
                direction.x.saturating_mul(speed),
                direction.y.saturating_mul(speed),
            );
            bullet.source = owner_net_id.clone();
            bullet.source_team = *owner_team;
            if let Some(player) = player {
                bullet.player_handle = player.handle;
            }
            if grade == TimingGrade::Perfect {
                bullet.damage = bullet.damage.saturating_mul(reflect.perfect_damage_mult);
            }
            if let Some(projectile) = projectile.as_deref_mut() {
                projectile.hits.clear();
            }
            info!(
                "ggrs{{f={} reflect bullet={} owner={} grade={:?}}}",
                now, bullet_id, owner_net_id, grade
            );
            if let Some(events) = reflected_events.as_mut() {
                events.send(ProjectileReflected {
                    bullet: bullet_id.clone(),
                    owner: owner_net_id.clone(),
                    grade,
                    frame: now,
                });
            }
        }
    }
}

#[cfg(test)]
mod system_tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    const FRAME: u32 = 100;

    /// Monde d'essai : le crochet d'ajout de `Rollback` exige la ressource `RollbackOrdered`.
    fn test_world() -> World {
        let mut world = World::new();
        world.init_resource::<bevy_ggrs::RollbackOrdered>();
        world.insert_resource(FrameCount { frame: FRAME });
        world
    }

    fn at(x: f32, y: f32) -> fixed_math::FixedTransform3D {
        fixed_math::FixedTransform3D::new(
            fixed_math::FixedVec3::new(
                fixed_math::Fixed::from_num(x),
                fixed_math::Fixed::from_num(y),
                fixed_math::Fixed::ZERO,
            ),
            fixed_math::FixedMat3::IDENTITY,
            fixed_math::FixedVec3::ONE,
        )
    }

    fn circle(radius: f32) -> Collider {
        Collider {
            shape: ColliderShape::Circle {
                radius: fixed_math::Fixed::from_num(radius),
            },
            offset: fixed_math::FixedVec3::ZERO,
        }
    }

    fn melee_weapon(reflect: Option<MeleeReflect>, parry: Option<MeleeParry>) -> MeleeWeapon {
        MeleeWeapon {
            config: MeleeWeaponConfig {
                name: "essai".into(),
                damage: fixed_math::Fixed::from_num(5.0),
                range: fixed_math::Fixed::from_num(50.0),
                attack_pattern: MeleeAttackPattern::SingleStrike,
                attack_duration_frames: 12,
                cooldown_frames: 40,
                knockback_force: fixed_math::Fixed::from_num(20.0),
                stamina_cost: fixed_math::Fixed::ZERO,
                friendly_fire: FriendlyFire::Never,
                test: None,
                reflect,
                parry,
            },
            sprite_config: MeleeWeaponSpriteConfig {
                name: "epee".into(),
                index: 0,
                weapon_offset: fixed_math::FixedVec2::ZERO,
            },
        }
    }

    fn reflect_config() -> MeleeReflect {
        MeleeReflect {
            perfect_frames: 0,
            perfect_damage_mult: fixed_math::Fixed::from_num(2.0),
        }
    }

    fn parry_config() -> MeleeParry {
        MeleeParry {
            perfect_frames: 4,
            perfect_knockback: fixed_math::Fixed::from_num(300.0),
        }
    }

    /// Porteur d'une arme de mêlée (X-3) : l'arme en enfant, l'équipe et le regard donnés.
    fn owner(
        world: &mut World,
        id: usize,
        team: Team,
        facing: FacingDirection,
        weapon: MeleeWeapon,
    ) -> Entity {
        let weapon = world.spawn(weapon).id();
        let owner = world
            .spawn((GgrsNetId(id, "essai".into()), facing, team, Rollback))
            .id();
        world.entity_mut(owner).add_child(weapon);
        owner
    }

    fn hitbox(
        world: &mut World,
        id: usize,
        owner_entity: Entity,
        owner_net_id: usize,
        team: Team,
        created_frame: u32,
        x: f32,
    ) -> Entity {
        world
            .spawn((
                GgrsNetId(id, "essai".into()),
                MeleeHitbox {
                    damage: fixed_math::Fixed::from_num(5.0),
                    knockback_force: fixed_math::Fixed::from_num(20.0),
                    owner_entity,
                    owner_net_id: GgrsNetId(owner_net_id, "essai".into()),
                    owner_handle: None,
                    created_frame,
                    duration_frames: 12,
                    facing: FacingDirection::Right,
                    owner_team: team,
                    tags: Tags::default(),
                    friendly_fire: FriendlyFire::Never,
                },
                circle(30.0),
                at(x, 0.0),
                Rollback,
            ))
            .id()
    }

    fn bullet(world: &mut World, id: usize, x: f32, speed: f32) -> Entity {
        world
            .spawn((
                GgrsNetId(id, "essai".into()),
                super::super::Bullet {
                    velocity: fixed_math::FixedVec2::new(
                        fixed_math::Fixed::from_num(speed),
                        fixed_math::Fixed::ZERO,
                    ),
                    bullet_type: super::super::BulletType::Standard {
                        damage: fixed_math::Fixed::from_num(8.0),
                        speed: fixed_math::Fixed::from_num(1.0),
                    },
                    damage: fixed_math::Fixed::from_num(8.0),
                    range: fixed_math::Fixed::from_num(1000.0),
                    distance_traveled: fixed_math::Fixed::ZERO,
                    player_handle: 0,
                    created_at: 0,
                    source: GgrsNetId(40, "tourelle".into()),
                    source_team: Team::Enemies,
                    tags: Tags::default(),
                    friendly_fire: FriendlyFire::Never,
                },
                circle(4.0),
                at(x, 0.0),
                Rollback,
            ))
            .id()
    }

    /// X-3 (revue, P2) : un projectile qui touche la hitbox **à la frame de sa création**
    /// est renvoyé avec la note `Perfect` (fenêtre parfaite nulle) : la parade passe après
    /// la création des hitbox de la frame.
    #[test]
    fn un_contact_a_la_frame_du_coup_est_renvoye_parfait() {
        let mut world = test_world();
        world.init_resource::<FrameEvents<ProjectileReflected>>();
        let pareur = owner(
            &mut world,
            7,
            Team::Players,
            FacingDirection::Right,
            melee_weapon(Some(reflect_config()), None),
        );
        hitbox(&mut world, 20, pareur, 7, Team::Players, FRAME, 25.0);
        let balle = bullet(&mut world, 29, 25.0, -1.0);
        world.run_system_once(melee_reflect_system).unwrap();
        let balle = world.get::<super::super::Bullet>(balle).unwrap();
        assert_eq!(balle.source_team, Team::Players);
        assert_eq!(balle.source, GgrsNetId(7, "essai".into()));
        assert!(
            balle.velocity.x > fixed_math::Fixed::ZERO,
            "repart vers la droite"
        );
        assert_eq!(
            balle.damage,
            fixed_math::Fixed::from_num(16.0),
            "parfait : dégâts doublés"
        );
        let notes: Vec<_> = world
            .resource::<FrameEvents<ProjectileReflected>>()
            .iter()
            .collect();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].grade, TimingGrade::Perfect);
        assert_eq!(notes[0].frame, FRAME);
    }

    /// X-3 (revue, P2) : une hitbox arrivée au bout de sa durée (`update_melee_hitboxes`
    /// passe avant la parade) est détruite avant la parade : le projectile qui la touche
    /// n'est pas renvoyé.
    #[test]
    fn une_hitbox_expiree_ne_renvoie_plus_rien() {
        let mut world = test_world();
        world.init_resource::<FrameEvents<ProjectileReflected>>();
        let pareur = owner(
            &mut world,
            7,
            Team::Players,
            FacingDirection::Right,
            melee_weapon(Some(reflect_config()), None),
        );
        let boite = hitbox(&mut world, 20, pareur, 7, Team::Players, FRAME - 12, 25.0);
        let balle = bullet(&mut world, 29, 25.0, -1.0);
        world.run_system_once(update_melee_hitboxes).unwrap();
        assert!(
            world.get::<MeleeHitbox>(boite).is_none(),
            "hitbox expirée détruite avant la parade"
        );
        world.run_system_once(melee_reflect_system).unwrap();
        let balle = world.get::<super::super::Bullet>(balle).unwrap();
        assert_eq!(balle.source_team, Team::Enemies, "équipe inchangée");
        assert_eq!(
            balle.damage,
            fixed_math::Fixed::from_num(8.0),
            "dégâts inchangés"
        );
        assert_eq!(
            world
                .resource::<FrameEvents<ProjectileReflected>>()
                .iter()
                .count(),
            0,
            "aucune note"
        );
    }

    /// Personnage pour une parade de mêlée (X-3) : position et vitesse, sans hitbox.
    fn character(world: &mut World, id: usize, team: Team, x: f32) -> Entity {
        world
            .spawn((
                GgrsNetId(id, "essai".into()),
                team,
                crate::actors::Velocity {
                    main: fixed_math::FixedVec2::ZERO,
                    knockback: fixed_math::FixedVec2::ZERO,
                },
                at(x, 0.0),
                Rollback,
            ))
            .id()
    }

    fn parry_world() -> (World, Entity, Entity) {
        let mut world = test_world();
        world.init_resource::<FrameEvents<MeleeParried>>();
        let pareur = owner(
            &mut world,
            7,
            Team::Players,
            FacingDirection::Right,
            melee_weapon(None, Some(parry_config())),
        );
        let attaquant = owner(
            &mut world,
            14,
            Team::Enemies,
            FacingDirection::Left,
            melee_weapon(None, None),
        );
        character(&mut world, 7, Team::Players, 0.0);
        character(&mut world, 14, Team::Enemies, -40.0);
        (world, pareur, attaquant)
    }

    /// X-3 (D149) : la hitbox adverse qui touche celle du pareur est annulée avant de blesser ;
    /// contact dans la fenêtre parfaite : l'attaquant est repoussé et la note est émise.
    #[test]
    fn un_coup_adverse_est_annule_et_l_attaquant_recule_sur_parfait() {
        let (mut world, pareur, attaquant) = parry_world();
        let garde = hitbox(&mut world, 20, pareur, 7, Team::Players, FRAME - 2, 25.0);
        let coup = hitbox(
            &mut world,
            21,
            attaquant,
            14,
            Team::Enemies,
            FRAME - 6,
            -15.0,
        );
        world.run_system_once(melee_parry_system).unwrap();
        assert!(
            world.get::<MeleeHitbox>(coup).is_none(),
            "le coup adverse est annulé"
        );
        assert!(
            world.get::<MeleeHitbox>(garde).is_some(),
            "la hitbox du pareur survit"
        );
        let notes: Vec<_> = world
            .resource::<FrameEvents<MeleeParried>>()
            .iter()
            .collect();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].grade, TimingGrade::Perfect);
        assert_eq!(notes[0].parrier, GgrsNetId(7, "essai".into()));
        assert_eq!(notes[0].attacker, GgrsNetId(14, "essai".into()));
        let mut vus = 0;
        for (id, _, velocity) in world
            .query::<(
                &GgrsNetId,
                &fixed_math::FixedTransform3D,
                Option<&crate::actors::Velocity>,
            )>()
            .iter(&world)
        {
            if id.0 == 14 {
                let recul = velocity.unwrap().knockback;
                assert!(
                    recul.x < fixed_math::Fixed::ZERO,
                    "repoussé loin du pareur, reçu {recul:?}"
                );
                vus += 1;
            }
        }
        assert_eq!(vus, 1);
    }

    /// X-3 (D149) : hors de la fenêtre parfaite, le coup est annulé sans recul (« bien ») ;
    /// deux pareurs mutuels s'annulent tous les deux, quel que soit l'ordre.
    #[test]
    fn un_coup_bien_pare_est_annule_sans_recul_et_mutuellement() {
        let (mut world, pareur, attaquant) = parry_world();
        hitbox(&mut world, 20, pareur, 7, Team::Players, FRAME - 8, 25.0);
        let coup = hitbox(
            &mut world,
            21,
            attaquant,
            14,
            Team::Enemies,
            FRAME - 6,
            -15.0,
        );
        world.run_system_once(melee_parry_system).unwrap();
        assert!(world.get::<MeleeHitbox>(coup).is_none());
        let notes: Vec<_> = world
            .resource::<FrameEvents<MeleeParried>>()
            .iter()
            .collect();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].grade, TimingGrade::Good);
        for (id, _, velocity) in world
            .query::<(
                &GgrsNetId,
                &fixed_math::FixedTransform3D,
                Option<&crate::actors::Velocity>,
            )>()
            .iter(&world)
        {
            if id.0 == 14 {
                assert_eq!(
                    velocity.unwrap().knockback,
                    fixed_math::FixedVec2::ZERO,
                    "aucun recul sur « bien »"
                );
            }
        }
        // Parade mutuelle : les deux armes parent, les deux coups tombent.
        let (mut world, pareur, attaquant) = parry_world();
        let coup_pareur = hitbox(&mut world, 20, pareur, 7, Team::Players, FRAME - 2, 25.0);
        let coup_attaquant = hitbox(
            &mut world,
            21,
            attaquant,
            14,
            Team::Enemies,
            FRAME - 6,
            -15.0,
        );
        // L'attaquant pare aussi : on lui donne la parade après coup.
        let enfants: Vec<Entity> = world
            .query::<(&GgrsNetId, &Children)>()
            .iter(&world)
            .flat_map(|(_, children)| children.iter())
            .collect();
        for child in enfants {
            if let Some(mut arme) = world.get_mut::<MeleeWeapon>(child) {
                if arme.config.parry.is_none() {
                    arme.config.parry = Some(parry_config());
                }
            }
        }
        let _ = coup_pareur;
        world.run_system_once(melee_parry_system).unwrap();
        assert!(world.get::<MeleeHitbox>(coup_pareur).is_none());
        assert!(world.get::<MeleeHitbox>(coup_attaquant).is_none());
        assert_eq!(
            world.resource::<FrameEvents<MeleeParried>>().iter().count(),
            2,
            "un événement par coup annulé"
        );
    }
}

#[cfg(test)]
mod reflect_tests {
    use super::*;

    #[test]
    fn regard_donne_un_vecteur_unitaire_fixe() {
        let right = facing_to_fixed(FacingDirection::Right);
        assert_eq!(right.x, fixed_math::Fixed::from_num(1));
        assert_eq!(right.y, fixed_math::Fixed::ZERO);
        let up = facing_to_fixed(FacingDirection::Up);
        assert_eq!(up.x, fixed_math::Fixed::ZERO);
        assert_eq!(up.y, fixed_math::Fixed::from_num(1));
        let left = facing_to_fixed(FacingDirection::Left);
        assert_eq!(left.x, fixed_math::Fixed::from_num(-1));
        // Diagonale : symétrique, de module 1 au bit près du CORDIC.
        let diag = facing_to_fixed(FacingDirection::UpRight);
        assert_eq!(diag.x, diag.y);
        assert!(diag.x > fixed_math::Fixed::ZERO);
    }
}
