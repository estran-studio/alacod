//! Sons d'événements du moteur (S1-7, `docs/taches/polissage-sons-evenements.md`) : vague,
//! salle, étage, boss, achat, ramassage, explosion, bombe, menus.
//!
//! Présentation seule, en `Update` : chaque son se **dérive de l'état** (changement d'une
//! ressource ou d'un composant d'une image à l'autre), des `FrameEvents` de la dernière frame
//! simulée ou du `FeedbackLog`. Aucun son ne part de `GgrsSchedule`. Les clés (`SOUND_KEYS`,
//! `content::feedback`) sont optionnelles : sans la clé dans `ui/feedback.ron`, rien ne joue et
//! le rendu d'avant est inchangé. Absent en headless (`FeedbackPlugin`), donc sans effet sur les
//! traces ; les décisions sont des fonctions pures testées ci-dessous.

use bevy::prelude::*;
use std::collections::BTreeSet;
use utils::net_id::GgrsNetId;

use crate::character::player::LocalPlayer;
use crate::feedback::FeedbackSounds;
use crate::feedback_log::{FeedbackKind, FeedbackLog};
use crate::frame_events::FrameEvents;
use crate::ui::screens::model::ScreensState;
use crate::ui::screens::Screens;
use crate::waves::state::{WavePhase, WaveState};

pub(crate) fn add_event_sounds(app: &mut App) {
    app.add_systems(
        Update,
        (
            play_wave_sounds,
            play_room_cleared_sound,
            play_floor_sound,
            play_boss_sounds,
            play_log_sounds,
            play_purchase_and_pickup_sounds,
            play_menu_sounds,
        ),
    );
}

/// Une image rendue sans nouvelle frame simulée voit encore les `FrameEvents` de la précédente :
/// vrai une seule fois par frame (et à chaque retour en arrière du compteur : nouvelle partie).
fn first_look(last: &mut Option<u32>, frame: u32) -> bool {
    let fresh = last.is_none_or(|l| frame != l);
    *last = Some(frame);
    fresh
}

// --- Vagues ----------------------------------------------------------------------------------

/// `wave_start` : le numéro de vague monte ; `wave_end` : la vague passe à `WaveComplete`.
/// `previous` : `(vague, phase)` de l'image d'avant ; `None` à la première image (référence).
fn wave_sound(previous: Option<(u32, WavePhase)>, now: (u32, WavePhase)) -> Option<&'static str> {
    let (wave, phase) = previous?;
    if now.0 > wave {
        Some("wave_start")
    } else if now.1 == WavePhase::WaveComplete && phase != WavePhase::WaveComplete {
        Some("wave_end")
    } else {
        None
    }
}

fn play_wave_sounds(
    sounds: FeedbackSounds,
    waves: Option<Res<WaveState>>,
    mut previous: Local<Option<(u32, WavePhase)>>,
) {
    let Some(waves) = waves else {
        *previous = None;
        return;
    };
    let now = (waves.current_wave, waves.phase);
    // Une vague qui redescend : nouvelle partie, on réarme sans jouer.
    if previous.is_some_and(|(wave, _)| now.0 < wave) {
        *previous = Some(now);
        return;
    }
    if let Some(key) = wave_sound(*previous, now) {
        sounds.play(key);
    }
    *previous = Some(now);
}

// --- Salles, étages --------------------------------------------------------------------------

/// `room_cleared` : une salle passe à `Cleared` (`FrameEvents<RoomChanged>`).
fn play_room_cleared_sound(
    sounds: FeedbackSounds,
    changes: Option<Res<FrameEvents<world::rooms::RoomChanged>>>,
    mut last_frame: Local<Option<u32>>,
) {
    let Some(changes) = changes else {
        return;
    };
    let cleared = changes
        .iter()
        .any(|c| c.to == world::rooms::RoomState::Cleared);
    if cleared && first_look(&mut last_frame, sounds.frame.frame) {
        sounds.play("room_cleared");
    }
}

/// `floor` : l'index d'étage monte (`FloorState`). Rien à la première image ni à un retour à 0.
fn play_floor_sound(
    sounds: FeedbackSounds,
    floors: Option<Res<run::floors::FloorState>>,
    mut previous: Local<Option<u32>>,
) {
    let now = floors.map(|f| f.index);
    if let (Some(before), Some(index)) = (*previous, now) {
        if index > before {
            sounds.play("floor");
        }
    }
    *previous = now;
}

// --- Boss ------------------------------------------------------------------------------------

/// `boss_spawn` : un `BossState` apparaît ; `boss_phase` : `FrameEvents<BossPhaseChanged>`.
/// (`boss_death` : indice `BossEnd` du journal, voir [`play_log_sounds`].)
fn play_boss_sounds(
    sounds: FeedbackSounds,
    bosses: Query<&GgrsNetId, With<behaviors::BossState>>,
    phases: Option<Res<FrameEvents<behaviors::BossPhaseChanged>>>,
    mut previous: Local<Option<BTreeSet<usize>>>,
    mut last_phase_frame: Local<Option<u32>>,
) {
    let now: BTreeSet<usize> = bosses.iter().map(|n| n.0).collect();
    if let Some(before) = previous.as_ref() {
        if now.difference(before).next().is_some() {
            sounds.play("boss_spawn");
        }
    }
    *previous = Some(now);
    if let Some(phases) = phases {
        if phases.iter().next().is_some() && first_look(&mut last_phase_frame, sounds.frame.frame) {
            sounds.play("boss_phase");
        }
    }
}

/// Clé jouée pour un indice du journal de feedback, s'il en a une.
fn log_sound_key(kind: &FeedbackKind) -> Option<&'static str> {
    match kind {
        FeedbackKind::BossEnd => Some("boss_death"),
        FeedbackKind::Explosion => Some("explosion"),
        FeedbackKind::BombFuse => Some("bomb"),
        _ => None,
    }
}

/// `boss_death`, `explosion`, `bomb` : les indices de la dernière frame simulée. Un son par clé et
/// par frame, quel que soit le nombre d'indices (une salve d'explosions n'en joue qu'un).
fn play_log_sounds(sounds: FeedbackSounds, log: Res<FeedbackLog>) {
    let keys: BTreeSet<&'static str> = log
        .frame_cues
        .iter()
        .filter_map(|cue| log_sound_key(&cue.kind))
        .collect();
    for key in keys {
        sounds.play(key);
    }
}

// --- Achat, ramassage ------------------------------------------------------------------------

/// `purchase` : un joueur local dépense (`CurrencyEvent` négatif) ; `pickup` : un joueur local
/// ramasse un objet (`ItemPicked`) ou un power-up (`PowerUpPickedUp`). Un son par clé et frame.
fn play_purchase_and_pickup_sounds(
    sounds: FeedbackSounds,
    currency: Option<Res<FrameEvents<run::currency::CurrencyEvent>>>,
    items: Option<Res<FrameEvents<items::ItemPicked>>>,
    powerups: Option<Res<FrameEvents<crate::powerups::PowerUpPickedUp>>>,
    local_players: Query<(&GgrsNetId, &crate::character::player::Player), With<LocalPlayer>>,
    mut last_frame: Local<Option<u32>>,
) {
    let handles: BTreeSet<usize> = local_players.iter().map(|(_, p)| p.handle).collect();
    let ids: BTreeSet<usize> = local_players.iter().map(|(n, _)| n.0).collect();
    let purchase =
        currency.is_some_and(|c| c.iter().any(|e| e.delta < 0 && handles.contains(&e.handle)));
    let pickup = items.is_some_and(|i| i.iter().any(|e| handles.contains(&e.player_handle)))
        || powerups.is_some_and(|p| p.iter().any(|e| ids.contains(&e.picked_up_by.0)));
    if !(purchase || pickup) || !first_look(&mut last_frame, sounds.frame.frame) {
        return;
    }
    if purchase {
        sounds.play("purchase");
    }
    if pickup {
        sounds.play("pickup");
    }
}

// --- Menus -----------------------------------------------------------------------------------

/// Son d'un changement d'état des écrans : ouvrir ou valider « Réglages » = `menu_confirm`,
/// fermer = `menu_back`, bouger un focus = `menu_navigate`. `volume_changed` : le curseur a bougé.
fn menu_sound(
    before: &ScreensState,
    now: &ScreensState,
    volume_changed: bool,
) -> Option<&'static str> {
    match (before.open, now.open) {
        (None, Some(_)) => Some("menu_confirm"),
        (Some(_), None) => Some("menu_back"),
        (None, None) => None,
        (Some(_), Some(_)) => {
            if now.settings_shown && !before.settings_shown {
                Some("menu_confirm")
            } else if before.settings_shown
                && !now.settings_shown
                && before.pause_focus == now.pause_focus
            {
                Some("menu_back")
            } else if volume_changed
                || before.pause_focus != now.pause_focus
                || before.inventory_focus != now.inventory_focus
                || before.merchant_focus != now.merchant_focus
            {
                Some("menu_navigate")
            } else {
                None
            }
        }
    }
}

fn play_menu_sounds(
    sounds: FeedbackSounds,
    screens: Res<Screens>,
    master: Res<crate::audio::MasterVolume>,
    mut previous: Local<Option<(ScreensState, u8)>>,
) {
    let now = (screens.state, master.0);
    if let Some((before, volume)) = *previous {
        if let Some(key) = menu_sound(&before, &now.0, volume != now.1) {
            sounds.play(key);
        }
    }
    *previous = Some(now);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::screens::model::ScreenId;

    #[test]
    fn vague_debut_et_fin() {
        use WavePhase::*;
        assert_eq!(wave_sound(None, (1, Spawning)), None);
        assert_eq!(wave_sound(Some((1, GracePeriod)), (1, Spawning)), None);
        assert_eq!(
            wave_sound(Some((1, GracePeriod)), (2, Spawning)),
            Some("wave_start")
        );
        assert_eq!(
            wave_sound(Some((2, InProgress)), (2, WaveComplete)),
            Some("wave_end")
        );
        assert_eq!(wave_sound(Some((2, WaveComplete)), (2, WaveComplete)), None);
    }

    #[test]
    fn indices_du_journal() {
        assert_eq!(log_sound_key(&FeedbackKind::BossEnd), Some("boss_death"));
        assert_eq!(log_sound_key(&FeedbackKind::Explosion), Some("explosion"));
        assert_eq!(log_sound_key(&FeedbackKind::BombFuse), Some("bomb"));
        assert_eq!(log_sound_key(&FeedbackKind::Flash), None);
        assert_eq!(log_sound_key(&FeedbackKind::Number), None);
    }

    #[test]
    fn une_fois_par_frame() {
        let mut last = None;
        assert!(first_look(&mut last, 10));
        assert!(!first_look(&mut last, 10));
        assert!(first_look(&mut last, 11));
        assert!(first_look(&mut last, 0));
    }

    #[test]
    fn sons_de_menu() {
        let closed = ScreensState::default();
        let pause = ScreensState {
            open: Some(ScreenId::Pause),
            ..closed
        };
        assert_eq!(menu_sound(&closed, &pause, false), Some("menu_confirm"));
        assert_eq!(menu_sound(&pause, &closed, false), Some("menu_back"));
        assert_eq!(menu_sound(&closed, &closed, false), None);
        assert_eq!(menu_sound(&pause, &pause, false), None);
        let moved = ScreensState {
            pause_focus: 1,
            ..pause
        };
        assert_eq!(menu_sound(&pause, &moved, false), Some("menu_navigate"));
        let settings = ScreensState {
            settings_shown: true,
            pause_focus: 3,
            ..pause
        };
        let before = ScreensState {
            pause_focus: 3,
            ..pause
        };
        assert_eq!(menu_sound(&before, &settings, false), Some("menu_confirm"));
        assert_eq!(
            menu_sound(&settings, &settings, true),
            Some("menu_navigate")
        );
    }
}
