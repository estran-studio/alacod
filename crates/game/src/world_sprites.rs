//! Sprites des objets du monde et des projectiles, dérivés de l'état courant, exclusivement dans la
//! présentation (lot C de `docs/plan-assets.md`, puis langage visuel de
//! `docs/conventions.md` §83).
//!
//! Une entité du monde (coffre, présentoir, douille, objet au sol, arme au sol, tonneau, table,
//! rocher) n'a pas de sprite à sa création : ce système lui en pose un **enfant** non rollback,
//! choisi dans la table `SpriteSheet` du jeu par une **liste d'ids du plus précis au plus
//! général** : un objet cherche `item/<id>`, puis `item/<genre>` (`passive`, `active`,
//! `consumable`), puis `item` ; une arme au sol `weapon/<id>` puis `weapon`. Un jeu sans aucune
//! des entrées garde le rendu d'avant. L'image est la case `Idle` de l'animation de l'entrée ;
//! un coffre ouvert prend la case `Open` ; un présentoir vendu est caché ; l'icône d'une arme au
//! sol remplace le petit sprite d'arme (caché). Les projectiles (`Bullet`) gardent leur `Sprite`
//! de couleur, remplacé par l'image `bullet_player` (balles des joueurs et alliés) ou
//! `bullet_enemy` quand l'entrée existe ; la taille suit le dégât.
//!
//! Un power-up au sol cherche `powerup/<id>` puis `powerup` (S1-8), une machine à perk `perk_machine/<perk>` puis
//! `perk_machine` (la tuile de l'entité LDtk `SodaLocation` devient transparente : `hide_soda_tiles`) ; une fenêtre porte une image d'état
//! (`window_h` 16 x 32, `window_v` 32 x 16 ; cases `Idle`, `Damaged`, `Critical`, `Broken` d'après sa
//! santé), comme une porte (`door_v`, `door_h`).
//!
//! Rien ici ne lit ni n'écrit la simulation : l'enfant est détruit avec son parent, une entité
//! ressuscitée par un rollback le reçoit de nouveau.
use animation::{create_child_sprite, AnimationMapConfig, SpriteSheetConfig};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_fixed::fixed_math;
use combat::projectile::{Projectile, BASE_SPRITE};
use combat::status::StatusLibrary;
use combat::weapons::{Bullet, BulletLook, BulletType, WeaponPickup};
use items::{Chest, ItemKind, ItemPickup, ItemTable, Shelf, ShellPickup};
use map::game::entity::map::door::{DoorComponent, DoorGridPosition};
use map::game::entity::map::soda_location::SodaLocationComponent;
use map::game::entity::map::window::WindowHealth;
use map::game::entity::MapRollbackItem;
use sim_core::team::Team;

use crate::collider::Collider;
use crate::economy::PerkMachine;
use crate::global_asset::GlobalAsset;
use crate::interaction::Interactable;
use crate::powerups::PowerUpPickup;
use crate::ui::hud::{HudConfig, HudConfigHandle};
use crate::visual_requests::{SecretDoor, SpriteRequest};

pub struct WorldSpritePlugin;

impl Plugin for WorldSpritePlugin {
    fn build(&self, app: &mut App) {
        // Le sprite d'une balle naît juste après la simulation (voir `attach_bullet_visuals`).
        app.add_systems(
            PreUpdate,
            attach_bullet_visuals.after(bevy_ggrs::RunGgrsSystems),
        );
        app.add_systems(
            Update,
            (
                attach_world_sprites,
                sync_world_sprites,
                skin_bullets,
                pair_doors,
                attach_door_sprites.after(pair_doors),
                mirror_door_pairs
                    .after(attach_door_sprites)
                    .after(pair_doors),
                sync_door_sprites.after(attach_door_sprites),
                attach_window_sprites,
                sync_window_sprites.after(attach_window_sprites),
                hide_replaced_map_tiles,
                hide_soda_tiles,
                reap_held_sprites,
                // M3-T17 : après `skin_bullets` (qui pose l'image et la taille de l'équipe).
                style_tears.after(skin_bullets),
                hide_despawned_world_sprites,
            ),
        );
    }
}

/// Marque une entité dont le sprite du monde est posé ; `child` est son sprite.
#[derive(Component)]
pub struct WorldSprite {
    id: String,
    child: Entity,
}

/// Sprite posé sur l'entité LDtk d'origine d'une entité rollback de carte (table, baril) : `owner` est
/// l'entité rollback. Quand elle disparaît (destruction), l'image est cachée (`reap_held_sprites`).
#[derive(Component)]
pub struct MapHeld(pub Entity);

/// Marque un projectile dont l'image est posée.
#[derive(Component)]
pub struct BulletSkinned;

/// Accès à la table des feuilles de sprites du jeu (`sprites.ron`) : feuille et case d'un id.
#[derive(SystemParam)]
pub(crate) struct SpriteTable<'w> {
    global: Option<Res<'w, GlobalAsset>>,
    sheets: Res<'w, Assets<SpriteSheetConfig>>,
    anims: Res<'w, Assets<AnimationMapConfig>>,
}

impl SpriteTable<'_> {
    /// Feuille du calque `body` de l'entrée `id`.
    pub(crate) fn sheet(&self, id: &str) -> Option<&SpriteSheetConfig> {
        let global = self.global.as_ref()?;
        self.sheets.get(global.spritesheets.get(id)?.get("body")?)
    }

    /// Case de l'animation `name` de l'entrée `id`, en indice absolu de la feuille.
    pub(crate) fn frame(&self, id: &str, name: &str) -> Option<usize> {
        let global = self.global.as_ref()?;
        let sheet = self.sheet(id)?;
        let animation = self.anims.get(global.animations.get(id)?)?;
        let (start, _) = animation.animations.get(name)?.to_absolute(sheet.columns);
        Some(start)
    }

    /// Première entrée de `candidates` qui a une feuille et une case `name`.
    pub(crate) fn find<'a>(
        &self,
        candidates: &'a [String],
        name: &str,
    ) -> Option<(&'a str, &SpriteSheetConfig, usize)> {
        candidates.iter().find_map(|id| {
            let sheet = self.sheet(id)?;
            Some((id.as_str(), sheet, self.frame(id, name)?))
        })
    }
}

/// `Sprite` d'une case de feuille, hors hiérarchie (halo, pointeur, étiquettes d'interface).
pub(crate) fn atlas_sprite(
    asset_server: &AssetServer,
    layouts: &mut Assets<TextureAtlasLayout>,
    sheet: &SpriteSheetConfig,
    frame: usize,
) -> Sprite {
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(sheet.tile_size.0, sheet.tile_size.1),
        sheet.columns,
        sheet.rows,
        None,
        None,
    );
    Sprite {
        image: asset_server.load(&sheet.path),
        texture_atlas: Some(TextureAtlas {
            layout: layouts.add(layout),
            index: frame,
        }),
        ..default()
    }
}

/// Ids candidats d'un objet au sol : `item/<id>`, `item/<genre>`, `item`.
pub(crate) fn item_candidates(item_id: &str, items: Option<&ItemTable>) -> Vec<String> {
    let mut ids = vec![format!("item/{item_id}")];
    if let Some(def) = items.and_then(|table| table.0.get(item_id)) {
        ids.push(
            match def.kind {
                ItemKind::Passive => "item/passive",
                ItemKind::Active(_) => "item/active",
                ItemKind::Consumable => "item/consumable",
            }
            .to_string(),
        );
    }
    ids.push("item".to_string());
    ids
}

/// Ids candidats d'une arme au sol : `weapon/<id>`, `weapon`.
pub(crate) fn weapon_candidates(weapon_id: &str) -> Vec<String> {
    vec![format!("weapon/{weapon_id}"), "weapon".to_string()]
}

/// Ids candidats d'un power-up au sol : `powerup/<id>`, `powerup`.
pub(crate) fn powerup_candidates(powerup_id: &str) -> Vec<String> {
    vec![format!("powerup/{powerup_id}"), "powerup".to_string()]
}

/// Ids candidats d'une machine à perk : `perk_machine/<perk>`, `perk_machine`.
pub(crate) fn perk_machine_candidates(perk_id: &str) -> Vec<String> {
    vec![
        format!("perk_machine/{perk_id}"),
        "perk_machine".to_string(),
    ]
}

#[allow(clippy::type_complexity)]
fn attach_world_sprites(
    mut commands: Commands,
    table: SpriteTable,
    items: Option<Res<ItemTable>>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut entities: Query<
        (
            Entity,
            Option<&Chest>,
            Option<&Shelf>,
            Option<&ShellPickup>,
            Option<&ItemPickup>,
            Option<&WeaponPickup>,
            Option<&world::BarrelState>,
            Option<&world::TableState>,
            Option<&world::Bomb>,
            Option<&world::Rock>,
            Option<&SpriteRequest>,
            Option<Mut<Sprite>>,
            Option<&MapRollbackItem>,
            Option<&PowerUpPickup>,
            Option<&PerkMachine>,
        ),
        (
            Without<WorldSprite>,
            Or<(
                With<Chest>,
                With<Shelf>,
                With<ShellPickup>,
                With<ItemPickup>,
                With<WeaponPickup>,
                With<world::BarrelState>,
                With<world::TableState>,
                With<world::Bomb>,
                With<world::Rock>,
                With<SpriteRequest>,
                With<PowerUpPickup>,
                With<PerkMachine>,
            )>,
        ),
    >,
) {
    for (
        entity,
        chest,
        shelf,
        shell,
        item,
        weapon,
        barrel,
        table_prop,
        bomb,
        rock,
        request,
        own,
        map_item,
        powerup,
        perk_machine,
    ) in &mut entities
    {
        let candidates: Vec<String> = if chest.is_some() {
            vec!["chest".into()]
        } else if shelf.is_some() {
            vec!["shelf".into()]
        } else if shell.is_some() {
            vec!["shell".into()]
        } else if let Some(item) = item {
            item_candidates(&item.item_id, items.as_deref())
        } else if let Some(weapon) = weapon {
            weapon_candidates(&weapon.weapon_id)
        } else if barrel.is_some() {
            vec!["barrel".into()]
        } else if table_prop.is_some() {
            vec!["table".into()]
        } else if bomb.is_some() {
            vec!["bomb".into()]
        } else if let Some(rock) = rock {
            // m3-salles-murs-isaac : un rocher à butin cherche `rock_loot`, puis `rock` (un jeu sans `rock_loot` ne change pas).
            if rock.loot {
                vec!["rock_loot".into(), "rock".into()]
            } else {
                vec!["rock".into()]
            }
        } else if let Some(powerup) = powerup {
            powerup_candidates(&powerup.id)
        } else if let Some(machine) = perk_machine {
            perk_machine_candidates(&machine.perk_id)
        } else if let Some(request) = request {
            vec![request.0.clone()]
        } else {
            continue;
        };
        let Some((id, sheet, frame)) = table.find(&candidates, "Idle") else {
            continue;
        };
        // Une entité rollback de carte (table, baril) n'a ni `Transform` ni sprite : l'image se pose
        // sur l'entité LDtk d'origine (`MapRollbackItem::parent`), dont la tuile devient transparente.
        let holder = map_item.map_or(entity, |item| item.parent);
        let child = create_child_sprite(
            &mut commands,
            &asset_server,
            &mut layouts,
            holder,
            sheet,
            frame,
        );
        commands.entity(entity).insert(WorldSprite {
            id: id.to_string(),
            child,
        });
        if holder != entity {
            commands.entity(child).insert(MapHeld(entity));
        }
        if let (Some(_), Some(mut own)) = (request, own) {
            own.color.set_alpha(0.0);
        }
    }
}

/// Coffre ouvert : case `Open` ; présentoir vendu : caché ; arme au sol : le petit sprite d'arme
/// d'origine est caché (l'icône du monde le remplace).
#[allow(clippy::type_complexity)]
fn sync_world_sprites(
    frame: Res<utils::frame::FrameCount>,
    table: SpriteTable,
    parents: Query<(
        &WorldSprite,
        Option<&Chest>,
        Option<&Shelf>,
        Option<&WeaponPickup>,
        Option<&world::Bomb>,
        Option<&Children>,
    )>,
    mut sprites: Query<(&mut Sprite, &mut Visibility, &mut Transform)>,
    markers: Query<
        (),
        Or<(
            With<crate::legibility::Halo>,
            With<crate::legibility::AimPivot>,
        )>,
    >,
) {
    for (world_sprite, chest, shelf, weapon, bomb, children) in &parents {
        // M3-T17 : mèche d'une bombe : clignote (rouge / teinte normale) de plus en plus vite et
        // gonfle, d'après les frames restantes (`Bomb::fuse_until`).
        if let Some(bomb) = bomb {
            if let Ok((mut sprite, _, mut layer)) = sprites.get_mut(world_sprite.child) {
                let blink =
                    crate::feedback_v3::fuse_blink(bomb.fuse_until.saturating_sub(frame.frame));
                let (r, g, b) = crate::feedback_v3::fuse_color(blink);
                sprite.color = Color::srgb(r, g, b);
                let base = table.sheet(&world_sprite.id).map_or(1.0, |s| s.scale);
                layer.scale = Vec3::splat(base * blink.scale);
            }
        }
        if let Some(chest) = chest {
            if let Ok((mut sprite, _, _)) = sprites.get_mut(world_sprite.child) {
                let name = if chest.opened { "Open" } else { "Idle" };
                if let (Some(frame), Some(atlas)) = (
                    table.frame(&world_sprite.id, name),
                    sprite.texture_atlas.as_mut(),
                ) {
                    atlas.index = frame;
                }
            }
        }
        if let Some(shelf) = shelf {
            if let Ok((_, mut visibility, _)) = sprites.get_mut(world_sprite.child) {
                *visibility = if shelf.sold {
                    Visibility::Hidden
                } else {
                    Visibility::Inherited
                };
            }
        }
        if let (Some(_), Some(children)) = (weapon, children) {
            for child in children.iter() {
                if child == world_sprite.child || markers.contains(child) {
                    continue;
                }
                if let Ok((_, mut visibility, _)) = sprites.get_mut(child) {
                    if *visibility != Visibility::Hidden {
                        *visibility = Visibility::Hidden;
                    }
                }
            }
        }
    }
}

/// Porte dont l'image d'état est posée ; `children` sont ses sprites (un seul, ou une rangée pour une porte plus
/// longue que l'image avec `double_doors`), `id` l'entrée (`door_v` : 16 x 32, `door_h` : 32 x 16).
#[derive(Component)]
pub struct DoorSprite {
    id: &'static str,
    children: Vec<Entity>,
}

/// Porte double (S1-8, `double_doors` de `hud.ron`) : cette porte et sa jumelle (`DoorConfig::paired_door`, la
/// porte de la salle voisine, collée contre elle de l'autre côté du mur) forment **une** porte. La simulation en a
/// deux (deux entités, deux colliders, deux prix : celle qu'on actionne se paie et ouvre l'autre, voir
/// `interaction::handle_door_interaction`) ; la présentation n'en fait qu'une : les deux battants sont
/// **symétriques** (le second est retourné) et il n'y a qu'une étiquette (`first`, au `center`). Chaque battant
/// montre l'état de **sa** porte, donc l'image ne ment jamais si les deux états diffèrent. Composant de
/// présentation sur une entité rollback, hors checksum et hors snapshot (comme [`DoorSprite`]).
#[derive(Component, Clone, Copy, Debug)]
pub struct DoorPair {
    pub partner: Entity,
    /// Le battant qui porte l'étiquette : celui de plus petit `GgrsNetId`.
    pub first: bool,
    /// Milieu des deux portes (unités monde).
    pub center: Vec2,
    /// Les deux portes sont côte à côte sur l'axe x (portes verticales) : le second battant est retourné en x ;
    /// sinon sur l'axe y (portes horizontales) : retourné en y.
    pub along_x: bool,
}

/// Distance maximale entre les centres de deux portes jumelles collées (unités monde) : deux cases.
const DOOR_PAIR_MAX_DISTANCE: f32 = 40.0;

/// Ce que `pair_doors` sait d'une porte : sa case, la case qu'elle désigne (`paired_door`) et sa position.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DoorFact {
    pub level: String,
    pub grid: (i32, i32),
    pub paired: Option<(String, (i32, i32))>,
    pub pos: Vec2,
}

/// Couples `(a, b)` (`a < b`, indices dans `doors`) de portes jumelles : chacune désigne l'autre par `paired_door`
/// (niveau et case) **et** leurs centres sont à moins de [`DOOR_PAIR_MAX_DISTANCE`]. Pure, pour les tests.
pub(crate) fn door_pairs(doors: &[DoorFact]) -> Vec<(usize, usize)> {
    let points_at = |a: &DoorFact, b: &DoorFact| {
        a.paired
            .as_ref()
            .is_some_and(|(level, grid)| level == &b.level && *grid == b.grid)
    };
    let mut pairs = Vec::new();
    for (i, a) in doors.iter().enumerate() {
        for (j, b) in doors.iter().enumerate().skip(i + 1) {
            if points_at(a, b) && points_at(b, a) && a.pos.distance(b.pos) <= DOOR_PAIR_MAX_DISTANCE
            {
                pairs.push((i, j));
            }
        }
    }
    pairs
}

/// Construit les [`DoorPair`] (voir [`door_pairs`]). Une porte jumelée à une porte éloignée (cartes de test) ou
/// sans jumelle reste seule. Sans `double_doors`, rien ne change.
#[allow(clippy::type_complexity)]
fn pair_doors(
    mut commands: Commands,
    hud: Res<HudConfigHandle>,
    configs: Res<Assets<HudConfig>>,
    doors: Query<
        (
            Entity,
            &utils::net_id::GgrsNetId,
            &DoorComponent,
            &DoorGridPosition,
            &fixed_math::FixedTransform3D,
        ),
        Without<DoorPair>,
    >,
) {
    if !hud.get(&configs).is_some_and(|c| c.double_doors) {
        return;
    }
    // `level_iid` est rempli après la création (`populate_door_level_iids`) : une porte sans niveau attend.
    let all: Vec<_> = doors.iter().filter(|d| !d.3.level_iid.is_empty()).collect();
    let facts: Vec<DoorFact> = all
        .iter()
        .map(|(_, _, door, grid, t)| DoorFact {
            level: grid.level_iid.clone(),
            grid: (grid.grid_x, grid.grid_y),
            paired: door.config.paired_door.clone(),
            pos: Vec2::new(
                t.translation.x.to_num::<f32>(),
                t.translation.y.to_num::<f32>(),
            ),
        })
        .collect();
    for (a, b) in door_pairs(&facts) {
        let a_first = all[a].1 .0 < all[b].1 .0;
        let pa = facts[a].pos;
        let pb = facts[b].pos;
        let along_x = (pa.x - pb.x).abs() > (pa.y - pb.y).abs();
        let center = (pa + pb) / 2.0;
        commands.entity(all[a].0).insert(DoorPair {
            partner: all[b].0,
            first: a_first,
            center,
            along_x,
        });
        commands.entity(all[b].0).insert(DoorPair {
            partner: all[a].0,
            first: !a_first,
            center,
            along_x,
        });
    }
}

/// Pose sur chaque porte (entité rollback d'une porte de carte) une image d'état ; l'orientation vient
/// de la taille de la tuile de l'entité LDtk d'origine (`MapRollbackItem::parent`). Sans entrée
/// `door_v`/`door_h` (autre jeu), la porte garde le rendu d'avant.
///
/// Avec `double_doors` : une porte plus longue que l'image (collider de 64 pour une image de 32, la fonderie)
/// reçoit une rangée d'images contiguës (le second battant d'une [`DoorPair`] est retourné par
/// [`mirror_door_pairs`]).
#[allow(clippy::type_complexity)]
fn attach_door_sprites(
    mut commands: Commands,
    table: SpriteTable,
    hud: Res<HudConfigHandle>,
    configs: Res<Assets<HudConfig>>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    doors: Query<
        (Entity, &MapRollbackItem, Option<&Collider>),
        (With<DoorComponent>, Without<DoorSprite>),
    >,
    tiles: Query<&Sprite>,
) {
    let double_doors = hud.get(&configs).is_some_and(|c| c.double_doors);
    for (entity, item, collider) in &doors {
        let size = tiles
            .get(item.parent)
            .ok()
            .and_then(|tile| map_tile_size(tile, &layouts));
        let id = match size {
            Some(size) if size.x > size.y => "door_h",
            _ => "door_v",
        };
        let (Some(sheet), Some(frame)) = (table.sheet(id), table.frame(id, "Idle")) else {
            continue;
        };
        // Nombre d'images : la longueur du collider (axe long) sur celle de l'image.
        let vertical = id == "door_v";
        let cell = if vertical {
            sheet.tile_size.1 as f32 * sheet.scale
        } else {
            sheet.tile_size.0 as f32 * sheet.scale
        };
        let long = collider.and_then(|c| match &c.shape {
            crate::collider::ColliderShape::Rectangle { width, height } => Some(if vertical {
                height.to_num::<f32>()
            } else {
                width.to_num::<f32>()
            }),
            _ => None,
        });
        let count = match long {
            Some(long) if double_doors && cell > 0.0 => ((long / cell).round() as usize).max(1),
            _ => 1,
        };
        // L'entité rollback de la porte n'a pas de `Transform` : l'image se pose sur l'entité LDtk.
        let mut children = Vec::with_capacity(count);
        for k in 0..count {
            let child = create_child_sprite(
                &mut commands,
                &asset_server,
                &mut layouts,
                item.parent,
                sheet,
                frame,
            );
            let along = (k as f32 - (count as f32 - 1.0) / 2.0) * cell;
            let offset = if vertical {
                Vec3::new(0.0, along, 0.0)
            } else {
                Vec3::new(along, 0.0, 0.0)
            };
            commands.queue(move |world: &mut World| {
                if let Ok(mut child) = world.get_entity_mut(child) {
                    if let Some(mut transform) = child.get_mut::<Transform>() {
                        transform.translation += offset;
                    }
                }
            });
            commands.entity(child).insert(MapHeld(entity));
            children.push(child);
        }
        commands.entity(entity).insert(DoorSprite { id, children });
    }
}

/// Le second battant d'une [`DoorPair`] est retourné (deux battants symétriques d'une même porte). Fait une fois par
/// battant (`MirroredDoor`), dès que la paire et les images existent, quel que soit l'ordre de leur création.
fn mirror_door_pairs(
    mut commands: Commands,
    doors: Query<(Entity, &DoorPair, &DoorSprite), Without<MirroredDoor>>,
    mut sprites: Query<&mut Sprite>,
) {
    for (entity, pair, door) in &doors {
        if !pair.first {
            for child in &door.children {
                if let Ok(mut sprite) = sprites.get_mut(*child) {
                    if pair.along_x {
                        sprite.flip_x = true;
                    } else {
                        sprite.flip_y = true;
                    }
                }
            }
        }
        commands.entity(entity).insert(MirroredDoor);
    }
}

/// Le battant a été retourné s'il le fallait (`mirror_door_pairs`).
#[derive(Component)]
struct MirroredDoor;

/// Fenêtre dont l'image d'état est posée ; `child` est son sprite, `id` l'entrée (`window_h` : 16 x 32,
/// `window_v` : 32 x 16, comme les entités LDtk `WindowHorizontal` et `WindowVertical`).
#[derive(Component)]
pub struct WindowSprite {
    id: &'static str,
    child: Entity,
}

/// Taille de la tuile d'une entité LDtk de carte (case de l'atlas, sinon taille personnalisée).
fn map_tile_size(tile: &Sprite, layouts: &Assets<TextureAtlasLayout>) -> Option<UVec2> {
    tile.texture_atlas
        .as_ref()
        .and_then(|atlas| layouts.get(&atlas.layout)?.textures.get(atlas.index))
        .map(|rect| rect.size())
        .or_else(|| tile.custom_size.map(|s| s.as_uvec2()))
}

/// Pose sur chaque fenêtre une image d'état ; l'orientation vient de la taille de la tuile de l'entité LDtk
/// d'origine (plus large que haute : `window_v`). Sans entrée `window_h`/`window_v` (autre jeu, ou zombies sans
/// la surcharge locale du pack), la fenêtre garde son rendu d'avant : la tuile de la carte.
fn attach_window_sprites(
    mut commands: Commands,
    table: SpriteTable,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    windows: Query<(Entity, &MapRollbackItem), (With<WindowHealth>, Without<WindowSprite>)>,
    tiles: Query<&Sprite>,
) {
    for (entity, item) in &windows {
        let size = tiles
            .get(item.parent)
            .ok()
            .and_then(|tile| map_tile_size(tile, &layouts));
        let id = match size {
            Some(size) if size.x > size.y => "window_v",
            _ => "window_h",
        };
        let (Some(sheet), Some(frame)) = (table.sheet(id), table.frame(id, "Idle")) else {
            continue;
        };
        let child = create_child_sprite(
            &mut commands,
            &asset_server,
            &mut layouts,
            item.parent,
            sheet,
            frame,
        );
        commands.entity(entity).insert(WindowSprite { id, child });
        commands.entity(child).insert(MapHeld(entity));
    }
}

/// Nom de la case d'une fenêtre d'après sa santé : `Idle` (intacte), `Damaged`, `Critical` (une planche),
/// `Broken` (0). Une feuille sans l'une de ces cases garde la précédente (repli sur `Idle` dans [`sync_window_sprites`]).
pub(crate) fn window_state(current: u8, max: u8) -> &'static str {
    if current >= max {
        "Idle"
    } else if current == 0 {
        "Broken"
    } else if current.saturating_mul(2) > max {
        "Damaged"
    } else {
        "Critical"
    }
}

/// Teinte d'une case de fenêtre (`window_state`) : blanc intact, puis de plus en plus rouge, sombre une fois brisée.
pub(crate) fn window_tint(state: &str) -> Color {
    match state {
        "Damaged" => Color::srgb(1.0, 0.88, 0.78),
        "Critical" => Color::srgb(1.0, 0.66, 0.58),
        "Broken" => Color::srgb(0.62, 0.58, 0.68),
        _ => Color::WHITE,
    }
}

/// État visuel de la fenêtre : dérivé de `WindowHealth`, donc juste après un rollback (l'état d'une frame
/// rejouée n'est jamais mémorisé ici).
fn sync_window_sprites(
    table: SpriteTable,
    windows: Query<(&WindowSprite, &WindowHealth)>,
    mut sprites: Query<&mut Sprite>,
) {
    for (window, health) in &windows {
        let name = window_state(health.current, health.max);
        let Some(frame) = table
            .frame(window.id, name)
            .or_else(|| table.frame(window.id, "Idle"))
        else {
            continue;
        };
        if let Ok(mut sprite) = sprites.get_mut(window.child) {
            if let Some(atlas) = sprite.texture_atlas.as_mut() {
                if atlas.index != frame {
                    atlas.index = frame;
                }
            }
            // Teinte d'état : le rouge monte avec les dégâts, la fenêtre brisée s'assombrit. Un renfort de lecture à
            // l'échelle du jeu (les planches de 15 px y tiennent en un ou deux pixels), pas une autre image.
            let tint = window_tint(name);
            if sprite.color != tint {
                sprite.color = tint;
            }
        }
    }
}

/// Cache l'image d'une table, d'un baril ou d'une porte dont l'entité rollback a disparu (explosion, table
/// brisée, changement d'étage, création annulée par un rollback : l'entité LDtk qui porte l'image survit) ;
/// la montre si un rollback la ressuscite. Règle dérivée de l'état, `Inherited` sinon : le monde caché d'un
/// étage quitté cache aussi ses images.
fn reap_held_sprites(
    owners: Query<(), Or<(With<WorldSprite>, With<DoorSprite>, With<WindowSprite>)>>,
    mut held: Query<(&MapHeld, &mut Visibility)>,
) {
    for (held, mut visibility) in &mut held {
        let wanted = if owners.contains(held.0) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        visibility.set_if_neq(wanted);
    }
}

/// L'image du monde remplace la tuile de la carte : la tuile de l'entité LDtk d'origine (porte, table,
/// baril) devient transparente dès que son entité rollback porte son propre sprite.
fn hide_replaced_map_tiles(
    items: Query<&MapRollbackItem, Or<(With<DoorSprite>, With<WorldSprite>, With<WindowSprite>)>>,
    mut tiles: Query<&mut Sprite>,
) {
    for item in &items {
        if let Ok(mut tile) = tiles.get_mut(item.parent) {
            if tile.color.alpha() != 0.0 {
                tile.color.set_alpha(0.0);
            }
        }
    }
}

/// La tuile de l'entité LDtk `SodaLocation` (machine à perk) devient transparente quand la feuille du jeu a une
/// image pour sa machine (`perk_machine/<perk>` ou `perk_machine`) : l'image de la machine, posée sur l'entité
/// rollback `PerkMachine` (même position), la remplace. Sans image, la tuile de la carte reste.
fn hide_soda_tiles(table: SpriteTable, mut tiles: Query<(&SodaLocationComponent, &mut Sprite)>) {
    for (location, mut tile) in &mut tiles {
        if tile.color.alpha() != 0.0
            && table
                .find(&perk_machine_candidates(&location.perk), "Idle")
                .is_some()
        {
            tile.color.set_alpha(0.0);
        }
    }
}

/// État de la porte : sans collider = ouverte (`Open`) ; avec collider et interaction = à ouvrir
/// (clé ou achat, `Locked`) ; avec collider seul = fermée par sa salle (`Idle`), ou **mur fissuré** (`Secret`) si la
/// porte est celle d'une salle secrète (`SecretDoor`) : sans image `Secret` dans la feuille, elle reste `Idle`.
fn sync_door_sprites(
    table: SpriteTable,
    doors: Query<(
        &DoorSprite,
        Has<Collider>,
        Has<Interactable>,
        Has<SecretDoor>,
    )>,
    mut sprites: Query<&mut Sprite>,
) {
    for (door, closed, interactable, secret) in &doors {
        let name = match (closed, interactable, secret) {
            (false, _, _) => "Open",
            (true, true, _) => "Locked",
            (true, false, true) => "Secret",
            (true, false, false) => "Idle",
        };
        let Some(frame) = table
            .frame(door.id, name)
            .or_else(|| table.frame(door.id, "Idle").filter(|_| name == "Secret"))
        else {
            continue;
        };
        for child in &door.children {
            if let Ok(mut sprite) = sprites.get_mut(*child) {
                if let Some(atlas) = sprite.texture_atlas.as_mut() {
                    if atlas.index != frame {
                        atlas.index = frame;
                    }
                }
            }
        }
    }
}

/// Côté affiché d'un projectile (unités monde) : croît avec la racine du dégât, borné pour rester
/// lisible (un tir faible reste visible, un tir lourd ne masque pas la salle).
pub(crate) fn bullet_display_size(damage: f32) -> f32 {
    (5.0 + 1.6 * damage.max(0.0).sqrt()).clamp(8.0, 22.0)
}

/// Le sprite d'une balle, posé en présentation (D69 : `combat` ne connaît plus `Sprite`). Toute balle
/// vivante sans sprite en reçoit un : le carré de couleur que `spawn_bullet` posait lui-même (noir,
/// blanc pour une balle explosive ; côté `BASE_SPRITE × Size` d'un projectile composable, 3,5 sinon ;
/// × `ShotSize` du tireur). Le filtre est `Without<Sprite>` et non `Added<Bullet>` : une balle
/// créée puis détruite (`despawn_rollback`) dans un même lot de frames p2p n'est jamais vue par ce
/// système, et un rollback qui la ressuscite ne la rend pas « ajoutée » (`RollbackDespawned` retiré,
/// `Bullet` inchangé) : elle aurait vécu sans sprite. Tourne juste après `GgrsSchedule`
/// (`PreUpdate`) : le sprite existe avant tout système d'`Update`, comme avant. Sans `BulletLook`
/// (entité recréée sans lui), le côté se dérive du `Projectile` et de `ShotSize` 1.
#[allow(clippy::type_complexity)]
fn attach_bullet_visuals(
    mut commands: Commands,
    new: Query<
        (Entity, &Bullet, Option<&BulletLook>, Option<&Projectile>),
        (With<Bullet>, Without<Sprite>),
    >,
) {
    for (entity, bullet, look, projectile) in &new {
        let look = look.copied().unwrap_or(BulletLook {
            projectile_size: projectile.map(|projectile| projectile.size),
            shot_size: fixed_math::FIXED_ONE,
        });
        let color = match bullet.bullet_type {
            BulletType::Standard { .. } | BulletType::Piercing { .. } => Color::BLACK,
            BulletType::Explosive { .. } => Color::WHITE,
        };
        let side = match look.projectile_size {
            Some(size) => fixed_math::to_f32(size) * BASE_SPRITE,
            None => 3.5,
        };
        let side = if look.shot_size == fixed_math::FIXED_ONE {
            side
        } else {
            side * fixed_math::to_f32(look.shot_size)
        };
        commands
            .entity(entity)
            .try_insert(Sprite::from_color(color, Vec2::new(side, side)));
    }
}

/// Remplace le carré de couleur d'un projectile par l'image de son équipe. Feuille `own_colors: true`
/// (gungeon : orbe rouge ennemi, losange bleu allié) : l'image porte sa couleur, taille = dégât ; sinon
/// teinte d'équipe et halo agrandi, comme avant.
fn skin_bullets(
    mut commands: Commands,
    table: SpriteTable,
    asset_server: Res<AssetServer>,
    mut bullets: Query<(Entity, &Bullet, &mut Sprite), Without<BulletSkinned>>,
) {
    if table.global.is_none() {
        return;
    }
    for (entity, bullet, mut sprite) in &mut bullets {
        let id = match bullet.source_team {
            Team::Enemies => "bullet_enemy",
            _ => "bullet_player",
        };
        let Some(sheet) = table.sheet(id) else {
            // Pas d'entrée (autre jeu) : le carré d'origine reste, on ne retente pas.
            commands.entity(entity).insert(BulletSkinned);
            continue;
        };
        sprite.image = asset_server.load(&sheet.path);
        if sheet.own_colors {
            sprite.color = Color::WHITE;
            sprite.custom_size = Some(Vec2::splat(bullet_display_size(fixed_math::to_f32(
                bullet.damage,
            ))));
        } else {
            sprite.color = match bullet.source_team {
                Team::Enemies => Color::srgb(1.0, 0.35, 0.3),
                _ => Color::srgb(1.0, 0.9, 0.4),
            };
            if let Some(size) = sprite.custom_size {
                // Le halo de l'image déborde du disque : on l'agrandit pour garder un cœur de la taille de la balle.
                sprite.custom_size = Some(size * 2.0);
            }
        }
        commands.entity(entity).insert(BulletSkinned);
    }
}

/// Une bombe ou un rocher détruit par `despawn_rollback` est désactivé (`RollbackDespawned`) jusqu'à la
/// confirmation de sa frame, mais son sprite enfant, lui, reste affiché : il serait encore là quelques
/// frames après l'explosion. Cache le sprite tant que le parent est désactivé, le montre si un rollback
/// le ressuscite. Ne touche que les bombes et les rochers (un présentoir vendu cache déjà le sien).
#[allow(clippy::type_complexity)]
fn hide_despawned_world_sprites(
    gone: Query<
        &WorldSprite,
        (
            With<bevy_ggrs::RollbackDespawned>,
            Or<(With<world::Bomb>, With<world::Rock>)>,
        ),
    >,
    alive: Query<
        &WorldSprite,
        (
            Or<(With<world::Bomb>, With<world::Rock>)>,
            Without<bevy_ggrs::RollbackDespawned>,
        ),
    >,
    mut visibilities: Query<&mut Visibility>,
) {
    for world_sprite in &gone {
        if let Ok(mut visibility) = visibilities.get_mut(world_sprite.child) {
            visibility.set_if_neq(Visibility::Hidden);
        }
    }
    for world_sprite in &alive {
        if let Ok(mut visibility) = visibilities.get_mut(world_sprite.child) {
            visibility.set_if_neq(Visibility::Inherited);
        }
    }
}

/// Marque une larme dont le look (couleur, opacité, taille) est posé.
#[derive(Component)]
pub struct TearStyled;

/// M3-T17 (§82) : couleur et taille d'une larme d'après son état résolu (`Projectile` : guidage,
/// spectral, division, statut posé à l'impact, génération), une fois par larme, après le skin
/// d'équipe. Une larme sans modificateur garde le rendu d'avant. Lecture seule de la simulation.
#[allow(clippy::type_complexity)]
fn style_tears(
    mut commands: Commands,
    library: Option<Res<StatusLibrary>>,
    mut tears: Query<
        (Entity, &Bullet, &Projectile, &mut Sprite),
        (With<BulletSkinned>, Without<TearStyled>),
    >,
) {
    let empty = StatusLibrary::default();
    let library = library.as_deref().unwrap_or(&empty);
    for (entity, bullet, projectile, mut sprite) in &mut tears {
        commands.entity(entity).insert(TearStyled);
        let base = match bullet.source_team {
            Team::Enemies => crate::feedback_v3::ENEMY_BULLET_COLOR,
            _ => crate::feedback_v3::PLAYER_BULLET_COLOR,
        };
        let statuses = crate::feedback_v3::on_hit_statuses(&projectile.on_hit, library);
        let look = crate::feedback_v3::tear_look(projectile, &statuses, base);
        if look.is_plain() {
            continue;
        }
        sprite.color = Color::srgba(look.color.0, look.color.1, look.color.2, look.alpha);
        if let Some(size) = sprite.custom_size {
            sprite.custom_size = Some(size * look.scale);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn l_image_d_une_porte_se_cache_quand_la_porte_disparait() {
        let mut world = World::new();
        let holder = world.spawn(Visibility::Inherited).id();
        let door = world.spawn_empty().id();
        let child = world
            .spawn((MapHeld(door), Visibility::Inherited, ChildOf(holder)))
            .id();
        // Porte posée (marqueur `DoorSprite`) : l'image reste héritée.
        world.entity_mut(door).insert(DoorSprite {
            id: "door_v",
            children: vec![child],
        });
        world.run_system_once(reap_held_sprites).unwrap();
        assert_eq!(world.get::<Visibility>(child), Some(&Visibility::Inherited));
        // Porte retirée (changement d'étage, création annulée) : l'image se cache.
        world.entity_mut(door).remove::<DoorSprite>();
        world.run_system_once(reap_held_sprites).unwrap();
        assert_eq!(world.get::<Visibility>(child), Some(&Visibility::Hidden));
        // Un rollback la ressuscite : l'image revient.
        world.entity_mut(door).insert(DoorSprite {
            id: "door_v",
            children: vec![child],
        });
        world.run_system_once(reap_held_sprites).unwrap();
        assert_eq!(world.get::<Visibility>(child), Some(&Visibility::Inherited));
    }

    fn bullet_of(bullet_type: combat::weapons::BulletType) -> Bullet {
        Bullet {
            velocity: bevy_fixed::fixed_math::FixedVec2::ZERO,
            bullet_type,
            damage: fixed_math::Fixed::ZERO,
            range: fixed_math::Fixed::ZERO,
            distance_traveled: fixed_math::Fixed::ZERO,
            player_handle: 0,
            created_at: 0,
            source: utils::net_id::GgrsNetId(0, "player".into()),
            source_team: Team::Players,
            tags: Default::default(),
            friendly_fire: Default::default(),
        }
    }

    fn sprite_after_attach(world: &mut World, entity: Entity) -> Sprite {
        world.run_system_once(attach_bullet_visuals).unwrap();
        world.get::<Sprite>(entity).expect("sprite posé").clone()
    }

    /// D69 : `spawn_bullet` ne pose plus de `Sprite`. Couleur par type de balle, côté 3,5 pour une
    /// balle ordinaire, `Size × BASE_SPRITE` pour un projectile composable, × `ShotSize` du tireur :
    /// les valeurs exactes que le sprite avait à l'apparition (les scénarios ne tirent aucune balle
    /// explosive, ce test est leur seule preuve).
    #[test]
    fn le_sprite_d_une_balle_reprend_couleur_et_taille_de_l_apparition() {
        use combat::weapons::BulletType::{Explosive, Piercing, Standard};
        let fx = fixed_math::new;
        let mut world = World::new();
        let standard = world
            .spawn((
                bullet_of(Standard {
                    damage: fx(1.0),
                    speed: fx(1.0),
                }),
                BulletLook {
                    projectile_size: None,
                    shot_size: fixed_math::FIXED_ONE,
                },
            ))
            .id();
        let sprite = sprite_after_attach(&mut world, standard);
        assert_eq!(sprite.color, Color::BLACK);
        assert_eq!(sprite.custom_size, Some(Vec2::new(3.5, 3.5)));

        let explosive = world
            .spawn((
                bullet_of(Explosive {
                    damage: fx(1.0),
                    speed: fx(1.0),
                    blast_radius: fx(1.0),
                    explosive_damage_multiplier: fx(1.0),
                }),
                BulletLook {
                    projectile_size: None,
                    shot_size: fx(1.5),
                },
            ))
            .id();
        let sprite = sprite_after_attach(&mut world, explosive);
        assert_eq!(sprite.color, Color::WHITE);
        assert_eq!(sprite.custom_size, Some(Vec2::new(3.5 * 1.5, 3.5 * 1.5)));

        let composable = world
            .spawn((
                bullet_of(Piercing {
                    damage: fx(1.0),
                    speed: fx(1.0),
                    penetration: 1,
                }),
                BulletLook {
                    projectile_size: Some(fx(2.0)),
                    shot_size: fx(1.5),
                },
            ))
            .id();
        let sprite = sprite_after_attach(&mut world, composable);
        assert_eq!(sprite.color, Color::BLACK);
        assert_eq!(
            sprite.custom_size,
            Some(Vec2::splat(2.0 * BASE_SPRITE * 1.5))
        );

        // Un sprite posé n'est jamais remplacé ; une balle qui l'a perdu (ou ne l'a jamais reçu : lot
        // de frames p2p, balle ressuscitée par un rollback) le retrouve.
        world.get_mut::<Sprite>(standard).unwrap().color = Color::srgb(1.0, 0.0, 0.0);
        world.entity_mut(explosive).remove::<Sprite>();
        world.run_system_once(attach_bullet_visuals).unwrap();
        assert_eq!(
            world.get::<Sprite>(standard).unwrap().color,
            Color::srgb(1.0, 0.0, 0.0)
        );
        assert_eq!(world.get::<Sprite>(explosive).unwrap().color, Color::WHITE);
    }

    #[test]
    fn la_taille_d_un_projectile_croit_avec_le_degat_et_reste_bornee() {
        assert_eq!(bullet_display_size(0.0), 8.0);
        assert!(bullet_display_size(12.0) < bullet_display_size(30.0));
        assert!(bullet_display_size(30.0) < bullet_display_size(100.0));
        assert_eq!(bullet_display_size(1.0e6), 22.0);
        assert_eq!(bullet_display_size(-5.0), 8.0);
    }

    /// La case d'une fenêtre suit sa santé, de l'intacte à la brisée, pour la santé maximale du jeu (3)
    /// comme pour une autre.
    #[test]
    fn l_etat_visuel_d_une_fenetre_suit_sa_sante() {
        let states: Vec<_> = (0..=3).rev().map(|hp| window_state(hp, 3)).collect();
        assert_eq!(states, ["Idle", "Damaged", "Critical", "Broken"]);
        assert_eq!(window_state(1, 1), "Idle");
        assert_eq!(window_state(0, 1), "Broken");
        assert_eq!(window_state(3, 5), "Damaged");
        assert_eq!(window_state(2, 5), "Critical");
    }

    fn fact(
        level: &str,
        grid: (i32, i32),
        paired: (&str, (i32, i32)),
        pos: (f32, f32),
    ) -> DoorFact {
        DoorFact {
            level: level.into(),
            grid,
            paired: Some((paired.0.into(), paired.1)),
            pos: Vec2::new(pos.0, pos.1),
        }
    }

    /// Deux portes qui se désignent l'une l'autre et sont collées forment une paire ; une porte jumelée à une
    /// porte éloignée, ou qui ne désigne pas en retour, reste seule (cartes de test : jumelles à l'autre bout).
    #[test]
    fn deux_portes_collees_qui_se_designent_forment_une_porte_double() {
        let doors = [
            fact("a", (25, 5), ("b", (0, 5)), (-136.0, 160.0)),
            fact("b", (0, 5), ("a", (25, 5)), (-120.0, 160.0)),
            // Jumelles désignées mais à 300 unités l'une de l'autre.
            fact("c", (1, 1), ("d", (2, 2)), (0.0, 0.0)),
            fact("d", (2, 2), ("c", (1, 1)), (300.0, 0.0)),
            // Collées, mais la seconde désigne une autre porte.
            fact("e", (3, 3), ("f", (4, 4)), (500.0, 0.0)),
            fact("f", (4, 4), ("g", (9, 9)), (516.0, 0.0)),
        ];
        assert_eq!(door_pairs(&doors), vec![(0, 1)]);
        assert!(door_pairs(&[]).is_empty());
    }

    #[test]
    fn chaque_etat_de_fenetre_a_sa_teinte() {
        let tints: Vec<_> = ["Idle", "Damaged", "Critical", "Broken"]
            .map(|state| window_tint(state))
            .into();
        assert_eq!(tints[0], Color::WHITE);
        assert!(tints
            .iter()
            .enumerate()
            .all(|(i, a)| tints.iter().enumerate().all(|(j, b)| i == j || a != b)));
    }

    #[test]
    fn un_power_up_cherche_son_id_puis_le_generique() {
        assert_eq!(
            powerup_candidates("max_ammo"),
            ["powerup/max_ammo", "powerup"]
        );
    }

    #[test]
    fn une_machine_a_perk_cherche_son_perk_puis_le_generique() {
        assert_eq!(
            perk_machine_candidates("juggernog"),
            ["perk_machine/juggernog", "perk_machine"]
        );
    }

    #[test]
    fn un_objet_cherche_son_id_puis_son_genre_puis_le_generique() {
        let mut items = ItemTable::default();
        items.0.insert(
            "key".into(),
            ron::from_str(r#"(name: "Clé", kind: Consumable, rarity: Common)"#).unwrap(),
        );
        assert_eq!(
            item_candidates("key", Some(&items)),
            ["item/key", "item/consumable", "item"]
        );
        assert_eq!(
            item_candidates("inconnu", Some(&items)),
            ["item/inconnu", "item"]
        );
        assert_eq!(weapon_candidates("arc"), ["weapon/arc", "weapon"]);
    }
}
