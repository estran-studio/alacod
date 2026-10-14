pub use combat::actors::DashState;

/// Dash imposé par un scénario (`PlayerScript::dash`, M2-T2) : remplace le `dash` du RON du
/// personnage. Hors rollback et jamais muté (comme `Tags`) : posé avant la première frame.
#[derive(Component, Clone, Debug)]
pub struct DashOverride(pub crate::character::movement::DashConfig);

use bevy::prelude::*;
use bevy_ggrs::Rollback;
use combat::actors::{DashExtra, Enemy, Velocity};
use utils::{net_id::GgrsNetId, order_iter, order_mut_iter};

use crate::collider::{is_colliding, Collider};

/// Bousculade (`DashEffect::Shove`, M2-T2, §43) : pendant un dash dont `shove_force` est non
/// nulle, chaque ennemi dont le collider touche celui du joueur reçoit un recul de cette force
/// dans la direction du dash (`Velocity.knockback`, assigné : un contact qui dure ne l'empile
/// pas). Ordre : joueurs puis ennemis par `GgrsNetId`.
pub fn dash_shove_system(
    players: Query<
        (
            &GgrsNetId,
            &DashState,
            &DashExtra,
            &Transform3dFixed,
            &Collider,
        ),
        With<Rollback>,
    >,
    mut enemies: Query<
        (&GgrsNetId, &Transform3dFixed, &Collider, &mut Velocity),
        (With<Enemy>, With<Rollback>),
    >,
) {
    for (_, dash, extra, player_pos, player_collider) in order_iter!(players) {
        if !dash.is_dashing || extra.shove_force <= fixed_math::FIXED_ZERO {
            continue;
        }
        for (_, enemy_pos, enemy_collider, mut velocity) in order_mut_iter!(enemies) {
            if is_colliding(
                &player_pos.translation,
                player_collider,
                &enemy_pos.translation,
                enemy_collider,
            ) {
                velocity.knockback = dash.dash_direction * extra.shove_force;
            }
        }
    }
}
type Transform3dFixed = fixed_math::FixedTransform3D;
use bevy_fixed::fixed_math;
