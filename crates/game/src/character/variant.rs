//! Variantes et élites (T1.5, chantier D2, `docs/conventions.md` §25) : un personnage peut
//! déclarer une table de variantes — modificateurs de stats, tags, skin — dont une est tirée
//! à son apparition. Une **élite** est une variante comme les autres (tag `champion` et
//! modificateurs), sans mécanisme à part.
//!
//! Tirage **déterministe et local** ([`draw_variant`]) : un `RollbackRng` de graine
//! `fnv1a("variants:<run_seed>:<net_id>")` replié sur 32 bits puis mélangé (deux tirages : la
//! chance, puis le poids), sans jamais toucher `RngStreams` (y créer un flux changerait le hash de la ressource, donc toutes
//! les traces). Un personnage sans table ne tire rien ; un `CharacterSpawn` LDtk peut imposer
//! la variante (champ `variant`, aucun tirage).

use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_fixed::{
    fixed_math::{self, Fixed},
    rng::{fnv1a_extend, fnv1a_extend_decimal, RollbackRng, FNV1A_OFFSET},
};
use content::expr::NumOrExpr;
use serde::Deserialize;
use sim_core::{
    modifier::{Modifier, ModifierOp, ModifierSource},
    stats::StatId,
    tag::Tags,
};

/// `variants: Some((chance: "0.25", table: { ... }))` d'un `CharacterConfig`.
#[derive(Debug, Clone, Deserialize)]
pub struct VariantsConfig {
    /// Probabilité d'avoir une variante (`[0, 1]`, défaut 1). M3-T0d : un nombre (valeur
    /// inchangée) **ou une expression** (`"0.1 + floor * 0.05"`, identifiants `players` et
    /// `floor`, `docs/conventions.md` §18 et §63).
    #[serde(default = "full_chance")]
    pub chance: NumOrExpr,
    /// Variantes par nom (ordre `BTreeMap` : ordre du tirage pondéré).
    pub table: BTreeMap<String, VariantDef>,
}

fn full_chance() -> NumOrExpr {
    NumOrExpr::Literal(fixed_math::FIXED_ONE)
}

impl VariantsConfig {
    /// Chance résolue pour `players` joueurs à l'étage `floor` (un littéral est inchangé). Une
    /// expression en erreur donne 0 : le lint la refuse avant le jeu.
    pub fn chance_at(&self, players: u32, floor: u32) -> Fixed {
        let ctx = std::collections::BTreeMap::from([
            ("players".to_string(), Fixed::from_num(players)),
            ("floor".to_string(), Fixed::from_num(floor)),
        ]);
        self.chance.resolve_with(&ctx).unwrap_or(Fixed::ZERO)
    }
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
    /// M3-T0d (`docs/conventions.md` §63) : actions jouées **à la mort** du porteur (`Explode`,
    /// `SpawnCharacters`, `SpawnPattern`). Vide : rien (v1 exacte). Contrat : exécuté par M3-T7 ;
    /// la définition est hors rollback, le composant [`Variant`] ne change pas.
    #[serde(default)]
    pub on_death: Vec<effects::Action>,
    /// M3-T0d : remplace la liste de behaviors du personnage. `None` : celle du personnage.
    #[serde(default)]
    pub behaviors: Option<Vec<behaviors::Behavior>>,
    /// M3-T0d : table de butin (`loot/<table>.ron`) lâchée à la mort, tirée par la graine de §48
    /// avec la clé de contenu `champion:<net_id>`.
    #[serde(default)]
    pub loot: Option<String>,
    /// M3-T0d : teinte `#rrggbb` du sprite (présentation seule, hors trace).
    #[serde(default)]
    pub tint: Option<String>,
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

/// Contexte d'un tirage (M3-T7, §72) : nombre de joueurs de la session et index de l'étage **à
/// l'apparition** (les identifiants `players` et `floor` de `chance`). Valeur figée au tirage : la
/// variante ne change plus ensuite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VariantContext {
    pub players: u32,
    pub floor: u32,
}

impl Default for VariantContext {
    /// Un joueur, étage 0 (hors partie : tests, outils).
    fn default() -> Self {
        Self {
            players: 1,
            floor: 0,
        }
    }
}

/// Graine du tirage d'un personnage : `fnv1a("variants:<run_seed>:<net_id>")` replié sur 32 bits
/// (même idiome que `loot_seed` et `curse_seed`), puis passé au finaliseur de MurmurHash3
/// (`fmix32`), indépendante de l'ordre d'apparition et du moment du chargement de la carte.
///
/// D88 : l'ancien `fnv ^ graine ^ net_id` suivi d'un pas de congruentiel donnait des tirages quasi
/// linéaires, donc corrélés. Le repli FNV seul ne suffit pas : le dernier octet haché n'entre que
/// par un produit avec un nombre premier à faible poids, et deux net ids voisins gardent une
/// corrélation de 0,07 (mesurée sur 20 000 graines) ; le finaliseur la ramène à zéro.
pub fn variant_seed(run_seed: u32, net_id: u64) -> u32 {
    // Les mêmes octets que `fnv1a(format!("variants:{run_seed}:{net_id}"))`, sans allocation
    // (appelé à chaque apparition d'un ennemi à variantes).
    let hash = fnv1a_extend(FNV1A_OFFSET, b"variants:");
    let hash = fnv1a_extend_decimal(hash, u64::from(run_seed));
    let hash = fnv1a_extend(hash, b":");
    let hash = fnv1a_extend_decimal(hash, net_id);
    fmix32((hash ^ (hash >> 32)) as u32)
}

/// Finaliseur de MurmurHash3 : chaque bit d'entrée change chaque bit de sortie avec probabilité ½.
fn fmix32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x85eb_ca6b);
    x ^= x >> 13;
    x = x.wrapping_mul(0xc2b2_ae35);
    x ^ (x >> 16)
}

/// Variante tirée (ou imposée par `forced`) pour un personnage. `None` : pas de variante
/// (pas de table, `chance` ratée, ou poids tous nuls). Une variante imposée inconnue de la
/// table est ignorée (refusée par le lint).
pub fn draw_variant(
    config: &VariantsConfig,
    forced: Option<&str>,
    run_seed: u32,
    net_id: u64,
    context: VariantContext,
) -> Option<String> {
    if let Some(name) = forced.filter(|name| !name.is_empty()) {
        return config.table.contains_key(name).then(|| name.to_string());
    }
    let mut rng = RollbackRng::new(variant_seed(run_seed, net_id));
    // M3-T7 : une expression se résout avec le nombre de joueurs et l'étage de l'apparition ; un
    // littéral est inchangé (même tirage qu'avant).
    if rng.next_fixed() >= config.chance_at(context.players, context.floor) {
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
            on_death: vec![],
            behaviors: None,
            loot: None,
            tint: None,
        }
    }

    fn table(chance: f32, entries: &[(&str, u32)]) -> VariantsConfig {
        VariantsConfig {
            chance: NumOrExpr::Literal(fixed_math::new(chance)),
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
                draw_variant(&config, None, 123456, net_id, VariantContext::default()),
                draw_variant(&config, None, 123456, net_id, VariantContext::default())
            );
        }
        // Des net ids différents donnent des tirages différents (pas tous identiques).
        let draws: Vec<_> = (0..50)
            .map(|id| draw_variant(&config, None, 123456, id, VariantContext::default()))
            .collect();
        assert!(draws.iter().any(|d| d != &draws[0]));
    }

    /// D88 : tirage indépendant entre graines et entre net ids voisins. Graine fixe (aucun
    /// hasard) : le résultat est le même à chaque exécution.
    #[test]
    fn tirage_independant_par_graine_et_net_id() {
        const SEEDS: u32 = 20_000;
        const NET_IDS: u64 = 16;
        const FIRST_NET_ID: u64 = 100;
        let config = table(0.1, &[("champion", 1)]);
        let context = VariantContext::default();
        let mut without_champion = 0u32;
        let mut hits = [0u32; NET_IDS as usize];
        // `pairs[i]` : graines où les net ids `i` et `i + 1` tirent tous deux un champion.
        let mut pairs = [0u32; NET_IDS as usize - 1];
        for seed in 1..=SEEDS {
            let draws: Vec<bool> = (0..NET_IDS)
                .map(|i| draw_variant(&config, None, seed, FIRST_NET_ID + i, context).is_some())
                .collect();
            if draws.iter().all(|hit| !hit) {
                without_champion += 1;
            }
            for i in 0..draws.len() {
                hits[i] += u32::from(draws[i]);
                if i + 1 < draws.len() {
                    pairs[i] += u32::from(draws[i] && draws[i + 1]);
                }
            }
        }
        // Indépendance : (0,9)^16 ≈ 18,5 % des graines sans aucun champion (l'ancien tirage : 89 %).
        let share = f64::from(without_champion) / f64::from(SEEDS);
        assert!(
            (0.15..=0.25).contains(&share),
            "{:.1} % des graines sans champion (attendu ≈ 18,5 %)",
            share * 100.0
        );
        // Corrélation de Pearson entre net ids voisins (indicateurs de Bernoulli de même p).
        let n = f64::from(SEEDS);
        for i in 0..pairs.len() {
            let (pi, pj) = (f64::from(hits[i]) / n, f64::from(hits[i + 1]) / n);
            assert!(
                (0.08..=0.12).contains(&pi),
                "net id {i} : chance mesurée {pi}"
            );
            let covariance = f64::from(pairs[i]) / n - pi * pj;
            let correlation = covariance / (pi * (1.0 - pi) * pj * (1.0 - pj)).sqrt();
            assert!(
                correlation.abs() < 0.03,
                "net ids {} et {} corrélés : {correlation:.4}",
                FIRST_NET_ID + i as u64,
                FIRST_NET_ID + i as u64 + 1
            );
        }
    }

    #[test]
    fn graines_voisines_tirages_differents() {
        let config = table(0.5, &[("a", 1), ("b", 1)]);
        let context = VariantContext::default();
        let vector = |seed: u32| -> Vec<Option<String>> {
            (0..32)
                .map(|net_id| draw_variant(&config, None, seed, net_id, context))
                .collect()
        };
        for seed in 123_456..123_456 + 200 {
            assert_ne!(
                vector(seed),
                vector(seed + 1),
                "graines {seed} et {}",
                seed + 1
            );
        }
    }

    /// Le hachage sans allocation produit la graine de la clé textuelle `variants:<graine>:<net id>`.
    #[test]
    fn graine_sans_allocation_egale_a_la_cle_textuelle() {
        for (run_seed, net_id) in [
            (0, 0),
            (1, 9),
            (123_456, 100),
            (u32::MAX, 10_000),
            (777, u64::MAX),
        ] {
            let hash = bevy_fixed::rng::fnv1a(format!("variants:{run_seed}:{net_id}").as_bytes());
            let expected = {
                let mut x = (hash ^ (hash >> 32)) as u32;
                x ^= x >> 16;
                x = x.wrapping_mul(0x85eb_ca6b);
                x ^= x >> 13;
                x = x.wrapping_mul(0xc2b2_ae35);
                x ^ (x >> 16)
            };
            assert_eq!(
                variant_seed(run_seed, net_id),
                expected,
                "{run_seed}:{net_id}"
            );
        }
    }

    #[test]
    fn chance_zero_jamais_un_toujours() {
        let never = table(0.0, &[("a", 1)]);
        let always = table(1.0, &[("a", 1)]);
        for net_id in 0..200 {
            assert_eq!(
                draw_variant(&never, None, 7, net_id, VariantContext::default()),
                None
            );
            assert_eq!(
                draw_variant(&always, None, 7, net_id, VariantContext::default()),
                Some("a".into())
            );
        }
    }

    #[test]
    fn poids_nul_jamais_tire() {
        let config = table(1.0, &[("jamais", 0), ("toujours", 3)]);
        for net_id in 0..200 {
            assert_eq!(
                draw_variant(&config, None, 99, net_id, VariantContext::default()),
                Some("toujours".into())
            );
        }
        assert_eq!(
            draw_variant(
                &table(1.0, &[("x", 0)]),
                None,
                1,
                1,
                VariantContext::default()
            ),
            None
        );
    }

    #[test]
    fn variante_imposee_sans_tirage() {
        let config = table(0.0, &[("rapide", 1), ("blinde", 1)]);
        assert_eq!(
            draw_variant(&config, Some("blinde"), 1, 1, VariantContext::default()),
            Some("blinde".into())
        );
        // Inconnue : ignorée (refusée par le lint) ; vide : pas imposée, tirage normal.
        assert_eq!(
            draw_variant(&config, Some("absente"), 1, 1, VariantContext::default()),
            None
        );
        assert_eq!(
            draw_variant(&config, Some(""), 1, 1, VariantContext::default()),
            None
        );
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

    /// M3-T7 : la chance en expression suit le nombre de joueurs et l'étage de l'apparition ; un
    /// littéral reste exact.
    #[test]
    fn chance_en_expression_suit_joueurs_et_etage() {
        let config = VariantsConfig {
            chance: ron::from_str("\"0.1 + floor * 0.2 + players * 0.1\"").unwrap(),
            table: BTreeMap::from([("a".to_string(), def(1))]),
        };
        let ctx = |players, floor| VariantContext { players, floor };
        let near = |value: Fixed, expected: f32| {
            (value - fixed_math::new(expected)).abs() < fixed_math::new(0.001)
        };
        assert!(near(config.chance_at(1, 0), 0.2));
        assert!(near(config.chance_at(1, 2), 0.6));
        assert!(near(config.chance_at(4, 2), 0.9));
        let hits = |context| {
            (0..400)
                .filter(|net_id| draw_variant(&config, None, 5, *net_id, context).is_some())
                .count()
        };
        let (low, mid, high) = (hits(ctx(1, 0)), hits(ctx(1, 2)), hits(ctx(4, 2)));
        assert!(low < mid && mid < high, "{low} < {mid} < {high}");
        // Même graine, même net id, même contexte : même tirage ; un tirage qui réussit à l'étage
        // 0 réussit aussi à l'étage 2 (la chance ne fait que croître : même nombre tiré).
        for net_id in 0..400 {
            let at0 = draw_variant(&config, None, 5, net_id, ctx(1, 0)).is_some();
            let at2 = draw_variant(&config, None, 5, net_id, ctx(1, 2)).is_some();
            assert!(!at0 || at2, "net id {net_id}");
        }
        let literal = table(0.5, &[("a", 1)]);
        for net_id in 0..100 {
            assert_eq!(
                draw_variant(&literal, None, 5, net_id, ctx(1, 0)),
                draw_variant(&literal, None, 5, net_id, ctx(4, 9)),
                "un littéral ne dépend ni des joueurs ni de l'étage"
            );
        }
    }
}
