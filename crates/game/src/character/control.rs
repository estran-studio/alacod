use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

// === Leafwing Input Actions ===
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum PlayerAction {
    #[actionlike(DualAxis)]
    Pan,

    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,

    Interaction,
    Sprint,
    Dash,

    SwitchWeapon,
    SwitchWeaponMode,

    Reload,
    MeleeAttack,
    DropWeapon,
    /// Objet actif (M2-T0b) : Espace ; blank : Q.
    UseActive,
    Blank,
    /// Bombe (M3-T0c) : touche E.
    Bomb,
    /// Choix de mutation (T1.10) : touches 1/2/3.
    ChoiceA,
    ChoiceB,
    ChoiceC,
    /// Écran de mutation (T1.16) : surbrillance à gauche / à droite (flèches, D-pad) et
    /// validation de la carte surlignée (Entrée, A) — état UI local ; seule la validation
    /// atteint la simulation, en bit `ChoiceA/B/C` (`ui::mutation_screen::confirm_input`).
    ChoicePrev,
    ChoiceNext,
    ChoiceConfirm,

    Modifier,

    PointerPosition,
    PointerClick,

    SwitchLockMode,
    SwitchToUnlockMode,
    SwitchTargetPlayer,

    MoveCameraUp,
    MoveCameraDown,
    MoveCameraLeft,
    MoveCameraRight,

    DebugForceCrash,
}

/// Seuil d'un axe du stick gauche pour compter comme une direction (`Move*`) : la simulation ne
/// connaît que quatre boutons de direction (`INPUT_UP/DOWN/LEFT/RIGHT`, bits inchangés), le stick
/// en pousse donc un par axe dépassé, soit huit directions.
pub const STICK_MOVE_THRESHOLD: f32 = 0.4;

/// Longueur minimale du stick droit pour viser (`input_devices::GamepadAim`) ; en dessous, la
/// dernière direction est gardée.
pub const STICK_AIM_THRESHOLD: f32 = 0.3;

/// Table des liaisons, **unique source** : le jeu la lit (`attach_local_player_input`), l'écran
/// d'aide de la pause l'affiche (`ui::screens::help`) et `docs/jouer.md` la recopie.
///
/// Manette (disposition Xbox, schéma du 2026-10-22, `docs/taches/polissage-manette.md`) : stick
/// gauche déplacement, stick droit visée, RT tir, LT objet actif, RB arme, LB esquive, L3 sprint,
/// R3 mode d'arme, X recharger, Y interagir, B mêlée, A confirmer / bombe, croix : haut blank, bas
/// lâcher l'arme, gauche/droite choix de mutation ; Start pause, Select inventaire. `Modifier` n'a
/// pas de liaison manette (dash arrière, clavier seulement).
pub fn get_input_map() -> InputMap<PlayerAction> {
    let mut map = InputMap::new([
        (PlayerAction::DebugForceCrash, KeyCode::F12),
        (PlayerAction::MoveUp, KeyCode::KeyW),
        (PlayerAction::MoveCameraUp, KeyCode::ArrowUp),
        (PlayerAction::MoveDown, KeyCode::KeyS),
        (PlayerAction::MoveCameraDown, KeyCode::ArrowDown),
        (PlayerAction::MoveLeft, KeyCode::KeyA),
        (PlayerAction::MoveCameraLeft, KeyCode::ArrowLeft),
        (PlayerAction::MoveRight, KeyCode::KeyD),
        (PlayerAction::MoveRight, KeyCode::ArrowRight),
        (PlayerAction::Interaction, KeyCode::KeyH),
        (PlayerAction::SwitchWeapon, KeyCode::Tab),
        (PlayerAction::SwitchWeaponMode, KeyCode::KeyZ),
        (PlayerAction::Reload, KeyCode::KeyR),
        (PlayerAction::MeleeAttack, KeyCode::KeyF),
        (PlayerAction::DropWeapon, KeyCode::KeyL),
        (PlayerAction::UseActive, KeyCode::Space),
        (PlayerAction::Blank, KeyCode::KeyQ),
        (PlayerAction::Bomb, KeyCode::KeyE),
        (PlayerAction::ChoiceA, KeyCode::Digit1),
        (PlayerAction::ChoiceB, KeyCode::Digit2),
        (PlayerAction::ChoiceC, KeyCode::Digit3),
        (PlayerAction::ChoicePrev, KeyCode::ArrowLeft),
        (PlayerAction::ChoiceNext, KeyCode::ArrowRight),
        (PlayerAction::ChoiceConfirm, KeyCode::Enter),
        (PlayerAction::MoveCameraRight, KeyCode::ArrowRight),
        (PlayerAction::Sprint, KeyCode::ShiftLeft),
        (PlayerAction::Dash, KeyCode::KeyC),
        (PlayerAction::Modifier, KeyCode::ControlLeft),
    ]);
    // Manette : sticks.
    map.insert(
        PlayerAction::MoveUp,
        GamepadControlDirection::LEFT_UP.threshold(STICK_MOVE_THRESHOLD),
    );
    map.insert(
        PlayerAction::MoveDown,
        GamepadControlDirection::LEFT_DOWN.threshold(STICK_MOVE_THRESHOLD),
    );
    map.insert(
        PlayerAction::MoveLeft,
        GamepadControlDirection::LEFT_LEFT.threshold(STICK_MOVE_THRESHOLD),
    );
    map.insert(
        PlayerAction::MoveRight,
        GamepadControlDirection::LEFT_RIGHT.threshold(STICK_MOVE_THRESHOLD),
    );
    map.insert(PlayerAction::Sprint, GamepadButton::LeftThumb);
    map.insert(PlayerAction::SwitchWeaponMode, GamepadButton::RightThumb);
    // Gâchettes : RT tire, LT objet actif. Analogiques : le seuil (mi-course, avec hystérésis) est
    // posé par `input_devices::trigger_pressed`, leafwing pressant l'action dès 0,02 de course.
    map.insert(PlayerAction::PointerClick, GamepadButton::RightTrigger2);
    map.insert(PlayerAction::UseActive, GamepadButton::LeftTrigger2);
    // Épaules : RB arme, LB esquive.
    map.insert(PlayerAction::SwitchWeapon, GamepadButton::RightTrigger);
    map.insert(PlayerAction::Dash, GamepadButton::LeftTrigger);
    // Boutons de face.
    map.insert(PlayerAction::Interaction, GamepadButton::North);
    map.insert(PlayerAction::Reload, GamepadButton::West);
    map.insert(PlayerAction::MeleeAttack, GamepadButton::East);
    map.insert(PlayerAction::ChoiceConfirm, GamepadButton::South);
    // Sud sert aussi de bombe ; `collect_device_inputs` la retire quand un écran ou un choix de
    // mutation est ouvert (Sud y valide).
    map.insert(PlayerAction::Bomb, GamepadButton::South);
    // Croix : blank, lâcher, choix de mutation (elle ne déplace plus).
    map.insert(PlayerAction::Blank, GamepadButton::DPadUp);
    map.insert(PlayerAction::DropWeapon, GamepadButton::DPadDown);
    map.insert(PlayerAction::ChoicePrev, GamepadButton::DPadLeft);
    map.insert(PlayerAction::ChoiceNext, GamepadButton::DPadRight);

    map.insert(PlayerAction::PointerClick, MouseButton::Left);

    map.insert(PlayerAction::SwitchLockMode, KeyCode::KeyP);
    map.insert(PlayerAction::SwitchToUnlockMode, KeyCode::KeyO);

    map.with_dual_axis(PlayerAction::Pan, GamepadStick::RIGHT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::input::gamepad::{
        GamepadConnection, GamepadConnectionEvent, RawGamepadAxisChangedEvent,
        RawGamepadButtonChangedEvent, RawGamepadEvent,
    };
    use bevy::input::InputPlugin;

    /// Une manette connectée, un joueur avec la table des liaisons (le vrai chemin de leafwing).
    struct Pad {
        app: App,
        pad: Entity,
        player: Entity,
    }

    impl Pad {
        fn new() -> Self {
            let mut app = App::new();
            app.add_plugins((
                MinimalPlugins,
                InputPlugin,
                InputManagerPlugin::<PlayerAction>::default(),
            ));
            let pad = app.world_mut().spawn_empty().id();
            app.world_mut().write_message(GamepadConnectionEvent::new(
                pad,
                GamepadConnection::Connected {
                    name: "test".into(),
                    vendor_id: None,
                    product_id: None,
                },
            ));
            let player = app
                .world_mut()
                .spawn((ActionState::<PlayerAction>::default(), get_input_map()))
                .id();
            app.update();
            Self { app, pad, player }
        }

        fn button(&mut self, button: GamepadButton, value: f32) {
            self.app.world_mut().write_message(RawGamepadEvent::Button(
                RawGamepadButtonChangedEvent::new(self.pad, button, value),
            ));
            self.app.update();
        }

        fn axis(&mut self, axis: GamepadAxis, value: f32) {
            self.app.world_mut().write_message(RawGamepadEvent::Axis(
                RawGamepadAxisChangedEvent::new(self.pad, axis, value),
            ));
            self.app.update();
        }

        fn actions(&self) -> &ActionState<PlayerAction> {
            self.app
                .world()
                .get::<ActionState<PlayerAction>>(self.player)
                .unwrap()
        }
    }

    /// Les sticks et la croix : stick gauche = déplacement à seuil par axe, croix = blank, lâcher,
    /// choix, mais jamais déplacement ; stick droit = visée.
    #[test]
    fn sticks_et_croix_suivent_le_schema() {
        let mut pad = Pad::new();
        pad.axis(GamepadAxis::LeftStickY, 0.3);
        assert!(
            !pad.actions().pressed(&PlayerAction::MoveUp),
            "sous le seuil"
        );
        pad.axis(GamepadAxis::LeftStickY, 0.9);
        assert!(pad.actions().pressed(&PlayerAction::MoveUp));
        assert!(!pad.actions().pressed(&PlayerAction::MoveDown));
        pad.axis(GamepadAxis::LeftStickY, 0.0);
        pad.axis(GamepadAxis::LeftStickX, -0.9);
        assert!(pad.actions().pressed(&PlayerAction::MoveLeft));
        pad.axis(GamepadAxis::LeftStickX, 0.0);

        pad.axis(GamepadAxis::RightStickY, 1.0);
        assert_eq!(
            pad.actions().axis_pair(&PlayerAction::Pan),
            Vec2::new(0.0, 1.0)
        );
        assert!(
            !pad.actions().pressed(&PlayerAction::MoveUp),
            "le stick droit ne déplace pas"
        );
        pad.axis(GamepadAxis::RightStickY, 0.0);

        pad.button(GamepadButton::DPadUp, 1.0);
        assert!(pad.actions().pressed(&PlayerAction::Blank));
        assert!(
            !pad.actions().pressed(&PlayerAction::MoveUp),
            "la croix ne déplace plus"
        );
        pad.button(GamepadButton::DPadUp, 0.0);
        pad.button(GamepadButton::DPadDown, 1.0);
        assert!(pad.actions().pressed(&PlayerAction::DropWeapon));
        pad.button(GamepadButton::DPadDown, 0.0);
        pad.button(GamepadButton::DPadLeft, 1.0);
        assert!(pad.actions().pressed(&PlayerAction::ChoicePrev));
        assert!(!pad.actions().pressed(&PlayerAction::MoveLeft));
    }

    /// La course analogique de RT et LT arrive jusqu'à l'action (`button_value`) : le seuil de tir est
    /// posé par `input_devices::trigger_pressed`, car leafwing tient l'action pour pressée dès 0,02.
    #[test]
    fn la_course_des_gachettes_arrive_a_l_action() {
        let mut pad = Pad::new();
        pad.button(GamepadButton::RightTrigger2, 0.2);
        let value = pad.actions().button_value(&PlayerAction::PointerClick);
        assert!((value - 0.2).abs() < 1e-6, "RT : {value}");
        pad.button(GamepadButton::LeftTrigger2, 0.9);
        let value = pad.actions().button_value(&PlayerAction::UseActive);
        assert!((value - 0.9).abs() < 1e-6, "LT : {value}");
        pad.button(GamepadButton::RightTrigger2, 0.0);
        assert_eq!(pad.actions().button_value(&PlayerAction::PointerClick), 0.0);
    }

    /// Sud est confirmer et bombe à la fois (le contexte tranche dans `collect_device_inputs`) ;
    /// LT est l'objet actif, RB l'arme, LB l'esquive.
    #[test]
    fn boutons_de_face_et_epaules() {
        let mut pad = Pad::new();
        pad.button(GamepadButton::South, 1.0);
        assert!(pad.actions().pressed(&PlayerAction::Bomb));
        assert!(pad.actions().pressed(&PlayerAction::ChoiceConfirm));
        pad.button(GamepadButton::South, 0.0);
        for (button, action) in [
            (GamepadButton::LeftTrigger2, PlayerAction::UseActive),
            (GamepadButton::RightTrigger, PlayerAction::SwitchWeapon),
            (GamepadButton::LeftTrigger, PlayerAction::Dash),
            (GamepadButton::LeftThumb, PlayerAction::Sprint),
            (GamepadButton::RightThumb, PlayerAction::SwitchWeaponMode),
            (GamepadButton::West, PlayerAction::Reload),
            (GamepadButton::North, PlayerAction::Interaction),
            (GamepadButton::East, PlayerAction::MeleeAttack),
        ] {
            pad.button(button, 1.0);
            assert!(pad.actions().pressed(&action), "{button:?} -> {action:?}");
            pad.button(button, 0.0);
            assert!(!pad.actions().pressed(&action), "{button:?} relâché");
        }
    }
}
