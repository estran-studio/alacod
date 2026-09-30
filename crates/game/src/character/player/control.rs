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
    // Add more bindings...
    map.insert(PlayerAction::PointerClick, MouseButton::Left);

    map.insert(PlayerAction::SwitchLockMode, KeyCode::KeyP);
    map.insert(PlayerAction::SwitchToUnlockMode, KeyCode::KeyO);

    map.with_dual_axis(PlayerAction::Pan, GamepadStick::LEFT)
}
