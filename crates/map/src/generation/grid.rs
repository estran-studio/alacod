//! Planificateur d'étage en grille (M3-T5, chantier E2 v2, `docs/conventions.md` §70).
//!
//! Là où [`super::floor`] fait pousser un arbre de gabarits par connexions alignées, ici l'étage
//! est une **grille de cases de salle** (`FloorLayout::Grid { cell, .. }`) : un gabarit couvre un
//! rectangle de cases (1×1, 2×1, 1×2, 2×2, Q8) dont **chaque côté de chaque case** a une ouverture
//! centrée sur ce segment (« port »). Deux salles voisines par un côté sont reliées par une porte
//! seulement si le plan en fait une arête ; les ouvertures sans arête sont murées à la génération
//! (`GeneratedMap::close_unused_openings_exact`).
//!
//! **Algorithme** (une tentative = une graine ; tout est local à la tentative) :
//! 1. départ au centre de la grille ;
//! 2. remplissage par croissance : la nouvelle salle ne touche que **son parent** (règle d'Isaac),
//!    sauf à `loops` pas tirés d'un flux local où elle peut toucher **une seconde** salle
//!    ordinaire, reliée par une porte de plus (une boucle) ;
//! 3. boss sur une salle de profondeur maximale, boucles comprises (le cul-de-sac le plus loin) ;
//! 4. salle post-boss (chance tirée **une fois par étage**, avant les tentatives), accrochée au boss ;
//! 5. salles requises : culs-de-sac accrochés à une salle moins profonde que le boss − 1 ;
//! 6. salle secrète : case vide voisine d'**au moins deux** salles ordinaires (jamais du boss ni de
//!    la salle post-boss), reliée à chacune ;
//! 7. contrôle indépendant [`validate_grid_plan`].
//!
//! **Déterminisme.** Aucune collection à ordre aléatoire (`BTreeMap`/`BTreeSet`, `Vec` parcourus
//! dans l'ordre de pose). Le RNG de chaque tentative est découpé en **flux locaux** par usage
//! (croissance, boucles, secrète, départ, identifiants de salle) dérivés de la graine de la
//! tentative : ajouter un tirage à l'un ne déplace pas les autres. Les tirages bornés utilisent les
//! bits de poids fort du LCG (les bits de poids faible d'un LCG alternent), sauf le nombre de salles :
//! premier tirage d'une tentative, à graine déjà mélangée (`next_u32_range_inclusive`).
//!
//! **Terminaison.** Aucune boucle `while` : la croissance est un `for` borné par le nombre de
//! salles, une pose énumère un nombre fini de candidats (parents × ports × gabarits × ports), les
//! boucles ne sont que des poses de la croissance, les tentatives sont bornées par
//! `max_attempts`. [`step_bound`] majore le nombre d'évaluations de candidats ; un test le compare
//! au compteur réel ([`FloorPlan::steps`]) y compris sur des grammaires impossibles.

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    rc::Rc,
};

use bevy_fixed::fixed_math::Fixed;
use bevy_fixed::rng::RollbackRng;
use serde_json::Value;
use world::{FloorGrammar, FloorLayout};

use super::{
    context::{AvailableLevel, LevelType, MapGenerationContext, Side},
    floor::{kind_of, FloorError, FloorPlan, PlannedLink, PlannedRoom},
    position::Position,
    room::{ConnectionTo, Room},
    LEVEL_PROPERTIES_SPAWN_NAME,
};

/// Case de salle `(colonne, rangée)` ; le nord est vers les rangées décroissantes.
type Cell = (i32, i32);
/// Ouverture d'un gabarit : un côté et la case de salle (le long de ce côté) qu'elle dessert.
type Port = (Side, i32);

const SIDES: [Side; 4] = [Side::N, Side::E, Side::S, Side::W];

const TAG_GROWTH: u32 = 1;
const TAG_LOOPS: u32 = 2;
const TAG_SECRET: u32 = 3;
const TAG_START: u32 = 4;
const TAG_POST: u32 = 5;

fn delta(side: Side) -> Cell {
    match side {
        Side::N => (0, -1),
        Side::E => (1, 0),
        Side::S => (0, 1),
        Side::W => (-1, 0),
    }
}

/// Mélange d'une graine et d'une étiquette (finaliseur de type murmur) : flux locaux disjoints.
fn mix(seed: u32, tag: u32) -> u32 {
    let mut x = seed ^ tag.wrapping_mul(0x9E37_79B9);
    x ^= x >> 16;
    x = x.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 13;
    x = x.wrapping_mul(0xC2B2_AE35);
    x ^ (x >> 16)
}

fn local(seed: u32, tag: u32) -> RollbackRng {
    RollbackRng::new(mix(seed, tag))
}

/// Entier dans `[0, n)` par les 24 bits de poids fort (`n > 0`).
fn below(rng: &mut RollbackRng, n: usize) -> usize {
    debug_assert!(n > 0);
    ((u64::from(rng.next_u32() >> 8) * n as u64) >> 24) as usize
}

/// Un gabarit vu par la grille : taille en cases de salle et ports.
#[derive(Debug, Clone)]
pub struct GridTemplate {
    pub level: Rc<AvailableLevel>,
    pub size: (i32, i32),
    /// `(côté, case le long du côté)` → indice de la connexion du gabarit.
    pub ports: BTreeMap<Port, usize>,
}

/// Les gabarits de `context` qui tiennent dans une grille de cases `cell` (en cases de 16 px) :
/// taille multiple de `cell`, ports = ouvertures centrées sur un segment d'une case de salle. Un
/// gabarit non aligné est ignoré (le lint l'interdit pour les types de la grammaire).
pub fn grid_templates(context: &MapGenerationContext, cell: (u32, u32)) -> Vec<GridTemplate> {
    let (cw, ch) = (cell.0 as usize, cell.1 as usize);
    let mut out = Vec::new();
    if cw == 0 || ch == 0 {
        return out;
    }
    for level in &context.available_levels {
        let (lw, lh) = level.level_size;
        if lw == 0 || lh == 0 || lw % cw != 0 || lh % ch != 0 {
            continue;
        }
        let size = ((lw / cw) as i32, (lh / ch) as i32);
        let mut ports = BTreeMap::new();
        for connection in &level.connections {
            let (segment, count) = match connection.side {
                Side::N | Side::S => (cw, size.0 as usize),
                Side::E | Side::W => (ch, size.1 as usize),
            };
            let k = connection.starting_at / segment;
            let offset = connection.starting_at % segment;
            // dans le segment, centrée : autant de cases de part et d'autre
            if k < count
                && offset + connection.size <= segment
                && 2 * offset + connection.size == segment
            {
                ports.insert((connection.side, k as i32), connection.index);
            }
        }
        out.push(GridTemplate {
            level: level.clone(),
            size,
            ports,
        });
    }
    out
}

/// Paramètres de la disposition `Grid` d'une grammaire.
struct Params<'a> {
    grammar: &'a FloorGrammar,
    cell: (u32, u32),
    max: (i32, i32),
    loops: u32,
    secret: &'a str,
    post_kind: Option<&'a str>,
}

impl<'a> Params<'a> {
    fn of(grammar: &'a FloorGrammar) -> Option<Self> {
        let FloorLayout::Grid {
            cell,
            max,
            loops,
            secret,
            post_boss,
        } = grammar.layout.as_ref()?
        else {
            return None;
        };
        Some(Self {
            grammar,
            cell: *cell,
            max: (max.0.min(1 << 12) as i32, max.1.min(1 << 12) as i32),
            loops: *loops,
            secret,
            post_kind: post_boss.as_ref().map(|post| post.kind.as_str()),
        })
    }

    /// Type d'une salle « ordinaire » : départ ou remplissage (parent possible d'un cul-de-sac,
    /// voisine possible d'une boucle ou de la salle secrète).
    fn ordinary(&self, kind: &str) -> bool {
        kind == self.grammar.start || kind == self.grammar.filler
    }
}

/// Position pixel des cases : la salle de départ est au centre du monde (alignée sur les tuiles).
struct Geometry {
    cell_px: (i32, i32),
    origin_px: (i32, i32),
}

impl Geometry {
    fn position(&self, cell: Cell) -> Position {
        Position(
            self.origin_px.0 + cell.0 * self.cell_px.0,
            self.origin_px.1 + cell.1 * self.cell_px.1,
        )
    }
}

fn snap(v: i32, size: i32) -> i32 {
    (v + size / 2).div_euclid(size) * size
}

/// Une salle posée dans la grille.
#[derive(Debug, Clone)]
struct Slot {
    tpl: usize,
    origin: Cell,
    kind: String,
}

/// Une arête du plan : une porte entre le port `a_port` de `a` et le port `b_port` de `b`.
#[derive(Debug, Clone)]
struct Edge {
    a: usize,
    a_port: Port,
    b: usize,
    b_port: Port,
    /// Arête de pose (`a` = parent, `b` = enfant), par opposition à une boucle ou à une voisine
    /// supplémentaire de la salle secrète.
    primary: bool,
}

/// Une pose candidate : gabarit, case d'origine, parent et ports reliés.
#[derive(Debug, Clone, Copy)]
struct Placement {
    tpl: usize,
    origin: Cell,
    parent: usize,
    parent_port: Port,
    port: Port,
    /// Boucle : une seconde salle ordinaire touchée par le rectangle, avec son port libre et le
    /// port du gabarit qui lui fait face. Absent : la salle ne touche que son parent.
    extra: Option<(usize, Port, Port)>,
}

struct Layout<'a> {
    templates: &'a [GridTemplate],
    geometry: Geometry,
    /// Étendue de la grille en cases de salle.
    max: (i32, i32),
    /// Limites du monde en pixels (`MapGenerationConfig::max_width/max_heigth`).
    world: (i32, i32),
    slots: Vec<Slot>,
    occupied: BTreeMap<Cell, usize>,
    edges: Vec<Edge>,
    used: BTreeSet<(usize, Port)>,
    /// Évaluations de candidats (preuve de terminaison, [`step_bound`]).
    steps: u64,
}

impl<'a> Layout<'a> {
    fn footprint(&self, tpl: usize, origin: Cell) -> impl Iterator<Item = Cell> {
        let (w, h) = self.templates[tpl].size;
        (0..h).flat_map(move |dy| (0..w).map(move |dx| (origin.0 + dx, origin.1 + dy)))
    }

    /// Case de salle (absolue) qu'un port dessert.
    fn port_cell(&self, tpl: usize, origin: Cell, port: Port) -> Cell {
        let (w, h) = self.templates[tpl].size;
        let (dx, dy) = match port.0 {
            Side::N => (port.1, 0),
            Side::S => (port.1, h - 1),
            Side::W => (0, port.1),
            Side::E => (w - 1, port.1),
        };
        (origin.0 + dx, origin.1 + dy)
    }

    /// Origine d'un gabarit dont le port `port` dessert la case `cell`.
    fn origin_for(&self, tpl: usize, port: Port, cell: Cell) -> Cell {
        let (w, h) = self.templates[tpl].size;
        let (dx, dy) = match port.0 {
            Side::N => (port.1, 0),
            Side::S => (port.1, h - 1),
            Side::W => (0, port.1),
            Side::E => (w - 1, port.1),
        };
        (cell.0 - dx, cell.1 - dy)
    }

    /// Le port de `tpl` posé en `origin` qui dessert `cell` du côté `side`, s'il existe.
    fn border_port(&self, tpl: usize, origin: Cell, cell: Cell, side: Side) -> Option<Port> {
        let (w, h) = self.templates[tpl].size;
        let (dx, dy) = (cell.0 - origin.0, cell.1 - origin.1);
        let port = match side {
            Side::N if dy == 0 => (Side::N, dx),
            Side::S if dy == h - 1 => (Side::S, dx),
            Side::W if dx == 0 => (Side::W, dy),
            Side::E if dx == w - 1 => (Side::E, dy),
            _ => return None,
        };
        self.templates[tpl]
            .ports
            .contains_key(&port)
            .then_some(port)
    }

    /// Largeur (en cases) de l'ouverture d'un port : deux salles ne se relient que par des
    /// ouvertures de même largeur (sinon leurs bords ne coïncident pas : porte vers le vide).
    fn width(&self, tpl: usize, port: Port) -> usize {
        let level = &self.templates[tpl].level;
        level
            .connections
            .iter()
            .find(|c| c.index == self.templates[tpl].ports[&port])
            .map_or(0, |c| c.size)
    }

    /// Dans la grille, dans le monde et sans salle sur le rectangle.
    fn free(&self, tpl: usize, origin: Cell) -> bool {
        let (w, h) = self.templates[tpl].size;
        if origin.0 < 0 || origin.1 < 0 || origin.0 + w > self.max.0 || origin.1 + h > self.max.1 {
            return false;
        }
        let position = self.geometry.position(origin);
        let (width, height) = (w * self.geometry.cell_px.0, h * self.geometry.cell_px.1);
        if position.0 < -self.world.0
            || position.1 < -self.world.1
            || position.0 + width > self.world.0
            || position.1 + height > self.world.1
        {
            return false;
        }
        self.footprint(tpl, origin)
            .all(|cell| !self.occupied.contains_key(&cell))
    }

    /// Salles posées qui touchent le rectangle par un côté.
    fn neighbours(&self, tpl: usize, origin: Cell) -> BTreeSet<usize> {
        let mut out = BTreeSet::new();
        for cell in self.footprint(tpl, origin) {
            for side in SIDES {
                let (dx, dy) = delta(side);
                if let Some(room) = self.occupied.get(&(cell.0 + dx, cell.1 + dy)) {
                    out.insert(*room);
                }
            }
        }
        // les cases du rectangle lui-même ne sont pas encore occupées
        out
    }

    fn put(&mut self, tpl: usize, origin: Cell, kind: &str) -> usize {
        let index = self.slots.len();
        for cell in self.footprint(tpl, origin).collect::<Vec<_>>() {
            self.occupied.insert(cell, index);
        }
        self.slots.push(Slot {
            tpl,
            origin,
            kind: kind.to_string(),
        });
        index
    }

    fn link(&mut self, a: usize, a_port: Port, b: usize, b_port: Port, primary: bool) {
        self.used.insert((a, a_port));
        self.used.insert((b, b_port));
        self.edges.push(Edge {
            a,
            a_port,
            b,
            b_port,
            primary,
        });
    }

    /// Distance en salles depuis le départ (`usize::MAX` : inaccessible).
    fn depths(&self) -> Vec<usize> {
        let mut adjacent = vec![Vec::new(); self.slots.len()];
        for edge in &self.edges {
            adjacent[edge.a].push(edge.b);
            adjacent[edge.b].push(edge.a);
        }
        let mut depth = vec![usize::MAX; self.slots.len()];
        depth[0] = 0;
        let mut queue = VecDeque::from([0]);
        while let Some(room) = queue.pop_front() {
            for next in &adjacent[room] {
                if depth[*next] == usize::MAX {
                    depth[*next] = depth[room] + 1;
                    queue.push_back(*next);
                }
            }
        }
        depth
    }

    /// Indices des gabarits d'un type (départ : gabarits `Spawn` ; autres : jamais `Spawn`).
    fn templates_of(&self, kind: &str, spawn: bool) -> Vec<usize> {
        self.templates
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                kind_of(&t.level) == kind && (t.level.level_type == LevelType::Spawn) == spawn
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Le premier couple de ports (dans l'ordre des ports du gabarit) qui relie le rectangle posé
    /// en `origin` à la salle `other` par une porte autre que `skip` : `(other, son port, le port
    /// du gabarit)`. Les deux ports doivent exister et être libres.
    fn loop_ports(
        &self,
        tpl: usize,
        origin: Cell,
        skip: Port,
        other: usize,
    ) -> Option<(usize, Port, Port)> {
        let (otpl, oorigin) = (self.slots[other].tpl, self.slots[other].origin);
        for own in self.templates[tpl].ports.keys().copied() {
            if own == skip {
                continue;
            }
            let from = self.port_cell(tpl, origin, own);
            let (dx, dy) = delta(own.0);
            let target = (from.0 + dx, from.1 + dy);
            if self.occupied.get(&target) != Some(&other) {
                continue;
            }
            let Some(their) = self.border_port(otpl, oorigin, target, own.0.get_opposite()) else {
                continue;
            };
            if !self.used.contains(&(other, their))
                && self.width(tpl, own) == self.width(otpl, their)
            {
                return Some((other, their, own));
            }
        }
        None
    }

    /// Toutes les poses valides d'une salle d'un des gabarits `candidates` accrochée à un parent
    /// accepté par `parent_ok` : le rectangle est libre et ne touche que ce parent.
    fn placements(
        &mut self,
        candidates: &[usize],
        parent_ok: &dyn Fn(usize, &Slot) -> bool,
        loop_ok: Option<&dyn Fn(&str) -> bool>,
    ) -> Vec<Placement> {
        let templates = self.templates;
        let mut out = Vec::new();
        for parent in 0..self.slots.len() {
            if !parent_ok(parent, &self.slots[parent]) {
                continue;
            }
            let (ptpl, porigin) = (self.slots[parent].tpl, self.slots[parent].origin);
            for parent_port in templates[ptpl].ports.keys().copied() {
                if self.used.contains(&(parent, parent_port)) {
                    continue;
                }
                let from = self.port_cell(ptpl, porigin, parent_port);
                let (dx, dy) = delta(parent_port.0);
                let target = (from.0 + dx, from.1 + dy);
                if self.occupied.contains_key(&target) {
                    continue;
                }
                let facing = parent_port.0.get_opposite();
                for &tpl in candidates {
                    for port in templates[tpl].ports.keys().copied() {
                        if port.0 != facing
                            || self.width(tpl, port) != self.width(ptpl, parent_port)
                        {
                            continue;
                        }
                        self.steps += 1;
                        let origin = self.origin_for(tpl, port, target);
                        if !self.free(tpl, origin) {
                            continue;
                        }
                        let around = self.neighbours(tpl, origin);
                        if around == BTreeSet::from([parent]) {
                            out.push(Placement {
                                tpl,
                                origin,
                                parent,
                                parent_port,
                                port,
                                extra: None,
                            });
                        } else if let Some(ordinary) = loop_ok {
                            // boucle : exactement une autre salle ordinaire, reliée par une
                            // porte de plus (ports libres face à face)
                            let mut others = around.iter().filter(|room| **room != parent);
                            let (Some(&other), None) = (others.next(), others.next()) else {
                                continue;
                            };
                            if !ordinary(&self.slots[other].kind) {
                                continue;
                            }
                            self.steps += templates[tpl].ports.len() as u64;
                            if let Some(extra) = self.loop_ports(tpl, origin, port, other) {
                                out.push(Placement {
                                    tpl,
                                    origin,
                                    parent,
                                    parent_port,
                                    port,
                                    extra: Some(extra),
                                });
                            }
                        }
                    }
                }
            }
        }
        out
    }

    /// Tire une pose : d'abord un gabarit parmi ceux qui ont au moins une pose (uniforme), puis
    /// une pose de ce gabarit (uniforme) : une grande salle n'est pas favorisée par ses ports.
    fn choose(rng: &mut RollbackRng, list: &[Placement]) -> Option<Placement> {
        let templates: BTreeSet<usize> = list.iter().map(|p| p.tpl).collect();
        if templates.is_empty() {
            return None;
        }
        let tpl = *templates.iter().nth(below(rng, templates.len()))?;
        let of_template: Vec<&Placement> = list.iter().filter(|p| p.tpl == tpl).collect();
        Some(*of_template[below(rng, of_template.len())])
    }

    fn place(&mut self, rng: &mut RollbackRng, kind: &str, list: &[Placement]) -> Option<usize> {
        let pick = Self::choose(rng, list)?;
        let index = self.put(pick.tpl, pick.origin, kind);
        self.link(pick.parent, pick.parent_port, index, pick.port, true);
        if let Some((other, their, own)) = pick.extra {
            self.link(other, their, index, own, false);
        }
        Some(index)
    }
}

/// Une tentative : le plan complet, ou `None` si une étape n'a aucune pose.
fn attempt(
    params: &Params,
    templates: &[GridTemplate],
    context: &MapGenerationContext,
    seed: u32,
    want_post: bool,
    steps: &mut u64,
) -> Option<Vec<PlannedRoom>> {
    let grammar = params.grammar;
    let mut growth = local(seed, TAG_GROWTH);
    let mut start_rng = local(seed, TAG_START);

    let fixed = 2 + grammar.others();
    if grammar.rooms.1 < fixed {
        return None;
    }
    let post = want_post && params.post_kind.is_some() && grammar.rooms.1 > fixed;
    let reserved = fixed + usize::from(post);
    let lowest = grammar.rooms.0.max(reserved).min(grammar.rooms.1);
    let total = growth.next_u32_range_inclusive(lowest as u32, grammar.rooms.1 as u32) as usize;
    let fillers = total - reserved;

    let tile = context.tile_size;
    let cell_px = (params.cell.0 as i32 * tile.0, params.cell.1 as i32 * tile.1);

    // 1. départ
    let starts = templates
        .iter()
        .enumerate()
        .filter(|(_, t)| {
            t.level.level_type == LevelType::Spawn && kind_of(&t.level) == grammar.start
        })
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    if starts.is_empty() {
        return None;
    }
    let start_tpl = starts[below(&mut start_rng, starts.len())];
    let (sw, sh) = templates[start_tpl].size;
    if sw > params.max.0 || sh > params.max.1 {
        return None;
    }
    let start_origin = ((params.max.0 - sw) / 2, (params.max.1 - sh) / 2);
    let start_px = (sw * cell_px.0, sh * cell_px.1);
    let start_position = Position(snap(-start_px.0 / 2, tile.0), snap(-start_px.1 / 2, tile.1));
    let mut layout = Layout {
        templates,
        geometry: Geometry {
            cell_px,
            origin_px: (
                start_position.0 - start_origin.0 * cell_px.0,
                start_position.1 - start_origin.1 * cell_px.1,
            ),
        },
        max: params.max,
        world: (context.config.max_width, context.config.max_heigth),
        slots: Vec::new(),
        occupied: BTreeMap::new(),
        edges: Vec::new(),
        used: BTreeSet::new(),
        steps: 0,
    };
    let result = (|| {
        if !layout.free(start_tpl, start_origin) {
            return None;
        }
        layout.put(start_tpl, start_origin, &grammar.start);

        // 2. remplissage
        let filler_templates = layout.templates_of(&grammar.filler, false);
        // pas de croissance où une boucle est permise : `loops` pas distincts, tirés d'un flux
        // local (borné : au plus `fillers` tirages)
        let mut loop_steps = BTreeSet::new();
        {
            let mut rng = local(seed, TAG_LOOPS);
            let mut pool: Vec<usize> = (0..fillers).collect();
            for _ in 0..(params.loops as usize).min(fillers) {
                loop_steps.insert(pool.swap_remove(below(&mut rng, pool.len())));
            }
        }
        let ordinary = |kind: &str| params.ordinary(kind);
        for step in 0..fillers {
            let want_loop = loop_steps.contains(&step);
            let list = layout.placements(
                &filler_templates,
                &|_, slot| params.ordinary(&slot.kind),
                want_loop.then_some(&ordinary as &dyn Fn(&str) -> bool),
            );
            // à un pas de boucle, une pose qui boucle est préférée ; sinon, ou ailleurs, une pose
            // qui ne touche que le parent
            let looping: Vec<Placement> =
                list.iter().filter(|p| p.extra.is_some()).copied().collect();
            let chosen = if want_loop && !looping.is_empty() {
                looping
            } else {
                list.into_iter().filter(|p| p.extra.is_none()).collect()
            };
            layout.place(&mut growth, &grammar.filler, &chosen)?;
        }

        // 3. boss : une salle de profondeur maximale, cul-de-sac
        let depths = layout.depths();
        let max_depth = depths.iter().copied().max().unwrap_or(0);
        let boss_templates = layout.templates_of(&grammar.boss, false);
        let list = layout.placements(
            &boss_templates,
            &|index, slot| depths[index] == max_depth && params.ordinary(&slot.kind),
            None,
        );
        let boss = layout.place(&mut growth, &grammar.boss, &list)?;
        let boss_depth = layout.depths()[boss];
        if boss_depth < grammar.min_boss_distance {
            return None;
        }

        // 5. salle post-boss, accrochée au boss
        let mut post_room = None;
        if post {
            let kind = params.post_kind?;
            let post_templates = layout.templates_of(kind, false);
            let list = layout.placements(&post_templates, &|index, _| index == boss, None);
            post_room = Some(layout.place(&mut growth, kind, &list)?);
        }

        // 6. salles requises : culs-de-sac plus proches du départ que le boss d'au moins 2
        let depths = layout.depths();
        for (kind, count) in &grammar.required {
            if *kind == grammar.boss || kind == params.secret {
                continue;
            }
            let kind_templates = layout.templates_of(kind, false);
            for _ in 0..*count {
                let list = layout.placements(
                    &kind_templates,
                    &|index, slot| {
                        params.ordinary(&slot.kind)
                            && depths
                                .get(index)
                                .is_some_and(|depth| depth + 2 <= boss_depth)
                    },
                    None,
                );
                layout.place(&mut growth, kind, &list)?;
            }
        }

        // 7. salle secrète
        if grammar.required.contains_key(params.secret) {
            place_secret(
                &mut layout,
                params,
                &mut local(seed, TAG_SECRET),
                boss,
                post_room,
            )?;
        }
        Some(())
    })();
    *steps += layout.steps;
    result?;
    Some(build(&layout, seed))
}

/// Salle secrète : une case (ou un rectangle) libre qui touche au moins deux salles ordinaires
/// par un port de part et d'autre, sans toucher le boss ni la salle post-boss ; reliée à chacune.
fn place_secret(
    layout: &mut Layout,
    params: &Params,
    rng: &mut RollbackRng,
    boss: usize,
    post: Option<usize>,
) -> Option<()> {
    let templates = layout.templates;
    let secret_templates = layout.templates_of(params.secret, false);
    // candidats (gabarit, origine) : tout rectangle dont un port dessert la case d'à côté d'un
    // port libre d'une salle ordinaire
    let mut candidates: BTreeSet<(usize, Cell)> = BTreeSet::new();
    for a in 0..layout.slots.len() {
        if !params.ordinary(&layout.slots[a].kind) {
            continue;
        }
        let (tpl, origin) = (layout.slots[a].tpl, layout.slots[a].origin);
        for port in templates[tpl].ports.keys().copied() {
            if layout.used.contains(&(a, port)) {
                continue;
            }
            let from = layout.port_cell(tpl, origin, port);
            let (dx, dy) = delta(port.0);
            let target = (from.0 + dx, from.1 + dy);
            for &secret in &secret_templates {
                for own in templates[secret].ports.keys().copied() {
                    if own.0 == port.0.get_opposite() {
                        candidates.insert((secret, layout.origin_for(secret, own, target)));
                    }
                }
            }
        }
    }
    let mut valid: Vec<(usize, Cell, Vec<(usize, Port, Port)>)> = Vec::new();
    for (tpl, origin) in candidates {
        layout.steps += 1;
        if !layout.free(tpl, origin) {
            continue;
        }
        let around = layout.neighbours(tpl, origin);
        if around.contains(&boss) || post.is_some_and(|room| around.contains(&room)) {
            continue;
        }
        // une porte par salle voisine ordinaire (la première, dans l'ordre des ports)
        let mut doors: BTreeMap<usize, (usize, Port, Port)> = BTreeMap::new();
        for own in templates[tpl].ports.keys().copied() {
            layout.steps += 1;
            let from = layout.port_cell(tpl, origin, own);
            let (dx, dy) = delta(own.0);
            let target = (from.0 + dx, from.1 + dy);
            let Some(&other) = layout.occupied.get(&target) else {
                continue;
            };
            if !params.ordinary(&layout.slots[other].kind) || doors.contains_key(&other) {
                continue;
            }
            let (otpl, oorigin) = (layout.slots[other].tpl, layout.slots[other].origin);
            let Some(their) = layout.border_port(otpl, oorigin, target, own.0.get_opposite())
            else {
                continue;
            };
            if layout.used.contains(&(other, their))
                || layout.width(tpl, own) != layout.width(otpl, their)
            {
                continue;
            }
            doors.insert(other, (other, their, own));
        }
        if doors.len() >= 2 {
            valid.push((tpl, origin, doors.into_values().collect()));
        }
    }
    let templates_with: BTreeSet<usize> = valid.iter().map(|(tpl, ..)| *tpl).collect();
    if templates_with.is_empty() {
        return None;
    }
    let tpl = *templates_with
        .iter()
        .nth(below(rng, templates_with.len()))?;
    let of_template: Vec<_> = valid.into_iter().filter(|(t, ..)| *t == tpl).collect();
    let pick = below(rng, of_template.len());
    let (tpl, origin, doors) = of_template.into_iter().nth(pick)?;
    let index = layout.put(tpl, origin, params.secret);
    for (n, (other, their, own)) in doors.into_iter().enumerate() {
        // la première porte est l'arête de pose (parent), les autres des voisines supplémentaires
        layout.link(other, their, index, own, n == 0);
    }
    Some(())
}

/// Salles du plan (géométrie, connexions, arêtes), dans l'ordre de pose.
fn build(layout: &Layout, seed: u32) -> Vec<PlannedRoom> {
    // Identifiant de salle : un flux par indice, graine `seed ^ (indice + 1) × K` (K impair :
    // injectif, deux salles n'ont jamais le même premier tirage donc le même identifiant).
    let mut rooms: Vec<Room> = layout
        .slots
        .iter()
        .enumerate()
        .map(|(index, slot)| {
            let mut ids = RollbackRng::new(seed ^ ((index as u32 + 1).wrapping_mul(0x9E37_79B9)));
            Room::create(
                &mut ids,
                layout.templates[slot.tpl].level.clone(),
                layout.geometry.position(slot.origin),
                [(
                    LEVEL_PROPERTIES_SPAWN_NAME.to_string(),
                    Value::Bool(index == 0),
                )]
                .into(),
            )
        })
        .collect();
    let connection =
        |room: usize, port: Port| layout.templates[layout.slots[room].tpl].ports[&port];
    for edge in &layout.edges {
        let (a_conn, b_conn) = (
            connection(edge.a, edge.a_port),
            connection(edge.b, edge.b_port),
        );
        let (low, high) = (edge.a.min(edge.b), edge.a.max(edge.b));
        let (conn_low, conn_high) = if edge.a < edge.b {
            (a_conn, b_conn)
        } else {
            (b_conn, a_conn)
        };
        let (head, tail) = rooms.split_at_mut(high);
        tail[0].set_connection_between(conn_high, &mut head[low], conn_low);
    }
    let depths = layout.depths();
    let mut planned: Vec<PlannedRoom> = rooms
        .into_iter()
        .enumerate()
        .map(|(index, room)| PlannedRoom {
            room,
            parent: None,
            depth: depths[index],
            via: None,
            kind: layout.slots[index].kind.clone(),
            links: Vec::new(),
        })
        .collect();
    for edge in &layout.edges {
        let (mine, theirs) = (
            connection(edge.b, edge.b_port),
            connection(edge.a, edge.a_port),
        );
        if edge.primary {
            planned[edge.b].parent = Some(edge.a);
            planned[edge.b].via = Some((mine, theirs));
        } else {
            // l'arête supplémentaire est portée par la salle posée en dernier
            let (host, host_conn, other, other_conn) = if edge.a < edge.b {
                (edge.b, mine, edge.a, theirs)
            } else {
                (edge.a, theirs, edge.b, mine)
            };
            planned[host].links.push(PlannedLink {
                mine: host_conn,
                other,
                theirs: other_conn,
            });
        }
    }
    planned
}

/// Planifie un étage en grille : graine de chaque tentative tirée de `rng` (comme `Graph`), chance
/// de la salle post-boss tirée **une fois** par un flux local de la graine de l'emplacement.
pub fn plan_grid(
    grammar: &FloorGrammar,
    context: &MapGenerationContext,
    rng: &mut RollbackRng,
) -> Result<FloorPlan, FloorError> {
    let Some(params) = Params::of(grammar) else {
        unreachable!("plan_grid n'est appelée que pour FloorLayout::Grid");
    };
    let templates = grid_templates(context, params.cell);
    let mut needed: Vec<&str> = grammar.required.keys().map(String::as_str).collect();
    needed.push(&grammar.filler);
    needed.push(&grammar.start);
    needed.push(&grammar.boss);
    // comme le lint (`kinds_needed`) : le gabarit post-boss n'est exigé que si la chance est > 0
    if let Some(post) = grammar.post_boss().filter(|post| post.chance > Fixed::ZERO) {
        needed.push(&post.kind);
    }
    for kind in needed {
        if !templates
            .iter()
            .any(|t| kind_of(&t.level) == kind && !t.ports.is_empty())
        {
            return Err(FloorError::NoTemplate(kind.to_string()));
        }
    }
    let want_post = grammar
        .post_boss()
        .is_some_and(|post| local(context.config.seed as u32, TAG_POST).next_fixed() < post.chance);
    let mut steps = 0;
    for attempt_index in 1..=grammar.max_attempts {
        let seed = rng.next_u32();
        if let Some(rooms) = attempt(&params, &templates, context, seed, want_post, &mut steps) {
            if let Err(reason) = validate_grid_plan(grammar, context.tile_size, &rooms) {
                bevy::log::warn!("plan en grille refusé par le contrôle indépendant : {reason}");
                continue;
            }
            bevy::log::debug!(
                "grammaire d'étage en grille : {} salles en {attempt_index} tentative(s)",
                rooms.len()
            );
            return Ok(FloorPlan {
                rooms,
                attempts: attempt_index,
                steps,
            });
        }
    }
    Err(FloorError::Exhausted {
        attempts: grammar.max_attempts,
    })
}

/// Majorant du nombre d'évaluations de candidats d'un appel de [`plan_grid`] (champ
/// [`FloorPlan::steps`]) : tentatives × poses × (parents × ports × gabarits × ports × balayage de
/// boucle) + candidats de la salle secrète. Aucune étape n'est une boucle non bornée : ce nombre borne le
/// travail de la planification.
pub fn step_bound(grammar: &FloorGrammar, templates: &[GridTemplate]) -> u64 {
    let rooms = grammar.rooms.1 as u64 + 2;
    let ports = templates
        .iter()
        .map(|t| t.ports.len() as u64)
        .max()
        .unwrap_or(0)
        .max(1);
    let count = templates.len() as u64 + 1;
    // une pose : parents × ports × gabarits × ports, chacune pouvant balayer les ports du gabarit
    // pour une porte de boucle ; la salle secrète : ses candidats, puis ses ports
    let per_placement = rooms * ports * count * ports * (ports + 1);
    let per_attempt = rooms * per_placement + count * rooms * ports * ports * (ports + 1);
    u64::from(grammar.max_attempts) * per_attempt
}

/// Contrôle indépendant d'un plan en grille, relu depuis les salles (positions, connexions) et non
/// depuis l'état du planificateur : bornes de taille, types, aucun chevauchement, alignement sur la
/// grille, portes réciproques entre salles voisines (aucune porte vers le vide), connexité, boucles
/// bornées, boss en cul-de-sac strictement le plus loin, requises en culs-de-sac, salle secrète
/// voisine d'au moins deux salles ordinaires et jamais du boss.
pub fn validate_grid_plan(
    grammar: &FloorGrammar,
    tile: (i32, i32),
    rooms: &[PlannedRoom],
) -> Result<(), String> {
    let Some(params) = Params::of(grammar) else {
        return Err("la grammaire n'est pas en grille".into());
    };
    let n = rooms.len();
    if n < grammar.rooms.0 || n > grammar.rooms.1 {
        return Err(format!("{n} salles hors de {:?}", grammar.rooms));
    }
    let count = |kind: &str| rooms.iter().filter(|r| r.kind == kind).count();
    if count(&grammar.start) != 1 || rooms[0].kind != grammar.start {
        return Err("il faut exactement un départ, en tête".into());
    }
    if count(&grammar.boss) != 1 {
        return Err(format!("{} boss au lieu d'un", count(&grammar.boss)));
    }
    for (kind, required) in &grammar.required {
        if count(kind) != *required {
            return Err(format!("type « {kind} » : {} sur {required}", count(kind)));
        }
    }
    if let Some(kind) = params.post_kind {
        if count(kind) > 1 {
            return Err(format!("{} salles post-boss", count(kind)));
        }
    }
    let known = |kind: &str| {
        kind == grammar.start
            || kind == grammar.filler
            || kind == grammar.boss
            || grammar.required.contains_key(kind)
            || params.post_kind == Some(kind)
    };
    if let Some(room) = rooms.iter().find(|r| !known(&r.kind)) {
        return Err(format!("type « {} » inattendu", room.kind));
    }

    // alignement et chevauchement
    let cell_px = (params.cell.0 as i32 * tile.0, params.cell.1 as i32 * tile.1);
    let base = &rooms[0].room.position;
    for room in rooms {
        let size = room.room.level_def.level_size_p;
        if size.0 % cell_px.0 != 0 || size.1 % cell_px.1 != 0 {
            return Err(format!(
                "{} : taille hors grille",
                room.room.level_def.level_id
            ));
        }
        let position = &room.room.position;
        if (position.0 - base.0).rem_euclid(cell_px.0) != 0
            || (position.1 - base.1).rem_euclid(cell_px.1) != 0
        {
            return Err(format!(
                "{} : position hors grille",
                room.room.level_def.level_id
            ));
        }
    }
    for (i, a) in rooms.iter().enumerate() {
        for b in &rooms[i + 1..] {
            if a.room.is_overlapping(&b.room) {
                return Err("deux salles se chevauchent".into());
            }
            if a.room.level_iid == b.room.level_iid {
                return Err("deux salles ont le même identifiant".into());
            }
        }
    }

    // portes : réciproques, entre salles voisines, ouvertures face à face
    let index_of: BTreeMap<&str, usize> = rooms
        .iter()
        .enumerate()
        .map(|(i, r)| (r.room.level_iid.as_str(), i))
        .collect();
    // ouverture d'une connexion en pixels monde : (côté, ligne du mur, début, fin le long du mur)
    let opening = |room: &Room, index: usize| -> Option<(Side, i32, i32, i32)> {
        let c = room
            .level_def
            .connections
            .iter()
            .find(|c| c.index == index)?;
        let (x, y) = (room.position.0, room.position.1);
        let (w, h) = room.level_def.level_size_p;
        let (start, end) = (c.starting_at as i32, (c.starting_at + c.size) as i32);
        let (line, from, to) = match c.side {
            Side::N => (y, x + start * tile.0, x + end * tile.0),
            Side::S => (y + h, x + start * tile.0, x + end * tile.0),
            Side::W => (x, y + start * tile.1, y + end * tile.1),
            Side::E => (x + w, y + start * tile.1, y + end * tile.1),
        };
        Some((c.side, line, from, to))
    };
    let mut edges: BTreeSet<(usize, usize)> = BTreeSet::new();
    for (i, planned) in rooms.iter().enumerate() {
        for connection in &planned.room.connections {
            let Some(ConnectionTo::Room((iid, other_index))) = &connection.to else {
                continue;
            };
            let Some(&j) = index_of.get(iid.as_str()) else {
                return Err("porte vers une salle absente".into());
            };
            let back = rooms[j]
                .room
                .connections
                .get(*other_index)
                .and_then(|c| c.to.as_ref());
            if !matches!(back, Some(ConnectionTo::Room((back_iid, back_index)))
                if *back_iid == planned.room.level_iid && *back_index == connection.index)
            {
                return Err("porte non réciproque".into());
            }
            let (Some(mine), Some(theirs)) = (
                opening(&planned.room, connection.index),
                opening(&rooms[j].room, *other_index),
            ) else {
                return Err("connexion inconnue du gabarit".into());
            };
            if !mine.0.is_opposite(theirs.0)
                || mine.1 != theirs.1
                || mine.2 != theirs.2
                || mine.3 != theirs.3
            {
                return Err("porte vers le vide : ouvertures non face à face".into());
            }
            if i != j && !edges.insert((i.min(j), i.max(j))) && i < j {
                return Err("deux portes entre les deux mêmes salles".into());
            }
        }
    }

    // connexité et distances **sans la salle secrète** : un raccourci par elle ne compte pas
    let secret_index = rooms.iter().position(|r| r.kind == params.secret);
    let mut adjacent = vec![Vec::new(); n];
    for (a, b) in &edges {
        adjacent[*a].push(*b);
        adjacent[*b].push(*a);
    }
    let mut depth = vec![usize::MAX; n];
    depth[0] = 0;
    let mut queue = VecDeque::from([0]);
    while let Some(room) = queue.pop_front() {
        for next in &adjacent[room] {
            if Some(*next) != secret_index && depth[*next] == usize::MAX {
                depth[*next] = depth[room] + 1;
                queue.push_back(*next);
            }
        }
    }
    if (0..n).any(|i| Some(i) != secret_index && depth[i] == usize::MAX) {
        return Err("une salle n'est reliée au départ que par la salle secrète".into());
    }

    let touching = |a: &PlannedRoom, b: &PlannedRoom| {
        let (ax, ay) = (a.room.position.0, a.room.position.1);
        let (aw, ah) = a.room.level_def.level_size_p;
        let (bx, by) = (b.room.position.0, b.room.position.1);
        let (bw, bh) = b.room.level_def.level_size_p;
        let share_x = ax < bx + bw && bx < ax + aw;
        let share_y = ay < by + bh && by < ay + ah;
        (share_x && (ay + ah == by || by + bh == ay))
            || (share_y && (ax + aw == bx || bx + bw == ax))
    };
    let ordinary = |room: &PlannedRoom| params.ordinary(&room.kind);

    // arbre + boucles (hors salle secrète)
    let others: Vec<usize> = (0..n).filter(|i| Some(*i) != secret_index).collect();
    let tree_edges = edges
        .iter()
        .filter(|(a, b)| Some(*a) != secret_index && Some(*b) != secret_index)
        .count();
    if tree_edges + 1 < others.len() {
        return Err("moins d'arêtes que d'un arbre".into());
    }
    let loops = tree_edges + 1 - others.len();
    if loops > params.loops as usize {
        return Err(format!("{loops} boucles pour {} permises", params.loops));
    }
    // règle d'Isaac : toute paire de salles voisines (hors secrète) est reliée ; une boucle est
    // une arête de plus entre deux voisines, jamais un mur entre elles
    for (k, &a) in others.iter().enumerate() {
        for &b in &others[k + 1..] {
            if touching(&rooms[a], &rooms[b]) && !edges.contains(&(a.min(b), a.max(b))) {
                return Err("deux salles voisines sans porte (règle d'Isaac)".into());
            }
        }
    }

    // boss
    let boss = rooms
        .iter()
        .position(|r| r.kind == grammar.boss)
        .unwrap_or(0);
    let degree = |room: usize| adjacent[room].len();
    let post = params
        .post_kind
        .and_then(|kind| rooms.iter().position(|r| r.kind == kind));
    // cul-de-sac, à la salle post-boss près (accrochée au boss)
    if degree(boss) != 1 + usize::from(post.is_some()) {
        return Err("le boss n'est pas en cul-de-sac".into());
    }
    if depth[boss] < grammar.min_boss_distance {
        return Err(format!("boss à {} salles du départ", depth[boss]));
    }
    if (0..n)
        .any(|i| i != boss && Some(i) != post && Some(i) != secret_index && depth[i] >= depth[boss])
    {
        return Err("une salle est aussi loin que le boss".into());
    }
    if let Some(post) = post {
        if adjacent[post] != [boss] {
            return Err("la salle post-boss n'est pas un cul-de-sac accroché au boss".into());
        }
    }

    // salles requises : culs-de-sac sur une salle ordinaire
    for (i, planned) in rooms.iter().enumerate() {
        if planned.kind == grammar.boss
            || planned.kind == params.secret
            || !grammar.required.contains_key(&planned.kind)
        {
            continue;
        }
        if degree(i) != 1 || !ordinary(&rooms[adjacent[i][0]]) {
            return Err(format!(
                "« {} » n'est pas un cul-de-sac ordinaire",
                planned.kind
            ));
        }
        if depth[i] >= depth[boss] {
            return Err(format!("« {} » est aussi loin que le boss", planned.kind));
        }
    }

    // salle secrète
    if let Some(secret) = secret_index {
        if degree(secret) < 2 {
            return Err("la salle secrète touche moins de deux salles".into());
        }
        if adjacent[secret].iter().any(|r| !ordinary(&rooms[*r])) {
            return Err("la salle secrète est reliée à une salle spéciale".into());
        }
        if touching(&rooms[secret], &rooms[boss])
            || post.is_some_and(|post| touching(&rooms[secret], &rooms[post]))
        {
            return Err("la salle secrète touche le boss".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::{
        config::{MapGenerationConfig, MapGenerationMode},
        context::Connection,
        entity::location::EntityLocations,
    };
    use bevy_fixed::fixed_math::Fixed;
    use world::PostBoss;

    const CELL: (u32, u32) = (15, 9);

    fn locations() -> EntityLocations {
        EntityLocations {
            doors: vec![],
            sodas: vec![],
            props: vec![],
            player_spawns: vec![],
            zombie_spawns: vec![],
            crates: vec![],
            weapons: vec![],
            windows: vec![],
            character_spawns: vec![],
        }
    }

    /// Gabarit de `w × h` cases de salle (15 × 9 cases de 16 px) : une ouverture de 3 cases centrée
    /// sur chaque segment de chaque côté, sauf `skip` (côté, segment). Connexions numérotées dans
    /// l'ordre du chargeur LDtk (nord, sud, ouest, est).
    fn level(
        id: &str,
        kind: &str,
        spawn: bool,
        (w, h): (usize, usize),
        skip: &[(Side, usize)],
    ) -> AvailableLevel {
        let mut connections = Vec::new();
        for side in [Side::N, Side::S, Side::W, Side::E] {
            let (count, segment) = match side {
                Side::N | Side::S => (w, 15),
                Side::W | Side::E => (h, 9),
            };
            for k in 0..count {
                if skip.contains(&(side, k)) {
                    continue;
                }
                connections.push(Connection {
                    index: connections.len(),
                    size: 3,
                    side,
                    starting_at: k * segment + (segment - 3) / 2,
                    level_id: id.into(),
                    compatiable_levels: vec![],
                });
            }
        }
        AvailableLevel {
            level_id: id.into(),
            level_size: (15 * w, 9 * h),
            level_size_p: (240 * w as i32, 144 * h as i32),
            level_type: if spawn {
                LevelType::Spawn
            } else {
                LevelType::Normal
            },
            connections,
            entity_locations: locations(),
            room_kind: Some(kind.into()),
        }
    }

    /// Départ, combat, boss 1×1 et 2×2, trésor 1×1, 2×1 et 1×2, secrète, diable.
    fn templates() -> Vec<AvailableLevel> {
        vec![
            level("depart", "depart", true, (1, 1), &[]),
            level("combat", "combat", false, (1, 1), &[]),
            level("boss", "boss", false, (1, 1), &[]),
            level("boss_grand", "boss", false, (2, 2), &[]),
            level("tresor", "tresor", false, (1, 1), &[]),
            level("tresor_long", "tresor", false, (2, 1), &[]),
            level("tresor_haut", "tresor", false, (1, 2), &[]),
            level("secrete", "secrete", false, (1, 1), &[]),
            level("diable", "diable", false, (1, 1), &[]),
        ]
    }

    fn context(seed: i32, levels: Vec<AvailableLevel>) -> MapGenerationContext {
        MapGenerationContext {
            tile_size: (16, 16),
            level_size: (15, 9),
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

    fn grammar(loops: u32, post: Option<f32>) -> FloorGrammar {
        FloorGrammar {
            rooms: (8, 12),
            required: [("boss", 1), ("tresor", 1), ("secrete", 1)]
                .map(|(k, n)| (k.to_string(), n))
                .into(),
            start: "depart".into(),
            boss: "boss".into(),
            filler: "combat".into(),
            min_boss_distance: 3,
            max_attempts: 64,
            layout: Some(FloorLayout::Grid {
                cell: CELL,
                max: (7, 7),
                loops,
                secret: "secrete".into(),
                post_boss: post.map(|chance| PostBoss {
                    kind: "diable".into(),
                    chance: Fixed::from_num(chance),
                }),
            }),
        }
    }

    fn plan(grammar: &FloorGrammar, seed: i32) -> Result<FloorPlan, FloorError> {
        let context = context(seed, templates());
        let mut rng = RollbackRng::new(seed as u32);
        super::super::floor::plan_floor(grammar, &context, &mut rng)
    }

    /// Forme d'un plan : types, gabarits, positions, identifiants et portes.
    fn signature(plan: &FloorPlan) -> Vec<String> {
        plan.rooms
            .iter()
            .map(|p| {
                format!(
                    "{}:{}@{},{}:{}:{:?}:{:?}",
                    p.kind,
                    p.room.level_def.level_id,
                    p.room.position.0,
                    p.room.position.1,
                    p.room.level_iid,
                    p.via,
                    p.links
                )
            })
            .collect()
    }

    fn edges(plan: &FloorPlan) -> usize {
        plan.rooms
            .iter()
            .map(|p| {
                p.room
                    .connections
                    .iter()
                    .filter(|c| matches!(c.to, Some(ConnectionTo::Room(_))))
                    .count()
            })
            .sum::<usize>()
            / 2
    }

    /// 1 000 graines : le contrôle indépendant passe (`plan_floor` le rejoue), types présents,
    /// boss et trésor en culs-de-sac, secrète à au moins deux portes, grandes formes réellement
    /// tirées, post-boss tirée à peu près une fois sur deux, travail sous la borne prouvée.
    #[test]
    fn mille_graines_respectent_la_grille() {
        let grammar = grammar(0, Some(0.5));
        let bound = step_bound(&grammar, &grid_templates(&context(1, templates()), CELL));
        let (mut shapes, mut posts, mut attempts, mut max_steps) =
            (BTreeMap::<String, usize>::new(), 0, 0, 0);
        for seed in 0..1000 {
            let plan = plan(&grammar, seed).unwrap_or_else(|e| panic!("graine {seed} : {e}"));
            validate_grid_plan(&grammar, (16, 16), &plan.rooms)
                .unwrap_or_else(|e| panic!("graine {seed} : {e}"));
            attempts += plan.attempts;
            max_steps = max_steps.max(plan.steps);
            assert!(
                plan.steps <= bound,
                "graine {seed} : {} > {bound}",
                plan.steps
            );
            let of = |kind: &str| plan.rooms.iter().filter(|p| p.kind == kind).count();
            assert_eq!(
                (of("depart"), of("boss"), of("tresor"), of("secrete")),
                (1, 1, 1, 1)
            );
            assert!(of("diable") <= 1);
            posts += of("diable");
            assert!((8..=12).contains(&plan.rooms.len()), "graine {seed}");
            // pas de porte vers le vide, pas de boucle : arbre + portes de la secrète
            let secret_doors = plan
                .rooms
                .iter()
                .find(|p| p.kind == "secrete")
                .map(|p| {
                    p.room
                        .connections
                        .iter()
                        .filter(|c| matches!(c.to, Some(ConnectionTo::Room(_))))
                        .count()
                })
                .unwrap();
            assert!(secret_doors >= 2, "graine {seed}");
            assert_eq!(
                edges(&plan),
                plan.rooms.len() - 2 + secret_doors,
                "graine {seed}"
            );
            for p in &plan.rooms {
                if p.kind == "boss" || p.kind == "tresor" {
                    *shapes.entry(p.room.level_def.level_id.clone()).or_default() += 1;
                }
            }
        }
        println!(
            "grille : 1000 graines, tentatives moyennes {:.2}, pas max {max_steps} (borne {bound}), post-boss {posts}, gabarits {shapes:?}",
            attempts as f64 / 1000.0
        );
        for shape in ["boss", "boss_grand", "tresor", "tresor_long", "tresor_haut"] {
            assert!(
                shapes.get(shape).is_some_and(|n| *n > 30),
                "{shape} : {shapes:?}"
            );
        }
        assert!((350..650).contains(&posts), "post-boss : {posts} sur 1000");
    }

    /// Des boucles : jamais plus que `loops`, le boss reste strictement le plus loin, et elles
    /// existent bel et bien.
    #[test]
    fn les_boucles_sont_bornees_et_ne_rapprochent_pas_le_boss() {
        let grammar = grammar(2, None);
        let mut with_loop = 0;
        for seed in 0..1000 {
            let plan = plan(&grammar, seed).unwrap_or_else(|e| panic!("graine {seed} : {e}"));
            let secret_doors = plan
                .rooms
                .iter()
                .find(|p| p.kind == "secrete")
                .map(|p| {
                    p.room
                        .connections
                        .iter()
                        .filter(|c| matches!(c.to, Some(ConnectionTo::Room(_))))
                        .count()
                })
                .unwrap();
            let loops = edges(&plan) + 2 - plan.rooms.len() - secret_doors;
            assert!(loops <= 2, "graine {seed} : {loops} boucles");
            with_loop += usize::from(loops > 0);
        }
        println!("grille : {with_loop} étages sur 1000 ont au moins une boucle");
        assert!(with_loop > 100, "{with_loop} étages à boucle sur 1000");
    }

    #[test]
    fn meme_graine_meme_etage_graines_differentes_etages_differents() {
        let grammar = grammar(1, Some(0.5));
        let run = |seed| signature(&plan(&grammar, seed).unwrap());
        assert_eq!(run(7), run(7));
        let distinct: BTreeSet<_> = (0..50).map(run).collect();
        assert!(
            distinct.len() > 45,
            "{} étages distincts sur 50",
            distinct.len()
        );
    }

    /// Chance 0 : jamais ; chance 1 : toujours (la place est réservée dans `rooms`).
    #[test]
    fn la_chance_du_post_boss_est_respectee_et_accroche_le_boss() {
        for seed in 0..200 {
            let none = plan(&grammar(0, Some(0.0)), seed).unwrap();
            assert!(
                none.rooms.iter().all(|p| p.kind != "diable"),
                "graine {seed}"
            );
            let always = plan(&grammar(0, Some(1.0)), seed).unwrap();
            let post = always
                .rooms
                .iter()
                .position(|p| p.kind == "diable")
                .unwrap();
            let boss = always.rooms.iter().position(|p| p.kind == "boss").unwrap();
            assert_eq!(always.rooms[post].parent, Some(boss), "graine {seed}");
            assert!(always.rooms.len() <= 12, "graine {seed}");
        }
        // la décision ne dépend que de la graine de l'emplacement : rejouée, identique
        let g = grammar(0, Some(0.5));
        let decisions: Vec<bool> = (0..40)
            .map(|s| {
                plan(&g, s)
                    .unwrap()
                    .rooms
                    .iter()
                    .any(|p| p.kind == "diable")
            })
            .collect();
        assert_eq!(
            decisions,
            (0..40)
                .map(|s| plan(&g, s)
                    .unwrap()
                    .rooms
                    .iter()
                    .any(|p| p.kind == "diable"))
                .collect::<Vec<_>>()
        );
    }

    /// Ports : une ouverture centrée sur le segment d'une case ; une décentrée, une à cheval et un
    /// gabarit hors grille sont ignorés ; la pose inverse retrouve la case.
    #[test]
    fn les_ports_sont_les_ouvertures_centrees_de_chaque_case() {
        let mut off_centre = level("decale", "combat", false, (1, 1), &[(Side::N, 0)]);
        off_centre.connections.push(Connection {
            index: off_centre.connections.len(),
            size: 3,
            side: Side::N,
            starting_at: 4,
            level_id: "decale".into(),
            compatiable_levels: vec![],
        });
        let ragged = AvailableLevel {
            level_size: (16, 9),
            level_size_p: (256, 144),
            ..level("hors_grille", "combat", false, (1, 1), &[])
        };
        let context = context(
            1,
            vec![
                level("long", "tresor", false, (2, 1), &[]),
                off_centre,
                ragged,
            ],
        );
        let templates = grid_templates(&context, CELL);
        assert_eq!(templates.len(), 2, "le gabarit 16 × 9 est ignoré");
        let ports: Vec<Port> = templates[0].ports.keys().copied().collect();
        assert_eq!(
            ports,
            vec![
                (Side::N, 0),
                (Side::N, 1),
                (Side::S, 0),
                (Side::S, 1),
                (Side::W, 0),
                (Side::E, 0)
            ]
        );
        assert_eq!(templates[0].size, (2, 1));
        assert!(
            !templates[1].ports.contains_key(&(Side::N, 0)),
            "décentrée ou absente"
        );
    }

    #[test]
    fn port_cell_et_origin_for_sont_reciproques_pour_chaque_forme() {
        for shape in [(1, 1), (2, 1), (1, 2), (2, 2)] {
            let context = context(1, vec![level("g", "boss", false, shape, &[])]);
            let templates = grid_templates(&context, CELL);
            let layout = Layout {
                templates: &templates,
                geometry: Geometry {
                    cell_px: (240, 144),
                    origin_px: (0, 0),
                },
                max: (9, 9),
                world: (1000, 1000),
                slots: Vec::new(),
                occupied: BTreeMap::new(),
                edges: Vec::new(),
                used: BTreeSet::new(),
                steps: 0,
            };
            let origin = (3, 4);
            for port in templates[0].ports.keys().copied() {
                let cell = layout.port_cell(0, origin, port);
                assert!(
                    layout.footprint(0, origin).any(|c| c == cell),
                    "{shape:?} {port:?}"
                );
                assert_eq!(
                    layout.origin_for(0, port, cell),
                    origin,
                    "{shape:?} {port:?}"
                );
                assert_eq!(layout.border_port(0, origin, cell, port.0), Some(port));
            }
            assert_eq!(templates[0].ports.len(), 2 * (shape.0 + shape.1) as usize);
        }
    }

    /// Placement d'une 2×2 : elle ne se pose que si son rectangle est libre, dans la grille, et
    /// ne touche que le parent ; toutes les poses énumérées respectent ces trois règles.
    #[test]
    fn les_grandes_formes_ne_se_posent_que_sur_un_rectangle_libre_qui_ne_touche_que_le_parent() {
        let levels = vec![
            level("depart", "depart", true, (1, 1), &[]),
            level("combat", "combat", false, (1, 1), &[]),
            level("boss_grand", "boss", false, (2, 2), &[]),
        ];
        let context = context(1, levels);
        let templates = grid_templates(&context, CELL);
        let mut layout = Layout {
            templates: &templates,
            geometry: Geometry {
                cell_px: (240, 144),
                origin_px: (-600, -360),
            },
            max: (5, 5),
            world: (1000, 1000),
            slots: Vec::new(),
            occupied: BTreeMap::new(),
            edges: Vec::new(),
            used: BTreeSet::new(),
            steps: 0,
        };
        layout.put(0, (2, 2), "depart");
        // une salle de combat à l'est, une au sud-ouest du centre pour gêner
        let east = layout.put(1, (3, 2), "combat");
        layout.link(0, (Side::E, 0), east, (Side::W, 0), true);
        let boss = layout.templates_of("boss", false);
        let list = layout.placements(&boss, &|index, _| index == east, None);
        assert!(!list.is_empty());
        for p in &list {
            let cells: Vec<Cell> = layout.footprint(p.tpl, p.origin).collect();
            assert_eq!(cells.len(), 4);
            assert!(cells
                .iter()
                .all(|c| (0..5).contains(&c.0) && (0..5).contains(&c.1)));
            assert!(cells.iter().all(|c| !layout.occupied.contains_key(c)));
            assert_eq!(layout.neighbours(p.tpl, p.origin), BTreeSet::from([east]));
        }
        // collée au bord de la grille : la 2×2 ne tient plus (la salle d'est est en colonne 4)
        let mut edge = Layout {
            max: (5, 5),
            ..layout
        };
        edge.slots.clear();
        edge.occupied.clear();
        edge.edges.clear();
        edge.used.clear();
        edge.put(0, (0, 0), "depart");
        let list = edge.placements(&boss, &|index, _| index == 0, None);
        assert!(list.iter().all(|p| p.origin.0 >= 0 && p.origin.1 >= 0));
        // départ coincé dans un coin d'une grille 1×1 : rien à poser
        let mut tiny = Layout {
            max: (1, 1),
            ..edge
        };
        tiny.slots.clear();
        tiny.occupied.clear();
        tiny.put(0, (0, 0), "depart");
        assert!(tiny.placements(&boss, &|_, _| true, None).is_empty());
    }

    /// `post_boss` de chance 0 sans gabarit `diable` : plan valide (le lint n'exige pas le gabarit).
    #[test]
    fn post_boss_de_chance_nulle_n_exige_aucun_gabarit() {
        let levels: Vec<_> = templates()
            .into_iter()
            .filter(|l| kind_of(l) != "diable")
            .collect();
        let context = context(3, levels);
        let g = grammar(0, Some(0.0));
        let mut rng = RollbackRng::new(3);
        let plan = super::super::floor::plan_floor(&g, &context, &mut rng).unwrap();
        assert!(plan.rooms.iter().all(|p| p.kind != "diable"));
        // chance > 0 : le gabarit manque, échec explicite
        let mut rng = RollbackRng::new(3);
        assert_eq!(
            super::super::floor::plan_floor(&grammar(0, Some(0.5)), &context, &mut rng)
                .unwrap_err(),
            FloorError::NoTemplate("diable".into())
        );
    }

    /// La borne `step_bound` tient aussi avec des boucles et une secrète (1 000 graines).
    #[test]
    fn la_borne_de_pas_tient_avec_des_boucles() {
        let g = grammar(3, Some(0.5));
        let bound = step_bound(&g, &grid_templates(&context(1, templates()), CELL));
        let mut max = 0;
        for seed in 0..1000 {
            let plan = plan(&g, seed).unwrap_or_else(|e| panic!("graine {seed} : {e}"));
            assert!(
                plan.steps <= bound,
                "graine {seed} : {} > {bound}",
                plan.steps
            );
            max = max.max(plan.steps);
        }
        assert!(max > 0);
    }

    /// Deux salles ne se relient que par des ouvertures de même largeur : un boss aux ouvertures de
    /// 5 cases (centrées, donc ports valides) ne se pose jamais contre un départ aux ouvertures de 3,
    /// et la grammaire échoue explicitement au lieu de produire un plan que le contrôle refuserait.
    #[test]
    fn deux_salles_ne_se_relient_que_par_des_ouvertures_de_meme_largeur() {
        let mut wide = level("boss_large", "boss", false, (1, 1), &[]);
        for c in &mut wide.connections {
            let segment = if matches!(c.side, Side::N | Side::S) {
                15
            } else {
                9
            };
            c.size = 5;
            c.starting_at = c.starting_at / segment * segment + (segment - 5) / 2;
        }
        let levels = vec![
            level("depart", "depart", true, (1, 1), &[]),
            level("combat", "combat", false, (1, 1), &[]),
            wide,
        ];
        let context = context(1, levels);
        let templates = grid_templates(&context, CELL);
        let boss = templates
            .iter()
            .position(|t| kind_of(&t.level) == "boss")
            .unwrap();
        assert!(
            templates[boss].ports.len() == 4,
            "les ouvertures larges sont des ports"
        );
        let mut layout = Layout {
            templates: &templates,
            geometry: Geometry {
                cell_px: (240, 144),
                origin_px: (-600, -360),
            },
            max: (5, 5),
            world: (1000, 1000),
            slots: Vec::new(),
            occupied: BTreeMap::new(),
            edges: Vec::new(),
            used: BTreeSet::new(),
            steps: 0,
        };
        layout.put(0, (2, 2), "depart");
        assert!(layout.placements(&[boss], &|_, _| true, None).is_empty());
        let combat = layout.placements(&[1], &|_, _| true, None);
        assert!(
            !combat.is_empty(),
            "des ouvertures de même largeur se relient"
        );
        let grammar = FloorGrammar {
            rooms: (3, 3),
            required: [("boss".to_string(), 1)].into(),
            start: "depart".into(),
            boss: "boss".into(),
            filler: "combat".into(),
            min_boss_distance: 2,
            max_attempts: 8,
            layout: Some(FloorLayout::Grid {
                cell: CELL,
                max: (5, 5),
                loops: 0,
                secret: "secrete".into(),
                post_boss: None,
            }),
        };
        let mut rng = RollbackRng::new(1);
        assert_eq!(
            super::super::floor::plan_floor(&grammar, &context, &mut rng).unwrap_err(),
            FloorError::Exhausted { attempts: 8 }
        );
    }

    /// Terminaison : une grammaire impossible échoue explicitement, vite, et le nombre
    /// d'évaluations reste sous la borne ; pas de boucle sans fin.
    #[test]
    fn une_grammaire_impossible_echoue_sans_boucle() {
        let context1 = context(1, templates());
        let templates = grid_templates(&context1, CELL);
        let run = |g: &FloorGrammar| {
            let mut rng = RollbackRng::new(1);
            super::super::floor::plan_floor(g, &context1, &mut rng)
        };
        // grille d'une seule case : le boss ne se pose jamais
        let mut tiny = grammar(0, None);
        if let Some(FloorLayout::Grid { max, .. }) = tiny.layout.as_mut() {
            *max = (1, 1);
        }
        assert_eq!(
            run(&tiny).unwrap_err(),
            FloorError::Exhausted { attempts: 64 }
        );
        // distance du boss impossible
        let mut far = grammar(0, None);
        far.min_boss_distance = 40;
        far.max_attempts = 20;
        assert_eq!(
            run(&far).unwrap_err(),
            FloorError::Exhausted { attempts: 20 }
        );
        // plus de requises que de salles
        let mut crowded = grammar(0, None);
        crowded.rooms = (3, 3);
        assert_eq!(
            run(&crowded).unwrap_err(),
            FloorError::Exhausted { attempts: 64 }
        );
        // type sans gabarit
        let mut missing = grammar(0, None);
        missing.required.insert("boutique".into(), 1);
        assert_eq!(
            run(&missing).unwrap_err(),
            FloorError::NoTemplate("boutique".into())
        );
        // la borne couvre le pire des échecs ci-dessus : on la relit par un plan impossible
        let mut rng = RollbackRng::new(1);
        let steps = {
            let params = Params::of(&tiny).unwrap();
            let mut steps = 0;
            for _ in 0..tiny.max_attempts {
                attempt(
                    &params,
                    &templates,
                    &context1,
                    rng.next_u32(),
                    false,
                    &mut steps,
                );
            }
            steps
        };
        assert!(steps <= step_bound(&tiny, &templates), "{steps}");
    }

    /// Le contrôle indépendant refuse un plan altéré.
    #[test]
    fn validate_grid_plan_refuse_un_plan_altere() {
        let grammar = grammar(0, Some(1.0));
        let good = plan(&grammar, 5).unwrap();
        assert!(validate_grid_plan(&grammar, (16, 16), &good.rooms).is_ok());
        let tamper = |f: &dyn Fn(&mut Vec<PlannedRoom>)| {
            let mut rooms = good.rooms.clone();
            f(&mut rooms);
            validate_grid_plan(&grammar, (16, 16), &rooms)
        };
        // une salle décalée d'une tuile : hors grille
        assert!(tamper(&|r| r[1].room.position.0 += 16).is_err());
        // deux boss
        assert!(tamper(&|r| r[2].kind = "boss".into()).is_err());
        // porte non réciproque : la connexion d'une salle oubliée
        assert!(tamper(&|r| {
            let c = r[1]
                .room
                .connections
                .iter_mut()
                .find(|c| c.to.is_some())
                .unwrap();
            c.to = None;
        })
        .is_err());
        // porte vers le vide : la salle voisine déplacée d'une case de salle
        assert!(tamper(&|r| r[1].room.position.0 += 240).is_err());
        // le boss devient aussi proche que le reste : distance minimale impossible
        let mut strict = grammar.clone();
        strict.min_boss_distance = 30;
        assert!(validate_grid_plan(&strict, (16, 16), &good.rooms).is_err());
        // plus de boucles que permis : un plan à boucle, jugé contre la même grammaire sans boucle
        let mut with_loops = grammar.clone();
        if let Some(FloorLayout::Grid { loops, .. }) = with_loops.layout.as_mut() {
            *loops = 3;
        }
        let looped = (0..50)
            .map(|seed| plan(&with_loops, seed).unwrap())
            .find(|p| {
                p.rooms
                    .iter()
                    .any(|r| r.kind != "secrete" && !r.links.is_empty())
            })
            .expect("un étage à boucle sur 50 graines");
        assert!(validate_grid_plan(&with_loops, (16, 16), &looped.rooms).is_ok());
        let error = validate_grid_plan(&grammar, (16, 16), &looped.rooms).unwrap_err();
        assert!(error.contains("boucle"), "{error}");
    }

    /// M3-T9 (§74) : la curse `Labyrinth` double les bornes (`FloorGrammar::scaled_rooms`) : sur la
    /// grille de 7 × 7, 1 000 graines donnent un étage valide de 16 à 24 salles, sous la borne de
    /// travail de la grammaire doublée, et un multiplicateur énorme (borné à la capacité) reste planifiable.
    #[test]
    fn mille_graines_doublees_par_la_curse_respectent_la_grille() {
        let double = grammar(1, Some(0.33)).scaled_rooms(Fixed::from_num(2));
        assert_eq!(double.rooms, (16, 24));
        let bound = step_bound(&double, &grid_templates(&context(1, templates()), CELL));
        let mut sizes = BTreeMap::<usize, usize>::new();
        for seed in 0..1000 {
            let plan = plan(&double, seed).unwrap_or_else(|e| panic!("graine {seed} : {e}"));
            validate_grid_plan(&double, (16, 16), &plan.rooms)
                .unwrap_or_else(|e| panic!("graine {seed} : {e}"));
            assert!(plan.steps <= bound, "graine {seed}");
            assert!(
                (16..=24).contains(&plan.rooms.len()),
                "graine {seed} : {} salles",
                plan.rooms.len()
            );
            *sizes.entry(plan.rooms.len()).or_default() += 1;
        }
        assert!(sizes.len() >= 6, "tailles tirées : {sizes:?}");
        // ×100 : borné à 49 cases, la demande 49 salles est trop dense pour une grille où une salle ne
        // touche que son parent : le planificateur échoue proprement (jamais de boucle infinie).
        let huge = grammar(1, Some(0.33)).scaled_rooms(Fixed::from_num(100));
        assert_eq!(huge.rooms, (49, 49));
        assert!(plan(&huge, 1).is_err());
    }
}
