//! Réglages de `ui/feedback.ron` (T2.13, étendus en T1.17, `docs/conventions.md` §7 et §31).
//!
//! Présentation seule : aucun de ces champs n'entre dans la simulation ni dans le checksum.
//! Le type vit ici (et non dans `game`) pour que le lint et le jeu lisent exactement la même
//! structure ; `game::feedback::FeedbackConfig` l'enveloppe comme asset.
//!
//! Résolution d'un coup ([`FeedbackSettings::resolve`]) : pour chaque effet (hit stop, flash,
//! secousse), l'entrée `by_weapon` de l'arme l'emporte, puis l'entrée `by_kind` du genre de
//! dégât, puis la valeur globale. Un champ absent d'une entrée laisse passer le niveau suivant.

use serde::{Deserialize, Serialize};
use sim_core::damage::DamageKind;
use std::collections::BTreeMap;

/// Contenu de `ui/feedback.ron`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeedbackSettings {
    /// Flash à l'impact sur les calques du personnage touché.
    pub hit_flash: HitFlashConfig,
    /// Secousse de caméra quand un joueur local est touché.
    pub shake: ShakeConfig,
    /// Hit stop (T1.17) : frames de **rendu** pendant lesquelles animations et caméra sont
    /// gelées quand un joueur local touche ou est touché. 0 : pas de hit stop. Ne fige jamais
    /// la simulation.
    #[serde(default)]
    pub hit_stop_frames: u32,
    /// Chiffres de dégâts flottants au-dessus de la cible (T1.17).
    #[serde(default)]
    pub damage_numbers: bool,
    /// Cercle de télégraphe au sol (T1.17).
    #[serde(default)]
    pub telegraph: TelegraphConfig,
    /// Forme des patterns d'émetteur pendant leur télégraphe (M2-T19, §55).
    #[serde(default)]
    pub pattern_telegraph: PatternTelegraphConfig,
    /// Anneau et flash du blank (M2-T19, §55).
    #[serde(default)]
    pub blank: BlankFlashConfig,
    /// Gel de rendu, zoom et flash à la mort d'un boss (M2-T19, §55).
    #[serde(default)]
    pub boss_end: BossEndConfig,
    /// Anneau et secousse d'une explosion de bombe, de baril ou de champion (M3-T17, §82).
    #[serde(default)]
    pub explosion: ExplosionFxConfig,
    /// Surcharges par genre de dégât (`DamageKind`, ex. `Explosion`, `Custom("acide")`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub by_kind: BTreeMap<DamageKind, FeedbackOverride>,
    /// Surcharges par arme (nom d'arme à distance ou de corps à corps du jeu). Prioritaires
    /// sur `by_kind`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub by_weapon: BTreeMap<String, FeedbackOverride>,
    /// Sons de base, chemins relatifs à `assets/`. Clés : `shot_pistol`, `shot_shotgun`,
    /// `shot_smg` (tir, selon la famille du mode de tir actif), `shot` (repli du tir), `reload`,
    /// et les événements `hit`, `hurt`, `death`, `dash`, `blank`, `door` (`game::feedback`).
    /// Une arme surcharge le tir et le rechargement de chacun de ses modes par son
    /// `audio_config` (`weapons.ron`).
    #[serde(default)]
    pub sounds: BTreeMap<String, String>,
    /// Volume par clé de `sounds`, en décibels (0 : inchangé, négatif : plus bas), dans
    /// [`SOUND_VOLUME_DB_RANGE`]. S'applique aussi au son d'arme qui remplace la clé (tir de sa
    /// famille, `reload`). Le lint vérifie que la clé existe dans `sounds`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub volumes: BTreeMap<String, f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HitFlashConfig {
    pub frames: u32,
    /// Multiplicateur de couleur des calques (linéaire). Au-delà de 1.0 le sprite est
    /// surexposé : un flash `(1.0, 1.0, 1.0)` est l'identité, donc invisible.
    pub color: (f32, f32, f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ShakeConfig {
    pub frames: u32,
    pub amplitude: f32,
}

/// Apparence du cercle de télégraphe.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TelegraphConfig {
    /// Couleur RGBA du cercle (contour, et remplissage selon la progression).
    pub color: (f32, f32, f32, f32),
}

impl Default for TelegraphConfig {
    fn default() -> Self {
        Self {
            color: (1.0, 0.3, 0.2, 0.8),
        }
    }
}

/// Télégraphe de **forme** d'un pattern (M2-T19) : rayons d'un éventail, amorce d'une spirale,
/// cône d'un burst, dessinés pendant la phase `Telegraph` d'un émetteur.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PatternTelegraphConfig {
    pub enabled: bool,
    /// Couleur RGBA ; absente : celle du cercle (`telegraph.color`).
    pub color: Option<(f32, f32, f32, f32)>,
}

impl Default for PatternTelegraphConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            color: None,
        }
    }
}

/// Flash du blank (M2-T19) : anneau qui s'étend sur le rayon du blank, plus un flash léger.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BlankFlashConfig {
    pub enabled: bool,
    /// Durée de l'anneau, en frames de simulation.
    pub ring_frames: u32,
    /// Couleur RGBA de l'anneau.
    pub color: (f32, f32, f32, f32),
    /// Opacité initiale du flash plein écran (0 : pas de flash) ; s'estompe sur `ring_frames`.
    pub flash_alpha: f32,
}

impl Default for BlankFlashConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ring_frames: 24,
            color: (0.7, 0.9, 1.0, 1.0),
            flash_alpha: 0.15,
        }
    }
}

/// Explosion (M3-T17) : anneau qui s'étend sur le rayon de l'explosion, secousse de caméra
/// proportionnelle au rayon (`shake_amplitude` pour un rayon de 100), débris des rochers cassés.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ExplosionFxConfig {
    pub enabled: bool,
    /// Durée de l'anneau, en frames de simulation.
    pub ring_frames: u32,
    /// Couleur RGBA de l'anneau.
    pub color: (f32, f32, f32, f32),
    /// Durée de la secousse (0 : pas de secousse).
    pub shake_frames: u32,
    /// Amplitude de la secousse pour un rayon de 100 (proportionnelle au rayon, plafonnée à 3×).
    pub shake_amplitude: f32,
}

impl Default for ExplosionFxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ring_frames: 18,
            color: (1.0, 0.55, 0.15, 1.0),
            shake_frames: 10,
            shake_amplitude: 8.0,
        }
    }
}

/// Fin de boss (M2-T19) : gel de **rendu** (jamais de la simulation), zoom sur la position du
/// boss et flash plein écran.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BossEndConfig {
    pub enabled: bool,
    /// Ticks de rendu gelés (animations, caméra), comme le hit stop.
    pub freeze_frames: u32,
    /// Zoom avant (> 1 : plus près) sur la position du boss, relâché sur `zoom_frames`.
    pub zoom: f32,
    /// Durée du zoom, en frames de simulation.
    pub zoom_frames: u32,
    /// Opacité initiale du flash plein écran, estompé sur `zoom_frames`.
    pub flash_alpha: f32,
    pub flash_color: (f32, f32, f32),
}

impl Default for BossEndConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            freeze_frames: 30,
            zoom: 1.5,
            zoom_frames: 60,
            flash_alpha: 0.6,
            flash_color: (1.0, 1.0, 1.0),
        }
    }
}

/// Bornes du zoom de fin de boss (lint).
pub const BOSS_ZOOM_RANGE: (f32, f32) = (1.0, 4.0);
/// Plafond des gels et durées de présentation (lint) : au-delà, le jeu paraît planté.
pub const MAX_PRESENTATION_FRAMES: u32 = 600;

/// Surcharge d'une entrée `by_kind` ou `by_weapon` : chaque champ est optionnel.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct FeedbackOverride {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit_stop_frames: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit_flash: Option<HitFlashConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shake: Option<ShakeConfig>,
}

/// Réglages effectifs d'un coup, après résolution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedFeedback {
    pub hit_stop_frames: u32,
    pub hit_flash: HitFlashConfig,
    pub shake: ShakeConfig,
    /// La secousse vient d'une surcharge (`by_weapon` ou `by_kind`) : elle s'applique aussi
    /// quand c'est le joueur local qui touche (impact lourd : grenade, explosion), pas
    /// seulement quand il est touché.
    pub shake_overridden: bool,
}

/// Couleur de flash au-delà de laquelle le lint refuse (multiplicateur linéaire).
pub const MAX_FLASH_COLOR: f32 = 4.0;

/// Volumes acceptés pour `volumes`, en décibels : de quasi muet à +12 dB (au-delà, un
/// échantillon normalisé à -3 dBFS sature).
pub const SOUND_VOLUME_DB_RANGE: std::ops::RangeInclusive<f32> = -60.0..=12.0;

/// Problème de valeur trouvé par [`FeedbackSettings::problems`] : chemin du champ et message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackProblem {
    pub field: String,
    pub message: String,
}

impl FeedbackSettings {
    /// Réglages d'un coup de genre `kind`, porté par l'arme `weapon` si elle est connue.
    pub fn resolve(&self, kind: &DamageKind, weapon: Option<&str>) -> ResolvedFeedback {
        let by_weapon = weapon.and_then(|name| self.by_weapon.get(name));
        let by_kind = self.by_kind.get(kind);
        let levels = || [by_weapon, by_kind].into_iter().flatten();
        ResolvedFeedback {
            hit_stop_frames: levels()
                .find_map(|entry| entry.hit_stop_frames)
                .unwrap_or(self.hit_stop_frames),
            hit_flash: levels()
                .find_map(|entry| entry.hit_flash)
                .unwrap_or(self.hit_flash),
            shake: levels().find_map(|entry| entry.shake).unwrap_or(self.shake),
            shake_overridden: levels().any(|entry| entry.shake.is_some()),
        }
    }

    /// Valeurs hors plage (le lint les rapporte en `OutOfRange`) : `frames > 0` pour le flash
    /// et la secousse, `amplitude >= 0`, composantes de couleur dans `[0, MAX_FLASH_COLOR]`,
    /// alpha du télégraphe dans `[0, 1]`. Les noms d'armes de `by_weapon` sont vérifiés par
    /// le lint, qui connaît le registre.
    pub fn problems(&self) -> Vec<FeedbackProblem> {
        let mut problems = Vec::new();
        check_flash("hit_flash", &self.hit_flash, &mut problems);
        check_shake("shake", &self.shake, &mut problems);
        let (r, g, b, a) = self.telegraph.color;
        for (component, value) in [("r", r), ("g", g), ("b", b), ("a", a)] {
            if !(0.0..=1.0).contains(&value) {
                problems.push(FeedbackProblem {
                    field: format!("telegraph.color.{component}"),
                    message: format!("{value} : doit être dans [0, 1]"),
                });
            }
        }
        self.blank_and_boss_problems(&mut problems);
        for (key, db) in &self.volumes {
            if !SOUND_VOLUME_DB_RANGE.contains(db) {
                problems.push(FeedbackProblem {
                    field: format!("volumes.{key}"),
                    message: format!(
                        "{db} dB : doit être dans [{}, {}]",
                        SOUND_VOLUME_DB_RANGE.start(),
                        SOUND_VOLUME_DB_RANGE.end()
                    ),
                });
            }
        }
        let overrides = self
            .by_kind
            .iter()
            .map(|(kind, entry)| (format!("by_kind.{kind:?}"), entry))
            .chain(
                self.by_weapon
                    .iter()
                    .map(|(name, entry)| (format!("by_weapon.{name}"), entry)),
            );
        for (prefix, entry) in overrides {
            if let Some(flash) = &entry.hit_flash {
                check_flash(&format!("{prefix}.hit_flash"), flash, &mut problems);
            }
            if let Some(shake) = &entry.shake {
                check_shake(&format!("{prefix}.shake"), shake, &mut problems);
            }
        }
        problems
    }
}

impl FeedbackSettings {
    /// M2-T19 : durées > 0 et bornées, opacités dans `[0, 1]`, zoom dans [`BOSS_ZOOM_RANGE`],
    /// couleurs du télégraphe de forme dans `[0, 1]`.
    fn blank_and_boss_problems(&self, problems: &mut Vec<FeedbackProblem>) {
        let mut bad = |field: &str, message: String| {
            problems.push(FeedbackProblem {
                field: field.into(),
                message,
            })
        };
        let frames = 1..=MAX_PRESENTATION_FRAMES;
        let unit = 0.0..=1.0;
        let blank = &self.blank;
        if !frames.contains(&blank.ring_frames) {
            bad(
                "blank.ring_frames",
                format!(
                    "{} : doit être dans [1, {MAX_PRESENTATION_FRAMES}]",
                    blank.ring_frames
                ),
            );
        }
        if !unit.contains(&blank.flash_alpha) {
            bad(
                "blank.flash_alpha",
                format!("{} : doit être dans [0, 1]", blank.flash_alpha),
            );
        }
        let (r, g, b, a) = blank.color;
        for (c, v) in [("r", r), ("g", g), ("b", b), ("a", a)] {
            if !unit.contains(&v) {
                bad(
                    &format!("blank.color.{c}"),
                    format!("{v} : doit être dans [0, 1]"),
                );
            }
        }
        let explosion = &self.explosion;
        if !frames.contains(&explosion.ring_frames) {
            bad(
                "explosion.ring_frames",
                format!(
                    "{} : doit être dans [1, {MAX_PRESENTATION_FRAMES}]",
                    explosion.ring_frames
                ),
            );
        }
        if explosion.shake_frames > MAX_PRESENTATION_FRAMES {
            bad(
                "explosion.shake_frames",
                format!(
                    "{} : doit être dans [0, {MAX_PRESENTATION_FRAMES}]",
                    explosion.shake_frames
                ),
            );
        }
        if explosion.shake_amplitude < 0.0 {
            bad(
                "explosion.shake_amplitude",
                format!("{} : doit être >= 0", explosion.shake_amplitude),
            );
        }
        let (r, g, b, a) = explosion.color;
        for (c, v) in [("r", r), ("g", g), ("b", b), ("a", a)] {
            if !unit.contains(&v) {
                bad(
                    &format!("explosion.color.{c}"),
                    format!("{v} : doit être dans [0, 1]"),
                );
            }
        }
        let boss = &self.boss_end;
        if boss.freeze_frames > MAX_PRESENTATION_FRAMES {
            bad(
                "boss_end.freeze_frames",
                format!(
                    "{} : doit être dans [0, {MAX_PRESENTATION_FRAMES}]",
                    boss.freeze_frames
                ),
            );
        }
        if boss.enabled && boss.freeze_frames == 0 {
            bad(
                "boss_end.freeze_frames",
                "0 : doit être > 0 (désactiver par enabled: false)".into(),
            );
        }
        if !frames.contains(&boss.zoom_frames) {
            bad(
                "boss_end.zoom_frames",
                format!(
                    "{} : doit être dans [1, {MAX_PRESENTATION_FRAMES}]",
                    boss.zoom_frames
                ),
            );
        }
        if !(BOSS_ZOOM_RANGE.0..=BOSS_ZOOM_RANGE.1).contains(&boss.zoom) {
            bad(
                "boss_end.zoom",
                format!(
                    "{} : doit être dans [{}, {}]",
                    boss.zoom, BOSS_ZOOM_RANGE.0, BOSS_ZOOM_RANGE.1
                ),
            );
        }
        if !unit.contains(&boss.flash_alpha) {
            bad(
                "boss_end.flash_alpha",
                format!("{} : doit être dans [0, 1]", boss.flash_alpha),
            );
        }
        let (r, g, b) = boss.flash_color;
        for (c, v) in [("r", r), ("g", g), ("b", b)] {
            if !unit.contains(&v) {
                bad(
                    &format!("boss_end.flash_color.{c}"),
                    format!("{v} : doit être dans [0, 1]"),
                );
            }
        }
        if let Some((r, g, b, a)) = self.pattern_telegraph.color {
            for (c, v) in [("r", r), ("g", g), ("b", b), ("a", a)] {
                if !unit.contains(&v) {
                    bad(
                        &format!("pattern_telegraph.color.{c}"),
                        format!("{v} : doit être dans [0, 1]"),
                    );
                }
            }
        }
    }
}

fn check_flash(prefix: &str, flash: &HitFlashConfig, problems: &mut Vec<FeedbackProblem>) {
    if flash.frames == 0 {
        problems.push(FeedbackProblem {
            field: format!("{prefix}.frames"),
            message: "0 : doit être > 0".into(),
        });
    }
    let (r, g, b) = flash.color;
    for (component, value) in [("r", r), ("g", g), ("b", b)] {
        if !(0.0..=MAX_FLASH_COLOR).contains(&value) {
            problems.push(FeedbackProblem {
                field: format!("{prefix}.color.{component}"),
                message: format!("{value} : doit être dans [0, {MAX_FLASH_COLOR}]"),
            });
        }
    }
}

fn check_shake(prefix: &str, shake: &ShakeConfig, problems: &mut Vec<FeedbackProblem>) {
    if shake.frames == 0 {
        problems.push(FeedbackProblem {
            field: format!("{prefix}.frames"),
            message: "0 : doit être > 0".into(),
        });
    }
    if shake.amplitude < 0.0 {
        problems.push(FeedbackProblem {
            field: format!("{prefix}.amplitude"),
            message: format!("{} : doit être >= 0", shake.amplitude),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> FeedbackSettings {
        ron::from_str(
            r#"(
                hit_flash: (frames: 4, color: (2.5, 2.5, 2.5)),
                shake: (frames: 8, amplitude: 12.0),
                hit_stop_frames: 2,
                by_kind: {
                    Explosion: (hit_stop_frames: Some(6), shake: Some((frames: 20, amplitude: 30.0))),
                },
                by_weapon: {
                    "fusil": (hit_stop_frames: Some(4)),
                },
            )"#,
        )
        .unwrap()
    }

    #[test]
    fn arme_puis_genre_puis_defaut() {
        let s = settings();
        let defaut = s.resolve(&DamageKind::Physical, None);
        assert_eq!(defaut.hit_stop_frames, 2);
        assert_eq!(defaut.shake.amplitude, 12.0);
        let explosion = s.resolve(&DamageKind::Explosion, None);
        assert_eq!(explosion.hit_stop_frames, 6);
        assert_eq!(explosion.shake.frames, 20);
        // L'arme gagne pour le hit stop, le genre reste pour la secousse (champ absent).
        let fusil_explosif = s.resolve(&DamageKind::Explosion, Some("fusil"));
        assert_eq!(fusil_explosif.hit_stop_frames, 4);
        assert_eq!(fusil_explosif.shake.frames, 20);
        assert_eq!(fusil_explosif.hit_flash, s.hit_flash);
        assert!(fusil_explosif.shake_overridden);
        assert!(!defaut.shake_overridden);
        // Arme inconnue : ignorée.
        assert_eq!(s.resolve(&DamageKind::Physical, Some("autre")), defaut);
    }

    #[test]
    fn champs_optionnels_absents_de_l_ancien_format() {
        let s: FeedbackSettings = ron::from_str(
            r#"(hit_flash: (frames: 4, color: (1.0, 1.0, 1.0)), shake: (frames: 8, amplitude: 12.0), sounds: {})"#,
        )
        .unwrap();
        assert_eq!(s.hit_stop_frames, 0);
        assert!(!s.damage_numbers);
        assert_eq!(s.telegraph, TelegraphConfig::default());
        assert!(s.problems().is_empty());
    }

    #[test]
    fn valeurs_hors_plage() {
        let mut s = settings();
        s.hit_flash.color.0 = 5.0;
        s.telegraph.color.3 = 1.5;
        s.by_kind.get_mut(&DamageKind::Explosion).unwrap().shake = Some(ShakeConfig {
            frames: 0,
            amplitude: -1.0,
        });
        let fields: Vec<_> = s.problems().into_iter().map(|p| p.field).collect();
        assert_eq!(
            fields,
            [
                "hit_flash.color.r",
                "telegraph.color.a",
                "by_kind.Explosion.shake.frames",
                "by_kind.Explosion.shake.amplitude",
            ]
        );
    }

    #[test]
    fn t19_defauts_surs_et_bornes() {
        let s = settings();
        assert!(s.pattern_telegraph.enabled && s.blank.enabled && s.boss_end.enabled);
        assert!(s.problems().is_empty());
        let mut bad = s.clone();
        bad.blank.ring_frames = 0;
        bad.blank.flash_alpha = 2.0;
        bad.boss_end.freeze_frames = 0;
        bad.boss_end.zoom = 9.0;
        bad.boss_end.zoom_frames = 100_000;
        bad.pattern_telegraph.color = Some((0.0, 0.0, 0.0, 3.0));
        let fields: Vec<_> = bad.problems().into_iter().map(|p| p.field).collect();
        for field in [
            "blank.ring_frames",
            "blank.flash_alpha",
            "boss_end.freeze_frames",
            "boss_end.zoom",
            "boss_end.zoom_frames",
            "pattern_telegraph.color.a",
        ] {
            assert!(fields.iter().any(|f| f == field), "{field} : {fields:?}");
        }
        // Désactivé : un gel à 0 est accepté.
        let mut off = s;
        off.boss_end.enabled = false;
        off.boss_end.freeze_frames = 0;
        assert!(off.problems().is_empty());
    }

    #[test]
    fn volume_hors_plage_signale() {
        let mut s = settings();
        s.volumes.insert("hit".into(), -6.0);
        s.volumes.insert("door".into(), 12.0);
        assert!(s.problems().is_empty(), "{:?}", s.problems());
        s.volumes.insert("hurt".into(), 20.0);
        s.volumes.insert("dash".into(), -80.0);
        let fields: Vec<_> = s.problems().into_iter().map(|p| p.field).collect();
        assert_eq!(fields, ["volumes.dash", "volumes.hurt"]);
    }
}
