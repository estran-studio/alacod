//! Esquive des projectiles (T1.14, profil `prudent` v1) : fonction pure en `Fixed`, testée sans
//! Bevy. Voir `docs/conventions.md` §24 (v1).
//!
//! Pour chaque projectile de la vue qui **approche** (`rel · v < 0`), point d'approche minimale
//! de sa trajectoire linéaire (vitesse en unités par frame) dans les [`HORIZON_FRAMES`]
//! prochaines frames. Menace si cette distance est sous le rayon du corps + la taille du
//! projectile + [`MARGIN`]. Direction
//! d'esquive : perpendiculaire à la vitesse du projectile, du côté qui s'éloigne du point
//! d'approche (à gauche de la trajectoire si le projectile arrive droit dessus) ; somme sur les
//! menaces, normalisée.

use bevy_fixed::fixed_math::{Fixed, FixedVec2, FIXED_ZERO};

use crate::view::{BotView, ProjectileView};

/// Horizon de prédiction, en frames.
pub const HORIZON_FRAMES: i32 = 30;
/// Marge ajoutée au rayon du corps et à la taille du projectile.
pub const MARGIN: Fixed = Fixed::from_bits(8 << 16);

/// Vecteur (non normalisé) qui écarte le corps de la trajectoire de `p`, ou `None` si `p` ne
/// menace pas. `rel` = position du projectile relative au bot.
fn threat(p: &ProjectileView, from: FixedVec2, body_radius: Fixed) -> Option<FixedVec2> {
    let rel = p.position - from;
    let v = p.velocity;
    let speed_sq = v.dot(&v);
    // Immobile, ou qui s'éloigne (ou passe perpendiculairement) : pas une menace
    if speed_sq <= FIXED_ZERO || rel.dot(&v) >= FIXED_ZERO {
        return None;
    }
    // Instant d'approche minimale t* = -(rel·v)/(v·v), borné à [0, horizon]
    let t = (-(rel.dot(&v)) / speed_sq).clamp(FIXED_ZERO, Fixed::from_num(HORIZON_FRAMES));
    let closest = rel + v * t;
    let reach = body_radius + p.size + MARGIN;
    // Composantes d'abord : évite un carré hors de `Fixed` pour un projectile lointain
    if closest.x.abs() >= reach || closest.y.abs() >= reach || closest.length() >= reach {
        return None;
    }
    let left = FixedVec2::new(-v.y, v.x);
    // S'éloigner du point d'approche : côté opposé à `closest` ; droit dessus : à gauche
    Some(if left.dot(&closest) > FIXED_ZERO {
        FixedVec2::new(v.y, -v.x)
    } else {
        left
    })
}

/// Direction d'esquive (normalisée) si au moins un projectile menace, sinon `None`.
pub fn dodge(view: &BotView) -> Option<FixedVec2> {
    let mut sum = FixedVec2::ZERO;
    let mut any = false;
    for p in &view.projectiles {
        if let Some(away) = threat(p, view.position, view.body_radius) {
            sum = sum + away.normalize_or_zero();
            any = true;
        }
    }
    if !any {
        return None;
    }
    let dir = sum.normalize_or_zero();
    // Deux menaces exactement opposées s'annulent : garder la première plutôt que rien
    if dir == FixedVec2::ZERO {
        return view
            .projectiles
            .iter()
            .find_map(|p| threat(p, view.position, view.body_radius))
            .map(|away| away.normalize_or_zero());
    }
    Some(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fixed::fixed_math::new as fx;

    fn view_with(projectiles: Vec<ProjectileView>) -> BotView {
        BotView {
            position: FixedVec2::new(fx(0.0), fx(0.0)),
            health: fx(100.0),
            health_max: fx(100.0),
            ammo: 10,
            wave: 0,
            nearest_enemy: None,
            nearest_window: None,
            hunter: None,
            portal: None,
            projectiles,
            body_radius: fx(10.0),
            reload: false,
            switch_weapon: false,
            trigger_ready: true,
            velocity: FixedVec2::ZERO,
            enemy_visible: false,
            route: None,
        }
    }

    fn proj(x: f32, y: f32, vx: f32, vy: f32) -> ProjectileView {
        let position = FixedVec2::new(fx(x), fx(y));
        ProjectileView {
            position,
            velocity: FixedVec2::new(fx(vx), fx(vy)),
            size: fx(5.0),
            distance: position.length(),
        }
    }

    #[test]
    fn projectile_frontal_pas_de_cote() {
        // Arrive de la droite, droit dessus : esquive perpendiculaire (vers le haut ou le bas)
        let dir = dodge(&view_with(vec![proj(100.0, 0.0, -4.0, 0.0)])).expect("menace");
        assert!(
            dir.x.abs() < fx(0.01),
            "pas de recul le long de la trajectoire : {dir:?}"
        );
        assert!(dir.y.abs() > fx(0.99));
    }

    #[test]
    fn projectile_qui_passe_a_cote_ignore() {
        // Passe à 60 px au-dessus : pas de menace
        assert_eq!(dodge(&view_with(vec![proj(100.0, 60.0, -4.0, 0.0)])), None);
        // S'éloigne : pas de menace
        assert_eq!(dodge(&view_with(vec![proj(20.0, 0.0, 4.0, 0.0)])), None);
        // Trop lent pour arriver dans l'horizon (30 frames × 1 px < 200 px)
        assert_eq!(dodge(&view_with(vec![proj(200.0, 0.0, -1.0, 0.0)])), None);
    }

    #[test]
    fn s_eloigne_du_point_d_approche() {
        // Passe à 10 px au-dessus du centre : esquiver vers le bas
        let dir = dodge(&view_with(vec![proj(100.0, 10.0, -4.0, 0.0)])).expect("menace");
        assert!(dir.y < fx(-0.99), "{dir:?}");
    }

    #[test]
    fn deux_menaces_sommees() {
        // Une par la droite qui passe au-dessus (→ bas), une par le haut qui passe à droite
        // (→ gauche) : somme vers le bas-gauche
        let dir = dodge(&view_with(vec![
            proj(100.0, 10.0, -4.0, 0.0),
            proj(10.0, 100.0, 0.0, -4.0),
        ]))
        .expect("menaces");
        assert!(dir.x < fx(-0.6) && dir.y < fx(-0.6), "{dir:?}");
    }
}
