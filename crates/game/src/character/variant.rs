//! Variantes et élites (T1.5, chantier D2, `docs/conventions.md` §25) : un personnage peut
//! déclarer une table de variantes — modificateurs de stats, tags, skin — dont une est tirée
//! à son apparition. Une **élite** est une variante comme les autres (tag `champion` et
//! modificateurs), sans mécanisme à part.
//!
//! Tirage **déterministe et local** ([`draw_variant`]) : un `RollbackRng` de graine
//! `fnv1a("variants") ^ run_seed ^ net_id` (deux tirages : la chance, puis le poids), sans
//! jamais toucher `RngStreams` (y créer un flux changerait le hash de la ressource, donc toutes
//! les traces). Un personnage sans table ne tire rien ; un `CharacterSpawn` LDtk peut imposer
//! la variante (champ `variant`, aucun tirage).

use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_fixed::{
    fixed_math::{self, Fixed},
    rng::{fnv1a, RollbackRng},
};
use serde::Deserialize;
use sim_core::{
    modifier::{Modifier, ModifierOp, ModifierSource},
    stats::StatId,
    tag::Tags,
};

/// `variants: Some((chance: "0.25", table: { ... }))` d'un `CharacterConfig`.
#[derive(Debug, Clone, Deserialize)]
pub struct VariantsConfig {
    /// Probabilité d'avoir une variante (`[0, 1]`, défaut 1).
    #[serde(default = "full_chance")]
    pub chance: Fixed,
    /// Variantes par nom (ordre `BTreeMap` : ordre du tirage pondéré).
    pub table: BTreeMap<String, VariantDef>,
}

fn full_chance() -> Fixed {
    fixed_math::FIXED_ONE
}

fn unit_weight() -> u32 {
    1
}

/// Une variante : poids de tirage, modificateurs (format §9), tags (union avec ceux du
/// personnage), skin (clé de `skins`, remplace `starting_skin`).
#[derive(Debug, Clone, Deserialize)]
pub struct VariantDef {
    #[serde(default = "unit_weight")]
    pub weight: u32,
    #[serde(default)]
    pub modifiers: Vec<VariantModifierDef>,
    #[serde(default)]
    pub tags: Tags,
    #[serde(default)]
    pub skin: Option<String>,
}

/// Modificateur de variante, `(stat: EnemyMoveSpeed, op: Mul, value: "1.5")`.
#[derive(Debug, Clone, Deserialize)]
pub struct VariantModifierDef {
    pub stat: StatId,
    pub op: ModifierOp,
    pub value: Fixed,
}

impl VariantDef {
    /// Modificateurs permanents, source `Named("variant:<nom>")`.
    pub fn modifiers(&self, name: &str) -> Vec<Modifier> {
        self.modifiers
            .iter()
            .map(|def| Modifier {
                stat: def.stat.clone(),
                op: def.op,
                value: def.value,
                source: ModifierSource::Named(format!("variant:{name}")),
                until: None,
            })
            .collect()
    }
}

/// Variante d'un personnage (T1.5), posée seulement si une variante est choisie. Composant
/// rollback à checksum **neutre** (`rollback_and_trace_neutral`) : aucun personnage
/// existant n'en porte, les traces existantes ne bougent pas.
#[derive(Component, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Variant(pub String);

/// Graine du tirage d'un personnage : `fnv1a("variants") ^ run_seed ^ net_id` (tronqués en
/// `u32`), indépendante de l'ordre d'apparition et du moment du chargement de la carte.
pub fn variant_seed(run_seed: u32, net_id: u64) -> u32 {
    (fnv1a(b"variants") as u32) ^ run_seed ^ (net_id as u32)
}

/// Variante tirée (ou imposée par `forced`) pour un personnage. `None` : pas de variante
/// (pas de table, `chance` ratée, ou poids tous nuls). Une variante imposée inconnue de la
/// table est ignorée (refusée par le lint).
pub fn draw_variant(
    config: &VariantsConfig,
    forced: Option<&str>,
    run_seed: u32,
    net_id: u64,
) -> Option<String> {
    if let Some(name) = forced.filter(|name| !name.is_empty()) {
        return config.table.contains_key(name).then(|| name.to_string());
    }
    let mut rng = RollbackRng::new(variant_seed(run_seed, net_id));
    if rng.next_fixed() >= config.chance {
        return None;
    }
    let total: u32 = config.table.values().map(|def| def.weight).sum();
    if total == 0 {
        return None;
    }
    let mut pick = rng.next_u32_range(0, total);
    for (name, def) in &config.table {
        if pick < def.weight {
            return Some(name.clone());
        }
        pick -= def.weight;
    }
    None
}

/// Santé de départ d'un personnage à variante : la santé résolue (F5) avec les modificateurs
/// `MaxHealth` de la variante (sinon `sync_health_from_stats` relève le maximum mais laisse la
/// santé courante à sa valeur de base).
pub fn variant_health(base: Fixed, modifiers: &[Modifier], frame: u32) -> Fixed {
    // Exception assumée à « `StatReader` seul lit les stats » (`docs/conventions.md` §9) : au
    // spawn, `Stats`/`Modifiers` n'existent pas encore sur l'entité, donc on résout ici les
    // modificateurs `MaxHealth` de la variante sur la santé de base.
    sim_core::modifier::resolve(
        base,
        modifiers.iter().filter(|m| m.stat == StatId::MaxHealth),
        frame,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def(weight: u32) -> VariantDef {
        VariantDef {
            weight,
            modifiers: vec![VariantModifierDef {
                stat: StatId::EnemyMoveSpeed,
                op: ModifierOp::Mul,
                value: fixed_math::new(1.5),
            }],
            tags: Tags::default(),
            skin: None,
        }
    }

    fn table(chance: f32, entries: &[(&str, u32)]) -> VariantsConfig {
        VariantsConfig {
            chance: fixed_math::new(chance),
            table: entries
                .iter()
                .map(|(name, weight)| (name.to_string(), def(*weight)))
                .collect(),
        }
    }

    #[test]
    fn meme_graine_et_net_id_meme_variante() {
        let config = table(0.5, &[("a", 1), ("b", 1), ("c", 1)]);
        for net_id in 0..50 {
            assert_eq!(
                draw_variant(&config, None, 123456, net_id),
                draw_variant(&config, None, 123456, net_id)
            );
        }
        // Des net ids différents donnent des tirages différents (pas tous identiques).
        let draws: Vec<_> = (0..50)
            .map(|id| draw_variant(&config, None, 123456, id))
            .collect();
        assert!(draws.iter().any(|d| d != &draws[0]));
    }

    #[test]
    fn chance_zero_jamais_un_toujours() {
        let never = table(0.0, &[("a", 1)]);
        let always = table(1.0, &[("a", 1)]);
        for net_id in 0..200 {
            assert_eq!(draw_variant(&never, None, 7, net_id), None);
            assert_eq!(draw_variant(&always, None, 7, net_id), Some("a".into()));
        }
    }

    #[test]
    fn poids_nul_jamais_tire() {
        let config = table(1.0, &[("jamais", 0), ("toujours", 3)]);
        for net_id in 0..200 {
            assert_eq!(
                draw_variant(&config, None, 99, net_id),
                Some("toujours".into())
            );
        }
        assert_eq!(draw_variant(&table(1.0, &[("x", 0)]), None, 1, 1), None);
    }

    #[test]
    fn variante_imposee_sans_tirage() {
        let config = table(0.0, &[("rapide", 1), ("blinde", 1)]);
        assert_eq!(
            draw_variant(&config, Some("blinde"), 1, 1),
            Some("blinde".into())
        );
        // Inconnue : ignorée (refusée par le lint) ; vide : pas imposée, tirage normal.
        assert_eq!(draw_variant(&config, Some("absente"), 1, 1), None);
        assert_eq!(draw_variant(&config, Some(""), 1, 1), None);
    }

    #[test]
    fn modificateurs_source_variante_et_sante() {
        let mut blinde = def(1);
        blinde.modifiers = vec![VariantModifierDef {
            stat: StatId::MaxHealth,
            op: ModifierOp::Mul,
            value: fixed_math::new(2.0),
        }];
        let modifiers = blinde.modifiers("blinde");
        assert_eq!(
            modifiers[0].source,
            ModifierSource::Named("variant:blinde".into())
        );
        assert_eq!(modifiers[0].until, None);
        assert_eq!(
            variant_health(fixed_math::new(60.0), &modifiers, 0),
            fixed_math::new(120.0)
        );
        // Sans modificateur de santé : santé de base.
        assert_eq!(
            variant_health(fixed_math::new(60.0), &def(1).modifiers("rapide"), 0),
            fixed_math::new(60.0)
        );
    }
}
