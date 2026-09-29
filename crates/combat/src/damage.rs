//! Résolution des dégâts : équipe, tir ami, tags, résistances, immunités, invulnérabilité.
//! `DamageEvent`/`DamageKind`/`FriendlyFire` (le contrat) viennent de `sim_core::damage`
//! (T0.2) ; ce module ajoute les règles (T1.1, chantier B1 « Équipes et dégâts »).

use std::collections::BTreeMap;

use bevy::prelude::Component;
use bevy_fixed::fixed_math::Fixed;
use serde::{Deserialize, Serialize};
use sim_core::damage::{DamageKind, FriendlyFire};
use sim_core::tag::{Tag, Tags};
use sim_core::team::Team;

use crate::team::team_allows_hit;

/// Résistances et immunités d'un personnage, posées en composant à la création
/// (`CharacterConfig::immune_to`/`resistances` → `character::create::create_character`).
/// Composant statique, hors rollback (même justification que `Tags`/`Team` : jamais muté en
/// T1.1).
#[derive(Component, Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Defenses {
    /// Un dégât dont `DamageEvent::tags` contient au moins un de ces tags est entièrement
    /// bloqué (sauf `DamageKind::True`, qui ignore les défenses).
    #[serde(default)]
    pub immune_to: Tags,
    /// Multiplicateur appliqué au montant pour chaque tag de `DamageEvent::tags` trouvé
    /// ici (composés entre eux si plusieurs correspondent — voir `resolve_damage`).
    #[serde(default)]
    pub resistances: BTreeMap<Tag, Fixed>,
}

/// Résout un dégât : équipe + politique de tir ami (bloque tout ou rien), puis, sauf
/// `DamageKind::True`, invulnérabilité et immunité par tag (bloquent tout), puis
/// résistances par tag (réduisent le montant). Fonction pure, testée directement (aucune
/// dépendance ECS).
///
/// Renvoie `None` si aucun dégât ne doit être appliqué (équipe/politique, cible `Neutral`,
/// invulnérable, immunisée, ou entièrement résisté — montant final `<= 0`) ; `Some(montant)`
/// sinon, avec `montant > 0`.
///
/// Ordre des vérifications : équipe d'abord (une règle de conception — qui a le droit
/// d'être visé — jamais contournée, même par un dégât `True`), puis les défenses de la
/// cible (`True` les ignore toutes : résistances, immunités, invulnérabilité).
///
/// `source_tags` sert deux rôles à la fois (voir la doc de `DamageEvent::tags`) : la
/// polarité `cursed` pour `FriendlyFire::Cursed`, et le genre d'attaque (`bullet`, `melee`,
/// `zombie`...) pour les résistances/immunités de la cible.
pub fn resolve_damage(
    source_team: Team,
    target_team: Team,
    weapon_policy: FriendlyFire,
    source_tags: &Tags,
    target_defenses: &Defenses,
    kind: DamageKind,
    amount: Fixed,
    target_invulnerable: bool,
) -> Option<Fixed> {
    if !team_allows_hit(source_team, target_team, weapon_policy, source_tags) {
        return None;
    }
    // Neutre : bloque le tir (voir `team_allows_hit`) mais n'est jamais blessée.
    if target_team == Team::Neutral {
        return None;
    }

    let true_damage = kind == DamageKind::True;

    if !true_damage {
        if target_invulnerable {
            return None;
        }
        if source_tags
            .iter()
            .any(|tag| target_defenses.immune_to.has(tag))
        {
            return None;
        }
    }

    let mut result = amount;
    if !true_damage {
        for tag in source_tags.iter() {
            if let Some(multiplier) = target_defenses.resistances.get(tag) {
                result = result.saturating_mul(*multiplier);
            }
        }
    }

    if result <= Fixed::ZERO {
        return None;
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tags(names: &[&str]) -> Tags {
        Tags::parse(names.iter().copied())
    }

    fn defenses(immune_to: &[&str], resistances: &[(&str, f32)]) -> Defenses {
        Defenses {
            immune_to: tags(immune_to),
            resistances: resistances
                .iter()
                .map(|(name, mult)| (Tag::new(*name), Fixed::from_num(*mult)))
                .collect(),
        }
    }

    const TEN: Fixed = Fixed::from_bits(10 << 16);

    #[test]
    fn cross_team_hit_deals_full_damage() {
        let result = resolve_damage(
            Team::Players,
            Team::Enemies,
            FriendlyFire::Never,
            &tags(&["bullet"]),
            &Defenses::default(),
            DamageKind::Physical,
            TEN,
            false,
        );
        assert_eq!(result, Some(TEN));
    }

    #[test]
    fn friendly_fire_never_blocks_same_side() {
        let result = resolve_damage(
            Team::Players,
            Team::Players,
            FriendlyFire::Never,
            &tags(&["bullet"]),
            &Defenses::default(),
            DamageKind::Physical,
            TEN,
            false,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn friendly_fire_cursed_allows_only_with_tag() {
        let without_tag = resolve_damage(
            Team::Players,
            Team::Players,
            FriendlyFire::Cursed,
            &tags(&["bullet"]),
            &Defenses::default(),
            DamageKind::Physical,
            TEN,
            false,
        );
        assert_eq!(without_tag, None);

        let with_tag = resolve_damage(
            Team::Players,
            Team::Players,
            FriendlyFire::Cursed,
            &tags(&["bullet", "cursed"]),
            &Defenses::default(),
            DamageKind::Physical,
            TEN,
            false,
        );
        assert_eq!(with_tag, Some(TEN));
    }

    #[test]
    fn neutral_target_never_takes_damage() {
        for policy in [
            FriendlyFire::Never,
            FriendlyFire::Always,
            FriendlyFire::Cursed,
        ] {
            let result = resolve_damage(
                Team::Players,
                Team::Neutral,
                policy,
                &tags(&["bullet"]),
                &Defenses::default(),
                DamageKind::True,
                TEN,
                false,
            );
            assert_eq!(result, None, "policy {policy:?}");
        }
    }

    #[test]
    fn invulnerable_blocks_physical_damage() {
        let result = resolve_damage(
            Team::Enemies,
            Team::Players,
            FriendlyFire::Never,
            &tags(&["melee"]),
            &Defenses::default(),
            DamageKind::Physical,
            TEN,
            true,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn true_damage_ignores_invulnerability() {
        let result = resolve_damage(
            Team::Enemies,
            Team::Players,
            FriendlyFire::Never,
            &tags(&["melee"]),
            &Defenses::default(),
            DamageKind::True,
            TEN,
            true,
        );
        assert_eq!(result, Some(TEN));
    }

    #[test]
    fn immune_to_tag_blocks_matching_damage() {
        let result = resolve_damage(
            Team::Enemies,
            Team::Players,
            FriendlyFire::Never,
            &tags(&["bullet"]),
            &defenses(&["bullet"], &[]),
            DamageKind::Physical,
            TEN,
            false,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn immune_to_tag_does_not_block_a_different_tag() {
        let result = resolve_damage(
            Team::Enemies,
            Team::Players,
            FriendlyFire::Never,
            &tags(&["melee"]),
            &defenses(&["bullet"], &[]),
            DamageKind::Physical,
            TEN,
            false,
        );
        assert_eq!(result, Some(TEN));
    }

    #[test]
    fn true_damage_ignores_immunity() {
        let result = resolve_damage(
            Team::Enemies,
            Team::Players,
            FriendlyFire::Never,
            &tags(&["bullet"]),
            &defenses(&["bullet"], &[]),
            DamageKind::True,
            TEN,
            false,
        );
        assert_eq!(result, Some(TEN));
    }

    #[test]
    fn resistance_multiplies_the_amount() {
        let result = resolve_damage(
            Team::Enemies,
            Team::Players,
            FriendlyFire::Never,
            &tags(&["bullet"]),
            &defenses(&[], &[("bullet", 0.5)]),
            DamageKind::Physical,
            TEN,
            false,
        );
        assert_eq!(result, Some(Fixed::from_num(5.0)));
    }

    #[test]
    fn resistance_zero_is_equivalent_to_immunity() {
        let result = resolve_damage(
            Team::Enemies,
            Team::Players,
            FriendlyFire::Never,
            &tags(&["bullet"]),
            &defenses(&[], &[("bullet", 0.0)]),
            DamageKind::Physical,
            TEN,
            false,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn multiple_matching_resistances_compose_multiplicatively() {
        let result = resolve_damage(
            Team::Enemies,
            Team::Players,
            FriendlyFire::Never,
            &tags(&["bullet", "cold"]),
            &defenses(&[], &[("bullet", 0.5), ("cold", 0.5)]),
            DamageKind::Physical,
            TEN,
            false,
        );
        assert_eq!(result, Some(Fixed::from_num(2.5)));
    }

    #[test]
    fn true_damage_ignores_resistances() {
        let result = resolve_damage(
            Team::Enemies,
            Team::Players,
            FriendlyFire::Never,
            &tags(&["bullet"]),
            &defenses(&[], &[("bullet", 0.0)]),
            DamageKind::True,
            TEN,
            false,
        );
        assert_eq!(result, Some(TEN));
    }

    #[test]
    fn true_damage_still_respects_team_and_friendly_fire() {
        let result = resolve_damage(
            Team::Players,
            Team::Players,
            FriendlyFire::Never,
            &tags(&["bullet"]),
            &Defenses::default(),
            DamageKind::True,
            TEN,
            false,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn zombie_immune_to_bullet_still_dies_to_melee() {
        let zombie_defenses = defenses(&["bullet"], &[]);
        let shot = resolve_damage(
            Team::Players,
            Team::Enemies,
            FriendlyFire::Never,
            &tags(&["bullet"]),
            &zombie_defenses,
            DamageKind::Physical,
            TEN,
            false,
        );
        assert_eq!(shot, None);

        let meleed = resolve_damage(
            Team::Players,
            Team::Enemies,
            FriendlyFire::Never,
            &tags(&["melee"]),
            &zombie_defenses,
            DamageKind::Physical,
            TEN,
            false,
        );
        assert_eq!(meleed, Some(TEN));
    }
}
