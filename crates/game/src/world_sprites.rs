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
use map::game::entity::map::door::DoorComponent;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_fixed::fixed_math;
use combat::weapons::{Bullet, WeaponPickup};
use items::{Chest, ItemKind, ItemPickup, ItemTable, Shelf, ShellPickup};
use sim_core::team::Team;

use crate::collider::Collider;
use crate::global_asset::GlobalAsset;
use crate::interaction::Interactable;

pub struct WorldSpritePlugin;

impl Plugin for WorldSpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                attach_world_sprites,
                sync_world_sprites,
                skin_bullets,
                attach_door_sprites,
                sync_door_sprites.after(attach_door_sprites),
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
            Option<&world::Rock>,
            Option<&SpriteRequest>,
            Option<Mut<Sprite>>,
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
                With<world::Rock>,
                With<SpriteRequest>,
            )>,
        ),
    >,
) {
    for (entity, chest, shelf, shell, item, weapon, barrel, table_prop, rock, request, own) in
        &mut entities
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
        let child = create_child_sprite(
            &mut commands,
            &asset_server,
            &mut layouts,
            entity,
            sheet,
            frame,
        );
        commands.entity(entity).insert(WorldSprite {
            id: id.to_string(),
            child,
        });
        if let (Some(_), Some(mut own)) = (request, own) {
            own.color.set_alpha(0.0);
        }
    }
}

/// Coffre ouvert : case `Open` ; présentoir vendu : caché ; arme au sol : le petit sprite d'arme
/// d'origine est caché (l'icône du monde le remplace).
#[allow(clippy::type_complexity)]
fn sync_world_sprites(
    table: SpriteTable,
    parents: Query<(
        &WorldSprite,
        Option<&Chest>,
        Option<&Shelf>,
        Option<&WeaponPickup>,
        Option<&Children>,
    )>,
    mut sprites: Query<(&mut Sprite, &mut Visibility)>,
    markers: Query<(), Or<(With<crate::legibility::Halo>, With<crate::legibility::AimPivot>)>>,
) {
    for (world_sprite, chest, shelf, weapon, children) in &parents {
        if let Some(chest) = chest {
            if let Ok((mut sprite, _)) = sprites.get_mut(world_sprite.child) {
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
            if let Ok((_, mut visibility)) = sprites.get_mut(world_sprite.child) {
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
                if let Ok((_, mut visibility)) = sprites.get_mut(child) {
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

/// Pose sur chaque porte une image d'état (l'orientation vient de la taille de sa tuile de carte) et
/// rend transparent son sprite de carte. Sans entrée `door_v`/`door_h` (autre jeu), la porte garde le
/// rendu d'avant.
fn attach_door_sprites(
    mut commands: Commands,
    table: SpriteTable,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut doors: Query<(Entity, &mut Sprite), (With<DoorComponent>, Without<DoorSprite>)>,
) {
    for (entity, mut own) in &mut doors {
        let size = own
            .texture_atlas
            .as_ref()
            .and_then(|atlas| layouts.get(&atlas.layout)?.textures.get(atlas.index))
            .map(|rect| rect.size())
            .or_else(|| own.custom_size.map(|s| s.as_uvec2()));
        let id = match size {
            Some(size) if size.x > size.y => "door_h",
            _ => "door_v",
        };
        let (Some(sheet), Some(frame)) = (table.sheet(id), table.frame(id, "Idle")) else {
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
        own.color.set_alpha(0.0);
        commands.entity(entity).insert(DoorSprite { id, child });
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

/// Remplace le carré de couleur d'un projectile par l'image de son équipe : orbe rouge pour un tir
/// ennemi, losange bleu pour un tir allié (formes et couleurs distinctes, l'image porte sa couleur).
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
        sprite.color = Color::WHITE;
        sprite.custom_size = Some(Vec2::splat(bullet_display_size(fixed_math::to_f32(
            bullet.damage,
        ))));
        commands.entity(entity).insert(BulletSkinned);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(item_candidates("inconnu", Some(&items)), ["item/inconnu", "item"]);
        assert_eq!(weapon_candidates("arc"), ["weapon/arc", "weapon"]);
    }
}
