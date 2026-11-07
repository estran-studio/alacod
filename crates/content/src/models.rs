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

/// Repères d'un clip.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ManifestMoment {
    /// Vitesse (m/s) où les pieds ne glissent pas.
    #[serde(rename = "vitesse", default)]
    pub speed: Option<f32>,
    /// Direction de la marche par rapport au corps (degrés, + vers sa gauche).
    #[serde(default)]
    pub direction: Option<f32>,
}

impl AnimeManifest {
    /// Durée d'un clip (s), d'après sa plage d'images.
    pub fn duration(&self, clip: &str) -> Option<f32> {
        let (start, end) = self.clips.get(clip)?;
        Some(((end - start) / self.fps.max(1.0)).max(1.0 / self.fps.max(1.0)))
    }

    /// Les variantes de direction d'une base de marche : `base` et `base-*` dont le moment déclare
    /// une direction. Triées par nom.
    pub fn directional(&self, base: &str) -> Vec<(&str, f32)> {
        let prefix = format!("{base}-");
        self.clips
            .keys()
            .filter(|name| name.as_str() == base || name.starts_with(&prefix))
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
}

/// Erreurs sémantiques du fichier `Model` : fichiers absents, manifeste illisible, plages, clips,
/// objets et repères inconnus du manifeste, arme inconnue de la simulation.
pub fn lint(
    table: &ModelsTable,
    asset_exists: &dyn Fn(&str) -> bool,
    weapon_known: &dyn Fn(&str) -> bool,
) -> Vec<(crate::LintErrorKind, String)> {
    use crate::LintErrorKind::{BrokenReference, OutOfRange, Parse};
    let mut out = Vec::new();
    let def = &table.def;
    if def.units_per_meter <= 0.0 {
        out.push((OutOfRange, "units_per_meter doit être > 0".to_string()));
    }
    if !(def.elevation_deg > 0.0 && def.elevation_deg < 90.0) {
        out.push((OutOfRange, "elevation_deg doit être dans ]0, 90[".to_string()));
    }
    let look = &def.look;
    if look.shade < 0.0 || look.highlight < 0.0 || look.outline < 0.0 {
        out.push((
            OutOfRange,
            "look : shade, highlight et outline doivent être >= 0".to_string(),
        ));
    }
    if look.outline_px_per_m <= 0.0 {
        out.push((OutOfRange, "look.outline_px_per_m doit être > 0".to_string()));
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
        let manifest = match table.manifests.get(id) {
            Some(Ok(manifest)) => manifest,
            Some(Err(e)) => {
                out.push((Parse, at(&format!("manifeste illisible : {e}"))));
                continue;
            }
            None => continue,
        };
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
        assert_eq!(m.directional("walk"), vec![("walk", 0.0), ("walk-left", 90.0)]);
        assert!(m.directional("idle").is_empty());
        assert!(m.knows("walk") && m.knows("idle") && !m.knows("run"));
        assert_eq!(m.duration("shoot"), Some(7.0 / 24.0));
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
        let errors = lint(&t, &|_| true, &|w| w == "pistol");
        let text: Vec<_> = errors.iter().map(|(_, m)| m.as_str()).collect();
        assert_eq!(errors.len(), 5, "{text:#?}");
        for wanted in ["« sprint »", "« shoot-moving »", "« roll »", "« muzzle »", "weapons.rifle"] {
            assert!(text.iter().any(|m| m.contains(wanted)), "{wanted} : {text:#?}");
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
        assert!(lint(&ok, &|_| true, &|_| true).is_empty());
        let missing = lint(&ok, &|path| path != "a.glb", &|_| true);
        assert_eq!(missing.len(), 1);
        let bad = table(&format!(
            r#"(units_per_meter: 0.0, elevation_deg: 90.0, {LOOK}, models: {{}})"#
        ));
        assert_eq!(lint(&bad, &|_| true, &|_| true).len(), 2);
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
}
