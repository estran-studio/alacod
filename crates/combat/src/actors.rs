//! Données communes aux systèmes d'armes et aux personnages, extraites sans changement de valeurs.
use bevy::prelude::*;
use bevy_fixed::fixed_math;
use ggrs::PlayerHandle;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct Player {
    pub handle: PlayerHandle,
    pub color: Color,
    pub name: String,
    pub pubkey: String,
}

/// Hash manuel : seul `handle` contribue au checksum GGRS. `color` est de la présentation
/// (`f32`, pas de `Hash`) ; `name` et `pubkey` sont des données d'identité **propres à
/// chaque client** en p2p (un client ne connaît que son propre nom, les autres joueurs
/// reçoivent un nom de repli) : les hacher faisait diverger les checksums de tous les
/// clients dès la frame 0 (test p2p headless de la CI de nuit, T2.14) sans que la
/// simulation diffère. Elles ne pilotent rien dans la simulation.
impl std::hash::Hash for Player {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.handle.hash(state);
    }
}

#[derive(Component, Reflect, Default, Debug, Clone, Hash)]
#[reflect(Component)]
pub struct Enemy {}
#[derive(Component, Default, Clone, Debug, Hash)]
pub struct SprintState {
    pub is_sprinting: bool,
    pub sprint_factor: fixed_math::Fixed, // Ranges from 0.0 to 1.0 for gradual acceleration
}

#[derive(Component, Default, Clone, Debug, Hash)]
pub struct Velocity {
    pub main: fixed_math::FixedVec2,
    pub knockback: fixed_math::FixedVec2,
}

/// Esquive (dash) d'un joueur : part sur l'appui (front montant du bouton), avance dès la
/// frame de l'appui, couvre exactement `dash_distance` px en `dash_duration` frames avec une
/// vitesse qui décroît jusqu'à celle de la course (on sort du dash en courant, voir
/// [`dash_step`]). Un appui qui tombe pendant le dash ou le cooldown est gardé
/// `dash_buffer_frames` frames (`MovementConfig`) puis part dès que possible.
#[derive(Component, Default, Clone, Debug, Hash)]
pub struct DashState {
    /// Dash en cours, de la frame de l'appui à sa dernière frame (lu par les armes : pas de
    /// tir pendant un dash, et par les collisions : pas de ralentissement par les ennemis).
    pub is_dashing: bool,
    pub dash_direction: fixed_math::FixedVec2,
    /// Frames du dash pas encore jouées.
    pub dash_frames_remaining: u32,
    pub dash_cooldown_remaining: u32,
    /// Durée (frames) du dash en cours.
    pub dash_duration: u32,
    /// Distance (px) du dash en cours.
    pub dash_distance: fixed_math::Fixed,
    /// Pas (px par frame) de la dernière frame du dash en cours : celui de la course.
    pub dash_end_step: fixed_math::Fixed,
    /// Appui en attente : frames pendant lesquelles il peut encore lancer un dash.
    pub dash_buffer_remaining: u32,
    /// Bouton tenu à la frame précédente : tenir le bouton ne relance pas de dash.
    pub dash_held: bool,
}

impl DashState {
    pub fn can_dash(&self) -> bool {
        !self.is_dashing && self.dash_cooldown_remaining == 0
    }

    /// Début de frame : avance le cooldown et termine le dash dont la dernière frame a été
    /// jouée à la frame précédente.
    pub fn begin_frame(&mut self) {
        self.dash_cooldown_remaining = self.dash_cooldown_remaining.saturating_sub(1);
        if self.is_dashing && self.dash_frames_remaining == 0 {
            self.is_dashing = false;
        }
    }

    /// Lit le bouton de la frame : vrai si un dash est demandé (appui de cette frame, ou
    /// appui gardé depuis moins de `buffer_frames` frames). La frame de l'appui compte dans
    /// le buffer : à 0, seul l'appui lui-même lance un dash.
    pub fn read_button(&mut self, held: bool, buffer_frames: u32) -> bool {
        if held && !self.dash_held {
            self.dash_buffer_remaining = buffer_frames.max(1);
        }
        self.dash_held = held;
        self.dash_buffer_remaining > 0
    }

    /// Fin de frame sans dash lancé : l'appui en attente vieillit d'une frame.
    pub fn age_buffer(&mut self) {
        self.dash_buffer_remaining = self.dash_buffer_remaining.saturating_sub(1);
    }

    /// Lance un dash de `distance` px en `duration_frames` frames qui sort à `end_step` px
    /// par frame (la course), et arme le cooldown (compté depuis cette frame).
    pub fn start_dash(
        &mut self,
        direction: fixed_math::FixedVec2,
        distance: fixed_math::Fixed,
        duration_frames: u32,
        end_step: fixed_math::Fixed,
        cooldown_frames: u32,
    ) {
        let duration = duration_frames.max(1);
        self.is_dashing = true;
        self.dash_direction = direction.normalize_or_zero();
        self.dash_frames_remaining = duration;
        self.dash_duration = duration;
        self.dash_distance = distance;
        self.dash_end_step = end_step;
        self.dash_buffer_remaining = 0;
        self.dash_cooldown_remaining = cooldown_frames;
    }

    /// Pas (px) du dash pour cette frame, `None` hors dash ou une fois ses frames jouées.
    pub fn take_step(&mut self) -> Option<fixed_math::Fixed> {
        if !self.is_dashing || self.dash_frames_remaining == 0 {
            return None;
        }
        let k = self.dash_duration - self.dash_frames_remaining;
        self.dash_frames_remaining -= 1;
        Some(dash_step(
            self.dash_distance,
            self.dash_duration,
            self.dash_end_step,
            k,
        ))
    }
}

/// Pas (px) de la frame `k` (de 0 à `frames - 1`) d'un dash de `distance` px en `frames`
/// frames : décroissance linéaire de `2·distance/frames − end` jusqu'à `end` (px par frame,
/// la vitesse de course à la sortie), donc une somme de `distance` px quelle que soit `end`.
/// `end` est bornée à `distance/frames` : au pire le dash est plat, jamais accéléré.
pub fn dash_step(
    distance: fixed_math::Fixed,
    frames: u32,
    end: fixed_math::Fixed,
    k: u32,
) -> fixed_math::Fixed {
    if frames <= 1 {
        return distance;
    }
    let mean = distance / fixed_math::Fixed::from_num(frames);
    let end = end.clamp(fixed_math::FIXED_ZERO, mean);
    let start = mean + mean - end;
    start
        + (end - start) * fixed_math::Fixed::from_num(k.min(frames - 1))
            / fixed_math::Fixed::from_num(frames - 1)
}
#[derive(Component, Clone, Debug, Serialize, Default, Deserialize, Hash)]
pub struct Health {
    pub current: fixed_math::Fixed,
    pub max: fixed_math::Fixed,
    pub invulnerable_until_frame: Option<u32>, // Optional invulnerability window
}

impl Health {
    /// Invulnérable à `frame` (i-frames du dash, borne comprise) : aucun coup ne la touche,
    /// les balles et projectiles la traversent, les hitbox de mêlée ne la voient pas.
    pub fn is_invulnerable_at(&self, frame: u32) -> bool {
        self.invulnerable_until_frame
            .is_some_and(|until| frame <= until)
    }
}

impl fmt::Display for Health {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HP: {}/{}", self.current, self.max)?;
        if let Some(frame) = self.invulnerable_until_frame {
            write!(f, " (Invulnerable until frame {})", frame)?;
        }
        Ok(())
    }
}

pub const INPUT_UP: u16 = 1 << 0;
pub const INPUT_DOWN: u16 = 1 << 1;
pub const INPUT_LEFT: u16 = 1 << 2;
pub const INPUT_RIGHT: u16 = 1 << 3;
pub const INPUT_RELOAD: u16 = 1 << 4;
pub const INPUT_SWITCH_WEAPON_MODE: u16 = 1 << 5;
pub const INPUT_SPRINT: u16 = 1 << 6;
pub const INPUT_DASH: u16 = 1 << 7;
pub const INPUT_MODIFIER: u16 = 1 << 8;
pub const INPUT_INTERACTION: u16 = 1 << 9;
pub const INPUT_MELEE_ATTACK: u16 = 1 << 10;
pub const INPUT_FORCE_CRASH: u16 = 1 << 11;
/// Lâche l'arme active au sol (T2.2, chantier B7). Touche `L` (`Devices`, voir
/// `character::player::control::get_input_map`), bouton `DropWeapon` des scénarios (voir
/// `game::replay::Button`).
pub const INPUT_DROP_WEAPON: u16 = 1 << 12;
/// Choix de mutation A/B/C (T1.10, `docs/conventions.md` §27) : touches `1`/`2`/`3`
/// (`character::player::control::get_input_map`), boutons `ChoiceA`/`ChoiceB`/`ChoiceC` des
/// scénarios (`game::replay::Button`). Sans effet hors d'un choix ouvert (`MutationChoice`).
pub const INPUT_CHOICE_A: u16 = 1 << 13;
pub const INPUT_CHOICE_B: u16 = 1 << 14;
pub const INPUT_CHOICE_C: u16 = 1 << 15;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BoxInput {
    pub buttons: u16,
    pub pan_x: i16,
    pub pan_y: i16,

    pub fire: bool,
    pub switch_weapon: bool,
}

/// Component for the weapon sprite's position relative to player
///
/// Donnée dérivée de l'input (`pan`), non rollbackée : `apply_inputs` la réécrit à chaque
/// frame, avant tout retour anticipé (dash), et `system_weapon_position` la lit dans la même
/// frame. Ne jamais la lire d'une frame à l'autre ni l'écrire après un `continue` : elle
/// redeviendrait un état caché que le rollback ne restaure pas (scénario `dash_aim_change`).
#[derive(Component, Clone, Copy, Default)]
pub struct CursorPosition {
    pub x: i32,
    pub y: i32,
}

pub type BoxConfig = bevy_ggrs::GgrsConfig<BoxInput>;
pub type PeerConfig = bevy_ggrs::GgrsConfig<BoxInput, bevy_matchbox::prelude::PeerId>;

#[cfg(test)]
mod dash_tests {
    use super::*;
    use fixed_math::{Fixed, FixedVec2};

    fn px(v: f32) -> Fixed {
        Fixed::from_num(v)
    }

    /// Joue un dash lancé à la frame 0 : (pas de chaque frame où il avance, frame de fin).
    fn play(state: &mut DashState, frames: u32) -> Vec<Fixed> {
        let mut steps = Vec::new();
        for _ in 0..frames {
            state.begin_frame();
            if let Some(step) = state.take_step() {
                steps.push(step);
            }
        }
        steps
    }

    #[test]
    fn profil_decroissant_somme_exacte() {
        let steps: Vec<Fixed> = (0..8).map(|k| dash_step(px(64.0), 8, px(2.5), k)).collect();
        let total: Fixed = steps.iter().copied().sum();
        assert!((total - px(64.0)).abs() < px(0.001), "total {total}");
        assert_eq!(steps[0], px(13.5));
        assert_eq!(steps[7], px(2.5));
        assert!(
            steps.windows(2).all(|w| w[1] < w[0]),
            "décroissant : {steps:?}"
        );
    }

    #[test]
    fn profil_plat_si_la_course_est_plus_rapide() {
        // Sortie plus rapide que la moyenne du dash : bornée, le dash ne s'accélère jamais
        for k in 0..4 {
            assert_eq!(dash_step(px(40.0), 4, px(25.0), k), px(10.0));
        }
        assert_eq!(dash_step(px(40.0), 1, px(2.0), 0), px(40.0));
    }

    #[test]
    fn avance_des_la_frame_de_l_appui_puis_sort_en_courant() {
        let mut state = DashState::default();
        state.begin_frame();
        assert!(state.read_button(true, 8));
        assert!(state.can_dash());
        state.start_dash(FixedVec2::new(px(1.0), px(0.0)), px(64.0), 8, px(2.5), 24);
        // Frame 0 : premier pas, le plus long
        assert_eq!(state.take_step(), Some(px(13.5)));
        let rest = play(&mut state, 7);
        assert_eq!(rest.len(), 7);
        assert_eq!(rest[6], px(2.5));
        assert!(state.is_dashing, "dernière frame jouée, encore en dash");
        state.begin_frame();
        assert!(!state.is_dashing, "fin à la frame suivante");
        assert_eq!(state.take_step(), None);
    }

    #[test]
    fn tenir_le_bouton_ne_relance_pas() {
        let mut state = DashState::default();
        assert!(state.read_button(true, 1));
        state.start_dash(FixedVec2::new(px(1.0), px(0.0)), px(64.0), 8, px(2.5), 24);
        for _ in 0..40 {
            state.begin_frame();
            state.take_step();
            assert!(!state.read_button(true, 1), "tenu : pas de nouvel appui");
            state.age_buffer();
        }
        state.begin_frame();
        assert!(!state.read_button(false, 1));
        state.age_buffer();
        state.begin_frame();
        assert!(state.read_button(true, 1), "relâché puis appuyé");
    }

    #[test]
    fn appui_garde_pendant_le_cooldown() {
        let mut state = DashState::default();
        assert!(state.read_button(true, 8));
        state.start_dash(FixedVec2::new(px(1.0), px(0.0)), px(64.0), 8, px(2.5), 24);
        state.take_step();
        // Frames 1 à 17 : bouton relâché ; appui à la frame 18, 6 frames avant la fin du
        // cooldown (frame 24) : gardé, part à la frame 24
        let mut started_at = None;
        for frame in 1..40u32 {
            state.begin_frame();
            let requested = state.read_button(frame == 18, 8);
            if requested && state.can_dash() {
                started_at = Some(frame);
                break;
            }
            state.take_step();
            state.age_buffer();
        }
        assert_eq!(started_at, Some(24));
    }

    #[test]
    fn appui_trop_tot_oublie() {
        let mut state = DashState::default();
        assert!(state.read_button(true, 8));
        state.start_dash(FixedVec2::new(px(1.0), px(0.0)), px(64.0), 8, px(2.5), 24);
        state.take_step();
        // Appui à la frame 10 : 14 frames avant la fin du cooldown, plus que le buffer (8)
        for frame in 1..40u32 {
            state.begin_frame();
            let requested = state.read_button(frame == 10, 8);
            assert!(
                !(requested && state.can_dash()),
                "appui oublié, aucun dash à la frame {frame}"
            );
            state.take_step();
            state.age_buffer();
        }
    }
}
