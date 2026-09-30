use animation::{FacingDirection, SpriteSheetConfig};
use bevy::{
    log::{tracing::span, Level},
    platform::collections::HashMap,
    prelude::*,
};
use bevy_fixed::fixed_math;
use bevy_ggrs::Rollback;
use combat::downed::Downed;
use combat::team::team_allows_hit;
use ggrs::PlayerHandle;
use serde::{Deserialize, Serialize};
use sim_core::damage::{DamageEvent, DamageKind, FriendlyFire};
use sim_core::tag::{Tag, Tags};
use sim_core::team::Team;
use utils::{
    net_id::{GgrsNetId, GgrsNetIdFactory},
    order_iter, order_mut_iter,
};

use crate::{
    character::{
        enemy::Enemy,
        movement::Velocity,
        player::{input::INPUT_MELEE_ATTACK, jjrs::PeerConfig, Player},
    },
    collider::{is_colliding, Collider, ColliderShape, CollisionLayer, CollisionSettings},
    frame_events::FrameEvents,
    global_asset::GlobalAsset,
};
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

// MELEE WEAPON CONFIG
#[derive(Debug, Clone, Hash, Serialize, Deserialize, PartialEq)]
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

// SLASH VISUAL EFFECT
#[derive(Component)]
pub struct SlashEffect {
    pub start_frame: u32,
    pub duration_frames: u32,
    pub frame_duration: u32, // Frames per animation frame
    pub animation_start: usize,
    pub animation_end: usize, // Inclusive
}

// MELEE WEAPONS CONFIG ASSET
#[derive(Asset, TypePath, Serialize, Deserialize)]
pub struct MeleeWeaponsConfig(pub HashMap<String, MeleeWeaponAsset>);

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
            Visibility::Hidden, // Melee weapons can be invisible or shown only during attack
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

// SYSTEM: UPDATE SLASH VISUAL EFFECTS
/// Présentation : crée l'effet de slash d'une hitbox de mêlée. Une hitbox recréée
/// ou resimulée par un rollback (même GgrsNetId) ne produit pas de second effet.
pub fn spawn_slash_effects(
    mut commands: Commands,
    global_assets: Res<GlobalAsset>,
    spritesheet_assets: Res<Assets<SpriteSheetConfig>>,
    animation_configs: Res<Assets<animation::AnimationMapConfig>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    frame: Res<FrameCount>,
    hitboxes: Query<(&GgrsNetId, &MeleeHitbox, &fixed_math::FixedTransform3D)>,
    mut shown: Local<HashMap<usize, u32>>,
) {
    // Oublier les hitbox trop anciennes pour être encore rejouées par un rollback
    shown.retain(|_, created_frame| frame.frame.saturating_sub(*created_frame) < 600);

    let (Some(slash_config), Some(anim_config)) = (
        spritesheet_assets.get(&global_assets.slash_effect_spritesheet),
        animation_configs.get(&global_assets.slash_effect_animation),
    ) else {
        return;
    };

    for (net_id, hitbox, hitbox_transform) in hitboxes.iter() {
        if shown.insert(net_id.0, hitbox.created_frame).is_some() {
            continue;
        }

        let texture_handle: Handle<Image> = asset_server.load(&slash_config.path);
        let layout = TextureAtlasLayout::from_grid(
            UVec2::new(slash_config.tile_size.0, slash_config.tile_size.1),
            slash_config.columns,
            slash_config.rows,
            None,
            None,
        );
        let layout_handle = texture_atlas_layouts.add(layout);

        // Get animation configuration dynamically
        let slash_anim = anim_config
            .animations
            .get("slash")
            .expect("slash animation not found in config");
        let columns = slash_config.columns;
        let (start, end) = slash_anim.to_absolute(columns);
        let frame_duration = anim_config.frame_duration as u32;
        let animation_frame_count = (end - start + 1) as u32;
        let total_duration = animation_frame_count * frame_duration;

        // Position effect at hitbox location
        let mut effect_transform = hitbox_transform.to_bevy_transform();
        effect_transform.translation.z = 5.0; // Place above everything

        // For 8-directional slashes, we need to handle flipping carefully
        // The sprite is designed for right-facing attacks (0 degrees)
        // For left-facing, we flip and use the opposite angle
        let (flip_x, rotation_angle) = match hitbox.facing {
            FacingDirection::Right => (false, 0.0),
            FacingDirection::UpRight => (false, std::f32::consts::PI / 4.0),
            FacingDirection::Up => (false, std::f32::consts::PI / 2.0),
            FacingDirection::UpLeft => (true, -std::f32::consts::PI / 4.0), // Flip + negative angle for upper left
            FacingDirection::Left => (true, 0.0),                           // Flip + 0° for left
            FacingDirection::DownLeft => (true, std::f32::consts::PI / 4.0), // Flip + positive angle for lower left
            FacingDirection::Down => (false, -std::f32::consts::PI / 2.0),
            FacingDirection::DownRight => (false, -std::f32::consts::PI / 4.0),
        };

        effect_transform.rotation = Quat::from_rotation_z(rotation_angle);

        commands.spawn((
            SlashEffect {
                start_frame: hitbox.created_frame,
                duration_frames: total_duration,
                frame_duration,
                animation_start: start,
                animation_end: end,
            },
            Sprite {
                image: texture_handle,
                texture_atlas: Some(TextureAtlas {
                    layout: layout_handle,
                    index: start, // Start at the correct animation frame
                }),
                flip_x,
                flip_y: false,
                ..default()
            },
            effect_transform,
        ));
    }
}

pub fn update_slash_effects(
    mut commands: Commands,
    frame: Res<FrameCount>,
    mut slash_query: Query<(Entity, &SlashEffect, &mut Sprite)>,
) {
    for (entity, slash_effect, mut sprite) in slash_query.iter_mut() {
        let frames_alive = frame.frame - slash_effect.start_frame;

        // Calculate current animation frame dynamically based on SlashEffect config
        let animation_progress = frames_alive / slash_effect.frame_duration;
        let animation_frame_count =
            (slash_effect.animation_end - slash_effect.animation_start + 1) as u32;
        let current_animation_frame = animation_progress.min(animation_frame_count - 1) as usize;
        let sprite_index = slash_effect.animation_start + current_animation_frame;

        if let Some(ref mut atlas) = sprite.texture_atlas {
            atlas.index = sprite_index;
        }

        // Despawn when animation is complete
        if frames_alive >= slash_effect.duration_frames {
            commands.entity(entity).despawn();
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
/// résistances/immunités/invulnérabilité, qui ne concernent que le *montant* du dégât).
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
    ) in order_mut_iter!(player_query)
    {
        let (input, _status) = inputs[player.handle];

        // Check if melee attack button is pressed — à terre (T1.3) : pas de mêlée.
        let wants_melee_attack = !is_downed && (input.buttons & INPUT_MELEE_ATTACK != 0);

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

    for (net_id, entity, transform, facing_direction, children, mut attack_state, team, opt_tags) in
        order_mut_iter!(enemy_query)
    {
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
            } else if attack_state.can_attack(frame.frame, config.cooldown_frames) {
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
                    spawn_melee_hitbox(
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

                    info!(
                        "Enemy {} started melee attack with {} at frame {}",
                        net_id, config.name, frame.frame
                    );
                }
            }
        }
    }
}
