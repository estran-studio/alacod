//! Interpolation d'affichage (S1-6, `docs/conventions.md` §94) : **présentation seule**.
//!
//! La simulation avance par pas fixes de 1/60 s (`bevy_ggrs`, accumulateur de temps réel) ; sans
//! interpolation, chaque image rendue recopie l'état de la dernière frame simulée
//! (`sync_bevy_transforms_from_fixed`). Sur un écran à 120 Hz, une frame simulée est montrée deux
//! fois : le joueur avance de 2,5 px, puis de 0, puis de 2,5 px… pendant que la caméra glisse à
//! chaque image ; à 60 Hz, une image sans frame simulée suivie d'une image à deux frames donne le
//! même à-coup. Ici, après la recopie, la `translation` x/y de chaque entité rollback est placée entre
//! ses deux derniers états simulés, au prorata du temps accumulé depuis le dernier pas (`alpha`) :
//! le rendu a au plus une frame de retard sur la simulation, et le mouvement est continu.
//!
//! Garanties :
//! - rien dans `GgrsSchedule` ni dans un hash : un système en `PostUpdate` qui n'écrit que le
//!   `Transform` bevy, et un composant ([`DisplayedMotion`]) qui n'est pas rollback. Seul ajout aux
//!   schedules du rollback : [`note_rollback`] dans `LoadWorld` (un drapeau de présentation), par ce
//!   plugin, absent en headless ;
//! - coupée (l'état simulé est montré tel quel) au téléport (écart d'une frame à l'autre plus grand
//!   que `snap_distance`, ce qui couvre la transition d'étage et une entité ramenée par un rollback),
//!   au retour en arrière de la frame (nouvelle partie), et au rollback d'une partie en ligne (les
//!   positions prédites ne sont pas mélangées aux positions corrigées) ;
//! - désactivable par jeu (`camera.ron`, `interpolation: (enabled: false)`) et coupée par la capture
//!   de `play_scenario` (une image par frame simulée : l'image `n` montre la frame `n`).

use std::time::Duration;

use bevy::prelude::*;
use bevy_fixed::fixed_math::{self, FixedTransform3D};
use utils::frame::FrameCount;

use crate::camera::CameraSettings;
use crate::session::{AppState, OnlineState, SIM_FPS};

/// Réglage par jeu (`camera.ron`, champ `interpolation`).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct DisplayInterpolationSettings {
    /// Interpoler l'affichage (défaut : oui).
    pub enabled: bool,
    /// Écart (px) d'une frame simulée à la suivante au-delà duquel l'entité est montrée à sa
    /// nouvelle position sans transition (téléport, étage, rollback). Le dash le plus rapide fait
    /// 13,5 px par frame et une balle de 1 200 px/s 20 px.
    pub snap_distance: f32,
}

impl Default for DisplayInterpolationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            snap_distance: 48.0,
        }
    }
}

/// Coupe l'interpolation quel que soit `camera.ron` (posée par la capture de `play_scenario`).
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct DisplayInterpolationOff;

/// Horloge de l'interpolation : copie de l'accumulateur de `bevy_ggrs` (privé), tenue à partir du
/// même `Time` et des frames simulées observées, plus un `alpha` lissé.
///
/// Mesuré le 2026-10-28 (Mac, 60 Hz, `scripts/frame-pacing.py`) : l'intervalle entre deux passages de
/// la boucle principale varie de 11 à 23 ms alors que l'écran présente toutes les 16,7 ms. Un `alpha`
/// tiré de ce temps brut fait varier le pas affiché de 1,6 à 3,5 px pour 2,5 px par frame : l'horloge
/// d'affichage avance donc du delta **lissé** et n'est rappelée que doucement vers l'accumulateur.
#[derive(Resource, Debug, Default)]
pub struct InterpolationClock {
    accumulator: Duration,
    last_frame: Option<u32>,
    /// Moyenne glissante du delta (s), 0 avant la première image.
    smoothed_delta: f32,
    /// Frames simulées depuis l'image précédente (0 : rien de neuf à montrer).
    pub advanced: u32,
    /// Part du pas en cours écoulée, dans `[0, 1]` : 0 montre l'avant-dernier état, 1 le dernier.
    pub alpha: f32,
    /// Partie remise à zéro ou rollback d'une partie en ligne : montrer l'état tel quel.
    pub cut: bool,
    /// Un `LoadWorld` a eu lieu depuis l'image précédente ([`note_rollback`]).
    rolled_back: bool,
}

/// Durée d'une frame simulée, au nanoseconde près comme `bevy_ggrs` (`1e9 / fps`).
fn tick() -> Duration {
    Duration::from_nanos(1_000_000_000 / SIM_FPS)
}

/// Poids de la nouvelle mesure dans la moyenne glissante du delta.
const DELTA_SMOOTHING: f32 = 0.1;
/// Part de l'écart entre l'`alpha` prédit et celui de l'accumulateur rattrapée à chaque image.
const ALPHA_CORRECTION: f32 = 0.1;

impl InterpolationClock {
    /// Avance l'horloge d'une image : `delta` de temps réel, simulation à la frame `frame`.
    /// Même arithmétique que `bevy_ggrs::run_ggrs_schedules` (ajout du delta, retrait d'un pas
    /// par frame simulée) ; bornée à `[0, pas]` pour ne pas dériver quand la session ralentit
    /// (`run_slow` en ligne) ou attend un pair (aucune frame simulée : l'image montre le dernier état).
    pub fn advance(&mut self, delta: Duration, frame: u32) {
        let step = tick();
        self.accumulator = self.accumulator.saturating_add(delta);
        self.cut = false;
        self.advanced = 0;
        match self.last_frame {
            Some(last) if frame >= last => {
                self.advanced = frame - last;
                self.accumulator = self.accumulator.saturating_sub(step * self.advanced);
            }
            Some(_) => {
                self.cut = true;
                self.accumulator = Duration::ZERO;
            }
            None => self.cut = true,
        }
        self.last_frame = Some(frame);
        self.accumulator = self.accumulator.min(step);
        let raw = self.accumulator.as_secs_f32() / step.as_secs_f32();
        let delta = delta.as_secs_f32();
        if self.cut || self.smoothed_delta == 0.0 {
            self.smoothed_delta = delta;
            self.alpha = raw;
            return;
        }
        self.smoothed_delta += (delta - self.smoothed_delta) * DELTA_SMOOTHING;
        let predicted =
            self.alpha + self.smoothed_delta / step.as_secs_f32() - self.advanced as f32;
        self.alpha = (predicted + (raw - predicted) * ALPHA_CORRECTION).clamp(0.0, 1.0);
    }
}

/// Deux derniers états simulés d'une entité, vus par le rendu (x, y monde). Pas rollback.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct DisplayedMotion {
    pub previous: Vec2,
    pub current: Vec2,
}

/// Position affichée entre `previous` et `current` ; `current` seul au-delà de `snap` (téléport).
pub fn interpolated(previous: Vec2, current: Vec2, alpha: f32, snap: f32) -> Vec2 {
    if previous.distance_squared(current) > snap * snap {
        current
    } else {
        previous.lerp(current, alpha.clamp(0.0, 1.0))
    }
}

/// `LoadWorld` : un état vient d'être restauré. Lu à l'image suivante par
/// [`advance_interpolation_clock`] ; aucune donnée de simulation n'est lue ni écrite.
pub fn note_rollback(mut clock: ResMut<InterpolationClock>) {
    clock.rolled_back = true;
}

/// Début de l'interpolation de l'image : horloge et coupures.
pub fn advance_interpolation_clock(
    time: Res<Time>,
    frame: Res<FrameCount>,
    online: Res<OnlineState>,
    settings: Res<CameraSettings>,
    off: Option<Res<DisplayInterpolationOff>>,
    mut clock: ResMut<InterpolationClock>,
) {
    clock.advance(time.delta(), frame.frame);
    // En synctest (`check_distance` > 0) chaque frame est restaurée à l'identique : seul un
    // rollback en ligne peut corriger des positions prédites.
    if std::mem::take(&mut clock.rolled_back) && *online == OnlineState::Online {
        clock.cut = true;
    }
    // Coupée : l'image montre la dernière frame simulée (ce que lit `ALACOD_PERF_CSV`).
    if !settings.interpolation.enabled || off.is_some() {
        clock.alpha = 1.0;
    }
}

/// Place le `Transform` (x, y) de chaque entité simulée entre ses deux derniers états. Après la
/// recopie `FixedTransform3D` → `Transform`, avant la propagation.
pub fn interpolate_transforms(
    mut commands: Commands,
    clock: Res<InterpolationClock>,
    settings: Res<CameraSettings>,
    off: Option<Res<DisplayInterpolationOff>>,
    mut entities: Query<(
        Entity,
        &FixedTransform3D,
        &mut Transform,
        Option<&mut DisplayedMotion>,
    )>,
) {
    let config = settings.interpolation;
    let enabled = config.enabled && off.is_none();
    for (entity, fixed, mut transform, motion) in &mut entities {
        let simulated = fixed_math::fixed_to_vec3(fixed.translation).truncate();
        let Some(mut motion) = motion else {
            commands.entity(entity).try_insert(DisplayedMotion {
                previous: simulated,
                current: simulated,
            });
            continue;
        };
        if clock.cut || !enabled {
            motion.previous = simulated;
            motion.current = simulated;
            continue;
        }
        if clock.advanced > 0 || motion.current != simulated {
            let frames = clock.advanced.max(1) as f32;
            let jump = simulated - motion.current;
            motion.previous = if jump.length_squared() > (config.snap_distance * frames).powi(2) {
                // Téléport : aucune transition.
                simulated
            } else {
                // Plusieurs frames simulées pendant une image : l'état d'une frame avant la
                // dernière est estimé sur la droite qui joint les deux états vus (la vitesse
                // affichée ne double pas).
                motion.current + jump * ((frames - 1.0) / frames)
            };
            motion.current = simulated;
        }
        let shown = interpolated(
            motion.previous,
            motion.current,
            clock.alpha,
            config.snap_distance,
        );
        transform.translation.x = shown.x;
        transform.translation.y = shown.y;
    }
}

/// Interpolation d'affichage (`PresentationPlugin`).
pub struct DisplayInterpolationPlugin;

impl Plugin for DisplayInterpolationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InterpolationClock>();
        // `ALACOD_INTERPOLATION=0` : comparaison avant/après sans toucher `camera.ron` (mesure §94).
        if std::env::var("ALACOD_INTERPOLATION").is_ok_and(|value| value == "0") {
            app.insert_resource(DisplayInterpolationOff);
        }
        app.add_systems(bevy_ggrs::LoadWorld, note_rollback)
            .add_systems(
                PostUpdate,
                (advance_interpolation_clock, interpolate_transforms)
                    .chain()
                    .after(fixed_math::sync_bevy_transforms_from_fixed)
                    .before(bevy::transform::TransformSystems::Propagate)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(value: f64) -> Duration {
        Duration::from_secs_f64(value / 1000.0)
    }

    /// Position affichée, en frames, de chaque image pour une suite de deltas, avec le même
    /// accumulateur que `bevy_ggrs`.
    fn shown_positions(deltas: &[f64]) -> Vec<f32> {
        let mut clock = InterpolationClock::default();
        let (mut frame, mut accumulated) = (0u32, Duration::ZERO);
        clock.advance(Duration::ZERO, 0);
        deltas
            .iter()
            .map(|&delta| {
                accumulated += ms(delta);
                while accumulated >= tick() {
                    accumulated -= tick();
                    frame += 1;
                }
                clock.advance(ms(delta), frame);
                frame as f32 - 1.0 + clock.alpha
            })
            .collect()
    }

    #[test]
    fn un_delta_irregulier_donne_un_pas_regulier() {
        // Boucle principale mesurée à 11-23 ms pour un écran à 60 Hz : 12 puis 21,3 ms.
        let deltas: Vec<f64> = (0..240)
            .map(|i| {
                if i % 2 == 0 {
                    12.0
                } else {
                    2.0 * 1000.0 / 60.0 - 12.0
                }
            })
            .collect();
        let shown = shown_positions(&deltas);
        let steps: Vec<f32> = shown.windows(2).map(|w| w[1] - w[0]).skip(120).collect();
        let (min, max) = steps
            .iter()
            .fold((f32::MAX, f32::MIN), |(lo, hi), &s| (lo.min(s), hi.max(s)));
        // Sans lissage, le pas affiché suivrait le delta : 0,72 puis 1,28 frame.
        assert!(max - min < 0.15, "pas de {min} à {max}");
    }

    #[test]
    fn a_120_hz_chaque_frame_simulee_est_montree_en_deux_moities() {
        let mut clock = InterpolationClock::default();
        clock.advance(Duration::ZERO, 0);
        assert!(clock.cut);
        // Images de 8,33 ms : la simulation avance une image sur deux.
        let mut alphas = Vec::new();
        let mut frame = 0;
        let mut accumulated = Duration::ZERO;
        for _ in 0..8 {
            accumulated += ms(1000.0 / 120.0);
            if accumulated >= tick() {
                accumulated -= tick();
                frame += 1;
            }
            clock.advance(ms(1000.0 / 120.0), frame);
            alphas.push((clock.alpha * 100.0).round() as i32);
        }
        assert_eq!(alphas, vec![50, 0, 50, 0, 50, 0, 50, 0]);
    }

    #[test]
    fn une_session_qui_attend_montre_le_dernier_etat() {
        let mut clock = InterpolationClock::default();
        clock.advance(Duration::ZERO, 10);
        clock.advance(ms(40.0), 10);
        assert_eq!(clock.advanced, 0);
        assert_eq!(clock.alpha, 1.0);
    }

    #[test]
    fn une_nouvelle_partie_coupe() {
        let mut clock = InterpolationClock::default();
        clock.advance(Duration::ZERO, 500);
        clock.advance(ms(16.0), 501);
        assert!(!clock.cut);
        clock.advance(ms(16.0), 3);
        assert!(clock.cut);
    }

    #[test]
    fn un_teleport_n_est_pas_interpole() {
        let a = Vec2::new(0.0, 0.0);
        assert_eq!(
            interpolated(a, Vec2::new(10.0, 0.0), 0.5, 48.0),
            Vec2::new(5.0, 0.0)
        );
        assert_eq!(
            interpolated(a, Vec2::new(100.0, 0.0), 0.5, 48.0),
            Vec2::new(100.0, 0.0)
        );
    }
}
