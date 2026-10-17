//! Lisibilité des entités (fiche `m2-lisibilite-gungeon`, `docs/conventions.md` §83) : tout ce qui
//! aide à comprendre ce qu'on voit, **dérivé de l'état dans `Update`**, jamais dans la simulation.
//!
//! - **Pointeur de visée** : un chevron tourne autour de chaque ennemi vers sa cible (visée figée de
//!   l'émetteur pendant un pattern, sinon cible de l'IA, sinon cible de déplacement).
//! - **Halo pulsant** sous chaque ramassable, coffre et présentoir : teinte par catégorie
//!   (douille, objet par rareté, arme, power-up, coffre, présentoir).
//! - **Couleur par handle** des joueurs (`player_colors` de `hud.ron`) via `MarkerTint`.
//! - **Étiquettes** (touche F6, `ALACOD_LABELS=1`) : nom de l'ennemi, de l'objet, prix, état des
//!   portes et coffres, au-dessus des entités. Allumées par défaut en capture (vidéos de revue).
//! - **Légende** (touche F7, source `legend` de `hud.ron`) : voir `ui::hud`.
//!
//! Tout est piloté par des **données** (`sprites.ron` : `halo`, `aim`, les silhouettes ; `hud.ron` :
//! `names`, `player_colors`, `legend`) : un jeu sans l'entrée garde le rendu d'avant. Rien ici n'est
//! spécifique à un jeu. Les composants posés (`AimPointer`, `Halo`, `Label`) ne sont pas rollback :
//! une entité ressuscitée par un rollback les reçoit de nouveau.
use animation::SpriteSheetConfig;
use bevy::prelude::*;
use bevy_fixed::fixed_math::{self, FixedTransform3D};
use combat::actors::{Enemy, Player};
use combat::emitter::Emitter;
use combat::weapons::WeaponPickup;
use items::{Chest, ItemPickup, ItemTable, Rarity, Shelf, ShellPickup};
use utils::frame::FrameCount;

use crate::character::enemy::ai::pathing::EnemyPath;
use crate::character::enemy::ai::state::EnemyTarget;
use crate::character::visuals::{CharacterAppearance, MarkerTint, VisualsAttached};
use crate::collider::{Collider, ColliderShape};
use crate::interaction::Interactable;
use crate::powerups::PowerUpPickup;
use crate::room_fog::{FogMap, FogView};
use crate::ui::hud::{HudConfig, HudConfigHandle};
use crate::world_sprites::{atlas_sprite, item_candidates, SpriteTable};
use map::game::entity::map::character_spawn::CharacterSpawnComponent;
use map::game::entity::map::door::DoorComponent;
use map::game::entity::MapRollbackItem;

/// Étiquettes au-dessus des entités (F6). Au démarrage : `ALACOD_LABELS=1`.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LabelsEnabled(pub bool);

impl Default for LabelsEnabled {
    fn default() -> Self {
        Self(std::env::var("ALACOD_LABELS").is_ok_and(|v| v == "1"))
    }
}

/// Légende du HUD ouverte (F7).
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LegendOpen(pub bool);

pub struct LegibilityPlugin;

impl Plugin for LegibilityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LabelsEnabled>()
            .init_resource::<LegendOpen>()
            .add_systems(
                Update,
                (
                    toggle_overlays,
                    attach_aim_pointers,
                    aim_pointers_face_target.after(attach_aim_pointers),
                    attach_halos,
                    pulse_halos.after(attach_halos),
                    tint_players_by_handle,
                    hide_map_markers,
                    sync_labels,
                ),
            );
    }
}

/// F6 : étiquettes ; F7 : légende.
fn toggle_overlays(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut labels: ResMut<LabelsEnabled>,
    mut legend: ResMut<LegendOpen>,
) {
    if keyboard.just_pressed(KeyCode::F6) {
        labels.0 = !labels.0;
        info!("Étiquettes : {}", if labels.0 { "oui" } else { "non" });
    }
    if keyboard.just_pressed(KeyCode::F7) {
        legend.0 = !legend.0;
    }
}

// ---------------------------------------------------------------------------------------------
// Pointeur de visée
// ---------------------------------------------------------------------------------------------

/// Pivot du pointeur (enfant de l'ennemi) : sa rotation est l'angle vers la cible.
#[derive(Component)]
pub struct AimPivot;

/// L'ennemi porte son pointeur ; `0` est le pivot.
#[derive(Component)]
pub struct AimPointer(Entity);

/// Côté affiché du chevron (unités monde) et écart entre le bord du collider et lui.
const POINTER_SIZE: f32 = 20.0;
const POINTER_GAP: f32 = 7.0;

fn collider_radius(collider: Option<&Collider>) -> f32 {
    match collider.map(|c| &c.shape) {
        Some(ColliderShape::Circle { radius }) => fixed_math::to_f32(*radius),
        Some(ColliderShape::Rectangle { width, height }) => {
            fixed_math::to_f32((*width).max(*height)) / 2.0
        }
        None => 10.0,
    }
}

#[allow(clippy::type_complexity)]
fn attach_aim_pointers(
    mut commands: Commands,
    table: SpriteTable,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    enemies: Query<
        (Entity, Option<&Collider>),
        (With<Enemy>, With<VisualsAttached>, Without<AimPointer>),
    >,
) {
    let Some(sheet) = table.sheet("aim") else {
        return;
    };
    let Some(frame) = table.frame("aim", "Idle") else {
        return;
    };
    for (entity, collider) in &enemies {
        let mut sprite = atlas_sprite(&asset_server, &mut layouts, sheet, frame);
        sprite.custom_size = Some(Vec2::splat(POINTER_SIZE));
        // Rouge : un pointeur d'ennemi est un danger ; l'image est blanche, la teinte vient d'ici.
        sprite.color = Color::srgba(1.0, 0.35, 0.3, 0.95);
        let radius = collider_radius(collider) + POINTER_GAP;
        let pivot = commands
            .spawn((
                AimPivot,
                Transform::from_xyz(0.0, 0.0, 1.5),
                Visibility::Hidden,
                children![(sprite, Transform::from_xyz(radius, 0.0, 0.0))],
            ))
            .id();
        commands.entity(entity).try_insert(AimPointer(pivot));
        commands.queue_silenced(move |world: &mut World| {
            if let Ok(mut parent) = world.get_entity_mut(entity) {
                parent.add_child(pivot);
            } else if let Ok(orphan) = world.get_entity_mut(pivot) {
                orphan.despawn();
            }
        });
    }
}

/// Direction vers la cible : visée figée de l'émetteur, sinon dernière position connue de la cible,
/// sinon cible de déplacement. `None` : aucune cible connue (le pointeur est caché).
fn aim_direction(
    transform: &FixedTransform3D,
    emitter: Option<&Emitter>,
    target: Option<&EnemyTarget>,
    path: Option<&EnemyPath>,
) -> Option<Vec2> {
    let from = transform.translation.truncate();
    let to_vec = |p: fixed_math::FixedVec2| {
        Vec2::new(
            (p.x - from.x).to_num::<f32>(),
            (p.y - from.y).to_num::<f32>(),
        )
    };
    let direction = if let Some(emitter) = emitter.filter(|e| e.aim != fixed_math::FixedVec2::ZERO)
    {
        Vec2::new(emitter.aim.x.to_num::<f32>(), emitter.aim.y.to_num::<f32>())
    } else if let Some(position) = target.and_then(|t| t.last_known_position) {
        to_vec(position)
    } else if let Some(path) = path {
        to_vec(path.target_position)
    } else {
        return None;
    };
    (direction.length_squared() > 1e-4).then_some(direction)
}

#[allow(clippy::type_complexity)]
fn aim_pointers_face_target(
    enemies: Query<
        (
            &FixedTransform3D,
            Option<&Emitter>,
            Option<&EnemyTarget>,
            Option<&EnemyPath>,
            &AimPointer,
        ),
        With<Enemy>,
    >,
    mut pivots: Query<(&mut Transform, &mut Visibility), With<AimPivot>>,
) {
    for (transform, emitter, target, path, pointer) in &enemies {
        let Ok((mut pivot, mut visibility)) = pivots.get_mut(pointer.0) else {
            continue;
        };
        match aim_direction(transform, emitter, target, path) {
            Some(direction) => {
                pivot.rotation = Quat::from_rotation_z(direction.y.atan2(direction.x));
                *visibility = Visibility::Inherited;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Halo pulsant
// ---------------------------------------------------------------------------------------------

/// Halo posé sous un ramassable (enfant) ; `base` est sa couleur de catégorie.
#[derive(Component)]
pub struct Halo {
    base: Color,
    phase: f32,
}

/// L'entité porte son halo.
#[derive(Component)]
pub struct PickupHalo;

/// Côté du halo (unités monde) et amplitude de sa respiration.
const HALO_SIZE: f32 = 34.0;

fn rarity_color(rarity: Rarity) -> Color {
    match rarity {
        Rarity::Common => Color::srgb(0.85, 0.92, 1.0),
        Rarity::Uncommon => Color::srgb(0.35, 0.8, 1.0),
        Rarity::Rare => Color::srgb(1.0, 0.85, 0.25),
        Rarity::Epic => Color::srgb(0.85, 0.45, 1.0),
        Rarity::Legendary => Color::srgb(1.0, 0.55, 0.2),
    }
}

#[allow(clippy::type_complexity)]
fn attach_halos(
    mut commands: Commands,
    table: SpriteTable,
    items: Option<Res<ItemTable>>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    pickups: Query<
        (
            Entity,
            Option<&ItemPickup>,
            Option<&ShellPickup>,
            Option<&WeaponPickup>,
            Option<&PowerUpPickup>,
            Option<&Chest>,
            Option<&Shelf>,
        ),
        (
            Without<PickupHalo>,
            Or<(
                With<ItemPickup>,
                With<ShellPickup>,
                With<WeaponPickup>,
                With<PowerUpPickup>,
                With<Chest>,
                With<Shelf>,
            )>,
        ),
    >,
) {
    let Some(sheet) = table.sheet("halo") else {
        return;
    };
    let Some(frame) = table.frame("halo", "Idle") else {
        return;
    };
    for (entity, item, shell, weapon, powerup, chest, shelf) in &pickups {
        let color = if let Some(item) = item {
            let rarity = items
                .as_deref()
                .and_then(|table| table.0.get(&item.item_id))
                .map_or(Rarity::Common, |def| def.rarity);
            rarity_color(rarity)
        } else if shell.is_some() {
            Color::srgb(1.0, 0.82, 0.2)
        } else if weapon.is_some() {
            Color::srgb(1.0, 0.6, 0.2)
        } else if powerup.is_some() {
            Color::srgb(0.3, 1.0, 0.55)
        } else if chest.is_some() {
            Color::srgb(0.3, 0.95, 1.0)
        } else if shelf.is_some() {
            Color::srgb(0.55, 1.0, 0.45)
        } else {
            continue;
        };
        let mut sprite = atlas_sprite(&asset_server, &mut layouts, sheet, frame);
        sprite.custom_size = Some(Vec2::splat(HALO_SIZE));
        sprite.color = color.with_alpha(0.5);
        let halo = commands
            .spawn((
                Halo {
                    base: color,
                    phase: (entity.index_u32() % 16) as f32 * 0.4,
                },
                sprite,
                Transform::from_xyz(0.0, 0.0, 0.1),
            ))
            .id();
        commands.entity(entity).try_insert(PickupHalo);
        commands.queue_silenced(move |world: &mut World| {
            if let Ok(mut parent) = world.get_entity_mut(entity) {
                parent.add_child(halo);
            } else if let Ok(orphan) = world.get_entity_mut(halo) {
                orphan.despawn();
            }
        });
    }
}

/// Respiration du halo (alpha et taille, sur l'horloge de simulation : image n = frame n en capture)
/// et disparition quand le coffre est ouvert ou le présentoir vendu.
#[allow(clippy::type_complexity)]
fn pulse_halos(
    frame: Res<FrameCount>,
    parents: Query<(Option<&Chest>, Option<&Shelf>)>,
    mut halos: Query<(&Halo, &ChildOf, &mut Sprite, &mut Visibility)>,
) {
    for (halo, child_of, mut sprite, mut visibility) in &mut halos {
        let (chest, shelf) = parents.get(child_of.parent()).unwrap_or((None, None));
        let spent = chest.is_some_and(|c| c.opened) || shelf.is_some_and(|s| s.sold);
        let wanted = if spent {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if *visibility != wanted {
            *visibility = wanted;
        }
        let wave = (frame.frame as f32 * 0.1 + halo.phase).sin();
        sprite.color = halo.base.with_alpha(0.42 + 0.22 * wave);
        sprite.custom_size = Some(Vec2::splat(HALO_SIZE * (1.0 + 0.1 * wave)));
    }
}

// ---------------------------------------------------------------------------------------------
// Couleur par handle
// ---------------------------------------------------------------------------------------------

/// Teinte les calques d'un joueur de la couleur de son handle (`player_colors` de `hud.ron`), sauf
/// si son personnage déclare déjà un marqueur. Lue ensuite par `feedback::apply_layer_colors`.
fn tint_players_by_handle(
    mut commands: Commands,
    handle: Res<HudConfigHandle>,
    configs: Res<Assets<HudConfig>>,
    players: Query<(Entity, &Player), (With<VisualsAttached>, Without<MarkerTint>)>,
) {
    let Some(config) = handle.get(&configs) else {
        return;
    };
    if config.player_colors.is_empty() {
        return;
    }
    for (entity, player) in &players {
        let color = &config.player_colors[player.handle % config.player_colors.len()];
        commands
            .entity(entity)
            .try_insert(MarkerTint(crate::ui::hud::parse_color(color)));
    }
}

// ---------------------------------------------------------------------------------------------
// Repères de carte
// ---------------------------------------------------------------------------------------------

/// Cache les repères d'auteur de la carte (apparition d'ennemis : une tuile crâne-et-os dans
/// l'éditeur) quand `hide_markers: true` dans `hud.ron` : ce ne sont pas des objets du jeu et on les
/// prenait pour des ennemis ou des butins. Le sprite devient transparent, l'entité reste.
fn hide_map_markers(
    handle: Res<HudConfigHandle>,
    configs: Res<Assets<HudConfig>>,
    mut markers: Query<&mut Sprite, With<CharacterSpawnComponent>>,
) {
    if !handle.get(&configs).is_some_and(|c| c.hide_markers) {
        return;
    }
    for mut sprite in &mut markers {
        if sprite.color.alpha() != 0.0 {
            sprite.color.set_alpha(0.0);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Étiquettes
// ---------------------------------------------------------------------------------------------

/// Étiquette d'une entité (entité indépendante, comme `PowerUpLabel` : elle suit sa cible).
#[derive(Component)]
pub struct Label {
    target: Entity,
    text: String,
}

/// Fond de l'étiquette.
#[derive(Component)]
struct LabelBackground;

/// Texte d'étiquette posé par un système de la simulation ou de la carte sur une entité qui n'a pas
/// d'autre source de nom (ex. le portail) : `WorldLabel("Portail")`.
#[derive(Component, Clone, Debug)]
pub struct WorldLabel(pub String);

const LABEL_FONT: f32 = 8.0;
/// Chasse de Fira Mono (largeur d'un caractère / corps).
const LABEL_ADVANCE: f32 = 0.6;

struct LabelWish {
    target: Entity,
    position: Vec2,
    text: String,
    color: Color,
}

fn pretty(id: &str) -> String {
    let spaced = id.replace('_', " ");
    let mut chars = spaced.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}

#[derive(bevy::ecs::system::SystemParam)]
struct LabelSources<'w, 's> {
    enemies: Query<
        'w,
        's,
        (
            Entity,
            &'static FixedTransform3D,
            &'static CharacterAppearance,
            Option<&'static Collider>,
        ),
        With<Enemy>,
    >,
    players: Query<'w, 's, (Entity, &'static FixedTransform3D, &'static Player)>,
    items: Query<'w, 's, (Entity, &'static FixedTransform3D, &'static ItemPickup)>,
    weapons: Query<'w, 's, (Entity, &'static FixedTransform3D, &'static WeaponPickup)>,
    shells: Query<'w, 's, (Entity, &'static FixedTransform3D, &'static ShellPickup)>,
    chests: Query<'w, 's, (Entity, &'static FixedTransform3D, &'static Chest)>,
    shelves: Query<'w, 's, (Entity, &'static FixedTransform3D, &'static Shelf)>,
    barrels: Query<'w, 's, (Entity, &'static FixedTransform3D), With<world::BarrelState>>,
    tables: Query<'w, 's, (Entity, &'static FixedTransform3D), With<world::TableState>>,
    rocks: Query<'w, 's, (Entity, &'static FixedTransform3D), With<world::Rock>>,
    custom: Query<'w, 's, (Entity, &'static GlobalTransform, &'static WorldLabel)>,
    doors: Query<
        'w,
        's,
        (
            Entity,
            &'static GlobalTransform,
            Has<Collider>,
            Has<Interactable>,
        ),
        (With<DoorComponent>, With<MapRollbackItem>),
    >,
}

fn at(t: &FixedTransform3D, up: f32) -> Vec2 {
    let p = t.translation;
    Vec2::new(p.x.to_num::<f32>(), p.y.to_num::<f32>() + up)
}

fn gather_labels(
    sources: &LabelSources,
    config: Option<&HudConfig>,
    items: Option<&ItemTable>,
) -> Vec<LabelWish> {
    let name_of = |id: &str, fallback: &str| -> String {
        config
            .and_then(|c| c.names.get(id))
            .cloned()
            .unwrap_or_else(|| fallback.to_string())
    };
    // Un présentoir vend un objet (`<id>`) ou une arme (`weapon:<id>`).
    let item_name = |id: &str| -> String {
        if let Some(weapon) = id.strip_prefix("weapon:") {
            return name_of(weapon, &pretty(weapon));
        }
        items
            .and_then(|t| t.0.get(id))
            .map_or_else(|| pretty(id), |def| def.name.clone())
    };
    let enemy_color = Color::srgb(1.0, 0.7, 0.65);
    let loot_color = Color::srgb(1.0, 0.95, 0.65);
    let world_color = Color::srgb(0.85, 0.92, 0.95);
    let mut out = Vec::new();
    for (target, t, appearance, collider) in &sources.enemies {
        let up = collider_radius(collider) + 18.0;
        out.push(LabelWish {
            target,
            position: at(t, up),
            text: name_of(&appearance.config_name, &pretty(&appearance.config_name)),
            color: enemy_color,
        });
    }
    for (target, t, player) in &sources.players {
        out.push(LabelWish {
            target,
            position: at(t, 26.0),
            text: format!("{} {}", name_of("player", "J"), player.handle + 1),
            color: Color::WHITE,
        });
    }
    for (target, t, item) in &sources.items {
        out.push(LabelWish {
            target,
            position: at(t, 20.0),
            text: item_name(&item.item_id),
            color: loot_color,
        });
    }
    for (target, t, weapon) in &sources.weapons {
        let name = name_of(&weapon.weapon_id, &pretty(&weapon.weapon_id));
        let text = match weapon.price {
            Some(price) => format!("{name} ${price}"),
            None => name,
        };
        out.push(LabelWish {
            target,
            position: at(t, 20.0),
            text,
            color: loot_color,
        });
    }
    for (target, t, shell) in &sources.shells {
        out.push(LabelWish {
            target,
            position: at(t, 14.0),
            text: format!("{} +{}", name_of("shell", "Douilles"), shell.amount),
            color: loot_color,
        });
    }
    for (target, t, chest) in &sources.chests {
        let (key, fallback) = match (chest.opened, chest.locked) {
            (true, _) => ("chest_open", "Coffre vide"),
            (false, true) => ("chest_locked", "Coffre (clé)"),
            (false, false) => ("chest", "Coffre"),
        };
        out.push(LabelWish {
            target,
            position: at(t, 22.0),
            text: name_of(key, fallback),
            color: world_color,
        });
    }
    for (target, t, shelf) in &sources.shelves {
        if shelf.sold {
            continue;
        }
        out.push(LabelWish {
            target,
            position: at(t, 24.0),
            text: format!("{} ${}", item_name(&shelf.item_id), shelf.price),
            color: Color::srgb(0.7, 1.0, 0.6),
        });
    }
    for (target, t) in &sources.barrels {
        out.push(LabelWish {
            target,
            position: at(t, 20.0),
            text: name_of("barrel", "Baril explosif"),
            color: world_color,
        });
    }
    for (target, t) in &sources.tables {
        out.push(LabelWish {
            target,
            position: at(t, 16.0),
            text: name_of("table", "Table"),
            color: world_color,
        });
    }
    for (target, t) in &sources.rocks {
        out.push(LabelWish {
            target,
            position: at(t, 14.0),
            text: name_of("rock", "Rocher"),
            color: world_color,
        });
    }
    for (target, global, closed, interactable) in &sources.doors {
        // Une porte ouverte est un passage : pas d'étiquette (il y en a des dizaines par étage).
        let (key, fallback) = match (closed, interactable) {
            (false, _) => continue,
            (true, true) => ("door_locked", "Porte verrouillée"),
            (true, false) => ("door_closed", "Porte fermée"),
        };
        out.push(LabelWish {
            target,
            position: global.translation().truncate() + Vec2::Y * 22.0,
            text: name_of(key, fallback),
            color: world_color,
        });
    }
    for (target, global, label) in &sources.custom {
        out.push(LabelWish {
            target,
            position: global.translation().truncate() + Vec2::Y * 30.0,
            text: name_of(&label.0, &pretty(&label.0)),
            color: world_color,
        });
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn sync_labels(
    mut commands: Commands,
    enabled: Res<LabelsEnabled>,
    sources: LabelSources,
    handle: Res<HudConfigHandle>,
    configs: Res<Assets<HudConfig>>,
    items: Option<Res<ItemTable>>,
    asset_server: Res<AssetServer>,
    fog: Res<FogView>,
    fog_map: Option<Res<FogMap>>,
    mut labels: Query<(Entity, &mut Label, &mut Transform, &mut Visibility)>,
    mut texts: Query<(&mut Text2d, &mut TextColor, &ChildOf), Without<LabelBackground>>,
    mut backgrounds: Query<(&mut Sprite, &ChildOf), With<LabelBackground>>,
    mut font: Local<Option<Handle<Font>>>,
) {
    if !enabled.0 {
        for (entity, ..) in &labels {
            commands.entity(entity).despawn();
        }
        return;
    }
    let config = handle.get(&configs);
    if font.is_none() {
        *font = config
            .and_then(|c| c.font.as_ref())
            .map(|path| asset_server.load(path));
    }
    let mut wishes = gather_labels(&sources, config, items.as_deref());
    declutter(&mut wishes);
    let mut seen = std::collections::BTreeSet::new();
    for wish in &wishes {
        seen.insert(wish.target);
    }
    // Étiquettes existantes : mise à jour de la position, du texte et de la visibilité.
    let mut have = std::collections::BTreeMap::new();
    for (entity, label, mut transform, mut visibility) in &mut labels {
        if !seen.contains(&label.target) {
            commands.entity(entity).despawn();
            continue;
        }
        have.insert(label.target, entity);
        if let Some(wish) = wishes.iter().find(|w| w.target == label.target) {
            transform.translation = wish.position.extend(60.0);
            let wanted = if fog.sees(fog_map.as_deref(), wish.position) {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            if *visibility != wanted {
                *visibility = wanted;
            }
        }
    }
    for (mut text, mut color, child_of) in &mut texts {
        let Ok((_, label, _, _)) = labels.get(child_of.parent()) else {
            continue;
        };
        if let Some(wish) = wishes.iter().find(|w| w.target == label.target) {
            if text.0 != wish.text {
                text.0 = wish.text.clone();
            }
            color.0 = wish.color;
        }
    }
    for (mut sprite, child_of) in &mut backgrounds {
        let Ok((_, label, _, _)) = labels.get(child_of.parent()) else {
            continue;
        };
        if let Some(wish) = wishes.iter().find(|w| w.target == label.target) {
            sprite.custom_size = Some(label_size(&wish.text));
        }
    }
    for wish in &wishes {
        if have.contains_key(&wish.target) {
            continue;
        }
        let size = label_size(&wish.text);
        let text_font = TextFont {
            font: font.clone().map(Into::into).unwrap_or_default(),
            font_size: FontSize::Px(LABEL_FONT),
            ..default()
        };
        commands.spawn((
            Label {
                target: wish.target,
                text: wish.text.clone(),
            },
            Transform::from_translation(wish.position.extend(60.0)),
            Visibility::Inherited,
            children![
                (
                    LabelBackground,
                    Sprite::from_color(Color::srgba(0.04, 0.04, 0.08, 0.72), size),
                    Transform::from_xyz(0.0, 0.0, -0.1),
                ),
                (
                    Text2d::new(wish.text.clone()),
                    text_font,
                    TextColor(wish.color),
                    Transform::from_xyz(0.0, 0.0, 0.0),
                ),
            ],
        ));
    }
}

/// Écarte les étiquettes qui se recouvrent : dans l'ordre des entités (stable d'une image à
/// l'autre), chacune monte tant qu'elle chevauche une étiquette déjà posée.
fn declutter(wishes: &mut [LabelWish]) {
    let mut order: Vec<usize> = (0..wishes.len()).collect();
    order.sort_by_key(|&i| wishes[i].target);
    let mut placed: Vec<(Vec2, Vec2)> = Vec::new();
    for i in order {
        let size = label_size(&wishes[i].text);
        let mut position = wishes[i].position;
        for _ in 0..12 {
            let clash = placed.iter().any(|(other, other_size)| {
                (position.x - other.x).abs() < (size.x + other_size.x) / 2.0
                    && (position.y - other.y).abs() < (size.y + other_size.y) / 2.0
            });
            if !clash {
                break;
            }
            position.y += size.y + 1.0;
        }
        wishes[i].position = position;
        placed.push((position, size));
    }
}

fn label_size(text: &str) -> Vec2 {
    Vec2::new(
        text.chars().count() as f32 * LABEL_FONT * LABEL_ADVANCE + 5.0,
        LABEL_FONT + 4.0,
    )
}

/// Ids de feuille que la légende sait montrer (`SpriteTable`), pour un id d'objet au sol.
pub(crate) fn legend_frame(
    table: &SpriteTable,
    id: &str,
    anim: &str,
    items: Option<&ItemTable>,
) -> Option<(SpriteSheetConfig, usize)> {
    let candidates = if id.starts_with("item/") {
        item_candidates(id.trim_start_matches("item/"), items)
    } else {
        vec![id.to_string()]
    };
    table
        .find(&candidates, anim)
        .map(|(_, sheet, frame)| (sheet.clone(), frame))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fixed::fixed_math::{Fixed, FixedVec2, FixedVec3};

    fn at_origin() -> FixedTransform3D {
        FixedTransform3D::new(
            FixedVec3::ZERO,
            fixed_math::FixedMat3::IDENTITY,
            FixedVec3::ONE,
        )
    }

    fn target_at(x: i32, y: i32) -> EnemyTarget {
        EnemyTarget {
            last_known_position: Some(FixedVec2::new(Fixed::from_num(x), Fixed::from_num(y))),
            ..Default::default()
        }
    }

    #[test]
    fn le_pointeur_vise_la_derniere_position_connue_de_la_cible() {
        let direction = aim_direction(&at_origin(), None, Some(&target_at(0, 30)), None).unwrap();
        assert!(direction.x.abs() < 1e-3 && direction.y > 29.0);
    }

    #[test]
    fn sans_cible_connue_le_pointeur_se_cache() {
        assert!(aim_direction(&at_origin(), None, None, None).is_none());
        assert!(aim_direction(&at_origin(), None, Some(&target_at(0, 0)), None).is_none());
    }

    fn wish(index: u32, x: f32, text: &str) -> LabelWish {
        LabelWish {
            target: Entity::from_raw_u32(index).unwrap(),
            position: Vec2::new(x, 0.0),
            text: text.into(),
            color: Color::WHITE,
        }
    }

    #[test]
    fn deux_etiquettes_qui_se_recouvrent_sont_empilees() {
        let mut wishes = vec![
            wish(1, 0.0, "Gilet pare-balles $30"),
            wish(2, 20.0, "Roulade offensive $20"),
        ];
        declutter(&mut wishes);
        // L'ordre de pose suit l'ordre des entités : l'une reste à 0, l'autre monte d'au moins une hauteur.
        let gap = (wishes[0].position.y - wishes[1].position.y).abs();
        assert!(gap >= label_size("x").y, "écart {gap}");
    }

    #[test]
    fn des_etiquettes_eloignees_ne_bougent_pas() {
        let mut wishes = vec![wish(1, 0.0, "Clé"), wish(2, 200.0, "Clé")];
        declutter(&mut wishes);
        assert!(wishes.iter().all(|w| w.position.y == 0.0));
    }

    #[test]
    fn un_nom_d_identifiant_devient_lisible() {
        assert_eq!(pretty("bullet_kin_squad"), "Bullet kin squad");
        assert_eq!(pretty(""), "");
    }

    #[test]
    fn la_largeur_d_une_etiquette_suit_le_nombre_de_caracteres() {
        assert!(label_size("Clé").x < label_size("Gilet pare-balles").x);
    }
}
