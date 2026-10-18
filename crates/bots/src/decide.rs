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
use sim_core::aim::AimMode;

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
/// Distance d'approche d'un ennemi immobile (voir [`BotView::enemy_still`]) : `prudent` ne
/// recule pas devant lui (il ne peut pas venir au contact).
pub(crate) const STILL_TARGET_DISTANCE: Fixed = Fixed::from_bits(120 << 16);
/// Contournement d'un combat bloqué (`BotView::engage_strafe`) : seulement tout près de l'ennemi, là où la tangente
/// change vraiment l'angle de tir ; plus loin, l'approche par le chemin continue.
const STRAFE_MAX_DISTANCE: Fixed = Fixed::from_bits(60 << 16);

/// `prudent` (T1.14) : approche du portail. Les boutons ne donnent que le signe de chaque
/// axe et le corps garde son élan : en visant le portail (ou le pas suivant de la route) à
/// pleine vitesse, le bot dépassait chaque point visé et tournait autour du portail sans entrer
/// dans son rayon (24) — en T1.14 sous 48 px, puis (m1-v3-bots-portail, throne : solo graines
/// 3 et 16, 2 bots graine 7) à 50–190 px en suivant la route.
///
/// m1-v3-bots-portail : pilotage en vitesse (vitesse voulue, écart corrigé axe par axe), pour
/// l'ancienne course glissante. m1-bots-apres-movement-feel : la course atteint sa vitesse en
/// 3 frames et s'arrête en 3 (`game::character::movement::run_velocity`, 50 px/s par frame) ;
/// le pilotage en vitesse (croisière 120, zone morte 12) alternait appui et bouton opposé à
/// chaque frame, et le bot zigzaguait (throne_floor_1 : 173 frames de l'ouverture du portail à
/// l'étage). Plus d'élan à compenser : [`steer`] presse la direction voulue, en 8 directions.
/// La route sert au-delà de cette distance du portail, la ligne droite en deçà (sauf mur).
pub(crate) const PORTAL_BRAKE_DISTANCE: Fixed = Fixed::from_bits(48 << 16);
/// Part minimale d'un axe dans la direction voulue (unitaire) pour presser son bouton :
/// sin 22,5°, direction quantifiée en 8 secteurs égaux.
const STEER_AXIS_SHARE: Fixed = Fixed::from_bits(25080); // 0,38268 × 65536

/// Décide l'input d'un joueur local piloté par un bot, pour une frame. Suivi Q17 : un personnage
/// à visée `FourWay` ([`BotView::aim`]) vise la direction que le moteur retiendra, voir
/// [`apply_four_way`] ; `Free` (tout le contenu d'avant) : l'input du profil, inchangé.
pub fn decide(profile: BotProfile, view: &BotView, rng: &mut RollbackRng) -> BoxInput {
    let mut input = decide_profile(profile, view, rng);
    if view.aim == AimMode::FourWay {
        // Combat engagé (blocage constaté, `BotView::engage_reach`) : seule la direction de tir est écrite, le bot se
        // déplace pour changer sa ligne de tir et la file de tir ne le fige pas (graines 95, 119, 134 d'`isaac`). Réanimation
        // tenue (`hold_revive`) : idem, le bot ne quitte pas la portée de son coéquipier pour s'aligner.
        let holding = view.hold_revive && view.revive.is_some_and(|revive| revive.interact);
        apply_four_way(
            &mut input,
            view.enemy_visible && view.enemy_shootable && view.engage_reach.is_none() && !holding,
        );
    }
    input
}

/// Demi-largeur (unités monde) de la file de tir d'un personnage à visée `FourWay` : une cible dont
/// l'écart perpendiculaire à la direction retenue dépasse cette valeur n'est pas touchée (rayon d'une
/// larme 5 + demi-corps d'un ennemi ~10 : marge de 1).
pub(crate) const FOUR_WAY_LANE: i16 = 14;

/// Distance (sur l'axe retenu) en deçà de laquelle un bot à visée `FourWay` s'aligne sur sa cible :
/// la portée de tir de `prudent` ([`PRUDENT_MAX_DISTANCE`]) ; au-delà, il approche comme avant.
const FOUR_WAY_ENGAGE: i16 = 320;

/// Visée `FourWay` (suivi Q17) : `pan` vaut l'écart du joueur à la cible visée (voir [`aim_at`]) ; le
/// moteur n'en garde que la direction la plus proche parmi haut, bas, gauche, droite
/// (`sim_core::aim::four_way`, l'horizontale l'emporte sur une diagonale parfaite). Le bot écrit cette
/// direction dans son input (pleine inclinaison, 127), puis, cible à moins de [`FOUR_WAY_ENGAGE`] sur
/// l'axe retenu :
/// - hors de la file (écart perpendiculaire > [`FOUR_WAY_LANE`]) : gâchette relâchée et décalage
///   **imposé** vers la cible sur l'axe perpendiculaire (le déplacement du profil y est remplacé), le
///   mouvement sur l'axe de tir restant celui du profil (approche ou recul) ;
/// - dans la file : le profil tire comme avant, sans dérive perpendiculaire (boutons de l'axe
///   perpendiculaire relâchés) pour rester aligné.
/// Sans visée (`pan` nul), cible lointaine ou sans ligne dégagée jusqu'à elle (`clear_line` faux : un
/// mur, le bot suit sa route et ne doit pas être détourné) : seule la direction est écrite.
pub(crate) fn apply_four_way(input: &mut BoxInput, clear_line: bool) {
    let (x, y) = sim_core::aim::four_way(input.pan_x, input.pan_y);
    if (x, y) == (0, 0) {
        return;
    }
    let (off_x, off_y) = (input.pan_x, input.pan_y);
    input.pan_x = i16::from(x) * 127;
    input.pan_y = i16::from(y) * 127;
    // Distance sur l'axe de tir, écart perpendiculaire et son signe.
    let (along, perpendicular) = if x != 0 {
        (off_x, off_y)
    } else {
        (off_y, off_x)
    };
    if !clear_line || i32::from(along).abs() > i32::from(FOUR_WAY_ENGAGE) {
        return;
    }
    let (toward, away) = match (x != 0, perpendicular > 0) {
        (true, true) => (INPUT_UP, INPUT_DOWN),
        (true, false) => (INPUT_DOWN, INPUT_UP),
        (false, true) => (INPUT_RIGHT, INPUT_LEFT),
        (false, false) => (INPUT_LEFT, INPUT_RIGHT),
    };
    if i32::from(perpendicular).abs() > i32::from(FOUR_WAY_LANE) {
        input.fire = false;
        input.buttons = (input.buttons & !away) | toward;
    } else {
        input.buttons &= !(toward | away);
    }
}

fn decide_profile(profile: BotProfile, view: &BotView, rng: &mut RollbackRng) -> BoxInput {
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
        BotProfile::Explorateur => crate::explorer::decide_explorer(view),
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

    // m1-v3-bots-reanimation : relever un coéquipier à terre d'abord, en se défendant.
    if let Some(revive) = view.revive {
        set_direction_buttons(&mut input, revive.direction);
        if revive.interact {
            input.buttons |= INPUT_INTERACTION;
        }
        if let Some(enemy) = view.nearest_enemy {
            aim_at(&mut input, view.position, enemy.position);
            input.fire = enemy.distance <= FONCEUR_THREAT_RANGE && in_range(view, enemy.distance);
        }
        maybe_reload(&mut input, view);
        return input;
    }

    // m1-v3-bots-softlocks : à sec, ramasser le butin, en se défendant.
    if let Some(loot) = view.loot {
        set_direction_buttons(&mut input, loot);
        if view.loot_interact {
            input.buttons |= INPUT_INTERACTION;
        }
        if let Some(enemy) = view.nearest_enemy {
            aim_at(&mut input, view.position, enemy.position);
            input.fire = enemy.distance <= FONCEUR_THREAT_RANGE && in_range(view, enemy.distance);
        }
        maybe_reload(&mut input, view);
        return input;
    }

    if let Some(enemy) = view.nearest_enemy {
        aim_at(&mut input, view.position, enemy.position);
        // Ligne droite vers un ennemi visible ; caché derrière un mur : le chemin
        let toward = enemy.position - view.position;
        set_direction_buttons(&mut input, route_unless(view.enemy_visible, view, toward));
        if enemy.distance <= FONCEUR_THREAT_RANGE {
            input.fire = in_range(view, enemy.distance);
        }
    } else if let Some(portal) = view.portal {
        // T1.8 : plus d'ennemi, portail ouvert : y aller (niveau suivant), par le chemin.
        set_direction_buttons(&mut input, view.route.unwrap_or(portal - view.position));
    }

    // m1-v3-bots-armes : en `Floors` (`fire_range` connu), `fonceur` change d'arme comme
    // `prudent` (choix par la config, `input::WeaponChoices`) ; ailleurs, inchangé.
    if view.fire_range.is_some() {
        manage_weapon(&mut input, view);
    } else {
        maybe_reload(&mut input, view);
    }
    input
}

pub(crate) fn decide_prudent(view: &BotView) -> BoxInput {
    let mut input = BoxInput::default();

    // T1.14 (v1) : un projectile menace → l'esquive remplace le déplacement ; la visée et le
    // tir vers l'ennemi le plus proche restent ceux de v0.
    if let Some(away) = crate::dodge::dodge(view) {
        set_direction_buttons(&mut input, away);
        if let Some(enemy) = view.nearest_enemy {
            aim_at(&mut input, view.position, enemy.position);
            if enemy.distance <= PRUDENT_MAX_DISTANCE {
                input.fire =
                    view.trigger_ready && line_of_fire(view) && in_range(view, enemy.distance);
            }
        }
        manage_weapon(&mut input, view);
        return input;
    }

    // m1-v3-bots-reanimation : relever un coéquipier à terre d'abord, en se défendant.
    if let Some(revive) = view.revive {
        set_direction_buttons(&mut input, revive.direction);
        if revive.interact {
            input.buttons |= INPUT_INTERACTION;
        }
        if let Some(enemy) = view.nearest_enemy {
            aim_at(&mut input, view.position, enemy.position);
            if enemy.distance <= PRUDENT_MAX_DISTANCE {
                input.fire =
                    view.trigger_ready && line_of_fire(view) && in_range(view, enemy.distance);
            }
        }
        manage_weapon(&mut input, view);
        return input;
    }

    // m1-v3-bots-softlocks : à sec (plus de réserve), ramasser le butin, en se défendant.
    if let Some(loot) = view.loot {
        set_direction_buttons(&mut input, loot);
        if view.loot_interact {
            input.buttons |= INPUT_INTERACTION;
        }
        if let Some(enemy) = view.nearest_enemy {
            aim_at(&mut input, view.position, enemy.position);
            if enemy.distance <= PRUDENT_MAX_DISTANCE {
                input.fire =
                    view.trigger_ready && line_of_fire(view) && in_range(view, enemy.distance);
            }
        }
        manage_weapon(&mut input, view);
        return input;
    }

    if let Some(enemy) = view.nearest_enemy {
        aim_at(&mut input, view.position, enemy.position);

        // Larmes en orbite (`satellite`) : elles ne touchent qu'à `orbit_reach` du porteur, un boss immobile à
        // 94 unités ne perdait jamais un point de vie (graine 41 d'`isaac` sans bombe, T11b) : le bot s'approche,
        // immobile ou non, et ne recule plus devant un ennemi qui l'approche. Même chose quand le tir stagne
        // (`engage_reach`, larmes arrêtées par un coin de mur).
        let reach = match (view.orbit_reach, view.engage_reach) {
            (Some(orbit), Some(engage)) => Some(orbit.min(engage)),
            (orbit, engage) => orbit.or(engage),
        };
        let close_in = match reach {
            Some(reach) => enemy.distance > reach,
            None => view.enemy_still && enemy.distance > STILL_TARGET_DISTANCE,
        };
        // m1-v3-bots-softlocks : ni recul ni tir vers un ennemi caché (mode `Floors`) : le bot
        // reculait dans la roche en vidant ses chargeurs sur elle (soft-locks des graines 63,
        // 111, 139, 149 ; munitions épuisées devant le boss, graines 23, 43, 76).
        if view.engage_strafe != 0 && enemy.distance <= STRAFE_MAX_DISTANCE {
            // Combat bloqué malgré l'approche : tourne autour de l'ennemi pour changer l'angle de tir.
            let away = view.position - enemy.position;
            let tangent = if view.engage_strafe > 0 {
                FixedVec2::new(-away.y, away.x)
            } else {
                FixedVec2::new(away.y, -away.x)
            };
            set_direction_buttons(&mut input, tangent);
        } else if reach.is_none()
            && enemy.distance < PRUDENT_MIN_DISTANCE
            && !view.enemy_still
            && view.enemy_visible
        {
            set_direction_buttons(&mut input, view.position - enemy.position); // s'éloigne
        } else if !view.enemy_visible || close_in || enemy.distance > PRUDENT_MAX_DISTANCE {
            // s'approche : par le chemin (ennemi caché, ou loin), ligne droite en repli
            let toward = enemy.position - view.position;
            match view.route {
                // m1-v3-bots-softlocks (throne_quad, graine 123456, 4 bots) : vers un ennemi
                // caché, la route suivie au signe faisait dépasser chaque case visée (élan) ;
                // le bot allait et venait dans un couloir sans prendre la sortie : pilotage en
                // vitesse, comme vers le portail.
                // Combat engagé et ligne dégagée : tout droit vers l'ennemi (jusqu'au contact, puis contournement) ; la
                // route contournait un coin de mur que les larmes ne passent pas (graines 63, 95, 119 d'`isaac`).
                Some(_) if view.enemy_visible && view.engage_reach.is_some() => {
                    steer(&mut input, toward)
                }
                Some(route) if !view.enemy_visible || view.engage_reach.is_some() => {
                    steer(&mut input, route)
                }
                _ => set_direction_buttons(&mut input, route_unless(false, view, toward)),
            }
        } // sinon : garde sa position (visible, dans la bande [MIN, MAX])

        if enemy.distance <= PRUDENT_MAX_DISTANCE {
            // T1.14 : relâcher entre deux tirs d'une arme non automatique ; jamais sans ligne
            // de tir (m1-v3-bots-softlocks)
            input.fire = view.trigger_ready && line_of_fire(view) && in_range(view, enemy.distance);
        }
    } else if let Some(portal) = view.portal {
        // T1.8 : plus d'ennemi, portail ouvert : y aller (niveau suivant) (T1.14, pilotage de m1-bots-apres-movement-feel)
        approach_portal(&mut input, view, portal);
    }

    manage_weapon(&mut input, view);
    input
}

/// m1-v3-bots-armes : la cible est à portée de l'arme en main (`BotView::fire_range`, `range` de
/// la config) ; sans portée connue (hors `Floors`), toujours vrai.
fn in_range(view: &BotView, distance: Fixed) -> bool {
    view.fire_range.is_none_or(|range| distance <= range)
}

/// m1-v3-bots-softlocks : `prudent` ne tire pas vers un ennemi caché (mode `Floors`) : il vidait
/// ses chargeurs dans la roche. Sauf un ennemi immobile de près (à moins de
/// [`STILL_TARGET_DISTANCE`]) que la ligne brute, sans marge, atteint : boss coincé contre la
/// roche (graine 43), caché avec la marge de 4 px alors que les balles le touchent. Une
/// tourelle derrière la roche (graine 76) reste sans tir : les chargeurs y partaient.
fn line_of_fire(view: &BotView) -> bool {
    view.enemy_visible
        || (view.enemy_still
            && view.enemy_shootable
            && view
                .nearest_enemy
                .is_some_and(|enemy| enemy.distance <= STILL_TARGET_DISTANCE))
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

/// Approche du portail (`prudent`, voir [`PORTAL_BRAKE_DISTANCE`]) : par le chemin tant qu'il
/// est loin, droit vers le portail dans les derniers [`PORTAL_BRAKE_DISTANCE`] sauf si un mur
/// coupe la ligne droite (le chemin alors).
fn approach_portal(input: &mut BoxInput, view: &BotView, portal: FixedVec2) {
    // La route n'existe sous `PORTAL_BRAKE_DISTANCE` que si un mur coupe la ligne droite
    // (m1-v3-bots-softlocks, `input::navigate`).
    steer(input, view.route.unwrap_or(portal - view.position));
}

/// Pilotage (m1-bots-apres-movement-feel, voir [`PORTAL_BRAKE_DISTANCE`]) : presse la
/// direction voulue quantifiée en 8 secteurs ; un axe dont la part est sous
/// [`STEER_AXIS_SHARE`] n'est pas pressé (un écart de quelques pixels sur l'autre axe ne fait
/// pas de pas en diagonale contre un coin de roche).
pub(crate) fn steer(input: &mut BoxInput, direction: FixedVec2) {
    let unit = direction.normalize_or_zero();
    let keep = |c: Fixed| {
        if c.abs() >= STEER_AXIS_SHARE {
            c
        } else {
            FIXED_ZERO
        }
    };
    set_direction_buttons(input, FixedVec2::new(keep(unit.x), keep(unit.y)));
}

/// `prudent` v1 (T1.14) : recharge si possible, sinon passe à une arme utilisable (même règle
/// que `chasseur`/`acheteur`) — sans ça, une mitrailleuse vide sans réserve bloquait le bot
/// devant le dernier ennemi d'un niveau.
pub(crate) fn manage_weapon(input: &mut BoxInput, view: &BotView) {
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

/// Vue de test partagée par les modules du crate (M2-T16).
#[cfg(test)]
pub(crate) mod tests_support {
    pub(crate) use super::tests::view;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::EnemyView;
    use bevy_fixed::fixed_math::new as fx;
    use game::character::player::input::{INPUT_DOWN, INPUT_LEFT, INPUT_RIGHT, INPUT_UP};

    pub(crate) fn view(position: FixedVec2) -> BotView {
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
            enemy_still: false,
            enemy_shootable: true,
            revive: None,
            engage_reach: None,
            engage_strafe: 0,
            hold_revive: false,
            loot: None,
            loot_interact: false,
            fire_range: None,
            orbit_reach: None,
            explore: None,
            aim: sim_core::aim::AimMode::Free,
        }
    }

    /// m1-bots-apres-movement-feel : la course s'arrête en 3 frames (≈ 5 px à pleine vitesse,
    /// rayon du portail 24) : plus de freinage, `prudent` va droit au portail, lancé ou non, et
    /// un petit écart sur l'autre axe (3 px sur 30) ne fait pas de diagonale.
    #[test]
    fn prudent_va_droit_au_portail() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.portal = Some(FixedVec2::new(fx(30.0), fx(3.0)));
        for velocity in [fx(150.0), fx(0.0), fx(-150.0)] {
            v.velocity = FixedVec2::new(velocity, fx(0.0));
            let input = decide(BotProfile::Prudent, &v, &mut rng());
            assert_eq!(input.buttons, INPUT_RIGHT, "lancé à {velocity}");
        }
        v.portal = Some(FixedVec2::new(fx(400.0), fx(0.0)));
        v.velocity = FixedVec2::new(fx(150.0), fx(0.0));
        assert_eq!(
            decide(BotProfile::Prudent, &v, &mut rng()).buttons,
            INPUT_RIGHT
        );
    }

    /// La route monte, le bot file vers la droite : il presse haut seulement (la course annule
    /// d'elle-même la vitesse transverse en 3 frames ; l'ancienne pressait gauche, m1-v3-bots-
    /// portail) ; une route en diagonale presse les deux axes, une route presque droite un seul.
    #[test]
    fn prudent_suit_la_route_en_huit_directions() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.portal = Some(FixedVec2::new(fx(0.0), fx(150.0)));
        v.route = Some(FixedVec2::new(fx(0.0), fx(8.0)));
        v.velocity = FixedVec2::new(fx(140.0), fx(0.0));
        assert_eq!(
            decide(BotProfile::Prudent, &v, &mut rng()).buttons,
            INPUT_UP
        );
        v.route = Some(FixedVec2::new(fx(-8.0), fx(8.0)));
        assert_eq!(
            decide(BotProfile::Prudent, &v, &mut rng()).buttons,
            INPUT_UP | INPUT_LEFT
        );
        // 2 px sur 8 : 14° de la verticale, sous les 22,5° d'un secteur
        v.route = Some(FixedVec2::new(fx(2.0), fx(8.0)));
        assert_eq!(
            decide(BotProfile::Prudent, &v, &mut rng()).buttons,
            INPUT_UP
        );
    }

    /// m1-v3-bots-portail : ennemi immobile visible à 300 (dans la bande de 320) : `prudent`
    /// suit le chemin pour s'en rapprocher, et tire toujours ; à 150 (sous la distance de
    /// recul de 180) il ne recule pas ; à 100, il tient sa position.
    #[test]
    fn prudent_se_rapproche_d_un_ennemi_immobile() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        let enemy_at = |d: f32| {
            Some(EnemyView {
                position: FixedVec2::new(fx(d), fx(0.0)),
                distance: fx(d),
            })
        };
        v.nearest_enemy = enemy_at(300.0);
        v.route = Some(FixedVec2::new(fx(0.0), fx(8.0)));
        let mobile = decide(BotProfile::Prudent, &v, &mut rng());
        assert_eq!(mobile.buttons & INPUT_UP, 0, "mobile : garde sa position");
        v.enemy_still = true;
        let input = decide(BotProfile::Prudent, &v, &mut rng());
        assert_ne!(input.buttons & INPUT_UP, 0, "immobile : suit le chemin");
        assert!(input.fire);
        v.nearest_enemy = enemy_at(150.0);
        let input = decide(BotProfile::Prudent, &v, &mut rng());
        assert_eq!(input.buttons & INPUT_LEFT, 0, "immobile : ne recule pas");
        assert_ne!(input.buttons & INPUT_UP, 0);
        v.nearest_enemy = enemy_at(100.0);
        assert_eq!(decide(BotProfile::Prudent, &v, &mut rng()).buttons, 0);
    }

    /// m1-v3-bots-reanimation : un coéquipier à terre à relever passe avant la chasse et la
    /// garde de position : déplacement vers lui, Interaction tenue à portée, tir conservé.
    #[test]
    fn prudent_et_fonceur_relevent_un_coequipier() {
        use crate::view::ReviveView;
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.nearest_enemy = Some(EnemyView {
            position: FixedVec2::new(fx(250.0), fx(0.0)),
            distance: fx(250.0),
        });
        v.revive = Some(ReviveView {
            direction: FixedVec2::new(fx(0.0), fx(8.0)),
            interact: false,
        });
        for profile in [BotProfile::Prudent, BotProfile::Fonceur] {
            let input = decide(profile, &v, &mut rng());
            assert_ne!(
                input.buttons & INPUT_UP,
                0,
                "{profile:?} : vers le coéquipier"
            );
            assert_eq!(input.buttons & INPUT_INTERACTION, 0);
        }
        assert!(
            decide(BotProfile::Prudent, &v, &mut rng()).fire,
            "prudent tire encore"
        );
        v.revive = Some(ReviveView {
            direction: FixedVec2::ZERO,
            interact: true,
        });
        for profile in [BotProfile::Prudent, BotProfile::Fonceur] {
            let input = decide(profile, &v, &mut rng());
            assert_ne!(input.buttons & INPUT_INTERACTION, 0, "{profile:?} : relève");
            assert_eq!(
                input.buttons & (INPUT_UP | INPUT_DOWN | INPUT_LEFT | INPUT_RIGHT),
                0,
                "{profile:?} : immobile pendant la réanimation"
            );
        }
    }

    #[test]
    fn reanimation_seulement_en_urgence() {
        use crate::input::revive_urgent;
        assert!(
            !revive_urgent(2000, 1000),
            "1000 frames de marge : se battre"
        );
        assert!(revive_urgent(1600, 1000), "600 : urgent");
        assert!(revive_urgent(1200, 1000));
        assert!(revive_urgent(900, 1000), "déjà échu");
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

    /// Portail loin : le chemin ; près (sous `PORTAL_BRAKE_DISTANCE`) : la ligne droite.
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
        // Près, ligne droite libre (`input::navigate` ne donne pas de route) : pas direct
        v.portal = Some(FixedVec2::new(fx(30.0), fx(0.0)));
        v.route = None;
        let input = decide(BotProfile::Prudent, &v, &mut rng());
        assert_ne!(input.buttons & INPUT_RIGHT, 0, "près : ligne droite");
    }

    /// m1-v3-bots-softlocks (graine 81) : portail à 26 px à gauche, 4 px plus haut, roche entre
    /// les deux. Tout droit, la composante verticale (sous la zone morte du pilotage) tombe : le
    /// bot pousserait dans la roche. La route (donnée seulement quand un mur coupe la ligne
    /// droite) le fait monter (y > 0) pour contourner.
    #[test]
    fn portail_contre_la_roche_par_le_chemin() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.portal = Some(FixedVec2::new(fx(-26.0), fx(4.0)));
        let input = decide(BotProfile::Prudent, &v, &mut rng());
        assert_ne!(input.buttons & INPUT_LEFT, 0);
        assert_eq!(
            input.buttons & INPUT_UP,
            0,
            "sans route : tout droit, dans la roche"
        );
        v.route = Some(FixedVec2::new(fx(-2.0), fx(12.0)));
        let input = decide(BotProfile::Prudent, &v, &mut rng());
        assert_ne!(input.buttons & INPUT_UP, 0, "contourne par le chemin");
        assert_eq!(input.buttons & INPUT_LEFT, 0);
    }

    /// m1-v3-bots-softlocks (graine 139) : ennemi caché à 169 px (sous `PRUDENT_MIN_DISTANCE`),
    /// en haut à droite. Visible, `prudent` recule et tire ; caché, il ne recule pas (il reculait
    /// dans la roche), ne tire pas (chargeurs vidés dans la roche) et va le chercher par le chemin.
    #[test]
    fn prudent_ni_recul_ni_tir_vers_un_ennemi_cache() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.nearest_enemy = Some(EnemyView {
            position: FixedVec2::new(fx(167.0), fx(30.0)),
            distance: fx(169.0),
        });
        v.route = Some(FixedVec2::new(fx(0.0), fx(8.0)));
        let input = decide(BotProfile::Prudent, &v, &mut rng());
        assert_ne!(input.buttons & INPUT_LEFT, 0, "visible : recule");
        assert!(input.fire, "visible : tire");
        v.enemy_visible = false;
        let input = decide(BotProfile::Prudent, &v, &mut rng());
        assert_eq!(
            input.buttons & (INPUT_LEFT | INPUT_DOWN),
            0,
            "caché : pas de recul"
        );
        assert_ne!(input.buttons & INPUT_UP, 0, "caché : par le chemin");
        assert!(!input.fire, "caché : pas de tir");
        v.enemy_still = true;
        assert!(
            !decide(BotProfile::Prudent, &v, &mut rng()).fire,
            "caché, immobile mais loin (graine 76) : pas de tir"
        );
        v.nearest_enemy = Some(EnemyView {
            position: FixedVec2::new(fx(85.0), fx(0.0)),
            distance: fx(85.0),
        });
        assert!(
            decide(BotProfile::Prudent, &v, &mut rng()).fire,
            "caché mais immobile, de près, ligne brute libre (boss coincé, graine 43) : tire"
        );
        v.enemy_shootable = false;
        assert!(
            !decide(BotProfile::Prudent, &v, &mut rng()).fire,
            "immobile derrière la roche (tourelle, graine 76) : pas de tir"
        );
    }

    /// m1-v3-bots-softlocks (graines 23, 43, 76) : à sec, `prudent` et `fonceur` vont au butin
    /// (pas suivant de `BotView::loot`) au lieu de garder leur position devant l'ennemi.
    #[test]
    fn a_sec_prudent_et_fonceur_vont_au_butin() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.nearest_enemy = Some(EnemyView {
            position: FixedVec2::new(fx(250.0), fx(0.0)),
            distance: fx(250.0),
        });
        v.loot = Some(FixedVec2::new(fx(-8.0), fx(0.0)));
        for profile in [BotProfile::Prudent, BotProfile::Fonceur] {
            let input = decide(profile, &v, &mut rng());
            assert_ne!(input.buttons & INPUT_LEFT, 0, "{profile:?} : vers le butin");
            assert_eq!(input.buttons & INPUT_RIGHT, 0, "{profile:?}");
        }
    }

    /// m1-v3-bots-softlocks (throne_quad) : ennemi caché, route vers le haut, le bot file encore
    /// vers le bas : il presse haut ; lancé vers le haut, il continue (m1-bots-apres-movement-
    /// feel : plus de vitesse de croisière, il presse la route).
    #[test]
    fn vers_un_ennemi_cache_la_route_est_suivie() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.nearest_enemy = Some(EnemyView {
            position: FixedVec2::new(fx(700.0), fx(200.0)),
            distance: fx(728.0),
        });
        v.enemy_visible = false;
        v.route = Some(FixedVec2::new(fx(0.0), fx(9.0)));
        v.velocity = FixedVec2::new(fx(0.0), fx(-110.0));
        let input = decide(BotProfile::Prudent, &v, &mut rng());
        assert_ne!(input.buttons & INPUT_UP, 0);
        assert_eq!(input.buttons & (INPUT_LEFT | INPUT_RIGHT), 0);
        v.velocity = FixedVec2::new(fx(0.0), fx(150.0));
        assert_eq!(
            decide(BotProfile::Prudent, &v, &mut rng()).buttons,
            INPUT_UP
        );
    }

    /// m1-v3-bots-armes : pas de tir au-delà de la portée de l'arme en main (`fire_range`) ;
    /// sans portée connue (hors `Floors`), comme avant.
    #[test]
    fn tir_seulement_a_portee_de_l_arme() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.nearest_enemy = Some(EnemyView {
            position: FixedVec2::new(fx(200.0), fx(0.0)),
            distance: fx(200.0),
        });
        for profile in [BotProfile::Prudent, BotProfile::Fonceur] {
            assert!(
                decide(profile, &v, &mut rng()).fire,
                "{profile:?} : sans portée connue"
            );
            v.fire_range = Some(fx(150.0));
            assert!(
                !decide(profile, &v, &mut rng()).fire,
                "{profile:?} : hors de portée"
            );
            v.fire_range = Some(fx(300.0));
            assert!(
                decide(profile, &v, &mut rng()).fire,
                "{profile:?} : à portée"
            );
            v.fire_range = None;
        }
    }

    /// Larmes en orbite (`orbit_reach`) : le bot s'approche d'un ennemi immobile à 94 unités (sans orbite, il garde sa
    /// position sous `STILL_TARGET_DISTANCE`) et ne recule plus devant un ennemi mobile à 100 ; arrivé à portée de
    /// l'orbite, il se tient.
    #[test]
    fn larmes_en_orbite_le_bot_s_approche() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.nearest_enemy = Some(EnemyView {
            position: FixedVec2::new(fx(94.0), fx(0.0)),
            distance: fx(94.0),
        });
        v.enemy_still = true;
        assert_eq!(
            decide(BotProfile::Prudent, &v, &mut rng()).buttons,
            0,
            "sans orbite : il tient"
        );
        v.orbit_reach = Some(fx(40.0));
        assert_eq!(
            decide(BotProfile::Prudent, &v, &mut rng()).buttons & INPUT_RIGHT,
            INPUT_RIGHT,
            "avec orbite : il s'approche"
        );
        v.enemy_still = false;
        v.nearest_enemy = Some(EnemyView {
            position: FixedVec2::new(fx(100.0), fx(0.0)),
            distance: fx(100.0),
        });
        assert_eq!(
            decide(BotProfile::Prudent, &v, &mut rng()).buttons & INPUT_LEFT,
            0,
            "ennemi mobile : il ne recule pas"
        );
        v.nearest_enemy = Some(EnemyView {
            position: FixedVec2::new(fx(30.0), fx(0.0)),
            distance: fx(30.0),
        });
        v.enemy_still = true;
        assert_eq!(
            decide(BotProfile::Prudent, &v, &mut rng()).buttons,
            0,
            "à portée de l'orbite : il tient"
        );
    }

    /// m1-v3-bots-armes : à portée d'une arme au sol choisie, Interaction tenue.
    #[test]
    fn ramasser_une_arme_tient_interaction() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.loot = Some(FixedVec2::ZERO);
        v.loot_interact = true;
        for profile in [BotProfile::Prudent, BotProfile::Fonceur] {
            let input = decide(profile, &v, &mut rng());
            assert_ne!(input.buttons & INPUT_INTERACTION, 0, "{profile:?}");
        }
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

    /// Suivi Q17 : un bot à visée `FourWay` écrit la direction que le moteur retiendra, ne tire que si
    /// la cible est dans la file, et se décale sinon.
    #[test]
    fn four_way_vise_la_direction_quantifiee() {
        let target = |x: f32, y: f32| {
            let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
            v.aim = AimMode::FourWay;
            v.nearest_enemy = Some(EnemyView {
                position: FixedVec2::new(fx(x), fx(y)),
                distance: fx(x.hypot(y)),
            });
            v
        };
        // Dans la file (écart 10 ≤ 14) : pan = est plein, tir conservé, aucune dérive verticale.
        let aligned = decide(BotProfile::Prudent, &target(200.0, 10.0), &mut rng());
        assert_eq!((aligned.pan_x, aligned.pan_y), (127, 0));
        assert!(aligned.fire);
        assert_eq!(aligned.buttons & (INPUT_UP | INPUT_DOWN), 0);
        // Hors de la file (écart 60) : même direction, gâchette relâchée, décalage vers la cible (haut).
        let off = decide(BotProfile::Prudent, &target(200.0, 60.0), &mut rng());
        assert_eq!((off.pan_x, off.pan_y), (127, 0));
        assert!(!off.fire);
        assert_ne!(off.buttons & INPUT_UP, 0, "se décale vers la cible");
        assert_eq!(off.buttons & INPUT_DOWN, 0);
        // Cible au sud, dominante verticale : bas, décalage horizontal vers la droite.
        let south = decide(BotProfile::Prudent, &target(60.0, -200.0), &mut rng());
        assert_eq!((south.pan_x, south.pan_y), (0, -127));
        assert!(!south.fire);
        assert_ne!(south.buttons & INPUT_RIGHT, 0);
        // Cible lointaine (> 320 sur l'axe) : seule la direction change, le profil approche comme avant.
        let far = decide(BotProfile::Prudent, &target(500.0, 60.0), &mut rng());
        assert_eq!((far.pan_x, far.pan_y), (127, 0));
        assert!(!far.fire, "hors de portée de tir");
    }

    /// `hold_revive` (`combat_aware`) : le bot qui tient une réanimation (Interaction, direction nulle) n'ajoute pas de
    /// décalage pour s'aligner sur un ennemi hors de la file (il quitterait la portée de son coéquipier) ; sans
    /// `hold_revive`, comme avant.
    #[test]
    fn four_way_ne_quitte_pas_une_reanimation_tenue() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.aim = AimMode::FourWay;
        v.nearest_enemy = Some(EnemyView {
            position: FixedVec2::new(fx(200.0), fx(60.0)),
            distance: fx(208.8),
        });
        v.revive = Some(crate::view::ReviveView {
            direction: FixedVec2::ZERO,
            interact: true,
        });
        let before = decide(BotProfile::Prudent, &v, &mut rng());
        assert_ne!(
            before.buttons & INPUT_UP,
            0,
            "sans hold_revive : le décalage de main"
        );
        v.hold_revive = true;
        let held = decide(BotProfile::Prudent, &v, &mut rng());
        assert_eq!(
            held.buttons & (INPUT_UP | INPUT_DOWN | INPUT_LEFT | INPUT_RIGHT),
            0
        );
        assert_eq!(
            (held.pan_x, held.pan_y),
            (127, 0),
            "la direction de tir reste quantifiée"
        );
    }

    /// `Free` : l'input du profil, au bit près (le contenu d'avant ne bouge pas).
    #[test]
    fn libre_ne_change_rien() {
        let mut v = view(FixedVec2::new(fx(0.0), fx(0.0)));
        v.nearest_enemy = Some(EnemyView {
            position: FixedVec2::new(fx(200.0), fx(60.0)),
            distance: fx(208.8),
        });
        let free = decide(BotProfile::Prudent, &v, &mut rng());
        assert_eq!((free.pan_x, free.pan_y), (200, 60));
        assert!(free.fire);
    }

    /// Le décalage remplace une dérive de sens contraire, la file garde le profil sans dérive, et sans
    /// visée rien ne bouge.
    #[test]
    fn four_way_pilote_l_axe_perpendiculaire() {
        let mut idle = BoxInput::default();
        apply_four_way(&mut idle, true);
        assert_eq!(idle, BoxInput::default());
        // Hors file, le profil s'éloignait (bas) alors que la cible est en haut : décalage vers le haut,
        // le mouvement sur l'axe de tir (gauche) est gardé.
        let mut retreating = BoxInput {
            pan_x: 200,
            pan_y: 60,
            fire: true,
            buttons: INPUT_DOWN | INPUT_LEFT,
            ..BoxInput::default()
        };
        apply_four_way(&mut retreating, true);
        assert!(!retreating.fire);
        assert_eq!(retreating.buttons, INPUT_UP | INPUT_LEFT);
        // Dans la file, la dérive perpendiculaire du profil est retirée.
        let mut drifting = BoxInput {
            pan_x: 200,
            pan_y: 5,
            fire: true,
            buttons: INPUT_UP | INPUT_RIGHT,
            ..BoxInput::default()
        };
        apply_four_way(&mut drifting, true);
        assert!(drifting.fire);
        assert_eq!(drifting.buttons, INPUT_RIGHT);
    }

    /// Sans ligne dégagée (mur), le bot garde sa route : seule la direction de visée est écrite.
    #[test]
    fn four_way_sans_ligne_laisse_la_route() {
        let mut input = BoxInput {
            pan_x: 200,
            pan_y: 60,
            fire: true,
            buttons: INPUT_DOWN | INPUT_LEFT,
            ..BoxInput::default()
        };
        apply_four_way(&mut input, false);
        assert_eq!((input.pan_x, input.pan_y), (127, 0));
        assert!(input.fire);
        assert_eq!(input.buttons, INPUT_DOWN | INPUT_LEFT);
    }
}
