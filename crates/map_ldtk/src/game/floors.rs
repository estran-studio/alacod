//! Mode `Floors` (T1.8, chantier F1, `docs/conventions.md` §17) : niveaux successifs, portail,
//! passage au niveau suivant **dans la simulation**.
//!
//! # Chargement : un monde LDtk par niveau
//!
//! [`FloorPlan`] (hors rollback, figé pour la partie) liste les cartes de la séquence
//! (contenu `Floors`, `content::registry::FloorsEntry`). Au chargement de la partie
//! (`AppState::GameLoading`), `loader::setup_generated_map` charge **toutes** les cartes
//! distinctes, un monde LDtk par carte, marqué [`FloorWorld`] (son emplacement dans la
//! séquence). Les mondes se superposent (même origine) : seul celui du niveau courant est
//! visible ([`floor_world_visibility_system`], présentation). Le chargement attend que tous
//! les mondes aient fait apparaître leurs niveaux ([`FloorWorldsReady`]) puis ne crée les
//! entités rollback (murs, portes, fenêtres, personnages, joueurs...) que du premier niveau
//! (emplacement 0) ; le registre des entités de carte (`LdtkMapEntityLoadingRegistry`) garde
//! celles de tous les niveaux, triées.
//!
//! # Passage de niveau : dans `GgrsSchedule`
//!
//! Aucune attente de chargement pendant la partie : les cartes sont déjà là, figées, et
//! identiques sur tous les clients (même graine). [`floor_transition_system`] (rollback,
//! `RollbackSystemSet::Run`) lit ces données immuables pour créer le niveau suivant à la frame
//! exacte où un joueur franchit le portail : un rollback qui remonte avant cette frame
//! ressuscite l'ancien niveau (`despawn_rollback`) et retire le nouveau, comme pour n'importe
//! quel spawn de la simulation. Ordre de création (numérotation `GgrsNetId` continue, triée
//! par contenu) : entités de carte du registre, murs, personnages, armes murales, machines à
//! perk — le même qu'au chargement.
//!
//! Hors du mode `Floors` ([`FloorPlan`] absent), rien de ce module ne tourne (sauf la
//! recherche d'emplacement [`FloorSlots`], qui rend `0` pour la carte unique) : traces des
//! autres modes inchangées.

use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_ecs_ldtk::prelude::*;
use bevy_fixed::fixed_math::{self, FixedVec2};
use bevy_ggrs::{GgrsSchedule, Rollback, RollbackDespawnCommandExtension};
use content::{manifest::GameManifest, registry::Registry};
use game::{
    character::{
        enemy::{ai::navigation::FlowFieldCache, Enemy},
        health::Death,
        player::Player,
    },
    run_state::{
        finalize_run_summary_system, floor_levels, floors_victory_at_end,
        resolve_run_mode_with_floors,
    },
    system_set::RollbackSystemSet,
};
use map::game::entity::{
    map::{
        character_spawn::CharacterSpawnComponent, player_spawn::PlayerSpawnConfig,
        soda_location::SodaLocationComponent, weapon_location::WeaponLocationComponent,
    },
    MapRollbackItem,
};
use run::{
    floors::{barycenter, crosses_portal, level_for_floor, portal_should_open},
    FloorState, Run,
};
use utils::{frame::FrameCount, net_id::GgrsNetId, net_id::GgrsNetIdFactory, order_iter};

use super::{
    collider::spawn_level_walls,
    local::{
        spawn_level_characters, spawn_level_soda_locations, spawn_level_weapon_locations,
        LevelSpawnAssets,
    },
    plugin::{spawn_map_items, LdtkMapEntityLoadingRegistry},
};

/// Emplacement d'un monde LDtk dans la séquence du mode `Floors` : l'index de la première
/// entrée de [`FloorPlan::levels`] qui désigne sa carte. `FloorWorld(0)` pour la carte unique
/// des autres modes. Posé sur l'entité racine du monde (`LdtkWorldBundle`), hors rollback.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FloorWorld(pub usize);

/// Séquence de niveaux de la partie (mode `Floors`), hors rollback : ressource présente si et
/// seulement si la partie est en mode `Floors`. Calculée à l'entrée de `GameLoading`
/// ([`compute_floor_plan`]), du même mode que `game::jjrs` résoudra au démarrage de session.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct FloorPlan {
    /// Id de la séquence (`RunMode::Floors::config`).
    pub config: String,
    /// Cartes des niveaux, dans l'ordre de jeu (chemins relatifs aux assets).
    pub levels: Vec<String>,
    /// M2-T0d : le dernier niveau vidé termine la partie par une victoire (voir
    /// `content::registry::FloorsEntry::victory_at_end`).
    pub victory_at_end: bool,
}

impl FloorPlan {
    /// Emplacement (monde LDtk) d'une entrée de la séquence : la première entrée de même carte
    /// (une carte répétée réutilise le même monde, ses entités sont recréées à chaque passage).
    pub fn slot_of_level(&self, level: usize) -> usize {
        let path = &self.levels[level];
        self.levels.iter().position(|p| p == path).unwrap_or(level)
    }

    /// Emplacement du niveau d'index `floor` (boucle sur le dernier niveau,
    /// `run::floors::level_for_floor`).
    pub fn slot_for_floor(&self, floor: u32) -> usize {
        self.slot_of_level(level_for_floor(floor, self.levels.len()))
    }

    /// Emplacements distincts, dans l'ordre : un monde LDtk chargé par emplacement.
    pub fn distinct_slots(&self) -> Vec<usize> {
        (0..self.levels.len())
            .filter(|&level| self.slot_of_level(level) == level)
            .collect()
    }
}

/// Recherche de l'emplacement ([`FloorWorld`]) d'une entité LDtk, en remontant ses parents.
#[derive(SystemParam)]
pub struct FloorSlots<'w, 's> {
    parents: Query<'w, 's, &'static ChildOf>,
    worlds: Query<'w, 's, &'static FloorWorld>,
    projects: Query<'w, 's, (), With<LdtkProjectHandle>>,
}

impl FloorSlots<'_, '_> {
    /// Emplacement du monde de `entity` ; `0` si aucun ancêtre ne porte [`FloorWorld`]
    /// (carte unique chargée hors de `setup_generated_map`, ex. outils de prévisualisation).
    pub fn slot_of(&self, mut entity: Entity) -> usize {
        loop {
            if let Ok(world) = self.worlds.get(entity) {
                return world.0;
            }
            match self.parents.get(entity) {
                Ok(parent) => entity = parent.parent(),
                Err(_) => return 0,
            }
        }
    }

    /// Entité racine du monde LDtk (`LdtkProjectHandle`) de `entity`.
    pub fn world_of(&self, mut entity: Entity) -> Option<Entity> {
        loop {
            if self.projects.contains(entity) {
                return Some(entity);
            }
            entity = self.parents.get(entity).ok()?.parent();
        }
    }
}

/// Vrai quand chaque monde LDtk du [`FloorPlan`] a son projet chargé et tous ses niveaux
/// apparus (mode `Floors`, voir `plugin::wait_for_all_map_rollback_entity`).
#[derive(SystemParam)]
pub struct FloorWorldsReady<'w, 's> {
    worlds: Query<'w, 's, (Entity, &'static LdtkProjectHandle), With<FloorWorld>>,
    levels: Query<'w, 's, &'static ChildOf, With<LevelIid>>,
    projects: Res<'w, Assets<LdtkProject>>,
}

impl FloorWorldsReady<'_, '_> {
    pub fn all_spawned(&self, plan: &FloorPlan) -> bool {
        if self.worlds.iter().count() < plan.distinct_slots().len() {
            return false;
        }
        self.worlds.iter().all(|(world, handle)| {
            let Some(project) = self.projects.get(handle) else {
                return false;
            };
            let expected = project.data().iter_raw_levels().count();
            let spawned = self
                .levels
                .iter()
                .filter(|parent| parent.parent() == world)
                .count();
            spawned >= expected
        })
    }
}

/// Décide du mode `Floors` à l'entrée de `GameLoading` (avant `setup_generated_map`) : même
/// résolution que `game::jjrs` au démarrage de session (`resolve_run_mode_with_floors`).
pub fn compute_floor_plan(
    mut commands: Commands,
    manifest: Option<Res<GameManifest>>,
    registry: Option<Res<Registry>>,
    floors_override: Option<Res<game::run_state::FloorsOverride>>,
) {
    let mode = resolve_run_mode_with_floors(
        manifest.as_deref(),
        registry.as_deref(),
        floors_override.as_deref().map(|o| o.0.as_str()),
    );
    let levels = floor_levels(&mode, registry.as_deref()).filter(|levels| !levels.is_empty());
    let mode_for_victory = mode.clone();
    match (mode, levels) {
        (run::RunMode::Floors { config }, Some(levels)) => {
            info!(
                "mode Floors : séquence « {config} », {} niveau(x) : {levels:?}",
                levels.len()
            );
            let victory_at_end = floors_victory_at_end(&mode_for_victory, registry.as_deref());
            commands.insert_resource(FloorPlan {
                config,
                levels,
                victory_at_end,
            });
        }
        (run::RunMode::Floors { config }, None) => {
            warn!("mode Floors : séquence « {config} » inconnue ou vide (voir `alacod lint`), carte unique");
            commands.remove_resource::<FloorPlan>();
        }
        _ => commands.remove_resource::<FloorPlan>(),
    }
}

pub struct FloorsPlugin;

impl Plugin for FloorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            GgrsSchedule,
            (floor_portal_open_system, floor_transition_system)
                .chain()
                .after(finalize_run_summary_system)
                .in_set(RollbackSystemSet::Run)
                .run_if(resource_exists::<FloorPlan>),
        );
        app.add_systems(
            Update,
            (
                floor_world_visibility_system,
                floor_window_bars_system,
                floor_portal_visual_system,
            )
                .run_if(resource_exists::<FloorPlan>),
        );
        app.add_systems(
            Update,
            init_floor_state_when_map_loaded
                .run_if(on_message::<super::plugin::LdtkMapLoadingEvent>)
                .run_if(resource_exists::<FloorPlan>)
                .after(super::plugin::MapNetIdAssignment),
        );
    }
}

/// Position du portail d'un niveau : le barycentre des `PlayerSpawn` (`run::floors::barycenter`).
/// D48 (graine 53 de throne) : sur une caverne, le barycentre peut tomber dans la roche, hors de
/// portée du joueur (rayon de franchissement 24 px) ; il est alors ramené au centre de la case
/// la plus proche que le champ de flux du gabarit des joueurs atteint depuis leurs points
/// (`world::nav::nearest_reached_cell`). Inchangé quand sa case est atteinte, et hors caverne.
fn portal_anchor(
    caves: Option<&crate::loader::CaveSlots>,
    slot: usize,
    points: &[FixedVec2],
) -> Option<(bevy_fixed::fixed_math::Fixed, bevy_fixed::fixed_math::Fixed)> {
    let anchor = barycenter(points)?;
    let Some(caves) = caves.filter(|caves| caves.0.contains_key(&slot)) else {
        return Some(anchor);
    };
    let grid = super::cave::grid_for_slot(caves, slot);
    let sources: Vec<(u32, u32)> = points
        .iter()
        .map(|p| world::CellGrid::cell_of(*p))
        .filter(|&(x, y)| x >= 0 && y >= 0)
        .map(|(x, y)| (x as u32, y as u32))
        .collect();
    let target = world::CellGrid::cell_of(FixedVec2::new(anchor.0, anchor.1));
    match world::nav::nearest_reached_cell(&grid, &sources, target, false) {
        Some((x, y)) if (x as i32, y as i32) != target => {
            let center = world::CellGrid::cell_center(x, y);
            Some((center.x, center.y))
        }
        _ => Some(anchor),
    }
}

/// Premier niveau : position du portail (barycentre des `PlayerSpawn` de l'emplacement 0),
/// posée au chargement de la carte, avant la première frame (comme `Run`). Les ennemis placés
/// sont comptés par `local::spawn_characters_when_map_loaded`.
fn init_floor_state_when_map_loaded(
    mut floor_state: ResMut<FloorState>,
    player_spawns: Query<(Entity, &GlobalTransform, &PlayerSpawnConfig)>,
    slots: FloorSlots,
    caves: Option<Res<crate::loader::CaveSlots>>,
) {
    let points: Vec<FixedVec2> = player_spawns
        .iter()
        .filter(|(entity, _, _)| slots.slot_of(*entity) == 0)
        .map(|(_, transform, _)| fixed_math::vec3_to_fixed(transform.translation()).truncate())
        .collect();
    floor_state.index = 0;
    floor_state.portal_open = false;
    floor_state.anchor = portal_anchor(caves.as_deref(), 0, &points);
    info!(
        "Floors : niveau 0, portail en {:?}, {} ennemi(s)",
        floor_state.anchor, floor_state.enemies_placed
    );
}

/// Ouvre le portail quand le niveau courant n'a plus d'ennemi (`EntityCount(enemy) == 0` :
/// toute entité `Enemy` vivante, alliés et civils du testbed compris), tant que la partie est
/// en cours.
pub fn floor_portal_open_system(
    frame: Res<FrameCount>,
    mut run: ResMut<Run>,
    plan: Res<FloorPlan>,
    mut floor_state: ResMut<FloorState>,
    enemies: Query<(), With<Enemy>>,
) {
    if !run.is_playing() {
        return;
    }
    let alive = enemies.iter().count();
    if portal_should_open(&floor_state, alive) {
        // M2-T0d : séquence `victory_at_end`, dernier niveau vidé (le boss était le dernier
        // ennemi) : victoire, pas de portail. Le résumé est posé par `finalize_run_summary_system`.
        if plan.victory_at_end && floor_state.index as usize + 1 >= plan.levels.len() {
            info!(
                "ggrs{{f={} floors_victory floor={}}}",
                frame.frame, floor_state.index
            );
            run.step = run::RunStep::Ended {
                at_frame: frame.frame,
                outcome: run::RunEnd::Victory,
            };
            return;
        }
        floor_state.portal_open = true;
        info!(
            "ggrs{{f={} floor_portal_open floor={}}}",
            frame.frame, floor_state.index
        );
    }
}

/// Données LDtk d'un niveau, lues (jamais modifiées) par [`floor_transition_system`].
#[derive(SystemParam)]
pub struct FloorLevelData<'w, 's> {
    slots: FloorSlots<'w, 's>,
    /// D48 : grilles des cavernes, pour ramener le portail sur une case atteinte.
    caves: Option<Res<'w, crate::loader::CaveSlots>>,
    registry: Res<'w, LdtkMapEntityLoadingRegistry>,
    levels: Query<'w, 's, (Entity, &'static LevelIid, &'static Transform)>,
    projects: Query<'w, 's, &'static LdtkProjectHandle>,
    project_assets: Res<'w, Assets<LdtkProject>>,
    player_spawns: Query<'w, 's, (Entity, &'static GlobalTransform, &'static PlayerSpawnConfig)>,
    character_spawns: Query<
        'w,
        's,
        (
            Entity,
            &'static GlobalTransform,
            &'static CharacterSpawnComponent,
        ),
    >,
    weapon_locations: Query<
        'w,
        's,
        (
            Entity,
            &'static GlobalTransform,
            &'static WeaponLocationComponent,
        ),
    >,
    soda_locations: Query<
        'w,
        's,
        (
            Entity,
            &'static GlobalTransform,
            &'static SodaLocationComponent,
        ),
    >,
}

/// Entités rollback de la partie, pour le passage de niveau.
#[derive(SystemParam)]
pub struct FloorTransitionEntities<'w, 's> {
    players: Query<
        'w,
        's,
        (
            &'static GgrsNetId,
            &'static Player,
            &'static mut fixed_math::FixedTransform3D,
            Has<Death>,
            Has<combat::downed::Downed>,
        ),
    >,
    /// Toute entité rollback qui n'est pas un joueur (les armes portées, enfants d'un joueur,
    /// sont gardées : voir [`Self::owned_by_player`]).
    others: Query<'w, 's, (&'static GgrsNetId, Entity), (With<Rollback>, Without<Player>)>,
    parents: Query<'w, 's, &'static ChildOf>,
    is_player: Query<'w, 's, (), With<Player>>,
}

impl FloorTransitionEntities<'_, '_> {
    fn owned_by_player(&self, mut entity: Entity) -> bool {
        while let Ok(parent) = self.parents.get(entity) {
            entity = parent.parent();
            if self.is_player.contains(entity) {
                return true;
            }
        }
        false
    }
}

/// Passage au niveau suivant (mode `Floors`) : à la frame où un joueur debout (ni à terre ni
/// mort) franchit le portail ouvert, détruit (`despawn_rollback`) toute entité rollback qui
/// n'appartient pas à un joueur (ennemis, murs, portes, fenêtres, balles, objets au sol...),
/// remet le flow field à zéro, crée les entités du niveau suivant (numérotation continue :
/// `GgrsNetIdFactory` n'est pas remise à zéro) et place les joueurs sur ses points de départ
/// (même handle). Les joueurs gardent tout le reste (santé, armes, munitions, monnaie, perks).
#[allow(clippy::too_many_arguments)]
pub fn floor_transition_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    run: Res<Run>,
    plan: Res<FloorPlan>,
    mut floor_state: ResMut<FloorState>,
    mut id_factory: ResMut<GgrsNetIdFactory>,
    mut flow_field_cache: ResMut<FlowFieldCache>,
    assets: LevelSpawnAssets,
    data: FloorLevelData,
    mut entities: FloorTransitionEntities,
    caves: (
        Res<crate::loader::CaveSlots>,
        ResMut<world::CellGrid>,
        ResMut<world::SurfaceGrid>,
    ),
) {
    if !run.is_playing() || !floor_state.portal_open {
        return;
    }
    let crossing = order_iter!(entities.players)
        .into_iter()
        .find(|(_, _, transform, dead, downed)| {
            !*dead && !*downed && crosses_portal(&floor_state, transform.translation.truncate())
        })
        .map(|(net_id, player, ..)| (net_id.0, player.handle));
    let Some((crossing_net_id, crossing_handle)) = crossing else {
        return;
    };

    let next = floor_state.index + 1;
    let slot = plan.slot_for_floor(next);
    info!(
        "ggrs{{f={} floor_transition from={} to={} slot={} player={} net_id={}}}",
        frame.frame, floor_state.index, next, slot, crossing_handle, crossing_net_id
    );

    // 1. Niveau quitté : tout ce qui n'appartient pas à un joueur.
    let mut despawned = 0u32;
    for (_, entity) in order_iter!(entities.others) {
        if entities.owned_by_player(entity) {
            continue;
        }
        commands.entity(entity).despawn_rollback();
        despawned += 1;
    }

    // 2. Flow field : repart d'un cache vide, rechargé par les murs du nouveau niveau.
    *flow_field_cache = FlowFieldCache::default();

    // 3. Entités de carte (portes, fenêtres, spawners) du registre, déjà triées.
    spawn_map_items(
        &mut commands,
        data.registry
            .entities
            .iter()
            .filter(|item| item.slot == slot),
        &mut id_factory,
        &assets.collision_settings,
        false,
    );

    // 4. Murs (et surfaces, T1.7).
    let level_data: Vec<_> = data
        .levels
        .iter()
        .filter(|(entity, _, _)| data.slots.slot_of(*entity) == slot)
        .filter_map(|(entity, iid, transform)| {
            let world = data.slots.world_of(entity)?;
            let project = data.project_assets.get(data.projects.get(world).ok()?)?;
            Some((iid, transform, project))
        })
        .collect();
    let (cave_slots, mut cell_grid, mut surfaces) = caves;
    *surfaces = super::collider::surface_grid_of_levels(&level_data);
    spawn_level_walls(
        &mut commands,
        level_data,
        &assets.collision_settings,
        &mut id_factory,
        &mut flow_field_cache,
    );

    // T1.6 : terrain du nouveau niveau (grille intacte d'une caverne, ou vide), cohérent
    // avec les murs IntGrid qui viennent d'être créés.
    *cell_grid = super::cave::grid_for_slot(&cave_slots, slot);

    // 5. Joueurs : points de départ du nouveau niveau, par handle (repli : le plus petit
    // index, pour un niveau qui en déclare moins que de joueurs).
    let mut spawns: Vec<(usize, fixed_math::FixedVec3)> = data
        .player_spawns
        .iter()
        .filter(|(entity, _, _)| data.slots.slot_of(*entity) == slot)
        .map(|(_, transform, config)| {
            (
                config.index,
                fixed_math::vec3_to_fixed(transform.translation()),
            )
        })
        .collect();
    spawns.sort_by_key(|(index, _)| *index);
    let anchor_points: Vec<FixedVec2> = spawns.iter().map(|(_, p)| p.truncate()).collect();
    for (net_id, player, mut transform, _, _) in order_mut_players(&mut entities) {
        let Some(position) = spawns
            .iter()
            .find(|(index, _)| *index == player.handle)
            .or_else(|| spawns.first())
            .map(|(_, position)| *position)
        else {
            warn!(
                "Floors : niveau {next} sans PlayerSpawn, joueur {} laissé en place",
                player.handle
            );
            continue;
        };
        transform.translation = position;
        info!(
            "ggrs{{f={} floor_player_placed net_id={} handle={} pos=({},{})}}",
            frame.frame, net_id.0, player.handle, position.x, position.y
        );
    }

    // 6. Personnages, armes murales, machines à perk (ordre du chargement).
    let characters = data
        .character_spawns
        .iter()
        .filter(|(entity, _, _)| data.slots.slot_of(*entity) == slot)
        .map(|(_, transform, spawn)| (transform, spawn))
        .collect();
    // T1.9 : difficulté du nouvel étage (évaluée à neuf : l'horloge ne le verra qu'en fin de
    // frame).
    let difficulty = assets.difficulty.at_floor(next);
    let placed = spawn_level_characters(
        &mut commands,
        &assets,
        &mut id_factory,
        characters,
        difficulty,
    );
    let weapons = data
        .weapon_locations
        .iter()
        .filter(|(entity, _, _)| data.slots.slot_of(*entity) == slot)
        .map(|(_, transform, location)| (transform, location))
        .collect();
    spawn_level_weapon_locations(
        &mut commands,
        &assets.global_assets,
        &assets.weapons_asset,
        &mut id_factory,
        weapons,
    );
    let sodas = data
        .soda_locations
        .iter()
        .filter(|(entity, _, _)| data.slots.slot_of(*entity) == slot)
        .map(|(_, transform, location)| (transform, location))
        .collect();
    spawn_level_soda_locations(&mut commands, &mut id_factory, sodas);

    // 7. État du mode.
    floor_state.index = next;
    floor_state.portal_open = false;
    floor_state.anchor = portal_anchor(data.caves.as_deref(), slot, &anchor_points);
    floor_state.enemies_placed = floor_state.enemies_placed.saturating_add(placed);
    info!(
        "ggrs{{f={} floor_loaded floor={} despawned={} enemies={}}}",
        frame.frame, next, despawned, placed
    );
}

/// `order_mut_iter!` sur les joueurs de [`FloorTransitionEntities`] (la macro veut une query
/// nommée, pas un champ emprunté à travers une méthode).
fn order_mut_players<'a>(
    entities: &'a mut FloorTransitionEntities,
) -> Vec<(
    &'a GgrsNetId,
    &'a Player,
    Mut<'a, fixed_math::FixedTransform3D>,
    bool,
    bool,
)> {
    let mut items: Vec<_> = entities.players.iter_mut().collect();
    items.sort_unstable_by_key(|item| item.0 .0);
    items
}

// ---------------------------------------------------------------------------------------
// Présentation (Update, dérivée de l'état, jamais lue par la simulation)
// ---------------------------------------------------------------------------------------

/// Seul le monde LDtk du niveau courant est visible (les mondes se superposent).
fn floor_world_visibility_system(
    plan: Res<FloorPlan>,
    floor_state: Res<FloorState>,
    mut worlds: Query<(&FloorWorld, &mut Visibility)>,
) {
    let current = plan.slot_for_floor(floor_state.index);
    for (world, mut visibility) in worlds.iter_mut() {
        let wanted = if world.0 == current {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != wanted {
            *visibility = wanted;
        }
    }
}

/// Barre de vie des fenêtres créées par un passage de niveau (la simulation ne crée pas ce
/// visuel, voir `plugin::spawn_map_items`) : une par entité LDtk de fenêtre qui n'en a pas.
fn floor_window_bars_system(
    mut commands: Commands,
    windows: Query<&MapRollbackItem, With<map::game::entity::map::window::WindowHealth>>,
    children: Query<&Children>,
    bars: Query<(), With<game::interaction::WindowHealthBar>>,
) {
    for item in windows.iter() {
        let has_bar = children
            .get(item.parent)
            .is_ok_and(|children| children.iter().any(|child| bars.contains(child)));
        if has_bar {
            continue;
        }
        commands.entity(item.parent).with_children(|parent| {
            parent.spawn((
                game::interaction::WindowHealthBar,
                Sprite {
                    color: Color::srgb(0.0, 1.0, 0.0),
                    custom_size: Some(Vec2::new(0.0, 2.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 8.0, 0.1)),
            ));
        });
    }
}

/// Visuel du portail ouvert : un carré violet à la position du portail.
#[derive(Component)]
struct PortalVisual;

fn floor_portal_visual_system(
    mut commands: Commands,
    floor_state: Res<FloorState>,
    mut visuals: Query<(Entity, &mut Transform), With<PortalVisual>>,
) {
    let wanted = floor_state
        .portal_open
        .then(|| floor_state.anchor_vec())
        .flatten();
    match (wanted, visuals.single_mut()) {
        (Some(anchor), Ok((_, mut transform))) => {
            transform.translation = fixed_math::fixed_to_vec3(anchor.extend()).with_z(5.0);
        }
        (Some(anchor), Err(_)) => {
            commands.spawn((
                PortalVisual,
                Sprite {
                    color: Color::srgba(0.6, 0.2, 0.9, 0.8),
                    custom_size: Some(Vec2::splat(40.0)),
                    ..default()
                },
                Transform::from_translation(fixed_math::fixed_to_vec3(anchor.extend()).with_z(5.0)),
            ));
        }
        (None, Ok((entity, _))) => commands.entity(entity).despawn(),
        (None, Err(_)) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(levels: &[&str]) -> FloorPlan {
        FloorPlan {
            config: "test".to_string(),
            levels: levels.iter().map(|l| l.to_string()).collect(),
            victory_at_end: false,
        }
    }

    #[test]
    fn un_monde_par_carte_distincte() {
        let p = plan(&["a.ldtk", "b.ldtk", "a.ldtk", "c.ldtk"]);
        assert_eq!(p.distinct_slots(), vec![0, 1, 3]);
        assert_eq!(p.slot_for_floor(0), 0);
        assert_eq!(p.slot_for_floor(1), 1);
        // Carte répétée : même monde que sa première occurrence.
        assert_eq!(p.slot_for_floor(2), 0);
        assert_eq!(p.slot_for_floor(3), 3);
        // Boucle infinie sur le dernier niveau.
        assert_eq!(p.slot_for_floor(10), 3);
    }
}

/// D48 (graine 53 de throne) : sur les trois cavernes de throne et 1 000 graines, l'ancre du
/// portail est le centre d'une case que le champ de flux du gabarit des joueurs atteint depuis
/// leurs points (barycentre gardé tel quel quand sa case l'est).
#[cfg(test)]
mod portal_anchor_tests {
    use super::*;
    use crate::loader::CaveSlots;
    use world::CellGrid;

    #[test]
    fn ancre_du_portail_atteinte_sur_les_cavernes_de_throne() {
        let mut corrected = 0;
        for name in ["niveau_1", "niveau_2", "niveau_3"] {
            let path = format!(
                "{}/../../games/throne/assets/caves/{name}.ron",
                env!("CARGO_MANIFEST_DIR")
            );
            let config: world::CaveConfig =
                ron::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
            for seed in 1..=1000i32 {
                let mut caves = CaveSlots::default();
                caves.0.insert(0, (seed, config.clone()));
                let grid = super::super::cave::grid_for_slot(&caves, 0);
                let players = world::points_of_interest(&grid, 4, 0, 1, false).player_spawns;
                let points: Vec<FixedVec2> = players
                    .iter()
                    .map(|&(x, y)| CellGrid::cell_center(x, y))
                    .collect();
                let (ax, ay) = portal_anchor(Some(&caves), 0, &points).unwrap();
                let cell = CellGrid::cell_of(FixedVec2::new(ax, ay));
                let dist = world::nav::nav_distances(&grid, &players, false);
                assert!(
                    cell.0 >= 0
                        && cell.1 >= 0
                        && dist[(cell.1 as u32 * grid.width + cell.0 as u32) as usize] != u32::MAX
                        && grid.get(cell.0, cell.1).is_some_and(|k| !k.is_solid()),
                    "{name} graine {seed} : portail en {cell:?} hors d'atteinte"
                );
                if Some((ax, ay)) != barycenter(&points) {
                    corrected += 1;
                }
            }
        }
        println!("ancres ramenées : {corrected} / 3000");
    }
}
