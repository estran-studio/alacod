//! Compteur de FPS d'affichage et de simulation (S1-5). Présentation seule : aucun système dans
//! `GgrsSchedule`, aucune ressource rollback, rien dans un hash ni dans une trace.
//!
//! Allumé par `?perf=1` (surcouche + console) ou `?perf=console` (console seule) dans l'URL du
//! chargeur (`loader.html`, relayé par la page `/play`) ; en natif par `ALACOD_FPS=1|console`. Le
//! plugin n'est **pas installé** sans demande : éteint, il ne coûte rien. Installé, la touche
//! `F8` montre ou cache la surcouche (la console reste).
//!
//! Une ligne par seconde réelle (`Time<Real>`), dans la console du navigateur ou le journal :
//! `perf fps_display=… fps_sim=… steps=… rollbacks=… advance_avg_ms=… advance_max_ms=… frame_max_ms=… frame=… entities=…`
//! - `fps_display` : mises à jour de la fenêtre (`Last`) par seconde ;
//! - `fps_sim` : progression nette de `FrameCount` par seconde (60 = temps réel) ;
//! - `steps` : passages de `AdvanceWorld` (une frame simulée, resimulations comprises) ;
//! - `rollbacks` : passages de `LoadWorld` ; `advance_*_ms` : coût d'une frame simulée ;
//! - `frame_max_ms` : la plus longue frame d'affichage de la seconde ; `entities` : entités rollback.
//!
//! Dans le navigateur, le dernier échantillon est aussi écrit en JSON dans l'attribut
//! `data-perf-sample` du canvas (lu par les mesures de `docs/taches/rapports/polissage-perf-web.md`).
//! Les chronos `AdvanceWorld` sont posés comme ceux de `utils::snapshot_profile` (aux bornes des
//! schedules de bevy_ggrs, hors `GgrsSchedule`), avec l'horloge de bevy (`web_time` en wasm).

use bevy::{platform::time::Instant, prelude::*};
use bevy_ggrs::{AdvanceWorld, AdvanceWorldSystems, LoadWorld, Rollback};
use std::time::Duration;
use utils::frame::FrameCount;

/// Durée d'une fenêtre de mesure (temps réel).
const WINDOW: Duration = Duration::from_secs(1);

/// Demande de compteur lue dans l'URL (`data-perf` du canvas), l'environnement ou rien.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FpsRequest {
    Overlay,
    ConsoleOnly,
}

impl FpsRequest {
    /// `1` / `true` / `overlay` : surcouche et console ; `console` : console seule ; sinon rien.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "1" | "true" | "overlay" => Some(Self::Overlay),
            "console" => Some(Self::ConsoleOnly),
            _ => None,
        }
    }

    /// Navigateur : attribut `data-perf` du canvas, posé par `loader.js` depuis `?perf=`.
    #[cfg(target_arch = "wasm32")]
    pub fn requested() -> Option<Self> {
        crate::jjrs::browser::attribute("data-perf").and_then(|value| Self::parse(&value))
    }

    /// Natif : `ALACOD_FPS`.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn requested() -> Option<Self> {
        std::env::var("ALACOD_FPS")
            .ok()
            .and_then(|value| Self::parse(&value))
    }
}

/// Un échantillon d'une seconde.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FpsSample {
    pub fps_display: f64,
    pub fps_sim: f64,
    pub steps: u32,
    pub rollbacks: u32,
    pub advance_avg_ms: f64,
    pub advance_max_ms: f64,
    pub frame_max_ms: f64,
    pub frame: u32,
    /// Entités rollback au moment de l'échantillon (charge : ennemis, balles, joueurs…).
    pub entities: u32,
}

impl FpsSample {
    pub fn line(&self) -> String {
        format!(
            "perf fps_display={:.1} fps_sim={:.1} steps={} rollbacks={} advance_avg_ms={:.3} advance_max_ms={:.3} frame_max_ms={:.1} frame={} entities={}",
            self.fps_display,
            self.fps_sim,
            self.steps,
            self.rollbacks,
            self.advance_avg_ms,
            self.advance_max_ms,
            self.frame_max_ms,
            self.frame,
            self.entities
        )
    }

    pub fn json(&self) -> String {
        format!(
            "{{\"fps_display\":{:.2},\"fps_sim\":{:.2},\"steps\":{},\"rollbacks\":{},\"advance_avg_ms\":{:.4},\"advance_max_ms\":{:.4},\"frame_max_ms\":{:.2},\"frame\":{},\"entities\":{}}}",
            self.fps_display,
            self.fps_sim,
            self.steps,
            self.rollbacks,
            self.advance_avg_ms,
            self.advance_max_ms,
            self.frame_max_ms,
            self.frame,
            self.entities
        )
    }
}

/// Fenêtre de mesure en cours. Pure (testable sans `App`).
#[derive(Debug, Default)]
struct Window {
    start: Duration,
    start_frame: Option<u32>,
    display_frames: u32,
    frame_max: Duration,
    steps: u32,
    rollbacks: u32,
    advance_total: Duration,
    advance_max: Duration,
}

impl Window {
    /// Ferme la fenêtre si une seconde réelle s'est écoulée, et en ouvre une neuve.
    fn close(&mut self, now: Duration, frame: u32) -> Option<FpsSample> {
        let elapsed = now.saturating_sub(self.start);
        let start_frame = *self.start_frame.get_or_insert(frame);
        if elapsed < WINDOW {
            return None;
        }
        let seconds = elapsed.as_secs_f64();
        let sample = FpsSample {
            fps_display: f64::from(self.display_frames) / seconds,
            fps_sim: (f64::from(frame) - f64::from(start_frame)) / seconds,
            steps: self.steps,
            rollbacks: self.rollbacks,
            advance_avg_ms: if self.steps == 0 {
                0.0
            } else {
                self.advance_total.as_secs_f64() * 1000.0 / f64::from(self.steps)
            },
            advance_max_ms: self.advance_max.as_secs_f64() * 1000.0,
            frame_max_ms: self.frame_max.as_secs_f64() * 1000.0,
            frame,
            entities: 0,
        };
        *self = Window {
            start: now,
            start_frame: Some(frame),
            ..Window::default()
        };
        Some(sample)
    }
}

#[derive(Resource, Default)]
struct FpsCounter {
    window: Window,
    step_started: Option<Instant>,
    last: Option<FpsSample>,
    overlay: bool,
    /// Numéro du dernier échantillon (la surcouche ne réécrit son texte qu'à chaque seconde).
    serial: u32,
}

#[derive(Component)]
struct FpsOverlayText;

pub struct FpsCounterPlugin(pub FpsRequest);

impl Plugin for FpsCounterPlugin {
    fn build(&self, app: &mut App) {
        let overlay = self.0 == FpsRequest::Overlay;
        app.insert_resource(FpsCounter {
            overlay,
            ..FpsCounter::default()
        })
        .add_systems(
            AdvanceWorld,
            (
                begin_step.before(AdvanceWorldSystems::First),
                end_step.after(AdvanceWorldSystems::Last),
            ),
        )
        .add_systems(LoadWorld, count_rollback)
        .add_systems(Update, (toggle_overlay, update_overlay))
        .add_systems(Last, sample);
        if overlay {
            app.add_systems(Startup, spawn_overlay);
        } else {
            app.add_systems(Startup, spawn_overlay_hidden);
        }
        info!("perf compteur FPS actif (F8 : surcouche, une ligne `perf …` par seconde)");
    }
}

fn begin_step(mut counter: ResMut<FpsCounter>) {
    counter.step_started = Some(Instant::now());
}

fn end_step(mut counter: ResMut<FpsCounter>) {
    let Some(started) = counter.step_started.take() else {
        return;
    };
    let elapsed = started.elapsed();
    let window = &mut counter.window;
    window.steps += 1;
    window.advance_total += elapsed;
    window.advance_max = window.advance_max.max(elapsed);
}

fn count_rollback(mut counter: ResMut<FpsCounter>) {
    counter.window.rollbacks += 1;
}

fn sample(
    mut counter: ResMut<FpsCounter>,
    time: Res<Time<Real>>,
    frame: Option<Res<FrameCount>>,
    rollback_entities: Query<(), With<Rollback>>,
) {
    let window = &mut counter.window;
    window.display_frames += 1;
    window.frame_max = window.frame_max.max(time.delta());
    let frame = frame.map_or(0, |frame| frame.frame);
    let Some(sample) = counter.window.close(time.elapsed(), frame) else {
        return;
    };
    let sample = FpsSample {
        entities: rollback_entities.iter().count() as u32,
        ..sample
    };
    info!("{}", sample.line());
    publish(&sample);
    counter.last = Some(sample);
    counter.serial += 1;
}

#[cfg(target_arch = "wasm32")]
fn publish(sample: &FpsSample) {
    if let Some(canvas) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.query_selector("#bevy-canvas").ok().flatten())
    {
        let _ = canvas.set_attribute("data-perf-sample", &sample.json());
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn publish(_sample: &FpsSample) {}

fn overlay_node() -> impl Bundle {
    (
        FpsOverlayText,
        Text::new("perf ..."),
        TextFont {
            font_size: FontSize::Px(14.0),
            ..Default::default()
        },
        TextColor(Color::WHITE),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        GlobalZIndex(1000),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(4.0),
            left: Val::Px(4.0),
            padding: UiRect::all(Val::Px(3.0)),
            ..default()
        },
    )
}

fn spawn_overlay(mut commands: Commands) {
    commands.spawn(overlay_node());
}

fn spawn_overlay_hidden(mut commands: Commands) {
    commands.spawn((overlay_node(), Visibility::Hidden));
}

fn toggle_overlay(
    keys: Res<ButtonInput<KeyCode>>,
    mut counter: ResMut<FpsCounter>,
    mut query: Query<&mut Visibility, With<FpsOverlayText>>,
) {
    if !keys.just_pressed(KeyCode::F8) {
        return;
    }
    counter.overlay = !counter.overlay;
    for mut visibility in &mut query {
        *visibility = if counter.overlay {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn update_overlay(
    counter: Res<FpsCounter>,
    mut shown: Local<u32>,
    mut query: Query<&mut Text, With<FpsOverlayText>>,
) {
    if !counter.overlay || counter.serial == *shown {
        return;
    }
    let Some(sample) = counter.last else {
        return;
    };
    *shown = counter.serial;
    for mut text in &mut query {
        text.0 = format!(
            "affichage {:.0} fps | sim {:.0} fps ({} pas, {} rollbacks)\npas {:.2} ms (max {:.1}) | frame max {:.0} ms | {} entites",
            sample.fps_display,
            sample.fps_sim,
            sample.steps,
            sample.rollbacks,
            sample.advance_avg_ms,
            sample.advance_max_ms,
            sample.frame_max_ms,
            sample.entities
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_parses_the_documented_values_only() {
        assert_eq!(FpsRequest::parse("1"), Some(FpsRequest::Overlay));
        assert_eq!(FpsRequest::parse("console"), Some(FpsRequest::ConsoleOnly));
        assert_eq!(FpsRequest::parse("0"), None);
        assert_eq!(FpsRequest::parse(""), None);
    }

    #[test]
    fn window_reports_rates_per_real_second_and_net_sim_progress() {
        let mut window = Window::default();
        assert!(window.close(Duration::from_millis(0), 100).is_none());
        for _ in 0..30 {
            window.display_frames += 1;
        }
        window.steps = 62;
        window.rollbacks = 2;
        window.advance_total = Duration::from_millis(124);
        window.advance_max = Duration::from_millis(9);
        assert!(window.close(Duration::from_millis(999), 158).is_none());
        // 2 s réelles : 60 frames d'affichage = 30 fps, 58 frames nettes de simulation = 29 fps
        // (les resimulations de rollback comptent dans `steps`, pas dans `fps_sim`).
        window.display_frames = 60;
        let sample = window.close(Duration::from_secs(2), 158).unwrap();
        assert!((sample.fps_display - 30.0).abs() < 1e-9);
        assert!((sample.fps_sim - 29.0).abs() < 1e-9);
        assert_eq!((sample.steps, sample.rollbacks), (62, 2));
        assert!((sample.advance_avg_ms - 2.0).abs() < 1e-9);
        assert!((sample.advance_max_ms - 9.0).abs() < 1e-9);
        // La fenêtre suivante repart de zéro et de la frame courante.
        assert!(window.close(Duration::from_millis(2500), 160).is_none());
        let next = window.close(Duration::from_secs(3), 218).unwrap();
        assert!((next.fps_sim - 60.0).abs() < 1e-9);
        assert_eq!(next.steps, 0);
    }
}
