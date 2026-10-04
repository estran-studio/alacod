//! Émetteurs (T1.2, chantier B5 v1, suite de T1.1) : une entité qui tire selon un
//! [`Pattern`] **dans le temps** — `Telegraph`, `Wait`, `Ring.every`, `Scatter` — là où
//! `on_expire` (§16) ne joue qu'une salve instantanée. Voir `docs/conventions.md` §20.
//!
//! # Sémantique
//!
//! Le pattern (résolu : plus de `Named`, voir [`crate::projectile::PatternLibrary`]) est
//! aplati en une suite d'étapes ([`Sequence`](Pattern::Sequence) imbriquées dépliées dans l'ordre RON) :
//!
//! - `Aimed`, `Spread`, `Ring` (`every = 0`) : une salve, comme §16 ; durée 0 frame, l'étape
//!   suivante joue la même frame ;
//! - `Ring { every: n > 0 }` : une salve tout de suite, puis une toutes les `n` frames **en
//!   tâche de fond** jusqu'à la fin de la séquence ; un `Ring` seul (pas dans une `Sequence`)
//!   est infini (l'émetteur ne finit jamais de lui-même) ;
//! - `Scatter` : `count` tirs à des angles tirés dans `±spread/2` autour de la visée, flux
//!   RNG `"patterns"`, consommé **uniquement** ici, dans l'ordre des `GgrsNetId` des
//!   émetteurs (même graine = même tir quel que soit le nombre de joueurs) ;
//! - `Telegraph(n)` : `n` frames sans tir, lisibles par la présentation
//!   ([`Emitter::telegraphing`]) ; `Wait(n)` : `n` frames sans tir.
//!
//! La visée est **figée au départ** ([`Emitter::aim`]) : direction de `Aimed`/`Scatter`,
//! centre de `Spread`, premier rayon de `Ring`. À la fin de la séquence, [`emitter_system`]
//! retire le composant (un ennemi repasse alors en refroidissement, voir
//! `game::character::enemy::ai::behavior::enemy_attack_system`).
//!
//! Les projectiles tirés viennent de la table `projectiles` de l'arme de l'émetteur
//! ([`Emitter::table`]) et passent par la seule fonction d'apparition
//! ([`crate::weapons::spawn_bullet`]), puis par la même chaîne que ceux des joueurs
//! (`Weapon` puis `Projectiles` : collisions, `on_hit`, `on_expire`).

use std::sync::Arc;

use bevy::{
    log::{tracing::span, Level},
    prelude::*,
};
use bevy_fixed::{
    fixed_math::{self, Fixed, FixedVec2},
    rng::RngStreams,
};
use sim_core::{damage::FriendlyFire, tag::Tags, team::Team};
use utils::{
    frame::FrameCount,
    net_id::{GgrsNetId, GgrsNetIdFactory},
    order_mut_iter,
};

use crate::{
    collider::CollisionSettings,
    projectile::{
        angle_of, direction_of, pattern_shots, Pattern, Projectile, ProjectileTable, Shot,
    },
    weapons::{spawn_bullet, BulletSpawn, BulletType, NO_PLAYER_HANDLE},
};

/// Nom du flux RNG des émetteurs (`Scatter`) : aucun autre système ne le consomme.
pub const PATTERNS_RNG_STREAM: &str = "patterns";

/// `Ring.every > 0` en cours : prochaine salve à `next_frame`.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RingRepeat {
    /// Index de l'étape `Ring` dans [`Emitter::steps`].
    pub step: u32,
    pub next_frame: u32,
    /// Salves déjà tirées (la première comprise).
    pub fired: u32,
}

/// État rollback d'un émetteur (enregistré par
/// `utils::rollback::RollbackTraceApp::rollback_and_trace_neutral` : aucune entité
/// existante n'en porte, les traces existantes ne bougent pas).
///
/// `Hash`/`Debug` manuels : le pattern est représenté par son nom (`pattern_name`) et la
/// table de l'arme par ses clés — partagés par `Arc`, identiques sur tous les clients ;
/// tout l'état qui avance (index, frames, couronnes, visée) est haché.
#[derive(Component, Clone)]
pub struct Emitter {
    /// Nom du pattern de contenu (`patterns/<nom>.ron`), ou une description pour un pattern
    /// inline.
    pub pattern_name: String,
    /// Étapes du pattern résolu et aplati (voir la doc du module).
    pub steps: Arc<Vec<Pattern>>,
    /// Un `Ring { every > 0 }` seul : l'émetteur ne finit jamais de lui-même.
    pub infinite: bool,
    /// Arme dont la table fournit les projectiles (`Emitter::table`).
    pub weapon: String,
    pub table: Arc<ProjectileTable>,
    /// Multiplicateur de dégâts du tireur au départ (stat `Damage`).
    pub damage_mult: Fixed,
    /// Politique de tir ami de l'arme.
    pub friendly_fire: FriendlyFire,
    /// Direction de visée, figée au départ de la séquence (unitaire).
    pub aim: FixedVec2,
    /// Frame de pose de l'émetteur ; la première étape joue à la frame suivante.
    pub started_at: u32,
    /// Prochaine étape à jouer.
    pub next_step: u32,
    /// Frames restantes de l'étape bloquante en cours (`Telegraph`/`Wait`).
    pub wait_left: u32,
    /// L'étape bloquante en cours est un `Telegraph`.
    pub telegraph: bool,
    pub rings: Vec<RingRepeat>,
    /// Séquence terminée : retiré par [`emitter_system`] cette frame.
    pub finished: bool,
}

impl std::hash::Hash for Emitter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pattern_name.hash(state);
        self.infinite.hash(state);
        self.weapon.hash(state);
        self.damage_mult.hash(state);
        self.friendly_fire.hash(state);
        self.aim.x.hash(state);
        self.aim.y.hash(state);
        self.started_at.hash(state);
        self.next_step.hash(state);
        self.wait_left.hash(state);
        self.telegraph.hash(state);
        self.rings.hash(state);
        self.finished.hash(state);
    }
}

impl std::fmt::Debug for Emitter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Emitter")
            .field("pattern", &self.pattern_name)
            .field("steps", &self.steps.len())
            .field("infinite", &self.infinite)
            .field("weapon", &self.weapon)
            .field("table", &self.table.keys().collect::<Vec<_>>())
            .field("damage_mult", &self.damage_mult)
            .field("friendly_fire", &self.friendly_fire)
            .field("aim", &self.aim)
            .field("started_at", &self.started_at)
            .field("next_step", &self.next_step)
            .field("wait_left", &self.wait_left)
            .field("telegraph", &self.telegraph)
            .field("rings", &self.rings)
            .field("finished", &self.finished)
            .finish()
    }
}

/// Déplie les `Sequence` imbriquées, dans l'ordre RON.
fn flatten(pattern: &Pattern, out: &mut Vec<Pattern>) {
    match pattern {
        Pattern::Sequence(children) => children.iter().for_each(|child| flatten(child, out)),
        other => out.push(other.clone()),
    }
}

impl Emitter {
    /// Émetteur au repos sur `pattern` (déjà résolu, sans `Named` : voir
    /// [`crate::projectile::PatternLibrary::resolve`]), visant `aim`, posé à la frame `frame`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pattern_name: impl Into<String>,
        pattern: &Pattern,
        weapon: impl Into<String>,
        table: Arc<ProjectileTable>,
        damage_mult: Fixed,
        friendly_fire: FriendlyFire,
        aim: FixedVec2,
        frame: u32,
    ) -> Self {
        let mut steps = Vec::new();
        flatten(pattern, &mut steps);
        let infinite = matches!(pattern, Pattern::Ring { every, .. } if *every > 0);
        Self {
            pattern_name: pattern_name.into(),
            steps: Arc::new(steps),
            infinite,
            weapon: weapon.into(),
            table,
            damage_mult,
            friendly_fire,
            aim: aim.normalize_or_zero(),
            started_at: frame,
            next_step: 0,
            wait_left: 0,
            telegraph: false,
            rings: Vec::new(),
            finished: false,
        }
    }

    /// Frames restantes d'un `Telegraph` en cours (lu par la présentation : le dessin du
    /// télégraphe est T1.17), `None` hors télégraphe.
    pub fn telegraphing(&self) -> Option<u32> {
        (self.telegraph && self.wait_left > 0).then_some(self.wait_left)
    }

    /// Salve d'une étape instantanée. `random` tire une valeur dans `[0, 1)` (flux
    /// `"patterns"`) : appelée seulement par `Scatter`.
    fn salvo(&self, step: &Pattern, random: &mut impl FnMut() -> Fixed) -> Vec<Shot> {
        match step {
            Pattern::Scatter {
                count,
                spread,
                projectile,
            } => {
                let center = angle_of(self.aim);
                (0..*count)
                    .map(|_| {
                        let offset = random()
                            .saturating_sub(fixed_math::FIXED_HALF)
                            .saturating_mul(*spread);
                        Shot {
                            projectile: projectile.clone(),
                            direction: direction_of(center + offset),
                            speed: None,
                        }
                    })
                    .collect()
            }
            other => pattern_shots(other, self.aim, Some(self.aim)),
        }
    }

    /// Fait avancer l'émetteur d'une frame (`frame`) et rend les projectiles à tirer, dans
    /// l'ordre : étapes de la séquence, puis couronnes de fond. Pose [`Emitter::finished`]
    /// à la fin de la séquence (jamais pour un émetteur [`Emitter::infinite`]).
    pub fn tick(&mut self, frame: u32, random: &mut impl FnMut() -> Fixed) -> Vec<Shot> {
        let mut shots = Vec::new();
        if self.finished {
            return shots;
        }
        let steps = self.steps.clone();

        // Étape bloquante en cours.
        let blocked = if self.wait_left > 0 {
            self.wait_left -= 1;
            self.wait_left > 0
        } else {
            false
        };
        if !blocked {
            self.telegraph = false;
            while let Some(step) = steps.get(self.next_step as usize) {
                let index = self.next_step;
                self.next_step += 1;
                match step {
                    Pattern::Telegraph(n) | Pattern::Wait(n) => {
                        if *n > 0 {
                            self.wait_left = *n;
                            self.telegraph = matches!(step, Pattern::Telegraph(_));
                            break;
                        }
                    }
                    Pattern::Ring { every, .. } if *every > 0 => {
                        shots.extend(self.salvo(step, random));
                        self.rings.push(RingRepeat {
                            step: index,
                            next_frame: frame.saturating_add(*every),
                            fired: 1,
                        });
                    }
                    _ => shots.extend(self.salvo(step, random)),
                }
            }
            if self.next_step as usize >= steps.len() && self.wait_left == 0 && !self.infinite {
                self.finished = true;
                return shots;
            }
        }

        // Couronnes de fond (celles posées cette frame attendent `every`).
        let mut rings = std::mem::take(&mut self.rings);
        for ring in &mut rings {
            if frame >= ring.next_frame {
                if let Some(step) = steps.get(ring.step as usize) {
                    shots.extend(self.salvo(step, random));
                    if let Pattern::Ring { every, .. } = step {
                        ring.next_frame = ring.next_frame.saturating_add(*every);
                    }
                    ring.fired += 1;
                }
            }
        }
        self.rings = rings;
        shots
    }
}

/// Fait avancer chaque émetteur d'une frame, par `GgrsNetId`, et fait apparaître ses
/// projectiles au centre du tireur ; retire l'émetteur à la fin de sa séquence.
/// `RollbackSystemSet::Weapon`, après le tir des joueurs (voir `BaseWeaponGamePlugin`).
#[allow(clippy::type_complexity)]
pub fn emitter_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    mut rng_streams: ResMut<RngStreams>,
    settings: Res<CollisionSettings>,
    mut id_factory: ResMut<GgrsNetIdFactory>,
    mut emitters: Query<(
        &GgrsNetId,
        Entity,
        &fixed_math::FixedTransform3D,
        &Team,
        Option<&Tags>,
        &mut Emitter,
        // T1.3 : `Stun`/`Freeze` (§19) : l'émetteur est suspendu (aucun pas, aucun tir).
        Option<&crate::status::Statuses>,
    )>,
) {
    let system_span = span!(Level::INFO, "ggrs", f = frame.frame, s = "emitter");
    let _enter = system_span.enter();

    let no_tags = Tags::default();
    for (net_id, entity, transform, team, tags, mut emitter, statuses) in order_mut_iter!(emitters)
    {
        if crate::status::incapacitated(statuses) {
            continue;
        }
        // Le flux n'est créé (et consommé) que si un `Scatter` tire.
        let shots = emitter.tick(frame.frame, &mut || {
            rng_streams.get_mut(PATTERNS_RNG_STREAM).next_fixed()
        });
        if !shots.is_empty() {
            info!(
                "ggrs{{f={} emitter net_id={} step={} shots={}}}",
                frame.frame,
                net_id,
                emitter.next_step,
                shots.len()
            );
        }
        for shot in &shots {
            let Some(def) = emitter.table.get(&shot.projectile) else {
                warn!(
                    "emitter {} : projectile « {} » absent de la table de l'arme « {} » (voir `alacod lint`), ignoré",
                    net_id, shot.projectile, emitter.weapon
                );
                continue;
            };
            let projectile = Projectile::new(
                shot.projectile.clone(),
                &def.spec(),
                emitter.table.clone(),
                emitter.damage_mult,
                0,
            );
            let speed = shot.speed.unwrap_or(def.speed);
            let velocity = FixedVec2::new(
                shot.direction.x.saturating_mul(speed) / Fixed::from_num(60),
                shot.direction.y.saturating_mul(speed) / Fixed::from_num(60),
            );
            spawn_bullet(
                &mut commands,
                &settings,
                &mut id_factory,
                BulletSpawn {
                    position: transform.translation,
                    rotation: fixed_math::FixedMat3::IDENTITY,
                    velocity,
                    bullet_type: BulletType::Standard {
                        damage: def.damage,
                        speed,
                    },
                    damage: def.damage.saturating_mul(emitter.damage_mult),
                    range: def.range,
                    player_handle: NO_PLAYER_HANDLE,
                    created_at: frame.frame,
                    source: net_id,
                    source_team: *team,
                    source_tags: tags.unwrap_or(&no_tags),
                    friendly_fire: emitter.friendly_fire,
                    projectile: Some(projectile),
                    id_label: shot.projectile.clone(),
                },
            );
        }
        if emitter.finished {
            info!(
                "ggrs{{f={} emitter net_id={} finished}}",
                frame.frame, net_id
            );
            commands.entity(entity).remove::<Emitter>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::PatternLibrary;
    use bevy_fixed::rng::RngStreams;
    use std::collections::BTreeMap;

    fn table() -> Arc<ProjectileTable> {
        Arc::new(BTreeMap::new())
    }

    fn emitter(pattern: Pattern) -> Emitter {
        Emitter::new(
            "test",
            &pattern,
            "gun",
            table(),
            fixed_math::FIXED_ONE,
            FriendlyFire::Never,
            FixedVec2::new(fixed_math::FIXED_ONE, Fixed::ZERO),
            0,
        )
    }

    fn ring(count: u32, every: u32) -> Pattern {
        Pattern::Ring {
            count,
            speed: fixed_math::new(100.0),
            projectile: "p".into(),
            every,
        }
    }

    fn aimed(count: u32) -> Pattern {
        Pattern::Aimed {
            count,
            spread: fixed_math::new(0.3),
            projectile: "p".into(),
        }
    }

    fn scatter(count: u32) -> Pattern {
        Pattern::Scatter {
            count,
            spread: fixed_math::new(0.6),
            projectile: "p".into(),
        }
    }

    /// Joue `frames` frames (à partir de 1) et rend le nombre de tirs par frame.
    fn run(emitter: &mut Emitter, frames: u32) -> Vec<usize> {
        let mut never = || -> Fixed { panic!("aucun Scatter ici") };
        (1..=frames)
            .map(|f| emitter.tick(f, &mut never).len())
            .collect()
    }

    #[test]
    fn ring_every_seul_est_infini() {
        let mut e = emitter(ring(8, 60));
        let counts = run(&mut e, 181);
        let salvos: Vec<usize> = counts
            .iter()
            .enumerate()
            .filter(|(_, n)| **n > 0)
            .map(|(i, _)| i + 1)
            .collect();
        assert_eq!(salvos, vec![1, 61, 121, 181]);
        assert!(counts.iter().all(|n| *n == 0 || *n == 8));
        assert!(!e.finished);
    }

    /// Couronne dont le premier rayon part vers le haut (π/2) : le 8e rayon tombe à
    /// ~7,07 rad, au-delà de 2π (débordait le CORDIC avant le repli de `direction_of`).
    #[test]
    fn couronne_visant_vers_le_haut_ne_deborde_pas() {
        let mut e = Emitter::new(
            "test",
            &ring(8, 0),
            "gun",
            table(),
            fixed_math::FIXED_ONE,
            FriendlyFire::Never,
            FixedVec2::new(Fixed::ZERO, fixed_math::FIXED_ONE),
            0,
        );
        let mut never = || -> Fixed { panic!() };
        let shots = e.tick(1, &mut never);
        assert_eq!(shots.len(), 8);
        // Le 8e rayon (π/2 + 7π/4 = π/4 modulo 2π) pointe en haut à droite.
        let last = shots[7].direction;
        assert!(
            last.x > fixed_math::new(0.6) && last.y > fixed_math::new(0.6),
            "{last:?}"
        );
    }

    #[test]
    fn ring_sans_every_tire_une_fois_et_finit() {
        let mut e = emitter(ring(8, 0));
        assert_eq!(run(&mut e, 3), vec![8, 0, 0]);
        assert!(e.finished);
    }

    #[test]
    fn ring_every_dans_une_sequence_tourne_jusqu_a_la_fin() {
        // Couronne toutes les 20 frames pendant le Wait(60) qui suit : f1, f21, f41 ; la
        // séquence finit à f61 (fin du Wait), avant une 4e salve.
        let mut e = emitter(Pattern::Sequence(vec![ring(4, 20), Pattern::Wait(60)]));
        let counts = run(&mut e, 80);
        let salvos: Vec<usize> = counts
            .iter()
            .enumerate()
            .filter(|(_, n)| **n > 0)
            .map(|(i, _)| i + 1)
            .collect();
        assert_eq!(salvos, vec![1, 21, 41]);
        assert!(e.finished);
    }

    #[test]
    fn sequence_joue_les_etapes_instantanees_la_meme_frame() {
        let mut e = emitter(Pattern::Sequence(vec![
            aimed(3),
            Pattern::Sequence(vec![aimed(1), ring(2, 0)]),
        ]));
        assert_eq!(run(&mut e, 2), vec![6, 0]);
        assert!(e.finished);
    }

    #[test]
    fn telegraph_retarde_et_se_lit() {
        let mut e = emitter(Pattern::Sequence(vec![Pattern::Telegraph(30), aimed(3)]));
        let mut never = || -> Fixed { panic!() };
        assert!(e.tick(1, &mut never).is_empty());
        assert_eq!(e.telegraphing(), Some(30));
        for f in 2..=30 {
            assert!(e.tick(f, &mut never).is_empty(), "frame {f}");
            assert_eq!(e.telegraphing(), Some(31 - f));
        }
        assert_eq!(e.tick(31, &mut never).len(), 3);
        assert_eq!(e.telegraphing(), None);
        assert!(e.finished);
    }

    #[test]
    fn wait_ne_tire_pas_et_n_est_pas_un_telegraphe() {
        let mut e = emitter(Pattern::Sequence(vec![
            aimed(1),
            Pattern::Wait(20),
            aimed(2),
        ]));
        let mut never = || -> Fixed { panic!() };
        assert_eq!(e.tick(1, &mut never).len(), 1);
        assert_eq!(e.telegraphing(), None);
        for f in 2..=20 {
            assert!(e.tick(f, &mut never).is_empty(), "frame {f}");
        }
        assert_eq!(e.tick(21, &mut never).len(), 2);
        assert!(e.finished);
    }

    #[test]
    fn scatter_tire_dans_l_eventail_et_consomme_le_flux() {
        let mut streams = RngStreams::new(42);
        let mut e = emitter(scatter(4));
        let shots = e.tick(1, &mut || streams.get_mut(PATTERNS_RNG_STREAM).next_fixed());
        assert_eq!(shots.len(), 4);
        let half = fixed_math::new(0.3);
        for shot in &shots {
            let angle = angle_of(shot.direction);
            assert!(angle.abs() <= half + fixed_math::new(0.01), "angle {angle}");
        }
        // Pas toutes identiques : le tir est bien aléatoire.
        assert!(shots.iter().any(|s| s.direction != shots[0].direction));
    }

    /// Critère de la fiche : même graine = même tir, à 1 et à 4 joueurs. Le flux
    /// `"patterns"` n'est consommé que par les émetteurs : les autres flux (tirs des
    /// joueurs, `"weapons"`, plus nombreux à 4 joueurs) n'y changent rien.
    #[test]
    fn meme_graine_meme_tir_a_un_et_quatre_joueurs() {
        let seed = 123456;
        let fire = |players: usize| -> Vec<FixedVec2> {
            let mut streams = RngStreams::new(seed);
            // Chaque joueur consomme le flux des armes avant les émetteurs (comme
            // `weapon_rollback_system` avant `emitter_system` dans la frame).
            for _ in 0..players {
                streams.get_mut("weapons").next_fixed();
            }
            let mut e = emitter(Pattern::Sequence(vec![
                scatter(4),
                Pattern::Wait(5),
                scatter(4),
            ]));
            let mut directions = Vec::new();
            for f in 1..=10 {
                for _ in 0..players {
                    streams.get_mut("weapons").next_fixed();
                }
                let shots = e.tick(f, &mut || streams.get_mut(PATTERNS_RNG_STREAM).next_fixed());
                directions.extend(shots.into_iter().map(|s| s.direction));
            }
            directions
        };
        let solo = fire(1);
        assert_eq!(solo.len(), 8);
        assert_eq!(solo, fire(4));
    }

    #[test]
    fn named_se_resout_par_la_bibliotheque() {
        let mut library = PatternLibrary::default();
        library
            .patterns
            .insert("volee".into(), Arc::new(Pattern::Sequence(vec![aimed(3)])));
        let resolved = library
            .resolve(&Pattern::Sequence(vec![Pattern::Named("volee".into())]))
            .unwrap();
        assert_eq!(
            resolved,
            Pattern::Sequence(vec![Pattern::Sequence(vec![aimed(3)])])
        );
        assert_eq!(
            library.resolve(&Pattern::Named("absent".into())),
            Err("absent".into())
        );
        // Cycle : refusé (profondeur bornée), jamais de boucle infinie.
        library
            .patterns
            .insert("a".into(), Arc::new(Pattern::Named("a".into())));
        assert!(library.resolve(&Pattern::Named("a".into())).is_err());
    }
}
