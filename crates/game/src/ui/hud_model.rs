//! Valeurs du HUD du joueur affiché (T1.18, `docs/conventions.md` §32) : présentation seule.
//!
//! [`hud_values`] (pure, testée sans rendu) compose [`HudPlayerValues`] à partir de l'état du
//! joueur ; [`player_source_text`] en tire le texte de chaque source. [`HudModelPlugin`] (aussi
//! en headless) tient à jour [`HudSnapshot`] en `Update`, lu par le rendu du HUD
//! (`ui::hud`) et par l'attente de scénario `HudText`. Lecture seule de la simulation : aucune
//! trace ne change.
//!
//! Sources du joueur : `health`, `ammo`, `weapon`, `currency` (avant T1.18), `rads`, `level`,
//! `ammo_by_type`, `statuses`, `floor` (T1.18). D22 : sans valeur, texte vide (pas même le
//! préfixe).

use std::collections::{BTreeMap, BTreeSet};

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_fixed::fixed_math::{Fixed, FixedTransform3D, FixedVec2};
use combat::inventory::AmmoReserves;
use combat::status::Statuses;
use items::{Inventory, ItemKind, ItemTable};
use run::floors::FloorState;
use sim_core::ammo::AmmoType;
use utils::frame::FrameCount;
use world::{RoomState, RoomStates};

use crate::camera::CameraFollowOverride;
use crate::character::health::Health;
use crate::character::player::{LocalPlayer, Player};
use crate::core::SIM_FPS;
use crate::effects_runtime::Gauges;
use crate::progression::{Level, ProgressionDef, ProgressionTable};
use crate::weapons::{WeaponInventory, WeaponModesState, WeaponState};

/// Sources lues par [`player_source_text`] (celles du joueur).
pub const PLAYER_SOURCES: &[&str] = &[
    "health",
    "ammo",
    "weapon",
    "currency",
    "rads",
    "level",
    "ammo_by_type",
    "statuses",
    "floor",
    "blanks",
    "keys",
    "bombs",
    "shells",
    "passives",
    "active_item",
    "active_charge",
    "minimap",
];

/// Joueur dont le HUD affiche l'état (T2.12) : celui que la caméra suit de force
/// (`CameraFollowOverride`, `play_scenario --follow <handle>`), sinon le joueur local de plus
/// petit handle.
#[derive(SystemParam)]
pub(crate) struct HudPlayer<'w, 's> {
    players: Query<'w, 's, (Entity, &'static Player, Has<LocalPlayer>)>,
    follow_override: Option<Res<'w, CameraFollowOverride>>,
}

impl HudPlayer<'_, '_> {
    /// Handle du joueur affiché (M2-T13 : son profil, son marchand).
    pub(crate) fn handle(&self) -> Option<usize> {
        let entity = self.entity()?;
        self.players
            .get(entity)
            .ok()
            .map(|(_, player, _)| player.handle)
    }

    pub(crate) fn entity(&self) -> Option<Entity> {
        if let Some(handle) = self.follow_override.as_deref().map(|o| o.0) {
            if let Some((entity, _, _)) = self.players.iter().find(|(_, p, _)| p.handle == handle) {
                return Some(entity);
            }
        }
        self.players
            .iter()
            .filter(|(_, _, local)| *local)
            .min_by_key(|(_, player, _)| player.handle)
            .map(|(entity, _, _)| entity)
    }
}

/// Rads du joueur : valeur de la jauge de la progression et seuil du prochain niveau.
#[derive(Debug, Clone, PartialEq)]
pub struct RadsValue {
    pub gauge: String,
    pub value: Fixed,
    /// Seuil du niveau courant (0 au niveau 0).
    pub previous: Fixed,
    /// Seuil du prochain niveau ; `None` au dernier niveau.
    pub next: Option<Fixed>,
}

impl RadsValue {
    /// Fraction vers le prochain seuil, dans `[0, 1]` ; pleine au dernier niveau.
    pub fn fraction(&self) -> f32 {
        let Some(next) = self.next else {
            return 1.0;
        };
        let span = (next - self.previous).to_num::<f32>();
        if span <= 0.0 {
            return 1.0;
        }
        ((self.value - self.previous).to_num::<f32>() / span).clamp(0.0, 1.0)
    }
}
/// Géométrie du seul étage courant, fournie par l'adaptateur de carte en `Update`.
/// Les salles activées se déduisent de `RoomStates` ; les autres visites (boutique,
/// récompense : jamais activées) viennent de [`MinimapVisits`].
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct HudRoomGeometry(pub Vec<HudRoomBounds>);

/// D72 : mémoire de visite de la minicarte, **présentation seule** (hors rollback, hors trace) :
/// ids des salles où un joueur local est entré, pour l'étage dont la géométrie est `geometry`.
/// Remise à zéro quand l'étage change ou disparaît (nouvel étage, restart). Un rollback ne
/// la défait pas : une salle entrée sur une frame annulée reste visitée, l'exploration d'un
/// joueur n'étant pas un fait de simulation.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct MinimapVisits {
    pub geometry: Vec<String>,
    pub visited: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HudRoomBounds {
    pub id: String,
    pub kind: String,
    pub position: FixedVec2,
    pub size: FixedVec2,
}

/// Rectangle normalisé dans l'étage complet : le cadrage ne change pas à l'exploration.
#[derive(Debug, Clone, PartialEq)]
pub struct HudMapRoom {
    pub id: String,
    pub kind: String,
    pub rect: [f32; 4],
    pub current: bool,
    pub visited: bool,
    pub state: RoomState,
    pub boss: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HudActiveItem {
    pub name: String,
    pub charge: u32,
    pub required: Option<u32>,
}

/// Projette les bornes monde (y vers le haut) en coordonnées UI (y vers le bas).
/// Les salles dormantes non courantes restent des silhouettes ; `depart` est le point
/// d'entrée, et les autres salles visitées sont celles activées (`Locked`/`Cleared`) ou
/// mémorisées par [`MinimapVisits`].
pub fn minimap_rooms(
    geometry: &HudRoomGeometry,
    states: &RoomStates,
    visits: &BTreeSet<String>,
    player: Option<FixedVec2>,
) -> Vec<HudMapRoom> {
    let mut rooms: Vec<_> = geometry
        .0
        .iter()
        .filter(|room| room.size.x > Fixed::ZERO && room.size.y > Fixed::ZERO)
        .collect();
    rooms.sort_by(|a, b| {
        (a.position.x, a.position.y, &a.id).cmp(&(b.position.x, b.position.y, &b.id))
    });
    let Some(first) = rooms.first() else {
        return Vec::new();
    };
    let mut min = first.position;
    let mut max = first.position + first.size;
    for room in &rooms {
        min.x = min.x.min(room.position.x);
        min.y = min.y.min(room.position.y);
        max.x = max.x.max(room.position.x + room.size.x);
        max.y = max.y.max(room.position.y + room.size.y);
    }
    let width = (max.x - min.x).to_num::<f32>();
    let height = (max.y - min.y).to_num::<f32>();
    // Même échelle sur les deux axes, centrée dans un carré.
    let span = width.max(height);
    let current = player.and_then(|p| {
        rooms
            .iter()
            .find(|room| world::inside_box(room.position, room.size, p, Fixed::ZERO))
            .map(|room| room.id.as_str())
    });
    rooms
        .into_iter()
        .map(|room| {
            let state = states.get(&room.id);
            let is_current = current.is_some_and(|current| current == room.id);
            HudMapRoom {
                id: room.id.clone(),
                kind: room.kind.clone(),
                rect: [
                    ((room.position.x - min.x).to_num::<f32>() + (span - width) / 2.0) / span,
                    ((max.y - room.position.y - room.size.y).to_num::<f32>()
                        + (span - height) / 2.0)
                        / span,
                    room.size.x.to_num::<f32>() / span,
                    room.size.y.to_num::<f32>() / span,
                ],
                current: is_current,
                visited: is_current
                    || room.kind == "depart"
                    || state != RoomState::Dormant
                    || visits.contains(&room.id),
                state,
                boss: room.kind == "boss",
            }
        })
        .collect()
}

/// Valeurs du joueur du HUD ; `None`/vide partout quand ce joueur n'existe plus (mort : son
/// entité est détruite) ou n'a pas la donnée (progression inactive, pas d'arme…).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct HudPlayerValues {
    /// `(actuelle, max)`.
    pub health: Option<(i32, i32)>,
    pub currency: Option<u32>,
    /// `(nom, munitions du chargeur du mode actif, réserve du type de munition)`.
    pub weapon: Option<(String, Option<u32>, u32)>,
    pub rads: Option<RadsValue>,
    pub level: Option<u32>,
    /// Réserve de chaque munition `Custom` (ordre des ids).
    pub ammo_by_type: Vec<(String, u32)>,
    /// `(id, piles, secondes restantes arrondies au-dessus)`, dans l'ordre de pose.
    pub statuses: Vec<(String, u32, u32)>,
    /// Index du niveau (mode `Floors` seulement).
    pub floor: Option<u32>,
    /// `(blanks, clés, bombes)`, absents hors jeu à objets ou sans joueur ; zéro avant ramassage.
    pub consumables: Option<[u32; 3]>,
    /// `(id, nom)` dans l'ordre de ramassage ; le rendu utilise l'id pour les icônes.
    pub passives: Vec<(String, String)>,
    pub active_item: Option<HudActiveItem>,
    pub minimap: Vec<HudMapRoom>,
}

/// L'état du joueur affiché, déjà lu dans l'ECS.
#[derive(Default)]
pub struct HudPlayerInput<'a> {
    pub health: Option<(Fixed, Fixed)>,
    pub currency: Option<u32>,
    pub weapon: Option<(String, Option<u32>, u32)>,
    pub gauges: Option<&'a Gauges>,
    pub level: Option<u32>,
    pub progression: Option<&'a ProgressionDef>,
    pub reserves: Option<&'a AmmoReserves>,
    pub statuses: Option<&'a Statuses>,
    pub floor: Option<&'a FloorState>,
    pub player_present: bool,
    pub items: Option<&'a Inventory>,
    pub item_table: Option<&'a ItemTable>,
    pub room_geometry: Option<&'a HudRoomGeometry>,
    pub room_states: Option<&'a RoomStates>,
    pub visits: Option<&'a BTreeSet<String>>,
    pub position: Option<FixedVec2>,
    pub frame: u32,
}

/// Les valeurs du HUD (pure).
pub fn hud_values(input: &HudPlayerInput) -> HudPlayerValues {
    let rads = match (input.progression, input.gauges) {
        (Some(def), Some(gauges)) => gauges.0.get(&def.gauge).map(|gauge| {
            let level = input.level.unwrap_or(0) as usize;
            RadsValue {
                gauge: def.gauge.clone(),
                value: gauge.value,
                previous: level
                    .checked_sub(1)
                    .and_then(|i| def.levels.get(i).copied())
                    .unwrap_or(Fixed::ZERO),
                next: def.levels.get(level).copied(),
            }
        }),
        _ => None,
    };
    let ammo_by_type = input
        .reserves
        .map(|reserves| {
            reserves
                .0
                .iter()
                .filter_map(|(ammo, amount)| match ammo {
                    AmmoType::Custom(id) => Some((id.clone(), *amount)),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();
    let statuses = input
        .statuses
        .map(|statuses| {
            statuses
                .0
                .iter()
                .map(|entry| {
                    let frames = entry.expires_at_frame.saturating_sub(input.frame);
                    let seconds = frames.div_ceil(SIM_FPS as u32);
                    (entry.id.clone(), entry.stacks, seconds)
                })
                .collect()
        })
        .unwrap_or_default();
    let item_name = |id: &str| {
        input
            .item_table
            .and_then(|table| table.0.get(id))
            .map_or_else(|| id.to_string(), |def| def.name.clone())
    };
    HudPlayerValues {
        health: input
            .health
            .map(|(current, max)| (current.to_num::<i32>(), max.to_num::<i32>())),
        currency: input.currency,
        weapon: input.weapon.clone(),
        rads,
        level: input.level,
        ammo_by_type,
        statuses,
        floor: input
            .floor
            .filter(|floor| floor.anchor.is_some())
            .map(|floor| floor.index),
        consumables: (input.player_present
            && input.item_table.is_some_and(|table| !table.0.is_empty()))
        .then(|| ["blank", "key", crate::loot::BOMB_ITEM].map(|id| input.items.map_or(0, |items| items.consumable(id)))),
        passives: input
            .items
            .map(|items| {
                items
                    .passives
                    .iter()
                    .map(|id| (id.clone(), item_name(id)))
                    .collect()
            })
            .unwrap_or_default(),
        active_item: input
            .items
            .and_then(|items| items.active.as_ref())
            .map(|slot| HudActiveItem {
                name: item_name(&slot.item),
                charge: slot.charge,
                required: input
                    .item_table
                    .and_then(|table| table.0.get(&slot.item))
                    .and_then(|def| match def.kind {
                        ItemKind::Active(charge) => Some(charge.required()),
                        _ => None,
                    }),
            }),
        minimap: input
            .room_geometry
            .zip(input.room_states)
            .map(|(geometry, states)| {
                let none = BTreeSet::new();
                minimap_rooms(
                    geometry,
                    states,
                    input.visits.unwrap_or(&none),
                    input.position,
                )
            })
            .unwrap_or_default(),
    }
}

/// Texte d'une source du joueur (voir [`PLAYER_SOURCES`]) ; `None` pour une autre source.
/// D22 : sans valeur, **rien**, pas même le préfixe.
pub fn player_source_text(source: &str, prefix: &str, values: &HudPlayerValues) -> Option<String> {
    let number = effects::describe::number;
    let value = match source {
        "health" => values
            .health
            .map(|(current, max)| format!("{current}/{max}")),
        "ammo" => values
            .weapon
            .as_ref()
            .and_then(|(_, mag, reserve)| mag.map(|mag| format!("{mag} | {reserve}"))),
        "weapon" => values.weapon.as_ref().map(|(name, _, _)| name.clone()),
        "currency" => values.currency.map(|amount| amount.to_string()),
        "rads" => values.rads.as_ref().map(|rads| match rads.next {
            Some(next) => format!("{} / {} {}", number(rads.value), number(next), rads.gauge),
            None => format!("{} {}", number(rads.value), rads.gauge),
        }),
        "level" => values.level.map(|level| format!("Niv. {level}")),
        "ammo_by_type" => (!values.ammo_by_type.is_empty()).then(|| {
            values
                .ammo_by_type
                .iter()
                .map(|(id, amount)| format!("{id} {amount}"))
                .collect::<Vec<_>>()
                .join("\n")
        }),
        "statuses" => (!values.statuses.is_empty()).then(|| {
            values
                .statuses
                .iter()
                .map(|(id, stacks, seconds)| match stacks {
                    0 | 1 => format!("{id} · {seconds} s"),
                    _ => format!("{id} ×{stacks} · {seconds} s"),
                })
                .collect::<Vec<_>>()
                .join("\n")
        }),
        "floor" => values.floor.map(|index| format!("Étage {}", index + 1)),
        "blanks" => values.consumables.map(|counts| counts[0].to_string()),
        "keys" => values.consumables.map(|counts| counts[1].to_string()),
        // M3-T3 (§68) : compteur de bombes du joueur affiché (par joueur, comme les clés).
        "bombs" => values.consumables.map(|counts| counts[2].to_string()),
        // Les douilles de boutique créditent Currency, pas un consommable `shell`.
        "shells" => values
            .consumables
            .and(values.currency)
            .map(|amount| amount.to_string()),
        "passives" => (!values.passives.is_empty()).then(|| {
            values
                .passives
                .iter()
                .map(|(_, name)| name.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        }),
        "active_item" => values.active_item.as_ref().map(|item| item.name.clone()),
        "active_charge" => values.active_item.as_ref().and_then(|item| {
            item.required
                .filter(|required| *required > 0)
                .map(|required| {
                    let ready = if item.charge >= required {
                        " · prêt"
                    } else {
                        ""
                    };
                    format!("{} / {required}{ready}", item.charge.min(required))
                })
        }),
        "minimap" => (!values.minimap.is_empty()).then(|| {
            values
                .minimap
                .iter()
                .map(|room| {
                    let status = match room.state {
                        RoomState::Locked => "verrouillée",
                        RoomState::Cleared => "nettoyée",
                        RoomState::Dormant if room.visited => "visitée",
                        RoomState::Dormant => "inexplorée",
                    };
                    format!(
                        "{}{} · {status}{}",
                        room.kind,
                        if room.boss { " · boss" } else { "" },
                        if room.current { " · actuelle" } else { "" },
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        }),
        _ => return None,
    };
    Some(value.map_or_else(String::new, |value| format!("{prefix}{value}")))
}

/// Ce que le HUD du joueur affiché montre cette frame : texte de chaque source du joueur (sans
/// préfixe, vide sans valeur) et remplissage des barres (`health`, `rads`, dans `[0, 1]`).
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct HudSnapshot {
    pub frame: u32,
    pub texts: BTreeMap<String, String>,
    pub bars: BTreeMap<String, f32>,
    pub values: HudPlayerValues,
}

/// Snapshot du HUD, en headless comme à l'écran.
pub struct HudModelPlugin;

/// L'adaptateur de carte fournit la géométrie avant la composition du snapshot.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HudModelSet;

impl Plugin for HudModelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HudSnapshot>()
            .init_resource::<HudRoomGeometry>()
            .init_resource::<MinimapVisits>()
            .add_systems(
                Update,
                (track_minimap_visits, update_hud_snapshot)
                    .chain()
                    .in_set(HudModelSet),
            );
    }
}

/// Ce que [`update_hud_snapshot`] lit du joueur affiché.
#[derive(SystemParam)]
pub(crate) struct HudPlayerState<'w, 's> {
    healths: Query<'w, 's, &'static Health, With<Player>>,
    currencies: Query<'w, 's, &'static run::currency::Currency>,
    inventories: Query<'w, 's, (&'static WeaponInventory, &'static AmmoReserves)>,
    items: Query<'w, 's, &'static Inventory>,
    positions: Query<'w, 's, &'static FixedTransform3D, With<Player>>,
    item_table: Option<Res<'w, ItemTable>>,
    geometry: Res<'w, HudRoomGeometry>,
    room_states: Option<Res<'w, RoomStates>>,
    visits: Res<'w, MinimapVisits>,
    weapons: Query<'w, 's, (&'static WeaponState, &'static WeaponModesState)>,
    progress: Query<
        'w,
        's,
        (
            Option<&'static Gauges>,
            Option<&'static Level>,
            Option<&'static Statuses>,
        ),
    >,
}

/// D72 : ajoute à [`MinimapVisits`] la salle où se trouve chaque joueur local, et repart de
/// zéro quand la géométrie de l'étage change.
pub(crate) fn track_minimap_visits(
    mut visits: ResMut<MinimapVisits>,
    geometry: Res<HudRoomGeometry>,
    players: Query<&FixedTransform3D, (With<Player>, With<LocalPlayer>)>,
) {
    let ids: Vec<String> = geometry.0.iter().map(|room| room.id.clone()).collect();
    if visits.geometry != ids {
        visits.geometry = ids;
        visits.visited.clear();
    }
    for transform in &players {
        let position = FixedVec2::new(transform.translation.x, transform.translation.y);
        for room in &geometry.0 {
            if !visits.visited.contains(&room.id)
                && world::inside_box(room.position, room.size, position, Fixed::ZERO)
            {
                visits.visited.insert(room.id.clone());
            }
        }
    }
}

pub(crate) fn update_hud_snapshot(
    mut snapshot: ResMut<HudSnapshot>,
    frame: Res<FrameCount>,
    hud_player: HudPlayer,
    state: HudPlayerState,
    table: Option<Res<ProgressionTable>>,
    floor: Option<Res<FloorState>>,
) {
    let player = hud_player.entity();
    let weapon =
        player
            .and_then(|e| state.inventories.get(e).ok())
            .and_then(|(inventory, reserves)| {
                let (entity, weapon) = inventory.weapons.get(inventory.active_weapon_index)?;
                let mode = state
                    .weapons
                    .get(*entity)
                    .ok()
                    .and_then(|(s, modes)| modes.modes.get(&s.active_mode).cloned());
                Some((
                    weapon.config.name.clone(),
                    mode.map(|m| m.mag_ammo),
                    reserves.get(&weapon.config.ammo_type),
                ))
            });
    let (gauges, level, statuses) = player
        .and_then(|e| state.progress.get(e).ok())
        .unwrap_or((None, None, None));
    let input = HudPlayerInput {
        health: player
            .and_then(|e| state.healths.get(e).ok())
            .map(|h| (h.current, h.max)),
        currency: player
            .and_then(|e| state.currencies.get(e).ok())
            .map(|c| c.0),
        weapon,
        gauges,
        level: level.map(|l| l.0),
        progression: table.as_deref().and_then(|t| t.active.as_ref()),
        reserves: player
            .and_then(|e| state.inventories.get(e).ok())
            .map(|(_, reserves)| reserves),
        statuses,
        floor: floor.as_deref(),
        player_present: player.is_some(),
        items: player.and_then(|e| state.items.get(e).ok()),
        item_table: state.item_table.as_deref(),
        room_geometry: Some(&state.geometry),
        room_states: state.room_states.as_deref(),
        visits: Some(&state.visits.visited),
        position: player
            .and_then(|e| state.positions.get(e).ok())
            .map(|transform| FixedVec2::new(transform.translation.x, transform.translation.y)),
        frame: frame.frame,
    };
    let values = hud_values(&input);
    let next = snapshot_of(frame.frame, values);
    if *snapshot != next {
        *snapshot = next;
    }
}

/// Le snapshot de `values` (pur).
pub fn snapshot_of(frame: u32, values: HudPlayerValues) -> HudSnapshot {
    let texts = PLAYER_SOURCES
        .iter()
        .filter_map(|source| {
            player_source_text(source, "", &values).map(|text| (source.to_string(), text))
        })
        .collect();
    let mut bars = BTreeMap::new();
    if let Some((current, max)) = values.health {
        let ratio = if max > 0 {
            current as f32 / max as f32
        } else {
            0.0
        };
        bars.insert("health".to_string(), ratio.clamp(0.0, 1.0));
    }
    if let Some(rads) = &values.rads {
        bars.insert("rads".to_string(), rads.fraction());
    }
    if let Some(item) = &values.active_item {
        if let Some(required) = item.required.filter(|required| *required > 0) {
            bars.insert(
                "active_charge".to_string(),
                (item.charge as f32 / required as f32).clamp(0.0, 1.0),
            );
        }
    }
    HudSnapshot {
        frame,
        texts,
        bars,
        values,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use combat::status::{StatusDef, StatusEntry};

    fn fx(v: f32) -> Fixed {
        Fixed::from_num(v)
    }

    fn def() -> ProgressionDef {
        ProgressionDef {
            id: "run".into(),
            gauge: "rads".into(),
            per_kill: fx(1.0),
            levels: vec![fx(3.0), fx(8.0)],
            choices: 3,
            choice_frames: 600,
            choice_timing: content::registry::MutationTiming::Immediate,
            pool: Vec::new(),
            weapon_pool: Vec::new(),
            weapon_drop_chance: Fixed::ZERO,
        }
    }

    fn gauges(value: f32) -> Gauges {
        let mut gauges = Gauges::default();
        let mut gauge = crate::progression::new_gauge();
        gauge.value = fx(value);
        gauges.0.insert("rads".into(), gauge);
        gauges
    }

    #[test]
    fn rads_vers_le_prochain_seuil() {
        let def = def();
        let g = gauges(5.0);
        let values = hud_values(&HudPlayerInput {
            gauges: Some(&g),
            level: Some(1),
            progression: Some(&def),
            ..default()
        });
        let rads = values.rads.clone().expect("rads");
        assert_eq!(rads.previous, fx(3.0));
        assert_eq!(rads.next, Some(fx(8.0)));
        assert!((rads.fraction() - 0.4).abs() < 1e-4, "(5 − 3) / (8 − 3)");
        assert_eq!(
            player_source_text("rads", "", &values).as_deref(),
            Some("5 / 8 rads")
        );
        assert_eq!(
            player_source_text("level", "", &values).as_deref(),
            Some("Niv. 1")
        );
        // Niveau 0 : depuis 0 ; dernier niveau : barre pleine, pas de seuil affiché.
        let g = gauges(1.5);
        let start = hud_values(&HudPlayerInput {
            gauges: Some(&g),
            level: Some(0),
            progression: Some(&def),
            ..default()
        });
        assert!((start.rads.as_ref().unwrap().fraction() - 0.5).abs() < 1e-4);
        assert_eq!(
            player_source_text("rads", "", &start).as_deref(),
            Some("1,5 / 3 rads")
        );
        let g = gauges(12.0);
        let max = hud_values(&HudPlayerInput {
            gauges: Some(&g),
            level: Some(2),
            progression: Some(&def),
            ..default()
        });
        assert_eq!(max.rads.as_ref().unwrap().fraction(), 1.0);
        assert_eq!(
            player_source_text("rads", "", &max).as_deref(),
            Some("12 rads")
        );
    }

    #[test]
    fn sans_progression_ni_floors_rien() {
        let g = gauges(5.0);
        let floor = FloorState::default();
        let values = hud_values(&HudPlayerInput {
            gauges: Some(&g),
            floor: Some(&floor),
            ..default()
        });
        for source in ["rads", "level", "ammo_by_type", "statuses", "floor"] {
            assert_eq!(
                player_source_text(source, "Préfixe ", &values),
                Some(String::new()),
                "{source}"
            );
        }
        let snapshot = snapshot_of(10, values);
        assert!(snapshot.bars.is_empty());
        assert_eq!(snapshot.texts["rads"], "");
    }

    #[test]
    fn munitions_custom_et_statuts() {
        let mut reserves = AmmoReserves::default();
        reserves.0.insert(AmmoType::Custom("obus".into()), 12);
        reserves.0.insert(AmmoType::Custom("balles".into()), 120);
        reserves.0.insert(AmmoType::Balle, 30);
        let statuses = Statuses(vec![
            StatusEntry {
                status: StatusDef::Burn,
                stacks: 2,
                expires_at_frame: 190,
                id: "brulure".into(),
                source: None,
                source_team: None,
                next_tick_frame: 130,
            },
            StatusEntry {
                status: StatusDef::Slow,
                stacks: 1,
                expires_at_frame: 160,
                id: "lenteur".into(),
                source: None,
                source_team: None,
                next_tick_frame: 0,
            },
        ]);
        let values = hud_values(&HudPlayerInput {
            reserves: Some(&reserves),
            statuses: Some(&statuses),
            frame: 100,
            ..default()
        });
        assert_eq!(
            player_source_text("ammo_by_type", "", &values).as_deref(),
            Some("balles 120\nobus 12"),
            "Custom seulement, ordre des ids"
        );
        assert_eq!(
            player_source_text("statuses", "", &values).as_deref(),
            Some("brulure ×2 · 2 s\nlenteur · 1 s"),
            "90 frames → 2 s, 60 → 1 s"
        );
    }

    #[test]
    fn etage_en_mode_floors() {
        let floor = FloorState {
            index: 2,
            anchor: Some((Fixed::ZERO, Fixed::ZERO)),
            ..default()
        };
        let values = hud_values(&HudPlayerInput {
            floor: Some(&floor),
            ..default()
        });
        assert_eq!(
            player_source_text("floor", "", &values).as_deref(),
            Some("Étage 3")
        );
    }

    #[test]
    fn snapshot_textes_et_barres() {
        let def = def();
        let g = gauges(5.0);
        let values = hud_values(&HudPlayerInput {
            health: Some((fx(80.0), fx(100.0))),
            gauges: Some(&g),
            level: Some(1),
            progression: Some(&def),
            ..default()
        });
        let snapshot = snapshot_of(540, values);
        assert_eq!(snapshot.frame, 540);
        assert_eq!(snapshot.texts["health"], "80/100");
        assert_eq!(snapshot.texts["level"], "Niv. 1");
        assert_eq!(snapshot.texts["weapon"], "");
        assert!((snapshot.bars["health"] - 0.8).abs() < 1e-6);
        assert!((snapshot.bars["rads"] - 0.4).abs() < 1e-4);
        for source in PLAYER_SOURCES {
            assert!(snapshot.texts.contains_key(*source), "{source}");
            assert!(content::ui::HUD_SOURCES.contains(source), "{source}");
        }
    }

    #[test]
    fn objets_compteurs_charge_et_absence() {
        let table: ItemTable = ItemTable(
            [(
                "fiole".into(),
                ron::from_str(include_str!(
                    "../../../../games/gungeon/assets/objets/fiole.ron"
                ))
                .unwrap(),
            )]
            .into_iter()
            .collect(),
        );
        let mut items = Inventory {
            passives: vec!["inconnu".into(), "inconnu".into()],
            active: Some(items::ActiveSlot {
                item: "fiole".into(),
                charge: 2,
                uses: None,
            }),
            consumables: [("blank".into(), 2), ("key".into(), 1), ("bomb".into(), 4)]
                .into_iter()
                .collect(),
        };
        let read = |items: Option<&Inventory>, present: bool| {
            snapshot_of(
                12,
                hud_values(&HudPlayerInput {
                    player_present: present,
                    currency: present.then_some(73),
                    items,
                    item_table: Some(&table),
                    ..default()
                }),
            )
        };
        let ready = read(Some(&items), true);
        assert_eq!(ready.texts["blanks"], "2");
        assert_eq!(ready.texts["keys"], "1");
        assert_eq!(ready.texts["bombs"], "4");
        assert_eq!(ready.texts["shells"], "73");
        assert_eq!(ready.texts["passives"], "inconnu\ninconnu");
        assert_eq!(ready.texts["active_item"], "Fiole de rage");
        assert_eq!(ready.texts["active_charge"], "2 / 2 · prêt");
        assert_eq!(ready.bars["active_charge"], 1.0);
        items.active.as_mut().unwrap().charge = 0;
        items.consumables.insert("blank".into(), 1);
        let used = read(Some(&items), true);
        assert_eq!(used.texts["active_charge"], "0 / 2");
        assert_eq!(used.bars["active_charge"], 0.0);
        assert_eq!(used.texts["blanks"], "1");
        let empty = read(None, true);
        assert_eq!(empty.texts["keys"], "0");
        assert_eq!(empty.texts["bombs"], "0");
        assert_eq!(empty.texts["active_item"], "");
        assert!(!empty.bars.contains_key("active_charge"));
        let dead = read(None, false);
        for source in [
            "blanks",
            "keys",
            "bombs",
            "shells",
            "active_item",
            "active_charge",
            "passives",
        ] {
            assert_eq!(dead.texts[source], "", "{source}");
            assert_eq!(
                player_source_text(source, "préfixe ", &dead.values),
                Some(String::new())
            );
        }
    }

    #[test]
    fn minicarte_spatiale_stable_et_derivee_sans_memoire() {
        let room = |id: &str, kind: &str, x: i32, y: i32| HudRoomBounds {
            id: id.into(),
            kind: kind.into(),
            position: FixedVec2::new(Fixed::from_num(x), Fixed::from_num(y)),
            size: FixedVec2::new(Fixed::from_num(100), Fixed::from_num(100)),
        };
        let geometry = HudRoomGeometry(vec![
            room("boss", "boss", 100, 100),
            room("combat", "combat", 100, 0),
            room("start", "depart", 0, 0),
        ]);
        let none = BTreeSet::new();
        let mut states = RoomStates::default();
        let before = minimap_rooms(
            &geometry,
            &states,
            &none,
            Some(FixedVec2::new(fx(50.0), fx(50.0))),
        );
        assert!(before[0].current && before[0].visited);
        assert_eq!(before[0].rect, [0.0, 0.5, 0.5, 0.5]);
        assert!(!before[1].visited);
        assert!(before[2].boss);
        assert_eq!(before[2].rect, [0.5, 0.0, 0.5, 0.5]);
        states.rooms.insert("combat".into(), RoomState::Locked);
        let locked = minimap_rooms(
            &geometry,
            &states,
            &none,
            Some(FixedVec2::new(fx(150.0), fx(50.0))),
        );
        assert_eq!(locked[1].state, RoomState::Locked);
        assert!(locked[1].current && locked[1].visited);
        assert!(!locked[0].current && locked[0].visited);
        assert_eq!(before[1].rect, locked[1].rect);
        states.rooms.insert("combat".into(), RoomState::Cleared);
        let cleared = minimap_rooms(&geometry, &states, &none, None);
        assert!(cleared[1].visited);
        assert_eq!(cleared[1].state, RoomState::Cleared);
        // Rollback : aucune salle "visitée" ne reste en cache.
        let rewound = minimap_rooms(&geometry, &RoomStates::default(), &none, None);
        assert!(!rewound[1].visited);
        assert!(minimap_rooms(&HudRoomGeometry::default(), &states, &none, None).is_empty());
        let snapshot = snapshot_of(
            100,
            HudPlayerValues {
                minimap: locked,
                ..default()
            },
        );
        assert!(snapshot.texts["minimap"].contains("combat · verrouillée · actuelle"));
        assert!(snapshot.texts["minimap"].contains("boss · boss · inexplorée"));
    }

    /// D72 : une salle non combattante (jamais activée, donc `Dormant`) visitée puis quittée
    /// reste marquée visitée ; l'étage suivant repart de zéro.
    #[test]
    fn salle_visitee_puis_quittee_reste_marquee() {
        let room = |id: &str, kind: &str, x: i32| HudRoomBounds {
            id: id.into(),
            kind: kind.into(),
            position: FixedVec2::new(Fixed::from_num(x), Fixed::ZERO),
            size: FixedVec2::new(Fixed::from_num(100), Fixed::from_num(100)),
        };
        let geometry = HudRoomGeometry(vec![
            room("start", "depart", 0),
            room("shop", "boutique", 100),
            room("boss", "boss", 200),
        ]);
        let mut app = App::new();
        app.insert_resource(geometry.clone())
            .init_resource::<MinimapVisits>()
            .add_systems(Update, track_minimap_visits);
        let player = app
            .world_mut()
            .spawn((
                Player {
                    handle: 0,
                    ..Default::default()
                },
                LocalPlayer {},
                FixedTransform3D::IDENTITY,
            ))
            .id();
        let mut move_to = |app: &mut App, x: i32| {
            app.world_mut()
                .get_mut::<FixedTransform3D>(player)
                .unwrap()
                .translation
                .x = Fixed::from_num(x);
            app.update();
        };
        move_to(&mut app, 150);
        move_to(&mut app, 50);
        let visits = app.world().resource::<MinimapVisits>().visited.clone();
        assert!(visits.contains("shop") && visits.contains("start") && !visits.contains("boss"));
        let none = RoomStates::default();
        let map = minimap_rooms(
            &geometry,
            &none,
            &visits,
            Some(FixedVec2::new(fx(50.0), fx(50.0))),
        );
        let shop = map.iter().find(|r| r.id == "shop").unwrap();
        assert!(shop.visited && !shop.current && shop.state == RoomState::Dormant);
        assert!(!map.iter().find(|r| r.id == "boss").unwrap().visited);
        let snapshot = snapshot_of(
            1,
            HudPlayerValues {
                minimap: map,
                ..default()
            },
        );
        assert!(snapshot.texts["minimap"].contains("boutique · visitée"));
        assert!(snapshot.texts["minimap"].contains("boss · boss · inexplorée"));
        // Nouvel étage : géométrie différente, mémoire vidée.
        app.insert_resource(HudRoomGeometry(vec![room("other", "depart", 0)]));
        app.update();
        assert!(!app
            .world()
            .resource::<MinimapVisits>()
            .visited
            .contains("shop"));
    }
}
