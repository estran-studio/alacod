//! Agrippeur accroché à un joueur (X-6d, `docs/conventions.md` §126).
//!
//! Le behavior `Grab` (`behaviors::Behavior::Grab`, exécuté par `game`) pose [`Riding`] sur l'agrippeur au
//! moment où il s'accroche, et le retire quand il se décroche. Le composant vit ici, dans le cœur de la
//! simulation, parce que des crates qui ne voient pas `game` en ont besoin : `combat` (une balle du porteur
//! ne touche pas l'agrippeur qu'il porte sur les épaules) ; `game` le lit aussi pour le joueur (déviation du
//! porteur, ralentissement « poussé par un ennemi » qui ne compte pas ses propres agrippeurs) et pour
//! l'agrippeur (il suit son porteur).
//!
//! # Rollback
//!
//! Composant rollback **neutre** (`rollback_and_trace_neutral`) : sans porteur (tout le contenu d'avant
//! X-6d), il ne contribue pas au checksum et aucune trace ne bouge.

use bevy::prelude::Component;
use bevy_fixed::fixed_math::Fixed;
use serde::{Deserialize, Serialize};
use utils::net_id::GgrsNetId;

/// Agrippeur accroché à `carrier` depuis la frame `since`, au plus jusqu'à `until` (exclue).
#[derive(Component, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Riding {
    /// `GgrsNetId` du joueur porteur.
    pub carrier: GgrsNetId,
    /// Frame de l'accroche.
    pub since: u32,
    /// Frame (exclue) où il se décroche de lui-même (durée maximale accroché).
    pub until: u32,
    /// Déviation du porteur : vitesse latérale (unités/s) et période (frames), copiées de la règle.
    pub sway: Option<(Fixed, u32)>,
    /// Alternances de direction du porteur comptées (secouage).
    pub shakes: u32,
    /// Frame de la dernière alternance comptée.
    pub last_shake: u32,
    /// Dernier sens non nul tenu par le porteur sur chaque axe (-1, 0 : jamais, 1).
    pub last_x: i8,
    pub last_y: i8,
}

impl Riding {
    pub fn new(carrier: GgrsNetId, since: u32, until: u32, sway: Option<(Fixed, u32)>) -> Self {
        Self {
            carrier,
            since,
            until,
            sway,
            shakes: 0,
            last_shake: since,
            last_x: 0,
            last_y: 0,
        }
    }

    /// Compte une alternance de direction quand le porteur tient le sens opposé au dernier sens non nul
    /// d'un axe (`x`, `y` dans -1, 0, 1, comme les touches) ; une pause de plus de `window` frames depuis la
    /// dernière alternance remet le compte à zéro. Renvoie le compte après cette frame.
    pub fn shake(&mut self, x: i8, y: i8, now: u32, window: u32) -> u32 {
        let reversed = |last: i8, current: i8| current != 0 && last != 0 && current != last;
        let turned = reversed(self.last_x, x) || reversed(self.last_y, y);
        if turned {
            if now.saturating_sub(self.last_shake) > window {
                self.shakes = 0;
            }
            self.shakes = self.shakes.saturating_add(1);
            self.last_shake = now;
        }
        if x != 0 {
            self.last_x = x;
        }
        if y != 0 {
            self.last_y = y;
        }
        self.shakes
    }

    /// Vitesse latérale de la déviation à `now` (signe alterné toutes les `period / 2` frames depuis
    /// l'accroche) ; `None` sans déviation.
    pub fn sway_at(&self, now: u32) -> Option<Fixed> {
        let (speed, period) = self.sway?;
        let half = (period / 2).max(1);
        let flipped = (now.saturating_sub(self.since) / half) % 2 == 1;
        Some(if flipped { -speed } else { speed })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn riding() -> Riding {
        Riding::new(GgrsNetId(1, "p".into()), 100, 400, None)
    }

    /// Gauche, droite, gauche : deux alternances ; tenir le même sens ou lâcher n'en compte pas.
    #[test]
    fn counts_reversals_on_each_axis() {
        let mut r = riding();
        assert_eq!(r.shake(-1, 0, 101, 20), 0);
        assert_eq!(r.shake(-1, 0, 102, 20), 0);
        assert_eq!(r.shake(0, 0, 103, 20), 0);
        assert_eq!(r.shake(1, 0, 104, 20), 1);
        assert_eq!(r.shake(-1, 0, 108, 20), 2);
        assert_eq!(r.shake(-1, 1, 110, 20), 2);
        assert_eq!(r.shake(0, -1, 112, 20), 3);
    }

    /// Une pause plus longue que la fenêtre fait repartir le compte.
    #[test]
    fn a_slow_wiggle_starts_over() {
        let mut r = riding();
        r.shake(-1, 0, 101, 20);
        assert_eq!(r.shake(1, 0, 105, 20), 1);
        assert_eq!(r.shake(-1, 0, 120, 20), 2);
        assert_eq!(r.shake(1, 0, 141, 20), 1);
    }

    #[test]
    fn sway_flips_every_half_period() {
        let r = Riding::new(GgrsNetId(1, "p".into()), 100, 400, Some((Fixed::from_num(30), 40)));
        assert_eq!(r.sway_at(100), Some(Fixed::from_num(30)));
        assert_eq!(r.sway_at(119), Some(Fixed::from_num(30)));
        assert_eq!(r.sway_at(120), Some(Fixed::from_num(-30)));
        assert_eq!(r.sway_at(140), Some(Fixed::from_num(30)));
        assert_eq!(riding().sway_at(120), None);
    }
}
