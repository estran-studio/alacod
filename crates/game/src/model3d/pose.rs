//! Choix du clip d'un modèle d'après l'état de la simulation (R-1, `docs/conventions.md` § 111). Pur :
//! pas de monde, pas de rendu, testé seul. Les règles sont celles du banc (`armes.js`, `allures.js`) :
//! - priorité : esquive, puis le geste en cours (coup, tir, mise en joue, en joue), puis le
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
    /// Esquive en cours : (numéro de l'esquive, avancement dans [0, 1]).
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

/// Fondus du banc (`armes.js`) : vers une allure, vers un geste.
pub const FADE_GAIT: f32 = 0.1;
pub const FADE_GESTURE: f32 = 0.05;
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
    let gesture = |g: &Option<Gesture>, u: f32| -> Option<PoseChoice> {
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
            fade: FADE_GESTURE,
        })
    };
    let duration = |g: &Option<Gesture>| -> Option<f32> {
        let g = g.as_ref()?;
        g.duration_s
            .or_else(|| g.still.as_deref().and_then(|c| manifest.duration(c)))
    };

    if let Some((n, u)) = input.dash {
        if !model.dash.is_empty() {
            let clip = model.dash[n as usize % model.dash.len()].clone();
            return Some(PoseChoice {
                clip,
                timing: Timing::At(u.clamp(0.0, 1.0)),
                object: poses.object.clone(),
                fade: FADE_GESTURE,
            });
        }
    }
    if let Some((elapsed, total)) = input.melee {
        let total = duration(&poses.melee).unwrap_or(total).max(1e-3);
        if let Some(choice) = gesture(&poses.melee, elapsed / total) {
            return Some(choice);
        }
    }
    if let (Some(since), Some(total)) = (input.since_shot, duration(&poses.shoot)) {
        if since < total {
            if let Some(choice) = gesture(&poses.shoot, since / total) {
                return Some(choice);
            }
        }
    }
    if input.trigger {
        if let Some(total) = duration(&poses.shoulder) {
            if input.trigger_for < total {
                if let Some(choice) = gesture(&poses.shoulder, input.trigger_for / total) {
                    return Some(choice);
                }
            }
        }
        if let Some(choice) = looping_gesture(manifest, poses, &poses.aim, input, moving) {
            return Some(choice);
        }
    }
    if let Some((u, _total)) = input.reload {
        if moving {
            if let Some(choice) = looping_gesture(manifest, poses, &poses.reload, input, moving) {
                return Some(choice);
            }
        }
        if let Some(choice) = gesture(&poses.reload, u) {
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
                    fade: FADE_GAIT,
                });
            }
        }
    }
    Some(PoseChoice {
        clip: g.still.clone()?,
        timing: Timing::Loop(1.0),
        object,
        fade: FADE_GAIT,
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
    let (clip, timing) = match base.and_then(|b| variant(manifest, b, input.rel, true)) {
        Some((clip, reversed)) => {
            let speed = playback(manifest, &clip, input.speed, reversed);
            (clip, Timing::Loop(speed))
        }
        None => (poses.idle.clone()?, Timing::Loop(1.0)),
    };
    Some(PoseChoice {
        clip,
        timing,
        object: poses.object.clone(),
        fade: FADE_GAIT,
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
                    gap < best_gap - 1e-3 || ((gap - best_gap).abs() <= 1e-3 && best_rev && !reversed)
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
            moments.push_str(&format!("\"{name}\": {{\"vitesse\": 3.0, \"direction\": {dir}}},"));
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
        assert_eq!(variant(&m, "walk", deg(80.0), true), Some(("walk-l".into(), false)));
        // Vers la droite : la marche vers la gauche à l'envers ; à reculons : l'avant à l'envers.
        assert_eq!(variant(&m, "walk", deg(-90.0), true), Some(("walk-l".into(), true)));
        assert_eq!(variant(&m, "walk", deg(180.0), true), Some(("walk".into(), true)));
        assert_eq!(variant(&m, "walk", deg(-135.0), true), Some(("walk-fl".into(), true)));
        // Un geste ne se lit jamais à l'envers : la direction cuite la plus proche.
        assert_eq!(variant(&m, "fire-walk", deg(-100.0), false), Some(("fire-walk-r".into(), false)));
        assert_eq!(variant(&m, "fire", 0.3, false), Some(("fire".into(), false)));
        assert_eq!(variant(&m, "nothing", 0.3, false), None);
    }

    #[test]
    fn a_gait_plays_at_the_ground_speed_over_the_clip_speed() {
        let walking = pick(PoseInput { speed: 1.5, ..default() });
        assert_eq!(walking.clip, "walk");
        assert_eq!(walking.timing, Timing::Loop(0.5));
        let backwards = pick(PoseInput { speed: 30.0, rel: std::f32::consts::PI, ..default() });
        assert_eq!(backwards.timing, Timing::Loop(-3.0));
        assert_eq!(pick(PoseInput { speed: 0.1, ..default() }).clip, "idle");
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
        let shot = pick(PoseInput { dash: None, ..all.clone() });
        assert_eq!((shot.clip.as_str(), shot.timing), ("fire-walk-l", Timing::At(0.5)));
        // Tir fini : le rechargement en marchant boucle à la vitesse de la marche.
        let reload = pick(PoseInput { dash: None, since_shot: Some(0.4), ..all.clone() });
        assert_eq!((reload.clip.as_str(), reload.timing), ("walk-l", Timing::Loop(1.0)));
        // À l'arrêt, le rechargement suit son avancement.
        let still = pick(PoseInput { speed: 0.0, dash: None, since_shot: None, ..all });
        assert_eq!((still.clip.as_str(), still.timing), ("reload", Timing::At(0.25)));
    }

    fn default() -> PoseInput {
        PoseInput::default()
    }
}
