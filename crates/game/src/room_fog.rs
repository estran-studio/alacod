//! Exploration fog: authored visibility grid, confirmed memory, one world-space texture.
//! This plugin only observes simulation; its state is never checksummed or read by gameplay.
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    character::player::Player, collider::Collider, core::AppState, system_set::RollbackSystemSet,
};
use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_fixed::fixed_math::{Fixed, FixedTransform3D, FixedVec2};
use bevy_ggrs::{ConfirmedFrameCount, GgrsSchedule, RollbackFrameCount};
use map::game::entity::map::door::DoorComponent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FogSettings {
    pub enabled: bool,
    pub view_tiles: i32,
    pub window_tiles: i32,
    pub transition_seconds: f32,
    pub edge_pixels: f32,
    pub explored_opacity: f32,
}
impl Default for FogSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            view_tiles: 20,
            window_tiles: 8,
            transition_seconds: 0.2,
            edge_pixels: 8.0,
            explored_opacity: 0.72,
        }
    }
}

/// Generated map metadata. Grid coordinates are LDtk top-down, world origin is bottom-left.
/// Stable room ids and door endpoints are retained independently of spawner defence areas.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct FogMap {
    pub width: usize,
    pub height: usize,
    pub tile_size: i32,
    pub origin: [i32; 2],
    pub walls: Vec<bool>,
    pub rooms: Vec<FogRoom>,
    pub doors: Vec<FogDoor>,
    pub windows: Vec<[i32; 4]>,
    pub settings: FogSettings,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FogRoom {
    pub id: String,
    pub rect: [i32; 4],
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FogDoor {
    pub rooms: [String; 2],
    pub rect: [i32; 4],
}

impl FogMap {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.width == 0
            || self.height == 0
            || self.width > 1024
            || self.height > 1024
            || self.tile_size <= 0
            || self.walls.len() != self.width * self.height
        {
            return Err("invalid visibility grid");
        }
        let ids: BTreeSet<_> = self.rooms.iter().map(|r| r.id.as_str()).collect();
        if ids.len() != self.rooms.len()
            || self.doors.iter().any(|d| {
                d.rooms[0] == d.rooms[1] || d.rooms.iter().any(|id| !ids.contains(id.as_str()))
            })
        {
            return Err("invalid room connections");
        }
        let s = &self.settings;
        if !(1..=128).contains(&s.view_tiles)
            || !(1..=s.view_tiles).contains(&s.window_tiles)
            || !s.transition_seconds.is_finite()
            || !(0.0..=5.0).contains(&s.transition_seconds)
            || !s.edge_pixels.is_finite()
            || !(0.0..=16.0).contains(&s.edge_pixels)
            || !s.explored_opacity.is_finite()
            || !(0.0..=1.0).contains(&s.explored_opacity)
        {
            return Err("invalid fog settings");
        }
        Ok(())
    }
    fn index(&self, p: (i32, i32)) -> Option<usize> {
        (p.0 >= 0 && p.1 >= 0 && p.0 < self.width as i32 && p.1 < self.height as i32)
            .then(|| p.1 as usize * self.width + p.0 as usize)
    }
    fn world_cell(&self, p: Vec2) -> (i32, i32) {
        let x = ((p.x - self.origin[0] as f32) / self.tile_size as f32).floor() as i32;
        let y = ((p.y - self.origin[1] as f32) / self.tile_size as f32).floor() as i32;
        (x, self.height as i32 - 1 - y)
    }
    fn fixed_cell(&self, p: FixedVec2) -> (i32, i32) {
        let size = Fixed::from_num(self.tile_size);
        let x = ((p.x - Fixed::from_num(self.origin[0])) / size)
            .floor()
            .to_num::<i32>();
        let y = ((p.y - Fixed::from_num(self.origin[1])) / size)
            .floor()
            .to_num::<i32>();
        (x, self.height as i32 - 1 - y)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct VisionInput {
    players: Vec<(i32, i32)>,
    closed: Vec<(i32, i32)>,
}
#[derive(Resource, Default)]
struct VisionHistory {
    pending: BTreeMap<i32, VisionInput>,
}
impl VisionHistory {
    fn record(&mut self, frame: i32, input: VisionInput) {
        // Resimulation invalidates all later predicted samples, even before Update runs.
        self.pending.split_off(&frame);
        self.pending.insert(frame, input);
    }
}

#[derive(Resource, Default)]
pub struct FogView {
    map_key: Option<String>,
    visible: Vec<bool>,
    explored: Vec<bool>,
    last_current: Option<VisionInput>,
    last_confirmed: Option<VisionInput>,
    last_confirmed_view: Vec<bool>,
    texture: Option<Handle<Image>>,
    overlay: Option<Entity>,
    alpha: Vec<f32>,
}
impl FogView {
    /// Also used by world-space feedback/labels so hidden entities cannot leak information.
    pub fn sees(&self, map: Option<&FogMap>, p: Vec2) -> bool {
        let Some(map) = map.filter(|m| m.settings.enabled) else {
            return true;
        };
        map.index(map.world_cell(p))
            .is_some_and(|i| self.visible.get(i).copied().unwrap_or(false))
    }
}

#[derive(Component)]
struct FogCameraRestore(bevy::camera::ClearColorConfig);

pub struct RoomFogPlugin;
impl Plugin for RoomFogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VisionHistory>()
            .init_resource::<FogView>()
            .add_systems(
                GgrsSchedule,
                capture_vision
                    .after(RollbackSystemSet::Run)
                    .before(RollbackSystemSet::FrameCounter),
            )
            .add_systems(PreUpdate, update_fog.after(bevy_ggrs::RunGgrsSystems))
            .add_systems(
                PostUpdate,
                filter_world_sprites
                    .after(bevy::transform::TransformSystems::Propagate)
                    .before(bevy::camera::visibility::VisibilitySystems::CheckVisibility),
            )
            .add_systems(OnExit(AppState::InGame), reset_fog);
    }
}

/// Read-only observation in rollback schedule, using integers/fixed point only.
fn capture_vision(
    map: Option<Res<FogMap>>,
    frame: Res<RollbackFrameCount>,
    players: Query<&FixedTransform3D, With<Player>>,
    doors: Query<(&FixedTransform3D, Option<&Collider>), With<DoorComponent>>,
    mut history: ResMut<VisionHistory>,
) {
    let Some(map) = map else {
        return;
    };
    if !map.settings.enabled {
        return;
    }
    let mut input = VisionInput {
        players: players
            .iter()
            .map(|p| map.fixed_cell(p.translation.truncate()))
            .collect(),
        closed: vec![],
    };
    input.players.sort_unstable();
    for portal in &map.doors {
        let opened = doors.iter().any(|(p, collider)| {
            collider.is_none() && in_rect(map.fixed_cell(p.translation.truncate()), portal.rect)
        });
        if !opened {
            let [x, y, w, h] = portal.rect;
            for yy in y..y + h {
                for xx in x..x + w {
                    input.closed.push((xx, yy));
                }
            }
        }
    }
    input.closed.sort_unstable();
    input.closed.dedup();
    history.record(frame.0, input);
}

fn in_rect(p: (i32, i32), r: [i32; 4]) -> bool {
    p.0 >= r[0] && p.1 >= r[1] && p.0 < r[0] + r[2] && p.1 < r[1] + r[3]
}

/// Supercover traversal, including both sides of diagonal corners. The first occluder
/// is visible, but cells behind it are not. Windows shorten the remaining exterior view.
fn ray_visible(map: &FogMap, blocked: &[bool], start: (i32, i32), end: (i32, i32)) -> bool {
    if start == end {
        return true;
    }
    let (dx, dy) = (end.0 - start.0, end.1 - start.1);
    let (nx, ny) = (dx.abs(), dy.abs());
    let (sx, sy) = (dx.signum(), dy.signum());
    let mut p = start;
    let (mut ix, mut iy) = (0, 0);
    let mut window: Option<(i32, i32)> = None;
    while ix < nx || iy < ny {
        let decision = (1 + 2 * ix) * ny - (1 + 2 * iy) * nx;
        if decision == 0 {
            // A ray grazing a solid corner cannot see through the diagonal gap.
            for side in [(p.0 + sx, p.1), (p.0, p.1 + sy)] {
                if map.index(side).is_none_or(|i| blocked[i]) {
                    return false;
                }
            }
            p.0 += sx;
            p.1 += sy;
            ix += 1;
            iy += 1;
        } else if decision < 0 {
            p.0 += sx;
            ix += 1;
        } else {
            p.1 += sy;
            iy += 1;
        }
        let Some(i) = map.index(p) else {
            return false;
        };
        if let Some(w) = window {
            if (p.0 - w.0).pow(2) + (p.1 - w.1).pow(2) > map.settings.window_tiles.pow(2) {
                return false;
            }
        }
        if p == end {
            return true;
        }
        if blocked[i] {
            return false;
        }
        if window.is_none() && map.windows.iter().any(|&r| in_rect(p, r)) {
            window = Some(p);
        }
    }
    true
}

fn field_of_view(map: &FogMap, input: &VisionInput) -> Vec<bool> {
    let mut blocked = map.walls.clone();
    // Only authored connections can be portals; missing runtime doors remain closed.
    // No proximity matching and no dependency on spawner rectangles.
    for &p in &input.closed {
        if let Some(i) = map.index(p) {
            blocked[i] = true;
        }
    }
    // Opening a portal makes its neighbour eligible for exploration, not fully visible.
    // Windows may reveal the exterior, never the contents of a still locked room.
    let mut accessible: BTreeSet<&str> = map
        .rooms
        .iter()
        .filter(|r| input.players.iter().any(|&p| in_rect(p, r.rect)))
        .map(|r| r.id.as_str())
        .collect();
    loop {
        let before = accessible.len();
        for portal in &map.doors {
            let closed = input.closed.iter().any(|&p| in_rect(p, portal.rect));
            if !closed
                && portal
                    .rooms
                    .iter()
                    .any(|id| accessible.contains(id.as_str()))
            {
                accessible.extend(portal.rooms.iter().map(String::as_str));
            }
        }
        if accessible.len() == before {
            break;
        }
    }
    let mut hidden = vec![false; blocked.len()];
    for room in map
        .rooms
        .iter()
        .filter(|r| !accessible.contains(r.id.as_str()))
    {
        let [x, y, w, h] = room.rect;
        for yy in y..y + h {
            for xx in x..x + w {
                if let Some(i) = map.index((xx, yy)) {
                    hidden[i] = true;
                    blocked[i] = true;
                }
            }
        }
    }
    let mut visible = vec![false; blocked.len()];
    let radius = map.settings.view_tiles;
    for &p in &input.players {
        if map.index(p).is_none() {
            continue;
        }
        for y in (p.1 - radius).max(0)..=(p.1 + radius).min(map.height as i32 - 1) {
            for x in (p.0 - radius).max(0)..=(p.0 + radius).min(map.width as i32 - 1) {
                if !hidden[y as usize * map.width + x as usize]
                    && (x - p.0).pow(2) + (y - p.1).pow(2) <= radius.pow(2)
                    && ray_visible(map, &blocked, p, (x, y))
                {
                    visible[y as usize * map.width + x as usize] = true;
                }
            }
        }
    }
    // Keep the complete authored door face legible when any part is in sight.
    // Only its wall cells are revealed; the closed portal remains an occluder.
    for portal in &map.doors {
        let [x, y, w, h] = portal.rect;
        let face_seen = (y..y + h)
            .any(|yy| (x..x + w).any(|xx| map.index((xx, yy)).is_some_and(|i| visible[i])));
        if face_seen {
            for yy in y..y + h {
                for xx in x..x + w {
                    if let Some(i) = map.index((xx, yy)) {
                        visible[i] = true;
                    }
                }
            }
        }
    }
    visible
}

const TEXELS_PER_TILE: usize = 4;

fn update_fog(
    mut commands: Commands,
    map: Option<Res<FogMap>>,
    confirmed: Option<Res<ConfirmedFrameCount>>,
    mut history: ResMut<VisionHistory>,
    mut view: ResMut<FogView>,
    mut images: ResMut<Assets<Image>>,
    time: Res<Time>,
    mut cameras: Query<
        (Entity, &mut Camera, Option<&FogCameraRestore>),
        With<crate::camera::GameCamera>,
    >,
) {
    let Some(map) = map.filter(|m| m.settings.enabled) else {
        return;
    };
    for (e, mut camera, restore) in &mut cameras {
        if restore.is_none() {
            commands
                .entity(e)
                .insert(FogCameraRestore(camera.clear_color.clone()));
        }
        camera.clear_color = bevy::camera::ClearColorConfig::Custom(Color::BLACK);
    }
    let key = format!(
        "{}:{}:{}:{:?}",
        map.width, map.height, map.tile_size, map.rooms
    );
    if view.map_key.as_ref() != Some(&key) {
        if let Some(e) = view.overlay.take() {
            commands.entity(e).try_despawn();
        }
        if let Some(h) = view.texture.take() {
            images.remove(h.id());
        }
        *view = FogView::default();
        view.map_key = Some(key);
        view.visible = vec![false; map.walls.len()];
        view.explored = view.visible.clone();
        let (w, h) = (map.width * TEXELS_PER_TILE, map.height * TEXELS_PER_TILE);
        let mut image = Image::new_fill(
            Extent3d {
                width: w as u32,
                height: h as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0, 0, 0, 255],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );
        // Nearest sampling keeps unknown texels fully opaque. Soft edges are generated
        // inside visible cells; linear filtering would bleed across closed walls.
        image.sampler = ImageSampler::nearest();
        let handle = images.add(image);
        let size = Vec2::new(map.width as f32, map.height as f32) * map.tile_size as f32;
        view.overlay = Some(
            commands
                .spawn((
                    Sprite {
                        image: handle.clone(),
                        custom_size: Some(size),
                        ..default()
                    },
                    Transform::from_xyz(
                        map.origin[0] as f32 + size.x / 2.0,
                        map.origin[1] as f32 + size.y / 2.0,
                        200.0,
                    ),
                ))
                .id(),
        );
        view.texture = Some(handle);
        view.alpha = vec![1.0; w * h];
    }
    if let Some((_, input)) = history.pending.last_key_value() {
        if view.last_current.as_ref() != Some(input) {
            view.visible = field_of_view(&map, input);
            view.last_current = Some(input.clone());
        }
    }
    let confirmed = confirmed.map_or(-1, |f| f.0);
    // Only confirmed samples can permanently reveal terrain. Predicted samples never
    // modify memory, including a door purchase that later rolls back.
    for (_, input) in history.pending.range(..=confirmed) {
        if view.last_confirmed.as_ref() != Some(input) {
            view.last_confirmed_view = field_of_view(&map, input);
            view.last_confirmed = Some(input.clone());
        }
        let seen = view.last_confirmed_view.clone();
        for (known, visible) in view.explored.iter_mut().zip(seen) {
            *known |= visible;
        }
    }
    history.pending = history.pending.split_off(&(confirmed.saturating_add(1)));
    let width = map.width * TEXELS_PER_TILE;
    let height = map.height * TEXELS_PER_TILE;
    let step = if map.settings.transition_seconds > 0.0 {
        time.delta_secs() / map.settings.transition_seconds
    } else {
        1.0
    };
    let Some(handle) = view.texture.clone() else {
        return;
    };
    let Some(mut image) = images.get_mut(&handle) else {
        return;
    };
    let Some(data) = image.data.as_mut() else {
        return;
    };
    for py in 0..height {
        for px in 0..width {
            let (x, y) = (px / TEXELS_PER_TILE, py / TEXELS_PER_TILE);
            let i = y * map.width + x;
            let mut target = if view.visible[i] {
                0.0
            } else if view.explored[i] {
                map.settings.explored_opacity
            } else {
                1.0
            };
            if view.visible[i] && map.settings.edge_pixels > 0.0 {
                let local = Vec2::new(
                    (px % TEXELS_PER_TILE) as f32 + 0.5,
                    (py % TEXELS_PER_TILE) as f32 + 0.5,
                ) * (map.tile_size as f32 / TEXELS_PER_TILE as f32);
                let tile = map.tile_size as f32;
                for (dx, dy, distance) in [
                    (-1, 0, local.x),
                    (1, 0, tile - local.x),
                    (0, -1, local.y),
                    (0, 1, tile - local.y),
                ] {
                    let neighbour = (x as i32 + dx, y as i32 + dy);
                    if map.index(neighbour).is_none_or(|j| !view.visible[j]) {
                        target = target.max(
                            (1.0 - distance / map.settings.edge_pixels).clamp(0.0, 1.0)
                                * map.settings.explored_opacity,
                        );
                    }
                }
            }
            let pixel = py * width + px;
            // Hide immediately when vision is lost. Only reveal is interpolated: fading
            // back to black could leak a newly closed room after rollback.
            view.alpha[pixel] = if target >= view.alpha[pixel] {
                target
            } else {
                (view.alpha[pixel] - step).max(target)
            };
            data[pixel * 4 + 3] = (view.alpha[pixel] * 255.0).round() as u8;
        }
    }
}

/// Preserve each sprite's own alpha (flashes/fades), instead of changing Visibility
/// on simulation entities or restoring an inactive weapon by accident.
#[derive(Component)]
struct FogSpriteTint {
    base_alpha: f32,
    applied_alpha: f32,
}
fn filter_world_sprites(
    mut commands: Commands,
    map: Option<Res<FogMap>>,
    view: Res<FogView>,
    transforms: Query<&GlobalTransform>,
    parents: Query<&ChildOf>,
    dynamic: Query<
        (),
        Or<(
            With<crate::character::enemy::Enemy>,
            With<crate::weapons::Bullet>,
            With<crate::powerups::PowerUpPickup>,
            With<crate::weapons::WeaponPickup>,
            With<crate::interaction::WindowHealthBar>,
            With<crate::character::health::ui::HealthBar>,
        )>,
    >,
    mut sprites: Query<(
        Entity,
        &GlobalTransform,
        &mut Sprite,
        Option<&mut FogSpriteTint>,
    )>,
) {
    if map.as_ref().is_none_or(|m| !m.settings.enabled) {
        for (_, _, mut sprite, tint) in &mut sprites {
            if let Some(tint) = tint {
                sprite.color.set_alpha(tint.base_alpha);
            }
        }
        return;
    }
    for (e, global, mut sprite, tint) in &mut sprites {
        let mut root = e;
        let mut moving = dynamic.contains(root);
        let mut subject = root;
        while let Ok(parent) = parents.get(root) {
            root = parent.parent();
            if dynamic.contains(root) {
                moving = true;
                subject = root;
            }
        }
        // Mask world actors and children (animation layers, health bars, weapons).
        // Static scenery is handled by the global texture and remains in explored memory.
        if !moving && tint.is_none() {
            continue;
        }
        let position = transforms
            .get(subject)
            .map_or(global.translation(), |t| t.translation())
            .truncate();
        let visible = view.sees(map.as_deref(), position);
        let actual = sprite.color.alpha();
        let applied = if let Some(mut tint) = tint {
            if (actual - tint.applied_alpha).abs() > f32::EPSILON {
                tint.base_alpha = actual;
            }
            tint.applied_alpha = if visible { tint.base_alpha } else { 0.0 };
            tint.applied_alpha
        } else {
            let applied = if visible { actual } else { 0.0 };
            commands.entity(e).insert(FogSpriteTint {
                base_alpha: actual,
                applied_alpha: applied,
            });
            applied
        };
        sprite.color.set_alpha(applied);
    }
}

fn reset_fog(
    mut commands: Commands,
    mut view: ResMut<FogView>,
    mut history: ResMut<VisionHistory>,
    mut images: ResMut<Assets<Image>>,
    mut cameras: Query<(Entity, &mut Camera, &FogCameraRestore)>,
) {
    for (e, mut camera, restore) in &mut cameras {
        camera.clear_color = restore.0.clone();
        commands.entity(e).remove::<FogCameraRestore>();
    }
    if let Some(e) = view.overlay.take() {
        commands.entity(e).try_despawn();
    }
    if let Some(h) = view.texture.take() {
        images.remove(h.id());
    }
    *view = FogView::default();
    history.pending.clear();
    commands.remove_resource::<FogMap>();
}

#[cfg(test)]
mod tests {
    use super::*;
    fn map() -> FogMap {
        let mut map = FogMap {
            width: 12,
            height: 10,
            tile_size: 16,
            origin: [0, -160],
            walls: vec![false; 120],
            rooms: vec![
                FogRoom {
                    id: "a".into(),
                    rect: [0, 0, 6, 10],
                },
                FogRoom {
                    id: "b".into(),
                    rect: [7, 0, 5, 10],
                },
            ],
            doors: vec![FogDoor {
                rooms: ["a".into(), "b".into()],
                rect: [6, 4, 1, 1],
            }],
            windows: vec![],
            settings: FogSettings {
                enabled: true,
                ..default()
            },
        };
        for y in 0..10 {
            map.walls[y * 12 + 6] = true;
        }
        map.walls[4 * 12 + 6] = false;
        map
    }
    #[test]
    fn closed_door_face_visible_room_hidden_open_door_reveals_only_line_of_sight() {
        let map = map();
        let input = VisionInput {
            players: vec![(3, 4)],
            closed: vec![(6, 4)],
        };
        let closed = field_of_view(&map, &input);
        assert!(closed[4 * 12 + 6]);
        assert!(!closed[4 * 12 + 8]);
        let open = field_of_view(
            &map,
            &VisionInput {
                closed: vec![],
                ..input
            },
        );
        assert!(open[4 * 12 + 8]);
        assert!(!open[12 + 10]);
    }
    #[test]
    fn exterior_unknown_window_limited_and_coop_unions_views() {
        let mut map = map();
        map.windows.push([6, 4, 1, 1]);
        map.settings.window_tiles = 2;
        let a = VisionInput {
            players: vec![(3, 4)],
            closed: vec![],
        };
        let visible = field_of_view(&map, &a);
        assert!(visible[4 * 12 + 8]);
        assert!(!visible[4 * 12 + 9]);
        let both = field_of_view(
            &map,
            &VisionInput {
                players: vec![(3, 4), (10, 8)],
                closed: vec![],
            },
        );
        assert!(both[8 * 12 + 10]);
        assert!(both[4 * 12 + 3]);
    }
    #[test]
    fn diagonal_corners_block_and_disconnected_wall_never_opens() {
        let mut map = map();
        map.walls[4 * 12 + 6] = true;
        assert!(
            !field_of_view(
                &map,
                &VisionInput {
                    players: vec![(3, 4)],
                    closed: vec![]
                }
            )[4 * 12 + 8]
        );
        let mut blocked = vec![false; 120];
        blocked[4 * 12 + 4] = true;
        blocked[5 * 12 + 3] = true;
        assert!(!ray_visible(&map, &blocked, (3, 4), (4, 5)));
    }
    #[test]
    fn rollback_discards_predicted_discovery_samples() {
        let mut history = VisionHistory::default();
        history.record(
            8,
            VisionInput {
                players: vec![(3, 4)],
                closed: vec![(6, 4)],
            },
        );
        history.record(
            9,
            VisionInput {
                players: vec![(8, 4)],
                closed: vec![],
            },
        );
        history.record(
            8,
            VisionInput {
                players: vec![(3, 4)],
                closed: vec![(6, 4)],
            },
        );
        assert_eq!(history.pending.len(), 1);
        assert!(!history.pending.contains_key(&9));
        let map = map();
        let known = field_of_view(&map, &history.pending[&8]);
        assert!(!known[4 * 12 + 8]);
    }
    #[test]
    fn texture_memory_dynamic_children_and_new_game_cleanup() {
        use bevy::ecs::system::RunSystemOnce;
        let map = map();
        let mut world = World::new();
        world.insert_resource(map.clone());
        world.init_resource::<Assets<Image>>();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(200));
        world.insert_resource(time);
        world.init_resource::<FogView>();
        world.init_resource::<VisionHistory>();
        world.insert_resource(ConfirmedFrameCount(7));
        world.resource_mut::<VisionHistory>().record(
            8,
            VisionInput {
                players: vec![(3, 4)],
                closed: vec![(6, 4)],
            },
        );
        world.run_system_once(update_fog).unwrap();
        let view = world.resource::<FogView>();
        assert!(view.visible[4 * 12 + 3]);
        assert!(!view.explored[4 * 12 + 3]);
        let h = view.texture.clone().unwrap();
        let images = world.resource::<Assets<Image>>();
        let image = images.get(&h).unwrap();
        // A texel beyond the door must be fully opaque, not merely darkened.
        let pixel = (4 * TEXELS_PER_TILE * 12 * TEXELS_PER_TILE + 8 * TEXELS_PER_TILE) * 4 + 3;
        assert_eq!(image.data.as_ref().unwrap()[pixel], 255);
        world.resource_mut::<ConfirmedFrameCount>().0 = 8;
        world.run_system_once(update_fog).unwrap();
        assert!(world.resource::<FogView>().explored[4 * 12 + 3]);
        assert!(world.resource::<VisionHistory>().pending.is_empty());
        // Simulate an enemy inside terrain already explored but currently out of sight.
        world.resource_mut::<FogView>().explored[4 * 12 + 8] = true;
        let enemy = world
            .spawn((
                crate::character::enemy::Enemy {},
                GlobalTransform::from_translation(Vec3::new(136.0, -72.0, 0.0)),
            ))
            .id();
        let sprite = world
            .spawn((Sprite::from_color(Color::WHITE, Vec2::ONE), ChildOf(enemy)))
            .id();
        world.run_system_once(filter_world_sprites).unwrap();
        assert_eq!(world.get::<Sprite>(sprite).unwrap().color.alpha(), 0.0);
        world.resource_mut::<FogView>().visible[4 * 12 + 8] = true;
        world.run_system_once(filter_world_sprites).unwrap();
        assert_eq!(world.get::<Sprite>(sprite).unwrap().color.alpha(), 1.0);
        world.run_system_once(reset_fog).unwrap();
        assert!(!world.contains_resource::<FogMap>());
        assert!(world.resource::<FogView>().explored.is_empty());
        assert!(world.resource::<Assets<Image>>().is_empty());
    }

    #[test]
    fn four_players_on_demo_sized_grid() {
        let mut map = map();
        map.width = 82;
        map.height = 70;
        map.walls = vec![false; 82 * 70];
        for y in 0..70 {
            for x in 0..82 {
                if x == 0 || y == 0 || x == 81 || y == 69 || (x % 23 == 6 && y > 5 && y < 64) {
                    map.walls[y * 82 + x] = true;
                }
            }
        }
        let input = VisionInput {
            players: vec![(12, 12), (35, 30), (58, 49), (70, 60)],
            closed: vec![],
        };
        for _ in 0..1000 {
            let view = field_of_view(&map, &input);
            assert!(
                view[12 * 82 + 12]
                    && view[30 * 82 + 35]
                    && view[49 * 82 + 58]
                    && view[60 * 82 + 70]
            );
            std::hint::black_box(view);
        }
    }
    #[test]
    fn windows_cannot_reveal_a_locked_room_even_when_geometry_is_clear() {
        let mut map = map();
        map.walls.fill(false);
        map.rooms[0].rect = [0, 0, 4, 10];
        map.rooms[1].rect = [8, 0, 4, 10];
        map.doors[0].rect = [6, 0, 1, 1];
        map.windows = vec![[4, 4, 1, 1], [7, 4, 1, 1]];
        let input = VisionInput {
            players: vec![(3, 4)],
            closed: vec![(6, 0)],
        };
        let view = field_of_view(&map, &input);
        assert!(view[4 * 12 + 5]);
        assert!(!view[4 * 12 + 9]);
        let open = field_of_view(
            &map,
            &VisionInput {
                closed: vec![],
                ..input
            },
        );
        assert!(open[4 * 12 + 9]);
    }
    #[test]
    fn two_tile_door_face_is_legible_without_revealing_locked_room() {
        let mut map = map();
        map.doors[0].rect = [6, 4, 1, 2];
        map.walls[5 * 12 + 6] = false;
        let view = field_of_view(
            &map,
            &VisionInput {
                players: vec![(3, 4)],
                closed: vec![(6, 4), (6, 5)],
            },
        );
        assert!(view[4 * 12 + 6]);
        assert!(view[5 * 12 + 6]);
        assert!(!view[5 * 12 + 7]);
    }
}
