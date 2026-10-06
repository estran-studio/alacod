//! m1-v3-bots-armes : choix d'arme par la config, jamais par le nom.
//!
//! Score d'une arme sur la cible courante = dégâts attendus par seconde : dégâts par projectile
//! × projectiles par tir × cadence × part des projectiles qui touchent, nul hors portée ou sans
//! munitions. La part qui touche vient de la dispersion réelle (D51) : un angle uniforme sur la
//! pleine largeur `spread` couvre `distance × spread` de travers, la cible en occupe `2 × rayon`,
//! d'où `min(1, 2 × rayon / (distance × spread))` (`spread` 0 : tout touche). Pas de cas « boss »
//! ni d'anticipation : un gros ennemi proche choisit de lui-même l'arme à gros dégâts par tir.
//!
//! Hystérésis : un bot ne change d'arme que si la meilleure fait au moins [`SWITCH_GAIN`] fois
//! l'arme en main (ou si celle-ci ne fait rien), et pas avant [`SWITCH_HOLD_FRAMES`] après son
//! dernier choix (`input::WeaponChoices`) : un changement coûte le temps de `switch_weapon`.

use std::collections::BTreeMap;

use bevy::prelude::Resource;
use bevy_fixed::fixed_math::{Fixed, FixedVec2};
use game::weapons::{BulletType, FiringMode, FiringModeConfig};

/// Pas de nouveau choix d'arme avant ce nombre de frames après le précédent.
pub const SWITCH_HOLD_FRAMES: u32 = 120;

/// Gain minimal (× les dégâts attendus de l'arme en main) pour changer d'arme.
pub const SWITCH_GAIN: Fixed = Fixed::from_bits(3 << 15); // 1,5

/// Une arme portée (ou au sol), vue par un bot pour le choix : lue dans le mode de tir courant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeaponView {
    /// A des munitions (chargeur non vide ou rechargeable).
    pub usable: bool,
    /// Dégâts d'un projectile (`bullet_type`).
    pub damage: Fixed,
    /// Projectiles par tir (plombs d'un fusil, balles d'une rafale, 1 sinon).
    pub projectiles: u32,
    /// Tirs par seconde (`firing_rate`).
    pub rate: Fixed,
    /// Portée (`range`).
    pub range: Fixed,
    /// Pleine largeur angulaire de la gerbe, radians : `spread_angle` d'un fusil, `spread` sinon.
    pub spread: Fixed,
    /// Vitesse d'un projectile (`bullet_type`), unités/s : temps de vol de l'anticipation
    /// (m1-v3-bots-lead).
    pub speed: Fixed,
}

impl WeaponView {
    pub fn from_config(config: &FiringModeConfig, usable: bool) -> Self {
        let (damage, speed) = match config.bullet_type {
            BulletType::Standard { damage, speed }
            | BulletType::Explosive { damage, speed, .. }
            | BulletType::Piercing { damage, speed, .. } => (damage, speed),
        };
        let (projectiles, spread) = match config.firing_mode {
            FiringMode::Shotgun {
                pellet_count,
                spread_angle,
            } => (pellet_count, spread_angle),
            FiringMode::Burst {
                pellets_per_shot, ..
            } => (pellets_per_shot, config.spread),
            FiringMode::Automatic {} | FiringMode::Manual {} => (1, config.spread),
        };
        Self {
            usable,
            damage,
            projectiles,
            rate: config.firing_rate,
            range: config.range,
            spread,
            speed,
        }
    }
}

/// Part des projectiles qui touchent une cible de rayon `radius` à `distance` :
/// `min(1, 2 × rayon / (distance × spread))`.
pub fn hit_fraction(distance: Fixed, radius: Fixed, spread: Fixed) -> Fixed {
    let width = distance.saturating_mul(spread);
    let target = radius.saturating_mul(Fixed::from_num(2));
    if width <= target {
        Fixed::ONE
    } else {
        target / width
    }
}

/// Dégâts attendus par seconde de `weapon` sur une cible de rayon `radius` à `distance` ; 0
/// hors portée ou sans munitions.
pub fn expected_dps(weapon: &WeaponView, distance: Fixed, radius: Fixed) -> Fixed {
    if !weapon.usable || distance > weapon.range {
        return Fixed::ZERO;
    }
    weapon
        .damage
        .saturating_mul(Fixed::from_num(weapon.projectiles))
        .saturating_mul(weapon.rate)
        .saturating_mul(hit_fraction(distance, radius, weapon.spread))
}

/// Arme vers laquelle passer (index dans `weapons`), ou `None` pour garder `active` : la
/// meilleure (égalité : le plus petit index) si elle fait au moins [`SWITCH_GAIN`] fois l'arme en
/// main, ou si l'arme en main ne fait rien. L'hystérésis en temps est appliquée par l'appelant.
pub fn better_weapon(
    weapons: &[WeaponView],
    active: usize,
    distance: Fixed,
    radius: Fixed,
) -> Option<usize> {
    let scores: Vec<Fixed> = weapons
        .iter()
        .map(|w| expected_dps(w, distance, radius))
        .collect();
    let (best, best_score) =
        scores
            .iter()
            .copied()
            .enumerate()
            .fold(None, |acc: Option<(usize, Fixed)>, (i, s)| match acc {
                Some((_, b)) if b >= s => acc,
                _ => Some((i, s)),
            })?;
    let current = scores.get(active).copied().unwrap_or(Fixed::ZERO);
    (best != active
        && best_score > Fixed::ZERO
        && (current == Fixed::ZERO || best_score >= current.saturating_mul(SWITCH_GAIN)))
    .then_some(best)
}

/// Mémoire des choix d'arme de chaque bot : handle → (index visé, frame de simulation du choix).
/// Hors rollback comme la navigation : les bots produisent des inputs, pas de l'état de
/// simulation ; déterministe (frame `FrameCount`, jamais d'horloge), vidée à chaque entrée en
/// partie (`OnEnter(AppState::InGame)` : première partie et chaque restart, D14).
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct WeaponChoices(pub BTreeMap<usize, (usize, u32)>);

impl WeaponChoices {
    /// Un pas de choix pour le bot `handle` à la frame `frame` : `true` s'il doit presser
    /// `switch_weapon`. Un choix en cours (index visé ≠ arme en main, toujours utile) se poursuit
    /// (le jeu cycle d'un emplacement par appui, 20 frames entre deux) ; sinon, pas de nouveau
    /// choix avant [`SWITCH_HOLD_FRAMES`] après le précédent, puis [`better_weapon`].
    pub fn step(
        &mut self,
        handle: usize,
        frame: u32,
        weapons: &[WeaponView],
        active: usize,
        distance: Fixed,
        radius: Fixed,
    ) -> bool {
        let choice = self.0.get(&handle).copied();
        if let Some((target, _)) = choice {
            if target != active && weapons.get(target).is_some_and(|w| w.usable) {
                return true;
            }
        }
        let held =
            choice.is_some_and(|(_, since)| frame.saturating_sub(since) < SWITCH_HOLD_FRAMES);
        if held {
            return false;
        }
        match better_weapon(weapons, active, distance, radius) {
            Some(best) => {
                self.0.insert(handle, (best, frame));
                true
            }
            None => false,
        }
    }
}

/// m1-v3-bots-lead : temps de vol plafonné de l'anticipation (au-delà, la cible a le temps de
/// changer de direction ; viser trop loin envoie la balle dans la roche).
pub const LEAD_MAX_SECONDS: Fixed = Fixed::ONE;

/// m1-v3-bots-lead : point visé pour toucher une cible en `target` qui se déplace à `velocity`
/// (unités/s) avec un projectile à `bullet_speed` : `target + velocity × t`, `t = distance /
/// bullet_speed` (une itération), plafonné à [`LEAD_MAX_SECONDS`]. Cible immobile, vitesse de
/// projectile nulle : la cible elle-même. La ligne de tir vers ce point est vérifiée par
/// l'appelant (repli sur la cible).
pub fn lead_point(
    target: FixedVec2,
    velocity: FixedVec2,
    distance: Fixed,
    bullet_speed: Fixed,
) -> FixedVec2 {
    if velocity == FixedVec2::ZERO || bullet_speed <= Fixed::ZERO {
        return target;
    }
    let t = (distance / bullet_speed).min(LEAD_MAX_SECONDS);
    target + velocity * t
}

/// Distance de référence (« portée moyenne ») d'une arme au sol comparée à l'arme en main.
pub const PICKUP_REFERENCE_DISTANCE: Fixed = Fixed::from_bits(250 << 16);

/// Rayon de cible de référence (ennemi sans collider, et comparaison au sol).
pub const TARGET_RADIUS: Fixed = Fixed::from_bits(12 << 16);

/// Une arme au sol vaut d'être ramassée : un emplacement est libre, ou elle fait mieux, à
/// [`PICKUP_REFERENCE_DISTANCE`], que l'arme en main (`held`, celle qui tomberait au sol,
/// emplacements pleins). Une arme déjà portée (même nom) est écartée par l'appelant.
pub fn worth_picking(ground: &WeaponView, held: Option<&WeaponView>, free_slot: bool) -> bool {
    let score = |w: &WeaponView| expected_dps(w, PICKUP_REFERENCE_DISTANCE, TARGET_RADIUS);
    free_slot || score(ground) > held.map_or(Fixed::ZERO, score)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fixed::fixed_math::new as fx;

    fn single(damage: f32, rate: f32, range: f32, spread: f32) -> WeaponView {
        WeaponView {
            usable: true,
            damage: fx(damage),
            projectiles: 1,
            rate: fx(rate),
            range: fx(range),
            spread: fx(spread),
            speed: fx(300.0),
        }
    }

    fn mitraillette() -> WeaponView {
        single(8.0, 10.0, 900.0, 0.15)
    }
    fn lance_lames() -> WeaponView {
        single(12.0, 3.0, 700.0, 0.0)
    }
    fn laser() -> WeaponView {
        single(12.0, 4.0, 900.0, 0.0)
    }
    fn fusil() -> WeaponView {
        WeaponView {
            usable: true,
            damage: fx(12.0),
            projectiles: 8,
            rate: fx(1.0),
            range: fx(400.0),
            spread: fx(0.4),
            speed: fx(250.0),
        }
    }

    fn approx(value: Fixed, expected: f32) -> bool {
        (value.to_num::<f32>() - expected).abs() < 0.5
    }

    /// Les chiffres du rapport (cible de rayon 12, armes de `throne`) : à 300 px, la mitraillette
    /// (dispersion 0,15) ≈ 43/s, lance-lames 36, laser 48 ; à 500 px, mitraillette ≈ 26.
    #[test]
    fn degats_attendus_avec_la_dispersion() {
        let r = fx(12.0);
        assert!(approx(expected_dps(&mitraillette(), fx(300.0), r), 42.7));
        assert!(approx(expected_dps(&lance_lames(), fx(300.0), r), 36.0));
        assert!(approx(expected_dps(&laser(), fx(300.0), r), 48.0));
        assert!(approx(expected_dps(&mitraillette(), fx(500.0), r), 25.6));
        // De près, la gerbe tient dans la cible : tout touche.
        assert!(approx(expected_dps(&mitraillette(), fx(100.0), r), 80.0));
        // Fusil : 8 plombs sur 0,4 rad ; à 100 px, 24 px de cible sur 40 de gerbe.
        assert!(approx(expected_dps(&fusil(), fx(100.0), r), 57.6));
    }

    /// Hors portée ou sans munitions : 0.
    #[test]
    fn hors_portee_ou_vide_ne_compte_pas() {
        assert_eq!(expected_dps(&fusil(), fx(450.0), fx(12.0)), Fixed::ZERO);
        let mut vide = laser();
        vide.usable = false;
        assert_eq!(expected_dps(&vide, fx(100.0), fx(12.0)), Fixed::ZERO);
    }

    /// Hystérésis de gain : à 300 px le laser ne fait que 1,1 × la mitraillette (on la garde) ;
    /// à 500 px, 1,9 × (on change) ; une arme en main vide cède à toute arme utile.
    #[test]
    fn changer_seulement_pour_un_gain_net() {
        let armes = [mitraillette(), lance_lames(), laser()];
        let r = fx(12.0);
        assert_eq!(better_weapon(&armes, 0, fx(300.0), r), None);
        assert_eq!(better_weapon(&armes, 0, fx(500.0), r), Some(2));
        let mut vides = armes;
        vides[0].usable = false;
        assert_eq!(better_weapon(&vides, 0, fx(300.0), r), Some(2));
        // Déjà la meilleure : rien.
        assert_eq!(better_weapon(&armes, 2, fx(500.0), r), None);
        // Aucune arme utile : rien.
        let aucune = [WeaponView {
            usable: false,
            ..laser()
        }];
        assert_eq!(better_weapon(&aucune, 0, fx(300.0), r), None);
    }

    /// Ramasser une arme au sol : emplacement libre, ou meilleure que l'arme en main à 250 px
    /// (laser 48 contre mitraillette ≈ 51 : non ; lance-lames contre une arme vide : oui).
    #[test]
    fn ramasser_une_arme_meilleure_ou_un_emplacement_libre() {
        assert!(!worth_picking(&laser(), Some(&mitraillette()), false));
        assert!(worth_picking(&laser(), Some(&mitraillette()), true));
        let vide = WeaponView {
            usable: false,
            ..mitraillette()
        };
        assert!(worth_picking(&lance_lames(), Some(&vide), false));
        assert!(worth_picking(&laser(), None, false));
    }

    /// Hystérésis en temps et déterminisme : un choix à f100 (vers le laser à 500 px), poursuivi
    /// tant que l'arme en main n'est pas la bonne ; pas de nouveau choix avant f220 ; deux suites
    /// identiques donnent les mêmes choix ; une mémoire vidée (nouvelle partie) repart de zéro.
    #[test]
    fn choix_deterministes_et_tenus() {
        let armes = [mitraillette(), lance_lames(), laser()];
        let (d, r) = (fx(500.0), fx(12.0));
        let joue = |memoire: &mut WeaponChoices| {
            let mut appuis = Vec::new();
            for (frame, active, distance) in [
                (100, 0, d),
                (121, 1, d),
                (142, 2, d),
                (150, 2, fx(100.0)),
                (219, 2, fx(100.0)),
                (220, 2, fx(100.0)),
            ] {
                appuis.push(memoire.step(0, frame, &armes, active, distance, r));
            }
            appuis
        };
        let mut a = WeaponChoices::default();
        let mut b = WeaponChoices::default();
        let suite = joue(&mut a);
        // Choix f100, poursuivi f121 (encore sur le lance-lames), atteint f142 ; de près, la
        // mitraillette (80) bat le laser (48) mais seulement après la tenue (f220).
        assert_eq!(suite, [true, true, false, false, false, true]);
        assert_eq!(joue(&mut b), suite, "même suite, mêmes choix");
        assert_eq!(a, b);
        a.0.clear();
        assert_eq!(joue(&mut a), suite, "mémoire vidée : repart de zéro");
    }

    /// m1-v3-bots-lead : cible immobile → la cible ; en translation à 60 px/s, à 300 px, balle à
    /// 300 px/s → t = 1 s → 60 px devant ; plafond à 1 s (à 600 px, toujours 60 px) ; mêmes
    /// entrées, même point.
    #[test]
    fn anticipation() {
        let cible = FixedVec2::new(fx(300.0), fx(0.0));
        assert_eq!(lead_point(cible, FixedVec2::ZERO, fx(300.0), fx(300.0)), cible);
        let v = FixedVec2::new(fx(0.0), fx(60.0));
        assert_eq!(
            lead_point(cible, v, fx(150.0), fx(300.0)),
            FixedVec2::new(fx(300.0), fx(30.0))
        );
        assert_eq!(
            lead_point(cible, v, fx(300.0), fx(300.0)),
            FixedVec2::new(fx(300.0), fx(60.0))
        );
        assert_eq!(
            lead_point(cible, v, fx(600.0), fx(300.0)),
            FixedVec2::new(fx(300.0), fx(60.0)),
            "plafond d'une seconde"
        );
        assert_eq!(lead_point(cible, v, fx(300.0), Fixed::ZERO), cible);
        assert_eq!(
            lead_point(cible, v, fx(300.0), fx(300.0)),
            lead_point(cible, v, fx(300.0), fx(300.0))
        );
    }
}
