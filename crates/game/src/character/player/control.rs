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

// Utility function to create the input map
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
        (PlayerAction::DropWeapon, KeyCode::KeyG),
        (PlayerAction::UseActive, KeyCode::Space),
        (PlayerAction::Blank, KeyCode::KeyQ),
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
    // Add gamepad support if needed
    map.insert(PlayerAction::MoveUp, GamepadButton::DPadUp);
    map.insert(PlayerAction::MoveDown, GamepadButton::DPadDown);
    map.insert(PlayerAction::MoveLeft, GamepadButton::DPadLeft);
    map.insert(PlayerAction::MoveRight, GamepadButton::DPadRight);
    map.insert(PlayerAction::Interaction, GamepadButton::North);
    map.insert(PlayerAction::Reload, GamepadButton::West);
    map.insert(PlayerAction::MeleeAttack, GamepadButton::East);
    map.insert(PlayerAction::ChoicePrev, GamepadButton::DPadLeft);
    map.insert(PlayerAction::ChoiceNext, GamepadButton::DPadRight);
    map.insert(PlayerAction::ChoiceConfirm, GamepadButton::South);
    // Add more bindings...
    map.insert(PlayerAction::PointerClick, MouseButton::Left);

    map.insert(PlayerAction::SwitchLockMode, KeyCode::KeyP);
    map.insert(PlayerAction::SwitchToUnlockMode, KeyCode::KeyO);

    map.with_dual_axis(PlayerAction::Pan, GamepadStick::LEFT)
}
