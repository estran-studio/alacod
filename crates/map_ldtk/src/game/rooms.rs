//! Salles typées et verrouillées (M2-E1, voie B « un étage = un monde assemblé »,
//! `docs/conventions.md` §35). Les contrats (types, état rollback neutre, géométrie pure) sont
//! dans `world::rooms` ; ce module est le branchement ECS qui a besoin des portes et des
//! bornes de salle du chargement LDtk.
//!
//! - Le type de salle vient du champ de niveau LDtk `room_kind` (recopié du gabarit par
//!   l'assembleur), posé en [`world::RoomKind`] sur l'entité du niveau, à côté de `RoomBounds`.
//!   Sur une carte sans `room_kind`, aucun système de ce module ne fait quoi que ce soit.
//! - Au premier tick d'une carte typée, les portes posées sur le bord d'une salle typée
//!   s'ouvrent (une porte de salle est ouverte par défaut) ; [`DoorShape`] garde de quoi les
//!   refermer.
//! - Une salle dont le type verrouille (`RoomKindDef::locks`) passe `Dormant → Locked` quand
//!   un joueur debout est à l'intérieur et qu'un ennemi y vit : ses portes (et les portes
//!   jumelles, posées au bord de la salle voisine) se referment, les joueurs restés dehors
//!   sont téléportés à l'entrée. Puis `Locked → Cleared` quand plus aucun ennemi n'y vit : les
//!   portes se rouvrent. Une porte n'est jamais refermée sur un occupant : le verrouillage est
//!   différé tant qu'un joueur ou un ennemi chevauche l'une d'elles.
//! - Tant qu'une salle verrouillante dort, ses ennemis portent [`world::RoomDormant`] : sélection
//!   de cible, déplacement et attaque sont sautés (`game::character::enemy::ai`).
//! - Porte à clé (M3-T5, `docs/conventions.md` §70) : la porte d'une salle `door_lock: Key(n)` à
//!   l'étage courant (et sa jumelle reliée par le plan) garde son collider ; son `Interactable`
//!   devient `KeyDoor` ([`key_door_system`] l'ouvre contre `n` clés). Ni l'ouverture d'office, ni le
//!   verrouillage d'une salle voisine n'y touchent.
//!
//! Les colliders des portes sont déjà rollback (`Collider`, `CollisionLayer`) ; le champ de flux
//! suit tout seul (une porte avec collider est un mur, `rebuild_blocked_cells`).

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_fixed::fixed_math::{Fixed, FixedVec2, FixedVec3};
use bevy_fixed::rng::RngStreams;
use bevy_ggrs::GgrsSchedule;
use sim_core::frame_events::FrameEvents;
use sim_core::system_set::RollbackSystemSet;
use utils::frame::FrameCount;
use utils::net_id::GgrsNetId;
use utils::order_iter;
use world::{
    entrance_point, inside_box, rect_within_radius, room_rng_stream, ArenaClocks, DoorLock,
    DoorLocks, ExplosionRequest, RoomChanged, RoomDormant, RoomKind, RoomKindTable, RoomState,
    RoomStates,
};

use combat::actors::Velocity;
use combat::downed::Downed;
use game::boss::ReinforcementRequest;
use game::character::config::CharacterConfig;
use game::character::enemy::ai::navigation::{FlowFieldCache, GridPos};
use game::character::enemy::create::spawn_enemy;
use game::character::enemy::Enemy;
use game::character::health::Death;
use game::character::player::Player;
use game::collider::CollisionSettings;
use game::collider::{is_colliding, Collider, CollisionLayer};
use game::global_asset::GlobalAsset;
use game::interaction::{Interactable, InteractionEvent, InteractionType};
use game::weapons::{melee::MeleeWeaponsConfig, WeaponsConfig};
use map::game::entity::map::door::DoorComponent;
use map::game::entity::map::level_id::LevelId;
use map::game::entity::map::room::RoomBounds;
use sim_core::team::Team;
use utils::net_id::GgrsNetIdFactory;

/// Collider d'une porte fermée, pour la refermer (statique, hors rollback : la même valeur
/// pour toute la partie).
#[derive(Component, Clone, Debug)]
pub struct DoorShape {
    pub collider: Collider,
    pub layer: CollisionLayer,
}

pub struct RoomsPlugin;

impl Plugin for RoomsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            GgrsSchedule,
            (
                // Avant l'IA : les ennemis d'une salle dormante sautent leur tour.
                room_dormancy_system
                    .before(game::boss::boss_phase_system)
                    .in_set(RollbackSystemSet::EnemySpawning),
                room_lock_system
                    .after(super::floors::floor_transition_system)
                    .in_set(RollbackSystemSet::Run),
                // M3-T4 (§69) : une explosion qui `destroys` ouvre le mur secret qu'elle touche,
                // après l'émission des demandes de la frame (`World`, comme le terrain).
                secret_wall_system
                    .after(game::bombs::bomb_fuse_system)
                    .in_set(RollbackSystemSet::World),
                // M3-T4 : butin des rochers cassés, après `explosion_terrain_system` qui les casse.
                rock_loot_system
                    .after(game::bombs::explosion_terrain_system)
                    .before(world::plugin::apply_destroy_terrain_system)
                    .in_set(RollbackSystemSet::World),
                room_chest_system
                    .after(room_lock_system)
                    .in_set(RollbackSystemSet::Run),
                // M2-T6 : présentoirs d'une salle `boutique`, après le coffre de salle (tous
                // deux consomment `GgrsNetIdFactory`).
                room_shop_system
                    .after(room_chest_system)
                    .in_set(RollbackSystemSet::Run),
                // M3-T8 (§73) : butin du boss assigné à la génération (pool à épuisement), après
                // les coffres et présentoirs (même ordre canonique : coffres, présentoirs, boss).
                room_boss_slot_system
                    .after(room_shop_system)
                    .in_set(RollbackSystemSet::Run),
                // M2-T7 : vagues internes d'une arène, après son verrouillage de la frame.
                // Après le coffre de salle (M2-T5) : tous deux consomment `GgrsNetIdFactory`.
                arena_waves_system
                    .after(room_lock_system)
                    .after(room_chest_system)
                    .after(room_shop_system)
                    .in_set(RollbackSystemSet::Run),
                // M2-T7 : renforts d'une frise de boss, dans la frame de la demande (`Run` suit
                // `EnemySpawning`, où `boss_phase_system` les demande).
                boss_reinforcements_system
                    .after(arena_waves_system)
                    .in_set(RollbackSystemSet::Run),
                // M3-T8 (§73) : butin du boss mort dans la frame (`Run` suit `DeathManagement`), **avant** la
                // transition d'étage : un joueur au portail quand le dernier boss meurt change d'étage dans
                // la même frame, et l'événement (position du boss) doit se lire dans l'étage d'origine
                // (`CurrentRooms` et `FloorState` d'avant). Tous les autres systèmes de `Run` qui
                // consomment `GgrsNetIdFactory` (coffres, présentoirs, arène, renforts) suivent
                // `floor_transition_system` : l'ordre reste total.
                boss_drop_system
                    .before(super::floors::floor_transition_system)
                    .in_set(RollbackSystemSet::Run),
                // M2-T7 : la mort d'un boss vide sa salle (despawn dans la même frame).
                boss_room_clear_system
                    .after(game::boss::boss_defeated_system)
                    .before(game::character::health::rollback_apply_death)
                    .in_set(RollbackSystemSet::DeathManagement),
                // M3-T5 (§70) : l'Interaction sur une porte à clé l'ouvre contre `n` clés, au bout
                // de la chaîne des handlers d'interaction (ils écrivent tous `Inventory` : ordre
                // total, jamais d'ambiguïté).
                key_door_system
                    .after(game::shop::shelf_interaction_system)
                    .after(game::hub::hub_station_system)
                    .before(RollbackSystemSet::Movement)
                    .in_set(RollbackSystemSet::Interaction),
            ),
        );
        app.add_systems(
            Update,
            update_hud_room_geometry.before(game::ui::hud_model::HudModelSet),
        );
    }
}

/// Une porte appartient à une salle si elle est à moins de `DOOR_MARGIN` de ses bords (la porte
/// de la salle et sa jumelle, posée au bord de la salle voisine).
const DOOR_MARGIN: i32 = 24;
/// Un joueur est « dans » la salle s'il est à plus de `PLAYER_INSET` des bords (passé la porte).
const PLAYER_INSET: i32 = 24;
/// Profondeur (dans la salle) du point où l'on téléporte les joueurs absents.
const ENTRANCE_DEPTH: i32 = 40;
/// Écart entre deux joueurs téléportés au même point d'entrée.
const ENTRANCE_SPACING: i32 = 14;

/// Salles typées de l'étage courant. En mode `Floors`, chaque emplacement est un monde LDtk
/// chargé dès le départ (M2-T10 : deux étages assemblés se superposent dans l'espace) : seules
/// les salles de l'emplacement de l'étage en cours comptent, sinon les salles d'un autre étage
/// se verrouilleraient et se videraient en même temps que celles où se tient le joueur.
#[derive(SystemParam)]
pub struct CurrentRooms<'w, 's> {
    rooms: Query<
        'w,
        's,
        (
            Entity,
            &'static LevelId,
            &'static RoomBounds,
            &'static RoomKind,
        ),
    >,
    slots: super::floors::FloorSlots<'w, 's>,
    plan: Option<Res<'w, super::floors::FloorPlan>>,
    state: Option<Res<'w, run::FloorState>>,
}

impl CurrentRooms<'_, '_> {
    /// Salles typées de l'étage courant (ordre de requête : à trier par l'appelant ; public pour
    /// le bot `explorateur`, M2-T16, qui ne lit que l'état).
    pub fn iter(&self) -> impl Iterator<Item = (&LevelId, &RoomBounds, &RoomKind)> {
        let current = self
            .plan
            .as_ref()
            .zip(self.state.as_ref())
            .map(|(plan, state)| plan.slot_for_floor(state.index));
        self.rooms
            .iter()
            .filter(move |(entity, ..)| {
                current.is_none_or(|slot| self.slots.slot_of(*entity) == slot)
            })
            .map(|(_, id, bounds, kind)| (id, bounds, kind))
    }
}

/// Une salle secrète (`door_lock: Secret`) tant que son mur n'est pas ouvert (M3-T5, §70).
fn hidden_from_minimap(
    table: Option<&RoomKindTable>,
    locks: Option<&DoorLocks>,
    id: &str,
    kind: &RoomKind,
) -> bool {
    table.is_some_and(|table| is_secret(table, kind))
        && !locks.is_some_and(|locks| locks.is_open(id))
}

/// Présentation seulement : remplace aussi la liste quand l'étage disparaît ou change.
/// `CurrentRooms` filtre les mondes LDtk superposés à partir de l'état rollback courant.
///
/// M3-T5 (§70) : une salle secrète (`door_lock: Secret`) n'apparaît sur la minicarte qu'ouverte
/// (`DoorLocks`) : fermée, elle n'est pas dessinée, ni dans le cadrage de la carte.
fn update_hud_room_geometry(
    rooms: CurrentRooms,
    table: Option<Res<RoomKindTable>>,
    locks: Option<Res<DoorLocks>>,
    mut geometry: ResMut<game::ui::hud_model::HudRoomGeometry>,
) {
    let mut next: Vec<_> = rooms
        .iter()
        .filter(|(id, _, kind)| {
            !hidden_from_minimap(table.as_deref(), locks.as_deref(), &id.0, kind)
        })
        .map(|(id, bounds, kind)| game::ui::hud_model::HudRoomBounds {
            id: id.0.clone(),
            kind: kind.0.clone(),
            boss: table.as_deref().is_some_and(|table| table.is_boss(&kind.0)),
            position: bounds.position,
            size: bounds.size,
        })
        .collect();
    next.sort_by(|a, b| a.id.cmp(&b.id));
    if geometry.0 != next {
        geometry.0 = next;
    }
}

/// Salles typées triées par position (x, y) puis identifiant : ordre stable, indépendant des
/// `Entity`.
pub fn sorted_rooms<'a>(
    rooms: &'a CurrentRooms,
) -> Vec<(&'a LevelId, &'a RoomBounds, &'a RoomKind)> {
    let mut typed: Vec<_> = rooms.iter().collect();
    typed.sort_by(|a, b| {
        (a.1.position.x, a.1.position.y, &a.0 .0).cmp(&(b.1.position.x, b.1.position.y, &b.0 .0))
    });
    typed
}

/// Le type de la salle porte-t-il un mur secret (`door_lock: Secret`) ?
fn is_secret(table: &RoomKindTable, kind: &RoomKind) -> bool {
    table.door_lock(&kind.0) == Some(DoorLock::Secret)
}

/// Nombre de clés qu'ouvre la porte d'un type de salle **à l'étage `floor`** (`door_lock: Key(n)`
/// à partir de `lock_from_floor`) ; `None` : porte ordinaire ou mur secret.
pub fn key_lock_cost(table: &RoomKindTable, kind: &RoomKind, floor: u32) -> Option<u32> {
    match table.door_lock_at(&kind.0, floor) {
        Some(DoorLock::Key(n)) => Some(n),
        _ => None,
    }
}

/// Portée d'interaction d'une porte sans `Interactable` (sa plus grande dimension, comme à la
/// création de la porte dans `plugin::spawn_map_items`).
fn door_interaction_range(collider: &Collider) -> Fixed {
    match collider.shape {
        game::collider::ColliderShape::Rectangle { width, height } => width.max(height),
        game::collider::ColliderShape::Circle { radius } => radius * Fixed::from_num(2),
    }
}

/// La porte en `p` appartient-elle à la salle `bounds` (à moins de `DOOR_MARGIN` de ses bords) ?
pub fn door_near(bounds: &RoomBounds, p: FixedVec2) -> bool {
    inside_box(
        bounds.position,
        bounds.size,
        p,
        -Fixed::from_num(DOOR_MARGIN),
    )
}

/// Pose [`RoomDormant`] sur les ennemis qui vivent dans une salle verrouillante encore
/// dormante, le retire des autres. Ne fait rien sur une carte sans salle typée.
#[allow(clippy::type_complexity)]
pub fn room_dormancy_system(
    mut commands: Commands,
    states: Res<RoomStates>,
    table: Res<RoomKindTable>,
    rooms: CurrentRooms,
    players: Query<
        (
            &GgrsNetId,
            &bevy_fixed::fixed_math::FixedTransform3D,
            Has<Downed>,
        ),
        With<Player>,
    >,
    doors: Query<
        (
            &GgrsNetId,
            &bevy_fixed::fixed_math::FixedTransform3D,
            &DoorShape,
            Has<Collider>,
        ),
        With<DoorComponent>,
    >,
    enemies: Query<
        (
            &GgrsNetId,
            Entity,
            &bevy_fixed::fixed_math::FixedTransform3D,
            Has<RoomDormant>,
            Option<&Collider>,
        ),
        With<Enemy>,
    >,
) {
    if rooms.iter().next().is_none() {
        return;
    }
    let standing: Vec<_> = order_iter!(players)
        .into_iter()
        .filter_map(|(_, t, downed)| (!downed).then_some(t.translation.truncate()))
        .collect();
    let open_doors: Vec<_> = order_iter!(doors)
        .into_iter()
        .filter_map(|(_, t, shape, closed)| (!closed).then_some((t, shape)))
        .collect();
    let enemy_bodies: Vec<_> = order_iter!(enemies)
        .into_iter()
        .filter_map(|(_, _, t, _, collider)| collider.map(|c| (t, c)))
        .collect();
    let dormant: Vec<&RoomBounds> = rooms
        .iter()
        .filter(|(id, bounds, kind)| {
            table.locks(&kind.0)
                && states.get(&id.0) == RoomState::Dormant
                // D68 : un ennemi dans une porte empêche sa fermeture et donc son propre
                // réveil. Si un joueur est entré, réveiller la salle avant l'IA, même tant
                // que cette porte reste ouverte. Une porte occupée seulement par un joueur
                // conserve la règle habituelle (pas de réveil anticipé au passage du seuil).
                && !(standing.iter().any(|p| inside_box(bounds.position, bounds.size, *p, Fixed::from_num(PLAYER_INSET)))
                    && open_doors.iter().any(|(door_t, shape)| {
                        door_near(bounds, door_t.translation.truncate())
                            && enemy_bodies.iter().any(|(t, collider)| is_colliding(&door_t.translation, &shape.collider, &t.translation, collider))
                    }))
        })
        .map(|(_, bounds, _)| bounds)
        .collect();
    for (_, entity, transform, is_dormant, _) in order_iter!(enemies) {
        let p = transform.translation.truncate();
        let want = dormant
            .iter()
            .any(|bounds| inside_box(bounds.position, bounds.size, p, Fixed::ZERO));
        if want && !is_dormant {
            commands.entity(entity).insert(RoomDormant);
        } else if !want && is_dormant {
            commands.entity(entity).remove::<RoomDormant>();
        }
    }
}

/// M2-T5 (`docs/conventions.md` §48) : pose le coffre d'un type de salle (`RoomKindDef::chest`) au
/// centre de chaque salle de l'étage courant qui n'en a pas encore (clé `room:<salle>`, tirage
/// indépendant du moment). Un coffre ouvert reste en place (`opened`), donc n'est pas reposé.
#[allow(clippy::too_many_arguments)]
pub fn room_chest_system(
    frame: Res<FrameCount>,
    mut commands: Commands,
    table: Option<Res<RoomKindTable>>,
    rooms: CurrentRooms,
    existing: Query<&items::Chest>,
    tables: Res<items::LootTables>,
    floors: Res<game::loot::FloorLootTables>,
    item_table: Res<items::ItemTable>,
    unlocks: Res<items::UnlockTable>,
    active: Res<items::RunUnlocks>,
    run_seed: Option<Res<bevy_fixed::rng::RunSeed>>,
    floor: Option<Res<run::FloorState>>,
    mut pools: ResMut<items::PoolState>,
    mut id_factory: ResMut<utils::net_id::GgrsNetIdFactory>,
) {
    let Some(table) = table else {
        return;
    };
    if !table.0.values().any(|def| def.chest.is_some()) {
        return;
    }
    let (run_seed, floor) = game::loot::seed_context(&run_seed, &floor);
    let ctx = game::loot::SlotContext {
        frame: frame.frame,
        tables: &tables,
        items: &item_table,
        unlocks: &unlocks,
        active: &active,
        run_seed,
        floor,
    };
    let present: std::collections::BTreeSet<&str> =
        existing.iter().map(|chest| chest.key.as_str()).collect();
    for (id, bounds, kind) in sorted_rooms(&rooms) {
        let Some(def) = table.0.get(&kind.0).and_then(|def| def.chest.as_ref()) else {
            continue;
        };
        let key = format!("room:{}", id.0);
        if present.contains(key.as_str()) {
            continue;
        }
        let two = Fixed::from_num(2);
        let center = bevy_fixed::fixed_math::FixedVec3::new(
            bounds.position.x + bounds.size.x / two,
            bounds.position.y + bounds.size.y / two,
            bevy_fixed::fixed_math::FIXED_ZERO,
        );
        let chest = items::Chest {
            key,
            table: def.table.clone(),
            rarity: def.rarity,
            locked: def.locked,
            opened: false,
        };
        // M3-T8 (§73) : un pool qui s'épuise assigne l'objet ici, dans l'ordre des salles (x, y,
        // identifiant), pas à l'ouverture.
        let table_id = game::loot::chest_table_id(&chest, &floors, floor).to_string();
        game::loot::assign_slot(&ctx, &mut pools, &table_id, chest.rarity, false, &chest.key);
        game::loot::spawn_chest(&mut commands, &mut id_factory, chest, center);
    }
}

/// M2-T6 (`docs/conventions.md` §50) : pose les présentoirs d'un type de salle à boutique
/// (`RoomKindDef::shop`) au centre de chaque salle de l'étage courant qui n'en a pas encore
/// (clés `shelf:<salle>:<i>`). Objets tirés dans le pool de la boutique par
/// `game::shop::draw_shelves` (graine de contenu, indépendante du moment), prix par rareté
/// (`economy.ron`). Un présentoir vendu reste en place (`sold`), donc n'est pas reposé.
#[allow(clippy::too_many_arguments)]
pub fn room_shop_system(
    frame: Res<FrameCount>,
    mut commands: Commands,
    table: Option<Res<RoomKindTable>>,
    rooms: CurrentRooms,
    existing: Query<&items::Shelf>,
    tables: Res<items::LootTables>,
    item_table: Res<items::ItemTable>,
    unlocks: Res<items::UnlockTable>,
    active: Res<items::RunUnlocks>,
    global_assets: Res<game::global_asset::GlobalAsset>,
    economies: Res<Assets<game::economy::EconomyConfig>>,
    run_seed: Option<Res<bevy_fixed::rng::RunSeed>>,
    floor: Option<Res<run::FloorState>>,
    mut pools: ResMut<items::PoolState>,
    mut id_factory: ResMut<utils::net_id::GgrsNetIdFactory>,
) {
    let Some(table) = table else {
        return;
    };
    if !table.0.values().any(|def| def.shop.is_some()) {
        return;
    }
    let economy = global_assets
        .economy_config
        .as_ref()
        .and_then(|handle| economies.get(handle));
    let (run_seed, floor) = game::loot::seed_context(&run_seed, &floor);
    let ctx = game::loot::SlotContext {
        frame: frame.frame,
        tables: &tables,
        items: &item_table,
        unlocks: &unlocks,
        active: &active,
        run_seed,
        floor,
    };
    let present: std::collections::BTreeSet<&str> =
        existing.iter().map(|shelf| shelf.key.as_str()).collect();
    for (id, bounds, kind) in sorted_rooms(&rooms) {
        let Some(def) = table.0.get(&kind.0).and_then(|def| def.shop.as_ref()) else {
            continue;
        };
        if present.contains(format!("shelf:{}:0", id.0).as_str()) {
            continue;
        }
        let table_id = def.table_for_floor(floor);
        let Some(pool) = tables.0.get(table_id) else {
            continue;
        };
        // M3-T8 (§73) : un pool qui s'épuise assigne un objet par présentoir à la génération
        // (salles dans l'ordre x, y, identifiant) ; sinon le tirage d'avant, un flux par boutique.
        let picks = game::shop::assign_shelves(
            &ctx,
            &mut pools,
            table_id,
            &id.0,
            def.shelves,
            def.weapon_shelves,
        )
        .unwrap_or_else(|| {
            game::shop::draw_shelves(
                pool,
                &item_table,
                run_seed,
                floor,
                &id.0,
                def.shelves,
                def.weapon_shelves,
            )
        });
        let two = Fixed::from_num(2);
        let center_x = bounds.position.x + bounds.size.x / two;
        let center_y = bounds.position.y + bounds.size.y / two;
        let count = picks.len() as i32;
        for (index, item_id) in picks.into_iter().enumerate() {
            let rarity = match items::loot_weapon(&item_id) {
                Some(weapon) => pool.weapon_rarity(weapon).unwrap_or_default(),
                None => item_table
                    .0
                    .get(&item_id)
                    .map(|item| item.rarity)
                    .unwrap_or_default(),
            };
            // `cost` du type de salle : un prix en douilles fixe `price` ; vie maximale et
            // consommable vont dans `Shelf::cost` (le prix en douilles reste 0).
            let (price, cost) = match &def.cost {
                None => (game::shop::shelf_price(economy, rarity), None),
                Some(items::Price::Currency(amount)) => (*amount, None),
                Some(other) => (0, Some(other.clone())),
            };
            let offset = (2 * index as i32 - (count - 1)) * game::shop::SHELF_SPACING / 2;
            game::shop::spawn_shelf(
                &mut commands,
                &mut id_factory,
                items::Shelf {
                    key: format!("shelf:{}:{index}", id.0),
                    item_id,
                    price,
                    sold: false,
                    retry_after: 0,
                    cost,
                },
                bevy_fixed::fixed_math::FixedVec3::new(
                    center_x + Fixed::from_num(offset),
                    center_y,
                    bevy_fixed::fixed_math::FIXED_ZERO,
                ),
            );
        }
    }
}

/// M3-T8 (§73) : assigne à la génération de l'étage le butin du boss (`RoomKindDef::boss_drop`) de
/// chaque salle de ce type, dans l'ordre des salles (x, y, identifiant), quand le pool a `exhaust` :
/// l'objet est retiré du pool pour la run **maintenant**, pas à la mort du boss. Clé `boss:<salle>`.
/// Sans épuisement, rien à faire ici (le tirage se fait à la mort, sur la même clé).
#[allow(clippy::too_many_arguments)]
pub fn room_boss_slot_system(
    frame: Res<FrameCount>,
    table: Option<Res<RoomKindTable>>,
    rooms: CurrentRooms,
    tables: Res<items::LootTables>,
    item_table: Res<items::ItemTable>,
    unlocks: Res<items::UnlockTable>,
    active: Res<items::RunUnlocks>,
    run_seed: Option<Res<bevy_fixed::rng::RunSeed>>,
    floor: Option<Res<run::FloorState>>,
    mut pools: ResMut<items::PoolState>,
) {
    let Some(table) = table else {
        return;
    };
    if !table.0.values().any(|def| def.boss_drop.is_some()) {
        return;
    }
    let (run_seed, floor) = game::loot::seed_context(&run_seed, &floor);
    let ctx = game::loot::SlotContext {
        frame: frame.frame,
        tables: &tables,
        items: &item_table,
        unlocks: &unlocks,
        active: &active,
        run_seed,
        floor,
    };
    for (id, _, kind) in sorted_rooms(&rooms) {
        let Some(drop) = table.0.get(&kind.0).and_then(|def| def.boss_drop.as_ref()) else {
            continue;
        };
        let key = format!("boss:{}", id.0);
        if pools.assignment(floor, &key).is_some() {
            continue;
        }
        game::loot::assign_slot(&ctx, &mut pools, &drop.table, drop.rarity, false, &key);
    }
}

/// M3-T8 (§73) : à la mort d'un boss dont la salle a un `boss_drop`, l'objet tombe au centre de la
/// salle. L'objet assigné à la génération s'il y en a un (pool à épuisement), sinon un tirage sur
/// la graine de l'emplacement `boss:<salle>` (indépendant du moment). `FrameEvents<BossDefeated>`
/// est lu dans la frame de sa mort (`Run` suit `DeathManagement`).
#[allow(clippy::too_many_arguments)]
pub fn boss_drop_system(
    frame: Res<FrameCount>,
    defeated: Res<FrameEvents<behaviors::BossDefeated>>,
    table: Option<Res<RoomKindTable>>,
    rooms: CurrentRooms,
    tables: Res<items::LootTables>,
    item_table: Res<items::ItemTable>,
    unlocks: Res<items::UnlockTable>,
    active: Res<items::RunUnlocks>,
    mut pools: ResMut<items::PoolState>,
    run_seed: Option<Res<bevy_fixed::rng::RunSeed>>,
    floor: Option<Res<run::FloorState>>,
    global_assets: Res<GlobalAsset>,
    weapons: Res<Assets<WeaponsConfig>>,
    mut commands: Commands,
    mut id_factory: ResMut<GgrsNetIdFactory>,
) {
    let Some(table) = table else {
        return;
    };
    let (run_seed, floor) = game::loot::seed_context(&run_seed, &floor);
    // Garde-fou : une salle ne lâche son butin qu'une fois par frame, même si deux `BossPlan` y meurent
    // ensemble (deux événements). Entre deux frames, rien ne l'empêche (dette, §73).
    let mut dropped: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for event in defeated.iter() {
        let at = FixedVec2::new(event.x, event.y);
        let Some((id, bounds, kind)) = sorted_rooms(&rooms)
            .into_iter()
            .find(|(_, bounds, _)| inside_box(bounds.position, bounds.size, at, Fixed::ZERO))
        else {
            continue;
        };
        let Some(drop) = table.0.get(&kind.0).and_then(|def| def.boss_drop.as_ref()) else {
            continue;
        };
        let key = format!("boss:{}", id.0);
        if !dropped.insert(key.clone()) {
            continue;
        }
        // Cas limite : un boss qui meurt au tout premier tick, avant `room_boss_slot_system` (qui suit
        // `floor_transition_system`) : un pool à épuisement est alors assigné ici, avec la même
        // fonction (idempotente), pour que le drop reste l'objet assigné et sorte du pool.
        let ctx = game::loot::SlotContext {
            frame: frame.frame,
            tables: &tables,
            items: &item_table,
            unlocks: &unlocks,
            active: &active,
            run_seed,
            floor,
        };
        if pools.assignment(floor, &key).is_none() {
            game::loot::assign_slot(&ctx, &mut pools, &drop.table, drop.rarity, false, &key);
        }
        let item = match pools.assignment(floor, &key) {
            Some(item) => Some(item.to_string()),
            None => tables.0.get(&drop.table).and_then(|pool| {
                let pool = items::unlocked_table(&drop.table, pool, &unlocks, &active);
                let mut rng =
                    bevy_fixed::rng::RollbackRng::new(items::loot_seed(run_seed, floor, &key));
                items::draw_loot_with(
                    &pool,
                    &item_table,
                    None,
                    drop.rarity,
                    false,
                    &unlocks.hidden_items(&active),
                    &mut rng,
                )
            }),
        };
        let Some(item) = item else {
            warn!(
                "butin du boss « {key} » : le pool « {} » ne donne rien",
                drop.table
            );
            continue;
        };
        let two = Fixed::from_num(2);
        let center = FixedVec3::new(
            bounds.position.x + bounds.size.x / two,
            bounds.position.y + bounds.size.y / two,
            bevy_fixed::fixed_math::FIXED_ZERO,
        );
        info!(
            "ggrs{{f={} boss_drop room={} item={item}}}",
            frame.frame, id.0
        );
        game::loot::spawn_drawn_loot(
            &mut commands,
            &mut id_factory,
            &item_table,
            &global_assets,
            &weapons,
            &item,
            center,
            event.net_id,
            &format!("butin du boss « {key} »"),
        );
    }
}

#[allow(clippy::type_complexity)]
pub fn room_lock_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    table: Res<RoomKindTable>,
    mut states: ResMut<RoomStates>,
    mut arena_clocks: ResMut<ArenaClocks>,
    mut changed: ResMut<FrameEvents<RoomChanged>>,
    rooms: CurrentRooms,
    doors: Query<
        (
            &GgrsNetId,
            Entity,
            &bevy_fixed::fixed_math::FixedTransform3D,
            &DoorShape,
            Has<Collider>,
        ),
        (With<DoorComponent>, Without<Player>, Without<Enemy>),
    >,
    enemies: Query<
        (
            &GgrsNetId,
            &bevy_fixed::fixed_math::FixedTransform3D,
            Option<&Collider>,
            Entity,
        ),
        (With<Enemy>, Without<Player>),
    >,
    mut players: Query<
        (
            &GgrsNetId,
            &mut bevy_fixed::fixed_math::FixedTransform3D,
            Option<&Collider>,
            Option<&mut Velocity>,
            Has<Downed>,
        ),
        (With<Player>, Without<Enemy>),
    >,
    floor: Option<Res<run::FloorState>>,
    door_configs: Query<(&DoorComponent, Option<&Interactable>)>,
) {
    let typed = sorted_rooms(&rooms);
    if typed.is_empty() {
        return;
    }
    let floor_index = floor.as_ref().map_or(0, |floor| floor.index);

    // Une porte du mur secret (la porte d'une salle `Secret` et sa jumelle, au bord de la salle
    // voisine) ne s'ouvre que par une explosion (`secret_wall_system`) : ni l'ouverture d'office,
    // ni la fin d'une salle voisine (`Locked → Cleared`) ne lui retirent son collider.
    let secret_door = |p: FixedVec2| {
        typed
            .iter()
            .any(|(_, bounds, kind)| is_secret(&table, kind) && door_near(bounds, p))
    };
    // M3-T5 : une porte à clé (la porte d'une salle `Key(n)` à l'étage courant et sa jumelle, reliées
    // par le plan : `paired_door`) ne s'ouvre que par `key_door_system`. Une ouverture murée de la
    // salle (sans jumelle) reste une porte de salle ordinaire.
    let key_door = |entity: Entity, p: FixedVec2| {
        door_configs
            .get(entity)
            .is_ok_and(|(door, _)| door.config.paired_door.is_some())
            && typed.iter().any(|(_, bounds, kind)| {
                key_lock_cost(&table, kind, floor_index).is_some() && door_near(bounds, p)
            })
    };
    // Porte dont l'état (ouverte ou fermée) est du seul ressort de son système : ni l'ouverture
    // d'office, ni le verrouillage ou la fin d'une salle voisine n'y touchent.
    let sealed_door = |entity: Entity, p: FixedVec2| secret_door(p) || key_door(entity, p);

    // Portes de salle ouvertes par défaut (une fois).
    if !states.doors_opened {
        for (_, entity, t, shape, closed) in order_iter!(doors) {
            let p = t.translation.truncate();
            if closed && typed.iter().any(|(_, bounds, _)| door_near(bounds, p)) {
                if secret_door(p) {
                    // M3-T4 : porte d'une salle secrète (et sa jumelle) : un mur plein, qu'aucune
                    // Interaction n'ouvre ; seule une explosion `destroys` l'ouvre.
                    commands.entity(entity).remove::<Interactable>();
                } else if key_door(entity, p) {
                    // M3-T5 : porte à clé : elle garde son collider, son Interactable devient
                    // `KeyDoor` (même portée) : `handle_door_interaction` ne la voit plus.
                    let range = door_configs
                        .get(entity)
                        .ok()
                        .and_then(|(_, interactable)| interactable)
                        .map_or_else(
                            || door_interaction_range(&shape.collider),
                            |interactable| interactable.interaction_range,
                        );
                    commands.entity(entity).insert(Interactable {
                        interaction_range: range,
                        interaction_type: InteractionType::KeyDoor,
                    });
                } else {
                    commands
                        .entity(entity)
                        .remove::<(Collider, CollisionLayer, Interactable)>();
                }
            }
        }
        states.doors_opened = true;
    }

    let inset = Fixed::from_num(PLAYER_INSET);
    for (index, (level_id, bounds, kind)) in typed.iter().enumerate() {
        if !table.locks(&kind.0) {
            continue;
        }
        let step = states.get(&level_id.0);
        // M2-T7 : une arène se verrouille vide et ne se rouvre qu'à la fin de son horloge.
        let arena = table.arena(&kind.0);
        let enemies_inside = enemies
            .iter()
            .filter(|(_, t, _, _)| {
                inside_box(
                    bounds.position,
                    bounds.size,
                    t.translation.truncate(),
                    Fixed::ZERO,
                )
            })
            .count();
        match step {
            RoomState::Dormant if enemies_inside > 0 || arena.is_some() => {
                // Joueurs debout dans la salle, par `GgrsNetId` croissant : le premier est celui
                // dont l'entrée verrouille (porte d'entrée = la sienne la plus proche).
                let mut standing: Vec<(usize, FixedVec2)> = players
                    .iter()
                    .filter(|(_, t, _, _, downed)| {
                        !*downed
                            && inside_box(
                                bounds.position,
                                bounds.size,
                                t.translation.truncate(),
                                inset,
                            )
                    })
                    .map(|(id, t, ..)| (id.0, t.translation.truncate()))
                    .collect();
                if standing.is_empty() {
                    continue;
                }
                standing.sort_by_key(|(id, _)| *id);
                let trigger = standing[0].1;

                // Portes à refermer (ouvertes, au bord de la salle), par `GgrsNetId`.
                let to_close: Vec<_> = order_iter!(doors)
                    .into_iter()
                    .filter(|(_, entity, t, _, closed)| {
                        let p = t.translation.truncate();
                        // Une porte secrète (ouverte par une explosion) ou une porte à clé
                        // (ouverte ou fermée) n'est jamais refermée par le verrouillage d'une
                        // salle voisine : son état est du seul ressort de `secret_wall_system`
                        // (resp. `key_door_system`).
                        !*closed && door_near(bounds, p) && !sealed_door(*entity, p)
                    })
                    .collect();

                // Jamais sur un occupant : un joueur ou un ennemi qui chevauche une porte
                // retarde le verrouillage (réévalué à la frame suivante).
                let occupied = to_close.iter().any(|(_, _, door_t, shape, _)| {
                    let overlaps =
                        |t: &bevy_fixed::fixed_math::FixedTransform3D,
                         collider: Option<&Collider>| {
                            collider.is_some_and(|c| {
                                is_colliding(
                                    &door_t.translation,
                                    &shape.collider,
                                    &t.translation,
                                    c,
                                )
                            })
                        };
                    enemies.iter().any(|(_, t, c, _)| overlaps(t, c))
                        || players.iter().any(|(_, t, c, _, _)| overlaps(t, c))
                });
                if occupied {
                    trace!(
                        "ggrs{{f={} room_lock_deferred room={} kind={}}}",
                        frame.frame,
                        index,
                        kind.0
                    );
                    continue;
                }

                for (_, entity, _, shape, _) in &to_close {
                    commands
                        .entity(*entity)
                        .insert((shape.collider.clone(), shape.layer.clone()));
                }

                // Coop : les joueurs absents sont téléportés à l'entrée (la porte la plus
                // proche du joueur qui a verrouillé), côte à côte.
                let entrance = to_close
                    .iter()
                    .map(|(id, _, t, _, _)| {
                        let p = t.translation.truncate();
                        (p.distance_squared(&trigger), id.0, p)
                    })
                    .min_by_key(|(distance, id, _)| (*distance, *id))
                    .map(|(_, _, p)| {
                        entrance_point(
                            bounds.position,
                            bounds.size,
                            p,
                            Fixed::from_num(ENTRANCE_DEPTH),
                        )
                    });
                if let Some((entrance, tangent)) = entrance {
                    let mut ordered: Vec<_> = players.iter_mut().collect();
                    ordered.sort_by_key(|(id, ..)| id.0);
                    for (k, (id, mut t, _, velocity, _)) in ordered
                        .into_iter()
                        .filter(|(_, t, ..)| {
                            !inside_box(
                                bounds.position,
                                bounds.size,
                                t.translation.truncate(),
                                Fixed::ZERO,
                            )
                        })
                        .enumerate()
                    {
                        let spread = Fixed::from_num(ENTRANCE_SPACING * k as i32);
                        t.translation.x = entrance.x + tangent.x * spread;
                        t.translation.y = entrance.y + tangent.y * spread;
                        if let Some(mut velocity) = velocity {
                            velocity.main = FixedVec2::ZERO;
                            velocity.knockback = FixedVec2::ZERO;
                        }
                        info!(
                            "ggrs{{f={} room_teleport room={} net_id={}}}",
                            frame.frame, index, id.0
                        );
                    }
                }

                states.rooms.insert(level_id.0.clone(), RoomState::Locked);
                if arena.is_some() {
                    arena_clocks.0.insert(level_id.0.clone(), frame.frame);
                }
                changed.send(RoomChanged {
                    frame: frame.frame,
                    room: level_id.0.clone(),
                    kind: kind.0.clone(),
                    from: RoomState::Dormant,
                    to: RoomState::Locked,
                });
                info!(
                    "ggrs{{f={} room_state room={} kind={} state=Locked enemies={}}}",
                    frame.frame, index, kind.0, enemies_inside
                );
            }
            RoomState::Locked if arena.is_some() => {
                // Fin de l'horloge de l'arène, un joueur en vie : les survivants meurent, la
                // salle se rouvre.
                let Some(arena) = arena else { continue };
                let started = arena_clocks
                    .0
                    .get(&level_id.0)
                    .copied()
                    .unwrap_or(frame.frame);
                if frame.frame.saturating_sub(started) < arena.clock || players.is_empty() {
                    continue;
                }
                for (id, t, _, entity) in order_iter!(enemies) {
                    if inside_box(
                        bounds.position,
                        bounds.size,
                        t.translation.truncate(),
                        Fixed::ZERO,
                    ) {
                        info!(
                            "ggrs{{f={} arena_kill room={} net_id={}}}",
                            frame.frame, index, id.0
                        );
                        commands.entity(entity).insert(Death { last_hit_by: None });
                    }
                }
                for (_, entity, t, _, closed) in order_iter!(doors) {
                    let p = t.translation.truncate();
                    if closed && door_near(bounds, p) && !sealed_door(entity, p) {
                        commands
                            .entity(entity)
                            .remove::<(Collider, CollisionLayer)>();
                    }
                }
                arena_clocks.0.remove(&level_id.0);
                states.rooms.insert(level_id.0.clone(), RoomState::Cleared);
                changed.send(RoomChanged {
                    frame: frame.frame,
                    room: level_id.0.clone(),
                    kind: kind.0.clone(),
                    from: RoomState::Locked,
                    to: RoomState::Cleared,
                });
                info!(
                    "ggrs{{f={} room_state room={} kind={} state=Cleared arena=held}}",
                    frame.frame, index, kind.0
                );
            }
            RoomState::Locked if enemies_inside == 0 => {
                for (_, entity, t, _, closed) in order_iter!(doors) {
                    let p = t.translation.truncate();
                    if closed && door_near(bounds, p) && !sealed_door(entity, p) {
                        commands
                            .entity(entity)
                            .remove::<(Collider, CollisionLayer)>();
                    }
                }
                states.rooms.insert(level_id.0.clone(), RoomState::Cleared);
                changed.send(RoomChanged {
                    frame: frame.frame,
                    room: level_id.0.clone(),
                    kind: kind.0.clone(),
                    from: RoomState::Locked,
                    to: RoomState::Cleared,
                });
                info!(
                    "ggrs{{f={} room_state room={} kind={} state=Cleared enemies=0}}",
                    frame.frame, index, kind.0
                );
            }
            _ => {}
        }
    }
}

/// Butin des rochers à butin cassés dans la frame (`game::bombs::RockBroken`, ordre `(y, x)`) : un
/// `ItemPickup` tiré par `items::draw_loot_with` avec un `RollbackRng` **local** de graine
/// `loot_seed(graine, étage, "rock:<salle>:<x>:<y>")`. `<salle>` vient des seules salles de l'étage
/// courant (`CurrentRooms`) : en `Floors` les mondes se superposent, une salle d'un étage inactif
/// donnerait une clé fausse. Pool = celui de l'étage s'il existe, sinon `salle`.
#[allow(clippy::too_many_arguments)]
pub fn rock_loot_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    broken: Res<FrameEvents<game::bombs::RockBroken>>,
    rooms: CurrentRooms,
    items: Res<items::ItemTable>,
    tables: Res<items::LootTables>,
    floors: Res<game::loot::FloorLootTables>,
    unlocks: Res<items::UnlockTable>,
    active: Res<items::RunUnlocks>,
    run_seed: Option<Res<bevy_fixed::rng::RunSeed>>,
    floor: Option<Res<run::FloorState>>,
    mut id_factory: ResMut<GgrsNetIdFactory>,
) {
    if !broken.iter().any(|rock| rock.loot) {
        return;
    }
    let (run_seed, floor) = game::loot::seed_context(&run_seed, &floor);
    let current: Vec<(String, FixedVec2, FixedVec2)> = sorted_rooms(&rooms)
        .into_iter()
        .map(|(id, bounds, _)| (id.0.clone(), bounds.position, bounds.size))
        .collect();
    for rock in broken.iter().filter(|rock| rock.loot) {
        let position = FixedVec2::new(rock.x, rock.y);
        let room = game::bombs::room_of(
            current.iter().map(|(id, p, s)| (id.as_str(), *p, *s)),
            position,
        );
        let key = game::bombs::rock_loot_key(&room, rock.cell);
        let table_id = floors
            .for_floor(floor)
            .filter(|id| tables.0.contains_key(*id))
            .unwrap_or(items::DEFAULT_REWARD_TABLE);
        let Some(table) = tables.0.get(table_id) else {
            warn!("rocher « {key} » : pool de butin « {table_id} » inconnu");
            continue;
        };
        let table = items::unlocked_table(table_id, table, &unlocks, &active);
        let mut rng = bevy_fixed::rng::RollbackRng::new(items::loot_seed(run_seed, floor, &key));
        let Some(item_id) = items::draw_loot_with(
            &table,
            &items,
            None,
            items::Rarity::Common,
            true,
            &unlocks.hidden_items(&active),
            &mut rng,
        ) else {
            warn!("rocher « {key} » : le pool « {table_id} » ne donne rien");
            continue;
        };
        info!(
            "ggrs{{f={} rock_loot key={} item={}}}",
            frame.frame, key, item_id
        );
        let position = FixedVec3::new(rock.x, rock.y, Fixed::ZERO);
        // M3-T3 (§68) : un tas de sous (entrée `Currency` du pool), sinon l'objet.
        if let Some(amount) = items::loot_currency(&item_id) {
            game::shop::spawn_shell_pickup(&mut commands, &mut id_factory, amount, position, 0);
        } else {
            game::items::spawn_item_pickup(
                &mut commands,
                &items,
                item_id,
                position,
                &mut id_factory,
            );
        }
    }
}

/// Mur secret (M3-T4, §69) : une `ExplosionRequest` qui `destroys` et dont le rayon touche une porte
/// fermée d'une salle `door_lock: Secret` (porte de la salle ou jumelle de la salle voisine, par le
/// bord de son collider) ouvre **toutes** les portes de cette salle : leurs colliders disparaissent
/// (le champ de flux relit les portes à chaque mise à jour : les deux salles sont reliées) et la
/// salle entre dans `DoorLocks`. Demandes dans leur ordre d'émission (`GgrsNetId` des bombes), salles
/// par position puis identifiant, portes par `GgrsNetId`. Sans salle secrète : aucun effet.
#[allow(clippy::type_complexity)]
pub fn secret_wall_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    requests: Res<FrameEvents<ExplosionRequest>>,
    table: Res<RoomKindTable>,
    mut locks: ResMut<DoorLocks>,
    rooms: CurrentRooms,
    doors: Query<
        (
            &GgrsNetId,
            Entity,
            &bevy_fixed::fixed_math::FixedTransform3D,
            &Collider,
        ),
        (With<DoorComponent>, Without<Player>, Without<Enemy>),
    >,
) {
    if requests.is_empty() {
        return;
    }
    let secrets: Vec<_> = sorted_rooms(&rooms)
        .into_iter()
        .filter(|(id, _, kind)| is_secret(&table, kind) && !locks.is_open(&id.0))
        .collect();
    if secrets.is_empty() {
        return;
    }
    let mut opened: std::collections::BTreeSet<String> = Default::default();
    for request in requests.iter().filter(|request| request.destroys) {
        let center = FixedVec2::new(request.x, request.y);
        for (level_id, bounds, _) in &secrets {
            if opened.contains(&level_id.0) {
                continue;
            }
            let touched = order_iter!(doors).into_iter().any(|(_, _, t, collider)| {
                let p = t.translation.truncate();
                if !door_near(bounds, p) {
                    return false;
                }
                let at = FixedVec2::new(p.x + collider.offset.x, p.y + collider.offset.y);
                match collider.shape {
                    game::collider::ColliderShape::Rectangle { width, height } => {
                        let two = Fixed::from_num(2);
                        rect_within_radius(center, at, width / two, height / two, request.radius)
                    }
                    game::collider::ColliderShape::Circle { radius } => {
                        rect_within_radius(center, at, radius, radius, request.radius)
                    }
                }
            });
            if !touched {
                continue;
            }
            for (_, entity, t, _) in order_iter!(doors) {
                if door_near(bounds, t.translation.truncate()) {
                    commands
                        .entity(entity)
                        .remove::<(Collider, CollisionLayer)>();
                }
            }
            locks.0.insert(level_id.0.clone());
            opened.insert(level_id.0.clone());
            info!(
                "ggrs{{f={} secret_wall_open room={} by={}}}",
                frame.frame, level_id.0, request.id
            );
        }
    }
}

/// Porte à clé (M3-T5, §70) : une `InteractionEvent` `KeyDoor` (ordre d'émission, `GgrsNetId` de
/// l'interacteur) sur une porte scellée d'une salle `door_lock: Key(n)` de l'étage courant l'ouvre si
/// l'interacteur a au moins `n` clés (`game::loot::KEY_ITEM`, la clé est à celui qui la possède) : `n`
/// clés sont consommées, les colliders (et l'`Interactable`) de **toutes** les portes scellées de la
/// salle (la porte de la salle et sa jumelle, reliées par le plan : `paired_door`) disparaissent, la
/// salle entre dans `DoorLocks`. Le champ de flux relit les portes à chaque mise à jour : les deux
/// salles sont reliées (comme `secret_wall_system`). Sinon la porte reste fermée et rien n'est
/// consommé. Une porte près de deux salles à clé appartient à la première (position puis
/// identifiant).
#[allow(clippy::type_complexity)]
pub fn key_door_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    events: Res<FrameEvents<InteractionEvent>>,
    table: Res<RoomKindTable>,
    mut locks: ResMut<DoorLocks>,
    floor: Option<Res<run::FloorState>>,
    rooms: CurrentRooms,
    doors: Query<
        (
            &GgrsNetId,
            Entity,
            &bevy_fixed::fixed_math::FixedTransform3D,
            &DoorComponent,
        ),
        (With<Collider>, Without<Player>, Without<Enemy>),
    >,
    mut players: Query<(&GgrsNetId, Option<&mut items::Inventory>), With<Player>>,
) {
    if !events
        .iter()
        .any(|event| event.interaction_type == InteractionType::KeyDoor)
    {
        return;
    }
    let floor_index = floor.as_ref().map_or(0, |floor| floor.index);
    let typed = sorted_rooms(&rooms);
    for event in events
        .iter()
        .filter(|event| event.interaction_type == InteractionType::KeyDoor)
    {
        let Ok((_, _, door_t, _)) = doors.get(event.interactable) else {
            continue;
        };
        let p = door_t.translation.truncate();
        let Some((level_id, bounds, cost)) = typed.iter().find_map(|(id, bounds, kind)| {
            let cost = key_lock_cost(&table, kind, floor_index)?;
            door_near(bounds, p).then_some((*id, *bounds, cost))
        }) else {
            continue;
        };
        if locks.is_open(&level_id.0) {
            continue;
        }
        let Some((_, mut inventory)) = players
            .iter_mut()
            .find(|(id, _)| id.0 == event.interactor_net_id.0)
        else {
            continue;
        };
        let keys = inventory
            .as_deref()
            .map_or(0, |inventory| inventory.consumable(game::loot::KEY_ITEM));
        if keys < cost {
            info!(
                "ggrs{{f={} key_door_refused room={} by={} keys={}/{}}}",
                frame.frame, level_id.0, event.interactor_net_id.0, keys, cost
            );
            continue;
        }
        if let Some(stock) = inventory
            .as_deref_mut()
            .and_then(|inventory| inventory.consumables.get_mut(game::loot::KEY_ITEM))
        {
            *stock -= cost;
        }
        for (_, entity, t, door) in order_iter!(doors) {
            if door.config.paired_door.is_some() && door_near(bounds, t.translation.truncate()) {
                commands
                    .entity(entity)
                    .remove::<(Collider, CollisionLayer, Interactable)>();
            }
        }
        locks.0.insert(level_id.0.clone());
        info!(
            "ggrs{{f={} key_door_open room={} by={}}}",
            frame.frame, level_id.0, event.interactor_net_id.0
        );
    }
}

/// Paramètres d'apparition d'un personnage en cours de partie (renforts de boss, vagues
/// d'arène, M2-T7).
#[derive(SystemParam)]
pub struct SpawnParams<'w> {
    global_assets: Res<'w, GlobalAsset>,
    collision_settings: Res<'w, CollisionSettings>,
    weapons_asset: Res<'w, Assets<WeaponsConfig>>,
    melee_weapons_asset: Res<'w, Assets<MeleeWeaponsConfig>>,
    characters_asset: Res<'w, Assets<CharacterConfig>>,
    balance: Res<'w, game::balance::ResolvedBalance>,
    difficulty: game::clock::DifficultyReader<'w>,
    id_factory: ResMut<'w, GgrsNetIdFactory>,
    rng: ResMut<'w, RngStreams>,
    flow: Res<'w, FlowFieldCache>,
    /// M3-T7 : étage courant (contexte de `chance` des variantes).
    floor: Option<Res<'w, run::FloorState>>,
}

/// Marge (en pixels) gardée entre une apparition et les bords de la salle.
const SPAWN_INSET: i32 = 40;
/// Tirages de position avant de se rabattre sur le centre de la salle.
const SPAWN_TRIES: u32 = 16;

impl SpawnParams<'_> {
    /// Fait apparaître `count` personnages `id` dans la salle `level` : positions tirées du flux
    /// RNG `room:<niveau>` (case libre pour le profil `Ground`), repli sur le centre.
    fn spawn_in_room(
        &mut self,
        commands: &mut Commands,
        level: &str,
        bounds: &RoomBounds,
        id: &str,
        count: u32,
    ) {
        let Some(health_max) = self.balance.health_max_by_character.get(id).copied() else {
            warn!("renforts : personnage « {id} » sans santé résolue (voir `alacod lint`), ignoré");
            return;
        };
        let health_max = game::clock::scale(health_max, self.difficulty.current());
        let inset = Fixed::from_num(SPAWN_INSET);
        let width = (bounds.size.x - inset * 2).to_num::<i32>().max(1) as u32;
        let height = (bounds.size.y - inset * 2).to_num::<i32>().max(1) as u32;
        let stream = room_rng_stream(level);
        for _ in 0..count {
            let mut position = FixedVec2::new(
                bounds.position.x + bounds.size.x / 2,
                bounds.position.y + bounds.size.y / 2,
            );
            for _ in 0..SPAWN_TRIES {
                let rng = self.rng.get_mut(&stream);
                let x = bounds.position.x + inset + Fixed::from_num(rng.next_u32_range(0, width));
                let y = bounds.position.y + inset + Fixed::from_num(rng.next_u32_range(0, height));
                let candidate = FixedVec2::new(x, y);
                if !self.flow.is_blocked(
                    &GridPos::from_fixed(candidate),
                    game::character::enemy::ai::navigation::NavProfile::Ground,
                ) {
                    position = candidate;
                    break;
                }
            }
            spawn_enemy(
                id.to_string(),
                FixedVec3::new(position.x, position.y, Fixed::ZERO),
                commands,
                &self.weapons_asset,
                &self.melee_weapons_asset,
                &self.characters_asset,
                &self.global_assets,
                &self.collision_settings,
                &mut self.id_factory,
                Team::Enemies,
                health_max,
                self.rng.run_seed,
                None,
                self.difficulty
                    .variant_context_at(self.floor.as_deref().map_or(0, |floor| floor.index)),
            );
        }
    }
}

/// Renforts d'une frise de boss (`SpawnEnemies`, M2-T7) : exécute les demandes de la frame dans
/// la salle qui contient le boss, dans l'ordre d'émission (`GgrsNetId` croissant du boss).
pub fn boss_reinforcements_system(
    mut commands: Commands,
    mut requests: ResMut<FrameEvents<ReinforcementRequest>>,
    rooms: CurrentRooms,
    mut spawn: SpawnParams,
) {
    let requests: Vec<ReinforcementRequest> = requests.iter().cloned().collect();
    for request in requests {
        let at = FixedVec2::new(request.x, request.y);
        let Some((level, bounds, _)) = sorted_rooms(&rooms)
            .into_iter()
            .find(|(_, bounds, _)| inside_box(bounds.position, bounds.size, at, Fixed::ZERO))
        else {
            warn!(
                "renforts : le boss {} n'est dans aucune salle, {} « {} » ignoré",
                request.boss, request.count, request.id
            );
            continue;
        };
        spawn.spawn_in_room(&mut commands, &level.0, bounds, &request.id, request.count);
    }
}

/// Vagues internes d'une arène verrouillée (M2-T7) : à `at` frames du verrouillage, les
/// personnages de la vague apparaissent. Aucun curseur : la vague due se déduit de l'horloge.
pub fn arena_waves_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    table: Res<RoomKindTable>,
    states: Res<RoomStates>,
    clocks: Res<ArenaClocks>,
    rooms: CurrentRooms,
    mut spawn: SpawnParams,
) {
    for (level, bounds, kind) in sorted_rooms(&rooms) {
        let Some(arena) = table.arena(&kind.0) else {
            continue;
        };
        if states.get(&level.0) != RoomState::Locked {
            continue;
        }
        let Some(started) = clocks.0.get(&level.0) else {
            continue;
        };
        let elapsed = frame.frame.saturating_sub(*started);
        for wave in arena.waves.iter().filter(|wave| wave.at == elapsed) {
            for (id, count) in &wave.enemies {
                info!(
                    "ggrs{{f={} arena_wave room={} at={} id={} count={}}}",
                    frame.frame, level.0, wave.at, id, count
                );
                spawn.spawn_in_room(&mut commands, &level.0, bounds, id, *count);
            }
        }
    }
}

/// Mort d'un boss (M2-T7) : les autres ennemis de sa salle meurent (renforts compris), pour que
/// la salle passe `Cleared` dans la frame (`room_lock_system`) et que les portes se rouvrent.
#[allow(clippy::type_complexity)]
pub fn boss_room_clear_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    defeated: Res<FrameEvents<behaviors::BossDefeated>>,
    rooms: CurrentRooms,
    enemies: Query<
        (
            &GgrsNetId,
            Entity,
            &bevy_fixed::fixed_math::FixedTransform3D,
            Has<Death>,
        ),
        (With<Enemy>, Without<Player>),
    >,
) {
    for event in defeated.iter() {
        let at = FixedVec2::new(event.x, event.y);
        let Some((_, bounds, _)) = sorted_rooms(&rooms)
            .into_iter()
            .find(|(_, bounds, _)| inside_box(bounds.position, bounds.size, at, Fixed::ZERO))
        else {
            continue;
        };
        for (net_id, entity, t, dead) in order_iter!(enemies) {
            if !dead
                && net_id.0 != event.net_id
                && inside_box(
                    bounds.position,
                    bounds.size,
                    t.translation.truncate(),
                    Fixed::ZERO,
                )
            {
                info!(
                    "ggrs{{f={} boss_room_clear net_id={}}}",
                    frame.frame, net_id.0
                );
                commands.entity(entity).insert(Death { last_hit_by: None });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use world::RoomKindDef;

    /// La salle secrète n'est dessinée qu'une fois son mur ouvert ; les autres types, toujours.
    #[test]
    fn la_salle_secrete_n_apparait_sur_la_minicarte_qu_ouverte() {
        let table = RoomKindTable(
            [
                (
                    "secrete".to_string(),
                    RoomKindDef {
                        door_lock: Some(DoorLock::Secret),
                        ..Default::default()
                    },
                ),
                (
                    "tresor".to_string(),
                    RoomKindDef {
                        door_lock: Some(DoorLock::Key(1)),
                        ..Default::default()
                    },
                ),
                ("combat".to_string(), RoomKindDef::default()),
            ]
            .into(),
        );
        let secrete = RoomKind("secrete".into());
        let mut locks = DoorLocks::default();
        assert!(hidden_from_minimap(
            Some(&table),
            Some(&locks),
            "s1",
            &secrete
        ));
        assert!(hidden_from_minimap(Some(&table), None, "s1", &secrete));
        locks.0.insert("s1".into());
        assert!(!hidden_from_minimap(
            Some(&table),
            Some(&locks),
            "s1",
            &secrete
        ));
        // une autre salle secrète, fermée, reste cachée
        assert!(hidden_from_minimap(
            Some(&table),
            Some(&locks),
            "s2",
            &secrete
        ));
        for kind in ["combat", "tresor", "inconnu"] {
            assert!(!hidden_from_minimap(
                Some(&table),
                Some(&locks),
                "x",
                &RoomKind(kind.into())
            ));
        }
        assert!(!hidden_from_minimap(None, None, "s1", &secrete));
    }
}
