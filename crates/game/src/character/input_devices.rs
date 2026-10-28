//! Lecture des périphériques du joueur local, moitié **application** (D69 étape 2, E4) : clavier, manette
//! et souris (leafwing), fenêtre et caméra. La simulation (`character::player::input::read_local_inputs`)
//! ne lit plus que la ressource [`DeviceInputs`] que ce module écrit ; elle ne nomme ni `ActionState`, ni
//! `Window`, ni `Camera`.
//!
//! Absent en headless (`CharacterAppPlugin::headless`) : aucun périphérique, aucun leafwing, input neutre.
//!
//! Chemin final (E9) : `game_app/src/character/input_devices.rs`.

use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use leafwing_input_manager::prelude::ActionState;

use crate::character::control::{get_input_map, PlayerAction, STICK_AIM_THRESHOLD};
use crate::character::player::input::{
    BoxInput, DeviceInputs, InputSource, INPUT_BLANK, INPUT_BOMB, INPUT_CHOICE_A, INPUT_CHOICE_B,
    INPUT_CHOICE_C, INPUT_DASH, INPUT_DOWN, INPUT_DROP_WEAPON, INPUT_FORCE_CRASH,
    INPUT_INTERACTION, INPUT_LEFT, INPUT_MELEE_ATTACK, INPUT_MODIFIER, INPUT_RELOAD, INPUT_RIGHT,
    INPUT_SPRINT, INPUT_SWITCH_WEAPON_MODE, INPUT_UP, INPUT_USE_ACTIVE,
};
use crate::character::player::{LocalPlayer, Player};

/// `PreUpdate`, avant `RunGgrsSystems` : le joueur local (créé par la simulation, sans périphérique) reçoit son
/// `ActionState` et son `InputMap` (et `AiDriver` sous `harmonium`). D4 : l'`ActionState` arrive au plus
/// une frame de rendu après la création du joueur ; sans effet sur les inputs (neutres jusque-là).
pub fn attach_local_player_input(mut commands: Commands, added: Query<Entity, Added<LocalPlayer>>) {
    for entity in &added {
        commands
            .entity(entity)
            .try_insert((ActionState::<PlayerAction>::default(), get_input_map()));

        #[cfg(feature = "harmonium")]
        commands
            .entity(entity)
            .try_insert(harmonium_bevy::components::AiDriver {
                ai_influence: 1.0,
                detection_radius: 300.0,
                ..default()
            });
    }
}

/// Visée à la manette, gardée d'une frame à l'autre : application, hors rollback (l'input qui part
/// vers GGRS est, lui, la seule source de la simulation).
#[derive(Default, Debug, Clone, PartialEq)]
pub struct GamepadAim {
    /// Dernière direction du stick droit (`pan_x`, `pan_y` dans `[-127, 127]`) ; `None` : le
    /// curseur (souris) vise.
    direction: Option<(i16, i16)>,
    /// Position du curseur (pixels de la fenêtre) quand la direction a été prise ou revue : le
    /// curseur qui bouge ensuite reprend la main.
    cursor: Option<Vec2>,
}

/// Déplacement du curseur (pixels) qui rend la visée à la souris.
const CURSOR_MOVED_PIXELS: f32 = 3.0;

/// Course d'une gâchette (0 à 1) à partir de laquelle elle tire, et en dessous de laquelle elle
/// relâche : la moitié, avec une marge de relâchement pour ne pas hacher le tir.
const TRIGGER_PRESS: f32 = 0.5;
const TRIGGER_RELEASE: f32 = 0.35;

/// État des gâchettes d'un joueur d'une frame à l'autre (hystérésis).
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriggerLatch {
    fire: bool,
    active: bool,
}

/// Une gâchette (ou un bouton : valeur 0 ou 1) est-elle pressée ? `was` : son état à la frame
/// d'avant, qui tient entre les deux seuils.
pub fn trigger_pressed(value: f32, was: bool) -> bool {
    if value >= TRIGGER_PRESS {
        true
    } else if value <= TRIGGER_RELEASE {
        false
    } else {
        was
    }
}

/// Direction d'un stick (`None` sous le seuil), en `pan` : le vecteur normalisé fois 127, l'échelle
/// des bots (`aim_direction` ne lit que la direction).
pub fn stick_pan(stick: Vec2) -> Option<(i16, i16)> {
    if stick.length() < STICK_AIM_THRESHOLD {
        return None;
    }
    let unit = stick.normalize();
    Some((
        (unit.x * 127.0).round() as i16,
        (unit.y * 127.0).round() as i16,
    ))
}

/// La visée de cette frame : le stick droit s'il est poussé ; sans lui, la dernière direction du
/// stick tant que le curseur n'a pas bougé ; sinon le curseur (`mouse`).
pub fn resolve_aim(
    state: &mut GamepadAim,
    stick: Vec2,
    cursor: Option<Vec2>,
    mouse: Option<(i16, i16)>,
) -> Option<(i16, i16)> {
    if let Some(direction) = stick_pan(stick) {
        state.direction = Some(direction);
        state.cursor = cursor.or(state.cursor);
        return Some(direction);
    }
    if let Some(direction) = state.direction {
        // Un curseur vu pour la première fois (la souris entre dans la page : le navigateur envoie
        // un mouvement à la prise de focus) n'a pas encore bougé : c'est la position de départ.
        if let (Some(now), None) = (cursor, state.cursor) {
            state.cursor = Some(now);
        }
        let moved = match (cursor, state.cursor) {
            (Some(now), Some(then)) => now.distance(then) > CURSOR_MOVED_PIXELS,
            _ => false,
        };
        if !moved {
            return Some(direction);
        }
        state.direction = None;
    }
    mouse
}

/// Boutons que l'input garde tant qu'un écran (pause, inventaire, marchand) est ouvert : le
/// déplacement et le sprint. La simulation continue sous l'écran, et A, B, la croix y servent à
/// naviguer : sans ce filtre, valider « Reprendre » lâcherait une bombe, retour donnerait un coup.
const SCREEN_OPEN_KEEPS: u32 = INPUT_UP | INPUT_DOWN | INPUT_LEFT | INPUT_RIGHT | INPUT_SPRINT;

/// `ReadInputs`, avant `read_local_inputs` : l'input de chaque joueur local (`InputSource::Devices`) d'après son
/// `ActionState`, la position du curseur dans le monde (fenêtre, caméra) et la carte de mutation
/// (`confirm_input`). Écrit [`DeviceInputs`], lue par la simulation.
pub fn collect_device_inputs(
    input_source: Res<InputSource>,
    players: Query<(&ActionState<PlayerAction>, &Transform, &Player), With<LocalPlayer>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mutation_screen: Option<Res<crate::ui::mutation_screen::MutationScreenView>>,
    screens: Option<Res<crate::ui::screens::Screens>>,
    mut aims: Local<BTreeMap<usize, GamepadAim>>,
    mut muted: Local<BTreeMap<usize, u32>>,
    mut triggers: Local<BTreeMap<usize, TriggerLatch>>,
    mut device_inputs: ResMut<DeviceInputs>,
) {
    device_inputs.0.clear();
    if *input_source != InputSource::Devices {
        return;
    }
    let screen_open = screens.is_some_and(|screens| screens.state.open.is_some());
    for (action_state, transform, player) in &players {
        let triggers = triggers.entry(player.handle).or_default();
        let mut input = BoxInput::default();

        if action_state.pressed(&PlayerAction::MoveUp) {
            input.buttons |= INPUT_UP;
        }
        if action_state.pressed(&PlayerAction::MoveDown) {
            input.buttons |= INPUT_DOWN;
        }
        if action_state.pressed(&PlayerAction::MoveLeft) {
            input.buttons |= INPUT_LEFT;
        }
        if action_state.pressed(&PlayerAction::MoveRight) {
            input.buttons |= INPUT_RIGHT;
        }

        // Tir et objet actif : les gâchettes sont analogiques, et leafwing tient une action pour
        // pressée dès 0,02 de course (`ActionState::set_button_value`) ; le seuil est posé ici.
        let fire = trigger_pressed(
            action_state.button_value(&PlayerAction::PointerClick),
            triggers.fire,
        );
        triggers.fire = fire;
        if fire {
            input.fire = true;
        }

        if action_state.pressed(&PlayerAction::SwitchWeapon) {
            input.switch_weapon = true;
        }
        if action_state.pressed(&PlayerAction::SwitchWeaponMode) {
            input.buttons |= INPUT_SWITCH_WEAPON_MODE;
        }

        if action_state.pressed(&PlayerAction::Reload) {
            input.buttons |= INPUT_RELOAD;
        }

        if action_state.pressed(&PlayerAction::Sprint) {
            input.buttons |= INPUT_SPRINT;
        }

        if action_state.pressed(&PlayerAction::Dash) {
            input.buttons |= INPUT_DASH;
        }

        if action_state.pressed(&PlayerAction::Interaction) {
            input.buttons |= INPUT_INTERACTION;
        }

        if action_state.pressed(&PlayerAction::Modifier) {
            input.buttons |= INPUT_MODIFIER;
        }

        if action_state.pressed(&PlayerAction::MeleeAttack) {
            input.buttons |= INPUT_MELEE_ATTACK;
        }

        if action_state.just_pressed(&PlayerAction::DropWeapon) {
            input.buttons |= INPUT_DROP_WEAPON;
        }

        let active = trigger_pressed(
            action_state.button_value(&PlayerAction::UseActive),
            triggers.active,
        );
        triggers.active = active;
        if active {
            input.buttons |= INPUT_USE_ACTIVE;
        }
        if action_state.pressed(&PlayerAction::Blank) {
            input.buttons |= INPUT_BLANK;
        }
        if action_state.pressed(&PlayerAction::Bomb) {
            input.buttons |= INPUT_BOMB;
        }

        if action_state.pressed(&PlayerAction::ChoiceA) {
            input.buttons |= INPUT_CHOICE_A;
        }
        if action_state.pressed(&PlayerAction::ChoiceB) {
            input.buttons |= INPUT_CHOICE_B;
        }
        if action_state.pressed(&PlayerAction::ChoiceC) {
            input.buttons |= INPUT_CHOICE_C;
        }
        // T1.16 : valider sur l'écran de mutation = le bit de la carte surlignée. Sud (A) y valide
        // et sert de bombe ailleurs : la bombe ne part pas pendant qu'il valide un choix.
        if action_state.pressed(&PlayerAction::ChoiceConfirm) {
            let confirm = crate::ui::mutation_screen::confirm_input(
                mutation_screen.as_deref(),
                player.handle,
            );
            if confirm != 0 {
                input.buttons &= !INPUT_BOMB;
            }
            input.buttons |= confirm;
        }

        // F12 to force crash (debug)
        if action_state.pressed(&PlayerAction::DebugForceCrash) {
            input.buttons |= INPUT_FORCE_CRASH;
        }

        // Visée : curseur (souris) ou stick droit (`resolve_aim`), recalculée à chaque frame. La
        // direction gardée du stick reste ici (application, hors rollback) : la simulation ne reçoit
        // que `pan_x`/`pan_y` de l'input, comme avec la souris.
        let cursor = q_window
            .single()
            .ok()
            .and_then(|window| window.cursor_position());
        let mouse = cursor.and_then(|cursor_position| {
            let (camera, camera_transform) = q_camera.single().ok()?;
            let world_position = camera
                .viewport_to_world_2d(camera_transform, cursor_position)
                .ok()?;
            let pointer_distance = world_position - transform.translation.truncate();
            let clamp = |v: f32| v.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            Some((clamp(pointer_distance.x), clamp(pointer_distance.y)))
        });
        let stick = action_state.axis_pair(&PlayerAction::Pan);
        if let Some((pan_x, pan_y)) =
            resolve_aim(aims.entry(player.handle).or_default(), stick, cursor, mouse)
        {
            input.pan_x = pan_x;
            input.pan_y = pan_y;
        }

        // Un écran ouvert garde le clavier et la manette pour lui (voir `SCREEN_OPEN_KEEPS`) ; un
        // bouton tenu pendant l'écran reste muet jusqu'à son relâchement (B qui ferme la pause
        // ne donne pas un coup de mêlée au retour dans la partie).
        let muted = muted.entry(player.handle).or_default();
        if screen_open {
            *muted |= input.buttons & !SCREEN_OPEN_KEEPS;
            input.buttons &= SCREEN_OPEN_KEEPS;
            input.fire = false;
            input.switch_weapon = false;
        } else {
            *muted &= input.buttons;
            input.buttons &= !*muted;
        }

        device_inputs.0.insert(player.handle, input);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::player::input::read_local_inputs;
    use crate::character::player::jjrs::PeerConfig;
    use bevy::ecs::system::RunSystemOnce;
    use bevy_ggrs::{LocalInputs, LocalPlayers};
    use utils::frame::FrameCount;

    /// Maintenir Lâcher n'envoie qu'une requête et se réarme au relâchement ; maintenir Interaction reste
    /// supporté. Le test passe par `collect_device_inputs` puis `read_local_inputs`, comme `ReadInputs`.
    #[test]
    fn held_drop_sends_one_request_and_rearms_after_release() {
        let mut world = World::new();
        world.insert_resource(InputSource::Devices);
        world.init_resource::<DeviceInputs>();
        world.insert_resource(LocalPlayers(vec![0]));
        world.insert_resource(FrameCount::default());
        let mut actions = ActionState::<PlayerAction>::default();
        actions.press(&PlayerAction::DropWeapon);
        actions.press(&PlayerAction::Interaction);
        let player = world
            .spawn((
                actions,
                Transform::default(),
                Player::default(),
                LocalPlayer {},
            ))
            .id();
        let read = |world: &mut World| {
            world.run_system_once(collect_device_inputs).unwrap();
            world.run_system_once(read_local_inputs).unwrap();
            world.resource::<LocalInputs<PeerConfig>>().0[&0].buttons
        };
        assert_ne!(read(&mut world) & INPUT_DROP_WEAPON, 0);
        let instant = bevy::time::Time::<bevy::time::Real>::default().startup();
        for _ in 0..120 {
            {
                let mut entity = world.entity_mut(player);
                let mut actions = entity.get_mut::<ActionState<PlayerAction>>().unwrap();
                actions.tick(instant, instant);
                actions.press(&PlayerAction::DropWeapon);
            }
            let buttons = read(&mut world);
            assert_eq!(buttons & INPUT_DROP_WEAPON, 0, "held drop must not repeat");
            assert_ne!(
                buttons & INPUT_INTERACTION,
                0,
                "holding interaction remains supported"
            );
        }
        {
            let mut entity = world.entity_mut(player);
            let mut actions = entity.get_mut::<ActionState<PlayerAction>>().unwrap();
            actions.release(&PlayerAction::DropWeapon);
            actions.tick(instant, instant);
            actions.press(&PlayerAction::DropWeapon);
        }
        assert_ne!(read(&mut world) & INPUT_DROP_WEAPON, 0);
    }

    /// Un joueur local sans périphérique (headless, `DeviceInputs` absente) envoie un input neutre :
    /// aucune entrée, aucune panique.
    #[test]
    fn without_device_inputs_the_local_player_is_neutral() {
        let mut world = World::new();
        world.insert_resource(InputSource::Devices);
        world.insert_resource(LocalPlayers(vec![0]));
        world.insert_resource(FrameCount::default());
        world.run_system_once(read_local_inputs).unwrap();
        assert_eq!(world.resource::<LocalInputs<PeerConfig>>().0[&0].buttons, 0);
    }

    #[test]
    fn le_stick_droit_vise_et_garde_sa_direction_au_repos() {
        assert_eq!(stick_pan(Vec2::new(0.1, 0.1)), None, "sous le seuil");
        assert_eq!(stick_pan(Vec2::new(1.0, 0.0)), Some((127, 0)));
        assert_eq!(stick_pan(Vec2::new(0.0, -0.5)), Some((0, -127)));
        assert_eq!(stick_pan(Vec2::new(0.4, 0.4)), Some((90, 90)));

        let mut aim = GamepadAim::default();
        let cursor = Some(Vec2::new(300.0, 200.0));
        let mouse = Some((-40, 12));
        // Sans stick ni direction gardée : la souris.
        assert_eq!(resolve_aim(&mut aim, Vec2::ZERO, cursor, mouse), mouse);
        // Le stick l'emporte, puis, relâché, sa dernière direction tient tant que le curseur reste.
        assert_eq!(
            resolve_aim(&mut aim, Vec2::new(0.0, 1.0), cursor, mouse),
            Some((0, 127))
        );
        assert_eq!(
            resolve_aim(&mut aim, Vec2::ZERO, cursor, mouse),
            Some((0, 127))
        );
        // Un curseur qui frétille d'un pixel ne reprend pas la main ; sortir du cadre non plus.
        assert_eq!(
            resolve_aim(&mut aim, Vec2::ZERO, Some(Vec2::new(301.0, 200.0)), mouse),
            Some((0, 127))
        );
        assert_eq!(
            resolve_aim(&mut aim, Vec2::ZERO, None, None),
            Some((0, 127))
        );
        // Un curseur qui bouge pour de bon rend la visée à la souris, et la garde.
        let moved = Some(Vec2::new(380.0, 200.0));
        assert_eq!(resolve_aim(&mut aim, Vec2::ZERO, moved, mouse), mouse);
        assert_eq!(resolve_aim(&mut aim, Vec2::ZERO, moved, mouse), mouse);

        // Le curseur qui apparaît après le stick (focus de la page) ne reprend pas la main ;
        // il le fait quand il bouge ensuite.
        let mut aim = GamepadAim::default();
        assert_eq!(
            resolve_aim(&mut aim, Vec2::new(1.0, 0.0), None, None),
            Some((127, 0))
        );
        assert_eq!(
            resolve_aim(&mut aim, Vec2::ZERO, cursor, mouse),
            Some((127, 0))
        );
        assert_eq!(resolve_aim(&mut aim, Vec2::ZERO, moved, mouse), mouse);
    }

    /// Une gâchette tire à la moitié de sa course, relâche sous le tiers, et tient entre les deux :
    /// un effleurement ne tire pas (leafwing, lui, presse l'action dès 0,02).
    #[test]
    fn la_gachette_tire_a_mi_course_avec_hysteresis() {
        assert!(!trigger_pressed(0.2, false), "effleurement");
        assert!(trigger_pressed(0.5, false), "mi-course");
        assert!(trigger_pressed(0.4, true), "tenue entre les seuils");
        assert!(!trigger_pressed(0.4, false), "pas encore entre les seuils");
        assert!(!trigger_pressed(0.3, true), "relâchée");
        assert!(trigger_pressed(1.0, false), "souris et clavier valent 1");

        let mut actions = ActionState::<PlayerAction>::default();
        actions.set_button_value(&PlayerAction::PointerClick, 0.2);
        let (mut world, player) = world_with_player(actions);
        let system = world.register_system(collect_device_inputs);
        let fire = |world: &World| world.resource::<DeviceInputs>().0[&0].fire;
        world.run_system(system).unwrap();
        assert!(!fire(&world), "RT à 0,2");
        for (value, expected) in [(0.6, true), (0.4, true), (0.2, false)] {
            world
                .entity_mut(player)
                .get_mut::<ActionState<PlayerAction>>()
                .unwrap()
                .set_button_value(&PlayerAction::PointerClick, value);
            world.run_system(system).unwrap();
            assert_eq!(fire(&world), expected, "RT à {value}");
        }
    }

    fn world_with_player(actions: ActionState<PlayerAction>) -> (World, Entity) {
        let mut world = World::new();
        world.insert_resource(InputSource::Devices);
        world.init_resource::<DeviceInputs>();
        let player = world
            .spawn((
                actions,
                Transform::default(),
                Player::default(),
                LocalPlayer {},
            ))
            .id();
        (world, player)
    }

    fn buttons_of(world: &World) -> u32 {
        world.resource::<DeviceInputs>().0[&0].buttons
    }

    /// Sous un écran ouvert, A, B et la croix naviguent : ils ne lâchent ni bombe ni coup, et un
    /// bouton tenu pendant l'écran reste muet à sa fermeture jusqu'à son relâchement. Le déplacement
    /// continue (la simulation tourne sous l'écran).
    #[test]
    fn un_ecran_ouvert_garde_la_manette_pour_lui() {
        let mut actions = ActionState::<PlayerAction>::default();
        actions.press(&PlayerAction::MoveLeft);
        actions.press(&PlayerAction::MeleeAttack);
        actions.press(&PlayerAction::Bomb);
        let (mut world, player) = world_with_player(actions);
        let system = world.register_system(collect_device_inputs);

        world.run_system(system).unwrap();
        assert_eq!(
            buttons_of(&world),
            INPUT_LEFT | INPUT_MELEE_ATTACK | INPUT_BOMB,
            "sans écran, tout part"
        );

        let mut screens = crate::ui::screens::Screens::default();
        screens.state.open = Some(crate::ui::screens::model::ScreenId::Pause);
        world.insert_resource(screens);
        world.run_system(system).unwrap();
        assert_eq!(
            buttons_of(&world),
            INPUT_LEFT,
            "sous l'écran : déplacement seul"
        );

        world
            .resource_mut::<crate::ui::screens::Screens>()
            .state
            .open = None;
        world.run_system(system).unwrap();
        assert_eq!(
            buttons_of(&world),
            INPUT_LEFT,
            "mêlée et bombe tenues pendant l'écran restent muets"
        );

        let mut entity = world.entity_mut(player);
        let mut actions = entity.get_mut::<ActionState<PlayerAction>>().unwrap();
        actions.release(&PlayerAction::MeleeAttack);
        drop(entity);
        world.run_system(system).unwrap();
        assert_eq!(
            buttons_of(&world) & INPUT_MELEE_ATTACK,
            0,
            "relâchée, puis pressée de nouveau, la mêlée repart"
        );
        world
            .entity_mut(player)
            .get_mut::<ActionState<PlayerAction>>()
            .unwrap()
            .press(&PlayerAction::MeleeAttack);
        world.run_system(system).unwrap();
        assert_ne!(buttons_of(&world) & INPUT_MELEE_ATTACK, 0);
    }

    /// Sud valide la carte surlignée du choix de mutation sans poser de bombe.
    #[test]
    fn sud_valide_la_mutation_sans_poser_de_bombe() {
        use crate::ui::mutation_screen::{MutationCard, MutationScreenView};
        let mut actions = ActionState::<PlayerAction>::default();
        actions.press(&PlayerAction::ChoiceConfirm);
        actions.press(&PlayerAction::Bomb);
        let (mut world, _) = world_with_player(actions);
        let system = world.register_system(collect_device_inputs);

        world.run_system(system).unwrap();
        assert_eq!(
            buttons_of(&world),
            INPUT_BOMB,
            "sans choix en cours : bombe"
        );

        world.insert_resource(MutationScreenView {
            open: true,
            handle: Some(0),
            options: vec![MutationCard::default(); 3],
            highlighted: 1,
            ..Default::default()
        });
        world.run_system(system).unwrap();
        assert_eq!(buttons_of(&world), INPUT_CHOICE_B);
    }
}
