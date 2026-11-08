//! Modèles 3D animés rendus « façon sprite » (R-1, `docs/conventions.md` § 111) : kind de contenu
//! `Model`, un fichier par jeu (`models/models.ron`).
//!
//! Un personnage dont l'`asset_name_ref` a une entrée dans la table `models` est dessiné par son
//! modèle (`anime.glb`, squelette et clips) au lieu de sa planche de sprites ; le reste (décor,
//! props, effets) garde ses sprites. Le manifeste `anime.json` est celui de l'étape `banc` de la
//! pipeline d'assets (actions, armes et leurs objets, prises, moments) : il est lu ici pour le lint
//! (clips, objets et repères nommés par le RON) et par la présentation (`game::model3d`), qui ne
//! relit aucun fichier. Tout est de la présentation : rien de ce fichier n'entre dans la simulation.
//!
//! Les valeurs (échelle, élévation, rampe, contour, ombre, clips) sont celles du jeu : l'engine n'en
//! a aucune par défaut.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Racine de `models.ron`.
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ModelsFile {
    /// Unités du monde par mètre de largeur (échelle du jeu, la même que celle de ses planches).
    #[schemars(extend("x-lint" = "> 0"))]
    pub units_per_meter: f32,
    /// Élévation de la caméra, en degrés sous l'horizontale, sans lacet. Le sol du monde 2D est
    /// calé dessus : 1 m de profondeur fait `sin(élévation)` m de largeur à l'écran.
    #[schemars(extend("x-lint" = "]0, 90["))]
    pub elevation_deg: f32,
    /// Tri en y : tous les personnages (sprites et modèles) et les balles se dessinent dans l'ordre
    /// de leur pied, le plus bas à l'écran par-dessus. Sans lui, l'ordre est celui du `z` de la
    /// simulation (comportement des jeux qui ne déclarent pas de modèle).
    #[serde(default)]
    pub y_sort: bool,
    /// Le rendu « façon sprite » : rampe de lumière, contour, ombre.
    pub look: ModelLook,
    /// Modèle par `asset_name_ref` de personnage.
    pub models: BTreeMap<String, ModelDef>,
}

/// Rendu « façon sprite » : couleur de la texture (lue en sRGB) × une rampe continue de `shade`
/// (à l'ombre) à `highlight` (au reflet) selon `0,5 + 0,5 × N·L`, normales lissées et tournées vers
/// la caméra ; contour d'un pixel de sprite hors de la silhouette ; ombre portée sur le sol.
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ModelLook {
    /// Multiplicateur de la couleur à l'ombre (`N·L = -1`).
    #[schemars(extend("x-lint" = ">= 0"))]
    pub shade: f32,
    /// Multiplicateur de la couleur au reflet (`N·L = 1`).
    #[schemars(extend("x-lint" = ">= 0"))]
    pub highlight: f32,
    /// Direction de la lumière, fixe dans le repère de la caméra (x vers la droite de l'écran,
    /// y vers le haut, z vers le spectateur) ; normalisée au chargement.
    pub light: (f32, f32, f32),
    /// Teinte du contour : moyenne des pixels voisins de la silhouette × `outline`.
    #[schemars(extend("x-lint" = ">= 0"))]
    pub outline: f32,
    /// Épaisseur du contour : un pixel de sprite, soit `1 / outline_px_per_m` m (borné d'un à trois
    /// pixels de l'écran).
    #[schemars(extend("x-lint" = "> 0"))]
    pub outline_px_per_m: f32,
    /// Ombre portée sur le sol (`None` : pas d'ombre).
    #[serde(default)]
    pub shadow: Option<ModelShadow>,
}

/// Ombre portée : le modèle projeté sur le sol le long de la lumière.
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ModelShadow {
    /// Direction vers la lumière, repère du monde (x vers la droite, y vers le haut, z vers la
    /// caméra) ; `y > 0`.
    pub direction: (f32, f32, f32),
    /// Opacité de l'ombre (noir) posée sur le décor.
    #[schemars(extend("x-lint" = "[0, 1]"))]
    pub opacity: f32,
}

/// Un modèle animé et ses poses.
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ModelDef {
    /// Modèle animé (`anime.glb`), chemin sous `assets/`.
    pub glb: String,
    /// Manifeste de la pipeline (`anime.json` : `fps`, `actions`, `armes`, `prises`, `moments`).
    pub manifest: String,
    /// Cadre de l'image du personnage (m, dans le plan de la caméra) : largeur, hauteur. Il
    /// contient le corps, l'arme tenue et l'ombre.
    pub frame_m: (f32, f32),
    /// Le pied (origine du modèle) dans le cadre, en fractions depuis le coin bas-gauche.
    pub pivot: (f32, f32),
    /// `z` local de l'image du personnage, comme `offset_z` des feuilles de sprites : à égalité avec
    /// celles des planches du jeu (1,0 pour `alacod import-sprite`), le tri en y ordonne les corps 3D et le
    /// décor sprite par leur seul pied. La barre de vie se pose juste devant.
    #[serde(default)]
    pub offset_z: f32,
    /// Hauteur du personnage (m) : la barre de vie se pose au-dessus.
    #[schemars(extend("x-lint" = "> 0"))]
    pub height_m: f32,
    /// Au-delà de cette vitesse (m/s, au sol), l'allure `run` remplace `walk`.
    #[serde(default)]
    pub run_above_m_s: Option<f32>,
    /// Poses sans arme à distance ou d'une arme absente de `weapons`.
    pub poses: Poses,
    /// Poses par id d'arme de la simulation (clé de `weapons.ron`) : chaque champ présent remplace
    /// celui de `poses`.
    #[serde(default)]
    pub weapons: BTreeMap<String, Poses>,
    /// Clips de l'esquive (dash), joués à tour de rôle, un par dash.
    #[serde(default)]
    pub dash: Vec<String>,
    /// M-6 (§116) : arme de mêlée de la simulation (clé de `melee_weapons.ron`) -> clip du manifeste
    /// qui joue son coup. Le `moments.<clip>.frappe` du clip (fractions du clip) donne la fenêtre de
    /// frappe de l'arme, en images entières à 60 Hz, quand le RON de l'arme n'a pas de `strike_frames`
    /// (le RON prime ; le lint signale l'écart). Sans lien, rien n'est dérivé.
    #[serde(default)]
    pub melee_clips: BTreeMap<String, String>,
}

/// Les clips d'un état de la simulation. Un clip d'allure (`idle`, `walk`, `run`) boucle ; un
/// clip « en marchant » (`Gesture::moving`) est cherché dans la direction de la marche par rapport
/// au corps (`moments.<clip>.direction` du manifeste, variantes `<base>` et `<base>-*`).
#[derive(Debug, Clone, Default, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Poses {
    /// Arme du manifeste (`armes`) dont l'objet est montré dans la main (côté choisi par la prise
    /// du clip en cours, comme au banc).
    #[serde(default)]
    pub object: Option<String>,
    /// Repère de l'arme du manifeste (`armes.<objet>.reperes.<côté>.<repère>`) d'où partent les
    /// balles à l'écran.
    #[serde(default)]
    pub muzzle: Option<String>,
    /// À l'arrêt.
    #[serde(default)]
    pub idle: Option<String>,
    /// En marchant (base des variantes de direction, jouées à la vitesse où les pieds ne glissent
    /// pas : `moments.<clip>.vitesse`).
    #[serde(default)]
    pub walk: Option<String>,
    /// En courant (`run_above_m_s`).
    #[serde(default)]
    pub run: Option<String>,
    /// Un tir vient de partir (`WeaponState::last_fire_frame`).
    #[serde(default)]
    pub shoot: Option<Gesture>,
    /// Détente tenue sans tir en cours (`WeaponState::is_firing`) : en joue.
    #[serde(default)]
    pub aim: Option<Gesture>,
    /// Passage de l'arme portée à la mise en joue (une fois, au début de `aim`).
    #[serde(default)]
    pub shoulder: Option<Gesture>,
    /// Rechargement (`WeaponInventory::reloading_ending_frame`) ; le clip à l'arrêt suit
    /// l'avancement du rechargement.
    #[serde(default)]
    pub reload: Option<Gesture>,
    /// Coup de mêlée (`MeleeAttackState::is_attacking`).
    #[serde(default)]
    pub melee: Option<Gesture>,
}

/// Un geste : son clip à l'arrêt et sa variante en marchant.
#[derive(Debug, Clone, Default, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Gesture {
    /// Clip à l'arrêt.
    #[serde(default)]
    pub still: Option<String>,
    /// Base des clips en marchant (une variante par direction, `moments.<clip>.direction`).
    #[serde(default)]
    pub moving: Option<String>,
    /// Durée du geste (s) ; sans elle, celle du clip.
    #[serde(default)]
    pub duration_s: Option<f32>,
    /// Arme du manifeste montrée pendant le geste (remplace `Poses::object`).
    #[serde(default)]
    pub object: Option<String>,
}

impl Poses {
    /// `self` complété par `base` champ par champ (les poses d'une arme sur celles du modèle).
    pub fn over(&self, base: &Poses) -> Poses {
        Poses {
            object: self.object.clone().or_else(|| base.object.clone()),
            muzzle: self.muzzle.clone().or_else(|| base.muzzle.clone()),
            idle: self.idle.clone().or_else(|| base.idle.clone()),
            walk: self.walk.clone().or_else(|| base.walk.clone()),
            run: self.run.clone().or_else(|| base.run.clone()),
            shoot: self.shoot.clone().or_else(|| base.shoot.clone()),
            aim: self.aim.clone().or_else(|| base.aim.clone()),
            shoulder: self.shoulder.clone().or_else(|| base.shoulder.clone()),
            reload: self.reload.clone().or_else(|| base.reload.clone()),
            melee: self.melee.clone().or_else(|| base.melee.clone()),
        }
    }

    fn gestures(&self) -> [(&'static str, &Option<Gesture>); 5] {
        [
            ("shoot", &self.shoot),
            ("aim", &self.aim),
            ("shoulder", &self.shoulder),
            ("reload", &self.reload),
            ("melee", &self.melee),
        ]
    }
}

/// Manifeste `anime.json` de la pipeline (étape `banc`), réduit à ce que lit l'engine. Les champs
/// inconnus sont ignorés (le manifeste porte aussi la gigue, le tempo, la source…).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AnimeManifest {
    /// Images par seconde des clips.
    pub fps: f32,
    /// Clip -> plage d'images `[début, fin]`.
    #[serde(rename = "actions")]
    pub clips: BTreeMap<String, (f32, f32)>,
    /// Arme du modèle -> ses objets (un par côté tenu) et ses repères.
    #[serde(rename = "armes", default)]
    pub weapons: BTreeMap<String, ManifestWeapon>,
    /// Clip -> prise (`"une-main"`, `"deux-mains"`).
    #[serde(rename = "prises", default)]
    pub grips: BTreeMap<String, String>,
    /// Clip -> repères du clip (vitesse de l'allure, direction de la marche).
    #[serde(rename = "moments", default)]
    pub moments: BTreeMap<String, ManifestMoment>,
}

/// Prise d'un clip où l'arme est tenue par l'os de l'arme (côté `"arme"`), si l'arme en a un.
pub const TWO_HANDED_GRIP: &str = "deux-mains";
/// Côté de l'objet d'une arme tenu par l'os de l'arme.
pub const WEAPON_SIDE: &str = "arme";
/// Côté de l'objet d'une arme tenu dans la main.
pub const HAND_SIDE: &str = "main";

/// Une arme du modèle : ses objets (nœuds du glTF) par côté et ses repères par côté.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ManifestWeapon {
    /// Côté (`"main"`, `"arme"`) -> nœud de l'objet.
    #[serde(rename = "objets", default)]
    pub objects: BTreeMap<String, String>,
    /// Clips où l'arme est en main.
    #[serde(default)]
    pub animations: Vec<String>,
    /// Côté -> repère (`"bouche"`…) -> nœud.
    #[serde(rename = "reperes", default)]
    pub markers: BTreeMap<String, BTreeMap<String, String>>,
}

/// Repères d'un clip, en fractions de sa durée (`[0, 1]`) sauf `vitesse` et `direction`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ManifestMoment {
    /// Vitesse (m/s) où les pieds ne glissent pas.
    #[serde(rename = "vitesse", default)]
    pub speed: Option<f32>,
    /// Direction de la marche par rapport au corps (degrés, + vers sa gauche).
    #[serde(default)]
    pub direction: Option<f32>,
    /// M-6 : fenêtre où le coup touche, `[début, fin]` (fractions du clip).
    #[serde(rename = "frappe", default)]
    pub strike: Option<(f32, f32)>,
    /// M-6 : instant où le tir part (fraction du clip). Lu et exposé, non câblé (D206).
    #[serde(rename = "depart", default)]
    pub start: Option<f32>,
    /// M-6 : déplacement du corps pendant le geste (le pas d'une baïonnette). Lu et exposé, non
    /// câblé (D206).
    #[serde(rename = "deplacement", default)]
    pub movement: Option<ManifestMovement>,
}

/// Déplacement du corps pendant un geste : `distance` (m) parcourue entre les fractions `debut` et
/// `fin` du clip.
#[derive(Debug, Clone, Deserialize)]
pub struct ManifestMovement {
    pub distance: f32,
    #[serde(rename = "debut")]
    pub start: f32,
    #[serde(rename = "fin")]
    pub end: f32,
}

/// Images de simulation par seconde : l'unité des `strike_frames` d'une arme de mêlée.
pub const SIM_FRAMES_PER_SECOND: u32 = 60;

/// Les repères d'un clip résolus en **images de simulation entières** (60 Hz, depuis le début du
/// clip) : la seule forme qui sorte du chargement du contenu, aucun `f32` ne passe au-delà.
#[derive(Debug, Clone, PartialEq)]
pub struct ClipMoments {
    /// Durée du clip, en images de simulation (au moins 1).
    pub frames: u32,
    /// Fenêtre de frappe `(début, fin)`, bornes comprises, comme `strike_frames` d'une arme.
    pub strike: Option<(u32, u32)>,
    /// Image où le tir part.
    pub start: Option<u32>,
    /// Déplacement du corps : non câblé (D206).
    pub movement: Option<ClipMovement>,
}

/// Déplacement résolu : `distance_m` (mètres, donnée de contenu qu'aucune simulation ne lit
/// encore, D206) entre les images `from` et `to`, bornes comprises.
#[derive(Debug, Clone, PartialEq)]
pub struct ClipMovement {
    pub distance_m: f32,
    pub from: u32,
    pub to: u32,
}

/// Un nombre du manifeste tel que l'auteur l'a écrit : le plus court décimal qui revient au même
/// `f32` (0,45 et non 0,449999988…), pour que les arrondis du chargement ne dépendent pas de la
/// représentation binaire.
fn decimal(x: f32) -> f64 {
    x.to_string().parse().unwrap_or_else(|_| f64::from(x))
}

/// Arrondi écrit des repères : à l'image la plus proche, la demi-image **vers le haut**
/// (9,5 -> 10) ; 1e-9 d'indulgence absorbe l'erreur d'un produit décimal en `f64`. Négatif ou
/// `NaN` : 0.
fn round_frames(x: f64) -> u32 {
    (x + 0.5 + 1e-9).floor().max(0.0) as u32
}

/// `[début, fin]` : deux fractions finies de `[0, 1]`, dans l'ordre.
fn fraction_range_ok(from: f32, to: f32) -> bool {
    (0.0..=1.0).contains(&from) && (0.0..=1.0).contains(&to) && from <= to
}

impl AnimeManifest {
    /// Durée d'un clip (s), d'après sa plage d'images.
    pub fn duration(&self, clip: &str) -> Option<f32> {
        let (start, end) = self.clips.get(clip)?;
        Some(((end - start) / self.fps.max(1.0)).max(1.0 / self.fps.max(1.0)))
    }

    /// Durée d'un clip en images de simulation, **non arrondie** (la même durée que
    /// [`Self::duration`], calculée en décimal : `(fin − début) / fps × 60`).
    fn exact_frames(&self, clip: &str) -> Option<f64> {
        let (start, end) = self.clips.get(clip)?;
        let fps = decimal(self.fps).max(1.0);
        let seconds = ((decimal(*end) - decimal(*start)) / fps).max(1.0 / fps);
        Some(seconds * f64::from(SIM_FRAMES_PER_SECOND))
    }

    /// Les repères d'un clip en images de simulation entières (M-6, § 116) : chaque fraction du
    /// clip × sa durée à 60 Hz, arrondie à l'image la plus proche (demi-image vers le haut), bornée
    /// à la dernière image du clip. `None` : clip inconnu. Un repère absent ou hors plage (le lint
    /// le refuse) reste `None` : rien n'est inventé.
    pub fn clip_moments(&self, clip: &str) -> Option<ClipMoments> {
        let exact = self.exact_frames(clip)?;
        let frames = round_frames(exact).max(1);
        let frame_of = |fraction: f32| round_frames(decimal(fraction) * exact).min(frames - 1);
        let moment = self.moments.get(clip);
        Some(ClipMoments {
            frames,
            strike: moment
                .and_then(|m| m.strike)
                .filter(|(from, to)| fraction_range_ok(*from, *to))
                .map(|(from, to)| (frame_of(from), frame_of(to))),
            start: moment
                .and_then(|m| m.start)
                .filter(|start| fraction_range_ok(*start, *start))
                .map(frame_of),
            movement: moment
                .and_then(|m| m.movement.as_ref())
                .filter(|m| m.distance.is_finite() && fraction_range_ok(m.start, m.end))
                .map(|m| ClipMovement {
                    distance_m: m.distance,
                    from: frame_of(m.start),
                    to: frame_of(m.end),
                }),
        })
    }

    /// Les variantes de direction d'une base de marche : `base` et `base-<direction>` dont le
    /// moment déclare une direction. Les suffixes de la pipeline sont français, anglais ou abrégés
    /// (`gauche`, `left`, `l`…). Un autre geste (`fire-walk` pour `fire`) n'est pas une direction.
    /// Triées par nom ; la base peut être absente si seules ses variantes ont été exportées.
    pub fn directional(&self, base: &str) -> Vec<(&str, f32)> {
        self.clips
            .keys()
            .filter(|name| {
                name.as_str() == base
                    || name
                        .strip_prefix(base)
                        .and_then(|suffix| suffix.strip_prefix('-'))
                        .is_some_and(|suffix| {
                            matches!(
                                suffix,
                                "avant"
                                    | "avant-gauche"
                                    | "avant-droite"
                                    | "arriere"
                                    | "arriere-gauche"
                                    | "arriere-droite"
                                    | "gauche"
                                    | "droite"
                                    | "front"
                                    | "front-left"
                                    | "front-right"
                                    | "back"
                                    | "back-left"
                                    | "back-right"
                                    | "left"
                                    | "right"
                                    | "f"
                                    | "fl"
                                    | "fr"
                                    | "b"
                                    | "bl"
                                    | "br"
                                    | "l"
                                    | "r"
                            )
                        })
            })
            .filter_map(|name| {
                let dir = self.moments.get(name)?.direction?;
                Some((name.as_str(), dir))
            })
            .collect()
    }

    /// Un clip est-il nommé, seul ou comme base de variantes de direction ?
    fn knows(&self, clip: &str) -> bool {
        self.clips.contains_key(clip) || !self.directional(clip).is_empty()
    }
}

/// Le fichier `Model` d'un jeu, chargé par le registre, avec le manifeste lu de chaque modèle.
#[derive(Debug, Clone)]
pub struct ModelsTable {
    pub file: PathBuf,
    pub def: ModelsFile,
    /// Id du modèle -> manifeste (ou l'erreur de lecture, rapportée par le lint).
    pub manifests: BTreeMap<String, Result<AnimeManifest, String>>,
}

impl ModelsTable {
    /// Le modèle d'un `asset_name_ref` et son manifeste lisible.
    pub fn get(&self, id: &str) -> Option<(&ModelDef, &AnimeManifest)> {
        let def = self.def.models.get(id)?;
        let manifest = self.manifests.get(id)?.as_ref().ok()?;
        Some((def, manifest))
    }

    /// Les liens arme de mêlée -> clip de tous les modèles (`ModelDef::melee_clips`), dans l'ordre
    /// (arme, modèle). `window` : le `moments.frappe` du clip résolu en images, s'il en a un.
    pub fn melee_bindings(&self) -> Vec<MeleeBinding> {
        let mut out = Vec::new();
        for (id, model) in &self.def.models {
            let Some(Ok(manifest)) = self.manifests.get(id) else {
                continue;
            };
            for (weapon, clip) in &model.melee_clips {
                let Some(moments) = manifest.clip_moments(clip) else {
                    continue;
                };
                out.push(MeleeBinding {
                    weapon: weapon.clone(),
                    model: id.clone(),
                    clip: clip.clone(),
                    window: moments.strike,
                });
            }
        }
        out.sort_by(|a, b| (&a.weapon, &a.model).cmp(&(&b.weapon, &b.model)));
        out
    }

    /// La fenêtre de frappe dérivée de chaque arme de mêlée liée : celle du premier modèle (par
    /// id) dont le clip a un `moments.frappe` (le lint refuse deux modèles qui divergent). C'est
    /// ce que `game::weapon_assets` pose en `strike_frames` d'une arme dont le RON n'en a pas.
    pub fn strike_windows(&self) -> BTreeMap<String, (u32, u32)> {
        let mut out = BTreeMap::new();
        for binding in self.melee_bindings() {
            if let Some(window) = binding.window {
                out.entry(binding.weapon).or_insert(window);
            }
        }
        out
    }
}

/// Un lien arme de mêlée -> clip d'un modèle (`ModelDef::melee_clips`), résolu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeleeBinding {
    pub weapon: String,
    pub model: String,
    pub clip: String,
    /// `moments.frappe` du clip en images de simulation ; `None` : le clip n'en a pas.
    pub window: Option<(u32, u32)>,
}

/// Avertissements entre le RON des armes de mêlée et les moments (M-6, § 116). `ron_strike` :
/// pour chaque arme de mêlée du jeu, son `strike_frames` (le RON prime). Deux cas : le RON donne une
/// fenêtre que le clip ne confirme pas ; ni le RON ni le clip n'en donnent (l'arme frappe pendant tout
/// son coup).
pub fn strike_warnings(
    table: &ModelsTable,
    ron_strike: &BTreeMap<String, Option<(u32, u32)>>,
) -> Vec<String> {
    let mut out = Vec::new();
    let windows = table.strike_windows();
    for binding in table.melee_bindings() {
        let Some(ron) = ron_strike.get(&binding.weapon) else {
            continue;
        };
        let MeleeBinding {
            weapon,
            model,
            clip,
            window,
        } = &binding;
        match (ron, window) {
            (Some(ron), Some(derived)) if ron != derived => out.push(format!(
                "arme de corps à corps « {weapon} » : strike_frames = {ron:?} du RON, mais moments.frappe du clip « {clip} » du modèle « {model} » donne {derived:?} : le RON prime"
            )),
            (None, None) if !windows.contains_key(weapon) => out.push(format!(
                "arme de corps à corps « {weapon} » : liée au clip « {clip} » du modèle « {model} » qui n'a pas de moments.frappe, et sans strike_frames : elle frappe pendant tout son coup"
            )),
            _ => {}
        }
    }
    out
}

/// Erreurs sémantiques du fichier `Model` : fichiers absents, manifeste illisible, plages, clips,
/// objets et repères inconnus du manifeste, arme inconnue de la simulation.
pub fn lint(
    table: &ModelsTable,
    asset_exists: &dyn Fn(&str) -> bool,
    weapon_known: &dyn Fn(&str) -> bool,
    melee_known: &dyn Fn(&str) -> bool,
) -> Vec<(crate::LintErrorKind, String)> {
    use crate::LintErrorKind::{BrokenReference, OutOfRange, Parse};
    let mut out = Vec::new();
    let def = &table.def;
    if def.units_per_meter <= 0.0 {
        out.push((OutOfRange, "units_per_meter doit être > 0".to_string()));
    }
    if !(def.elevation_deg > 0.0 && def.elevation_deg < 90.0) {
        out.push((
            OutOfRange,
            "elevation_deg doit être dans ]0, 90[".to_string(),
        ));
    }
    let look = &def.look;
    if look.shade < 0.0 || look.highlight < 0.0 || look.outline < 0.0 {
        out.push((
            OutOfRange,
            "look : shade, highlight et outline doivent être >= 0".to_string(),
        ));
    }
    if look.outline_px_per_m <= 0.0 {
        out.push((
            OutOfRange,
            "look.outline_px_per_m doit être > 0".to_string(),
        ));
    }
    if look.light == (0.0, 0.0, 0.0) {
        out.push((OutOfRange, "look.light ne peut pas être nulle".to_string()));
    }
    if let Some(shadow) = &look.shadow {
        if shadow.direction.1 <= 0.0 {
            out.push((
                OutOfRange,
                "look.shadow.direction : la lumière doit venir d'en haut (y > 0)".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&shadow.opacity) {
            out.push((
                OutOfRange,
                "look.shadow.opacity doit être dans [0, 1]".to_string(),
            ));
        }
    }

    for (id, model) in &def.models {
        let at = |field: &str| format!("modèle « {id} » : {field}");
        for (field, path) in [("glb", &model.glb), ("manifest", &model.manifest)] {
            if !asset_exists(path) {
                out.push((
                    BrokenReference,
                    at(&format!("{field} = « {path} » : fichier absent de assets/")),
                ));
            }
        }
        if model.frame_m.0 <= 0.0 || model.frame_m.1 <= 0.0 {
            out.push((OutOfRange, at("frame_m doit être > 0")));
        }
        if !(0.0..=1.0).contains(&model.pivot.0) || !(0.0..=1.0).contains(&model.pivot.1) {
            out.push((OutOfRange, at("pivot doit être dans [0, 1]")));
        }
        if model.height_m <= 0.0 {
            out.push((OutOfRange, at("height_m doit être > 0")));
        }
        for weapon in model.weapons.keys() {
            if !weapon_known(weapon) {
                out.push((
                    BrokenReference,
                    at(&format!("weapons.{weapon} : arme inconnue (kind Weapon)")),
                ));
            }
        }
        for weapon in model.melee_clips.keys() {
            if !melee_known(weapon) {
                out.push((
                    BrokenReference,
                    at(&format!(
                        "melee_clips.{weapon} : arme de corps à corps inconnue (kind MeleeWeapon)"
                    )),
                ));
            }
        }
        let manifest = match table.manifests.get(id) {
            Some(Ok(manifest)) => manifest,
            Some(Err(e)) => {
                out.push((Parse, at(&format!("manifeste illisible : {e}"))));
                continue;
            }
            None => continue,
        };
        for (name, moment) in &manifest.moments {
            let at_moment = |field: &str| at(&format!("moments.{name}.{field}"));
            if let Some((from, to)) = moment.strike {
                if !fraction_range_ok(from, to) {
                    out.push((
                        OutOfRange,
                        at_moment("frappe : doit être [début, fin] dans [0, 1], début <= fin"),
                    ));
                }
            }
            if let Some(start) = moment.start {
                if !fraction_range_ok(start, start) {
                    out.push((OutOfRange, at_moment("depart : doit être dans [0, 1]")));
                }
            }
            if let Some(movement) = &moment.movement {
                if !movement.distance.is_finite() {
                    out.push((
                        OutOfRange,
                        at_moment("deplacement : distance doit être un nombre fini"),
                    ));
                }
                if !fraction_range_ok(movement.start, movement.end) {
                    out.push((
                        OutOfRange,
                        at_moment(
                            "deplacement : debut et fin doivent être dans [0, 1], debut <= fin",
                        ),
                    ));
                }
            }
        }
        for (weapon, clip_name) in &model.melee_clips {
            if !manifest.clips.contains_key(clip_name) {
                out.push((
                    BrokenReference,
                    at(&format!(
                        "melee_clips.{weapon} = « {clip_name} » : clip absent du manifeste (actions)"
                    )),
                ));
            }
        }
        let mut clip = |field: String, name: &str| {
            if !manifest.knows(name) {
                out.push((
                    BrokenReference,
                    at(&format!("{field} = « {name} » : clip absent du manifeste")),
                ));
            }
        };
        for (n, name) in model.dash.iter().enumerate() {
            clip(format!("dash[{n}]"), name);
        }
        let poses = std::iter::once(("poses".to_string(), &model.poses)).chain(
            model
                .weapons
                .iter()
                .map(|(w, p)| (format!("weapons.{w}"), p)),
        );
        let mut objects = Vec::new();
        for (where_, poses) in poses {
            for (field, name) in [
                ("idle", &poses.idle),
                ("walk", &poses.walk),
                ("run", &poses.run),
            ] {
                if let Some(name) = name {
                    clip(format!("{where_}.{field}"), name);
                }
            }
            for (field, gesture) in poses.gestures() {
                let Some(gesture) = gesture else { continue };
                for (side, name) in [("still", &gesture.still), ("moving", &gesture.moving)] {
                    if let Some(name) = name {
                        clip(format!("{where_}.{field}.{side}"), name);
                    }
                }
                if let Some(object) = &gesture.object {
                    objects.push((format!("{where_}.{field}.object"), object.clone(), None));
                }
            }
            if let Some(object) = &poses.object {
                objects.push((
                    format!("{where_}.object"),
                    object.clone(),
                    poses.muzzle.clone(),
                ));
            }
        }
        for (field, object, muzzle) in objects {
            let Some(weapon) = manifest.weapons.get(&object) else {
                out.push((
                    BrokenReference,
                    at(&format!(
                        "{field} = « {object} » : arme absente du manifeste (armes)"
                    )),
                ));
                continue;
            };
            if let Some(muzzle) = muzzle {
                if !weapon.markers.values().any(|m| m.contains_key(&muzzle)) {
                    out.push((
                        BrokenReference,
                        at(&format!(
                            "muzzle = « {muzzle} » : repère absent de l'arme « {object} » du manifeste"
                        )),
                    ));
                }
            }
        }
    }
    let mut derived: BTreeMap<String, Vec<(String, (u32, u32))>> = BTreeMap::new();
    for binding in table.melee_bindings() {
        if let Some(window) = binding.window {
            derived
                .entry(binding.weapon)
                .or_default()
                .push((binding.model, window));
        }
    }
    for (weapon, windows) in derived {
        if windows.iter().any(|(_, w)| *w != windows[0].1) {
            out.push((
                OutOfRange,
                format!(
                    "arme de corps à corps « {weapon} » : liée par plusieurs modèles dont les fenêtres de frappe diffèrent {windows:?}"
                ),
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> AnimeManifest {
        serde_json::from_str(
            r#"{"fps": 24, "actions": {"idle": [1, 49], "walk": [1, 9], "walk-left": [1, 9], "shoot": [1, 8]},
                "armes": {"gun": {"objets": {"main": "gun_hand"}, "animations": ["idle"], "reperes": {"main": {"tip": "gun_tip"}}}},
                "moments": {"walk": {"vitesse": 3.0, "direction": 0}, "walk-left": {"vitesse": 3.0, "direction": 90}},
                "gigue": []}"#,
        )
        .unwrap()
    }

    #[test]
    fn directional_variants_need_a_declared_direction() {
        let m = manifest();
        assert_eq!(
            m.directional("walk"),
            vec![("walk", 0.0), ("walk-left", 90.0)]
        );
        assert!(m.directional("idle").is_empty());
        assert!(m.knows("walk") && m.knows("idle") && !m.knows("run"));
        assert_eq!(m.duration("shoot"), Some(7.0 / 24.0));
    }

    #[test]
    fn directional_families_do_not_include_other_gestures() {
        let m: AnimeManifest = serde_json::from_str(
            r#"{"fps": 24,
                "actions": {"fire": [1, 8], "fire-walk": [1, 9], "fire-walk-l": [1, 9],
                    "fire-walk-arriere-droite": [1, 9], "run-left": [1, 9]},
                "moments": {"fire-walk": {"direction": 0}, "fire-walk-l": {"direction": 90},
                    "fire-walk-arriere-droite": {"direction": -135}, "run-left": {"direction": 90}}}"#,
        )
        .unwrap();
        assert!(m.directional("fire").is_empty());
        assert_eq!(
            m.directional("fire-walk"),
            vec![
                ("fire-walk", 0.0),
                ("fire-walk-arriere-droite", -135.0),
                ("fire-walk-l", 90.0),
            ]
        );
        assert_eq!(m.directional("run"), vec![("run-left", 90.0)]);
        assert!(m.knows("fire") && m.knows("run"));
        assert!(!m.knows("fi") && !m.knows("missing"));
    }

    #[test]
    fn get_requires_a_readable_manifest() {
        let mut t = table(&format!(
            r#"(units_per_meter: 24.0, elevation_deg: 45.0, {LOOK}, models: {{
                "ok": (glb: "a.glb", manifest: "a.json", frame_m: (3.0, 3.0), pivot: (0.5, 0.2), height_m: 1.8, poses: ()),
                "broken": (glb: "b.glb", manifest: "b.json", frame_m: (3.0, 3.0), pivot: (0.5, 0.2), height_m: 1.8, poses: ()),
            }})"#
        ));
        t.manifests.insert("broken".into(), Err("illisible".into()));
        assert!(t.get("ok").is_some());
        assert!(
            t.get("broken").is_none(),
            "manifeste illisible : pas de modèle"
        );
        assert!(t.get("absent").is_none());
    }

    fn table(models_ron: &str) -> ModelsTable {
        let def: ModelsFile = ron::from_str(models_ron).unwrap();
        let manifests = def
            .models
            .keys()
            .map(|id| (id.clone(), Ok(manifest())))
            .collect();
        ModelsTable {
            file: "models/models.ron".into(),
            def,
            manifests,
        }
    }

    const LOOK: &str = "look: (shade: 0.66, highlight: 1.12, light: (-0.5, 0.75, 0.45), outline: 0.75, outline_px_per_m: 91.0)";

    #[test]
    fn unknown_clips_objects_markers_and_weapons_are_reported() {
        let t = table(&format!(
            r#"(units_per_meter: 24.0, elevation_deg: 45.0, {LOOK}, models: {{
                "hero": (glb: "a.glb", manifest: "a.json", frame_m: (3.0, 3.0), pivot: (0.5, 0.2), height_m: 1.8,
                    poses: (idle: Some("idle"), walk: Some("walk"), run: Some("sprint"),
                            shoot: Some((still: Some("shoot"), moving: Some("shoot-moving")))),
                    weapons: {{"rifle": (object: Some("gun"), muzzle: Some("muzzle"))}},
                    dash: ["roll"]),
            }})"#
        ));
        let errors = lint(&t, &|_| true, &|w| w == "pistol", &|_| true);
        let text: Vec<_> = errors.iter().map(|(_, m)| m.as_str()).collect();
        assert_eq!(errors.len(), 5, "{text:#?}");
        for wanted in [
            "« sprint »",
            "« shoot-moving »",
            "« roll »",
            "« muzzle »",
            "weapons.rifle",
        ] {
            assert!(
                text.iter().any(|m| m.contains(wanted)),
                "{wanted} : {text:#?}"
            );
        }
    }

    #[test]
    fn a_valid_table_and_ranges() {
        let ok = table(&format!(
            r#"(units_per_meter: 24.0, elevation_deg: 45.0, y_sort: true, {LOOK}, models: {{
                "hero": (glb: "a.glb", manifest: "a.json", frame_m: (3.0, 3.0), pivot: (0.5, 0.2), height_m: 1.8,
                    poses: (object: Some("gun"), muzzle: Some("tip"), idle: Some("idle"), walk: Some("walk"))),
            }})"#
        ));
        assert!(lint(&ok, &|_| true, &|_| true, &|_| true).is_empty());
        let missing = lint(&ok, &|path| path != "a.glb", &|_| true, &|_| true);
        assert_eq!(missing.len(), 1);
        let bad = table(&format!(
            r#"(units_per_meter: 0.0, elevation_deg: 90.0, {LOOK}, models: {{}})"#
        ));
        assert_eq!(lint(&bad, &|_| true, &|_| true, &|_| true).len(), 2);
    }

    #[test]
    fn weapon_poses_override_field_by_field() {
        let base = Poses {
            idle: Some("idle".into()),
            walk: Some("walk".into()),
            ..Default::default()
        };
        let gun = Poses {
            idle: Some("gun-idle".into()),
            ..Default::default()
        };
        let merged = gun.over(&base);
        assert_eq!(merged.idle.as_deref(), Some("gun-idle"));
        assert_eq!(merged.walk.as_deref(), Some("walk"));
    }

    /// Manifeste à 24 images/s : `coup` dure 8,4 images de clip = 0,35 s = 21 images de simulation.
    fn moments_manifest(moments: &str) -> AnimeManifest {
        serde_json::from_str(&format!(
            r#"{{"fps": 24, "actions": {{"coup": [1, 9.4], "tir": [1, 9.4], "lent": [0, 15]}}, "moments": {moments}}}"#
        ))
        .unwrap()
    }

    #[test]
    fn moments_resolve_to_whole_frames_rounded_to_nearest() {
        let m = moments_manifest(
            r#"{"coup": {"frappe": [0.45, 0.65], "depart": 0.5,
                         "deplacement": {"distance": 0.4, "debut": 0.25, "fin": 0.45}}}"#,
        );
        let moments = m.clip_moments("coup").unwrap();
        assert_eq!(moments.frames, 21);
        // 0,45 × 21 = 9,45 -> 9 ; 0,65 × 21 = 13,65 -> 14.
        assert_eq!(moments.strike, Some((9, 14)));
        // 0,5 × 21 = 10,5 : la demi-image part vers le haut.
        assert_eq!(moments.start, Some(11));
        assert_eq!(
            moments.movement,
            Some(ClipMovement {
                distance_m: 0.4,
                from: 5,
                to: 9
            })
        );
        // Un clip sans repère : durée seule.
        let bare = m.clip_moments("tir").unwrap();
        assert_eq!(bare.frames, 21);
        assert_eq!((bare.strike, bare.start, bare.movement), (None, None, None));
    }

    #[test]
    fn moment_ties_round_up_whatever_the_binary_representation() {
        // `lent` : 15 images à 24/s = 0,625 s = 37,5 images de simulation ; 0,2 × 37,5 = 7,5 -> 8
        // et 0,6 × 37,5 = 22,5 -> 23 (demi-images exactes, arrondies vers le haut).
        let m = moments_manifest(r#"{"lent": {"frappe": [0.2, 0.6]}}"#);
        assert_eq!(m.clip_moments("lent").unwrap().strike, Some((8, 23)));
        // 0,35 n'est pas un `f32` exact : 0,35 × 20 doit rester 7, pas 6,99999… arrondi vers le bas.
        let m: AnimeManifest = serde_json::from_str(
            r#"{"fps": 30, "actions": {"c": [0, 10]}, "moments": {"c": {"frappe": [0.35, 0.65]}}}"#,
        )
        .unwrap();
        // 10 images à 30/s = 20 images de simulation : 7 et 13.
        assert_eq!(m.clip_moments("c").unwrap().strike, Some((7, 13)));
    }

    #[test]
    fn a_window_to_the_end_of_the_clip_stops_at_its_last_frame() {
        let m = moments_manifest(r#"{"coup": {"frappe": [0.9, 1.0]}}"#);
        // 0,9 × 21 = 18,9 -> 19 ; 1,0 × 21 = 21 -> dernière image du clip, 20.
        assert_eq!(m.clip_moments("coup").unwrap().strike, Some((19, 20)));
    }

    #[test]
    fn unknown_clips_and_invalid_moments_resolve_to_nothing() {
        let m = moments_manifest(
            r#"{"coup": {"frappe": [0.7, 0.2], "depart": 1.5,
                         "deplacement": {"distance": 0.4, "debut": 0.6, "fin": 0.2}}}"#,
        );
        assert_eq!(m.clip_moments("absent"), None);
        let moments = m.clip_moments("coup").unwrap();
        assert_eq!(
            (moments.strike, moments.start, moments.movement),
            (None, None, None)
        );
    }

    const MOMENT_TABLE: &str = r#"(units_per_meter: 24.0, elevation_deg: 45.0, look: (shade: 0.66, highlight: 1.12, light: (-0.5, 0.75, 0.45), outline: 0.75, outline_px_per_m: 91.0), models: {
        "a": (glb: "a.glb", manifest: "a.json", frame_m: (3.0, 3.0), pivot: (0.5, 0.2), height_m: 1.8, poses: (),
              melee_clips: {"hache": "coup", "absente": "coup", "perdue": "inconnu"}),
        "b": (glb: "b.glb", manifest: "b.json", frame_m: (3.0, 3.0), pivot: (0.5, 0.2), height_m: 1.8, poses: (),
              melee_clips: {"hache": "coup"}),
    })"#;

    fn moment_table(a: &str, b: &str) -> ModelsTable {
        let def: ModelsFile = ron::from_str(MOMENT_TABLE).unwrap();
        let manifests = BTreeMap::from([
            ("a".to_string(), Ok(moments_manifest(a))),
            ("b".to_string(), Ok(moments_manifest(b))),
        ]);
        ModelsTable {
            file: "models/models.ron".into(),
            def,
            manifests,
        }
    }

    #[test]
    fn a_bound_weapon_takes_the_window_of_its_clip() {
        let t = moment_table(r#"{"coup": {"frappe": [0.45, 0.65]}}"#, r#"{}"#);
        // Une entrée par arme liée à un clip connu qui a une frappe ; « perdue » (clip inconnu) : rien.
        assert_eq!(
            t.strike_windows(),
            BTreeMap::from([
                ("absente".to_string(), (9, 14)),
                ("hache".to_string(), (9, 14))
            ])
        );
        let errors = lint(&t, &|_| true, &|_| true, &|w| w != "absente");
        let text: Vec<_> = errors.iter().map(|(_, m)| m.as_str()).collect();
        assert_eq!(errors.len(), 2, "{text:#?}");
        for wanted in [
            "melee_clips.absente : arme de corps à corps inconnue",
            "« inconnu » : clip absent",
        ] {
            assert!(
                text.iter().any(|m| m.contains(wanted)),
                "{wanted} : {text:#?}"
            );
        }
    }

    #[test]
    fn two_models_binding_one_weapon_to_different_windows_are_refused() {
        let t = moment_table(
            r#"{"coup": {"frappe": [0.45, 0.65]}}"#,
            r#"{"coup": {"frappe": [0.1, 0.2]}}"#,
        );
        // Le premier modèle (par id) l'emporte pour la donnée ; le lint refuse l'écart.
        assert_eq!(t.strike_windows()["hache"], (9, 14));
        let errors = lint(&t, &|_| true, &|_| true, &|_| true);
        assert!(
            errors.iter().any(|(_, m)| m.contains("plusieurs modèles")),
            "{errors:#?}"
        );
    }

    #[test]
    fn invalid_moments_are_lint_errors() {
        let t = moment_table(
            r#"{"coup": {"frappe": [0.7, 0.2], "depart": 1.5,
                         "deplacement": {"distance": 0.4, "debut": 0.6, "fin": 0.2}}}"#,
            r#"{}"#,
        );
        let errors = lint(&t, &|_| true, &|_| true, &|_| true);
        let text: Vec<_> = errors.iter().map(|(_, m)| m.as_str()).collect();
        for wanted in [
            "moments.coup.frappe",
            "moments.coup.depart",
            "moments.coup.deplacement",
        ] {
            assert!(
                text.iter().any(|m| m.contains(wanted)),
                "{wanted} : {text:#?}"
            );
        }
    }

    #[test]
    fn a_ron_window_that_differs_is_a_warning_and_a_missing_one_is_too() {
        let t = moment_table(r#"{"coup": {"frappe": [0.45, 0.65]}}"#, r#"{}"#);
        let ron = |hache: Option<(u32, u32)>, absente: Option<(u32, u32)>| {
            BTreeMap::from([
                ("hache".to_string(), hache),
                ("absente".to_string(), absente),
            ])
        };
        // Même fenêtre, ou RON sans fenêtre contre un clip qui en donne une : rien à dire.
        assert!(strike_warnings(&t, &ron(Some((9, 14)), None)).is_empty());
        // Le RON diffère : avertissement, et le texte dit qui prime.
        let warned = strike_warnings(&t, &ron(Some((10, 13)), Some((9, 14))));
        assert_eq!(warned.len(), 1, "{warned:#?}");
        assert!(
            warned[0].contains("(10, 13) du RON")
                && warned[0].contains("(9, 14)")
                && warned[0].contains("le RON prime"),
            "{warned:#?}"
        );
        // Un autre modèle donne la fenêtre de l'arme (modèle « a ») : le second sans frappe ne dit rien.
        assert!(strike_warnings(&t, &ron(None, None)).is_empty());
        // Ni le RON ni aucun clip lié ne donnent de fenêtre : un avertissement par lien.
        let bare = moment_table("{}", "{}");
        let none = strike_warnings(&bare, &ron(None, None));
        assert_eq!(none.len(), 3, "{none:#?}");
        assert!(none.iter().all(|w| w.contains("pas de moments.frappe")));
        // Une arme inconnue du RON n'est pas jugée ici (le lint de référence la refuse).
        assert!(strike_warnings(&t, &BTreeMap::new()).is_empty());
    }
}
