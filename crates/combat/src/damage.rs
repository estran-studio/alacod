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

/// Dégât après armure (D61, formule de League of Legends) : `dégât × 100 / (100 + armure)`, soit
/// 100 d'armure = moitié des dégâts. En `Fixed`. Armure négative bornée à 0 ; **armure 0 =
/// identité exacte** (le montant est rendu tel quel : aucun arrondi, aucune trace ne bouge).
/// `DamageKind::True` ignore l'armure (à tester par l'appelant : cette fonction est pure).
pub fn after_armor(amount: Fixed, armor: Fixed) -> Fixed {
    if armor <= Fixed::ZERO {
        return amount;
    }
    let hundred = Fixed::from_num(100);
    amount.saturating_mul(hundred) / hundred.saturating_add(armor)
}

/// Ce qu'une frappe devient après les couches de défense de la cible ([`apply_layers`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layered {
    /// Un point de bouclier a absorbé le coup entier : rien ne passe, pas de dégât réel.
    ShieldAbsorbed,
    /// Le coup passe le bouclier. `hit` : le dégât après armure (base des i-frames de
    /// `hit_iframes`, de `HitLanded` et de `HitCount` : le coup a eu lieu même si des cœurs d'âme
    /// l'ont absorbé) ; `to_health` : ce qui reste après `ExtraHealth`, retiré aux PV.
    Through { hit: Fixed, to_health: Fixed },
}

/// Couches de défense d'une cible, dans l'ordre retenu (M3-T3, `docs/conventions.md` §68) :
/// **bouclier** (un point = un coup entier) → **armure** (proportionnelle) → **`ExtraHealth`**
/// (point par point) → **PV**. `DamageKind::True` ignore le bouclier et l'armure (ce sont des
/// atténuations) mais **pas** `ExtraHealth` (c'est de la vie : une chute en fosse ou des piques
/// entament d'abord les cœurs d'âme, comme dans Isaac).
///
/// Pure : `shield` et `soul` sont les valeurs de la cible (0 sans le composant) et sont mis à jour.
/// `amount` est le dégât après `resolve_damage` (> 0). Sans bouclier, avec une armure ≤ 0 et sans
/// `soul`, `to_health == hit == amount` exactement (aucune trace ne bouge).
pub fn apply_layers(
    kind: &DamageKind,
    amount: Fixed,
    shield: &mut u32,
    armor: Fixed,
    soul: &mut Fixed,
) -> Layered {
    let true_damage = *kind == DamageKind::True;
    if !true_damage && *shield > 0 {
        *shield -= 1;
        return Layered::ShieldAbsorbed;
    }
    let hit = if true_damage {
        amount
    } else {
        after_armor(amount, armor)
    };
    let absorbed = hit.min(*soul).max(Fixed::ZERO);
    *soul -= absorbed;
    Layered::Through {
        hit,
        to_health: hit - absorbed,
    }
}

#[cfg(test)]
mod layer_tests {
    use super::*;

    fn fx(value: i32) -> Fixed {
        Fixed::from_num(value)
    }

    #[test]
    fn no_layer_is_the_identity() {
        let (mut shield, mut soul) = (0, Fixed::ZERO);
        let out = apply_layers(&DamageKind::Physical, fx(7), &mut shield, fx(0), &mut soul);
        assert_eq!(
            out,
            Layered::Through {
                hit: fx(7),
                to_health: fx(7)
            }
        );
        assert_eq!((shield, soul), (0, Fixed::ZERO));
    }

    #[test]
    fn shield_goes_first_and_leaves_armor_and_soul_untouched() {
        let (mut shield, mut soul) = (1, fx(4));
        let out = apply_layers(
            &DamageKind::Physical,
            fx(8),
            &mut shield,
            fx(100),
            &mut soul,
        );
        assert_eq!(out, Layered::ShieldAbsorbed);
        assert_eq!((shield, soul), (0, fx(4)));
    }

    #[test]
    fn armor_comes_before_soul() {
        // 8 dégâts, 100 d'armure : 4 passent l'armure, 3 points d'âme les absorbent, 1 reste.
        let (mut shield, mut soul) = (0, fx(3));
        let out = apply_layers(
            &DamageKind::Physical,
            fx(8),
            &mut shield,
            fx(100),
            &mut soul,
        );
        assert_eq!(
            out,
            Layered::Through {
                hit: fx(4),
                to_health: fx(1)
            }
        );
        assert_eq!(soul, Fixed::ZERO);
    }

    #[test]
    fn soul_absorbs_point_by_point_before_health() {
        let (mut shield, mut soul) = (0, fx(5));
        // Un coup de 2 entame 2 des 5 points, rien aux PV ; le coup de 4 suivant en prend 3 puis 1 aux PV.
        let first = apply_layers(&DamageKind::Physical, fx(2), &mut shield, fx(0), &mut soul);
        assert_eq!(
            first,
            Layered::Through {
                hit: fx(2),
                to_health: Fixed::ZERO
            }
        );
        assert_eq!(soul, fx(3));
        let second = apply_layers(&DamageKind::Physical, fx(4), &mut shield, fx(0), &mut soul);
        assert_eq!(
            second,
            Layered::Through {
                hit: fx(4),
                to_health: fx(1)
            }
        );
        assert_eq!(soul, Fixed::ZERO);
    }

    #[test]
    fn true_damage_skips_shield_and_armor_but_not_soul() {
        let (mut shield, mut soul) = (1, fx(2));
        let out = apply_layers(&DamageKind::True, fx(5), &mut shield, fx(100), &mut soul);
        assert_eq!(
            out,
            Layered::Through {
                hit: fx(5),
                to_health: fx(3)
            }
        );
        assert_eq!((shield, soul), (1, Fixed::ZERO));
    }

    #[test]
    fn a_hit_fully_absorbed_by_soul_is_still_a_hit() {
        let (mut shield, mut soul) = (0, fx(9));
        match apply_layers(&DamageKind::Physical, fx(3), &mut shield, fx(0), &mut soul) {
            Layered::Through { hit, to_health } => {
                assert_eq!(hit, fx(3));
                assert_eq!(to_health, Fixed::ZERO);
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(soul, fx(6));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn armor_formula_matches_league() {
        let fx = Fixed::from_num;
        assert_eq!(after_armor(fx(10), fx(0)), fx(10));
        assert_eq!(after_armor(fx(10), fx(-30)), fx(10));
        assert_eq!(after_armor(fx(10), fx(100)), fx(5));
        assert_eq!(after_armor(fx(30), fx(200)), fx(10));
        assert_eq!(after_armor(fx(10), fx(100)).to_num::<i32>(), 5);
    }

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
