//! [`Action`] : vocabulaire fermé des effets qu'un power-up peut appliquer au ramassage
//! (T2.5). Sémantique CoD Zombies (Insta-Kill, Double Points, Max Ammo, Carpenter, Nuke) :
//! un power-up ramassé s'applique à **tous les joueurs** de la partie, jamais au seul
//! joueur qui l'a ramassé — aucun de ces cinq power-ups n'est individuel dans le jeu de
//! référence. `game::powerups::apply_powerup_actions_system` applique cette règle : il
//! itère tous les joueurs pour chaque action, quel que soit le joueur qui a déclenché le
//! ramassage.

use bevy_fixed::fixed_math::Fixed;
use serde::{Deserialize, Serialize};
use sim_core::modifier::{Modifier, ModifierOp, ModifierSource};
use sim_core::stats::StatId;

/// `StatId::Custom` sous laquelle [`Action::CurrencyMultiplier`] pose son modificateur
/// (`ModifierOp::Mul`, lu par `game::economy::award_points_system` via `StatReader`, base
/// neutre 1.0 — voir `docs/conventions.md` §9, `sim_core::modifier::resolve` : aucun
/// modificateur actif ne multiplie même pas par 1). Pas une vraie stat de personnage
/// (mouvement, dégâts...) : un simple point d'ancrage pour réutiliser `Modifiers`/
/// `StatReader` déjà en place plutôt que d'inventer un composant dédié (voir la doc du
/// module `game::powerups`, décision « pas de ressource globale toujours présente »).
pub const CURRENCY_MULTIPLIER_STAT: &str = "powerup_currency_multiplier";

/// Une action appliquée par un power-up ramassé. Liste fermée et minimale (T2.5, graine de
/// C1 — `docs/plan-engine.md` §5) : un power-up futur qui a besoin d'un autre effet ajoute
/// une variante ici, jamais un type de contenu séparé.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Action {
    /// Modificateur de stat temporaire posé sur chaque joueur (ex. Insta-Kill : `stat:
    /// Damage, op: Set, value: "100.0"`). `frames` est une **durée** relative à la frame de
    /// ramassage (jamais une frame absolue dans le RON, pour que le contenu ne dépende pas
    /// du moment où le power-up est ramassé) ; voir [`Self::as_modifier`] pour la
    /// conversion déterministe en `until: Some(frame)`.
    TimedModifier {
        stat: StatId,
        op: ModifierOp,
        value: Fixed,
        frames: u32,
    },
    /// Remplit le chargeur de chaque arme portée par chaque joueur et recharge sa réserve
    /// de munitions jusqu'à la contribution par défaut de ses armes (Max Ammo). Résolu par
    /// `game::powerups` (pas de forme pure possible : dépend de `WeaponInventory`/
    /// `AmmoReserves`, inconnus de ce crate).
    RefillAmmo,
    /// Répare toutes les fenêtres de la carte à santé maximale (Carpenter).
    RepairAllWindows,
    /// Tue tous les ennemis actuellement en vie (Nuke). Voir `game::powerups` pour la
    /// portée exacte (équipe `Enemies`, pas seulement les ennemis de vague : le nom vient
    /// de la spec T2.5, la portée réelle couvre aussi un ennemi de laboratoire du testbed).
    KillAllWaveEnemies,
    /// Multiplie les points gagnés par chaque joueur pendant `frames` (Double Points).
    /// `factor` s'applique par `ModifierOp::Mul` sur [`CURRENCY_MULTIPLIER_STAT`] — deux
    /// `CurrencyMultiplier` actifs en même temps se cumulent multiplicativement (limitation
    /// documentée, v0 minimal : pas de règle de non-stacking).
    CurrencyMultiplier { factor: Fixed, frames: u32 },
}

impl Action {
    /// Modificateur déterministe posé par cette action sur **chaque** joueur, si elle est
    /// modifier-based (`TimedModifier`/`CurrencyMultiplier`) ; `None` pour les trois autres
    /// variantes, résolues directement par `game::powerups` (pas de représentation en
    /// `Modifier` possible : `RefillAmmo`/`RepairAllWindows`/`KillAllWaveEnemies` mutent un
    /// état qui n'est pas une stat).
    ///
    /// C'est l'« applicateur déterministe » de la tâche T2.5 : pure, sans ECS, donc
    /// testable directement (voir les tests de ce module) — `frame` est la frame de
    /// ramassage, `source` identifie l'origine du modificateur (typiquement
    /// `ModifierSource::Named("powerup:<id>:<net_id du joueur>")`, voir
    /// `game::powerups::apply_powerup_actions_system`) pour que
    /// `Modifiers::remove_by_source` puisse un jour le retirer explicitement si un chantier
    /// futur en a besoin (v0 : laissé expirer par `stats::expire_modifiers_system`, jamais
    /// retiré à la main).
    pub fn as_modifier(&self, frame: u32, source: ModifierSource) -> Option<Modifier> {
        match self {
            Action::TimedModifier {
                stat,
                op,
                value,
                frames,
            } => Some(Modifier {
                stat: stat.clone(),
                op: *op,
                value: *value,
                source,
                until: Some(frame.saturating_add(*frames)),
            }),
            Action::CurrencyMultiplier { factor, frames } => Some(Modifier {
                stat: StatId::Custom(CURRENCY_MULTIPLIER_STAT.to_string()),
                op: ModifierOp::Mul,
                value: *factor,
                source,
                until: Some(frame.saturating_add(*frames)),
            }),
            Action::RefillAmmo | Action::RepairAllWindows | Action::KillAllWaveEnemies => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fixed::fixed_math::new as fx;

    fn source() -> ModifierSource {
        ModifierSource::Named("powerup:test:1".into())
    }

    #[test]
    fn timed_modifier_until_is_pickup_frame_plus_frames() {
        let action = Action::TimedModifier {
            stat: StatId::Damage,
            op: ModifierOp::Set,
            value: fx(100.0),
            frames: 1800,
        };
        let modifier = action.as_modifier(120, source()).expect("modifier attendu");
        assert_eq!(modifier.stat, StatId::Damage);
        assert_eq!(modifier.op, ModifierOp::Set);
        assert_eq!(modifier.value, fx(100.0));
        assert_eq!(modifier.until, Some(1920));
    }

    #[test]
    fn timed_modifier_until_saturates_instead_of_overflowing() {
        let action = Action::TimedModifier {
            stat: StatId::Damage,
            op: ModifierOp::Set,
            value: fx(1.0),
            frames: u32::MAX,
        };
        let modifier = action.as_modifier(10, source()).unwrap();
        assert_eq!(modifier.until, Some(u32::MAX));
    }

    #[test]
    fn currency_multiplier_targets_dedicated_custom_stat() {
        let action = Action::CurrencyMultiplier {
            factor: fx(2.0),
            frames: 1800,
        };
        let modifier = action.as_modifier(60, source()).expect("modifier attendu");
        assert_eq!(
            modifier.stat,
            StatId::Custom(CURRENCY_MULTIPLIER_STAT.to_string())
        );
        assert_eq!(modifier.op, ModifierOp::Mul);
        assert_eq!(modifier.value, fx(2.0));
        assert_eq!(modifier.until, Some(1860));
    }

    #[test]
    fn non_modifier_actions_have_no_pure_form() {
        assert!(Action::RefillAmmo.as_modifier(0, source()).is_none());
        assert!(Action::RepairAllWindows.as_modifier(0, source()).is_none());
        assert!(Action::KillAllWaveEnemies.as_modifier(0, source()).is_none());
    }

    #[test]
    fn action_round_trips_through_ron() {
        let actions = vec![
            Action::TimedModifier {
                stat: StatId::Damage,
                op: ModifierOp::Set,
                value: fx(100.0),
                frames: 1800,
            },
            Action::RefillAmmo,
            Action::RepairAllWindows,
            Action::KillAllWaveEnemies,
            Action::CurrencyMultiplier {
                factor: fx(2.0),
                frames: 1800,
            },
        ];
        let text = ron::to_string(&actions).expect("sérialisation");
        let parsed: Vec<Action> = ron::from_str(&text).expect("désérialisation");
        assert_eq!(actions, parsed);
    }

    /// Une référence de stat qui ne correspond à aucune variante connue de `StatId` (et
    /// n'est pas écrite `Custom("...")`) échoue au chargement RON — comme `StatId` partout
    /// ailleurs dans le projet (voir `content::registry::PerkModifierSchema`). C'est ce qui
    /// permet à `content::lint` de refuser une référence de stat inconnue dans
    /// `items/powerups.ron` sans règle de lint dédiée (voir la fixture
    /// `powerup_unknown_stat`).
    #[test]
    fn unknown_bare_stat_identifier_fails_to_parse() {
        let ron_text = r#"TimedModifier(stat: PasUneStatConnue, op: Set, value: "1.0", frames: 60)"#;
        assert!(ron::from_str::<Action>(ron_text).is_err());
    }
}
