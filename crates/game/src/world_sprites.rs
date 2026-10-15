//! Sprites des objets du monde et des projectiles, dérivés de l'état courant, exclusivement dans la
//! présentation (lot C de `docs/plan-assets.md`).
//!
//! Une entité du monde (coffre, présentoir, douille, objet au sol, tonneau, table) n'a pas de
//! sprite à sa création : ce système lui en pose un **enfant** non rollback, choisi dans la table
//! `SpriteSheet` du jeu par un id fixe (`chest`, `shelf`, `shell`, `item`, `barrel`, `table`). Un jeu
//! sans l'entrée garde le rendu d'avant. L'image est la case `Idle` de l'animation de l'entrée ;
//! un coffre ouvert prend la case `Open` ; un présentoir vendu est caché. Les projectiles (`Bullet`)
//! gardent leur `Sprite` de couleur, remplacé par l'image `bullet_player` (balles des joueurs et
//! alliés) ou `bullet_enemy` quand l'entrée existe, teintée par l'équipe.
//!
//! Rien ici ne lit ni n'écrit la simulation : l'enfant est détruit avec son parent, une entité
//! ressuscitée par un rollback le reçoit de nouveau.
use animation::{create_child_sprite, AnimationMapConfig, SpriteSheetConfig};
use bevy::prelude::*;
use combat::weapons::Bullet;
use items::{Chest, ItemPickup, Shelf, ShellPickup};
use sim_core::team::Team;

use crate::global_asset::GlobalAsset;

pub struct WorldSpritePlugin;

impl Plugin for WorldSpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (attach_world_sprites, sync_world_sprites, skin_bullets),
        );
    }
}

/// Marque une entité dont le sprite du monde est posé ; `child` est son sprite.
#[derive(Component)]
pub struct WorldSprite {
    id: &'static str,
    child: Entity,
}

/// Marque un projectile dont l'image est posée.
#[derive(Component)]
pub struct BulletSkinned;

/// Case d'une animation de l'entrée `id` (`Idle` ou `Open`), en indice absolu de la feuille.
fn frame_of(
    global: &GlobalAsset,
    anims: &Assets<AnimationMapConfig>,
    sheet: &SpriteSheetConfig,
    id: &str,
    name: &str,
) -> Option<usize> {
    let animation = anims.get(global.animations.get(id)?)?;
    let (start, _) = animation.animations.get(name)?.to_absolute(sheet.columns);
    Some(start)
}

fn sheet_of<'a>(
    global: &GlobalAsset,
    sheets: &'a Assets<SpriteSheetConfig>,
    id: &str,
) -> Option<&'a SpriteSheetConfig> {
    sheets.get(global.spritesheets.get(id)?.get("body")?)
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn attach_world_sprites(
    mut commands: Commands,
    global: Option<Res<GlobalAsset>>,
    sheets: Res<Assets<SpriteSheetConfig>>,
    anims: Res<Assets<AnimationMapConfig>>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    entities: Query<
        (
            Entity,
            Option<&Chest>,
            Option<&Shelf>,
            Option<&ShellPickup>,
            Option<&ItemPickup>,
            Option<&world::BarrelState>,
            Option<&world::TableState>,
        ),
        (
            Without<WorldSprite>,
            Or<(
                With<Chest>,
                With<Shelf>,
                With<ShellPickup>,
                With<ItemPickup>,
                With<world::BarrelState>,
                With<world::TableState>,
            )>,
        ),
    >,
) {
    let Some(global) = global else {
        return;
    };
    for (entity, chest, shelf, shell, item, barrel, table) in &entities {
        let id: &'static str = if chest.is_some() {
            "chest"
        } else if shelf.is_some() {
            "shelf"
        } else if shell.is_some() {
            "shell"
        } else if item.is_some() {
            "item"
        } else if barrel.is_some() {
            "barrel"
        } else if table.is_some() {
            "table"
        } else {
            continue;
        };
        let Some(sheet) = sheet_of(&global, &sheets, id) else {
            continue;
        };
        let Some(frame) = frame_of(&global, &anims, sheet, id, "Idle") else {
            continue;
        };
        let child = create_child_sprite(
            &mut commands,
            &asset_server,
            &mut layouts,
            entity,
            sheet,
            frame,
        );
        commands.entity(entity).insert(WorldSprite { id, child });
    }
}

/// Coffre ouvert : case `Open` ; présentoir vendu : caché.
fn sync_world_sprites(
    global: Option<Res<GlobalAsset>>,
    sheets: Res<Assets<SpriteSheetConfig>>,
    anims: Res<Assets<AnimationMapConfig>>,
    parents: Query<(&WorldSprite, Option<&Chest>, Option<&Shelf>)>,
    mut sprites: Query<(&mut Sprite, &mut Visibility)>,
) {
    let Some(global) = global else {
        return;
    };
    for (world_sprite, chest, shelf) in &parents {
        let Ok((mut sprite, mut visibility)) = sprites.get_mut(world_sprite.child) else {
            continue;
        };
        if let Some(chest) = chest {
            let name = if chest.opened { "Open" } else { "Idle" };
            if let Some(sheet) = sheet_of(&global, &sheets, world_sprite.id) {
                if let (Some(frame), Some(atlas)) = (
                    frame_of(&global, &anims, sheet, world_sprite.id, name),
                    sprite.texture_atlas.as_mut(),
                ) {
                    atlas.index = frame;
                }
            }
        }
        if let Some(shelf) = shelf {
            *visibility = if shelf.sold {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
        }
    }
}

/// Remplace le carré de couleur d'un projectile par l'image de son équipe.
fn skin_bullets(
    mut commands: Commands,
    global: Option<Res<GlobalAsset>>,
    sheets: Res<Assets<SpriteSheetConfig>>,
    asset_server: Res<AssetServer>,
    mut bullets: Query<(Entity, &Bullet, &mut Sprite), Without<BulletSkinned>>,
) {
    let Some(global) = global else {
        return;
    };
    for (entity, bullet, mut sprite) in &mut bullets {
        let (id, color) = match bullet.source_team {
            Team::Enemies => ("bullet_enemy", Color::srgb(1.0, 0.35, 0.3)),
            _ => ("bullet_player", Color::srgb(1.0, 0.9, 0.4)),
        };
        let Some(sheet) = sheet_of(&global, &sheets, id) else {
            // Pas d'entrée (autre jeu) : le carré d'origine reste, on ne retente pas.
            commands.entity(entity).insert(BulletSkinned);
            continue;
        };
        sprite.image = asset_server.load(&sheet.path);
        sprite.color = color;
        if let Some(size) = sprite.custom_size {
            // Le halo de l'image déborde du disque : on l'agrandit pour garder un cœur de la taille de la balle.
            sprite.custom_size = Some(size * 2.0);
        }
        commands.entity(entity).insert(BulletSkinned);
    }
}
