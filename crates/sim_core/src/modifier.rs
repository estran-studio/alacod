//! Modificateurs de stats et leur résolution déterministe.

use bevy_fixed::fixed_math::{self, Fixed};
use serde::{Deserialize, Serialize};

use crate::stats::StatId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModifierOp {
    Add,
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
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Modifiers(pub Vec<Modifier>);

impl Modifiers {
    pub fn push(&mut self, modifier: Modifier) {
        self.0.push(modifier);
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
/// à `frame`.
///
/// Ordre fixe et documenté (`docs/taches.md`, T0.2), indépendant de l'ordre de parcours de
/// `modifiers` pour les `Add`/`Mul` (commutatifs), mais **pas** pour les `Set` :
///
/// 1. `Set` : chaque modificateur `Set` rencontré remplace le résultat courant — **le
///    dernier de l'itération l'emporte**. Appeler avec un ordre stable (ex. l'ordre
///    d'insertion de [`Modifiers`]) si plusieurs `Set` sur la même stat doivent avoir un
///    gagnant déterministe entre clients.
/// 2. `Add` : somme de tous les `Add` actifs, ajoutée au résultat de l'étape 1.
/// 3. `Mul` : produit de tous les `Mul` actifs (facteur multiplicatif direct, pas
///    `1 + valeur`), appliqué au résultat de l'étape 2.
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
}
