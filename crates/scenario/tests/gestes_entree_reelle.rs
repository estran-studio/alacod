//! T-3b (`docs/conventions.md` § 123) : chaque geste du joueur part d'une **entrée réelle** : un `CursorMoved` de fenêtre
//! (fenêtre factice, sans rendu), les touches et le clic, ou les axes et boutons d'une manette, convertis par le vrai chemin
//! de l'application (leafwing, `collect_device_inputs`, `read_local_inputs`, puis l'enregistreur `record_local_inputs`),
//! jamais par `PlayerScript::aim` ni un segment écrit à la main. L'enregistrement est rejoué par le runner (la simulation et
//! la décision de pose de P-1d, `ModelPlayback`), qui vérifie le geste : clip joué (`Clip`), cap du corps (`ModelYaw`).
//!
//! Le seul rôle que le harnais joue à la place de winit : poser la position du curseur de la fenêtre à la lecture d'un
//! `CursorMoved` (ce que fait `bevy_winit` à chaque événement). La caméra du jeu est centrée sur le joueur (vue 800 × 600,
//! 1 pixel = 1 unité) : le curseur à `(400 + dx, 300 - dy)` vise `(dx, dy)` depuis le joueur, comme en jeu caméra suivie.

use bevy::camera::{ComputedCameraValues, RenderTargetInfo};
use bevy::input::gamepad::{
    GamepadConnection, GamepadConnectionEvent, RawGamepadAxisChangedEvent,
    RawGamepadButtonChangedEvent, RawGamepadEvent,
};
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::input::mouse::MouseButtonInput;
use bevy::input::{ButtonState, InputPlugin};
use bevy::prelude::*;
use bevy::window::{CursorMoved, PrimaryWindow};
use bevy_ggrs::LocalPlayers;
use game::camera::GameCamera;
use game::character::control::PlayerAction;
use game::character::input_devices::{attach_local_player_input, collect_device_inputs};
use game::character::player::input::{read_local_inputs, DeviceInputs, InputSource};
use game::character::player::{LocalPlayer, Player};
use game::recording::{record_local_inputs, InputRecorder, RecordedSettings};
use leafwing_input_manager::plugin::InputManagerPlugin;
use scenario::Scenario;
use utils::frame::FrameCount;

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;

/// Ce que `bevy_winit` fait d'un mouvement de souris : la position du curseur de la fenêtre.
fn fake_window_backend(mut moved: MessageReader<CursorMoved>, mut windows: Query<&mut Window>) {
    for event in moved.read() {
        if let Ok(mut window) = windows.get_mut(event.window) {
            window.set_cursor_position(Some(event.position));
        }
    }
}

/// Avance la frame de la « session » du harnais (comme GGRS : une frame par update).
fn advance_frame(mut frame: ResMut<FrameCount>) {
    frame.frame += 1;
}

/// Une fenêtre, une manette, le joueur local, et le chemin d'input de l'application jusqu'à l'enregistreur.
struct Devices {
    app: App,
    window: Entity,
    pad: Entity,
}

impl Devices {
    fn new(base: &Scenario) -> Self {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            InputPlugin,
            InputManagerPlugin::<PlayerAction>::default(),
        ));
        // `CursorMoved` est d'ordinaire déclaré par `WindowPlugin` (absent : aucune vraie fenêtre).
        app.add_message::<CursorMoved>();
        app.insert_resource(InputSource::Devices)
            .init_resource::<DeviceInputs>()
            .insert_resource(LocalPlayers(vec![0]))
            .insert_resource(FrameCount::default())
            .insert_resource(InputRecorder::new(RecordedSettings::from_scenario(base)));
        app.add_systems(PreUpdate, (fake_window_backend, attach_local_player_input));
        // L'ordre de `ReadInputs` (`CharacterAppPlugin`, `recording`) : périphériques, input local, enregistrement.
        app.add_systems(
            Update,
            (
                collect_device_inputs,
                read_local_inputs,
                record_local_inputs,
                advance_frame,
            )
                .chain(),
        );

        let mut window = Window::default();
        window.resolution.set(WIDTH, HEIGHT);
        let window = app.world_mut().spawn((window, PrimaryWindow)).id();
        app.world_mut().spawn((
            Camera2d,
            Camera {
                computed: ComputedCameraValues {
                    clip_from_view: Mat4::orthographic_rh(
                        -WIDTH / 2.0,
                        WIDTH / 2.0,
                        -HEIGHT / 2.0,
                        HEIGHT / 2.0,
                        -1000.0,
                        1000.0,
                    ),
                    target_info: Some(RenderTargetInfo {
                        physical_size: UVec2::new(WIDTH as u32, HEIGHT as u32),
                        scale_factor: 1.0,
                    }),
                    ..default()
                },
                ..default()
            },
            GlobalTransform::default(),
            GameCamera::default(),
        ));
        app.world_mut()
            .spawn((Transform::default(), Player::default(), LocalPlayer {}));
        let pad = app.world_mut().spawn_empty().id();
        app.world_mut().write_message(GamepadConnectionEvent::new(
            pad,
            GamepadConnection::Connected {
                name: "manette de test".into(),
                vendor_id: None,
                product_id: None,
            },
        ));
        Self { app, window, pad }
    }

    fn frame(&self) -> u32 {
        self.app.world().resource::<FrameCount>().frame
    }

    /// Joue les updates jusqu'à la frame `frame` (exclue).
    fn until(&mut self, frame: u32) {
        while self.frame() < frame {
            self.app.update();
        }
    }

    /// Le curseur se déplace pour viser `(dx, dy)` depuis le joueur (y vers le haut).
    fn cursor(&mut self, dx: f32, dy: f32) {
        let window = self.window;
        self.app.world_mut().write_message(CursorMoved {
            window,
            position: Vec2::new(WIDTH / 2.0 + dx, HEIGHT / 2.0 - dy),
            delta: None,
        });
    }

    fn key(&mut self, key_code: KeyCode, pressed: bool) {
        let window = self.window;
        self.app.world_mut().write_message(KeyboardInput {
            key_code,
            logical_key: Key::Unidentified(NativeKey::Unidentified),
            state: if pressed {
                ButtonState::Pressed
            } else {
                ButtonState::Released
            },
            text: None,
            repeat: false,
            window,
        });
    }

    fn click(&mut self, pressed: bool) {
        let window = self.window;
        self.app.world_mut().write_message(MouseButtonInput {
            button: MouseButton::Left,
            state: if pressed {
                ButtonState::Pressed
            } else {
                ButtonState::Released
            },
            window,
        });
    }

    fn button(&mut self, button: GamepadButton, value: f32) {
        let pad = self.pad;
        self.app.world_mut().write_message(RawGamepadEvent::Button(
            RawGamepadButtonChangedEvent::new(pad, button, value),
        ));
    }

    fn axis(&mut self, axis: GamepadAxis, value: f32) {
        let pad = self.pad;
        self.app
            .world_mut()
            .write_message(RawGamepadEvent::Axis(RawGamepadAxisChangedEvent::new(
                pad, axis, value,
            )));
    }

    /// Le scénario de base, avec les inputs enregistrés (`InputRecorder::to_scenario`, le chemin de `ALACOD_RECORD`).
    fn recorded(&mut self, base: &Scenario) -> Scenario {
        self.until(base.frames);
        let recorded = self
            .app
            .world()
            .resource::<InputRecorder>()
            .to_scenario(None);
        assert!(
            !recorded.players.is_empty() && !recorded.players[0].inputs.is_empty(),
            "aucun input enregistré depuis les périphériques"
        );
        Scenario {
            players: recorded.players,
            ..base.clone()
        }
    }
}

fn base(source: &str) -> Scenario {
    Scenario::from_ron(source).expect("scénario de base valide")
}

/// Joue le scénario enregistré ; échoue sur toute attente ou invariant manqué (dont `animation_vivante`).
fn play(name: &str, scenario: &Scenario) {
    for (n, segment) in scenario.players[0].inputs.iter().enumerate() {
        eprintln!("{name} : segment {n} : {segment:?}");
    }
    let outcome = scenario::run(scenario);
    assert!(
        outcome.failures.is_empty(),
        "{name} : {}",
        outcome.failures.join("\n")
    );
}

/// Pantin au pistolet (arme par défaut du personnage, `bare_hands` en mêlée) : visée, tir, rechargement, coup, pas.
const PANTIN: &str = r#"Scenario(
    powerup_drop_chance_override: "0.0",
    game: "testbed",
    map: "testbed/arena.ldtk",
    frames: 230,
    players: [(character: "pantin")],
    expect: [
        ModelYaw(who: Player(0), toward: Direction(Left), at_frame: 40),
        Clip(who: Player(0), clip: "shoot", once: true, at_frame: 52),
        Clip(who: Player(0), clip: "reload", once: true, at_frame: 85),
        // Rechargement du pistolet (1 s) : le clip de 1,2 s étiré, vitesse 1,2
        ClipSpeed(who: Player(0), min: 1.15, max: 1.25, at_frame: 90),
        Clip(who: Player(0), clip: "strike", once: true, at_frame: 148),
        ModelYaw(who: Player(0), toward: Direction(Up), at_frame: 184),
        Clip(who: Player(0), clip: "roll", once: true, at_frame: 192),
    ],
)"#;

/// Pantin à l'arme à gestes (`arme_a_gestes` : épaulé de 9 frames, geste du tir, rechargement automatique).
const PANTIN_GESTES: &str = r#"Scenario(
    powerup_drop_chance_override: "0.0",
    game: "testbed",
    map: "testbed/arena.ldtk",
    frames: 140,
    players: [(character: "pantin", weapon: "arme_a_gestes")],
    expect: [
        ModelYaw(who: Player(0), toward: Direction(Right), at_frame: 28),
        // Épaulé : `raise_frames` (9) sur le clip de 0,15 s, au rythme du manifeste
        Clip(who: Player(0), clip: "shoulder", once: true, at_frame: 40),
        ClipSpeed(who: Player(0), min: 0.95, max: 1.05, at_frame: 40),
        // Geste du tir (`fire_frames` 18 = le clip de 0,3 s), puis rechargement automatique (1,2 s)
        Clip(who: Player(0), clip: "shoot", once: true, at_frame: 52),
        Clip(who: Player(0), clip: "reload", once: true, at_frame: 90),
        ClipSpeed(who: Player(0), min: 0.95, max: 1.05, at_frame: 90),
        ClipProgress(who: Player(0), min: 0.9, max: 1.0, at_frame: 133),
    ],
)"#;

#[test]
fn gestes_du_pantin_depuis_la_souris_et_le_clavier() {
    let base = base(PANTIN);
    let mut devices = Devices::new(&base);
    devices.until(20);
    devices.cursor(-120.0, 0.0); // visée à gauche
    devices.until(45);
    devices.click(true); // tir
    devices.until(47);
    devices.click(false);
    devices.until(70);
    devices.key(KeyCode::KeyR, true); // rechargement
    devices.until(72);
    devices.key(KeyCode::KeyR, false);
    devices.until(140);
    devices.key(KeyCode::KeyF, true); // coup (mêlée)
    devices.until(142);
    devices.key(KeyCode::KeyF, false);
    devices.until(175);
    devices.cursor(0.0, 120.0); // visée vers le haut
    devices.key(KeyCode::KeyW, true);
    devices.until(186);
    devices.key(KeyCode::KeyC, true); // pas (esquive)
    devices.until(188);
    devices.key(KeyCode::KeyC, false);
    devices.until(215);
    devices.key(KeyCode::KeyW, false);
    let scenario = devices.recorded(&base);
    play("souris", &scenario);
}

#[test]
fn gestes_du_pantin_depuis_la_manette() {
    let base = base(PANTIN);
    let mut devices = Devices::new(&base);
    devices.until(20);
    devices.axis(GamepadAxis::RightStickX, -1.0); // visée à gauche
    devices.until(45);
    devices.button(GamepadButton::RightTrigger2, 1.0); // tir (RT)
    devices.until(47);
    devices.button(GamepadButton::RightTrigger2, 0.0);
    devices.until(70);
    devices.button(GamepadButton::West, 1.0); // rechargement (X)
    devices.until(72);
    devices.button(GamepadButton::West, 0.0);
    devices.until(140);
    devices.button(GamepadButton::East, 1.0); // coup (B)
    devices.until(142);
    devices.button(GamepadButton::East, 0.0);
    devices.until(175);
    devices.axis(GamepadAxis::RightStickX, 0.0);
    devices.axis(GamepadAxis::RightStickY, 1.0); // visée vers le haut
    devices.axis(GamepadAxis::LeftStickY, 1.0); // marche vers le haut
    devices.until(186);
    devices.button(GamepadButton::LeftTrigger, 1.0); // pas (LB)
    devices.until(188);
    devices.button(GamepadButton::LeftTrigger, 0.0);
    devices.until(215);
    devices.axis(GamepadAxis::LeftStickY, 0.0);
    let scenario = devices.recorded(&base);
    play("manette", &scenario);
}

#[test]
fn epaule_tir_et_rechargement_depuis_la_souris() {
    let base = base(PANTIN_GESTES);
    let mut devices = Devices::new(&base);
    devices.until(10);
    devices.cursor(150.0, 0.0); // visée à droite
    devices.until(30);
    devices.click(true); // un clic : épaulé, tir, rechargement automatique
    devices.until(32);
    devices.click(false);
    let scenario = devices.recorded(&base);
    play("souris, arme à gestes", &scenario);
}

#[test]
fn epaule_tir_et_rechargement_depuis_la_manette() {
    let base = base(PANTIN_GESTES);
    let mut devices = Devices::new(&base);
    devices.until(10);
    devices.axis(GamepadAxis::RightStickX, 1.0); // visée à droite
    devices.until(30);
    devices.button(GamepadButton::RightTrigger2, 1.0);
    devices.until(32);
    devices.button(GamepadButton::RightTrigger2, 0.0);
    let scenario = devices.recorded(&base);
    play("manette, arme à gestes", &scenario);
}

/// X-6d (§ 126) : un agrippeur s'accroche au pantin immobile (f64) ; le joueur s'en défait depuis l'entrée réelle (secouage au
/// stick, aux touches, ou coup de mêlée) : décroché, il retombe étourdi (pose `stunned`). Sans geste, il resterait accroché
/// jusqu'à f244 (`grab_bond_accroche`, `grab_secoue_lent`).
const AGRIPPEUR: &str = r#"Scenario(
    powerup_drop_chance_override: "0.0",
    game: "testbed",
    map: "testbed/arena.ldtk",
    frames: 140,
    characters: [(character: "agrippeur", x: "540.0", y: "696.0", at_frame: 1)],
    players: [(character: "pantin")],
    expect: [
        GrabState(entity: Placed(0), phase: Ride, carrier: Some(0), at_frame: 95),
        GrabState(entity: Placed(0), phase: Dazed, at_frame: 135),
        Clip(who: Entity(Placed(0)), clip: "stunned", once: Some(false), at_frame: 138),
    ],
)"#;

/// Secoue : quatre alternances de direction, une toutes les 4 frames (`press` tient une direction, `release` la lâche).
fn shake(devices: &mut Devices, mut press: impl FnMut(&mut Devices, i8)) {
    devices.until(100);
    for (n, side) in [-1i8, 1, -1, 1, -1].into_iter().enumerate() {
        press(devices, side);
        devices.until(104 + 4 * n as u32);
    }
    press(devices, 0);
}

#[test]
fn agrippeur_secoue_au_stick() {
    let base = base(AGRIPPEUR);
    let mut devices = Devices::new(&base);
    shake(&mut devices, |d, side| {
        d.axis(GamepadAxis::LeftStickX, side as f32)
    });
    let scenario = devices.recorded(&base);
    play("agrippeur, stick", &scenario);
}

#[test]
fn agrippeur_secoue_au_clavier() {
    let base = base(AGRIPPEUR);
    let mut devices = Devices::new(&base);
    shake(&mut devices, |d, side| {
        d.key(KeyCode::KeyA, side < 0);
        d.key(KeyCode::KeyD, side > 0);
    });
    let scenario = devices.recorded(&base);
    play("agrippeur, clavier", &scenario);
}

#[test]
fn agrippeur_frappe_depuis_la_manette() {
    let base = base(AGRIPPEUR);
    let mut devices = Devices::new(&base);
    devices.until(110);
    devices.button(GamepadButton::East, 1.0); // coup (B)
    devices.until(112);
    devices.button(GamepadButton::East, 0.0);
    let scenario = devices.recorded(&base);
    play("agrippeur, coup à la manette", &scenario);
}
