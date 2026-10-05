//! `decide(profile, view, rng) -> BoxInput` : la logique des profils v0 (`immobile`, `fonceur`,
//! `prudent`). Fonction pure, tout en `Fixed` (jamais `f32`, CLAUDE.md règle 1) : testable sans
//! Bevy, sans I/O. Le tie-breaking déterministe (ennemi/fenêtre le plus proche) est fait en
//! amont par [`crate::view::nearest_by_net_id`], pas ici.
//!
//! `rng: &mut RollbackRng` fait partie de la signature pour les profils futurs (« aléatoire
//! seedé », plan §9.7) ; aucun des trois profils v0 n'en a besoin (leurs décisions ne
//! choisissent jamais entre plusieurs candidats à égalité — ce tie-breaking passe par
//! `GgrsNetId`, pas par le RNG, CLAUDE.md règle 4).

use bevy_fixed::fixed_math::{Fixed, FixedVec2, FIXED_ZERO};
use bevy_fixed::rng::RollbackRng;
use game::character::player::input::{
    BoxInput, INPUT_DOWN, INPUT_INTERACTION, INPUT_LEFT, INPUT_RELOAD, INPUT_RIGHT, INPUT_UP,
};
use game::replay::BotProfile;

use crate::view::{BotView, WindowView};

/// `fonceur` : distance en dessous (ou égale) de laquelle un ennemi est une menace (charge +
/// tire) ; au-dessus (ou en l'absence d'ennemi), la réparation d'une fenêtre est sans danger.
/// Une seule constante pour les deux : au-delà, une balle n'atteindrait de toute façon pas grand
/// chose d'utile en v0 (pas de portée d'arme dans `BotView`, voir la doc du module `view`).
const FONCEUR_THREAT_RANGE: Fixed = Fixed::from_bits(240 << 16);

/// `fonceur`/`prudent` : distance à laquelle s'arrêter d'approcher une fenêtre réparable et
/// maintenir l'interaction. Nettement sous `Interactable::interaction_range` (50.0, valeur par
/// défaut de `crates/game/src/interaction.rs`, non exposée dans `BotView`) : marge de sécurité
/// contre l'arrondi fixed-point et le mouvement d'une frame à l'autre.
const REPAIR_REACH: Fixed = Fixed::from_bits(40 << 16);

/// `prudent` : sous cette distance, recule.
const PRUDENT_MIN_DISTANCE: Fixed = Fixed::from_bits(180 << 16);
/// `prudent` : au-dessus de cette distance, avance. Entre les deux, tient sa position. Tire
/// dès que l'ennemi est à cette distance ou moins (donc aussi en avançant ou en reculant).
pub(crate) const PRUDENT_MAX_DISTANCE: Fixed = Fixed::from_bits(320 << 16);

/// `prudent` (T1.14) : approche du portail. Le jeu ne freine que si aucun bouton de
/// déplacement n'est tenu, et les boutons ne donnent que le signe de chaque axe : en visant le
/// portail à pleine vitesse, le bot le dépassait et tournait autour sans entrer dans son rayon
/// (24). Sous [`PORTAL_BRAKE_DISTANCE`], il relâche tout tant que sa vitesse dépasse
/// [`PORTAL_BRAKE_SPEED`], puis avance par petits pas ; un axe dont l'écart est sous
/// [`PORTAL_DEAD_ZONE`] n'est pas pressé.
///
/// m1-v3-bots-portail : le freinage ne commençait qu'à 48, trop tard à pleine vitesse (≈ 140) :
/// la glissade l'emmenait au-delà de 48 et la route le relançait à fond — le bot tournait autour
/// du portail à 50–100 sans jamais y entrer (throne, solo, graines 3 et 16). Sous
/// [`PORTAL_SLOW_DISTANCE`], il relâche donc tout dès que sa vitesse dépasse
/// [`PORTAL_BRAKE_SPEED`], puis reprend la route par petits pas.
pub(crate) const PORTAL_BRAKE_DISTANCE: Fixed = Fixed::from_bits(48 << 16);
const PORTAL_SLOW_DISTANCE: Fixed = Fixed::from_bits(112 << 16);
const PORTAL_BRAKE_SPEED: Fixed = Fixed::from_bits(30 << 16);
const PORTAL_DEAD_ZONE: Fixed = Fixed::from_bits(6 << 16);

/// Décide l'input d'un joueur local piloté par un bot, pour une frame.
pub fn decide(profile: BotProfile, view: &BotView, rng: &mut RollbackRng) -> BoxInput {
    let _ = rng; // réservé aux profils futurs (voir la doc du module)
    match profile {
        BotProfile::Immobile => BoxInput::default(),
        BotProfile::Fonceur => decide_fonceur(view),
        BotProfile::Prudent => decide_prudent(view),
        BotProfile::Chasseur | BotProfile::Acheteur => view
            .hunter
            .as_ref()
            .map(crate::hunter::decide_hunter)
            .unwrap_or_default(),
    }
}

fn decide_fonceur(view: &BotView) -> BoxInput {
    let mut input = BoxInput::default();

    let threatened = view
        .nearest_enemy
        .is_some_and(|enemy| enemy.distance <= FONCEUR_THREAT_RANGE);

    if !threatened {
        if let Some(window) = view.nearest_window.filter(|w| w.repairable) {
            approach_and_repair(&mut input, view.position, &window);
            maybe_reload(&mut input, view);
            return input;
        }
    }

    if let Some(enemy) = view.nearest_enemy {
        aim_at(&mut input, view.position, enemy.position);
        // Ligne droite vers un ennemi visible ; caché derrière un mur : le chemin
        let toward = enemy.position - view.position;
        set_direction_buttons(&mut input, route_unless(view.enemy_visible, view, toward));
        if enemy.distance <= FONCEUR_THREAT_RANGE {
            input.fire = true;
        }
    } else if let Some(portal) = view.portal {
        // T1.8 : plus d'ennemi, portail ouvert : y aller (niveau suivant), par le chemin.
        set_direction_buttons(&mut input, view.route.unwrap_or(portal - view.position));
    }

    maybe_reload(&mut input, view);
    input
}

fn decide_prudent(view: &BotView) -> BoxInput {
    let mut input = BoxInput::default();

    // T1.14 (v1) : un projectile menace → l'esquive remplace le déplacement ; la visée et le
    // tir vers l'ennemi le plus proche restent ceux de v0.
    if let Some(away) = crate::dodge::dodge(view) {
        set_direction_buttons(&mut input, away);
        if let Some(enemy) = view.nearest_enemy {
            aim_at(&mut input, view.position, enemy.position);
            if enemy.distance <= PRUDENT_MAX_DISTANCE {
                input.fire = view.trigger_ready;
            }
        }
        manage_weapon(&mut input, view);
        return input;
    }

    if let Some(enemy) = view.nearest_enemy {
        aim_at(&mut input, view.position, enemy.position);

        if enemy.distance < PRUDENT_MIN_DISTANCE {
            set_direction_buttons(&mut input, view.position - enemy.position); // s'éloigne
        } else if !view.enemy_visible || enemy.distance > PRUDENT_MAX_DISTANCE {
            // s'approche : par le chemin (ennemi caché, ou loin), ligne droite en repli
            let toward = enemy.position - view.position;
            set_direction_buttons(&mut input, route_unless(false, view, toward));
        } // sinon : garde sa position (visible, dans la bande [MIN, MAX])

        if enemy.distance <= PRUDENT_MAX_DISTANCE {
            // T1.14 : relâcher entre deux tirs d'une arme non automatique
            input.fire = view.trigger_ready;
        }
    } else if let Some(portal) = view.portal {
        // T1.8 : plus d'ennemi, portail ouvert : y aller (niveau suivant), en freinant (T1.14)
        approach_portal(&mut input, view, portal);
    }

    manage_weapon(&mut input, view);
    input
}

/// Direction de déplacement : `straight` si `direct`, sinon le pas suivant du champ de
/// navigation ([`BotView::route`]), avec la ligne droite en repli quand il n'y a pas de chemin.
fn route_unless(direct: bool, view: &BotView, straight: FixedVec2) -> FixedVec2 {
    if direct {
        straight
    } else {
        view.route.unwrap_or(straight)
    }
}

/// Approche freinée du portail (`prudent`, T1.14, voir [`PORTAL_BRAKE_DISTANCE`]) : par le
/// chemin tant qu'il est loin, freinée dans les derniers [`PORTAL_BRAKE_DISTANCE`].
fn approach_portal(input: &mut BoxInput, view: &BotView, portal: FixedVec2) {
    let delta = portal - view.position;
    if delta.length() < PORTAL_SLOW_DISTANCE && view.velocity.length() > PORTAL_BRAKE_SPEED {
        return; // aucun bouton : friction
    }
    if delta.length() >= PORTAL_BRAKE_DISTANCE {
        if let Some(route) = view.route {
            set_direction_buttons(input, route);
            return;
        }
    }
    let mut step = FixedVec2::ZERO;
    if delta.x.abs() > PORTAL_DEAD_ZONE {
        step.x = delta.x;
    }
    if delta.y.abs() > PORTAL_DEAD_ZONE {
        step.y = delta.y;
    }
    set_direction_buttons(input, step);
}

/// `prudent` v1 (T1.14) : recharge si possible, sinon passe à une arme utilisable (même règle
/// que `chasseur`/`acheteur`) — sans ça, une mitrailleuse vide sans réserve bloquait le bot
/// devant le dernier ennemi d'un niveau.
fn manage_weapon(input: &mut BoxInput, view: &BotView) {
    if view.reload {
        input.buttons |= INPUT_RELOAD;
    } else if view.switch_weapon {
        input.switch_weapon = true;
    } else {
        maybe_reload(input, view);
    }
}

/// Marche vers `window` tant qu'elle est hors de portée de réparation, puis maintient
/// l'interaction une fois à portée (ne bouge plus : `interaction_detection_system` exige d'être
/// entré dans `Interactable::interaction_range`, pas seulement de s'en approcher).
fn approach_and_repair(input: &mut BoxInput, from: FixedVec2, window: &WindowView) {
    if window.distance > REPAIR_REACH {
        set_direction_buttons(input, window.position - from);
    } else {
        input.buttons |= INPUT_INTERACTION;
    }
}

fn maybe_reload(input: &mut BoxInput, view: &BotView) {
    if view.ammo == 0 {
        input.buttons |= INPUT_RELOAD;
    }
}

/// Visée : vecteur du joueur vers `target`, en unités monde (même convention que
/// `game::replay::Segment::pan` et `read_local_inputs`, qui envoie `pointer_world_pos -
/// player_pos`). Conversion Fixed → i16 directe (`to_num`, jamais via `f32`, CLAUDE.md règle 7).
pub(crate) fn aim_at(input: &mut BoxInput, from: FixedVec2, target: FixedVec2) {
    let delta = target - from;
    input.pan_x = clamp_to_i16(delta.x);
    input.pan_y = clamp_to_i16(delta.y);
}

/// `BoxInput::buttons` n'a que 4 bits de direction (pas un vecteur analogique) : seul le signe
/// de chaque composante de `direction` compte, la magnitude est ignorée (`apply_inputs`
/// normalise ensuite). Direction nulle → aucun bouton.
pub(crate) fn set_direction_buttons(input: &mut BoxInput, direction: FixedVec2) {
    if direction.x > FIXED_ZERO {
        input.buttons |= INPUT_RIGHT;
    } else if direction.x < FIXED_ZERO {
        input.buttons |= INPUT_LEFT;
    }
    if direction.y > FIXED_ZERO {
        input.buttons |= INPUT_UP;
    } else if direction.y < FIXED_ZERO {
        input.buttons |= INPUT_DOWN;
    }
}

fn clamp_to_i16(value: Fixed) -> i16 {
    let truncated = value.to_num::<i32>();
    truncated.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::EnemyView;
    use bevy_fixed::fixed_math::new as fx;
    use game::character::player::input::{INPUT_DOWN, INPUT_LEFT, INPUT_RIGHT, INPUT_UP};

    fn view(position: FixedVec2) -> BotView {
        BotView {
            position,
            health: fx(100.0),
            health_max: fx(100.0),
            ammo: 10,
            wave: 1,
            nearest_enemy: None,
            nearest_window: None,
            hunter: None,
            portal: None,
            projectiles: vec![],
            body_radius: bevy_fixed::fixed_math::Fixed::from_num(10),
            reload: false,
            switch_weapon: false,
            trigger_ready: true,
            velocity: FixedVec2::ZERO,
            enemy_visible: true,
            route: None,
        }
    }

    #[test]
    fn prudent_freine_pres_du_portail() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        // Portail à 30 px, lancé à pleine vitesse : aucun bouton (friction)
        v.portal = Some(FixedVec2::new(fx(30.0), fx(3.0)));
        v.velocity = FixedVec2::new(fx(150.0), fx(0.0));
        assert_eq!(decide(BotProfile::Prudent, &v, &mut rng()).buttons, 0);
        // Arrêté : petit pas, l'axe sous la zone morte (3 < 6) n'est pas pressé
        v.velocity = FixedVec2::ZERO;
        let input = decide(BotProfile::Prudent, &v, &mut rng());
        assert_ne!(input.buttons & INPUT_RIGHT, 0);
        assert_eq!(input.buttons & (INPUT_UP | INPUT_DOWN), 0);
        // Loin du portail : pas de freinage, même lancé
        v.portal = Some(FixedVec2::new(fx(200.0), fx(0.0)));
        v.velocity = FixedVec2::new(fx(150.0), fx(0.0));
        assert_ne!(
            decide(BotProfile::Prudent, &v, &mut rng()).buttons & INPUT_RIGHT,
            0
        );
    }

    /// m1-v3-bots-portail : à 80 px du portail (au-delà de l'ancienne zone de 48), lancé à
    /// pleine vitesse : freine (aucun bouton) au lieu de suivre la route et d'orbiter ; à
    /// l'arrêt : reprend la route.
    #[test]
    fn prudent_freine_avant_la_zone_du_portail() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.portal = Some(FixedVec2::new(fx(80.0), fx(0.0)));
        v.route = Some(FixedVec2::new(fx(0.0), fx(8.0)));
        v.velocity = FixedVec2::new(fx(100.0), fx(100.0));
        assert_eq!(decide(BotProfile::Prudent, &v, &mut rng()).buttons, 0);
        v.velocity = FixedVec2::new(fx(10.0), fx(0.0));
        assert_ne!(
            decide(BotProfile::Prudent, &v, &mut rng()).buttons & INPUT_UP,
            0,
            "lent : la route"
        );
    }

    /// Navigation (suite T1.14) : ennemi caché derrière un mur → le pas du champ, pas la
    /// ligne droite ; sans chemin, ligne droite en repli ; visible et dans la bande : immobile.
    #[test]
    fn ennemi_cache_suivre_le_chemin() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        // Ennemi à droite (dans la bande de prudent), caché ; le chemin part vers le haut
        v.nearest_enemy = Some(EnemyView {
            position: FixedVec2::new(fx(250.0), fx(0.0)),
            distance: fx(250.0),
        });
        v.enemy_visible = false;
        v.route = Some(FixedVec2::new(fx(0.0), fx(8.0)));
        for profile in [BotProfile::Fonceur, BotProfile::Prudent] {
            let input = decide(profile, &v, &mut rng());
            assert_ne!(input.buttons & INPUT_UP, 0, "{profile:?} : suit le chemin");
            assert_eq!(
                input.buttons & INPUT_RIGHT,
                0,
                "{profile:?} : pas tout droit"
            );
        }
        // Sans chemin : ligne droite en repli
        v.route = None;
        for profile in [BotProfile::Fonceur, BotProfile::Prudent] {
            let input = decide(profile, &v, &mut rng());
            assert_ne!(input.buttons & INPUT_RIGHT, 0, "{profile:?} : repli");
        }
        // Visible dans la bande : prudent tient sa position même avec un chemin
        v.enemy_visible = true;
        v.route = Some(FixedVec2::new(fx(0.0), fx(8.0)));
        let input = decide(BotProfile::Prudent, &v, &mut rng());
        assert_eq!(
            input.buttons & (INPUT_UP | INPUT_DOWN | INPUT_LEFT | INPUT_RIGHT),
            0
        );
    }

    /// Portail loin : le chemin ; près (sous `PORTAL_BRAKE_DISTANCE`) : l'approche freinée.
    #[test]
    fn portail_par_le_chemin() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.portal = Some(FixedVec2::new(fx(200.0), fx(0.0)));
        v.route = Some(FixedVec2::new(fx(0.0), fx(-8.0)));
        for profile in [BotProfile::Fonceur, BotProfile::Prudent] {
            let input = decide(profile, &v, &mut rng());
            assert_ne!(input.buttons & INPUT_DOWN, 0, "{profile:?}");
            assert_eq!(input.buttons & INPUT_RIGHT, 0, "{profile:?}");
        }
        v.portal = Some(FixedVec2::new(fx(30.0), fx(0.0)));
        let input = decide(BotProfile::Prudent, &v, &mut rng());
        assert_ne!(
            input.buttons & INPUT_RIGHT,
            0,
            "freinage : petit pas direct"
        );
    }

    #[test]
    fn sans_ennemi_fonceur_et_prudent_vont_au_portail() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.portal = Some(FixedVec2::new(fx(-50.0), fx(30.0)));
        for profile in [BotProfile::Fonceur, BotProfile::Prudent] {
            let input = decide(profile, &v, &mut rng());
            assert_ne!(
                input.buttons & INPUT_LEFT,
                0,
                "{profile:?} : portail à gauche"
            );
            assert_ne!(input.buttons & INPUT_UP, 0, "{profile:?} : portail en haut");
            assert!(!input.fire);
        }
        // Un ennemi présent passe avant le portail.
        v.nearest_enemy = Some(EnemyView {
            position: FixedVec2::new(fx(50.0), fx(0.0)),
            distance: fx(50.0),
        });
        let input = decide(BotProfile::Fonceur, &v, &mut rng());
        assert_ne!(input.buttons & INPUT_RIGHT, 0);
        assert_eq!(input.buttons & INPUT_LEFT, 0);
        // Immobile ne bouge jamais.
        v.nearest_enemy = None;
        assert_eq!(
            decide(BotProfile::Immobile, &v, &mut rng()),
            BoxInput::default()
        );
    }

    fn rng() -> RollbackRng {
        RollbackRng::new(1)
    }

    #[test]
    fn immobile_never_moves_or_fires() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.nearest_enemy = Some(EnemyView {
            position: FixedVec2::new(fx(10.0), fx(0.0)),
            distance: fx(10.0),
        });
        let input = decide(BotProfile::Immobile, &v, &mut rng());
        assert_eq!(input, BoxInput::default());
    }

    #[test]
    fn fonceur_charges_and_fires_in_range() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.nearest_enemy = Some(EnemyView {
            position: FixedVec2::new(fx(50.0), fx(0.0)),
            distance: fx(50.0),
        });
        let input = decide(BotProfile::Fonceur, &v, &mut rng());
        assert!(input.fire, "à portée : doit tirer");
        assert_ne!(
            input.buttons & INPUT_RIGHT,
            0,
            "ennemi à droite : doit avancer vers la droite"
        );
        assert_eq!(input.buttons & INPUT_LEFT, 0);
        assert!(input.pan_x > 0, "vise vers la droite (pan_x > 0)");
    }

    #[test]
    fn fonceur_advances_without_firing_when_far() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.nearest_enemy = Some(EnemyView {
            position: FixedVec2::new(fx(1000.0), fx(0.0)),
            distance: fx(1000.0),
        });
        let input = decide(BotProfile::Fonceur, &v, &mut rng());
        assert!(!input.fire, "hors de portée : ne tire pas");
        assert_ne!(input.buttons & INPUT_RIGHT, 0, "continue d'avancer");
    }

    #[test]
    fn fonceur_reloads_when_empty() {
        let mut v = view(FixedVec2::ZERO);
        v.ammo = 0;
        let input = decide(BotProfile::Fonceur, &v, &mut rng());
        assert_ne!(input.buttons & INPUT_RELOAD, 0);
    }

    #[test]
    fn fonceur_repairs_when_no_enemy_nearby() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.nearest_window = Some(WindowView {
            position: FixedVec2::new(fx(200.0), fx(0.0)),
            health: 1,
            distance: fx(200.0),
            repairable: true,
        });
        let input = decide(BotProfile::Fonceur, &v, &mut rng());
        assert!(!input.fire);
        assert_ne!(
            input.buttons & INPUT_RIGHT,
            0,
            "marche vers la fenêtre, hors de portée de réparation"
        );
        assert_eq!(input.buttons & INPUT_INTERACTION, 0);
    }

    #[test]
    fn fonceur_holds_interaction_once_in_repair_reach() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.nearest_window = Some(WindowView {
            position: FixedVec2::new(fx(10.0), fx(0.0)),
            health: 1,
            distance: fx(10.0),
            repairable: true,
        });
        let input = decide(BotProfile::Fonceur, &v, &mut rng());
        assert_ne!(input.buttons & INPUT_INTERACTION, 0);
        assert_eq!(
            input.buttons & (INPUT_RIGHT | INPUT_LEFT | INPUT_UP | INPUT_DOWN),
            0,
            "à portée : ne bouge plus"
        );
    }

    #[test]
    fn fonceur_ignores_repairable_window_when_enemy_is_a_threat() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.nearest_enemy = Some(EnemyView {
            position: FixedVec2::new(fx(10.0), fx(0.0)),
            distance: fx(10.0),
        });
        v.nearest_window = Some(WindowView {
            position: FixedVec2::new(fx(-10.0), fx(0.0)),
            health: 1,
            distance: fx(10.0),
            repairable: true,
        });
        let input = decide(BotProfile::Fonceur, &v, &mut rng());
        assert!(input.fire, "l'ennemi proche prime sur la réparation");
        assert_eq!(input.buttons & INPUT_INTERACTION, 0);
    }

    #[test]
    fn fonceur_does_not_repair_a_full_health_window() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.nearest_window = Some(WindowView {
            position: FixedVec2::new(fx(10.0), fx(0.0)),
            health: 3,
            distance: fx(10.0),
            repairable: false,
        });
        let input = decide(BotProfile::Fonceur, &v, &mut rng());
        assert_eq!(input, BoxInput::default(), "rien à réparer, rien à tirer");
    }

    #[test]
    fn prudent_retreats_when_too_close() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.nearest_enemy = Some(EnemyView {
            position: FixedVec2::new(fx(50.0), fx(0.0)),
            distance: fx(50.0),
        });
        let input = decide(BotProfile::Prudent, &v, &mut rng());
        assert!(input.fire, "reste à portée en reculant : tire");
        assert_ne!(
            input.buttons & INPUT_LEFT,
            0,
            "ennemi à droite, trop près : recule vers la gauche"
        );
        assert_eq!(input.buttons & INPUT_RIGHT, 0);
        assert!(input.pan_x > 0, "vise quand même vers l'ennemi");
    }

    #[test]
    fn prudent_advances_when_too_far() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.nearest_enemy = Some(EnemyView {
            position: FixedVec2::new(fx(1000.0), fx(0.0)),
            distance: fx(1000.0),
        });
        let input = decide(BotProfile::Prudent, &v, &mut rng());
        assert!(!input.fire, "hors de portée : ne tire pas");
        assert_ne!(input.buttons & INPUT_RIGHT, 0);
    }

    #[test]
    fn prudent_holds_position_in_band() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.nearest_enemy = Some(EnemyView {
            position: FixedVec2::new(fx(250.0), fx(0.0)),
            distance: fx(250.0),
        });
        let input = decide(BotProfile::Prudent, &v, &mut rng());
        assert!(input.fire, "dans la bande : tire");
        assert_eq!(
            input.buttons & (INPUT_RIGHT | INPUT_LEFT | INPUT_UP | INPUT_DOWN),
            0,
            "dans [MIN, MAX] : ne bouge pas"
        );
    }

    #[test]
    fn prudent_never_repairs() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.nearest_window = Some(WindowView {
            position: FixedVec2::new(fx(10.0), fx(0.0)),
            health: 1,
            distance: fx(10.0),
            repairable: true,
        });
        let input = decide(BotProfile::Prudent, &v, &mut rng());
        assert_eq!(
            input,
            BoxInput::default(),
            "pas d'ennemi, prudent n'agit pas"
        );
    }

    #[test]
    fn decide_is_pure_same_view_same_input() {
        let mut v = view(FixedVec2::new(fx(3.0), fx(4.0)));
        v.nearest_enemy = Some(EnemyView {
            position: FixedVec2::new(fx(103.0), fx(4.0)),
            distance: fx(100.0),
        });
        let a = decide(BotProfile::Fonceur, &v, &mut rng());
        let b = decide(BotProfile::Fonceur, &v, &mut rng());
        assert_eq!(a, b);
    }
}
