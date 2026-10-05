//! Vue en fixed-point de l'état lisible par un joueur, construite par
//! [`crate::input::read_bot_inputs`] et consommée par [`crate::decide::decide`]. Pas de
//! dépendance à Bevy ECS : uniquement des données, en `Fixed` (jamais `f32`).

use bevy_fixed::fixed_math::{Fixed, FixedVec2};

/// Ennemi le plus proche du joueur, vu par un bot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnemyView {
    pub position: FixedVec2,
    /// Distance au joueur (`FixedVec2::distance`, jamais recalculée dans `decide`).
    pub distance: Fixed,
}

/// Fenêtre la plus proche du joueur, vue par un bot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowView {
    pub position: FixedVec2,
    /// Santé courante (`WindowHealth::current`, 0..=max).
    pub health: u8,
    /// Distance au joueur.
    pub distance: Fixed,
    /// `true` si la fenêtre a perdu de la santé (`current < max`) : ça vaut la peine de la
    /// réparer. Ne dépend pas du cooldown de réparation (`WindowHealth::can_repair_after_frame`) :
    /// un bot qui maintient l'interaction pendant le cooldown ne fait juste rien ce temps-là.
    pub repairable: bool,
}

/// Projectile d'une autre équipe proche du joueur (T1.14, esquive de `prudent` v1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProjectileView {
    pub position: FixedVec2,
    /// Vitesse en unités par frame (`Bullet::velocity`).
    pub velocity: FixedVec2,
    /// Rayon de collision du projectile (collider de la balle, `Size` compris).
    pub size: Fixed,
    /// Distance au joueur.
    pub distance: Fixed,
}

/// Rayon dans lequel les projectiles entrent dans la vue (T1.14).
pub const PROJECTILE_VIEW_RANGE: Fixed = Fixed::from_bits(320 << 16);
/// Nombre maximal de projectiles dans la vue (les plus proches).
pub const PROJECTILE_VIEW_MAX: usize = 16;

/// Vue d'un joueur sur son propre état et son entourage immédiat, en fixed-point. Construite
/// une fois par joueur local piloté par un bot, à chaque frame de `ReadInputs`.
#[derive(Debug, Clone, PartialEq)]
pub struct BotView {
    pub position: FixedVec2,
    pub health: Fixed,
    pub health_max: Fixed,
    /// Munitions dans le chargeur de l'arme active (0 si aucune arme).
    pub ammo: u32,
    /// Vague courante (`WaveState::current_wave`).
    pub wave: u32,
    pub nearest_enemy: Option<EnemyView>,
    pub nearest_window: Option<WindowView>,
    /// Navigation et interactions des profils v1 ; absent pour les profils v0.
    pub hunter: Option<crate::hunter::HunterView>,
    /// Portail ouvert du mode `Floors` (T1.8, `run::FloorState`), `None` sinon (portail fermé,
    /// ou autre mode) : un bot sans ennemi s'y rend pour passer au niveau suivant.
    pub portal: Option<FixedVec2>,
    /// T1.14 : projectiles d'une autre équipe à moins de [`PROJECTILE_VIEW_RANGE`], au plus
    /// [`PROJECTILE_VIEW_MAX`] (les plus proches), triés par `GgrsNetId` ([`projectile_views`]).
    pub projectiles: Vec<ProjectileView>,
    /// T1.14 : rayon du corps du joueur (demi-diagonale de son collider), pour l'esquive.
    pub body_radius: Fixed,
    /// T1.14 (gestion d'arme de `prudent` v1, même règle que `chasseur`/`acheteur`) : chargeur
    /// vide et réserve disponible.
    pub reload: bool,
    /// Arme active inutilisable (vide, sans réserve) et une autre arme utilisable.
    pub switch_weapon: bool,
    /// Le tir peut repartir : arme automatique, ou détente relâchée (`WeaponState::is_firing`
    /// faux) pour `Manual`/`Shotgun`/`Burst`.
    pub trigger_ready: bool,
    /// T1.14 : vitesse du joueur (`Velocity::main`, unités par seconde), pour freiner à
    /// l'approche du portail.
    pub velocity: FixedVec2,
    /// Ligne de vue sans mur (`Wall`) entre le joueur et l'ennemi le plus proche (faux sans
    /// ennemi) : `prudent`/`fonceur` ne vont en ligne droite que vers un ennemi visible. Vrai
    /// quand la navigation ne tourne pas (hors mode `Floors`) : comportement de T1.14.
    pub enemy_visible: bool,
    /// Direction du pas suivant par le champ de navigation ([`crate::navigation`], grille de 8)
    /// vers le but courant : poste de tir de l'ennemi le plus proche par le chemin, ou portail
    /// ouvert sans ennemi. `None` : pas de chemin (repli : ligne droite).
    pub route: Option<FixedVec2>,
    /// m1-v3-bots-portail : l'ennemi le plus proche est immobile (`MoveSpeed` de base nulle :
    /// tourelle ; ou `Velocity::main` nulle : ennemi coincé dans un recoin). `prudent` s'en
    /// rapproche jusqu'à [`crate::decide::STILL_TARGET_DISTANCE`] sans reculer, au lieu de
    /// tirer de loin (dispersion : il vidait ses munitions, ou touchait si rarement que la
    /// partie n'avançait plus). Faux hors navigation (hors mode `Floors`).
    pub enemy_still: bool,
}

/// Sélection des projectiles de la vue : `candidates` = (net id, équipe adverse ?, vue).
/// Garde ceux d'une autre équipe à portée, les [`PROJECTILE_VIEW_MAX`] plus proches
/// (distance, puis net id), rendus dans l'ordre des `GgrsNetId`.
pub fn projectile_views(
    candidates: impl Iterator<Item = (usize, bool, ProjectileView)>,
) -> Vec<ProjectileView> {
    let mut kept: Vec<(usize, ProjectileView)> = candidates
        .filter(|(_, hostile, p)| *hostile && p.distance < PROJECTILE_VIEW_RANGE)
        .map(|(id, _, p)| (id, p))
        .collect();
    kept.sort_by(|a, b| a.1.distance.cmp(&b.1.distance).then(a.0.cmp(&b.0)));
    kept.truncate(PROJECTILE_VIEW_MAX);
    kept.sort_by_key(|(id, _)| *id);
    kept.into_iter().map(|(_, p)| p).collect()
}

/// Choisit l'élément le plus proche parmi `candidates` (distance, puis `GgrsNetId.0` comme
/// tie-break déterministe, CLAUDE.md règle 4 : jamais le premier trouvé dans un ordre
/// arbitraire). Le résultat ne dépend pas de l'ordre d'itération de `candidates` : ce runner
/// trie quand même ses requêtes par `GgrsNetId` avant d'appeler cette fonction (convention du
/// projet, `order_iter!`), mais cette fonction elle-même n'en a pas besoin pour être correcte.
pub fn nearest_by_net_id<T: Copy>(
    candidates: impl Iterator<Item = (usize, Fixed, T)>,
) -> Option<T> {
    let mut best: Option<(usize, Fixed, T)> = None;
    for (net_id, distance, value) in candidates {
        let better = match &best {
            None => true,
            Some((best_net_id, best_distance, _)) => {
                distance < *best_distance || (distance == *best_distance && net_id < *best_net_id)
            }
        };
        if better {
            best = Some((net_id, distance, value));
        }
    }
    best.map(|(_, _, value)| value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fixed::fixed_math::new as fx;

    #[test]
    fn nearest_by_net_id_picks_closest() {
        let candidates = vec![(5usize, fx(100.0), "far"), (2usize, fx(10.0), "near")];
        assert_eq!(
            nearest_by_net_id(candidates.into_iter()),
            Some("near"),
            "l'élément à distance 10 doit gagner sur celui à distance 100"
        );
    }

    #[test]
    fn nearest_by_net_id_breaks_ties_by_net_id() {
        // Même distance : le plus petit GgrsNetId gagne, quel que soit l'ordre d'itération.
        let a = vec![(9usize, fx(50.0), "a"), (3usize, fx(50.0), "b")];
        let b = vec![(3usize, fx(50.0), "b"), (9usize, fx(50.0), "a")];
        assert_eq!(nearest_by_net_id(a.into_iter()), Some("b"));
        assert_eq!(nearest_by_net_id(b.into_iter()), Some("b"));
    }

    #[test]
    fn projectile_views_filtre_equipe_portee_et_trie_par_net_id() {
        let p = |d: f32| ProjectileView {
            position: FixedVec2::new(fx(d), fx(0.0)),
            velocity: FixedVec2::ZERO,
            size: fx(5.0),
            distance: fx(d),
        };
        let views = projectile_views(
            vec![
                (9usize, true, p(50.0)),
                (2, true, p(100.0)),
                (5, false, p(10.0)), // même équipe : ignoré
                (1, true, p(400.0)), // hors portée
            ]
            .into_iter(),
        );
        assert_eq!(views, vec![p(100.0), p(50.0)], "ordre des net ids 2 puis 9");
        // Au plus 16, les plus proches
        let many = projectile_views((0..20usize).map(|i| (i, true, p(300.0 - i as f32))));
        assert_eq!(many.len(), PROJECTILE_VIEW_MAX);
        assert!(many.iter().all(|v| v.distance <= fx(296.0)));
    }

    #[test]
    fn nearest_by_net_id_empty_is_none() {
        let empty: Vec<(usize, Fixed, ())> = vec![];
        assert_eq!(nearest_by_net_id(empty.into_iter()), None);
    }
}
