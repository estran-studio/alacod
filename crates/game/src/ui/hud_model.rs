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

use std::collections::BTreeMap;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_fixed::fixed_math::Fixed;
use combat::inventory::AmmoReserves;
use combat::status::Statuses;
use run::floors::FloorState;
use sim_core::ammo::AmmoType;
use utils::frame::FrameCount;

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

impl Plugin for HudModelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HudSnapshot>()
            .add_systems(Update, update_hud_snapshot);
    }
}

/// Ce que [`update_hud_snapshot`] lit du joueur affiché.
#[derive(SystemParam)]
pub(crate) struct HudPlayerState<'w, 's> {
    healths: Query<'w, 's, &'static Health, With<Player>>,
    currencies: Query<'w, 's, &'static run::currency::Currency>,
    inventories: Query<'w, 's, (&'static WeaponInventory, &'static AmmoReserves)>,
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
}
