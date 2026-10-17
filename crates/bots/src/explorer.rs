//! Profil `explorateur` (M2-T16, `docs/conventions.md` §47 ; M3-T14, §79) : joue seul un étage de
//! salles typées (mode `Floors`, grammaire d'étage §40, grille d'`isaac` §70).
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
//!   échangerait sans fin ; pas un consommable que son stock ou sa vie fait refuser, il attendrait
//!   sans fin sur le tas), une salle verrouillante encore à visiter (`Locked`, ou `Dormant` avec
//!   un ennemi dedans), la salle du boss en dernier (ou seule, [`ExplorerOptions::boss_direct`]).
//!   Portail ouvert : les salles ne comptent plus, `prudent` va au portail.
//! - **M3-T14 (isaac)**, politique en données ([`game::replay::ExplorerPolicy`]) : porte à clé
//!   (Interaction avec assez de clés), bombe contre le mur d'une salle secrète probable et devant
//!   le rocher qui coupe le chemin d'un but, fuite de la mèche, achats en douilles (cœur d'abord
//!   à vie basse, sinon le moins cher), marché du diable (`DevilPolicy`), sous au sol. Chaque
//!   décision est **inerte** sans son objet (aucune bombe, porte à clé, salle secrète, rocher,
//!   présentoir à `cost` : rien ne change pour `gungeon` ou `throne`).
//!
//! **Mémoire du bot** (piège « donnée dérivée non rollbackée » de `CLAUDE.md`) : aucune, hors
//! [`ExplorerVisits`]. « Salle visitée » se lit dans l'état : une salle verrouillante `Cleared`, ou
//! `Dormant` sans ennemi dedans (elle ne se verrouillera jamais), n'est plus un but ; une salle non
//! verrouillante (départ, récompense, boutique) n'est un but que par ses objets au sol. La porte
//! à clé ouverte, la salle secrète ouverte, le rocher cassé, la bombe posée se lisent dans
//! `DoorLocks`, les colliders, `Rock` et `Bomb`. Tout est recalculé à chaque frame depuis les
//! composants et ressources rollback ; les champs de navigation sont des caches dont les clés
//! contiennent toute la géométrie (vider le cache donne le même input).

use std::collections::{BTreeMap, BTreeSet};

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_fixed::fixed_math::{Fixed, FixedTransform3D, FixedVec2};
use bevy_ggrs::Rollback;
use game::character::enemy::ai::navigation::{FlowField, GridPos};
use game::character::enemy::Enemy;
use game::character::health::Health;
use game::character::player::input::{BoxInput, INPUT_BLANK, INPUT_BOMB, INPUT_INTERACTION};
use game::character::player::Player;
use game::items::BlankState;
use game::replay::{DevilPolicy, ExplorerPolicy, ShopOrder};
use items::{Inventory, ItemKind, ItemPickup, ItemTable, Price};
use sim_core::interaction::{Interactable, InteractionType};
use utils::{net_id::GgrsNetId, order_iter};
use world::{
    BombCarrier, BombDef, DoorLock, DoorLocks, RoomDormant, RoomKindTable, RoomState, RoomStates,
};

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

/// Marge intérieure au-delà de laquelle un joueur « est dans » une salle non verrouillante
/// (`PLAYER_INSET` de `map_ldtk::game::rooms`) : entrer suffit pour la visiter.
const VISIT_INSET: Fixed = Fixed::from_bits(24 << 16);

/// Une porte appartient à une salle si elle est à moins de cette distance de ses bords
/// (`DOOR_MARGIN` de `map_ldtk::game::rooms`).
const DOOR_MARGIN: Fixed = Fixed::from_bits(24 << 16);

/// Marge de la bombe sous son rayon : l'explosion doit toucher **strictement** (`world::explosion`),
/// et le bot mesure depuis son centre, pas depuis le point de pose.
const BOMB_STAND_MARGIN: Fixed = Fixed::from_bits(8 << 16);

/// Marge de fuite au-delà du rayon d'une bombe posée.
pub const FLEE_MARGIN: Fixed = Fixed::from_bits(16 << 16);

/// Distance (au centre d'un rocher) en deçà de laquelle une case du chemin sans rocher « passe par »
/// ce rocher : le corps du joueur (demi-diagonale ~14) y toucherait.
const ROCK_PATH_MARGIN: Fixed = Fixed::from_bits(14 << 16);

/// Salles non verrouillantes (récompense, boutique, départ) déjà visitées : `(index d'étage,
/// identifiant de salle)`. **Seule mémoire de l'explorateur**, hors simulation et hors rollback :
/// une salle sans état (`RoomStates` ne suit que les salles verrouillantes) ne laisse aucune trace
/// à lire. Elle ne sert qu'à produire des inputs, que GGRS enregistre ensuite : un rollback ne
/// rappelle jamais le bot pour une frame passée.
#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
pub struct ExplorerVisits(pub BTreeSet<(usize, String)>);

/// Navigation sans les rochers (M3-T14) : sert à savoir ce que la bombe ouvrirait. Cache dérivé,
/// comme [`BotNavigation`].
#[derive(Resource, Default)]
pub struct RockNavigation(pub BotNavigation);

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
    /// M3-T14 : presser Bomb cette frame (arrivé à la zone d'une pose ; front montant, `held`
    /// de `BombCarrier` est lu avant).
    pub bomb: bool,
    /// M3-T14 : pas de fuite hors du rayon d'une bombe posée (prioritaire sur tout le reste).
    pub flee: Option<FixedVec2>,
}

/// Réglages des `explorateur` (ressource ordinaire, hors rollback et hors simulation, posée avant
/// la partie par le runner ou par `alacod-sim`).
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq)]
pub struct ExplorerOptions {
    /// Aller droit au boss : les autres salles et les objets ne sont plus des buts.
    pub boss_direct: bool,
    /// Politique d'achat et de bombe (M3-T14) ; `policy.shopping` = D62 : ouvre les coffres et
    /// achète les présentoirs qu'il peut payer (`alacod-sim --shopping`). Faux par défaut : les
    /// traces des scénarios d'explorateur ne bougent pas.
    pub policy: ExplorerPolicy,
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
}

/// Ce qu'un objet exige de celui qui le vise (lu dans son état, jamais deviné).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GoalKind {
    /// Objet, coffre ou présentoir déjà filtré par `frame` (comportement d'avant M3-T14).
    Plain,
    /// Porte à clé : `keys` clés de celui qui interagit.
    Door { keys: u32 },
    /// Coffre : `keys` clé(s) (0 : ouvert d'office).
    Chest { keys: u32 },
    /// Présentoir payé en douilles ; `heart` : consommable `refuse_when_full`.
    Shelf { price: u32, heart: bool },
    /// Présentoir payé en vie maximale (`loss` points de vie).
    Devil { loss: Fixed },
}

/// Règle d'acceptation d'un consommable par un joueur (`items::ItemDef::accepts`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickupRule {
    pub id: String,
    pub max_stack: Option<u32>,
    pub refuse_when_full: bool,
}

impl PickupRule {
    /// Même règle que `ItemDef::accepts` pour un consommable.
    pub fn accepts(&self, stock: u32, at_full_health: bool) -> bool {
        if self.max_stack.is_some_and(|cap| stock >= cap) {
            return false;
        }
        !(self.refuse_when_full && at_full_health)
    }
}

/// Un objet au sol, un coffre, un présentoir ou une porte à clé.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemGoal {
    pub net_id: usize,
    pub position: FixedVec2,
    /// Ramassé par Interaction (passif, actif), sinon au contact (consommable).
    pub surface: bool,
    /// Objet actif (`ItemKind::Active`).
    pub active: bool,
    /// Zone d'approche : le point de l'objet, ou le collider d'une porte.
    pub zone: Rect,
    /// Portée d'approche autour de `zone`.
    pub reach: Fixed,
    pub kind: GoalKind,
    pub rule: Option<PickupRule>,
    /// Coût de chemin au-delà duquel l'objet n'est plus un but (sous au sol : `ExplorerPolicy::shells`).
    pub max_cost: Option<u32>,
}

impl ItemGoal {
    pub fn new(net_id: usize, position: FixedVec2, surface: bool, active: bool) -> Self {
        Self {
            net_id,
            position,
            surface,
            active,
            zone: Rect {
                min: position,
                max: position,
            },
            reach: ITEM_REACH,
            kind: GoalKind::Plain,
            rule: None,
            max_cost: None,
        }
    }
}

/// Une bombe posée (mèche en cours).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BombView {
    pub position: FixedVec2,
    pub radius: Fixed,
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
    /// M3-T14 : portes closes (colliders) des salles secrètes probables encore fermées.
    pub secret_doors: Vec<Rect>,
    /// M3-T14 : centre de chaque rocher, par `GgrsNetId`.
    pub rocks: Vec<(usize, Rect)>,
    /// M3-T14 : bombes posées.
    pub bombs: Vec<BombView>,
}

/// Ce que le joueur porte et ce qu'il a (lu dans son `Inventory`, son `BlankState`, sa santé).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Holder {
    pub has_active: bool,
    pub blanks: u32,
    /// Blank tenu à la frame précédente (`BlankState::held`).
    pub blank_held: bool,
    /// Clés (`game::loot::KEY_ITEM`).
    pub keys: u32,
    /// Bombes en stock (`game::loot::BOMB_ITEM`).
    pub bombs: u32,
    /// Définition de sa bombe (`BombCarrier`) ; `None` : il ne peut pas en poser.
    pub bomb: Option<BombDef>,
    /// Bouton Bomb tenu à la frame précédente (`BombCarrier::held`).
    pub bomb_held: bool,
    /// Écart entre le centre de son collider (point de pose de la bombe) et sa position.
    pub bomb_inset: Fixed,
    /// Douilles (`run::Currency`).
    pub wallet: u32,
    pub health: Fixed,
    pub health_max: Fixed,
    /// Points de vie d'âme (`combat::actors::ExtraHealth`).
    pub soul: Fixed,
    /// Consommables par identifiant.
    pub stock: BTreeMap<String, u32>,
}

impl Holder {
    fn at_full_health(&self) -> bool {
        self.health >= self.health_max
    }

    /// Peut-il poser une bombe maintenant (stock au-dessus de la réserve) ?
    pub fn can_bomb(&self, policy: &ExplorerPolicy) -> bool {
        self.bomb.is_some() && self.bombs > policy.bombs.reserve
    }
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
            Option<&'static Health>,
            Option<&'static BombCarrier>,
            Option<&'static combat::actors::ExtraHealth>,
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
    shells: Query<
        'w,
        's,
        (&'static GgrsNetId, &'static FixedTransform3D),
        (With<items::ShellPickup>, With<Rollback>),
    >,
    doors: Query<
        'w,
        's,
        (
            &'static GgrsNetId,
            &'static FixedTransform3D,
            &'static game::collider::Collider,
            Option<&'static Interactable>,
        ),
        (
            With<map::game::entity::map::door::DoorComponent>,
            With<Rollback>,
        ),
    >,
    rocks: Query<
        'w,
        's,
        (
            &'static GgrsNetId,
            &'static FixedTransform3D,
            &'static game::collider::Collider,
        ),
        (With<world::Rock>, With<Rollback>),
    >,
    bombs: Query<
        'w,
        's,
        (
            &'static GgrsNetId,
            &'static FixedTransform3D,
            &'static world::Bomb,
        ),
        With<Rollback>,
    >,
    door_locks: Option<Res<'w, DoorLocks>>,
    wallets: Query<'w, 's, (&'static Player, &'static run::Currency)>,
    options: Option<Res<'w, ExplorerOptions>>,
    visits: ResMut<'w, ExplorerVisits>,
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
                    });
                }
            }
        }
        rooms.sort_by(|a, b| a.id.cmp(&b.id));
        let policy = self.options().policy;
        let mut items: Vec<ItemGoal> = order_iter!(self.items)
            .into_iter()
            .map(|(id, t, pickup, surface)| {
                let def = self
                    .item_table
                    .as_deref()
                    .and_then(|table| table.0.get(&pickup.item_id));
                let mut goal = ItemGoal::new(
                    id.0,
                    t.translation.truncate(),
                    surface,
                    def.is_some_and(|def| matches!(def.kind, ItemKind::Active(_))),
                );
                goal.rule = def.and_then(|def| pickup_rule(&pickup.item_id, def));
                goal
            })
            .collect();
        if policy.shopping {
            if matches!(policy.shop, ShopOrder::Nearest) {
                self.shopping_goals(&mut items);
            } else {
                self.per_player_shopping_goals(&mut items);
            }
        }
        if let Some(limit) = policy.shells {
            for (id, t) in order_iter!(self.shells) {
                let mut goal = ItemGoal::new(id.0, t.translation.truncate(), false, false);
                goal.max_cost = Some(limit);
                items.push(goal);
            }
        }
        let floor_u32 = floor as u32;
        self.key_door_goals(&mut items, floor_u32);
        items.sort_by_key(|goal| goal.net_id);
        let secret_doors = self.secret_doors(&policy);
        let rocks: Vec<(usize, Rect)> = order_iter!(self.rocks)
            .into_iter()
            .map(|(id, t, collider)| (id.0, Rect::collider(t.translation.truncate(), collider)))
            .collect();
        let bombs: Vec<BombView> = order_iter!(self.bombs)
            .into_iter()
            .map(|(_, t, bomb)| BombView {
                position: t.translation.truncate(),
                radius: bomb.radius,
            })
            .collect();
        ExploreFrame {
            rooms,
            items,
            dormant,
            secret_doors,
            rocks,
            bombs,
        }
    }

    /// D62 : présentoirs payables et coffres ouvrables (but d'exploration par Interaction, comme
    /// un objet passif au sol). L'arme qui en sort est ramassée par le combat de `prudent`
    /// (butin au sol), jamais par un but d'ici. Solde et clés se lisent dans l'état.
    fn shopping_goals(&self, items: &mut Vec<ItemGoal>) {
        let wallet = self.wallets.iter().map(|(_, c)| c.0).max().unwrap_or(0);
        let keys = self
            .holders
            .iter()
            .filter_map(|(_, inventory, ..)| inventory)
            .map(|inventory| inventory.consumable(game::loot::KEY_ITEM))
            .max()
            .unwrap_or(0);
        for (id, t, shelf, interactable) in order_iter!(self.shelves) {
            // M3-T8 (§73) : un présentoir à `cost` (marché du diable : vie maximale) n'est pas un achat
            // en douilles, son `price` vaut 0 ; sans politique d'isaac le bot ne le vise pas (T14 : `DevilPolicy`).
            if interactable && !shelf.sold && shelf.cost.is_none() && shelf.price <= wallet {
                items.push(ItemGoal::new(id.0, t.translation.truncate(), true, false));
            }
        }
        for (id, t, chest, interactable) in order_iter!(self.chests) {
            if interactable && !chest.opened && (!chest.locked || keys > 0) {
                items.push(ItemGoal::new(id.0, t.translation.truncate(), true, false));
            }
        }
    }

    /// M3-T14 : présentoirs (douilles ou vie maximale) et coffres de la frame, avec ce que chacun
    /// exige ; `ordered_goals` retient pour chaque joueur ce qu'il peut payer et prendre.
    fn per_player_shopping_goals(&self, items: &mut Vec<ItemGoal>) {
        for (id, t, shelf, interactable) in order_iter!(self.shelves) {
            if !interactable || shelf.sold {
                continue;
            }
            let def = self
                .item_table
                .as_deref()
                .and_then(|table| table.0.get(&shelf.item_id));
            let mut goal = ItemGoal::new(id.0, t.translation.truncate(), true, false);
            goal.rule = def.and_then(|def| pickup_rule(&shelf.item_id, def));
            goal.kind = match &shelf.cost {
                None => GoalKind::Shelf {
                    price: shelf.price,
                    heart: def.is_some_and(|def| def.refuse_when_full),
                },
                Some(Price::MaxHealth(loss)) => GoalKind::Devil { loss: *loss },
                // Prix en consommable : aucun contenu ne l'emploie, le bot ne le vise pas.
                Some(_) => continue,
            };
            items.push(goal);
        }
        for (id, t, chest, interactable) in order_iter!(self.chests) {
            if interactable && !chest.opened {
                let mut goal = ItemGoal::new(id.0, t.translation.truncate(), true, false);
                goal.kind = GoalKind::Chest {
                    keys: u32::from(chest.locked),
                };
                items.push(goal);
            }
        }
    }

    /// M3-T14 : portes à clé encore closes de l'étage courant (`door_lock: Key(n)`), approchées par
    /// leur collider à la portée d'interaction. Une porte ouverte a perdu son `Interactable`
    /// (`key_door_system`), une salle déjà dans `DoorLocks` n'est plus un but.
    fn key_door_goals(&self, items: &mut Vec<ItemGoal>, floor: u32) {
        let Some(table) = self.table.as_deref() else {
            return;
        };
        for (id, t, collider, interactable) in order_iter!(self.doors) {
            let Some(interactable) =
                interactable.filter(|i| i.interaction_type == InteractionType::KeyDoor)
            else {
                continue;
            };
            let p = t.translation.truncate();
            // Même ordre que `key_door_system` (`sorted_rooms` : position puis identifiant) : la première salle
            // à clé dont la porte est proche est celle dont le jeu lit le prix.
            let mut rooms: Vec<_> = self.rooms.iter().collect();
            rooms.sort_by(|a, b| {
                (a.1.position.x, a.1.position.y, &a.0 .0).cmp(&(
                    b.1.position.x,
                    b.1.position.y,
                    &b.0 .0,
                ))
            });
            let Some((level, keys)) = rooms.into_iter().find_map(|(level, bounds, kind)| {
                let Some(DoorLock::Key(keys)) = table.door_lock_at(&kind.0, floor) else {
                    return None;
                };
                world::rooms::inside_box(bounds.position, bounds.size, p, -DOOR_MARGIN)
                    .then(|| (level.0.clone(), keys))
            }) else {
                continue;
            };
            if self
                .door_locks
                .as_deref()
                .is_some_and(|l| l.is_open(&level))
            {
                continue;
            }
            let mut goal = ItemGoal::new(id.0, p, true, false);
            goal.zone = Rect::collider(p, collider);
            goal.reach = (interactable.interaction_range - Fixed::from_num(8)).max(Fixed::ZERO);
            goal.kind = GoalKind::Door { keys };
            items.push(goal);
        }
    }

    /// M3-T14 : portes closes des salles secrètes « probables » : type `door_lock: Secret` encore
    /// fermé (`DoorLocks`) qui touche au moins `bombs.secret_min_neighbours` autres salles par un
    /// bord commun (règle de la grille, §70). Le bot lit le type de la salle dans le contenu : la
    /// minicarte, elle, ne la montre pas (dette §79 : une inférence sans cette lecture).
    fn secret_doors(&self, policy: &ExplorerPolicy) -> Vec<Rect> {
        let Some(table) = self.table.as_deref() else {
            return Vec::new();
        };
        if !policy.bombs.secret {
            return Vec::new();
        }
        let rooms: Vec<_> = self.rooms.iter().collect();
        let mut out = Vec::new();
        for (level, bounds, kind) in &rooms {
            if table.door_lock(&kind.0) != Some(DoorLock::Secret)
                || self
                    .door_locks
                    .as_deref()
                    .is_some_and(|locks| locks.is_open(&level.0))
            {
                continue;
            }
            let neighbours = rooms
                .iter()
                .filter(|(other, other_bounds, _)| {
                    other.0 != level.0
                        && share_an_edge(
                            bounds.position,
                            bounds.size,
                            other_bounds.position,
                            other_bounds.size,
                        )
                })
                .count();
            if (neighbours as u32) < policy.bombs.secret_min_neighbours {
                continue;
            }
            for (_, t, collider, _) in order_iter!(self.doors) {
                let p = t.translation.truncate();
                if world::rooms::inside_box(bounds.position, bounds.size, p, -DOOR_MARGIN) {
                    out.push(Rect::collider(p, collider));
                }
            }
        }
        // Ordre indépendant de celui de la requête des salles : (position, taille) croissantes.
        out.sort_by_key(|r| (r.min.x, r.min.y, r.max.x, r.max.y));
        out.dedup();
        out
    }

    /// Ce que porte le joueur `handle`.
    pub fn holder(&self, handle: usize) -> Holder {
        let Some((_, inventory, blank, health, carrier, soul)) =
            self.holders.iter().find(|(p, ..)| p.handle == handle)
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
            keys: inventory.map_or(0, |i| i.consumable(game::loot::KEY_ITEM)),
            bombs: inventory.map_or(0, |i| i.consumable(game::loot::BOMB_ITEM)),
            bomb: carrier.map(|c| c.def),
            bomb_held: carrier.is_some_and(|c| c.held),
            bomb_inset: Fixed::ZERO,
            wallet: self
                .wallets
                .iter()
                .find(|(p, _)| p.handle == handle)
                .map_or(0, |(_, c)| c.0),
            health: health.map_or(Fixed::ZERO, |h| h.current),
            health_max: health.map_or(Fixed::ZERO, |h| h.max),
            soul: soul.map_or(Fixed::ZERO, |s| s.soul),
            stock: inventory.map(|i| i.consumables.clone()).unwrap_or_default(),
        }
    }
}

/// Règle d'acceptation d'un consommable (`None` pour un passif ou un actif, toujours pris).
fn pickup_rule(id: &str, def: &items::ItemDef) -> Option<PickupRule> {
    matches!(def.kind, ItemKind::Consumable).then(|| PickupRule {
        id: id.to_string(),
        max_stack: def.max_stack,
        refuse_when_full: def.refuse_when_full,
    })
}

/// Deux rectangles (position, taille) se touchent par un bord commun de longueur non nulle
/// (tolérance d'un pixel : les salles d'une grille sont alignées sur des cases entières).
pub fn share_an_edge(a: FixedVec2, a_size: FixedVec2, b: FixedVec2, b_size: FixedVec2) -> bool {
    let tolerance = Fixed::ONE;
    let overlap = |a0: Fixed, a1: Fixed, b0: Fixed, b1: Fixed| a0.max(b0) < a1.min(b1);
    let touch = |a_end: Fixed, b_start: Fixed| (a_end - b_start).abs() <= tolerance;
    let (ax1, ay1, bx1, by1) = (
        a.x + a_size.x,
        a.y + a_size.y,
        b.x + b_size.x,
        b.y + b_size.y,
    );
    (touch(ax1, b.x) || touch(bx1, a.x)) && overlap(a.y, ay1, b.y, by1)
        || (touch(ay1, b.y) || touch(by1, a.y)) && overlap(a.x, ax1, b.x, bx1)
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

/// Case la moins chère (champ `field`) à moins de `reach` de `zone`, avec son coût (égalité : la
/// plus petite case).
pub fn zone_cell(field: &FlowField, zone: Rect, reach: Fixed) -> Option<(u32, GridPos)> {
    let margin = (reach / Fixed::from_num(8)).ceil().to_num::<i32>() + 1;
    let (a, b) = (cell(zone.min), cell(zone.max));
    let mut best: Option<(u32, GridPos)> = None;
    for x in a.x - margin..=b.x + margin {
        for y in a.y - margin..=b.y + margin {
            let p = GridPos::new(x, y);
            if zone.distance(point(p)) > reach {
                continue;
            }
            if let Some(&cost) = field.costs.get(&p) {
                if best.is_none_or(|(c, q)| (cost, p) < (c, q)) {
                    best = Some((cost, p));
                }
            }
        }
    }
    best
}

/// Coût du chemin (champ `field`) jusqu'à la case la moins chère à moins de `reach` de `zone`.
pub fn zone_cost(field: &FlowField, zone: Rect, reach: Fixed) -> Option<u32> {
    zone_cell(field, zone, reach).map(|(cost, _)| cost)
}

/// Ce qu'un but d'exploration demande au bot.
#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    Room(Rect),
    Item(ItemGoal),
    /// Se tenir à moins de `reach` de `zone` (mur secret, rocher) et poser une bombe.
    Bomb {
        zone: Rect,
        reach: Fixed,
    },
}

/// Clé d'ordre d'un but : coût du chemin, objets et bombes (0) avant salles (1), `GgrsNetId` de
/// l'objet (rang de la porte pour une bombe), identifiant de la salle.
type GoalKey = (u32, u8, usize, String);

/// Ce qui décide des buts d'un joueur : ce qu'il porte, l'état du portail et la politique.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GoalContext {
    pub holder: Holder,
    pub portal_open: bool,
    pub boss_direct: bool,
    pub policy: ExplorerPolicy,
    /// Positions des autres joueurs **de plus petit handle** qui peuvent poser une bombe (stock au-dessus de la
    /// réserve, porteur) : un mur ou un rocher dont l'un d'eux est à portée de pose est le sien, on n'y pose
    /// pas une seconde bombe (deux bots arrivés ensemble au même mur en posaient chacun une). Le plus petit
    /// handle n'attend jamais personne.
    pub peers: Vec<FixedVec2>,
}

impl GoalContext {
    /// Portée de pose d'une bombe : l'explosion touche strictement à moins de son rayon.
    fn bomb_reach(&self) -> Option<Fixed> {
        let def = self.holder.bomb?;
        Some((def.radius - BOMB_STAND_MARGIN - self.holder.bomb_inset).max(Fixed::from_num(4)))
    }

    /// Peut-il viser une pose de bombe : un porteur avec du stock, aucune mèche en cours.
    fn bomb_ready(&self, frame: &ExploreFrame) -> bool {
        self.holder.can_bomb(&self.policy) && frame.bombs.is_empty()
    }

    /// Un joueur de plus petit handle, porteur de bombe, est déjà à portée de pose de `zone` (plus une marge
    /// de 40) : c'est lui qui la posera.
    fn peer_covers(&self, zone: Rect, reach: Fixed) -> bool {
        self.peers
            .iter()
            .any(|peer| zone.distance(*peer) <= reach + Fixed::from_num(40))
    }

    /// Le joueur prendrait-il cet objet, le paierait-il ? (Les filtres d'avant M3-T14 sont dans
    /// `ExplorerWorld::frame` : `Plain` passe.)
    fn wants(&self, item: &ItemGoal) -> bool {
        let holder = &self.holder;
        if item.active && holder.has_active {
            return false;
        }
        if let Some(rule) = &item.rule {
            let stock = holder.stock.get(&rule.id).copied().unwrap_or(0);
            if !rule.accepts(stock, holder.at_full_health()) {
                return false;
            }
        }
        match item.kind {
            GoalKind::Plain => true,
            GoalKind::Door { keys } | GoalKind::Chest { keys } => holder.keys >= keys,
            GoalKind::Shelf { price, .. } => holder.wallet >= price,
            GoalKind::Devil { loss } => devil_accepts(self.policy.devil, holder, loss),
        }
    }
}

/// Le marché du diable (`Price::MaxHealth(loss)`) : l'âme paie d'abord (`Price::soul_split`) ; ce
/// qu'elle ne couvre pas se paie en vie maximale, refusé s'il tuerait (`vie <= reste`). `IfHealthAbove(n)` :
/// seulement si la vie courante dépasse **strictement** `n` — ou si l'âme couvre tout le prix
/// (rien ne part).
pub fn devil_accepts(policy: DevilPolicy, holder: &Holder, loss: Fixed) -> bool {
    let DevilPolicy::IfHealthAbove(threshold) = policy else {
        return false;
    };
    let rest = (loss - holder.soul).max(Fixed::ZERO);
    rest == Fixed::ZERO
        || (holder.health > threshold && holder.health > rest && rest <= holder.health_max)
}

/// Buts d'exploration triés par [`GoalKey`] (seulement ceux qu'un chemin atteint).
pub fn ordered_goals(
    field: &FlowField,
    frame: &ExploreFrame,
    ctx: &GoalContext,
) -> Vec<(GoalKey, Target)> {
    let mut goals: Vec<(GoalKey, Target)> = Vec::new();
    if !ctx.boss_direct {
        // Présentoirs : un seul retenu par groupe (cœur à vie basse, puis le moins cher ; diable :
        // la perte la plus faible), les autres suivront une fois celui-ci pris.
        let mut shelves: Vec<((u8, u32, u32, usize), GoalKey, ItemGoal)> = Vec::new();
        let mut devils: Vec<((Fixed, u32, usize), GoalKey, ItemGoal)> = Vec::new();
        for item in &frame.items {
            if !ctx.wants(item) {
                continue;
            }
            let Some(cost) = zone_cost(field, item.zone, item.reach) else {
                continue;
            };
            if item.max_cost.is_some_and(|limit| cost > limit) {
                continue;
            }
            let key: GoalKey = (cost, 0, item.net_id, String::new());
            match item.kind {
                GoalKind::Shelf { price, heart } => {
                    let hearts_first = match ctx.policy.shop {
                        ShopOrder::HeartsThenCheapest { low_health } => {
                            heart
                                && ctx.holder.health_max > Fixed::ZERO
                                && ctx.holder.health <= ctx.holder.health_max * low_health
                        }
                        ShopOrder::Nearest => false,
                    };
                    let rank = (u8::from(!hearts_first), price, cost, item.net_id);
                    shelves.push((rank, key, item.clone()));
                }
                GoalKind::Devil { loss } => {
                    devils.push(((loss, cost, item.net_id), key, item.clone()));
                }
                _ => goals.push((key, Target::Item(item.clone()))),
            }
        }
        if let Some((_, key, item)) = shelves.into_iter().min_by(|a, b| a.0.cmp(&b.0)) {
            goals.push((key, Target::Item(item)));
        }
        if let Some((_, key, item)) = devils.into_iter().min_by(|a, b| a.0.cmp(&b.0)) {
            goals.push((key, Target::Item(item)));
        }
        if let (true, Some(reach)) = (ctx.bomb_ready(frame), ctx.bomb_reach()) {
            for (rank, door) in frame.secret_doors.iter().enumerate() {
                if ctx.peer_covers(*door, reach) {
                    continue;
                }
                if let Some(cost) = zone_cost(field, *door, reach) {
                    goals.push((
                        (cost, 0, usize::MAX - rank, String::new()),
                        Target::Bomb { zone: *door, reach },
                    ));
                }
            }
        }
    }
    if !ctx.portal_open {
        let bosses = frame.rooms.iter().any(|room| room.boss);
        let mut rooms: Vec<(GoalKey, Target)> = Vec::new();
        let mut boss_rooms: Vec<(GoalKey, Target)> = Vec::new();
        for room in &frame.rooms {
            let Some(cost) = zone_cost(field, room.zone, Fixed::ZERO) else {
                continue;
            };
            let entry = ((cost, 1, 0, room.id.clone()), Target::Room(room.zone));
            if room.boss {
                boss_rooms.push(entry);
            } else {
                rooms.push(entry);
            }
        }
        if ctx.boss_direct && bosses {
            goals.extend(boss_rooms);
        } else if rooms.is_empty() {
            // Le boss en dernier : seulement quand aucune autre salle n'est atteignable.
            goals.extend(boss_rooms);
        } else {
            goals.extend(rooms);
        }
    }
    goals.sort_by(|a, b| a.0.cmp(&b.0));
    goals
}

/// M3-T14 : un rocher qui coupe le chemin d'un but. Quand plus aucun but n'est atteignable alors
/// qu'il en reste (salles à visiter, objets, portail), le champ **sans les rochers** (`free`) dit
/// lesquels seraient atteints ; le rocher visé est le plus proche du joueur sur le chemin vers le
/// but le moins cher de ce champ, et le but est la zone de pose, à portée de bombe du rocher,
/// atteignable dans le champ réel (`field`).
pub fn rock_goal(
    field: &FlowField,
    free: &FlowField,
    frame: &ExploreFrame,
    ctx: &GoalContext,
    portal: Option<FixedVec2>,
) -> Option<Target> {
    if !ctx.policy.bombs.rocks || frame.rocks.is_empty() || !ctx.bomb_ready(frame) {
        return None;
    }
    let reach = ctx.bomb_reach()?;
    let mut targets: Vec<(Rect, Fixed)> = Vec::new();
    if let (true, Some(p)) = (ctx.portal_open, portal) {
        targets.push((Rect { min: p, max: p }, ITEM_REACH));
    }
    if !ctx.portal_open {
        targets.extend(frame.rooms.iter().map(|room| (room.zone, Fixed::ZERO)));
    }
    targets.extend(
        frame
            .items
            .iter()
            .filter(|item| ctx.wants(item))
            .map(|item| (item.zone, item.reach)),
    );
    // Seulement un but que le champ réel n'atteint pas : un but atteignable (détour, sous au-delà de `shells`)
    // ne vaut pas une bombe.
    let mut candidates: Vec<(u32, GridPos)> = targets
        .iter()
        .filter(|(zone, reach)| zone_cell(field, *zone, *reach).is_none())
        .filter_map(|(zone, reach)| zone_cell(free, *zone, *reach))
        .collect();
    candidates.sort();
    candidates.into_iter().find_map(|(_, start)| {
        let mut at = start;
        let mut last: Option<(Fixed, usize, Rect)> = None;
        for _ in 0..free.costs.len().max(1) {
            for (id, rock) in &frame.rocks {
                // Distance au bord du rocher (16 x 16) : le corps (demi-diagonale ~14) le touche dès là.
                let distance = rock.distance(point(at));
                if distance <= ROCK_PATH_MARGIN {
                    last = Some((distance, *id, *rock));
                }
            }
            match free.directions.get(&at) {
                Some(&next) if next != at => at = next,
                _ => break,
            }
        }
        let (_, _, rock) = last?;
        // La bombe vise le centre du rocher (`cells_in_radius` : centre à moins du rayon).
        let center = (rock.min + rock.max) / Fixed::from_num(2);
        let zone = Rect {
            min: center,
            max: center,
        };
        if ctx.peer_covers(zone, reach) {
            return None;
        }
        zone_cost(field, zone, reach).map(|_| Target::Bomb { zone, reach })
    })
}

/// Un pas d'exploration : direction (nulle : arrivé), Interaction à tenir, bombe à poser.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Step {
    pub direction: FixedVec2,
    pub interact: bool,
    pub bomb: bool,
    /// Nature du but suivi (`room`, `boss`…, `door`, `shelf`, `devil`, `chest`, `item`, `shell`, `bomb`) : diagnostic
    /// (`ALACOD_BOT_DEBUG`), jamais lue par une décision.
    pub what: &'static str,
    /// Identité du but suivi (objet : son `GgrsNetId`, salle ou mur : sa zone) : l'état anti-jitter ne s'accroche
    /// qu'au même but.
    pub target: u64,
}

/// Un point d'attache tenu sans quitter [`JITTER_RADIUS`] pendant [`JITTER_FRAMES`] frames en voulant se déplacer est
/// un balancement : deux buts (ou deux routes) de coût presque égal dont l'ordre s'inverse à chaque pas, la commande
/// arrivant avec le délai d'input (graine 19 d'`isaac` sans bombe : deux salles, 600 frames d'aller-retour). Le bot
/// **s'attache** alors à son but du moment (il passe en premier tant qu'il est atteignable) et **tient** son prochain
/// point de passage jusqu'à l'atteindre.
pub const JITTER_RADIUS: Fixed = Fixed::from_bits(12 << 16);
/// Frames sur place avant de s'attacher à un but.
pub const JITTER_FRAMES: u32 = 40;
/// Un point de passage est atteint à moins de cette distance.
const WAYPOINT_REACHED: Fixed = Fixed::from_bits(3 << 16);

/// État anti-jitter d'un joueur.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jitter {
    anchor: FixedVec2,
    since: u32,
    /// But auquel le bot s'est attaché (`Step::target`).
    focus: Option<u64>,
    hold: Option<(u64, FixedVec2)>,
}

/// Anti-jitter des `explorateur` (`ExplorerPolicy::anti_jitter`) : **seconde mémoire du bot** après [`ExplorerVisits`],
/// hors simulation et hors rollback pour la même raison (elle ne sert qu'à produire des inputs que GGRS enregistre).
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct ExplorerJitter(pub BTreeMap<usize, Jitter>);

impl ExplorerJitter {
    /// But auquel le joueur `handle` s'est attaché (`explore_step` le passe en premier).
    pub fn focus(&self, handle: usize) -> Option<u64> {
        self.0.get(&handle).and_then(|state| state.focus)
    }

    /// Corrige `step` (le pas vers un but, direction non nulle) si le joueur `handle` balance sur place ; `clear` :
    /// le segment `de → vers` est libre pour son corps.
    pub fn steady(
        &mut self,
        handle: usize,
        frame: u32,
        position: FixedVec2,
        step: &mut Step,
        clear: impl Fn(FixedVec2, FixedVec2) -> bool,
    ) {
        if step.direction == FixedVec2::ZERO {
            self.0.remove(&handle);
            return;
        }
        let fresh = Jitter {
            anchor: position,
            since: frame,
            focus: None,
            hold: None,
        };
        let state = self.0.entry(handle).or_insert(fresh);
        if position.distance(&state.anchor) > JITTER_RADIUS {
            *state = fresh;
        }
        if let Some((target, waypoint)) = state.hold {
            if target == step.target
                && position.distance(&waypoint) > WAYPOINT_REACHED
                && clear(position, waypoint)
            {
                step.direction = crate::input::dead_zone(waypoint - position);
                return;
            }
            state.hold = None;
        }
        if frame.saturating_sub(state.since) >= JITTER_FRAMES {
            state.focus = Some(step.target);
            state.hold = Some((step.target, position + step.direction));
        }
    }
}

impl Target {
    /// Identité du but (voir [`Step::target`]).
    fn identity(&self) -> u64 {
        let rect = |r: &Rect| {
            let bits = |f: Fixed| u64::from(f.to_bits() as u32);
            (bits(r.min.x) << 48) ^ (bits(r.min.y) << 32) ^ (bits(r.max.x) << 16) ^ bits(r.max.y)
        };
        match self {
            Target::Room(zone) => rect(zone),
            Target::Bomb { zone, .. } => rect(zone) ^ 1,
            Target::Item(item) => (1 << 63) | item.net_id as u64,
        }
    }
}

impl Target {
    /// Nature du but, pour le diagnostic.
    fn describe(&self) -> &'static str {
        match self {
            Target::Room(_) => "room",
            Target::Bomb { .. } => "bomb",
            Target::Item(item) => match item.kind {
                GoalKind::Door { .. } => "door",
                GoalKind::Chest { .. } => "chest",
                GoalKind::Shelf { .. } => "shelf",
                GoalKind::Devil { .. } => "devil",
                GoalKind::Plain if item.max_cost.is_some() => "shell",
                GoalKind::Plain => "item",
            },
        }
    }
}

/// Pas suivant vers le premier but d'exploration accessible (voir le module) et Interaction à
/// tenir. La navigation doit avoir été mise à jour (`update_from`) pour cette frame. `free` : la
/// navigation sans les rochers, mise à jour de même (`None` : pas de rochers à considérer).
#[allow(clippy::too_many_arguments)]
pub fn explore_step(
    nav: &mut BotNavigation,
    free: Option<&mut BotNavigation>,
    frame: &ExploreFrame,
    ctx: &GoalContext,
    position: FixedVec2,
    portal: Option<FixedVec2>,
    surfaces: &[(usize, Rect, Fixed, bool)],
    prefer: Option<u64>,
) -> Option<Step> {
    let goals: Vec<Target> = {
        let field = nav.costs_from(position)?;
        let mut goals: Vec<Target> = ordered_goals(field, frame, ctx)
            .into_iter()
            .map(|(_, target)| target)
            .collect();
        if goals.is_empty() {
            if let Some(free) = free {
                if let Some(free_field) = free.costs_from(position) {
                    goals.extend(rock_goal(field, free_field, frame, ctx, portal));
                }
            }
        }
        // But d'attache de l'anti-jitter : en premier tant qu'il est un but atteignable.
        if let Some(index) = prefer.and_then(|who| goals.iter().position(|g| g.identity() == who)) {
            let goal = goals.remove(index);
            goals.insert(0, goal);
        }
        goals
    };
    goals.into_iter().find_map(|target| {
        let what = target.describe();
        let who = target.identity();
        let step = match target {
            Target::Item(item) if item.surface => {
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
                .map(|(direction, interact)| Step {
                    direction,
                    interact,
                    bomb: false,
                    what,
                    target: 0,
                })
            }
            Target::Item(item) => {
                nav.approach(position, item.zone, ITEM_REACH)
                    .map(|(direction, _)| Step {
                        direction: crate::input::dead_zone(direction),
                        interact: false,
                        bomb: false,
                        what,
                        target: 0,
                    })
            }
            Target::Room(zone) => {
                nav.approach(position, zone, Fixed::ZERO)
                    .map(|(direction, _)| Step {
                        direction: crate::input::dead_zone(direction),
                        interact: false,
                        bomb: false,
                        what,
                        target: 0,
                    })
            }
            Target::Bomb { zone, reach } => {
                nav.approach(position, zone, reach).map(|(direction, _)| {
                    let direction = crate::input::dead_zone(direction);
                    Step {
                        direction,
                        interact: false,
                        bomb: direction == FixedVec2::ZERO && !ctx.holder.bomb_held,
                        what,
                        target: 0,
                    }
                })
            }
        };
        step.map(|step| Step {
            target: who,
            ..step
        })
    })
}

/// Fuite hors du rayon d'une bombe posée (mèche en cours, tir ami des bombes d'`isaac`) : vers la
/// case accessible la moins chère qui est au-delà du rayon (et de [`FLEE_MARGIN`]) de **toutes** les
/// bombes ; sinon, directement à l'opposé de la bombe. `None` : aucune bombe ne menace le joueur.
/// La navigation doit avoir été mise à jour (`update_from`).
pub fn flee_step(
    nav: &mut BotNavigation,
    bombs: &[BombView],
    position: FixedVec2,
) -> Option<FixedVec2> {
    let danger = |b: &BombView| b.radius + FLEE_MARGIN;
    let threat = bombs
        .iter()
        .filter(|b| position.distance(&b.position) < danger(b))
        .min_by_key(|b| position.distance(&b.position))?;
    let target = nav.costs_from(position).and_then(|field| {
        field
            .costs
            .iter()
            .filter(|(p, _)| {
                bombs
                    .iter()
                    .all(|b| point(**p).distance(&b.position) >= danger(b) + Fixed::from_num(8))
            })
            .min_by_key(|(p, cost)| (**cost, **p))
            .map(|(p, _)| point(*p))
    });
    let direction = target
        .and_then(|t| {
            nav.approach(position, Rect { min: t, max: t }, Fixed::ZERO)
                .map(|(direction, _)| crate::input::dead_zone(direction))
        })
        .filter(|direction| *direction != FixedVec2::ZERO)
        .unwrap_or_else(|| nav.safe_direction(position, position - threat.position));
    Some(direction)
}

/// Blank : une balle ennemie de la vue approche à moins de [`BLANK_DISTANCE`], le joueur a un
/// blank et ne tenait pas Blank à la frame précédente (front montant du jeu).
pub fn should_blank(view: &BotView, holder: &Holder) -> bool {
    holder.blanks > 0
        && !holder.blank_held
        && view.projectiles.iter().any(|p| {
            p.distance < BLANK_DISTANCE
                && (p.position - view.position).dot(&p.velocity) < Fixed::ZERO
        })
}

/// Décision de l'`explorateur` : fuir une bombe posée (prioritaire) ; sinon `prudent` dès qu'il y a
/// à se battre, esquiver, relever ou ramasser au passage ; sinon le but d'exploration (pilotage en
/// 8 directions, Interaction ou Bomb à l'arrivée), ou `prudent` (portail, attente) sans but. Blank
/// en plus.
pub fn decide_explorer(view: &BotView) -> BoxInput {
    let explore = view.explore.unwrap_or_default();
    if let Some(direction) = explore.flee {
        let mut input = BoxInput::default();
        crate::decide::steer(&mut input, direction);
        if explore.blank {
            input.buttons |= INPUT_BLANK;
        }
        return input;
    }
    let busy = view.nearest_enemy.is_some()
        || view.revive.is_some()
        || view.loot.is_some()
        || crate::dodge::dodge(view).is_some();
    let mut input = match explore.direction {
        Some(direction) if !busy => {
            let mut input = BoxInput::default();
            crate::decide::steer(&mut input, direction);
            if explore.interact {
                input.buttons |= INPUT_INTERACTION;
            }
            if explore.bomb {
                input.buttons |= INPUT_BOMB;
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
    use game::character::enemy::ai::navigation::AgentBody;

    fn v(x: f32, y: f32) -> FixedVec2 {
        FixedVec2::new(fx(x), fx(y))
    }

    /// Champ de coûts : distance de Manhattan × 10 depuis la case (0, 0), sur [-80, 80]².
    fn field() -> FlowField {
        let mut field = FlowField::default();
        for x in -80..=80 {
            for y in -80..=80 {
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
        }
    }

    fn ctx() -> GoalContext {
        GoalContext::default()
    }

    fn key_of(goal: &(GoalKey, Target)) -> &GoalKey {
        &goal.0
    }

    fn item_id(goal: &(GoalKey, Target)) -> Option<usize> {
        match &goal.1 {
            Target::Item(item) => Some(item.net_id),
            _ => None,
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
        let goals = ordered_goals(&field(), &frame, &ctx());
        let ids: Vec<_> = goals.iter().map(|g| key_of(g).3.clone()).collect();
        assert_eq!(
            ids,
            vec!["b", "a"],
            "le boss (pourtant le plus proche) attend"
        );
        let frame = ExploreFrame {
            rooms: vec![room("z", 50.0, true)],
            ..Default::default()
        };
        let goals = ordered_goals(&field(), &frame, &ctx());
        assert_eq!(goals.len(), 1, "plus que le boss : il devient le but");
    }

    #[test]
    fn egalite_departagee_par_identifiant() {
        // Deux salles symétriques : même coût, l'identifiant décide (quel que soit l'ordre).
        let a = RoomGoal {
            id: "a".into(),
            zone: room_zone(v(-192.0, -64.0), v(128.0, 128.0)),
            boss: false,
        };
        let b = RoomGoal {
            id: "b".into(),
            zone: room_zone(v(64.0, -64.0), v(128.0, 128.0)),
            boss: false,
        };
        for rooms in [vec![a.clone(), b.clone()], vec![b, a]] {
            let frame = ExploreFrame {
                rooms,
                ..Default::default()
            };
            let goals = ordered_goals(&field(), &frame, &ctx());
            assert_eq!(key_of(&goals[0]).0, key_of(&goals[1]).0, "même coût");
            assert_eq!(key_of(&goals[0]).3, "a");
        }
    }

    #[test]
    fn boss_direct_et_portail() {
        let frame = ExploreFrame {
            rooms: vec![room("b", 100.0, false), room("z", 300.0, true)],
            items: vec![ItemGoal::new(7, v(16.0, 0.0), false, false)],
            ..Default::default()
        };
        let direct = GoalContext {
            boss_direct: true,
            ..ctx()
        };
        let goals = ordered_goals(&field(), &frame, &direct);
        assert_eq!(goals.len(), 1);
        assert_eq!(key_of(&goals[0]).3, "z", "boss direct : seulement le boss");
        let open = GoalContext {
            portal_open: true,
            ..ctx()
        };
        let goals = ordered_goals(&field(), &frame, &open);
        assert_eq!(goals.len(), 1, "portail ouvert : seulement les objets");
        assert_eq!(item_id(&goals[0]), Some(7));
    }

    #[test]
    fn pas_d_actif_quand_on_en_tient_un() {
        let frame = ExploreFrame {
            items: vec![ItemGoal::new(3, v(16.0, 0.0), true, true)],
            ..Default::default()
        };
        let holding = GoalContext {
            holder: Holder {
                has_active: true,
                ..Default::default()
            },
            ..ctx()
        };
        assert!(ordered_goals(&field(), &frame, &holding).is_empty());
        assert_eq!(ordered_goals(&field(), &frame, &ctx()).len(), 1);
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
            blanks: 1,
            ..Default::default()
        };
        view.projectiles = vec![bullet(30.0, -4.0)];
        assert!(should_blank(&view, &holder));
        assert!(!should_blank(
            &view,
            &Holder {
                blanks: 0,
                ..holder.clone()
            }
        ));
        assert!(
            !should_blank(
                &view,
                &Holder {
                    blank_held: true,
                    ..holder.clone()
                }
            ),
            "front montant : relâcher d'abord"
        );
        view.projectiles = vec![bullet(30.0, 4.0)];
        assert!(!should_blank(&view, &holder), "s'éloigne");
        view.projectiles = vec![bullet(80.0, -4.0)];
        assert!(!should_blank(&view, &holder), "trop loin");
    }

    fn heart() -> PickupRule {
        PickupRule {
            id: "coeur".into(),
            max_stack: None,
            refuse_when_full: true,
        }
    }

    /// Un cœur au sol que la vie pleine fait refuser n'est pas un but (le bot attendrait sur le tas
    /// sans fin) ; blessé, il en est un. Un stock au plafond refuse de même.
    #[test]
    fn un_objet_refuse_n_est_pas_un_but() {
        let mut item = ItemGoal::new(5, v(16.0, 0.0), false, false);
        item.rule = Some(heart());
        let frame = ExploreFrame {
            items: vec![item.clone()],
            ..Default::default()
        };
        let holder = |health: i32| GoalContext {
            holder: Holder {
                health: Fixed::from_num(health),
                health_max: Fixed::from_num(6),
                ..Default::default()
            },
            ..ctx()
        };
        assert!(ordered_goals(&field(), &frame, &holder(6)).is_empty());
        assert_eq!(ordered_goals(&field(), &frame, &holder(4)).len(), 1);
        item.rule = Some(PickupRule {
            id: "key".into(),
            max_stack: Some(99),
            refuse_when_full: false,
        });
        let frame = ExploreFrame {
            items: vec![item],
            ..Default::default()
        };
        let mut capped = holder(4);
        capped.holder.stock.insert("key".into(), 99);
        assert!(ordered_goals(&field(), &frame, &capped).is_empty());
        capped.holder.stock.insert("key".into(), 98);
        assert_eq!(ordered_goals(&field(), &frame, &capped).len(), 1);
    }

    #[test]
    fn la_porte_a_cle_exige_les_cles() {
        let mut door = ItemGoal::new(9, v(80.0, 0.0), true, false);
        door.kind = GoalKind::Door { keys: 1 };
        door.zone = Rect {
            min: v(72.0, -16.0),
            max: v(88.0, 16.0),
        };
        door.reach = fx(24.0);
        let frame = ExploreFrame {
            items: vec![door],
            ..Default::default()
        };
        let with = |keys: u32| GoalContext {
            holder: Holder {
                keys,
                ..Default::default()
            },
            ..ctx()
        };
        assert!(ordered_goals(&field(), &frame, &with(0)).is_empty());
        assert_eq!(ordered_goals(&field(), &frame, &with(1)).len(), 1);
    }

    fn shelf(id: usize, x: f32, price: u32, heart_item: bool) -> ItemGoal {
        let mut goal = ItemGoal::new(id, v(x, 0.0), true, false);
        goal.kind = GoalKind::Shelf {
            price,
            heart: heart_item,
        };
        if heart_item {
            goal.rule = Some(heart());
        }
        goal
    }

    /// Achats : seulement ce que le joueur paie ; cœur d'abord à vie basse, sinon le moins cher.
    #[test]
    fn achats_coeur_d_abord_a_vie_basse_sinon_le_moins_cher() {
        let frame = ExploreFrame {
            items: vec![
                shelf(1, 16.0, 5, false),
                shelf(2, 160.0, 3, false),
                shelf(3, 320.0, 9, true),
                shelf(4, 64.0, 100, false),
            ],
            ..Default::default()
        };
        let ctx = |health: i32| GoalContext {
            holder: Holder {
                wallet: 10,
                health: Fixed::from_num(health),
                health_max: Fixed::from_num(6),
                ..Default::default()
            },
            policy: ExplorerPolicy {
                shop: ShopOrder::HeartsThenCheapest {
                    low_health: fx(0.5),
                },
                ..Default::default()
            },
            ..GoalContext::default()
        };
        let pick = |health| {
            let goals = ordered_goals(&field(), &frame, &ctx(health));
            assert_eq!(goals.len(), 1, "un seul présentoir à la fois");
            item_id(&goals[0]).unwrap()
        };
        assert_eq!(
            pick(3),
            3,
            "vie basse (3/6) : le cœur, pourtant le plus loin et le plus cher"
        );
        assert_eq!(
            pick(4),
            2,
            "vie correcte : le moins cher (3), pas le plus proche (5)"
        );
        assert_eq!(pick(6), 2, "vie pleine : le cœur serait refusé");
    }

    fn devil(id: usize, loss: i32) -> ItemGoal {
        let mut goal = ItemGoal::new(id, v(16.0, 0.0), true, false);
        goal.kind = GoalKind::Devil {
            loss: Fixed::from_num(loss),
        };
        goal
    }

    #[test]
    fn le_diable_suit_la_politique() {
        let holder = |health: i32, soul: i32| Holder {
            health: Fixed::from_num(health),
            health_max: Fixed::from_num(health.max(6)),
            soul: Fixed::from_num(soul),
            ..Default::default()
        };
        let above = |n: i32| DevilPolicy::IfHealthAbove(Fixed::from_num(n));
        let loss = Fixed::from_num(2);
        assert!(!devil_accepts(DevilPolicy::Never, &holder(6, 0), loss));
        assert!(devil_accepts(above(4), &holder(6, 0), loss), "6 > 4");
        assert!(
            !devil_accepts(above(4), &holder(4, 0), loss),
            "strictement au-dessus"
        );
        assert!(
            !devil_accepts(above(0), &holder(2, 0), loss),
            "le prix tuerait (2 <= 2)"
        );
        assert!(devil_accepts(above(0), &holder(3, 0), loss));
        assert!(
            devil_accepts(above(100), &holder(1, 4), loss),
            "l'âme couvre le prix : rien ne part de la vie"
        );
        assert!(
            devil_accepts(above(4), &holder(5, 1), loss),
            "l'âme couvre la moitié : il reste 1 point de vie maximale à payer, 5 > 4"
        );
        let frame = ExploreFrame {
            items: vec![devil(1, 2)],
            ..Default::default()
        };
        let with = |policy| GoalContext {
            holder: holder(6, 0),
            policy: ExplorerPolicy {
                devil: policy,
                ..Default::default()
            },
            ..ctx()
        };
        assert!(ordered_goals(&field(), &frame, &with(DevilPolicy::Never)).is_empty());
        assert_eq!(ordered_goals(&field(), &frame, &with(above(4))).len(), 1);
    }

    fn carrier(bombs: u32) -> GoalContext {
        GoalContext {
            holder: Holder {
                bombs,
                bomb: Some(BombDef {
                    fuse_frames: 60,
                    radius: fx(56.0),
                    damage: fx(4.0),
                    friendly_fire: sim_core::damage::FriendlyFire::Always,
                }),
                ..Default::default()
            },
            ..GoalContext::default()
        }
    }

    /// Un mur secret est un but quand il y a une bombe à poser (stock au-dessus de la réserve, aucune
    /// mèche en cours) ; sans porteur ni bombe : rien (inerte).
    #[test]
    fn le_mur_secret_est_un_but_avec_une_bombe() {
        let door = Rect {
            min: v(72.0, -16.0),
            max: v(88.0, 16.0),
        };
        let mut frame = ExploreFrame {
            secret_doors: vec![door],
            ..Default::default()
        };
        let goals = ordered_goals(&field(), &frame, &carrier(1));
        assert_eq!(goals.len(), 1);
        match &goals[0].1 {
            Target::Bomb { zone, reach } => {
                assert_eq!(*zone, door);
                assert_eq!(*reach, fx(48.0), "rayon 56 - 8, sans écart de collider");
            }
            other => panic!("{other:?}"),
        }
        assert!(
            ordered_goals(&field(), &frame, &carrier(0)).is_empty(),
            "sans bombe"
        );
        assert!(
            ordered_goals(&field(), &frame, &ctx()).is_empty(),
            "sans porteur"
        );
        let mut reserve = carrier(1);
        reserve.policy.bombs.reserve = 1;
        assert!(
            ordered_goals(&field(), &frame, &reserve).is_empty(),
            "stock gardé en réserve"
        );
        frame.bombs.push(BombView {
            position: v(0.0, 0.0),
            radius: fx(56.0),
        });
        assert!(
            ordered_goals(&field(), &frame, &carrier(1)).is_empty(),
            "mèche en cours"
        );
    }

    /// Un champ « sans rochers » sur une ligne y = 0 (cases 0 à 30, source en 0).
    fn line_field() -> FlowField {
        let mut field = FlowField::default();
        for x in 0..=30 {
            field.costs.insert(GridPos::new(x, 0), (x * 10) as u32);
            field
                .directions
                .insert(GridPos::new(x, 0), GridPos::new((x - 1).max(0), 0));
        }
        field
    }

    /// Le champ réel s'arrête devant les rochers (cases 0 à 9).
    fn blocked_field() -> FlowField {
        let mut field = FlowField::default();
        for x in 0..=9 {
            field.costs.insert(GridPos::new(x, 0), (x * 10) as u32);
        }
        field
    }

    /// Rochers de 16 x 16 centrés sur `(x, 0)`.
    fn rocks(xs: &[f32]) -> Vec<(usize, Rect)> {
        xs.iter()
            .enumerate()
            .map(|(i, x)| {
                (
                    i,
                    Rect {
                        min: v(*x - 8.0, -8.0),
                        max: v(*x + 8.0, 8.0),
                    },
                )
            })
            .collect()
    }

    /// Plus aucun but atteignable, mais un but derrière deux rochers : la bombe vise **le premier**
    /// rocher du chemin (le plus proche du joueur), pas le plus proche du but.
    #[test]
    fn la_bombe_vise_le_premier_rocher_du_chemin() {
        let frame = ExploreFrame {
            rooms: vec![RoomGoal {
                id: "x".into(),
                zone: Rect {
                    min: v(200.0, 0.0),
                    max: v(200.0, 0.0),
                },
                boss: false,
            }],
            // x = 84 et x = 116 : sur le chemin, au-delà de la case 9 (x = 72) que le joueur atteint.
            rocks: rocks(&[116.0, 84.0]),
            ..Default::default()
        };
        let ctx = carrier(2);
        assert!(
            ordered_goals(&blocked_field(), &frame, &ctx).is_empty(),
            "la salle n'est pas atteignable : aucun but ordinaire"
        );
        let target = rock_goal(&blocked_field(), &line_field(), &frame, &ctx, None);
        match target {
            Some(Target::Bomb { zone, .. }) => assert_eq!(zone.min, v(84.0, 0.0)),
            other => panic!("{other:?}"),
        }
        // Sans bombe, ou règle coupée : rien.
        assert!(rock_goal(&blocked_field(), &line_field(), &frame, &carrier(0), None).is_none());
        let mut off = carrier(2);
        off.policy.bombs.rocks = false;
        assert!(rock_goal(&blocked_field(), &line_field(), &frame, &off, None).is_none());
        // Un but que le champ sans rochers n'atteint pas non plus : pas de bombe pour rien.
        let far = ExploreFrame {
            rooms: vec![RoomGoal {
                id: "x".into(),
                zone: Rect {
                    min: v(900.0, 0.0),
                    max: v(900.0, 0.0),
                },
                boss: false,
            }],
            rocks: rocks(&[84.0]),
            ..Default::default()
        };
        assert!(rock_goal(&blocked_field(), &line_field(), &far, &ctx, None).is_none());
        // Un but que le champ réel atteint (ici : le même champ que sans rochers) ne vaut pas une bombe.
        assert!(
            rock_goal(&line_field(), &line_field(), &frame, &ctx, None).is_none(),
            "but atteignable : aucun rocher à faire sauter"
        );
    }

    #[test]
    fn deux_salles_se_touchent_par_un_bord() {
        let cell = v(240.0, 144.0);
        assert!(
            share_an_edge(v(0.0, 0.0), cell, v(240.0, 0.0), cell),
            "côte à côte"
        );
        assert!(
            share_an_edge(v(0.0, 0.0), cell, v(0.0, 144.0), cell),
            "l'une sur l'autre"
        );
        assert!(
            !share_an_edge(v(0.0, 0.0), cell, v(240.0, 144.0), cell),
            "en diagonale : un coin"
        );
        assert!(
            !share_an_edge(v(0.0, 0.0), cell, v(480.0, 0.0), cell),
            "séparées d'une case"
        );
        assert!(
            share_an_edge(v(0.0, 0.0), cell, v(240.0, 72.0), cell),
            "décalées : bord partiel"
        );
    }

    #[test]
    fn la_fuite_sort_du_rayon_de_la_bombe() {
        let body = AgentBody {
            left: fx(10.0),
            right: fx(10.0),
            down: fx(16.0),
            up: fx(4.0),
        };
        let mut nav = BotNavigation::default();
        nav.update_from(&[], &body, &[], v(10.0, 0.0));
        let bomb = BombView {
            position: v(0.0, 0.0),
            radius: fx(56.0),
        };
        let away = flee_step(&mut nav, &[bomb], v(10.0, 0.0)).expect("dans le rayon : il fuit");
        assert!(away.x > Fixed::ZERO, "à l'opposé de la bombe : {away:?}");
        assert!(
            flee_step(&mut nav, &[bomb], v(200.0, 0.0)).is_none(),
            "hors du rayon et de la marge : rien"
        );
        assert!(
            flee_step(&mut nav, &[], v(10.0, 0.0)).is_none(),
            "aucune bombe"
        );
    }
}
