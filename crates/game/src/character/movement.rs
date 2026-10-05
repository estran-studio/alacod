use bevy::prelude::*;
use bevy_fixed::fixed_math;
use bevy_ggrs::Rollback;
use serde::Deserialize;
use utils::{net_id::GgrsNetId, order_mut_iter};

/// Déplacement d'un personnage. Les ennemis n'en lisent que `max_speed` (leur IA décide du
/// reste, voir `enemy::ai::pathing`) ; pour un joueur, la course suit [`run_velocity`] et le
/// dash [`combat::actors::DashState`].
#[derive(Deserialize, Debug, Clone)]
pub struct MovementConfig {
    /// Prise de vitesse (px/s²) d'un joueur dans le sens des touches. Aussi l'adhérence :
    /// un modificateur de la stat `Acceleration` (surface glacée...) change dans la même
    /// proportion le freinage (`deceleration`).
    pub acceleration: fixed_math::Fixed,
    pub max_speed: fixed_math::Fixed,
    /// Freinage (px/s²) : touches relâchées, vitesse au-dessus de la cible ou demi-tour.
    /// Absent : même valeur que `acceleration`.
    #[serde(default)]
    pub deceleration: Option<fixed_math::Fixed>,
    pub sprint_multiplier: fixed_math::Fixed, // How much faster sprint is (e.g. 2.0 for double speed)
    pub sprint_acceleration_per_frame: fixed_math::Fixed, // How much sprint increases each frame (0-1)
    pub sprint_deceleration_per_frame: fixed_math::Fixed,

    pub dash_distance: fixed_math::Fixed, // Distance exacte du dash (px), hors mur
    pub dash_duration_frames: u32,        // Frames du dash, celle de l'appui comprise
    pub dash_cooldown_frames: u32,        // Frames entre deux départs de dash
    /// Frames d'invulnérabilité au début du dash, celle de l'appui comprise (0 : aucune).
    #[serde(default)]
    pub dash_iframes: u32,
    /// Un appui qui ne peut pas lancer de dash (dash ou cooldown en cours) est gardé ce
    /// nombre de frames, celle de l'appui comprise (0 : seul l'appui compte).
    #[serde(default)]
    pub dash_buffer_frames: u32,
}

impl MovementConfig {
    pub fn deceleration(&self) -> fixed_math::Fixed {
        self.deceleration.unwrap_or(self.acceleration)
    }
}

/// Adhérence : rapport entre l'accélération résolue (stat `Acceleration`, modificateurs
/// compris) et sa valeur de config. 1 sans modificateur ; s'applique aussi au freinage.
pub fn grip(acceleration: fixed_math::Fixed, base: fixed_math::Fixed) -> fixed_math::Fixed {
    if base > fixed_math::FIXED_ZERO {
        acceleration / base
    } else {
        fixed_math::FIXED_ONE
    }
}

/// Un axe de la course : `v` va vers `target` d'au plus `accel` (prise de vitesse dans le
/// sens de la cible) ou d'au plus `decel` (cible plus lente, nulle ou de sens opposé).
/// `accel` et `decel` sont des variations de vitesse par frame.
pub fn approach_axis(
    v: fixed_math::Fixed,
    target: fixed_math::Fixed,
    accel: fixed_math::Fixed,
    decel: fixed_math::Fixed,
) -> fixed_math::Fixed {
    let speeding_up = target.abs() > v.abs()
        && (v == fixed_math::FIXED_ZERO
            || (v > fixed_math::FIXED_ZERO) == (target > fixed_math::FIXED_ZERO));
    let rate = if speeding_up { accel } else { decel };
    v + (target - v).clamp(-rate, rate)
}

/// Vitesse de course de la frame : chaque axe va vers la vitesse visée (`target`, direction
/// des touches × vitesse max, nulle sans touche) par [`approach_axis`]. Par axe plutôt
/// qu'en norme : arithmétique exacte en `Fixed`, sans racine, et un demi-tour sur un axe ne
/// freine pas l'autre.
pub fn run_velocity(
    v: fixed_math::FixedVec2,
    target: fixed_math::FixedVec2,
    accel: fixed_math::Fixed,
    decel: fixed_math::Fixed,
) -> fixed_math::FixedVec2 {
    fixed_math::FixedVec2::new(
        approach_axis(v.x, target.x, accel, decel),
        approach_axis(v.y, target.y, accel, decel),
    )
}

/// Resource for configuring knockback damping
/// IMPORTANT: Uses Fixed instead of f32 for determinism across rollback
#[derive(Resource, Clone, Debug, Hash)]
pub struct KnockbackDampingConfig {
    pub damping: fixed_math::Fixed, // e.g., 0.85 means 15% decay per frame
}

impl Default for KnockbackDampingConfig {
    fn default() -> Self {
        Self {
            damping: fixed_math::new(0.85),
        }
    }
}

/// System to apply damping to knockback velocity each frame
/// IMPORTANT: Uses order_mut_iter for deterministic iteration order
pub fn apply_knockback_damping(
    mut query: Query<(&GgrsNetId, &mut Velocity), With<Rollback>>,
    config: Res<KnockbackDampingConfig>,
) {
    let damping = config.damping;
    for (_net_id, mut velocity) in order_mut_iter!(query) {
        velocity.knockback = velocity.knockback * damping;
        // If knockback is very small, zero it out
        if velocity.knockback.length_squared() < fixed_math::new(0.01) {
            velocity.knockback = fixed_math::FixedVec2::ZERO;
        }
    }
}

pub use combat::actors::{SprintState, Velocity};

#[cfg(test)]
mod tests {
    use super::*;
    use fixed_math::{Fixed, FixedVec2};

    /// Réglage des joueurs (`player_config.ron`) en variation de vitesse par frame :
    /// 3000 px/s² × 1/60 s.
    fn step() -> Fixed {
        Fixed::from_num(3000) * fixed_math::new(1.0 / 60.0)
    }

    fn speed(v: f32) -> Fixed {
        Fixed::from_num(v)
    }

    /// Frames pour que `v` atteigne `target` à 0,1 px/s près.
    fn frames_to(mut v: FixedVec2, target: FixedVec2, accel: Fixed, decel: Fixed) -> u32 {
        for frame in 1..=200 {
            v = run_velocity(v, target, accel, decel);
            if (v.x - target.x).abs() < speed(0.1) && (v.y - target.y).abs() < speed(0.1) {
                return frame;
            }
        }
        panic!("cible jamais atteinte");
    }

    #[test]
    fn course_nerveuse() {
        let max = FixedVec2::new(speed(150.0), speed(0.0));
        let back = FixedVec2::new(speed(-150.0), speed(0.0));
        // Vitesse max en 3 frames, arrêt en 3, demi-tour en 6
        assert_eq!(frames_to(FixedVec2::ZERO, max, step(), step()), 3);
        assert_eq!(frames_to(max, FixedVec2::ZERO, step(), step()), 3);
        assert_eq!(frames_to(max, back, step(), step()), 6);
    }

    #[test]
    fn la_glace_fait_glisser() {
        // Adhérence 0,2 (`surfaces/glace.ron`) : accélération et freinage cinq fois plus lents
        let ice = grip(speed(600.0), speed(3000.0));
        let (accel, decel) = (step() * ice, step() * ice);
        let max = FixedVec2::new(speed(150.0), speed(0.0));
        assert_eq!(frames_to(FixedVec2::ZERO, max, accel, decel), 15);
        assert_eq!(frames_to(max, FixedVec2::ZERO, accel, decel), 15);
        assert_eq!(grip(speed(10.0), Fixed::ZERO), fixed_math::FIXED_ONE);
    }

    #[test]
    fn freinage_et_acceleration_distincts() {
        // Accélère dans le sens de la cible, freine sinon (cible plus lente ou opposée)
        let (accel, decel) = (speed(10.0), speed(40.0));
        assert_eq!(
            approach_axis(speed(0.0), speed(100.0), accel, decel),
            speed(10.0)
        );
        assert_eq!(
            approach_axis(speed(0.0), speed(-100.0), accel, decel),
            speed(-10.0)
        );
        assert_eq!(
            approach_axis(speed(100.0), speed(0.0), accel, decel),
            speed(60.0)
        );
        assert_eq!(
            approach_axis(speed(100.0), speed(50.0), accel, decel),
            speed(60.0)
        );
        assert_eq!(
            approach_axis(speed(100.0), speed(-100.0), accel, decel),
            speed(60.0)
        );
        assert_eq!(
            approach_axis(speed(-100.0), speed(-100.0), accel, decel),
            speed(-100.0)
        );
        // Ne dépasse jamais la cible
        assert_eq!(
            approach_axis(speed(5.0), speed(0.0), accel, decel),
            speed(0.0)
        );
        assert_eq!(
            approach_axis(speed(95.0), speed(100.0), accel, decel),
            speed(100.0)
        );
    }

    #[test]
    fn un_demi_tour_sur_un_axe_ne_freine_pas_l_autre() {
        let v = FixedVec2::new(speed(100.0), speed(100.0));
        let target = FixedVec2::new(speed(-100.0), speed(100.0));
        let next = run_velocity(v, target, speed(10.0), speed(40.0));
        assert_eq!(next, FixedVec2::new(speed(60.0), speed(100.0)));
    }
}
