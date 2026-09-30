//! Moments clés d'une partie (vague, kills, morts, fenêtres, portes, armes), détectés en
//! comparant l'état du jeu d'une frame à l'autre. Affichés sous les vidéos de la page de
//! revue pour aller directement aux instants importants.
//!
//! Hors simulation : lit l'état après chaque update, ne modifie rien.

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use combat::downed::{Downed, RunOutcome};
use game::{
    character::{health::Health, player::Player},
    collider::Collider,
    waves::{WavePhase, WaveState},
    weapons::{WeaponInventory, WeaponModesState, WeaponState},
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
    /// Arme active : nom, mode, munitions du chargeur et chargeurs restants
    weapon: String,
    mode: String,
    ammo: u32,
    mags: u32,
    dashing: bool,
    sprinting: bool,
    melee: bool,
    position: (i32, i32),
    /// À terre (T1.3, chantier B6) : `combat::downed::Downed` présent sur ce joueur.
    downed: bool,
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
    /// À terre (T1.3, chantier B6) : `combat::downed::RunOutcome::defeat_at_frame.is_some()`.
    defeat: bool,
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
    run_outcome: Option<Res<RunOutcome>>,
    players: Query<(
        &Player,
        &Health,
        Option<&WeaponInventory>,
        Option<&game::character::dash::DashState>,
        Option<&game::character::movement::SprintState>,
        Option<&game::weapons::melee::MeleeAttackState>,
        &bevy_fixed::fixed_math::FixedTransform3D,
        Has<Downed>,
    )>,
    weapons: Query<(&WeaponState, &WeaponModesState)>,
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
    now.defeat = run_outcome.is_some_and(|outcome| outcome.defeat_at_frame.is_some());
    for (player, health, inventory, dash, sprint, melee, transform, downed) in &players {
        let mut snapshot = PlayerSnapshot {
            reloading: inventory.is_some_and(|i| i.reloading_ending_frame.is_some()),
            weapon_index: inventory.map_or(0, |i| i.active_weapon_index),
            hit: health.current < health.max,
            dashing: dash.is_some_and(|d| d.is_dashing),
            sprinting: sprint.is_some_and(|s| s.is_sprinting),
            melee: melee.is_some_and(|m| m.is_attacking),
            position: (
                transform.translation.x.to_num::<i32>(),
                transform.translation.y.to_num::<i32>(),
            ),
            downed,
            ..Default::default()
        };
        if let Some((entity, weapon)) = inventory.and_then(|i| i.weapons.get(i.active_weapon_index))
        {
            snapshot.weapon = weapon.config.name.clone();
            if let Ok((state, modes)) = weapons.get(*entity) {
                snapshot.mode = state.active_mode.clone();
                if let Some(mode) = modes.modes.get(&state.active_mode) {
                    snapshot.ammo = mode.mag_ammo;
                    snapshot.mags = mode.mag_quantity;
                }
            }
        }
        now.players.insert(player.handle, snapshot);
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
        events.events.push(GameEvent {
            frame: f,
            kind,
            label,
        });
    };

    if now.wave != before.wave || now.phase != before.phase {
        if let Some(phase) = now.phase {
            push(
                "wave",
                format!("vague {} : {}", now.wave, phase_label(phase)),
            );
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
                    push(
                        "reload",
                        format!(
                            "joueur {handle} recharge ({} {})",
                            player.weapon, player.mode
                        ),
                    );
                }
                if !player.reloading && previous.reloading {
                    push(
                        "reload",
                        format!(
                            "joueur {handle} a rechargé : {} ({} chargeurs)",
                            player.ammo, player.mags
                        ),
                    );
                }
                if player.weapon_index != previous.weapon_index {
                    push(
                        "weapon",
                        format!(
                            "joueur {handle} prend {} ({}, {} balles)",
                            player.weapon, player.mode, player.ammo
                        ),
                    );
                } else if player.mode != previous.mode {
                    push(
                        "weapon",
                        format!(
                            "joueur {handle} passe en mode {} ({} balles)",
                            player.mode, player.ammo
                        ),
                    );
                }
                if player.dashing && !previous.dashing {
                    push(
                        "move",
                        format!("joueur {handle} dash depuis {:?}", previous.position),
                    );
                }
                if !player.dashing && previous.dashing {
                    push(
                        "move",
                        format!("joueur {handle} fin du dash en {:?}", player.position),
                    );
                }
                if player.sprinting != previous.sprinting {
                    let what = if player.sprinting {
                        "sprinte"
                    } else {
                        "arrête de sprinter"
                    };
                    push("move", format!("joueur {handle} {what}"));
                }
                if player.melee && !previous.melee {
                    push("melee", format!("joueur {handle} attaque au corps à corps"));
                }
                // À terre (T1.3, chantier B6). `revived` seulement quand le joueur est
                // toujours là (`now.players` le contient) : `Downed` retiré par une mort
                // de saignement, sans joueur qui reste, tombe dans la boucle `death`
                // ci-dessous, pas ici.
                if player.downed && !previous.downed {
                    push("downed", format!("joueur {handle} tombe à terre"));
                }
                if !player.downed && previous.downed {
                    push("revived", format!("joueur {handle} réanimé"));
                }
                if player.weapon_index == previous.weapon_index
                    && player.ammo == 0
                    && previous.ammo > 0
                {
                    push(
                        "weapon",
                        format!("joueur {handle} : chargeur vide ({})", player.weapon),
                    );
                }
            }
        }
    }
    for handle in before
        .players
        .keys()
        .filter(|h| !now.players.contains_key(h))
    {
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
    if now.defeat && !before.defeat {
        push(
            "defeat",
            "défaite : tous les joueurs sont à terre ou morts".to_string(),
        );
    }
}
