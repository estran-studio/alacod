//! Accomplissements (M3-T0e, `docs/conventions.md` §64) : une condition sur la fin d'une run qui
//! débloque des [`items::UnlockDef`] du jeu (M2-T12, §51).
//!
//! - [`AchievementDef`] : un accomplissement de `achievements/<id>.ron` (kind `Achievement`,
//!   id = nom de fichier) : `when` (toutes les conditions, ET) et `unlocks` (ids de `Unlock`).
//! - [`RunFacts`] : ce que la fin d'une run sait, réuni **hors simulation** : le [`RunSummary`]
//!   (étage atteint, issue), les boss vaincus et les objets tenus. Ces deux derniers n'entrent
//!   pas dans `RunSummary` : c'est un état de la partie (rollback, `Copy`, haché) que la
//!   simulation n'a pas à porter pour un accomplissement qu'elle ne lit jamais.
//! - [`achieved`] : règle pure, sans effet de bord.
//!
//! Évalué **hors `GgrsSchedule`**, à la fin de la run, comme l'écriture du profil
//! (`game::profile`) : ce module ne lit ni n'écrit rien. L'exécution (réunir [`RunFacts`],
//! ajouter les `unlocks` au profil) est M3-T10.

use std::collections::BTreeSet;

use run::{RunEnd, RunSummary};
use serde::{Deserialize, Serialize};

/// Une condition sur la fin de run.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum AchievementCondition {
    /// La run s'est terminée par une victoire (`RunEnd::Victory`).
    Victory,
    /// Étage atteint `>= n`, même index que [`RunSummary::floor_reached`] et que l'attente
    /// `FloorIndex` : `0` = le premier étage de la séquence.
    FloorAtLeast(u32),
    /// Le boss `id` (un personnage du jeu qui porte `boss:`) a été vaincu pendant la run.
    BossDefeated(String),
    /// Un joueur de la run tenait l'objet `id` (fichier de `items/`) à la fin.
    ItemHeld(String),
}

/// Un accomplissement, `achievements/<id>.ron` (kind `Achievement`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AchievementDef {
    /// Nom affiché (écran de fin, HUD) ; vide : l'identifiant (M3-T10).
    #[serde(default)]
    pub name: String,
    /// Conditions cumulatives (ET) ; au moins une (sans condition, toute run l'accomplirait).
    pub when: Vec<AchievementCondition>,
    /// Déblocages accordés (ids de `unlocks/<id>.ron`), au moins un.
    pub unlocks: Vec<String>,
}

/// Les accomplissements du jeu par identifiant (hors rollback), posés au chargement de la carte
/// depuis le registre de contenu (`map_ldtk::loader::achievement_table`). Lus par
/// `game::profile` à la fin de la run, jamais par la simulation.
#[derive(bevy::prelude::Resource, Clone, Debug, Default)]
pub struct Achievements(pub std::collections::BTreeMap<String, AchievementDef>);

impl AchievementDef {
    /// Nom affiché de `id` : `name`, sinon l'identifiant.
    pub fn display_name<'a>(&'a self, id: &'a str) -> &'a str {
        if self.name.is_empty() {
            id
        } else {
            &self.name
        }
    }
}

/// Les faits d'une run terminée que les accomplissements lisent (voir la doc du module).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunFacts {
    pub summary: RunSummary,
    /// Personnages-boss vaincus pendant la run.
    pub bosses_defeated: BTreeSet<String>,
    /// Objets tenus à la fin par au moins un joueur de la run.
    pub items_held: BTreeSet<String>,
}

impl AchievementCondition {
    fn holds(&self, facts: &RunFacts) -> bool {
        match self {
            Self::Victory => facts.summary.outcome == RunEnd::Victory,
            Self::FloorAtLeast(n) => facts.summary.floor_reached >= *n,
            Self::BossDefeated(id) => facts.bosses_defeated.contains(id),
            Self::ItemHeld(id) => facts.items_held.contains(id),
        }
    }
}

/// `def` est accompli : toutes ses conditions tiennent (et il en a au moins une : un
/// accomplissement sans condition n'est jamais accompli, le lint le refuse).
pub fn achieved(facts: &RunFacts, def: &AchievementDef) -> bool {
    !def.when.is_empty() && def.when.iter().all(|condition| condition.holds(facts))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(outcome: RunEnd, floor_reached: u32) -> RunSummary {
        RunSummary {
            wave_reached: 0,
            kills: 0,
            points_total: 0,
            frames: 100,
            outcome,
            floor_reached,
        }
    }

    fn facts(outcome: RunEnd, floor: u32, bosses: &[&str], items: &[&str]) -> RunFacts {
        RunFacts {
            summary: summary(outcome, floor),
            bosses_defeated: bosses.iter().map(|id| id.to_string()).collect(),
            items_held: items.iter().map(|id| id.to_string()).collect(),
        }
    }

    fn def(when: Vec<AchievementCondition>) -> AchievementDef {
        AchievementDef {
            name: String::new(),
            when,
            unlocks: vec!["a".into()],
        }
    }

    #[test]
    fn victory_needs_a_victory_not_a_defeat_or_an_abandon() {
        let def = def(vec![AchievementCondition::Victory]);
        assert!(achieved(&facts(RunEnd::Victory, 0, &[], &[]), &def));
        assert!(!achieved(&facts(RunEnd::Defeat, 5, &[], &[]), &def));
        assert!(!achieved(&facts(RunEnd::Abandon, 5, &[], &[]), &def));
    }

    #[test]
    fn floor_threshold_is_inclusive_and_counts_from_zero() {
        let def = def(vec![AchievementCondition::FloorAtLeast(2)]);
        assert!(!achieved(&facts(RunEnd::Defeat, 1, &[], &[]), &def));
        assert!(achieved(&facts(RunEnd::Defeat, 2, &[], &[]), &def));
        assert!(achieved(&facts(RunEnd::Defeat, 3, &[], &[]), &def));
        // Une défaite compte : l'étage atteint ne dépend pas de l'issue.
        let zero = self::def(vec![AchievementCondition::FloorAtLeast(0)]);
        assert!(achieved(&facts(RunEnd::Defeat, 0, &[], &[]), &zero));
    }

    #[test]
    fn boss_and_item_conditions_look_up_their_own_id() {
        let boss = def(vec![AchievementCondition::BossDefeated("gardien".into())]);
        let item = def(vec![AchievementCondition::ItemHeld("lame".into())]);
        let got = facts(RunEnd::Defeat, 0, &["gardien"], &["lame"]);
        assert!(achieved(&got, &boss) && achieved(&got, &item));
        let other = facts(RunEnd::Defeat, 0, &["autre"], &["autre"]);
        assert!(!achieved(&other, &boss) && !achieved(&other, &item));
    }

    #[test]
    fn conditions_are_cumulative() {
        let def = def(vec![
            AchievementCondition::Victory,
            AchievementCondition::BossDefeated("gardien".into()),
            AchievementCondition::ItemHeld("lame".into()),
        ]);
        assert!(achieved(
            &facts(RunEnd::Victory, 1, &["gardien"], &["lame"]),
            &def
        ));
        assert!(!achieved(
            &facts(RunEnd::Victory, 1, &["gardien"], &[]),
            &def
        ));
        assert!(!achieved(
            &facts(RunEnd::Defeat, 1, &["gardien"], &["lame"]),
            &def
        ));
    }

    #[test]
    fn an_achievement_without_condition_is_never_achieved() {
        assert!(!achieved(
            &facts(RunEnd::Victory, 9, &["x"], &["y"]),
            &def(vec![])
        ));
    }

    #[test]
    fn achievement_files_parse_from_ron() {
        let def: AchievementDef = ron::from_str(
            r#"(when: [Victory, FloorAtLeast(1), BossDefeated("gardien"), ItemHeld("lame")], unlocks: ["x"])"#,
        )
        .expect("RON valide");
        assert_eq!(def.when.len(), 4);
        assert_eq!(def.unlocks, ["x"]);
    }
}
