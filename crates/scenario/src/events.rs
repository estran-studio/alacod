//! Moments clés d'une partie (vague, kills, morts, fenêtres, portes, armes), détectés en
//! comparant l'état du jeu d'une frame à l'autre. Affichés sous les vidéos de la page de
//! revue pour aller directement aux instants importants.
//!
//! Hors simulation : lit l'état après chaque update, ne modifie rien.

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use game::{
    character::{health::Health, player::Player},
    collider::Collider,
    waves::{WavePhase, WaveState},
    weapons::WeaponInventory,
};
use map::game::entity::map::{door::DoorComponent, window::WindowHealth};
use serde::Serialize;
use utils::{frame::FrameCount, net_id::GgrsNetId};

/// Un moment clé : sa frame, sa catégorie (pour le style) et son libellé.
#[derive(Debug, Clone, Serialize)]
pub struct GameEvent {
    pub frame: u32,
    pub kind: &'static str,
    pub label: String,
}

#[derive(Resource, Default)]
pub struct GameEvents {
    pub events: Vec<GameEvent>,
    previous: Option<Snapshot>,
}

#[derive(Default, Clone, PartialEq)]
struct PlayerSnapshot {
    reloading: bool,
    weapon_index: usize,
    hit: bool,
}

#[derive(Default, Clone)]
struct Snapshot {
    wave: u32,
    phase: Option<WavePhase>,
    kills: u32,
    players: BTreeMap<usize, PlayerSnapshot>,
    windows: BTreeMap<usize, u8>,
    closed_doors: BTreeSet<usize>,
    open_doors: BTreeSet<usize>,
}

pub struct GameEventsPlugin;

impl Plugin for GameEventsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameEvents>()
            .add_systems(Last, detect_events);
    }
}

fn phase_label(phase: WavePhase) -> &'static str {
    match phase {
        WavePhase::NotStarted => "pas commencée",
        WavePhase::GracePeriod => "préparation",
        WavePhase::Spawning => "arrivée des zombies",
        WavePhase::InProgress => "tous les zombies sont là",
        WavePhase::WaveComplete => "vague terminée",
    }
}

#[allow(clippy::too_many_arguments)]
fn detect_events(
    frame: Res<FrameCount>,
    mut events: ResMut<GameEvents>,
    wave: Option<Res<WaveState>>,
    players: Query<(&Player, &Health, Option<&WeaponInventory>)>,
    windows: Query<(&GgrsNetId, &WindowHealth)>,
    // Porte ouverte = sans collider (une porte non interactive reste fermée)
    doors: Query<(&GgrsNetId, Has<Collider>), With<DoorComponent>>,
) {
    let mut now = Snapshot::default();
    if let Some(wave) = &wave {
        now.wave = wave.current_wave;
        now.phase = Some(wave.phase);
        now.kills = wave.total_enemies_killed;
    }
    for (player, health, inventory) in &players {
        now.players.insert(
            player.handle,
            PlayerSnapshot {
                reloading: inventory.is_some_and(|i| i.reloading_ending_frame.is_some()),
                weapon_index: inventory.map_or(0, |i| i.active_weapon_index),
                hit: health.current < health.max,
            },
        );
    }
    for (id, health) in &windows {
        now.windows.insert(id.0, health.current);
    }
    for (id, closed) in &doors {
        if closed {
            now.closed_doors.insert(id.0);
        } else {
            now.open_doors.insert(id.0);
        }
    }

    // Première frame vue : référence, rien à signaler (sauf joueurs présents)
    let Some(before) = events.previous.replace(now.clone()) else {
        return;
    };
    let f = frame.frame;
    let mut push = |kind: &'static str, label: String| {
        events.events.push(GameEvent { frame: f, kind, label });
    };

    if now.wave != before.wave || now.phase != before.phase {
        if let Some(phase) = now.phase {
            push("wave", format!("vague {} : {}", now.wave, phase_label(phase)));
        }
    }
    if now.kills > before.kills {
        push("kill", format!("zombie tué ({} au total)", now.kills));
    }
    for (handle, player) in &now.players {
        match before.players.get(handle) {
            None => push("player", format!("joueur {handle} apparaît")),
            Some(previous) => {
                if player.hit && !previous.hit {
                    push("hit", format!("joueur {handle} touché"));
                }
                if player.reloading && !previous.reloading {
                    push("reload", format!("joueur {handle} recharge"));
                }
                if player.weapon_index != previous.weapon_index {
                    push("weapon", format!("joueur {handle} change d'arme"));
                }
            }
        }
    }
    for handle in before.players.keys().filter(|h| !now.players.contains_key(h)) {
        push("death", format!("joueur {handle} meurt"));
    }
    for (id, health) in &now.windows {
        let previous = before.windows.get(id).copied().unwrap_or(*health);
        if *health == 0 && previous > 0 {
            push("window", format!("fenêtre {id} cassée"));
        } else if *health > previous {
            push("window", format!("fenêtre {id} réparée ({health})"));
        }
    }
    for id in now.open_doors.intersection(&before.closed_doors) {
        push("door", format!("porte {id} ouverte"));
    }
}
