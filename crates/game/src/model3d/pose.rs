//! Choix du clip d'un modèle d'après l'état de la simulation (R-1, `docs/conventions.md` § 111). Pur :
//! pas de monde, pas de rendu, testé seul. Les règles sont celles du banc (`armes.js`, `allures.js`) :
//! - priorité : transition de phase d'un boss, ordre d'un chef, décharge d'un pattern lié (X-6e), esquive, puis le geste
//!   en cours (coup, tir, mise en joue, en joue), puis le
//!   rechargement, puis l'allure (repos, marche, course) ;
//! - en marchant, un geste prend sa variante dans la direction de la marche par rapport au corps
//!   (cuite dans les huit directions) ; une allure prend la plus proche, ou la direction opposée jouée
//!   à l'envers (quatre directions cuites suffisent, les pieds ne glissent pas) ;
//! - une allure est lue à la vitesse du déplacement sur celle du clip (`moments.<clip>.vitesse`),
//!   bornée à [0,3 ; 3].

use content::models::{AnimeManifest, Gesture, ModelDef, Poses};

/// Ce que la présentation lit de la simulation pour un personnage, à l'instant affiché.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PoseInput {
    /// Vitesse au sol (m/s).
    pub speed: f32,
    /// Direction du déplacement moins le cap du corps (radians, + vers sa gauche).
    pub rel: f32,
    /// Esquive en cours : (rang de l'esquive, 0 pour la première ; avancement dans [0, 1]).
    pub dash: Option<(u32, f32)>,
    /// Coup de mêlée en cours : secondes depuis son début, durée du coup (s).
    pub melee: Option<(f32, f32)>,
    /// Secondes depuis le dernier tir de l'arme active (`None` : aucun tir).
    pub since_shot: Option<f32>,
    /// Détente tenue.
    pub trigger: bool,
    /// Secondes depuis que la détente est tenue sans interruption (début de la mise en joue).
    pub trigger_for: f32,
    /// Rechargement en cours : avancement dans [0, 1], durée totale (s).
    pub reload: Option<(f32, f32)>,
    /// Coup reçu en cours (tireur posté) : avancement dans [0, 1] de la stupeur.
    pub hit: Option<f32>,
    /// X-6e : geste de transition d'un boss en cours : avancement dans [0, 1] de `Phase::transition`.
    pub transition: Option<f32>,
    /// X-6e : ordre d'un chef en cours : avancement dans [0, 1] du télégraphe de la règle `Command`.
    pub order: Option<f32>,
    /// X-6e : décharge d'un tireur dont le pattern est lié à un clip (`ModelDef::pattern_clips`) : le clip et les secondes
    /// écoulées depuis le premier pas de l'émetteur ; le clip joue une fois, sa dernière image est tenue.
    pub discharge: Option<(String, f32)>,
    /// Phase de behavior en cours (C-2, § 131) : annonce ou ruée d'une charge, recul de `ReturnHome`, repli de `TakeCover`.
    pub behavior: Option<BehaviorPhase>,
    /// Fondus de ce personnage (ceux du joueur par défaut).
    pub fades: Fades,
}

/// Phase d'un behavior à état de l'ennemi (`BehaviorRuntime`), lue sur la simulation (C-2, § 131) : chacune a un champ
/// optionnel de [`Poses`] ; sans lui la pose ordinaire joue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviorPhase {
    /// `Charge` : annonce (`ChargePhase::Telegraph`), `poses.charge_windup`.
    ChargeWindup,
    /// `Charge` : ruée (`ChargePhase::Rush`), `poses.charge`.
    Charge,
    /// `ReturnHome` : recul (`ReturnPhase::Retreat`), `poses.return_home`.
    ReturnHome,
    /// `TakeCover` : trajet ou attente à l'abri (`CoverPhase`), `poses.take_cover`.
    TakeCover,
}

impl BehaviorPhase {
    /// Le geste de `poses` de cette phase.
    pub fn gesture(self, poses: &Poses) -> &Option<Gesture> {
        match self {
            Self::ChargeWindup => &poses.charge_windup,
            Self::Charge => &poses.charge,
            Self::ReturnHome => &poses.return_home,
            Self::TakeCover => &poses.take_cover,
        }
    }

    /// Nom de l'état pour l'invariant `animation_vivante`.
    pub fn label(self) -> &'static str {
        match self {
            Self::ChargeWindup => "annonce de charge",
            Self::Charge => "ruée",
            Self::ReturnHome => "recul au point de départ",
            Self::TakeCover => "repli à couvert",
        }
    }
}

/// La pose voulue : un clip et comment le lire.
#[derive(Debug, Clone, PartialEq)]
pub struct PoseChoice {
    pub clip: String,
    pub timing: Timing,
    /// Arme du manifeste montrée dans la main.
    pub object: Option<String>,
    /// Durée du fondu vers ce clip (s).
    pub fade: f32,
}

/// Lecture d'un clip.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Timing {
    /// Boucle à cette vitesse de lecture (négative : à l'envers). Un changement d'allure reprend à la
    /// même phase.
    Loop(f32),
    /// Position imposée, en fraction de la durée du clip (geste lu d'après l'état de la simulation).
    At(f32),
}

/// Fondus (s) vers un clip, selon ce qu'il est. Le défaut est celui du joueur (le Frère, validé par William :
/// inchangé) ; [`Fades::SHOOTER`] est celui des fantassins du banc (`armes.js` A:686-735 : `epauler` 0,08,
/// `viser` 0,08, `tir` 0,04, `rechargement` 0,1, `touche` 0,04, attente 0,15, marche du retour 0,12).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fades {
    pub gait: f32,
    pub idle: f32,
    pub dash: f32,
    pub melee: f32,
    pub shot: f32,
    pub shoulder: f32,
    pub aim: f32,
    pub reload: f32,
    pub hit: f32,
}

/// Fondus du banc (`armes.js`) du joueur : vers une allure, vers un geste.
pub const FADE_GAIT: f32 = 0.1;
pub const FADE_GESTURE: f32 = 0.05;

impl Default for Fades {
    fn default() -> Self {
        Self {
            gait: FADE_GAIT,
            idle: FADE_GAIT,
            dash: FADE_GESTURE,
            melee: FADE_GESTURE,
            shot: FADE_GESTURE,
            shoulder: FADE_GESTURE,
            aim: FADE_GAIT,
            reload: FADE_GESTURE,
            hit: FADE_GESTURE,
        }
    }
}

impl Fades {
    /// Les fondus du fantassin du banc.
    pub const SHOOTER: Fades = Fades {
        gait: 0.12,
        idle: 0.15,
        dash: FADE_GESTURE,
        melee: FADE_GESTURE,
        shot: 0.04,
        shoulder: 0.08,
        aim: 0.08,
        reload: 0.1,
        hit: 0.04,
    };
}

/// En dessous de cette vitesse (m/s), le personnage est à l'arrêt.
pub const STILL_BELOW: f32 = 0.2;

/// La pose d'un modèle pour `input`, ses `poses` (celles de l'arme active déjà fusionnées) et son
/// manifeste. `None` : aucun clip déclaré ne convient (le modèle garde sa pose).
pub fn choose(
    model: &ModelDef,
    poses: &Poses,
    manifest: &AnimeManifest,
    input: &PoseInput,
) -> Option<PoseChoice> {
    let moving = input.speed >= STILL_BELOW;
    let fades = input.fades;
    let gesture = |g: &Option<Gesture>, u: f32, fade: f32| -> Option<PoseChoice> {
        let g = g.as_ref()?;
        let clip = moving
            .then(|| g.moving.as_deref())
            .flatten()
            .and_then(|base| variant(manifest, base, input.rel, false).map(|(c, _)| c))
            .or_else(|| g.still.clone())?;
        Some(PoseChoice {
            clip,
            timing: Timing::At(u.clamp(0.0, 1.0)),
            object: g.object.clone().or_else(|| poses.object.clone()),
            fade,
        })
    };
    let duration = |g: &Option<Gesture>| -> Option<f32> {
        let g = g.as_ref()?;
        g.duration_s
            .or_else(|| g.still.as_deref().and_then(|c| manifest.duration(c)))
    };

    // X-6e : la transition d'un boss, puis l'ordre d'un chef, puis la décharge d'un tireur à pattern lié passent avant
    // tout le reste (l'ennemi est immobile, aucun autre état ne se lit pendant eux). Fondu du geste d'épaulé.
    if let Some(u) = input.transition {
        if let Some(choice) = gesture(&poses.transition, u, fades.shoulder) {
            return Some(choice);
        }
    }
    if let Some(u) = input.order {
        if let Some(choice) = gesture(&poses.order, u, fades.shoulder) {
            return Some(choice);
        }
    }
    if let Some((clip, elapsed)) = &input.discharge {
        if let Some(total) = manifest.duration(clip) {
            return Some(PoseChoice {
                clip: clip.clone(),
                timing: Timing::At((elapsed / total).clamp(0.0, 1.0)),
                object: poses
                    .shoot
                    .as_ref()
                    .and_then(|g| g.object.clone())
                    .or_else(|| poses.object.clone()),
                fade: fades.shot,
            });
        }
    }
    if let Some((n, u)) = input.dash {
        if !model.dash.is_empty() {
            let clip = model.dash[n as usize % model.dash.len()].clone();
            return Some(PoseChoice {
                clip,
                timing: Timing::At(u.clamp(0.0, 1.0)),
                object: poses.object.clone(),
                fade: fades.dash,
            });
        }
    }
    if let Some(u) = input.hit {
        if let Some(choice) = gesture(&poses.hit, u, fades.hit) {
            return Some(choice);
        }
    }
    if let Some((elapsed, total)) = input.melee {
        let total = duration(&poses.melee).unwrap_or(total).max(1e-3);
        if let Some(choice) = gesture(&poses.melee, elapsed / total, fades.melee) {
            return Some(choice);
        }
    }
    if let (Some(since), Some(total)) = (input.since_shot, duration(&poses.shoot)) {
        if since < total {
            if let Some(choice) = gesture(&poses.shoot, since / total, fades.shot) {
                return Some(choice);
            }
        }
    }
    // C-2 (§ 131) : la phase d'un behavior (annonce, ruée, recul, repli) joue son clip quand le modèle l'a posé ; sinon la pose
    // ordinaire (rien ne change pour un modèle qui ne la déclare pas).
    if let Some(phase) = input.behavior {
        if let Some(choice) = looping_gesture(
            manifest,
            poses,
            phase.gesture(poses),
            input,
            moving,
            fades.gait,
        ) {
            return Some(choice);
        }
    }
    if input.trigger {
        if let Some(total) = duration(&poses.shoulder) {
            if input.trigger_for < total {
                if let Some(choice) =
                    gesture(&poses.shoulder, input.trigger_for / total, fades.shoulder)
                {
                    return Some(choice);
                }
            }
        }
        if let Some(choice) = looping_gesture(manifest, poses, &poses.aim, input, moving, fades.aim)
        {
            return Some(choice);
        }
    }
    if let Some((u, _total)) = input.reload {
        if moving {
            if let Some(choice) =
                looping_gesture(manifest, poses, &poses.reload, input, moving, fades.gait)
            {
                return Some(choice);
            }
        }
        if let Some(choice) = gesture(&poses.reload, u, fades.reload) {
            return Some(choice);
        }
    }
    gait(model, poses, manifest, input)
}

/// Un geste qui dure tant que l'état dure (en joue, recharger en marchant) : à l'arrêt, le clip seul
/// en boucle ; en marchant, la variante de la direction lue à la vitesse de la marche.
fn looping_gesture(
    manifest: &AnimeManifest,
    poses: &Poses,
    g: &Option<Gesture>,
    input: &PoseInput,
    moving: bool,
    fade: f32,
) -> Option<PoseChoice> {
    let g = g.as_ref()?;
    let object = g.object.clone().or_else(|| poses.object.clone());
    if moving {
        if let Some(base) = &g.moving {
            if let Some((clip, reversed)) = variant(manifest, base, input.rel, true) {
                return Some(PoseChoice {
                    timing: Timing::Loop(playback(manifest, &clip, input.speed, reversed)),
                    clip,
                    object,
                    fade,
                });
            }
        }
    }
    Some(PoseChoice {
        clip: g.still.clone()?,
        timing: Timing::Loop(1.0),
        object,
        fade,
    })
}

fn gait(
    model: &ModelDef,
    poses: &Poses,
    manifest: &AnimeManifest,
    input: &PoseInput,
) -> Option<PoseChoice> {
    let moving = input.speed >= STILL_BELOW;
    let base = if !moving {
        None
    } else if model
        .run_above_m_s
        .is_some_and(|threshold| input.speed >= threshold)
        && poses.run.is_some()
    {
        poses.run.as_deref()
    } else {
        poses.walk.as_deref().or(poses.run.as_deref())
    };
    let (clip, timing, fade) = match base.and_then(|b| variant(manifest, b, input.rel, true)) {
        Some((clip, reversed)) => {
            let speed = playback(manifest, &clip, input.speed, reversed);
            (clip, Timing::Loop(speed), input.fades.gait)
        }
        None => (poses.idle.clone()?, Timing::Loop(1.0), input.fades.idle),
    };
    Some(PoseChoice {
        clip,
        timing,
        object: poses.object.clone(),
        fade,
    })
}

/// La variante de `base` la plus proche de la direction `rel` (radians, + vers la gauche) : `(clip,
/// à l'envers)`. Une allure (`reversible`) peut prendre la direction opposée jouée à l'envers ; à égalité
/// le sens avant l'emporte. Sans variante dirigée, `base` lui-même s'il existe.
pub fn variant(
    manifest: &AnimeManifest,
    base: &str,
    rel: f32,
    reversible: bool,
) -> Option<(String, bool)> {
    let wanted = rel.to_degrees();
    let mut best: Option<(&str, bool, f32)> = None;
    for (clip, dir) in manifest.directional(base) {
        let mut candidates = vec![(false, angle_between(wanted, dir))];
        if reversible {
            candidates.push((true, angle_between(wanted, dir + 180.0)));
        }
        for (reversed, gap) in candidates {
            let better = match best {
                None => true,
                Some((_, best_rev, best_gap)) => {
                    gap < best_gap - 1e-3
                        || ((gap - best_gap).abs() <= 1e-3 && best_rev && !reversed)
                }
            };
            if better {
                best = Some((clip, reversed, gap));
            }
        }
    }
    match best {
        Some((clip, reversed, _)) => Some((clip.to_string(), reversed)),
        None => manifest
            .clips
            .contains_key(base)
            .then(|| (base.to_string(), false)),
    }
}

/// Vitesse de lecture d'une allure : celle du déplacement sur celle du clip, bornée à [0,3 ; 3], et
/// négative à l'envers. Sans vitesse déclarée, 1.
pub fn playback(manifest: &AnimeManifest, clip: &str, speed: f32, reversed: bool) -> f32 {
    let rate = manifest
        .moments
        .get(clip)
        .and_then(|m| m.speed)
        .filter(|v| *v > 0.0)
        .map_or(1.0, |v| (speed / v).clamp(0.3, 3.0));
    if reversed {
        -rate
    } else {
        rate
    }
}

/// Écart absolu entre deux angles (degrés), dans [0, 180].
fn angle_between(a: f32, b: f32) -> f32 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    /// Les allures cuites dans quatre directions (avant, avant-gauche, gauche, arrière-gauche) et un
    /// geste cuit dans les huit, comme les modèles de la pipeline.
    fn manifest() -> AnimeManifest {
        let mut clips = String::new();
        let mut moments = String::new();
        for (name, dir) in [
            ("walk", 0),
            ("walk-fl", 45),
            ("walk-l", 90),
            ("walk-bl", 135),
            ("fire-walk", 0),
            ("fire-walk-l", 90),
            ("fire-walk-b", 180),
            ("fire-walk-r", -90),
        ] {
            clips.push_str(&format!("\"{name}\": [1, 9],"));
            moments.push_str(&format!(
                "\"{name}\": {{\"vitesse\": 3.0, \"direction\": {dir}}},"
            ));
        }
        serde_json::from_str(&format!(
            r#"{{"fps": 24, "actions": {{{clips} "idle": [1, 49], "fire": [1, 8], "reload": [1, 48], "roll": [1, 13]}},
                "moments": {{{}}}}}"#,
            moments.trim_end_matches(',')
        ))
        .unwrap()
    }

    fn model() -> ModelDef {
        ron::from_str(
            r#"(glb: "m.glb", manifest: "m.json", frame_m: (3.0, 3.0), pivot: (0.5, 0.2), height_m: 1.8,
                run_above_m_s: Some(3.6),
                poses: (idle: Some("idle"), walk: Some("walk"),
                    shoot: Some((still: Some("fire"), moving: Some("fire-walk"), duration_s: Some(0.3))),
                    reload: Some((still: Some("reload"), moving: Some("walk")))),
                dash: ["roll"])"#,
        )
        .unwrap()
    }

    fn pick(input: PoseInput) -> PoseChoice {
        let model = model();
        choose(&model, &model.poses, &manifest(), &input).unwrap()
    }

    #[test]
    fn gaits_take_the_nearest_baked_direction_or_its_opposite_backwards() {
        let m = manifest();
        let deg = |d: f32| d.to_radians();
        assert_eq!(variant(&m, "walk", 0.0, true), Some(("walk".into(), false)));
        assert_eq!(
            variant(&m, "walk", deg(80.0), true),
            Some(("walk-l".into(), false))
        );
        // Vers la droite : la marche vers la gauche à l'envers ; à reculons : l'avant à l'envers.
        assert_eq!(
            variant(&m, "walk", deg(-90.0), true),
            Some(("walk-l".into(), true))
        );
        assert_eq!(
            variant(&m, "walk", deg(180.0), true),
            Some(("walk".into(), true))
        );
        assert_eq!(
            variant(&m, "walk", deg(-135.0), true),
            Some(("walk-fl".into(), true))
        );
        // Un geste ne se lit jamais à l'envers : la direction cuite la plus proche.
        assert_eq!(
            variant(&m, "fire-walk", deg(-100.0), false),
            Some(("fire-walk-r".into(), false))
        );
        assert_eq!(
            variant(&m, "fire", 0.3, false),
            Some(("fire".into(), false))
        );
        assert_eq!(variant(&m, "nothing", 0.3, false), None);
    }

    #[test]
    fn a_gait_plays_at_the_ground_speed_over_the_clip_speed() {
        let walking = pick(PoseInput {
            speed: 1.5,
            ..default()
        });
        assert_eq!(walking.clip, "walk");
        assert_eq!(walking.timing, Timing::Loop(0.5));
        let backwards = pick(PoseInput {
            speed: 30.0,
            rel: std::f32::consts::PI,
            ..default()
        });
        assert_eq!(backwards.timing, Timing::Loop(-3.0));
        assert_eq!(
            pick(PoseInput {
                speed: 0.1,
                ..default()
            })
            .clip,
            "idle"
        );
    }

    #[test]
    fn the_dash_wins_then_the_shot_then_the_reload() {
        let all = PoseInput {
            speed: 3.0,
            rel: FRAC_PI_2,
            dash: Some((3, 0.5)),
            since_shot: Some(0.15),
            reload: Some((0.25, 1.2)),
            ..default()
        };
        assert_eq!(pick(all.clone()).clip, "roll");
        assert_eq!(pick(all.clone()).timing, Timing::At(0.5));
        let shot = pick(PoseInput {
            dash: None,
            ..all.clone()
        });
        assert_eq!(
            (shot.clip.as_str(), shot.timing),
            ("fire-walk-l", Timing::At(0.5))
        );
        // Tir fini : le rechargement en marchant boucle à la vitesse de la marche.
        let reload = pick(PoseInput {
            dash: None,
            since_shot: Some(0.4),
            ..all.clone()
        });
        assert_eq!(
            (reload.clip.as_str(), reload.timing),
            ("walk-l", Timing::Loop(1.0))
        );
        // À l'arrêt, le rechargement suit son avancement.
        let still = pick(PoseInput {
            speed: 0.0,
            dash: None,
            since_shot: None,
            ..all
        });
        assert_eq!(
            (still.clip.as_str(), still.timing),
            ("reload", Timing::At(0.25))
        );
    }

    /// Modèle d'un tireur posté : épauler, en joue, tir, rechargement, coup reçu (P-1d).
    fn shooter_model() -> ModelDef {
        ron::from_str(
            r#"(glb: "m.glb", manifest: "m.json", frame_m: (3.0, 3.0), pivot: (0.5, 0.2), height_m: 1.8,
                poses: (idle: Some("idle"), walk: Some("walk"),
                    shoulder: Some((still: Some("fire"))), aim: Some((still: Some("roll"))),
                    shoot: Some((still: Some("fire"))), reload: Some((still: Some("reload"))),
                    hit: Some((still: Some("roll"))))
                )"#,
        )
        .unwrap()
    }

    /// P-1d (banc `armes.js` A:686-735) : le cycle d'un fantassin joue chaque geste avec le fondu du banc ;
    /// le rechargement joue le clip **entier une fois**, étiré sur toute sa durée (A:695-696).
    #[test]
    fn a_shooter_plays_every_gesture_of_the_cycle_with_the_bench_fades() {
        let model = shooter_model();
        let at = |input: PoseInput| {
            choose(
                &model,
                &model.poses,
                &manifest(),
                &PoseInput {
                    fades: Fades::SHOOTER,
                    ..input
                },
            )
            .unwrap()
        };
        let idle = at(default());
        assert_eq!((idle.clip.as_str(), idle.fade), ("idle", 0.15));
        let raise = at(PoseInput {
            trigger: true,
            trigger_for: 0.1,
            ..default()
        });
        assert_eq!((raise.clip.as_str(), raise.fade), ("fire", 0.08));
        assert!(
            matches!(raise.timing, Timing::At(u) if (u - 0.1 / manifest().duration("fire").unwrap()).abs() < 1e-5)
        );
        let aim = at(PoseInput {
            trigger: true,
            trigger_for: 1.0,
            ..default()
        });
        assert_eq!((aim.clip.as_str(), aim.fade), ("roll", 0.08));
        assert_eq!(aim.timing, Timing::Loop(1.0));
        let shot = at(PoseInput {
            since_shot: Some(0.1),
            ..default()
        });
        assert_eq!((shot.clip.as_str(), shot.fade), ("fire", 0.04));
        for done in [0.0, 0.5, 0.99] {
            let reload = at(PoseInput {
                reload: Some((done, 3.0)),
                ..default()
            });
            assert_eq!(reload.clip, "reload");
            assert_eq!(reload.timing, Timing::At(done), "avancement {done}");
            assert_eq!(reload.fade, 0.1);
        }
        // Un coup reçu passe avant tout geste en cours.
        let hit = at(PoseInput {
            hit: Some(0.5),
            trigger: true,
            reload: Some((0.5, 3.0)),
            ..default()
        });
        assert_eq!(
            (hit.clip.as_str(), hit.timing, hit.fade),
            ("roll", Timing::At(0.5), 0.04)
        );
    }

    /// Le Frère est validé tel quel (William, 16 nov) : sans `fades`, les fondus restent ceux d'avant P-1d.
    #[test]
    fn the_default_fades_are_the_players_unchanged() {
        assert_eq!(pick(default()).fade, FADE_GAIT);
        let shot = pick(PoseInput {
            since_shot: Some(0.15),
            ..default()
        });
        assert_eq!(shot.fade, FADE_GESTURE);
        assert_eq!(
            pick(PoseInput {
                dash: Some((0, 0.5)),
                ..default()
            })
            .fade,
            FADE_GESTURE
        );
    }

    /// Modèle d'un chef de bande (X-6e) : un ordre, une transition, et le geste de tir d'une arme.
    fn chief_model() -> ModelDef {
        ron::from_str(
            r#"(glb: "m.glb", manifest: "m.json", frame_m: (3.0, 3.0), pivot: (0.5, 0.2), height_m: 1.8,
                poses: (object: Some("gun"), idle: Some("idle"), walk: Some("walk"),
                    order: Some((still: Some("fire"))), transition: Some((still: Some("roll"))),
                    hit: Some((still: Some("roll"))))
                )"#,
        )
        .unwrap()
    }

    /// X-6e : la transition d'un boss passe avant l'ordre, l'ordre avant la décharge, la décharge avant le reste ; chaque
    /// geste joue une fois, à l'avancement que la simulation donne.
    #[test]
    fn a_transition_wins_over_an_order_over_a_discharge_over_the_rest() {
        let model = chief_model();
        let at = |input: PoseInput| {
            choose(&model, &model.poses, &manifest(), &input).unwrap()
        };
        let all = PoseInput {
            transition: Some(0.25),
            order: Some(0.5),
            discharge: Some(("reload".into(), 0.4)),
            dash: Some((0, 0.5)),
            since_shot: Some(0.1),
            ..default()
        };
        let transition = at(all.clone());
        assert_eq!(
            (transition.clip.as_str(), transition.timing),
            ("roll", Timing::At(0.25))
        );
        let order = at(PoseInput {
            transition: None,
            ..all.clone()
        });
        assert_eq!((order.clip.as_str(), order.timing), ("fire", Timing::At(0.5)));
        assert_eq!(order.object.as_deref(), Some("gun"));
        // `reload` dure 48/24 = 2 s : 0,4 s = 20 % du clip, avant l'esquive et le tir.
        let discharge = at(PoseInput {
            transition: None,
            order: None,
            ..all.clone()
        });
        assert_eq!(discharge.clip, "reload");
        assert!(matches!(discharge.timing, Timing::At(u) if (u - 0.2).abs() < 1e-5));
        assert_eq!(discharge.fade, FADE_GESTURE);
        let rest = at(PoseInput {
            transition: None,
            order: None,
            discharge: None,
            ..all
        });
        // Sans esquive déclarée ni geste de tir, c'est l'attente : aucun des états ci-dessus ne fuit.
        assert_eq!(rest.clip, "idle");
    }

    /// X-6e : la décharge joue son clip une fois et tient sa dernière image ; un clip absent du manifeste ne la joue pas.
    #[test]
    fn a_discharge_holds_its_last_frame_and_ignores_an_unknown_clip() {
        let model = chief_model();
        let m = manifest();
        let pick = |discharge: (&str, f32)| {
            choose(
                &model,
                &model.poses,
                &m,
                &PoseInput {
                    discharge: Some((discharge.0.to_string(), discharge.1)),
                    ..default()
                },
            )
            .unwrap()
        };
        assert_eq!(pick(("reload", 10.0)).timing, Timing::At(1.0));
        assert_eq!(pick(("reload", 0.0)).timing, Timing::At(0.0));
        assert_eq!(pick(("absent", 0.5)).clip, "idle");
    }

    /// C-2 (§ 131) : une phase de behavior joue le clip que le modèle lui a posé (la variante de direction en marchant, le clip
    /// seul à l'arrêt) ; un champ absent garde la pose ordinaire, comme avant.
    #[test]
    fn a_behavior_phase_plays_its_pose_or_leaves_the_ordinary_one() {
        let model: ModelDef = ron::from_str(
            r#"(glb: "m.glb", manifest: "m.json", frame_m: (3.0, 3.0), pivot: (0.5, 0.2), height_m: 1.8,
                poses: (idle: Some("idle"), walk: Some("walk"),
                    charge_windup: Some((still: Some("idle-wind"))),
                    take_cover: Some((still: Some("reload"), moving: Some("walk")))))"#,
        )
        .unwrap();
        let play = |input: PoseInput| choose(&model, &model.poses, &manifest(), &input).unwrap();
        let windup = play(PoseInput {
            behavior: Some(BehaviorPhase::ChargeWindup),
            ..default()
        });
        assert_eq!(windup.clip, "idle-wind");
        assert_eq!(windup.timing, Timing::Loop(1.0));
        // En route vers l'abri : la variante de direction de `walk`, vers la gauche du corps.
        let trip = play(PoseInput {
            behavior: Some(BehaviorPhase::TakeCover),
            speed: 3.0,
            rel: FRAC_PI_2,
            ..default()
        });
        assert_eq!(trip.clip, "walk-l");
        // Champ absent (`charge`, `return_home`) : l'attente ou l'allure d'avant.
        for phase in [BehaviorPhase::Charge, BehaviorPhase::ReturnHome] {
            assert_eq!(
                play(PoseInput {
                    behavior: Some(phase),
                    ..default()
                })
                .clip,
                "idle"
            );
        }
    }

    fn default() -> PoseInput {
        PoseInput::default()
    }
}
