//! Présentation des personnages : sprites animés et barre de vie.
//!
//! La simulation ne crée que l'entité logique avec un [`CharacterAppearance`](super::appearance::CharacterAppearance).
//! [`attach_character_visuals`] y ajoute les sprites dès qu'il voit une entité sans
//! visuels, y compris une entité recréée par un rollback.

use std::collections::BTreeMap;

use animation::{
    create_child_sprite, AnimationMapConfig, AnimationVisualsBundle, PresentationClip,
    SpriteSheetConfig,
};
use bevy::prelude::*;
use utils::{frame::FrameCount, net_id::GgrsNetId};

use crate::global_asset::GlobalAsset;

use super::{appearance::CharacterAppearance, config::CharacterConfigs, health_bar::HealthBar};

/// Teinte de base des calques (D66, `CharacterConfig::marker`) : relue par
/// `feedback::apply_layer_colors` à la place du blanc. Présentation, hors rollback.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct MarkerTint(pub Color);

/// Frames de simulation pendant lesquelles un coup reste « en cours » pour la pose d'attaque de présentation :
/// un peu plus que le délai entre deux coups d'un zombie (`attack_cooldown_frames` : 45 pour le zombie et le
/// boss), pour que des coups enchaînés ne retombent pas en `Run` entre deux frappes.
pub const ATTACK_POSE_FRAMES: u32 = 48;

/// Vrai si le coup `last_attack_frame` date de moins de [`ATTACK_POSE_FRAMES`] frames. Pure, pour les tests.
pub fn struck_recently(frame: u32, last_attack_frame: u32) -> bool {
    frame >= last_attack_frame && frame - last_attack_frame <= ATTACK_POSE_FRAMES
}

/// Pose d'attaque (S1-8, présentation, `Update`) : tant qu'un ennemi est en `MonsterState::Attacking` et que son
/// dernier coup est récent, la clé `"Attack"` est proposée à l'animation, qui l'applique **si** sa carte la définit
/// (les feuilles publiques n'en ont pas : rien ne change). Lit l'état rollback existant et la frame courante ;
/// n'écrit ni dans la simulation ni dans `AnimationState`. Même clé de présentation que la pose `Shoot` de X-4b
/// (`PresentationClip`) : les ennemis portent aussi un `WeaponInventory`, [`present_shooting`] les exclut
/// (`Without<MonsterState>`) pour que les deux systèmes ne se disputent aucune entité.
pub fn present_attacking(
    frame: Res<FrameCount>,
    mut enemies: Query<(
        &crate::character::enemy::ai::state::MonsterState,
        &mut animation::PresentationClip,
    )>,
) {
    use crate::character::enemy::ai::state::MonsterState;
    for (state, mut clip) in &mut enemies {
        let attacking = matches!(
            state,
            MonsterState::Attacking { last_attack_frame, .. }
                if struck_recently(frame.frame, *last_attack_frame)
        );
        let wanted = attacking.then(|| "Attack".to_string());
        if clip.0 != wanted {
            clip.0 = wanted;
        }
    }
}

/// Marque une entité dont les visuels ont été créés (hors rollback).
#[derive(Component)]
pub struct VisualsAttached;

pub fn attach_character_visuals(
    mut commands: Commands,
    global_assets: Res<GlobalAsset>,
    character_configs: Res<CharacterConfigs>,
    spritesheet_assets: Res<Assets<SpriteSheetConfig>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    characters: Query<(Entity, &CharacterAppearance), Without<VisualsAttached>>,
) {
    for (entity, appearance) in characters.iter() {
        let Some(config) = character_configs.get(&appearance.config_name) else {
            continue;
        };
        let (Some(map_layers), Some(animation_handle)) = (
            global_assets.spritesheets.get(&config.asset_name_ref),
            global_assets.animations.get(&config.asset_name_ref),
        ) else {
            continue;
        };
        let Some(skin) = config
            .skins
            .get(appearance.skin.as_deref().unwrap_or(&config.starting_skin))
        else {
            continue;
        };

        // `try_insert` (pas `insert`) : l'entité a pu mourir et être détruite
        // entre cette requête et l'application différée des commandes (plusieurs
        // frames de simulation par tick de rendu en capture) — `insert` y
        // paniquait (« Entity despawned », captures `clone_solo`/`avant_poste`).
        commands.entity(entity).try_insert((
            AnimationVisualsBundle::new(map_layers.clone(), animation_handle.clone(), 0),
            VisualsAttached,
            // `create_character` (simulation) ne pose plus `Visibility` (D69 E6/E4) : la présentation la pose
            // avec les sprites enfants.
            Visibility::default(),
        ));

        // Barre de vie au-dessus de la tête : le haut du plus grand calque (hors ombre), pas un
        // offset fixe — un sprite plus grand que les 32 px historiques (postapo : 17 px × échelle 2)
        // se faisait traverser la tête par la barre à +10.
        let mut sprite_top = 10.0f32;
        // Un skin sans calque (ennemis de `gungeon`, lot C) montre tous les calques de l'entrée de la
        // table `SpriteSheet` : déclarer des calques dans le RON ajouterait `ActiveLayers` à l'état
        // rollback et déplacerait les traces ; la présentation les lit donc dans la table.
        let layer_names: Vec<&String> = if skin.layers.is_empty() {
            let mut names: Vec<&String> = map_layers.keys().collect();
            names.sort();
            names
        } else {
            skin.layers.keys().collect()
        };
        for layer in layer_names {
            let Some(spritesheet_config) = map_layers
                .get(layer)
                .and_then(|handle| spritesheet_assets.get(handle))
            else {
                continue;
            };
            if spritesheet_config.name != "shadow" {
                sprite_top = sprite_top.max(
                    spritesheet_config.offset_y
                        + spritesheet_config.tile_size.1 as f32 * spritesheet_config.scale / 2.0,
                );
            }
            let layer_entity = create_child_sprite(
                &mut commands,
                &asset_server,
                &mut texture_atlas_layouts,
                entity,
                spritesheet_config,
                0,
            );
            // D66 : échelle du marqueur (ombre comprise, le personnage grossit d'un bloc).
            if let Some(scale) = config.marker.as_ref().and_then(|m| m.scale) {
                commands.entity(layer_entity).insert(Transform {
                    translation: Vec3::new(
                        spritesheet_config.offset_x,
                        spritesheet_config.offset_y,
                        spritesheet_config.offset_z,
                    ),
                    scale: Vec3::splat(spritesheet_config.scale * scale),
                    ..default()
                });
            }
        }
        if let Some((r, g, b)) = config.marker.as_ref().and_then(|m| m.tint) {
            commands
                .entity(entity)
                .try_insert(MarkerTint(Color::srgb(r, g, b)));
        }

        // Même course que ci-dessus : attache silencieuse si le personnage est
        // déjà détruit à l'application différée (orphelin détruit, pas de panique).
        let health_bar = commands
            .spawn((
                HealthBar,
                Sprite {
                    color: appearance.health_bar_color,
                    custom_size: Some(Vec2::new(30.0, 3.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, sprite_top + 4.0, 0.1)),
            ))
            .id();
        commands.queue_silenced(move |world: &mut World| {
            if let Ok(mut parent) = world.get_entity_mut(entity) {
                parent.add_child(health_bar);
            } else if let Ok(mut orphan) = world.get_entity_mut(health_bar) {
                orphan.despawn();
            }
        });
    }
}

/// Fantôme de mort (présentation seule) : un ennemi qui disparaît laisse un sprite
/// qui joue son animation `Dying_*` puis s'estompe. Nécessaire parce que la
/// simulation détruit l'ennemi le tick de sa mort (`despawn_rollback`, immédiat en
/// partie locale) : sans fantôme, aucune animation de mort n'est jamais visible
/// (constaté en partie réelle le 2026-10-16). Aucun état rollback : le fantôme naît
/// et meurt dans `Update`, dérivé de l'état comme les autres visuels.
#[derive(Component)]
pub struct DeathGhost {
    /// `GgrsNetId` du mort : un rollback qui annule la mort le fait revivre, le
    /// fantôme doit alors disparaître (`remove_revived_death_ghosts`).
    net_id: usize,
    start: usize,
    end: usize,
    age: u32,
}

/// Supprime le fantôme dont l'ennemi est de nouveau vivant (mort prédite annulée
/// par un rollback) : l'ennemi est revenu dans la query, hors `RollbackDespawned`.
pub fn remove_revived_death_ghosts(
    mut commands: Commands,
    ghosts: Query<(Entity, &DeathGhost)>,
    enemies: Query<&GgrsNetId, With<combat::actors::Enemy>>,
) {
    if ghosts.is_empty() {
        return;
    }
    let alive: std::collections::BTreeSet<usize> = enemies.iter().map(|id| id.0).collect();
    for (entity, ghost) in &ghosts {
        if alive.contains(&ghost.net_id) {
            commands.entity(entity).despawn();
        }
    }
}

/// Ticks de rendu par image de l'animation de mort (~90 ms à 60 ticks/s, le
/// `frame_duration` des feuilles actuelles).
const DEATH_GHOST_TICKS_PER_FRAME: u32 = 6;

type EnemySnapshot = BTreeMap<usize, (String, Option<String>, Vec2)>;

#[allow(clippy::too_many_arguments)]
pub fn spawn_death_ghosts(
    mut commands: Commands,
    global_assets: Res<GlobalAsset>,
    character_configs: Res<CharacterConfigs>,
    spritesheet_assets: Res<Assets<SpriteSheetConfig>>,
    animation_assets: Res<Assets<AnimationMapConfig>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    frame: Res<FrameCount>,
    enemies: Query<
        (
            &GgrsNetId,
            &CharacterAppearance,
            &bevy_fixed::fixed_math::FixedTransform3D,
        ),
        With<combat::actors::Enemy>,
    >,
    mut previous: Local<EnemySnapshot>,
    ghosts: Query<&DeathGhost>,
    mut last_frame: Local<Option<u32>>,
) {
    // Nouvelle partie : les disparitions sont des déchargements, pas des morts.
    if last_frame.is_some_and(|last| frame.frame < last) {
        previous.clear();
    }
    *last_frame = Some(frame.frame);
    let mut current: EnemySnapshot = BTreeMap::new();
    for (net_id, appearance, transform) in enemies.iter() {
        current.insert(
            net_id.0,
            (
                appearance.config_name.clone(),
                appearance.skin.clone(),
                Vec2::new(
                    transform.translation.x.to_num::<f32>(),
                    transform.translation.y.to_num::<f32>(),
                ),
            ),
        );
    }
    let ghosted: std::collections::BTreeSet<usize> = ghosts.iter().map(|g| g.net_id).collect();
    for (net_id, (config_name, _skin, position)) in previous.iter() {
        if current.contains_key(net_id) || ghosted.contains(net_id) {
            continue;
        }
        let Some(config) = character_configs.get(config_name) else {
            continue;
        };
        let (Some(map_layers), Some(animation_handle)) = (
            global_assets.spritesheets.get(&config.asset_name_ref),
            global_assets.animations.get(&config.asset_name_ref),
        ) else {
            continue;
        };
        let Some(sheet) = map_layers
            .get("body")
            .and_then(|handle| spritesheet_assets.get(handle))
        else {
            continue;
        };
        let Some(animation) = animation_assets.get(animation_handle) else {
            continue;
        };
        let Some(indices) = animation.animations.get("Dying_1").or_else(|| {
            animation
                .animations
                .iter()
                .find(|(name, _)| name.starts_with("Dying"))
                .map(|(_, indices)| indices)
        }) else {
            continue;
        };
        let (start, end) = indices.to_absolute(sheet.columns);
        let layout = TextureAtlasLayout::from_grid(
            UVec2::new(sheet.tile_size.0, sheet.tile_size.1),
            sheet.columns,
            sheet.rows,
            None,
            None,
        );
        commands.spawn((
            Sprite {
                image: asset_server.load(&sheet.path),
                texture_atlas: Some(TextureAtlas {
                    layout: texture_atlas_layouts.add(layout),
                    index: start,
                }),
                ..default()
            },
            sheet.anchor.to_anchor(),
            // z au-dessus du sol et des tuiles (le `offset_z` de la feuille est
            // relatif au parent d'un personnage vivant ; en absolu, à 1, le corps
            // passait sous le sol et ne se voyait pas).
            Transform::from_translation(Vec3::new(position.x, position.y, 10.0))
                .with_scale(Vec3::splat(sheet.scale)),
            DeathGhost {
                net_id: *net_id,
                start,
                end,
                age: 0,
            },
        ));
    }
    *previous = current;
}

/// Ticks de maintien du corps après l'animation de mort, avant le fondu : sans
/// eux le fantôme dure ~0,2 s (2 images), un clignement invisible en partie
/// (retour de William, 2026-10-16). Total ≈ animation + 0,5 s + fondu 0,25 s.
const DEATH_GHOST_HOLD_TICKS: u32 = 30;
const DEATH_GHOST_FADE_TICKS: u32 = 15;

/// Avance les fantômes de mort : images `Dying_*` à cadence fixe, corps maintenu
/// sur la dernière image, puis fondu. Compté en ticks de rendu, jamais en
/// frames de simulation.
pub fn animate_death_ghosts(
    mut commands: Commands,
    mut ghosts: Query<(Entity, &mut DeathGhost, &mut Sprite)>,
) {
    for (entity, mut ghost, mut sprite) in ghosts.iter_mut() {
        ghost.age += 1;
        let frames = ghost.end - ghost.start + 1;
        let anim_ticks = frames as u32 * DEATH_GHOST_TICKS_PER_FRAME;
        let total = anim_ticks + DEATH_GHOST_HOLD_TICKS + DEATH_GHOST_FADE_TICKS;
        let index =
            ghost.start + ((ghost.age / DEATH_GHOST_TICKS_PER_FRAME) as usize).min(frames - 1);
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = index;
        }
        let fade_start = anim_ticks + DEATH_GHOST_HOLD_TICKS;
        if ghost.age > fade_start {
            let alpha = 1.0 - (ghost.age - fade_start) as f32 / DEATH_GHOST_FADE_TICKS as f32;
            sprite.color = Color::srgba(1.0, 1.0, 1.0, alpha.clamp(0.0, 1.0));
        }
        if ghost.age >= total {
            commands.entity(entity).despawn();
        }
    }
}

/// D66 (présentation) : un ennemi **stationnaire** regarde sa cible. `FacingDirection` (rollback)
/// ne suit que la vitesse, nulle pour une tourelle : le sprite gardait son orientation de
/// création. Direction : visée figée de l'émetteur pendant un pattern, sinon cible de
/// déplacement (`EnemyPath`) ; ne retourne que le `flip_x` des calques (comme `set_sprite_flip`,
/// dont ce système passe après). Ne touche ni l'état ni les traces.
pub fn face_stationary_enemies(
    enemies: Query<
        (
            &bevy_fixed::fixed_math::FixedTransform3D,
            &crate::character::enemy::ai::state::EnemyAiConfig,
            Option<&combat::emitter::Emitter>,
            Option<&crate::character::enemy::ai::pathing::EnemyPath>,
            &Children,
        ),
        With<combat::actors::Enemy>,
    >,
    // Un calque à `directions` (X-4a) choisit sa ligne par `FacingDirection` : jamais retourné ici (D153).
    mut sprites: Query<&mut Sprite, Without<animation::DirectionRow>>,
) {
    for (transform, ai, emitter, path, children) in &enemies {
        if !ai.stationary {
            continue;
        }
        let dx = match (emitter, path) {
            (Some(emitter), _) if emitter.aim != bevy_fixed::fixed_math::FixedVec2::ZERO => {
                emitter.aim.x.to_num::<f32>()
            }
            (_, Some(path)) => (path.target_position.x - transform.translation.x).to_num::<f32>(),
            _ => continue,
        };
        if dx.abs() < 1e-3 {
            continue;
        }
        for child in children.iter() {
            if let Ok(mut sprite) = sprites.get_mut(child) {
                sprite.flip_x = dx < 0.0;
            }
        }
    }
}

/// M3-T17 (§82), présentation : orientation des familiers. Un **suiveur** regarde du côté où va son
/// porteur (sinon vers lui) ; un **orbital** tourne sur lui-même, la pointe de son sprite vers
/// l'extérieur du cercle. Ne retourne/tourne que les calques à image (pas la barre de vie) ; passe
/// après `set_sprite_flip`. Ne lit que des positions et ne touche ni l'état ni les traces.
pub fn orient_familiars(
    familiars: Query<(
        &behaviors::FamiliarOf,
        &crate::familiars::FamiliarBehaviors,
        &bevy_fixed::fixed_math::FixedTransform3D,
        &Children,
    )>,
    owners: Query<
        (
            &utils::net_id::GgrsNetId,
            &bevy_fixed::fixed_math::FixedTransform3D,
        ),
        Without<behaviors::FamiliarOf>,
    >,
    mut sprites: Query<(
        &mut Sprite,
        &mut Transform,
        Option<&animation::DirectionRow>,
    )>,
    mut owner_x: Local<std::collections::BTreeMap<usize, f32>>,
) {
    let positions: std::collections::BTreeMap<usize, Vec2> = owners
        .iter()
        .map(|(net_id, t)| {
            (
                net_id.0,
                Vec2::new(
                    t.translation.x.to_num::<f32>(),
                    t.translation.y.to_num::<f32>(),
                ),
            )
        })
        .collect();
    let previous = std::mem::take(&mut *owner_x);
    for (net_id, position) in &positions {
        owner_x.insert(*net_id, position.x);
    }
    for (of, behaviors, transform, children) in &familiars {
        let Some(owner) = positions.get(&of.owner.0) else {
            continue;
        };
        let at = Vec2::new(
            transform.translation.x.to_num::<f32>(),
            transform.translation.y.to_num::<f32>(),
        );
        let motion = behaviors.motion();
        let angle = match motion {
            Some(crate::familiars::FamiliarMotion::Orbit { .. }) => {
                Some(crate::feedback_v3::outward_angle(*owner, at))
            }
            _ => None,
        };
        let faces_left = match motion {
            Some(crate::familiars::FamiliarMotion::Follow { .. }) => {
                let moved = previous.get(&of.owner.0).map_or(0.0, |x| owner.x - x);
                crate::feedback_v3::follower_faces_left(moved, owner.x - at.x)
            }
            _ => None,
        };
        for child in children.iter() {
            let Ok((mut sprite, mut layer, directional)) = sprites.get_mut(child) else {
                continue;
            };
            if sprite.texture_atlas.is_none() {
                continue;
            }
            if let (Some(left), None) = (faces_left, directional) {
                sprite.flip_x = left;
            }
            if let Some(angle) = angle {
                layer.rotation = Quat::from_rotation_z(angle);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ghost(net_id: usize) -> DeathGhost {
        DeathGhost {
            net_id,
            start: 0,
            end: 1,
            age: 0,
        }
    }

    /// Mort prédite annulée par un rollback : l ennemi revient, son fantôme part ;
    /// le fantôme d un autre ennemi mort reste.
    #[test]
    fn revived_enemy_removes_only_its_ghost() {
        let mut app = App::new();
        app.add_systems(Update, remove_revived_death_ghosts);
        let g1 = app.world_mut().spawn(ghost(1)).id();
        let g2 = app.world_mut().spawn(ghost(2)).id();
        app.update();
        assert!(app.world().get_entity(g1).is_ok(), "ennemi toujours mort");
        app.world_mut()
            .spawn((combat::actors::Enemy {}, GgrsNetId(1, String::new())));
        app.update();
        assert!(
            app.world().get_entity(g1).is_err(),
            "ennemi revenu : fantôme supprimé"
        );
        assert!(app.world().get_entity(g2).is_ok(), "autre fantôme intact");
    }
}

#[cfg(test)]
mod attack_pose_tests {
    use super::*;
    use crate::character::enemy::ai::state::{AttackTarget, MonsterState};
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn un_coup_reste_en_cours_la_duree_de_la_pose() {
        assert!(struck_recently(100, 100));
        assert!(struck_recently(100 + ATTACK_POSE_FRAMES, 100));
        assert!(!struck_recently(101 + ATTACK_POSE_FRAMES, 100));
        // Un coup « dans le futur » (rollback vers une frame plus ancienne) ne pose rien.
        assert!(!struck_recently(99, 100));
    }

    fn world_at(frame: u32) -> World {
        let mut world = World::new();
        world.insert_resource(FrameCount { frame });
        world
    }

    fn attacking(last: u32) -> MonsterState {
        MonsterState::Attacking {
            target: AttackTarget::Obstacle {
                net_id: GgrsNetId(7, "window".into()),
            },
            last_attack_frame: last,
        }
    }

    /// La pose suit l'état d'attaque et n'écrit rien d'autre : un zombie qui frappe propose `Attack`, un
    /// zombie qui poursuit ou dont le dernier coup est ancien repasse à `None` (donc `Run`).
    #[test]
    fn la_pose_d_attaque_derive_de_l_etat_d_attaque() {
        let mut world = world_at(200);
        let striking = world
            .spawn((attacking(180), animation::PresentationClip::default()))
            .id();
        let stale = world
            .spawn((attacking(100), animation::PresentationClip::default()))
            .id();
        let chasing = world
            .spawn((
                MonsterState::Chasing,
                animation::PresentationClip::default(),
            ))
            .id();
        world.run_system_once(present_attacking).unwrap();
        let clip = |world: &World, e| {
            world
                .get::<animation::PresentationClip>(e)
                .unwrap()
                .0
                .clone()
        };
        assert_eq!(clip(&world, striking).as_deref(), Some("Attack"));
        assert_eq!(clip(&world, stale), None);
        assert_eq!(clip(&world, chasing), None);
        // Le coup suivant à la frame courante : toujours en pose ; le temps passe : la pose retombe.
        world.insert_resource(FrameCount {
            frame: 180 + ATTACK_POSE_FRAMES + 1,
        });
        world.run_system_once(present_attacking).unwrap();
        assert_eq!(clip(&world, striking), None);
    }
}

/// Frames de simulation pendant lesquelles un tir reste « en cours » pour la pose de présentation
/// (un tiers de seconde à 60 Hz : couvre la cadence des armes courantes sans coller après le tir).
pub const SHOOT_POSE_FRAMES: u32 = 20;

/// Vrai si un tir a eu lieu depuis moins de [`SHOOT_POSE_FRAMES`] frames (`last_fire_frame` à 0 :
/// aucun tir encore). Pure, pour les tests.
pub fn shot_recently(frame: u32, last_fire_frame: u32) -> bool {
    last_fire_frame > 0 && frame >= last_fire_frame && frame - last_fire_frame <= SHOOT_POSE_FRAMES
}

/// Pose de tir (X-4b, présentation, `Update`) : quand l'arme active du personnage vient de tirer,
/// la clé `"Shoot"` est proposée à l'animation, qui l'applique **si** sa carte la définit (sinon
/// rien ne change). Lit l'état rollback existant de l'arme (`WeaponState::last_fire_frame`) et la
/// frame courante ; jamais l'input, rien n'est écrit dans la simulation ni dans `AnimationState`.
/// Les ennemis (`MonsterState`, qui portent aussi un `WeaponInventory`) sont exclus : leur pose est
/// celle de [`present_attacking`] (S1-8), sinon les deux systèmes se disputeraient le même composant.
pub fn present_shooting(
    frame: Res<FrameCount>,
    mut shooters: Query<
        (&combat::weapons::WeaponInventory, &mut PresentationClip),
        Without<crate::character::enemy::ai::state::MonsterState>,
    >,
    weapons: Query<&combat::weapons::WeaponState>,
) {
    for (inventory, mut clip) in shooters.iter_mut() {
        let shooting = inventory
            .weapons
            .get(inventory.active_weapon_index)
            .and_then(|(entity, _)| weapons.get(*entity).ok())
            .is_some_and(|state| shot_recently(frame.frame, state.last_fire_frame));
        let wanted = shooting.then(|| "Shoot".to_string());
        if clip.0 != wanted {
            clip.0 = wanted;
        }
    }
}

#[cfg(test)]
mod shoot_pose_tests {
    use super::*;
    use combat::weapons::{Weapon, WeaponInventory, WeaponState};

    /// Une arme du testbed (`weapons.ron`), pour remplir un `WeaponInventory`.
    fn some_weapon() -> Weapon {
        #[derive(serde::Deserialize)]
        struct Table(BTreeMap<String, crate::weapons::WeaponAsset>);
        let table = ron::from_str::<Table>(include_str!(
            "../../../../games/testbed/assets/ZombieShooter/Sprites/Character/weapons.ron"
        ))
        .unwrap()
        .0;
        Weapon::from(table.get("pistol").unwrap().clone())
    }

    #[test]
    fn a_shot_keeps_the_pose_for_a_short_window_only() {
        assert!(!shot_recently(100, 0), "jamais tiré");
        assert!(shot_recently(100, 100));
        assert!(shot_recently(100 + SHOOT_POSE_FRAMES, 100));
        assert!(!shot_recently(101 + SHOOT_POSE_FRAMES, 100));
        assert!(!shot_recently(50, 100), "rollback avant le tir");
    }

    #[test]
    fn the_active_weapon_state_drives_the_presented_key() {
        let mut app = App::new();
        app.insert_resource(FrameCount { frame: 120 });
        app.add_systems(Update, present_shooting);
        let weapon = app
            .world_mut()
            .spawn(WeaponState {
                last_fire_frame: 115,
                ..default()
            })
            .id();
        let shooter = app
            .world_mut()
            .spawn((
                WeaponInventory {
                    weapons: vec![(weapon, some_weapon())],
                    ..default()
                },
                PresentationClip::default(),
            ))
            .id();
        app.update();
        assert_eq!(
            app.world()
                .get::<PresentationClip>(shooter)
                .unwrap()
                .0
                .as_deref(),
            Some("Shoot")
        );
        app.world_mut().resource_mut::<FrameCount>().frame = 200;
        app.update();
        assert_eq!(
            app.world().get::<PresentationClip>(shooter).unwrap().0,
            None
        );
    }
}
