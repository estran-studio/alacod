//! Modificateurs de stats et leur résolution déterministe.

use bevy::prelude::Component;
use bevy_fixed::fixed_math::{self, Fixed};
use serde::{Deserialize, Serialize};

use crate::stats::StatId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModifierOp {
    Add,
    /// Pourcentage additif : les `Pct` actifs s'additionnent entre eux puis s'appliquent en
    /// un seul facteur `(1 + Σ Pct)`, après les `Add` et avant les `Mul` (voir [`resolve`]).
    /// `0.5` = +50 % ; plusieurs `Pct` de `0.5` s'additionnent (+100 %, pas +125 %).
    Pct,
    Mul,
    Set,
}

/// Origine d'un modificateur : un nom stable (objet, statut, effet...) ou un identifiant
/// ordonné (ex. compteur de stacks). Sert à retrouver ou retirer un modificateur précis
/// sans dépendre d'un `Entity` (CLAUDE.md, règle 3 : les `Entity` diffèrent d'un client à
/// l'autre).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModifierSource {
    Named(String),
    Id(u64),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Modifier {
    pub stat: StatId,
    pub op: ModifierOp,
    pub value: Fixed,
    pub source: ModifierSource,
    /// Frame d'expiration : le modificateur est actif tant que `frame <= until`.
    /// `None` = permanent. Voir [`Modifier::is_expired`].
    pub until: Option<u32>,
}

impl Modifier {
    /// Un modificateur est expiré strictement après sa frame d'expiration : actif tant
    /// que `frame <= until` (`until == Some(frame)` est donc encore actif).
    pub fn is_expired(&self, frame: u32) -> bool {
        self.until.is_some_and(|until| until < frame)
    }
}

/// Liste ordonnée de modificateurs, dans l'ordre où ils ont été ajoutés (pas triée par
/// contenu). C'est cet ordre qui rend [`resolve`] déterministe pour les `Set` multiples
/// sur une même stat : construire `Modifiers` à partir d'une source non ordonnée
/// (`HashMap`) casserait ça (CLAUDE.md, règle 5).
///
/// `Component` (T1.2, chantier B2) : posé sur chaque personnage à sa création
/// (`character::create::create_character`, vide par défaut), enregistré en rollback par
/// `stats::StatsPlugin` (`app.rollback_and_trace::<Modifiers>()`) — sa valeur entre donc
/// dans le `Checksum` GGRS comparé par le synctest et la détection de desync p2p.
#[derive(Component, Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Modifiers(pub Vec<Modifier>);

impl Modifiers {
    pub fn push(&mut self, modifier: Modifier) {
        self.0.push(modifier);
    }

    /// Construit un [`Modifier`] depuis ses champs et l'ajoute (voir [`Self::push`]).
    /// Pratique pour les appelants qui composent un modificateur à la volée (statuts,
    /// perks, effets de salle) sans passer par la syntaxe struct complète.
    pub fn push_from(
        &mut self,
        source: ModifierSource,
        stat: StatId,
        op: ModifierOp,
        value: Fixed,
        until: Option<u32>,
    ) {
        self.push(Modifier {
            stat,
            op,
            value,
            source,
            until,
        });
    }

    /// Retire tous les modificateurs dont la source est exactement `source`. Sert à
    /// terminer un effet groupé sans connaître le détail des modificateurs posés : un
    /// statut qui expire ou une salle qu'on quitte appelle `remove_by_source` avec le même
    /// [`ModifierSource`] qu'il a donné à [`Self::push_from`]/[`Self::push`] en posant
    /// l'effet (fin par statut ou par salle — les chantiers futurs, statuts B3, salles E1,
    /// appellent cette méthode plutôt que de retirer les modificateurs un à un).
    pub fn remove_by_source(&mut self, source: &ModifierSource) {
        self.0.retain(|m| &m.source != source);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Modifier> {
        self.0.iter()
    }

    /// Retire les modificateurs expirés à `frame` (voir [`Modifier::is_expired`]).
    pub fn retain_active(&mut self, frame: u32) {
        self.0.retain(|m| !m.is_expired(frame));
    }
}

/// Résout la valeur finale d'une stat à partir de sa base et de ses modificateurs actifs
/// à `frame` : `(base + Σ Add) × (1 + Σ Pct) × Π Mul`, `Set` appliqué d'abord.
///
/// Ordre fixe et documenté (T0.2, étendu par T1.2 pour `Pct`), indépendant de l'ordre de
/// parcours de `modifiers` pour les `Add`/`Pct`/`Mul` (commutatifs), mais **pas** pour les
/// `Set` :
///
/// 1. `Set` : chaque modificateur `Set` rencontré remplace le résultat courant — **le
///    dernier de l'itération l'emporte**. Appeler avec un ordre stable (ex. l'ordre
///    d'insertion de [`Modifiers`]) si plusieurs `Set` sur la même stat doivent avoir un
///    gagnant déterministe entre clients.
/// 2. `Add` : somme de tous les `Add` actifs, ajoutée au résultat de l'étape 1.
/// 3. `Pct` : somme de tous les `Pct` actifs (pourcentage **additif** : deux `Pct` de
///    `0.5` font `+100 %`, pas `+125 %`), appliquée comme un facteur unique
///    `(1 + Σ Pct)` au résultat de l'étape 2. Aucun `Pct` actif : aucune multiplication
///    (pas même par `1`), pour garantir un résultat bit-à-bit identique à avant T1.2 quand
///    ce chantier n'est pas utilisé.
/// 4. `Mul` : produit de tous les `Mul` actifs (facteur multiplicatif direct, pas
///    `1 + valeur`), appliqué au résultat de l'étape 3.
///
/// Un modificateur expiré (`until < frame`, [`Modifier::is_expired`]) est ignoré. Tout en
/// fixed-point (`bevy_fixed::fixed_math`), jamais de `f32` (CLAUDE.md, règle 1) ; les
/// opérations saturent plutôt que de paniquer en cas de dépassement.
pub fn resolve<'a>(
    base: Fixed,
    modifiers: impl Iterator<Item = &'a Modifier>,
    frame: u32,
) -> Fixed {
    let active: Vec<&Modifier> = modifiers.filter(|m| !m.is_expired(frame)).collect();

    let mut value = base;
    for m in active.iter().filter(|m| m.op == ModifierOp::Set) {
        value = m.value;
    }

    let add_sum = active
        .iter()
        .filter(|m| m.op == ModifierOp::Add)
        .fold(fixed_math::FIXED_ZERO, |acc, m| acc.saturating_add(m.value));
    value = value.saturating_add(add_sum);

    let pct_sum = active
        .iter()
        .filter(|m| m.op == ModifierOp::Pct)
        .fold(fixed_math::FIXED_ZERO, |acc, m| acc.saturating_add(m.value));
    if pct_sum != fixed_math::FIXED_ZERO {
        value = value.saturating_mul(fixed_math::FIXED_ONE.saturating_add(pct_sum));
    }

    for m in active.iter().filter(|m| m.op == ModifierOp::Mul) {
        value = value.saturating_mul(m.value);
    }

    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fixed::fixed_math::new as fx;

    fn modifier(op: ModifierOp, value: f32, until: Option<u32>) -> Modifier {
        Modifier {
            stat: StatId::MoveSpeed,
            op,
            value: fx(value),
            source: ModifierSource::Named("test".into()),
            until,
        }
    }

    #[test]
    fn no_modifiers_returns_base() {
        let mods: Vec<Modifier> = vec![];
        assert_eq!(resolve(fx(42.0), mods.iter(), 0), fx(42.0));
    }

    #[test]
    fn add_only_sums() {
        let mods = vec![
            modifier(ModifierOp::Add, 10.0, None),
            modifier(ModifierOp::Add, 5.0, None),
        ];
        assert_eq!(resolve(fx(100.0), mods.iter(), 0), fx(115.0));
    }

    #[test]
    fn mul_only_multiplies() {
        let mods = vec![
            modifier(ModifierOp::Mul, 2.0, None),
            modifier(ModifierOp::Mul, 3.0, None),
        ];
        assert_eq!(resolve(fx(10.0), mods.iter(), 0), fx(60.0));
    }

    #[test]
    fn set_last_wins() {
        let mods = vec![
            modifier(ModifierOp::Set, 5.0, None),
            modifier(ModifierOp::Set, 20.0, None),
        ];
        assert_eq!(resolve(fx(100.0), mods.iter(), 0), fx(20.0));
    }

    #[test]
    fn order_is_set_then_add_then_mul() {
        // base=100 ; Set -> 50 ; +10 -> 60 ; *2 -> 120 (peu importe l'ordre d'entrée du Vec)
        let mods = vec![
            modifier(ModifierOp::Mul, 2.0, None),
            modifier(ModifierOp::Add, 10.0, None),
            modifier(ModifierOp::Set, 50.0, None),
        ];
        assert_eq!(resolve(fx(100.0), mods.iter(), 0), fx(120.0));
    }

    #[test]
    fn expired_modifier_is_ignored() {
        let mods = vec![modifier(ModifierOp::Add, 999.0, Some(10))];
        assert_eq!(resolve(fx(1.0), mods.iter(), 11), fx(1.0));
    }

    #[test]
    fn modifier_still_active_when_until_equals_frame() {
        let mods = vec![modifier(ModifierOp::Add, 1.0, Some(10))];
        assert_eq!(resolve(fx(0.0), mods.iter(), 10), fx(1.0));
    }

    #[test]
    fn modifiers_retain_active_drops_expired() {
        let mut mods = Modifiers::default();
        mods.push(modifier(ModifierOp::Add, 1.0, Some(10)));
        mods.push(modifier(ModifierOp::Add, 2.0, None));
        mods.retain_active(11);
        assert_eq!(mods.iter().count(), 1);
    }

    #[test]
    fn pct_only_sums_additively() {
        // 100 * (1 + 0.5 + 0.25) = 175 : les `Pct` s'additionnent avant de multiplier une
        // seule fois (pas 100 * 1.5 * 1.25 = 187.5).
        let mods = vec![
            modifier(ModifierOp::Pct, 0.5, None),
            modifier(ModifierOp::Pct, 0.25, None),
        ];
        assert_eq!(resolve(fx(100.0), mods.iter(), 0), fx(175.0));
    }

    #[test]
    fn pct_absent_is_true_noop() {
        // Aucun `Pct` actif : la valeur ne passe même pas par une multiplication par 1
        // (voir la doc de `resolve`) — vérifié indirectement ici par l'égalité exacte.
        let mods: Vec<Modifier> = vec![];
        assert_eq!(resolve(fx(42.0), mods.iter(), 0), fx(42.0));
    }

    #[test]
    fn order_is_set_then_add_then_pct_then_mul() {
        // base=100 ; Set -> 50 ; +10 -> 60 ; *(1+0.5) -> 90 ; *2 -> 180 (peu importe l'ordre
        // d'entrée du Vec).
        let mods = vec![
            modifier(ModifierOp::Mul, 2.0, None),
            modifier(ModifierOp::Pct, 0.5, None),
            modifier(ModifierOp::Add, 10.0, None),
            modifier(ModifierOp::Set, 50.0, None),
        ];
        assert_eq!(resolve(fx(100.0), mods.iter(), 0), fx(180.0));
    }

    #[test]
    fn mixed_ops_with_expiration() {
        // Un `Pct` expiré est ignoré ; les autres s'appliquent normalement.
        let mods = vec![
            modifier(ModifierOp::Add, 20.0, None),
            modifier(ModifierOp::Pct, 1.0, Some(5)), // expiré à la frame 6
            modifier(ModifierOp::Mul, 3.0, None),
        ];
        assert_eq!(resolve(fx(10.0), mods.iter(), 6), fx(90.0)); // (10+20) * 3, Pct ignoré
    }

    #[test]
    fn modifiers_push_from_builds_modifier() {
        let mut mods = Modifiers::default();
        mods.push_from(
            ModifierSource::Named("buff".into()),
            StatId::MoveSpeed,
            ModifierOp::Mul,
            fx(0.5),
            Some(100),
        );
        assert_eq!(mods.iter().count(), 1);
        let m = mods.iter().next().unwrap();
        assert_eq!(m.stat, StatId::MoveSpeed);
        assert_eq!(m.op, ModifierOp::Mul);
        assert_eq!(m.value, fx(0.5));
        assert_eq!(m.source, ModifierSource::Named("buff".into()));
        assert_eq!(m.until, Some(100));
    }

    #[test]
    fn remove_by_source_drops_only_matching_source() {
        let mut mods = Modifiers::default();
        mods.push_from(
            ModifierSource::Named("statut_gel".into()),
            StatId::MoveSpeed,
            ModifierOp::Mul,
            fx(0.5),
            None,
        );
        mods.push_from(
            ModifierSource::Named("statut_gel".into()),
            StatId::Acceleration,
            ModifierOp::Mul,
            fx(0.5),
            None,
        );
        mods.push_from(
            ModifierSource::Named("autre_source".into()),
            StatId::MoveSpeed,
            ModifierOp::Add,
            fx(10.0),
            None,
        );

        mods.remove_by_source(&ModifierSource::Named("statut_gel".into()));

        assert_eq!(mods.iter().count(), 1);
        assert_eq!(
            mods.iter().next().unwrap().source,
            ModifierSource::Named("autre_source".into())
        );
    }
}
