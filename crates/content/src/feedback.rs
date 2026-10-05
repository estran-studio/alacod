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
    /// Surcharges par genre de dégât (`DamageKind`, ex. `Explosion`, `Custom("acide")`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub by_kind: BTreeMap<DamageKind, FeedbackOverride>,
    /// Surcharges par arme (nom d'arme à distance ou de corps à corps du jeu). Prioritaires
    /// sur `by_kind`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub by_weapon: BTreeMap<String, FeedbackOverride>,
    /// Sons (clés : `shot`, `reload`), chemins relatifs à `assets/`.
    #[serde(default)]
    pub sounds: BTreeMap<String, String>,
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
}
