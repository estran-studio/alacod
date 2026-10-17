//! Curses de run (M3-T0e, `docs/conventions.md` §64) : un modificateur d'**étage** tiré au début
//! de chaque étage d'une séquence `Floors`, façon Isaac (Darkness, Unknown, Blind, Labyrinth…).
//!
//! - [`CurseDef`] : une curse de `curses/<id>.ron` (kind `Curse`), un poids de tirage et ses
//!   [`CurseEffect`] (liste fermée, extensible **en fin d'enum**).
//! - [`CurseConfig`] : champ `curses` d'une séquence `Floors` (absent : aucune curse) : la chance
//!   par étage et le pool d'identifiants.
//! - [`draw_curse`] : tirage **pur** et indépendant du moment : un RNG local seedé par
//!   [`curse_seed`] (`fnv1a("curse:<graine>:<étage>")`), aucun flux de `RngStreams`. Les deux
//!   tirages (chance, choix) sont toujours consommés, comme `items::draw_loot`.
//! - [`RunCurses`] : la curse de chaque étage déjà entré et le dernier étage traité (ressource
//!   rollback **neutre** : vide tant qu'aucune curse n'est tirée, donc aucune trace existante ne
//!   bouge).
//!
//! Ce module ne contient que les contrats et les règles pures. Le tirage à l'entrée d'un étage et
//! l'application des effets (modificateurs de stat sur les joueurs, grammaire, HUD) sont
//! `game::curses` (M3-T9, `docs/conventions.md` §74).

use std::collections::BTreeMap;

use bevy::prelude::Resource;
use bevy_fixed::fixed_math::Fixed;
use bevy_fixed::rng::{fnv1a, RollbackRng};
use serde::{Deserialize, Serialize};
use sim_core::modifier::{Modifier, ModifierOp, ModifierSource};
use sim_core::stats::StatId;

/// Source des modificateurs posés par la curse `id` (`curse:<id>`, comme `item:<id>`).
pub fn curse_source(id: &str) -> ModifierSource {
    ModifierSource::Named(format!("curse:{id}"))
}

/// Vrai pour une source de modificateur posée par une curse (`curse:<id>`) : ce que
/// `game::curses` retire à la sortie de l'étage.
pub fn is_curse_source(source: &ModifierSource) -> bool {
    matches!(source, ModifierSource::Named(name) if name.starts_with("curse:"))
}

/// Ce qu'une curse fait pendant l'étage où elle est active. Les quatre premières variantes ne
/// touchent ni la simulation ni la stat d'un joueur : présentation ou grammaire.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CurseEffect {
    /// Cache la minicarte (présentation seule, lue par le HUD).
    HideMinimap,
    /// Cache la santé (présentation seule, lue par le HUD).
    HideHealth,
    /// Cache le nom des objets au sol (présentation seule, lue par le HUD).
    HideItemNames,
    /// Multiplie le nombre de salles de l'étage (lu par la grammaire en grille, M3-T5/T9). `> 0`.
    FloorRooms(
        #[cfg_attr(feature = "schema", schemars(with = "bevy_fixed::schema::FixedString"))]
        #[cfg_attr(feature = "schema", schemars(extend("x-lint" = "> 0")))]
        Fixed,
    ),
    /// Modificateur posé sur chaque joueur pour la durée de l'étage, source `curse:<id>`
    /// ([`CurseEffect::to_modifier`]). Pour `Mul`, la valeur doit être `> 0`.
    Modifier(
        StatId,
        ModifierOp,
        #[cfg_attr(feature = "schema", schemars(with = "bevy_fixed::schema::FixedString"))] Fixed,
    ),
}

impl CurseEffect {
    /// Le [`Modifier`] d'une variante `Modifier`, de source `curse:<id>` et sans expiration (il
    /// est retiré à la sortie de l'étage) ; `None` pour les autres variantes.
    pub fn to_modifier(&self, id: &str) -> Option<Modifier> {
        match self {
            Self::Modifier(stat, op, value) => Some(Modifier {
                stat: stat.clone(),
                op: *op,
                value: *value,
                source: curse_source(id),
                until: None,
            }),
            _ => None,
        }
    }
}

/// Multiplicateur de salles de l'étage porté par une liste d'effets : le produit de ses `FloorRooms`
/// (`Fixed`, saturant), `None` sans aucun.
pub fn floor_rooms_multiplier(effects: &[CurseEffect]) -> Option<Fixed> {
    effects
        .iter()
        .filter_map(|effect| match effect {
            CurseEffect::FloorRooms(mult) => Some(*mult),
            _ => None,
        })
        .reduce(Fixed::saturating_mul)
}

/// Une curse, `curses/<id>.ron` (kind `Curse`, id = nom de fichier).
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CurseDef {
    /// Poids de tirage dans le pool de la séquence. `> 0`.
    #[cfg_attr(feature = "schema", schemars(extend("x-lint" = "> 0")))]
    pub weight: u32,
    /// Effets cumulés (au moins un).
    pub effects: Vec<CurseEffect>,
}

/// Champ `curses` d'une séquence `Floors` : à chaque étage, une curse du `pool` tombe avec la
/// probabilité `chance`. Absent : aucune curse dans cette séquence.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CurseConfig {
    /// Probabilité (0 à 1) qu'un étage porte une curse.
    #[cfg_attr(feature = "schema", schemars(with = "bevy_fixed::schema::FixedString"))]
    #[cfg_attr(feature = "schema", schemars(extend("x-lint" = "0 à 1")))]
    pub chance: Fixed,
    /// Identifiants de curse (`curses/<id>.ron`), non vide. Les doublons s'additionnent au poids.
    pub pool: Vec<String>,
}

/// La curse de chaque étage déjà entré (index d'étage → id de curse) et le dernier étage dont
/// l'entrée a été traitée. Ressource rollback **neutre** (`RunPlugin`) : vide, donc de hash `0`,
/// tant que le tirage de M3-T9 (`game::curses`) n'a rien écrit, ce que seule une séquence `Floors`
/// à champ `curses` fait. Remise à zéro à chaque relance (`game::run_state`). Un étage sans entrée
/// n'est pas maudit.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct RunCurses {
    /// Curse de chaque étage entré qui en porte une (un étage sans curse n'a pas d'entrée).
    pub floors: BTreeMap<u32, String>,
    /// Dernier étage dont le tirage a été fait (curse ou non), `None` avant le premier : sans cette
    /// mémoire, « pas de curse » ne se distinguerait pas de « pas encore tiré ».
    pub entered: Option<u32>,
}

impl RunCurses {
    /// La curse de l'étage `floor`, si elle a été tirée.
    pub fn get(&self, floor: u32) -> Option<&str> {
        self.floors.get(&floor).map(String::as_str)
    }

    /// Pose la curse de l'étage `floor` (écrase la précédente).
    pub fn set(&mut self, floor: u32, id: impl Into<String>) {
        self.floors.insert(floor, id.into());
    }

    /// Enregistre l'entrée dans l'étage `floor` avec le résultat de [`draw_curse`].
    pub fn enter(&mut self, floor: u32, curse: Option<String>) {
        match curse {
            Some(id) => {
                self.floors.insert(floor, id);
            }
            None => {
                self.floors.remove(&floor);
            }
        }
        self.entered = Some(floor);
    }
}

/// Graine du tirage de la curse de l'étage `floor` d'une partie de graine `run_seed` :
/// `fnv1a("curse:<graine>:<étage>")` replié sur 32 bits. Indépendante du temps, des `Entity` et
/// de tout flux de `RngStreams`.
pub fn curse_seed(run_seed: u32, floor: u32) -> u32 {
    let hash = fnv1a(format!("curse:{run_seed}:{floor}").as_bytes());
    (hash ^ (hash >> 32)) as u32
}

/// Tire la curse de l'étage `floor` : `None` (aucune curse) ou l'id d'une entrée de `config.pool`.
///
/// Deux tirages toujours consommés dans le même ordre : la chance (`next_fixed() < chance` :
/// `0` ne tombe jamais, `1` toujours), puis le choix pondéré (`next_u32() % total`). Les entrées
/// du pool absentes de `defs` ou de poids `0` sont ignorées (le lint les refuse) ; un pool sans
/// entrée valide ne donne jamais de curse. L'ordre du pool est celui du fichier.
pub fn draw_curse(
    config: &CurseConfig,
    defs: &BTreeMap<String, CurseDef>,
    run_seed: u32,
    floor: u32,
) -> Option<String> {
    let mut rng = RollbackRng::new(curse_seed(run_seed, floor));
    let roll = rng.next_fixed();
    let pick = rng.next_u32();
    if roll >= config.chance {
        return None;
    }
    let weighted = |id: &String| defs.get(id).map_or(0, |def| u64::from(def.weight));
    let total: u64 = config.pool.iter().map(weighted).sum();
    if total == 0 {
        return None;
    }
    let mut at = u64::from(pick) % total;
    for id in &config.pool {
        let weight = weighted(id);
        if at < weight {
            return Some(id.clone());
        }
        at -= weight;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def(weight: u32) -> CurseDef {
        CurseDef {
            weight,
            effects: vec![CurseEffect::HideMinimap],
        }
    }

    fn defs() -> BTreeMap<String, CurseDef> {
        [("a", 1), ("b", 3), ("zero", 0)]
            .into_iter()
            .map(|(id, weight)| (id.to_string(), def(weight)))
            .collect()
    }

    fn config(chance: Fixed, pool: &[&str]) -> CurseConfig {
        CurseConfig {
            chance,
            pool: pool.iter().map(|id| id.to_string()).collect(),
        }
    }

    #[test]
    fn chance_zero_never_curses_and_chance_one_always_does() {
        let defs = defs();
        for floor in 0..200 {
            assert_eq!(
                draw_curse(&config(Fixed::ZERO, &["a", "b"]), &defs, 7, floor),
                None
            );
            assert!(draw_curse(&config(Fixed::ONE, &["a", "b"]), &defs, 7, floor).is_some());
        }
    }

    #[test]
    fn the_draw_depends_only_on_seed_and_floor() {
        let defs = defs();
        let cfg = config(Fixed::from_num(0.5), &["a", "b"]);
        let first: Vec<_> = (0..64).map(|f| draw_curse(&cfg, &defs, 42, f)).collect();
        // Rejouer dans un autre ordre, ou étage seul : même résultat (aucun état partagé).
        for floor in (0..64u32).rev() {
            assert_eq!(draw_curse(&cfg, &defs, 42, floor), first[floor as usize]);
        }
        let other: Vec<_> = (0..64).map(|f| draw_curse(&cfg, &defs, 43, f)).collect();
        assert_ne!(first, other, "une autre graine tire autrement");
    }

    #[test]
    fn chance_is_a_probability() {
        let defs = defs();
        let cfg = config(Fixed::from_num(0.25), &["a"]);
        let hits = (0..4000)
            .filter(|floor| draw_curse(&cfg, &defs, 1, *floor).is_some())
            .count();
        assert!((800..1200).contains(&hits), "{hits} curses sur 4000 à 25 %");
    }

    #[test]
    fn pick_follows_the_weights_and_skips_unusable_entries() {
        let defs = defs();
        // « zero » (poids 0) et « inconnue » ne sortent jamais ; b pèse 3 fois a.
        let cfg = config(Fixed::ONE, &["zero", "a", "inconnue", "b"]);
        let mut counts: BTreeMap<String, u32> = BTreeMap::new();
        for floor in 0..4000 {
            *counts
                .entry(draw_curse(&cfg, &defs, 9, floor).unwrap())
                .or_default() += 1;
        }
        assert_eq!(
            counts.keys().map(String::as_str).collect::<Vec<_>>(),
            ["a", "b"]
        );
        let ratio = f64::from(counts["b"]) / f64::from(counts["a"]);
        assert!((2.5..3.6).contains(&ratio), "b/a = {ratio} ({counts:?})");
    }

    #[test]
    fn a_pool_without_usable_entry_gives_no_curse() {
        let defs = defs();
        assert_eq!(draw_curse(&config(Fixed::ONE, &[]), &defs, 1, 0), None);
        assert_eq!(
            draw_curse(&config(Fixed::ONE, &["zero", "inconnue"]), &defs, 1, 0),
            None
        );
    }

    #[test]
    fn the_chance_roll_does_not_depend_on_the_pool() {
        // Même graine et même étage : « tombe-t-il une curse » ne dépend pas du contenu du pool.
        let defs = defs();
        for floor in 0..200 {
            let one = draw_curse(&config(Fixed::from_num(0.5), &["a"]), &defs, 5, floor);
            let two = draw_curse(&config(Fixed::from_num(0.5), &["a", "b"]), &defs, 5, floor);
            assert_eq!(one.is_some(), two.is_some());
        }
    }

    #[test]
    fn modifier_effect_builds_a_curse_sourced_modifier() {
        let effect =
            CurseEffect::Modifier(StatId::MoveSpeed, ModifierOp::Mul, Fixed::from_num(0.8));
        let modifier = effect.to_modifier("maze").expect("variante Modifier");
        assert_eq!(modifier.source, ModifierSource::Named("curse:maze".into()));
        assert_eq!(modifier.op, ModifierOp::Mul);
        assert_eq!(modifier.until, None);
        assert_eq!(CurseEffect::HideHealth.to_modifier("maze"), None);
    }

    #[test]
    fn run_curses_default_is_empty_and_sets_by_floor() {
        let mut curses = RunCurses::default();
        assert_eq!(curses.get(0), None);
        curses.set(1, "darkness");
        assert_eq!(curses.get(1), Some("darkness"));
        assert_eq!(curses.get(0), None);
    }

    #[test]
    fn entering_a_floor_remembers_it_even_without_a_curse() {
        let mut curses = RunCurses::default();
        assert_eq!(curses.entered, None);
        curses.enter(0, None);
        assert_eq!(curses.entered, Some(0));
        assert_eq!(curses.get(0), None);
        assert_ne!(curses, RunCurses::default(), "l'entrée sans curse se voit");
        curses.enter(1, Some("darkness".into()));
        assert_eq!((curses.entered, curses.get(1)), (Some(1), Some("darkness")));
        // Une nouvelle entrée dans le même étage remplace la curse précédente.
        curses.enter(1, None);
        assert_eq!(curses.get(1), None);
    }

    #[test]
    fn floor_rooms_multiplier_is_the_product_of_floor_rooms_effects() {
        let two = CurseEffect::FloorRooms(Fixed::from_num(2));
        let half = CurseEffect::FloorRooms(Fixed::from_num(0.5));
        assert_eq!(floor_rooms_multiplier(&[CurseEffect::HideMinimap]), None);
        assert_eq!(floor_rooms_multiplier(&[]), None);
        assert_eq!(
            floor_rooms_multiplier(&[CurseEffect::HideHealth, two.clone()]),
            Some(Fixed::from_num(2))
        );
        assert_eq!(
            floor_rooms_multiplier(&[two.clone(), half, two]),
            Some(Fixed::from_num(2))
        );
    }

    #[test]
    fn curse_files_parse_from_ron() {
        let def: CurseDef = ron::from_str(
            r#"(weight: 2, effects: [HideMinimap, FloorRooms("2.0"), Modifier(MoveSpeed, Mul, "0.8")])"#,
        )
        .expect("RON valide");
        assert_eq!(def.weight, 2);
        assert_eq!(def.effects.len(), 3);
        assert_eq!(def.effects[1], CurseEffect::FloorRooms(Fixed::from_num(2)));
    }
}
