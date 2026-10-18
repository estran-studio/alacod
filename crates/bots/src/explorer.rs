//! Profil `explorateur` (M2-T16, `docs/conventions.md` §47) : joue seul un étage de salles
//! typées (mode `Floors`, grammaire d'étage §40).
//!
//! - **Combat** : celui de `prudent` (chasse par le chemin, esquive, gestion d'arme, réanimation,
//!   ramassage au passage), mais seulement contre les ennemis **éveillés** : un ennemi
//!   `RoomDormant` (salle verrouillante pas encore activée, §35) n'est pas une cible, c'est une
//!   salle à visiter.
//! - **Blank** (§41) : appui quand une balle ennemie qui approche est à moins de
//!   [`BLANK_DISTANCE`] et que le joueur a un blank ; le front montant est lu dans `BlankState`
//!   (état rollback), jamais dans une mémoire du bot.
//! - **Exploration**, sans ennemi éveillé : le but le plus proche **par le chemin** (champ à une
//!   source, [`BotNavigation::costs_from`]), ordre déterministe (coût, objets avant salles, puis
//!   `GgrsNetId` de l'objet ou identifiant de la salle) : un objet au sol (consommable au contact,
//!   passif ou actif par Interaction ; pas d'actif si le joueur en tient déjà un, sinon il
//!   échangerait sans fin), une salle verrouillante encore à visiter (`Locked`, ou `Dormant` avec
//!   un ennemi dedans), la salle du boss en dernier (ou seule, [`ExplorerOptions::boss_direct`]).
//!   Portail ouvert : les salles ne comptent plus, `prudent` va au portail.
//!
//! **Mémoire du bot** (piège « donnée dérivée non rollbackée » de `CLAUDE.md`) : aucune. « Salle
//! visitée » se lit dans l'état : une salle verrouillante `Cleared`, ou `Dormant` sans ennemi
//! dedans (elle ne se verrouillera jamais), n'est plus un but ; une salle non verrouillante
//! (départ, récompense, boutique) n'est un but que par ses objets au sol. Tout est recalculé à
//! chaque frame depuis les composants et ressources rollback ; les champs de navigation sont des
//! caches dont les clés contiennent toute la géométrie (vider le cache donne le même input).

use std::collections::{BTreeMap, BTreeSet};

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_fixed::fixed_math::{Fixed, FixedTransform3D, FixedVec2};
use bevy_ggrs::Rollback;
use game::character::enemy::ai::navigation::{FlowField, GridPos};
use game::character::enemy::Enemy;
use game::character::player::input::{BoxInput, INPUT_BLANK, INPUT_INTERACTION};
use game::character::player::Player;
use game::items::BlankState;
use items::{Inventory, ItemKind, ItemPickup, ItemTable};
use utils::{net_id::GgrsNetId, order_iter};
use world::{RoomDormant, RoomKindTable, RoomState, RoomStates};

use crate::navigation::{cell, point, BotNavigation, Rect};
use crate::view::BotView;

/// Une balle ennemie qui approche à moins de cette distance (unités monde) déclenche un blank,
/// s'il en reste un.
pub const BLANK_DISTANCE: Fixed = Fixed::from_bits(48 << 16);

/// Marge intérieure de la zone visée dans une salle : au-delà des 24 px de `PLAYER_INSET`
/// (`map_ldtk::game::rooms`) qui font qu'un joueur debout est « dans » la salle et la verrouille.
pub const ROOM_INSET: Fixed = Fixed::from_bits(40 << 16);

/// Portée visée pour ramasser un objet au contact (sous le `pickup_range` de 24 par défaut).
const ITEM_REACH: Fixed = Fixed::from_bits(16 << 16);

/// Coût de chemin (10 par case orthogonale de la grille du bot, 8 px) au-delà duquel un objet, un
/// présentoir, un coffre ou une salle sans verrou (récompense, boutique) ne vaut plus le détour :
/// 600, soit 60 cases (480 px, ~190 frames à 2,5 px par frame ; les pénalités de proximité des
/// murs en font moins en ligne droite). Dans les salles agrandies de
/// m2-salles-murs-gungeon un étage mesure plus de 2000 px et le soft-lock `Floors` tombe à 1200
/// frames sans dégât ni mort : à courir d'un objet lointain à l'autre (butin d'un combat passé,
/// présentoirs aux deux bouts de l'étage), le bot laissait passer ce délai sans combattre
/// (graines 8, 10, 12, 16, 29 de `etages_deux`). Les buts lointains deviennent proches quand le
/// bot passe à côté, en allant d'une salle de combat à la suivante.
pub const OPTIONAL_DETOUR: u32 = 600;

/// Même limite quand plus aucune salle de combat ne reste (le boss et d'éventuels détours) : le
/// trajet jusqu'au boss, souvent à l'autre bout de l'étage, est le dernier avant le combat, et
/// chaque détour s'ajoute aux 1200 frames du soft-lock (graine 12). 200, soit 20 cases (160 px).
pub const LAST_STRETCH_DETOUR: u32 = 200;

/// Marge intérieure au-delà de laquelle un joueur « est dans » une salle non verrouillante
/// (`PLAYER_INSET` de `map_ldtk::game::rooms`) : entrer suffit pour la visiter.
const VISIT_INSET: Fixed = Fixed::from_bits(24 << 16);

/// Salles non verrouillantes (récompense, boutique, départ) déjà visitées : `(index d'étage,
/// identifiant de salle)`. **Seule mémoire de l'explorateur**, hors simulation et hors rollback :
/// une salle sans état (`RoomStates` ne suit que les salles verrouillantes) ne laisse aucune trace
/// à lire. Elle ne sert qu'à produire des inputs, que GGRS enregistre ensuite : un rollback ne
/// rappelle jamais le bot pour une frame passée.
#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
pub struct ExplorerVisits(pub BTreeSet<(usize, String)>);

/// Phase de patience d'un `explorateur` (m2-salles-murs-gungeon, D82) : le pilotage touche par touche,
/// décalé de 5 frames et avec de l'élan, peut tourner en cycle limite devant une ouverture de 32 px
/// (corps de 20 px) quel que soit le gain de correction (essais : avance de 5 frames, de 8 frames : chacun
/// débloque certaines graines et en bloque d'autres). Un bot resté à moins de [`STALL_RADIUS`] de son
/// ancre pendant [`STALL_FRAMES`] frames sans ennemi ni butin change de manière : **Settle** (aucune
/// touche : l'élan s'éteint) puis **Go** (cap droit sur l'axe du trajet, l'écart transverse de moins de
/// [`GO_TRANSVERSE`] n'est pas corrigé) ; la séquence recommence tant que le bot ne bouge pas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Patience {
    #[default]
    Normal,
    Settle,
    Go,
}

/// Rayon de l'ancre (le cycle limite observé fait ±8 px en x et ±12 px en y, soit 24 px d'amplitude).
const STALL_RADIUS: Fixed = Fixed::from_bits(40 << 16);
/// Frames sans quitter l'ancre avant la première phase de repli.
const STALL_FRAMES: u32 = 100;
/// Durée de l'arrêt (frames), puis du cap droit ; la séquence dure [`PATIENCE_CYCLE`].
const SETTLE_FRAMES: u32 = 15;
const PATIENCE_CYCLE: u32 = 60;
/// Écart transverse toléré pendant le cap droit (la marge d'une ouverture de 32 px pour un corps de 20 px).
const GO_TRANSVERSE: Fixed = Fixed::from_bits(6 << 16);

/// Ancre (position, frame d'arrivée) par handle : mémoire hors rollback, comme [`ExplorerVisits`].
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct ExplorerPatience(pub BTreeMap<usize, (FixedVec2, u32)>);

impl ExplorerPatience {
    /// Phase du joueur `handle` à la frame `frame` ; `has_goal` : un cap d'exploration existe (sinon l'ancre
    /// est oubliée : attendre sans but n'est pas bloqué).
    pub fn phase(
        &mut self,
        handle: usize,
        frame: u32,
        position: FixedVec2,
        has_goal: bool,
    ) -> Patience {
        let entry = self.0.entry(handle).or_insert((position, frame));
        if !has_goal || entry.0.distance(&position) > STALL_RADIUS {
            *entry = (position, frame);
            return Patience::Normal;
        }
        let stalled = frame.saturating_sub(entry.1);
        if stalled < STALL_FRAMES {
            return Patience::Normal;
        }
        if (stalled - STALL_FRAMES) % PATIENCE_CYCLE < SETTLE_FRAMES {
            Patience::Settle
        } else {
            Patience::Go
        }
    }
}

/// Ce que l'exploration ajoute à la [`BotView`] d'un `explorateur`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ExploreView {
    /// Pas suivant vers le but d'exploration (nul : arrivé) ; `None` : aucun but accessible, ou
    /// un ennemi éveillé (le combat de `prudent` décide).
    pub direction: Option<FixedVec2>,
    /// Tenir Interaction (objet par interaction, à portée et sélectionné par le jeu).
    pub interact: bool,
    /// Presser Blank cette frame (front montant).
    pub blank: bool,
    /// Phase de patience (voir [`Patience`]).
    pub patience: Patience,
}

/// Réglages des `explorateur` (ressource ordinaire, hors rollback et hors simulation, posée avant
/// la partie par `alacod-sim --boss-direct`).
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExplorerOptions {
    /// Aller droit au boss : les autres salles et les objets ne sont plus des buts.
    pub boss_direct: bool,
    /// D62 : ouvre les coffres et achète les présentoirs qu'il peut payer (`alacod-sim
    /// --shopping`). Faux par défaut : les traces des scénarios d'explorateur ne bougent pas.
    pub shopping: bool,
}

/// Une salle verrouillante encore à visiter.
#[derive(Debug, Clone, PartialEq)]
pub struct RoomGoal {
    /// `LevelId` de la salle (identifiant stable, identique sur tous les clients).
    pub id: String,
    /// Zone visée : les bornes de la salle réduites de [`ROOM_INSET`].
    pub zone: Rect,
    /// Un boss (`BossState`) y vit.
    pub boss: bool,
    /// La salle se verrouille (combat, arène, boss) : seule une salle verrouillante fait tuer,
    /// donc avancer l'étage. Une autre (départ, récompense, boutique) ne vaut que le détour.
    pub locking: bool,
}

/// Un objet au sol.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemGoal {
    pub net_id: usize,
    pub position: FixedVec2,
    /// Ramassé par Interaction (passif, actif), sinon au contact (consommable).
    pub surface: bool,
    /// Objet actif (`ItemKind::Active`).
    pub active: bool,
}

/// L'étage vu par les `explorateur`, une fois par frame.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExploreFrame {
    /// Salles à visiter, par identifiant.
    pub rooms: Vec<RoomGoal>,
    /// Objets au sol, par `GgrsNetId`.
    pub items: Vec<ItemGoal>,
    /// `GgrsNetId` des ennemis endormis (`RoomDormant`).
    pub dormant: BTreeSet<usize>,
    /// Présentoirs payables qui vendent une arme : `(GgrsNetId du présentoir, identifiant de l'arme)`.
    /// Une arme achetée tombe au pied du joueur que l'Interaction tenue ramasse aussitôt (un seul
    /// emplacement : l'arme en main tombe) ; voir [`ExploreFrame::without_unusable_weapons`].
    pub weapon_shelves: Vec<(usize, String)>,
}

impl ExploreFrame {
    /// La même frame sans les présentoirs dont l'arme est **inutilisable** pour ce joueur
    /// (`usable(arme)` faux : aucune réserve du type de munition de l'arme, une arme ramassée
    /// n'en ajoute pas). Le bot achetait un `fusil_precision` (munitions `calibre`, réserve nulle)
    /// qui remplaçait son arme de départ : chargeur vide sans recharge possible, il restait sans
    /// tir jusqu'au soft-lock (graines 3, 4, 18 de `etages_deux`). `None` si rien n'est écarté.
    pub fn without_unusable_weapons(&self, usable: impl Fn(&str) -> bool) -> Option<ExploreFrame> {
        let unusable: BTreeSet<usize> = self
            .weapon_shelves
            .iter()
            .filter(|(_, weapon)| !usable(weapon))
            .map(|(id, _)| *id)
            .collect();
        if unusable.is_empty() {
            return None;
        }
        let mut frame = self.clone();
        frame.items.retain(|item| !unusable.contains(&item.net_id));
        Some(frame)
    }
}

/// Ce que le joueur porte (lu dans son `Inventory` et son `BlankState`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Holder {
    pub has_active: bool,
    pub blanks: u32,
    /// Blank tenu à la frame précédente (`BlankState::held`).
    pub blank_held: bool,
}

/// Lecture (seule) de l'état dont l'exploration a besoin.
#[derive(SystemParam)]
pub struct ExplorerWorld<'w, 's> {
    rooms: map_ldtk::game::rooms::CurrentRooms<'w, 's>,
    states: Option<Res<'w, RoomStates>>,
    table: Option<Res<'w, RoomKindTable>>,
    enemies: Query<
        'w,
        's,
        (
            &'static GgrsNetId,
            &'static FixedTransform3D,
            Has<RoomDormant>,
            Has<behaviors::BossState>,
        ),
        (With<Enemy>, With<Rollback>),
    >,
    items: Query<
        'w,
        's,
        (
            &'static GgrsNetId,
            &'static FixedTransform3D,
            &'static ItemPickup,
            Has<game::interaction::Interactable>,
        ),
        With<Rollback>,
    >,
    item_table: Option<Res<'w, ItemTable>>,
    holders: Query<
        'w,
        's,
        (
            &'static Player,
            Option<&'static Inventory>,
            Option<&'static BlankState>,
        ),
    >,
    shelves: Query<
        'w,
        's,
        (
            &'static GgrsNetId,
            &'static FixedTransform3D,
            &'static items::Shelf,
            Has<game::interaction::Interactable>,
        ),
        With<Rollback>,
    >,
    chests: Query<
        'w,
        's,
        (
            &'static GgrsNetId,
            &'static FixedTransform3D,
            &'static items::Chest,
            Has<game::interaction::Interactable>,
        ),
        With<Rollback>,
    >,
    wallets: Query<'w, 's, (&'static Player, &'static run::Currency)>,
    options: Option<Res<'w, ExplorerOptions>>,
    visits: ResMut<'w, ExplorerVisits>,
    pub patience: ResMut<'w, ExplorerPatience>,
    floor: Option<Res<'w, run::FloorState>>,
}

impl ExplorerWorld<'_, '_> {
    pub fn options(&self) -> ExplorerOptions {
        self.options.as_deref().copied().unwrap_or_default()
    }

    /// Salles à visiter, objets au sol et ennemis endormis de la frame. `positions` : les joueurs
    /// (une salle non verrouillante où l'un d'eux se tient est visitée).
    pub fn frame(&mut self, positions: &[FixedVec2]) -> ExploreFrame {
        let floor = self
            .floor
            .as_deref()
            .map_or(0, |state| state.index as usize);
        if let Some(table) = self.table.as_deref() {
            for (level, bounds, kind) in self.rooms.iter() {
                if !table.locks(&kind.0)
                    && positions.iter().any(|p| {
                        world::rooms::inside_box(bounds.position, bounds.size, *p, VISIT_INSET)
                    })
                {
                    self.visits.0.insert((floor, level.0.clone()));
                }
            }
        }
        let enemies: Vec<(usize, FixedVec2, bool, bool)> = order_iter!(self.enemies)
            .into_iter()
            .map(|(id, t, dormant, boss)| (id.0, t.translation.truncate(), dormant, boss))
            .collect();
        let dormant = enemies
            .iter()
            .filter(|(.., dormant, _)| *dormant)
            .map(|(id, ..)| *id)
            .collect();
        let mut rooms = Vec::new();
        if let (Some(states), Some(table)) = (self.states.as_deref(), self.table.as_deref()) {
            for (level, bounds, kind) in self.rooms.iter() {
                let locking = table.locks(&kind.0);
                if !locking && self.visits.0.contains(&(floor, level.0.clone())) {
                    continue;
                }
                let inside = |p: FixedVec2| {
                    world::rooms::inside_box(bounds.position, bounds.size, p, Fixed::ZERO)
                };
                let pending = room_pending(
                    locking,
                    table.arena(&kind.0).is_some(),
                    states.get(&level.0),
                    enemies.iter().any(|(_, p, ..)| inside(*p)),
                );
                if pending {
                    rooms.push(RoomGoal {
                        id: level.0.clone(),
                        zone: room_zone(bounds.position, bounds.size),
                        boss: enemies.iter().any(|(_, p, _, boss)| *boss && inside(*p)),
                        locking,
                    });
                }
            }
        }
        rooms.sort_by(|a, b| a.id.cmp(&b.id));
        let mut items: Vec<ItemGoal> = order_iter!(self.items)
            .into_iter()
            .map(|(id, t, pickup, surface)| ItemGoal {
                net_id: id.0,
                position: t.translation.truncate(),
                surface,
                active: self
                    .item_table
                    .as_deref()
                    .and_then(|table| table.0.get(&pickup.item_id))
                    .is_some_and(|def| matches!(def.kind, ItemKind::Active(_))),
            })
            .collect();
        let mut weapon_shelves = Vec::new();
        if self.options().shopping {
            self.shopping_goals(&mut items, &mut weapon_shelves);
        }
        ExploreFrame {
            rooms,
            items,
            dormant,
            weapon_shelves,
        }
    }

    /// D62 : présentoirs payables et coffres ouvrables (but d'exploration par Interaction, comme
    /// un objet passif au sol). L'arme qui en sort est ramassée par le combat de `prudent`
    /// (butin au sol), jamais par un but d'ici. Solde et clés se lisent dans l'état.
    fn shopping_goals(&self, items: &mut Vec<ItemGoal>, weapon_shelves: &mut Vec<(usize, String)>) {
        let wallet = self.wallets.iter().map(|(_, c)| c.0).max().unwrap_or(0);
        let keys = self
            .holders
            .iter()
            .filter_map(|(_, inventory, _)| inventory)
            .map(|inventory| inventory.consumable(game::loot::KEY_ITEM))
            .max()
            .unwrap_or(0);
        for (id, t, shelf, interactable) in order_iter!(self.shelves) {
            // M3-T8 (§73) : un présentoir à `cost` (marché du diable : vie maximale) n'est pas un achat
            // en douilles, son `price` vaut 0 ; le bot ne le vise pas (la décision du diable est T14).
            if interactable && !shelf.sold && shelf.cost.is_none() && shelf.price <= wallet {
                items.push(ItemGoal {
                    net_id: id.0,
                    position: t.translation.truncate(),
                    surface: true,
                    active: false,
                });
                if let Some(weapon) = items::loot_weapon(&shelf.item_id) {
                    weapon_shelves.push((id.0, weapon.to_string()));
                }
            }
        }
        for (id, t, chest, interactable) in order_iter!(self.chests) {
            if interactable && !chest.opened && (!chest.locked || keys > 0) {
                items.push(ItemGoal {
                    net_id: id.0,
                    position: t.translation.truncate(),
                    surface: true,
                    active: false,
                });
            }
        }
        items.sort_by_key(|goal| goal.net_id);
    }

    /// Ce que porte le joueur `handle`.
    pub fn holder(&self, handle: usize) -> Holder {
        let Some((_, inventory, blank)) = self.holders.iter().find(|(p, ..)| p.handle == handle)
        else {
            return Holder::default();
        };
        let blanks = inventory.map_or(0, |inventory| {
            self.item_table.as_deref().map_or(0, |table| {
                table
                    .0
                    .iter()
                    .filter(|(_, def)| def.blank.is_some())
                    .map(|(id, _)| inventory.consumable(id))
                    .sum()
            })
        });
        Holder {
            has_active: inventory.is_some_and(|i| i.active.is_some()),
            blanks,
            blank_held: blank.is_some_and(|b| b.held),
        }
    }
}

/// Une salle est-elle encore un but ? Une salle qui ne verrouille pas (récompense, boutique,
/// départ) l'est tant qu'elle n'est pas visitée (l'appelant a déjà écarté les visitées) ; une
/// arène dort vide (ses vagues naissent au verrou) et l'est jusqu'à son nettoyage ; une autre
/// salle verrouillante l'est quand elle est verrouillée, ou dormante avec un ennemi dedans.
pub fn room_pending(locking: bool, arena: bool, state: RoomState, enemy_inside: bool) -> bool {
    !locking
        || match state {
            RoomState::Locked => true,
            RoomState::Dormant => arena || enemy_inside,
            RoomState::Cleared => false,
        }
}

/// Zone visée dans une salle de bornes `(position, size)` : réduite de [`ROOM_INSET`] (ramenée au
/// centre sur un axe trop court).
pub fn room_zone(position: FixedVec2, size: FixedVec2) -> Rect {
    let axis = |start: Fixed, length: Fixed| {
        let center = start + length / Fixed::from_num(2);
        if length > ROOM_INSET * Fixed::from_num(2) {
            (start + ROOM_INSET, start + length - ROOM_INSET)
        } else {
            (center, center)
        }
    };
    let (x0, x1) = axis(position.x, size.x);
    let (y0, y1) = axis(position.y, size.y);
    Rect {
        min: FixedVec2::new(x0, y0),
        max: FixedVec2::new(x1, y1),
    }
}

/// Coût du chemin (champ `field`) jusqu'à la case la moins chère à moins de `reach` de `zone`.
pub fn zone_cost(field: &FlowField, zone: Rect, reach: Fixed) -> Option<u32> {
    let margin = (reach / Fixed::from_num(8)).ceil().to_num::<i32>() + 1;
    let (a, b) = (cell(zone.min), cell(zone.max));
    let mut best: Option<u32> = None;
    for x in a.x - margin..=b.x + margin {
        for y in a.y - margin..=b.y + margin {
            let p = GridPos::new(x, y);
            if zone.distance(point(p)) > reach {
                continue;
            }
            if let Some(&cost) = field.costs.get(&p) {
                best = Some(best.map_or(cost, |b| b.min(cost)));
            }
        }
    }
    best
}

/// Un but d'exploration.
#[derive(Debug, Clone, PartialEq)]
enum Goal {
    Item(ItemGoal),
    Room(Rect),
}

/// Clé d'ordre d'un but : coût du chemin, objets (0) avant salles (1), `GgrsNetId` de l'objet,
/// identifiant de la salle.
type GoalKey = (u32, u8, usize, String);

/// Buts d'exploration triés par [`GoalKey`] (seulement ceux qu'un chemin atteint).
pub fn ordered_goals(
    field: &FlowField,
    frame: &ExploreFrame,
    holder: Holder,
    portal_open: bool,
    boss_direct: bool,
) -> Vec<(GoalKey, Rect, Option<ItemGoal>)> {
    let mut goals: Vec<(GoalKey, Goal)> = Vec::new();
    let detour = if frame.rooms.iter().any(|room| room.locking && !room.boss) {
        OPTIONAL_DETOUR
    } else {
        LAST_STRETCH_DETOUR
    };
    if !portal_open {
        let bosses = frame.rooms.iter().any(|room| room.boss);
        let mut rooms: Vec<(GoalKey, Goal)> = Vec::new();
        let mut boss_rooms: Vec<(GoalKey, Goal)> = Vec::new();
        for room in &frame.rooms {
            let Some(cost) = zone_cost(field, room.zone, Fixed::ZERO) else {
                continue;
            };
            // Une salle sans verrou n'est qu'un détour (voir `OPTIONAL_DETOUR`) : lointaine, elle
            // ne retarde ni les combats ni le boss.
            if !room.locking && !room.boss && cost > detour {
                continue;
            }
            let entry = ((cost, 1, 0, room.id.clone()), Goal::Room(room.zone));
            if room.boss {
                boss_rooms.push(entry);
            } else {
                rooms.push(entry);
            }
        }
        if boss_direct && bosses {
            goals.extend(boss_rooms);
        } else if rooms.is_empty() {
            // Le boss en dernier : seulement quand aucune autre salle n'est atteignable.
            goals.extend(boss_rooms);
        } else {
            goals.extend(rooms);
        }
    }
    if !boss_direct {
        for item in &frame.items {
            if item.active && holder.has_active {
                continue;
            }
            let zone = Rect {
                min: item.position,
                max: item.position,
            };
            if let Some(cost) = zone_cost(field, zone, ITEM_REACH) {
                if cost > detour {
                    continue;
                }
                goals.push(((cost, 0, item.net_id, String::new()), Goal::Item(*item)));
            }
        }
    }
    goals.sort_by(|a, b| a.0.cmp(&b.0));
    goals
        .into_iter()
        .map(|(key, goal)| match goal {
            Goal::Item(item) => (
                key,
                Rect {
                    min: item.position,
                    max: item.position,
                },
                Some(item),
            ),
            Goal::Room(zone) => (key, zone, None),
        })
        .collect()
}

/// Pas suivant vers le premier but d'exploration accessible (voir le module) et Interaction à
/// tenir. La navigation doit avoir été mise à jour (`update_from`) pour cette frame.
#[allow(clippy::too_many_arguments)]
pub fn explore_step(
    nav: &mut BotNavigation,
    frame: &ExploreFrame,
    holder: Holder,
    position: FixedVec2,
    portal_open: bool,
    boss_direct: bool,
    surfaces: &[(usize, Rect, Fixed, bool)],
) -> Option<(FixedVec2, bool)> {
    let goals = {
        let field = nav.costs_from(position)?;
        ordered_goals(field, frame, holder, portal_open, boss_direct)
    };
    goals.into_iter().find_map(|(_, zone, item)| match item {
        Some(item) if item.surface => {
            let (_, rect, reach, _) = surfaces.iter().find(|(id, ..)| *id == item.net_id)?;
            crate::input::surface_step_with(
                nav,
                position,
                surfaces,
                item.net_id,
                *rect,
                *reach,
                true,
            )
        }
        Some(_) => nav
            .approach(position, zone, ITEM_REACH)
            .map(|(direction, _)| (crate::input::dead_zone(direction), false)),
        None => nav
            .approach(position, zone, Fixed::ZERO)
            .map(|(direction, _)| (crate::input::dead_zone(direction), false)),
    })
}

/// Blank : une balle ennemie de la vue approche à moins de [`BLANK_DISTANCE`], le joueur a un
/// blank et ne tenait pas Blank à la frame précédente (front montant du jeu).
pub fn should_blank(view: &BotView, holder: Holder) -> bool {
    holder.blanks > 0
        && !holder.blank_held
        && view.projectiles.iter().any(|p| {
            p.distance < BLANK_DISTANCE
                && (p.position - view.position).dot(&p.velocity) < Fixed::ZERO
        })
}

/// Anticipation de l'élan sur l'axe **transverse** au trajet (m2-salles-murs-gungeon, D82) : une touche
/// n'agit qu'après [`INPUT_LAG_SECONDS`] (délai d'input du synctest et du p2p, 5 frames) et le héros
/// garde son élan ; devant une ouverture de 32 px (corps de 20 px) le bot déborde donc de ±8 px, rebrousse,
/// déborde de l'autre côté : un cycle limite sans fin (graines 51 et 182 de `etages_deux`). On corrige
/// l'écart transverse de la distance que l'élan fait encore parcourir (`direction − vitesse × délai`) : le
/// bot freine avant de dépasser l'axe. L'axe du trajet (le plus grand des deux) n'est pas touché : la case
/// suivante n'est qu'à 8 px, un freinage là l'arrêterait.
pub(crate) fn lead(direction: FixedVec2, velocity: FixedVec2) -> FixedVec2 {
    let ahead = INPUT_LAG_SECONDS;
    if direction.x.abs() >= direction.y.abs() {
        FixedVec2::new(direction.x, direction.y - velocity.y * ahead)
    } else {
        FixedVec2::new(direction.x - velocity.x * ahead, direction.y)
    }
}

/// Cap droit de la phase [`Patience::Go`] : l'écart transverse sous [`GO_TRANSVERSE`] n'est pas corrigé.
pub(crate) fn straight(direction: FixedVec2) -> FixedVec2 {
    if direction.x.abs() >= direction.y.abs() {
        if direction.y.abs() < GO_TRANSVERSE {
            return FixedVec2::new(direction.x, Fixed::ZERO);
        }
    } else if direction.x.abs() < GO_TRANSVERSE {
        return FixedVec2::new(Fixed::ZERO, direction.y);
    }
    direction
}

/// Délai entre la décision du bot et l'effet de la touche (5 frames à 60 Hz, `ALACOD_INPUT_DELAY`).
const INPUT_LAG_SECONDS: Fixed = Fixed::from_bits(5461); // 5/60 × 65536

/// Décision de l'`explorateur` : `prudent` dès qu'il y a à se battre, esquiver, relever ou
/// ramasser au passage ; sinon le but d'exploration (pilotage en 8 directions), ou `prudent`
/// (portail, attente) sans but. Blank en plus.
pub fn decide_explorer(view: &BotView) -> BoxInput {
    let explore = view.explore.unwrap_or_default();
    let busy = view.nearest_enemy.is_some()
        || view.revive.is_some()
        || view.loot.is_some()
        || crate::dodge::dodge(view).is_some();
    let mut input = match explore.direction {
        Some(direction) if !busy => {
            let mut input = BoxInput::default();
            match explore.patience {
                Patience::Normal => {
                    crate::decide::steer(&mut input, lead(direction, view.velocity));
                }
                Patience::Settle => {}
                Patience::Go => crate::decide::steer(&mut input, straight(direction)),
            }
            if explore.interact {
                input.buttons |= INPUT_INTERACTION;
            }
            crate::decide::manage_weapon(&mut input, view);
            input
        }
        _ => crate::decide::decide_prudent(view),
    };
    if explore.blank {
        input.buttons |= INPUT_BLANK;
    }
    input
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::ProjectileView;
    use bevy_fixed::fixed_math::new as fx;

    fn v(x: f32, y: f32) -> FixedVec2 {
        FixedVec2::new(fx(x), fx(y))
    }

    /// Champ de coûts : distance de Manhattan × 10 depuis la case (0, 0) (cases de 8 px), sur [-160, 160]².
    fn field() -> FlowField {
        let mut field = FlowField::default();
        for x in -160..=160 {
            for y in -160..=160 {
                field
                    .costs
                    .insert(GridPos::new(x, y), ((x.abs() + y.abs()) * 10) as u32);
            }
        }
        field
    }

    fn room(id: &str, x: f32, boss: bool) -> RoomGoal {
        RoomGoal {
            id: id.into(),
            zone: room_zone(v(x, -64.0), v(128.0, 128.0)),
            boss,
            locking: true,
        }
    }

    #[test]
    fn salle_a_visiter() {
        // Récompense / boutique : un but jusqu'à la visite, quel que soit l'état.
        assert!(room_pending(false, false, RoomState::Dormant, false));
        // Combat dormant sans ennemi : jamais un but ; avec un ennemi : oui.
        assert!(!room_pending(true, false, RoomState::Dormant, false));
        assert!(room_pending(true, false, RoomState::Dormant, true));
        // Arène dormante et vide : un but ; nettoyée : non.
        assert!(room_pending(true, true, RoomState::Dormant, false));
        assert!(room_pending(true, true, RoomState::Locked, false));
        assert!(!room_pending(true, true, RoomState::Cleared, false));
    }

    #[test]
    fn zone_de_salle_reduite() {
        let zone = room_zone(v(0.0, 0.0), v(128.0, 128.0));
        assert_eq!(zone.min, v(40.0, 40.0));
        assert_eq!(zone.max, v(88.0, 88.0));
        let thin = room_zone(v(0.0, 0.0), v(64.0, 128.0));
        assert_eq!(thin.min.x, fx(32.0));
        assert_eq!(thin.max.x, fx(32.0));
    }

    #[test]
    fn salle_la_plus_proche_puis_boss_en_dernier() {
        let frame = ExploreFrame {
            rooms: vec![
                room("b", 100.0, false),
                room("a", 200.0, false),
                room("z", 50.0, true),
            ],
            ..Default::default()
        };
        let goals = ordered_goals(&field(), &frame, Holder::default(), false, false);
        let ids: Vec<_> = goals.iter().map(|(k, ..)| k.3.clone()).collect();
        assert_eq!(
            ids,
            vec!["b", "a"],
            "le boss (pourtant le plus proche) attend"
        );
        let frame = ExploreFrame {
            rooms: vec![room("z", 50.0, true)],
            ..Default::default()
        };
        let goals = ordered_goals(&field(), &frame, Holder::default(), false, false);
        assert_eq!(goals.len(), 1, "plus que le boss : il devient le but");
    }

    #[test]
    fn egalite_departagee_par_identifiant() {
        // Deux salles symétriques : même coût, l'identifiant décide (quel que soit l'ordre).
        let a = RoomGoal {
            id: "a".into(),
            zone: room_zone(v(-192.0, -64.0), v(128.0, 128.0)),
            boss: false,
            locking: true,
        };
        let b = RoomGoal {
            id: "b".into(),
            zone: room_zone(v(64.0, -64.0), v(128.0, 128.0)),
            boss: false,
            locking: true,
        };
        for rooms in [vec![a.clone(), b.clone()], vec![b, a]] {
            let frame = ExploreFrame {
                rooms,
                ..Default::default()
            };
            let goals = ordered_goals(&field(), &frame, Holder::default(), false, false);
            assert_eq!(goals[0].0 .0, goals[1].0 .0, "même coût");
            assert_eq!(goals[0].0 .3, "a");
        }
    }

    #[test]
    fn boss_direct_et_portail() {
        let frame = ExploreFrame {
            rooms: vec![room("b", 100.0, false), room("z", 300.0, true)],
            items: vec![ItemGoal {
                net_id: 7,
                position: v(16.0, 0.0),
                surface: false,
                active: false,
            }],
            ..Default::default()
        };
        let goals = ordered_goals(&field(), &frame, Holder::default(), false, true);
        assert_eq!(goals.len(), 1);
        assert_eq!(goals[0].0 .3, "z", "boss direct : seulement le boss");
        let goals = ordered_goals(&field(), &frame, Holder::default(), true, false);
        assert_eq!(goals.len(), 1, "portail ouvert : seulement les objets");
        assert_eq!(goals[0].2.map(|i| i.net_id), Some(7));
    }

    #[test]
    fn pas_d_actif_quand_on_en_tient_un() {
        let frame = ExploreFrame {
            items: vec![ItemGoal {
                net_id: 3,
                position: v(16.0, 0.0),
                surface: true,
                active: true,
            }],
            ..Default::default()
        };
        let holding = Holder {
            has_active: true,
            ..Default::default()
        };
        assert!(ordered_goals(&field(), &frame, holding, false, false).is_empty());
        assert_eq!(
            ordered_goals(&field(), &frame, Holder::default(), false, false).len(),
            1
        );
    }

    fn item(net_id: usize, x: f32) -> ItemGoal {
        ItemGoal {
            net_id,
            position: v(x, 0.0),
            surface: true,
            active: false,
        }
    }

    #[test]
    fn pas_de_detour_lointain() {
        // Un objet à 800 px (coût ~1000) ne vaut pas le détour, celui à 80 px oui : seul le boss
        // reste, il passe devant l'objet lointain.
        let frame = ExploreFrame {
            rooms: vec![room("z", 1000.0, true)],
            items: vec![item(1, 800.0), item(2, 80.0)],
            ..Default::default()
        };
        let goals = ordered_goals(&field(), &frame, Holder::default(), false, false);
        let ids: Vec<_> = goals.iter().map(|(k, ..)| (k.1, k.2)).collect();
        assert_eq!(ids, vec![(0, 2), (1, 0)], "objet proche puis boss");
        // Même règle quand il reste des salles de combat, et après la mort du boss (portail).
        let frame = ExploreFrame {
            rooms: vec![room("a", 200.0, false), room("z", 1000.0, true)],
            items: vec![item(1, 800.0), item(2, 80.0)],
            ..Default::default()
        };
        let goals = ordered_goals(&field(), &frame, Holder::default(), false, false);
        assert_eq!(goals.len(), 2, "l'objet proche et la salle");
        let goals = ordered_goals(&field(), &frame, Holder::default(), true, false);
        assert_eq!(goals.len(), 1, "portail ouvert : l'objet proche seulement");
    }

    #[test]
    fn detour_plus_court_sur_la_derniere_ligne_droite() {
        // Objet à 250 px (coût ~290) : un détour tant qu'une salle de combat reste, plus ensuite.
        let with_fight = ExploreFrame {
            rooms: vec![room("a", 200.0, false), room("z", 1000.0, true)],
            items: vec![item(1, 250.0)],
            ..Default::default()
        };
        let goals = ordered_goals(&field(), &with_fight, Holder::default(), false, false);
        assert_eq!(goals.len(), 2, "l'objet et la salle de combat");
        let boss_only = ExploreFrame {
            rooms: vec![room("z", 1000.0, true)],
            ..with_fight
        };
        let goals = ordered_goals(&field(), &boss_only, Holder::default(), false, false);
        assert_eq!(goals.len(), 1, "le boss seul");
    }

    #[test]
    fn salle_sans_verrou_lointaine_n_est_pas_un_but() {
        let passive = |id: &str, x: f32| RoomGoal {
            locking: false,
            ..room(id, x, false)
        };
        // Récompense lointaine (coût > 600) : le combat proche puis le boss ne l'attendent pas ;
        // proche, elle reste un détour.
        let frame = ExploreFrame {
            rooms: vec![
                passive("loin", 900.0),
                passive("pres", 100.0),
                room("combat", 300.0, false),
            ],
            ..Default::default()
        };
        let goals = ordered_goals(&field(), &frame, Holder::default(), false, false);
        let ids: Vec<_> = goals.iter().map(|(k, ..)| k.3.clone()).collect();
        assert_eq!(ids, vec!["pres", "combat"]);
        // Plus qu'une salle sans verrou lointaine et le boss : le boss est le but.
        let frame = ExploreFrame {
            rooms: vec![passive("loin", 900.0), room("z", 600.0, true)],
            ..Default::default()
        };
        let goals = ordered_goals(&field(), &frame, Holder::default(), false, false);
        let ids: Vec<_> = goals.iter().map(|(k, ..)| k.3.clone()).collect();
        assert_eq!(ids, vec!["z"]);
    }

    #[test]
    fn pas_de_presentoir_d_arme_sans_reserve() {
        let frame = ExploreFrame {
            items: vec![item(10, 80.0), item(11, 96.0), item(12, 112.0)],
            weapon_shelves: vec![(10, "fusil_precision".into()), (11, "pistolet".into())],
            ..Default::default()
        };
        // Réserve de `balles` seulement : le fusil (calibre) n'est pas utilisable.
        let filtered = frame
            .without_unusable_weapons(|weapon| weapon != "fusil_precision")
            .expect("un présentoir écarté");
        let ids: Vec<_> = filtered.items.iter().map(|i| i.net_id).collect();
        assert_eq!(
            ids,
            vec![11, 12],
            "le présentoir d'arme utilisable et le coffre restent"
        );
        // Tout est utilisable : aucune copie.
        assert!(frame.without_unusable_weapons(|_| true).is_none());
    }

    #[test]
    fn blank_sur_balle_proche_qui_approche() {
        let mut view = crate::decide::tests_support::view(v(0.0, 0.0));
        let bullet = |x: f32, vx: f32| ProjectileView {
            position: v(x, 0.0),
            velocity: v(vx, 0.0),
            size: fx(4.0),
            distance: fx(x.abs()),
        };
        let holder = Holder {
            has_active: false,
            blanks: 1,
            blank_held: false,
        };
        view.projectiles = vec![bullet(30.0, -4.0)];
        assert!(should_blank(&view, holder));
        assert!(!should_blank(
            &view,
            Holder {
                blanks: 0,
                ..holder
            }
        ));
        assert!(
            !should_blank(
                &view,
                Holder {
                    blank_held: true,
                    ..holder
                }
            ),
            "front montant : relâcher d'abord"
        );
        view.projectiles = vec![bullet(30.0, 4.0)];
        assert!(!should_blank(&view, holder), "s'éloigne");
        view.projectiles = vec![bullet(80.0, -4.0)];
        assert!(!should_blank(&view, holder), "trop loin");
    }

    #[test]
    fn l_elan_transverse_est_anticipe() {
        // Cap au nord (écart transverse de 3 px à droite) mais élan de 150 px/s vers la gauche : ~12 px
        // d'élan à rattraper, donc freiner à droite (la composante x garde son signe et grandit).
        let d = lead(v(3.0, 10.0), v(-150.0, 0.0));
        assert!(d.x > fx(10.0) && d.y == fx(10.0), "{d:?}");
        // Même écart sans élan : inchangé.
        assert_eq!(lead(v(3.0, 10.0), v(0.0, 0.0)), v(3.0, 10.0));
        // Élan vers l'axe : on lâche (le signe s'inverse : freinage avant de dépasser).
        assert!(lead(v(3.0, 10.0), v(150.0, 0.0)).x < fx(0.0));
        // L'axe du trajet n'est pas touché : à toute vitesse vers le nord, y reste.
        assert_eq!(lead(v(1.0, 10.0), v(0.0, 150.0)), v(1.0, 10.0));
        // Cap à l'est : c'est y qui est transverse.
        let d = lead(v(10.0, 2.0), v(0.0, -150.0));
        assert!(d.y > fx(10.0) && d.x == fx(10.0), "{d:?}");
    }

    #[test]
    fn patience_arret_puis_cap_droit_quand_le_bot_tourne_en_rond() {
        let mut patience = ExplorerPatience::default();
        let at = |x: f32, y: f32| v(x, y);
        // En mouvement (ou sans but) : jamais de repli, l'ancre suit.
        for frame in 0..300 {
            let p = patience.phase(0, frame, at(frame as f32 * 3.0, 0.0), true);
            assert_eq!(p, Patience::Normal);
        }
        // Oscille sur place (±12 px) avec un but : Normal jusqu'à STALL_FRAMES, puis arrêt, puis cap droit.
        let mut phases = Vec::new();
        for frame in 1000..1000 + STALL_FRAMES + PATIENCE_CYCLE + 5 {
            let wobble = if frame % 20 < 10 { 12.0 } else { -12.0 };
            phases.push(patience.phase(0, frame, at(0.0, wobble), true));
        }
        let first = |phase| phases.iter().position(|p| *p == phase).unwrap() as u32;
        assert_eq!(first(Patience::Settle), STALL_FRAMES);
        assert_eq!(first(Patience::Go), STALL_FRAMES + SETTLE_FRAMES);
        // La séquence recommence (arrêt puis cap droit) tant que le bot ne bouge pas.
        assert_eq!(
            phases[(STALL_FRAMES + PATIENCE_CYCLE) as usize],
            Patience::Settle
        );
        // Sans but, l'ancre est oubliée ; un autre joueur a sa propre ancre.
        assert_eq!(
            patience.phase(0, 2000, at(0.0, 0.0), false),
            Patience::Normal
        );
        assert_eq!(
            patience.phase(1, 2000, at(0.0, 0.0), true),
            Patience::Normal
        );
    }

    #[test]
    fn le_cap_droit_ne_corrige_pas_un_petit_ecart_transverse() {
        // Cap au nord, 4 px d'écart latéral (< 6) : tout droit.
        assert_eq!(straight(v(4.0, 10.0)), v(0.0, 10.0));
        // Écart de 8 px : corrigé comme avant.
        assert_eq!(straight(v(8.0, 10.0)), v(8.0, 10.0));
        // Cap à l'est : c'est y qui est transverse.
        assert_eq!(straight(v(10.0, -5.0)), v(10.0, 0.0));
    }
}
