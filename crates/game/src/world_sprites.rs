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
use map::game::entity::map::door::DoorComponent;
use map::game::entity::MapRollbackItem;
use sim_core::team::Team;

use crate::collider::Collider;
use crate::global_asset::GlobalAsset;
use crate::interaction::Interactable;

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
                attach_door_sprites,
                sync_door_sprites.after(attach_door_sprites),
                hide_replaced_map_tiles,
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

/// Demande explicite d'un sprite du monde par id (`sprites.ron`) : posé sur une entité de la carte
/// qui n'a pas de composant de jeu reconnaissable (le portail). L'image propre de l'entité (carré
/// de couleur de repli) est alors rendue transparente.
#[derive(Component, Clone, Debug)]
pub struct SpriteRequest(pub String);

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
        } else if rock.is_some() {
            vec!["rock".into()]
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

/// Porte dont l'image d'état est posée ; `child` est son sprite, `id` l'entrée (`door_v` : 16 x 32,
/// `door_h` : 32 x 16).
#[derive(Component)]
pub struct DoorSprite {
    id: &'static str,
    child: Entity,
}

/// Pose sur chaque porte (entité rollback d'une porte de carte) une image d'état ; l'orientation vient
/// de la taille de la tuile de l'entité LDtk d'origine (`MapRollbackItem::parent`). Sans entrée
/// `door_v`/`door_h` (autre jeu), la porte garde le rendu d'avant.
fn attach_door_sprites(
    mut commands: Commands,
    table: SpriteTable,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    doors: Query<(Entity, &MapRollbackItem), (With<DoorComponent>, Without<DoorSprite>)>,
    tiles: Query<&Sprite>,
) {
    for (entity, item) in &doors {
        let size = tiles.get(item.parent).ok().and_then(|tile| {
            tile.texture_atlas
                .as_ref()
                .and_then(|atlas| layouts.get(&atlas.layout)?.textures.get(atlas.index))
                .map(|rect| rect.size())
                .or_else(|| tile.custom_size.map(|s| s.as_uvec2()))
        });
        let id = match size {
            Some(size) if size.x > size.y => "door_h",
            _ => "door_v",
        };
        let (Some(sheet), Some(frame)) = (table.sheet(id), table.frame(id, "Idle")) else {
            continue;
        };
        // L'entité rollback de la porte n'a pas de `Transform` : l'image se pose sur l'entité LDtk.
        let child = create_child_sprite(
            &mut commands,
            &asset_server,
            &mut layouts,
            item.parent,
            sheet,
            frame,
        );
        commands.entity(entity).insert(DoorSprite { id, child });
        commands.entity(child).insert(MapHeld(entity));
    }
}

/// Cache l'image d'une table, d'un baril ou d'une porte dont l'entité rollback a disparu (explosion, table
/// brisée, changement d'étage, création annulée par un rollback : l'entité LDtk qui porte l'image survit) ;
/// la montre si un rollback la ressuscite. Règle dérivée de l'état, `Inherited` sinon : le monde caché d'un
/// étage quitté cache aussi ses images.
fn reap_held_sprites(
    owners: Query<(), Or<(With<WorldSprite>, With<DoorSprite>)>>,
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
    items: Query<&MapRollbackItem, Or<(With<DoorSprite>, With<WorldSprite>)>>,
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

/// État de la porte : sans collider = ouverte (`Open`) ; avec collider et interaction = à ouvrir
/// (clé ou achat, `Locked`) ; avec collider seul = fermée par sa salle (`Idle`).
fn sync_door_sprites(
    table: SpriteTable,
    doors: Query<(&DoorSprite, Has<Collider>, Has<Interactable>)>,
    mut sprites: Query<&mut Sprite>,
) {
    for (door, closed, interactable) in &doors {
        let name = match (closed, interactable) {
            (false, _) => "Open",
            (true, true) => "Locked",
            (true, false) => "Idle",
        };
        let Some(frame) = table.frame(door.id, name) else {
            continue;
        };
        if let Ok(mut sprite) = sprites.get_mut(door.child) {
            if let Some(atlas) = sprite.texture_atlas.as_mut() {
                if atlas.index != frame {
                    atlas.index = frame;
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

/// Le sprite d'une balle, posé en présentation (D69 : `combat` ne connaît plus `Sprite`). Une balle
/// qui apparaît (`Added<Bullet>`, y compris une balle recréée par une re-simulation) reçoit le
/// carré de couleur que `spawn_bullet` posait lui-même : noir (blanc pour une balle explosive), côté
/// `BASE_SPRITE × Size` d'un projectile composable (3,5 sinon), × `ShotSize` du tireur. Tourne juste
/// après `GgrsSchedule` (`PreUpdate`) : le sprite existe avant tout système d'`Update`, comme avant.
/// Sans `BulletLook` (entité recréée sans lui), le côté se dérive du `Projectile` et de `ShotSize` 1.
fn attach_bullet_visuals(
    mut commands: Commands,
    new: Query<(Entity, &Bullet, Option<&BulletLook>, Option<&Projectile>), Added<Bullet>>,
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
            child,
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
            child,
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
    }

    #[test]
    fn la_taille_d_un_projectile_croit_avec_le_degat_et_reste_bornee() {
        assert_eq!(bullet_display_size(0.0), 8.0);
        assert!(bullet_display_size(12.0) < bullet_display_size(30.0));
        assert!(bullet_display_size(30.0) < bullet_display_size(100.0));
        assert_eq!(bullet_display_size(1.0e6), 22.0);
        assert_eq!(bullet_display_size(-5.0), 8.0);
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
