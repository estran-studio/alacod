//! Grammaire d'étage (M2-T10, chantier E2, `docs/conventions.md` §40) : assemble des gabarits LDtk
//! typés (`room_kind`, M2-E1) en un étage de salles qui respecte des exigences — un départ, un boss
//! à distance maximale et en cul-de-sac, des salles spéciales requises (récompense, boutique…),
//! des salles de combat pour le reste — à la différence de `Basic`, qui enchaîne des gabarits sans
//! rien savoir de leur type.
//!
//! **Déterminisme.** Rien d'autre que le `RollbackRng` de génération : chaque tentative a sa graine
//! dérivée (`rng.next_u32()` du générateur principal), le tirage se fait sur des listes ordonnées
//! (parents par ordre de placement, connexions par indice, gabarits dans l'ordre de
//! `compatiable_levels`) ; aucune collection à ordre aléatoire.
//!
//! **Algorithme** (un étage = un arbre, plaqué directement sur les gabarits, sans chevauchement) :
//! 1. départ : un gabarit `Spawn` du type `start`, au centre du monde ;
//! 2. salles de remplissage (`filler`, par défaut `combat`) en croissance aléatoire : à chaque
//!    pas, tirage uniforme parmi toutes les poses valides (parent, connexion libre, gabarit
//!    compatible du bon type, sans chevauchement ni sortie de carte) ;
//! 3. boss : posé sur une salle de **profondeur maximale** (distance en salles depuis le départ),
//!    avec un gabarit de type `boss` à **une seule connexion** (cul-de-sac) : sa distance est donc
//!    strictement la plus grande de l'étage ;
//! 4. salles requises (`required`, hors boss) : posées sur une salle moins profonde que le boss
//!    (profondeur ≤ celle du boss − 2), jamais plus loin que lui ;
//! 5. contrôle de [`validate_plan`] ; en cas d'échec, nouvelle tentative avec une autre graine,
//!    **bornée** par `max_attempts` ([`FloorError::Exhausted`] au-delà).

use std::collections::BTreeMap;

use bevy_fixed::rng::RollbackRng;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    context::{AvailableLevel, LevelType, MapGenerationContext},
    position::Position,
    room::{ConnectionTo, Room, RoomConnection},
    LEVEL_PROPERTIES_SPAWN_NAME,
};

fn default_start() -> String {
    "depart".to_string()
}
fn default_boss() -> String {
    "boss".to_string()
}
fn default_filler() -> String {
    "combat".to_string()
}
fn default_attempts() -> u32 {
    64
}

/// Exigences d'un étage (RON : fichier du kind de contenu `FloorGrammar`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FloorGrammar {
    /// Nombre de salles de l'étage, bornes incluses (départ, boss et salles requises comprises).
    pub rooms: (usize, usize),
    /// Salles requises par type, **boss compris** (`"boss": 1`) : `{"boss": 1, "boutique": 1,
    /// "recompense": 1}`. Le type du boss est exactement une salle.
    #[serde(default)]
    pub required: BTreeMap<String, usize>,
    /// Type de la salle de départ (gabarit `spawn: true`).
    #[serde(default = "default_start")]
    pub start: String,
    /// Type du boss (une seule connexion, à distance maximale du départ).
    #[serde(default = "default_boss")]
    pub boss: String,
    /// Type des salles de remplissage.
    #[serde(default = "default_filler")]
    pub filler: String,
    /// Distance minimale (en salles) du départ au boss.
    #[serde(default)]
    pub min_boss_distance: usize,
    /// Tentatives au plus avant d'échouer.
    #[serde(default = "default_attempts")]
    pub max_attempts: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FloorError {
    /// Aucun gabarit compatible pour ce type (grammaire impossible avec ces gabarits).
    NoTemplate(String),
    /// Aucune tentative n'a satisfait la grammaire.
    Exhausted { attempts: u32 },
}

impl std::fmt::Display for FloorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoTemplate(kind) => write!(f, "aucun gabarit de type « {kind} »"),
            Self::Exhausted { attempts } => {
                write!(f, "grammaire d'étage non satisfaite en {attempts} tentatives")
            }
        }
    }
}

/// Une salle posée : la salle, son parent (index dans le plan) et sa profondeur.
#[derive(Debug, Clone)]
pub struct PlannedRoom {
    pub room: Room,
    pub parent: Option<usize>,
    pub depth: usize,
    /// Connexions utilisées à la pose : (celle de cette salle, celle du parent).
    pub via: Option<(usize, usize)>,
    pub kind: String,
}

/// Plan d'un étage : salles dans l'ordre de pose (chaque salle suit son parent).
#[derive(Debug, Clone)]
pub struct FloorPlan {
    pub rooms: Vec<PlannedRoom>,
    /// Numéro (à partir de 1) de la tentative réussie.
    pub attempts: u32,
}

fn kind_of(level: &AvailableLevel) -> &str {
    level.room_kind.as_deref().unwrap_or("")
}

/// Une pose candidate : le parent, sa connexion libre, le gabarit et la connexion de ce dernier.
struct Candidate {
    parent: usize,
    parent_connection: usize,
    level: std::rc::Rc<AvailableLevel>,
    level_connection: usize,
    position: Position,
}

struct Attempt<'a> {
    context: &'a MapGenerationContext,
    rng: RollbackRng,
    placed: Vec<PlannedRoom>,
}

impl Attempt<'_> {
    /// Toutes les poses valides d'une salle de type `kind` (et, pour `cul_de_sac`, à une seule
    /// connexion) sur les parents autorisés par `parent_ok`, dans l'ordre déterministe.
    fn candidates(
        &self,
        kind: &str,
        dead_end: bool,
        parent_ok: &dyn Fn(&PlannedRoom) -> bool,
    ) -> Vec<Candidate> {
        let mut out = Vec::new();
        for (parent_index, parent) in self.placed.iter().enumerate() {
            if !parent_ok(parent) {
                continue;
            }
            for connection in parent.room.connections.iter().filter(|c| c.to.is_none()) {
                let def = &parent.room.level_def.connections[connection.index];
                for (level_id, level_connection) in &def.compatiable_levels {
                    let Some(level) = self
                        .context
                        .available_levels
                        .iter()
                        .find(|l| &l.level_id == level_id)
                    else {
                        continue;
                    };
                    if level.level_type == LevelType::Spawn
                        || kind_of(level) != kind
                        || (dead_end && level.connections.len() != 1)
                    {
                        continue;
                    }
                    let position = parent.room.get_connecting_room_position(
                        def,
                        level,
                        *level_connection,
                        &self.context.tile_size,
                    );
                    if !self.fits(level, &position) {
                        continue;
                    }
                    out.push(Candidate {
                        parent: parent_index,
                        parent_connection: connection.index,
                        level: level.clone(),
                        level_connection: *level_connection,
                        position,
                    });
                }
            }
        }
        out
    }

    /// Dans la carte et sans chevauchement avec une salle posée.
    fn fits(&self, level: &std::rc::Rc<AvailableLevel>, position: &Position) -> bool {
        let probe = Room {
            level_iid: String::new(),
            position: position.clone(),
            connections: Vec::new(),
            entity_locations: level.entity_locations.clone(),
            level_def: level.clone(),
            properties: Default::default(),
        };
        !probe.is_outside(&self.context.config)
            && !self.placed.iter().any(|placed| probe.is_overlapping(&placed.room))
    }

    /// Tire une pose parmi `candidates` et la place ; `false` s'il n'y en a aucune.
    fn place(&mut self, kind: &str, mut candidates: Vec<Candidate>) -> bool {
        if candidates.is_empty() {
            return false;
        }
        let pick = self.rng.next_u32_range(0, candidates.len() as u32) as usize;
        let candidate = candidates.swap_remove(pick);
        let mut room = Room::create(
            &mut self.rng,
            candidate.level.clone(),
            candidate.position.clone(),
            [(LEVEL_PROPERTIES_SPAWN_NAME.to_string(), Value::Bool(false))].into(),
        );
        let depth = self.placed[candidate.parent].depth + 1;
        room.set_connection_between(
            candidate.level_connection,
            &mut self.placed[candidate.parent].room,
            candidate.parent_connection,
        );
        self.placed.push(PlannedRoom {
            room,
            parent: Some(candidate.parent),
            depth,
            via: Some((candidate.level_connection, candidate.parent_connection)),
            kind: kind.to_string(),
        });
        true
    }
}

fn snap(v: i32, size: i32) -> i32 {
    (v + size / 2).div_euclid(size) * size
}

fn try_attempt(
    grammar: &FloorGrammar,
    context: &MapGenerationContext,
    seed: u32,
) -> Option<Vec<PlannedRoom>> {
    let mut attempt = Attempt {
        context,
        rng: RollbackRng::new(seed),
        placed: Vec::new(),
    };
    let total = attempt
        .rng
        .next_u32_range_inclusive(grammar.rooms.0 as u32, grammar.rooms.1 as u32)
        as usize;
    let others: usize = grammar
        .required
        .iter()
        .filter(|(kind, _)| **kind != grammar.boss)
        .map(|(_, n)| *n)
        .sum();
    // départ + boss + requises ; le reste est du remplissage
    let fillers = total.checked_sub(2 + others)?;

    // 1. départ, au centre du monde
    let starts: Vec<_> = context
        .available_levels
        .iter()
        .filter(|l| l.level_type == LevelType::Spawn && kind_of(l) == grammar.start)
        .collect();
    if starts.is_empty() {
        return None;
    }
    let start = starts[attempt.rng.next_u32_range(0, starts.len() as u32) as usize];
    let position = Position(
        snap(-start.level_size_p.0 / 2, context.tile_size.0),
        snap(-start.level_size_p.1 / 2, context.tile_size.1),
    );
    let room = Room::create(
        &mut attempt.rng,
        (*start).clone(),
        position,
        [(LEVEL_PROPERTIES_SPAWN_NAME.to_string(), Value::Bool(true))].into(),
    );
    attempt.placed.push(PlannedRoom {
        room,
        parent: None,
        depth: 0,
        via: None,
        kind: grammar.start.clone(),
    });

    // 2. remplissage
    for _ in 0..fillers {
        let candidates = attempt.candidates(&grammar.filler, false, &|_| true);
        if !attempt.place(&grammar.filler, candidates) {
            return None;
        }
    }

    // 3. boss : sur une salle de profondeur maximale, en cul-de-sac
    let max_depth = attempt.placed.iter().map(|p| p.depth).max().unwrap_or(0);
    let candidates = attempt.candidates(&grammar.boss, true, &|p| p.depth == max_depth);
    if !attempt.place(&grammar.boss, candidates) {
        return None;
    }
    let boss_depth = max_depth + 1;
    if boss_depth < grammar.min_boss_distance {
        return None;
    }

    // 4. salles requises, moins profondes que le boss
    for (kind, count) in &grammar.required {
        if *kind == grammar.boss {
            continue;
        }
        for _ in 0..*count {
            let candidates =
                attempt.candidates(kind, false, &|p| p.depth + 2 <= boss_depth && p.kind != grammar.boss);
            if !attempt.place(kind, candidates) {
                return None;
            }
        }
    }

    validate_plan(grammar, &attempt.placed).ok()?;
    Some(attempt.placed)
}

/// Contrôle indépendant d'un plan : bornes de taille, types requis, un seul départ, boss en
/// cul-de-sac et strictement le plus loin du départ, connexité (chaque salle suit son parent par
/// une connexion réciproque), aucun chevauchement.
pub fn validate_plan(grammar: &FloorGrammar, rooms: &[PlannedRoom]) -> Result<(), String> {
    let n = rooms.len();
    if n < grammar.rooms.0 || n > grammar.rooms.1 {
        return Err(format!("{n} salles hors de {:?}", grammar.rooms));
    }
    let count = |kind: &str| rooms.iter().filter(|r| r.kind == kind).count();
    if count(&grammar.start) != 1 || rooms[0].kind != grammar.start {
        return Err("il faut exactement un départ, en tête".into());
    }
    for (kind, required) in &grammar.required {
        if count(kind) < *required {
            return Err(format!("type « {kind} » : {} sur {required}", count(kind)));
        }
    }
    let boss: Vec<_> = rooms.iter().enumerate().filter(|(_, r)| r.kind == grammar.boss).collect();
    let [(boss_index, boss_room)] = boss.as_slice() else {
        return Err(format!("{} boss au lieu d'un", boss.len()));
    };
    if boss_room.room.level_def.connections.len() != 1 {
        return Err("le boss n'est pas en cul-de-sac".into());
    }
    if rooms
        .iter()
        .enumerate()
        .any(|(i, r)| i != *boss_index && r.depth >= boss_room.depth)
    {
        return Err("une salle est aussi loin que le boss".into());
    }
    if boss_room.depth < grammar.min_boss_distance {
        return Err(format!("boss à {} salles du départ", boss_room.depth));
    }
    for (i, planned) in rooms.iter().enumerate().skip(1) {
        let parent = planned.parent.ok_or("salle sans parent")?;
        if parent >= i {
            return Err("un parent suit son enfant".into());
        }
        let (mine, theirs) = planned.via.ok_or("salle sans connexion")?;
        let linked = |room: &Room, index: usize, other_iid: &str, other_index: usize| {
            matches!(
                room.connections.get(index).and_then(|c| c.to.as_ref()),
                Some(ConnectionTo::Room((iid, idx))) if iid == other_iid && *idx == other_index
            )
        };
        if !linked(&planned.room, mine, &rooms[parent].room.level_iid, theirs)
            || !linked(&rooms[parent].room, theirs, &planned.room.level_iid, mine)
        {
            return Err("connexion non réciproque".into());
        }
        if planned.depth != rooms[parent].depth + 1 {
            return Err("profondeur incohérente".into());
        }
    }
    for (i, a) in rooms.iter().enumerate() {
        for b in &rooms[i + 1..] {
            if a.room.is_overlapping(&b.room) {
                return Err("deux salles se chevauchent".into());
            }
        }
    }
    Ok(())
}

/// Planifie un étage. `rng` est le générateur principal : il ne fournit que la graine de chaque
/// tentative (`next_u32`), ce qui rend le plan reproductible et l'échec explicite.
pub fn plan_floor(
    grammar: &FloorGrammar,
    context: &MapGenerationContext,
    rng: &mut RollbackRng,
) -> Result<FloorPlan, FloorError> {
    for kind in grammar.required.keys().chain([&grammar.filler]) {
        if !context.available_levels.iter().any(|l| kind_of(l) == kind) {
            return Err(FloorError::NoTemplate(kind.clone()));
        }
    }
    for attempt in 1..=grammar.max_attempts {
        let seed = rng.next_u32();
        if let Some(rooms) = try_attempt(grammar, context, seed) {
            bevy::log::debug!("grammaire d'étage : {} salles en {attempt} tentative(s)", rooms.len());
            return Ok(FloorPlan {
                rooms,
                attempts: attempt,
            });
        }
    }
    Err(FloorError::Exhausted {
        attempts: grammar.max_attempts,
    })
}

/// Connexions de la salle `index` du plan, clonées (pour [`super::IMapGeneration::get_next_room`]).
pub fn connection_of(room: &Room, index: usize) -> RoomConnection {
    room.connections[index].clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::{
        config::{MapGenerationConfig, MapGenerationMode},
        context::{populate_level_connections, Connection, Side},
        entity::location::EntityLocations,
    };
    use std::rc::Rc;

    fn locations() -> EntityLocations {
        EntityLocations {
            doors: vec![],
            sodas: vec![],
            player_spawns: vec![],
            zombie_spawns: vec![],
            crates: vec![],
            weapons: vec![],
            windows: vec![],
            character_spawns: vec![],
        }
    }

    /// Gabarit de 10 × 10 cases (160 px) dont les ouvertures de 2 cases sont au milieu des côtés
    /// listés.
    fn level(id: &str, kind: &str, spawn: bool, sides: &[Side]) -> AvailableLevel {
        AvailableLevel {
            level_id: id.into(),
            level_size: (10, 10),
            level_size_p: (160, 160),
            level_type: if spawn { LevelType::Spawn } else { LevelType::Normal },
            connections: sides
                .iter()
                .enumerate()
                .map(|(index, side)| Connection {
                    index,
                    size: 2,
                    side: *side,
                    starting_at: 4,
                    level_id: id.into(),
                    compatiable_levels: vec![],
                })
                .collect(),
            entity_locations: locations(),
            room_kind: Some(kind.into()),
        }
    }

    const ALL: [Side; 4] = [Side::N, Side::E, Side::S, Side::W];

    /// Neuf gabarits de combat, un départ à quatre portes, et pour chaque type spécial un
    /// gabarit en cul-de-sac par côté.
    fn templates() -> Vec<AvailableLevel> {
        let mut levels = vec![
            level("depart", "depart", true, &ALL),
            level("combat_croix", "combat", false, &ALL),
            level("combat_couloir_ew", "combat", false, &[Side::E, Side::W]),
            level("combat_couloir_ns", "combat", false, &[Side::N, Side::S]),
            level("combat_coude_ne", "combat", false, &[Side::N, Side::E]),
            level("combat_coude_sw", "combat", false, &[Side::S, Side::W]),
            level("combat_t_nes", "combat", false, &[Side::N, Side::E, Side::S]),
        ];
        for kind in ["boss", "boutique", "recompense"] {
            for side in ALL {
                levels.push(level(&format!("{kind}_{}", side.to_dir_str()), kind, false, &[side]));
            }
        }
        levels
    }

    fn context(seed: i32, levels: Vec<AvailableLevel>) -> MapGenerationContext {
        let mut levels = levels;
        populate_level_connections(&mut levels);
        MapGenerationContext {
            tile_size: (16, 16),
            level_size: (10, 10),
            available_levels: levels.into_iter().map(Rc::new).collect(),
            config: MapGenerationConfig {
                seed,
                max_width: 1000,
                max_heigth: 1000,
                max_room: 0,
                map_path: String::new(),
                mode: MapGenerationMode::Basic,
            },
        }
    }

    fn grammar() -> FloorGrammar {
        FloorGrammar {
            rooms: (8, 12),
            required: [("boss", 1), ("boutique", 1), ("recompense", 1)]
                .map(|(k, n)| (k.to_string(), n))
                .into(),
            start: "depart".into(),
            boss: "boss".into(),
            filler: "combat".into(),
            min_boss_distance: 3,
            max_attempts: 64,
        }
    }

    fn signature(plan: &FloorPlan) -> Vec<(String, i32, i32, String)> {
        plan.rooms
            .iter()
            .map(|p| (p.kind.clone(), p.room.position.0, p.room.position.1, p.room.level_iid.clone()))
            .collect()
    }

    /// 1 000 graines : connexité, types requis, boss à distance maximale et en cul-de-sac, aucun
    /// chevauchement, taille dans les bornes (le contrôle indépendant est `validate_plan`, rejoué
    /// ici). Rapporte le temps de génération (`--nocapture`).
    #[test]
    fn mille_graines_respectent_la_grammaire() {
        let grammar = grammar();
        let mut times = Vec::new();
        let mut attempts = 0;
        let mut sizes = std::collections::BTreeMap::<usize, usize>::new();
        for seed in 0..1000 {
            let context = context(seed, templates());
            let mut rng = RollbackRng::new(seed as u32);
            let started = std::time::Instant::now();
            let plan = plan_floor(&grammar, &context, &mut rng)
                .unwrap_or_else(|e| panic!("graine {seed} : {e}"));
            times.push(started.elapsed());
            attempts += plan.attempts as usize;
            validate_plan(&grammar, &plan.rooms).unwrap_or_else(|e| panic!("graine {seed} : {e}"));
            *sizes.entry(plan.rooms.len()).or_default() += 1;
            let boss = plan.rooms.iter().find(|p| p.kind == "boss").unwrap();
            assert_eq!(boss.room.level_def.connections.len(), 1, "graine {seed}");
            assert!(boss.depth >= 3, "graine {seed}");
            assert!(plan.rooms.iter().filter(|p| p.kind == "boutique").count() == 1);
            assert!(plan.rooms.iter().filter(|p| p.kind == "recompense").count() == 1);
        }
        let total: std::time::Duration = times.iter().sum();
        println!(
            "grammaire : 1000 graines, moyenne {:?}, max {:?}, tentatives moyennes {:.2}, tailles {:?}",
            total / 1000,
            times.iter().max().unwrap(),
            attempts as f64 / 1000.0,
            sizes
        );
    }

    #[test]
    fn meme_graine_meme_etage_graines_differentes_etages_differents() {
        let grammar = grammar();
        let run = |seed: i32| {
            let context = context(seed, templates());
            let mut rng = RollbackRng::new(seed as u32);
            signature(&plan_floor(&grammar, &context, &mut rng).unwrap())
        };
        assert_eq!(run(7), run(7));
        let distinct: std::collections::BTreeSet<_> = (0..50).map(run).collect();
        assert!(distinct.len() > 40, "{} étages distincts sur 50", distinct.len());
    }

    #[test]
    fn grammaire_impossible_echoue_explicitement_et_sans_boucle() {
        // aucun gabarit de boutique
        let sans_boutique: Vec<_> = templates()
            .into_iter()
            .filter(|l| kind_of(l) != "boutique")
            .collect();
        let mut rng = RollbackRng::new(1);
        assert_eq!(
            plan_floor(&grammar(), &context(1, sans_boutique), &mut rng).unwrap_err(),
            FloorError::NoTemplate("boutique".into())
        );
        // boss uniquement à deux connexions : jamais en cul-de-sac, tentatives épuisées
        let mut boss_double: Vec<_> = templates()
            .into_iter()
            .filter(|l| kind_of(l) != "boss")
            .collect();
        boss_double.push(level("boss_ew", "boss", false, &[Side::E, Side::W]));
        let mut rng = RollbackRng::new(1);
        let mut g = grammar();
        g.max_attempts = 5;
        assert_eq!(
            plan_floor(&g, &context(1, boss_double), &mut rng).unwrap_err(),
            FloorError::Exhausted { attempts: 5 }
        );
    }

    #[test]
    fn validate_plan_refuse_un_boss_trop_proche() {
        let grammar = grammar();
        let context = context(3, templates());
        let mut rng = RollbackRng::new(3);
        let mut plan = plan_floor(&grammar, &context, &mut rng).unwrap();
        let boss = plan.rooms.iter_mut().find(|p| p.kind == "boss").unwrap();
        boss.depth = 0;
        assert!(validate_plan(&grammar, &plan.rooms).is_err());
    }
}
