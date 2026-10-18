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
//! [`ExplorerVisits`] et, si `Oscillation::Hysteresis`, [`ExplorerJitter`] (position d'attache et but
//! suivi, pour sortir d'un balancement). « Salle visitée » se lit dans l'état : une salle verrouillante `Cleared`, ou
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
use map_ldtk::game::rooms::{door_near, key_lock_cost, sorted_rooms};
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

/// Marge de la bombe sous son rayon : l'explosion doit toucher **strictement** (`world::explosion`),
/// et le bot mesure depuis son centre, pas depuis le point de pose.
const BOMB_STAND_MARGIN: Fixed = Fixed::from_bits(8 << 16);

/// Marge de fuite au-delà du rayon d'une bombe posée.
pub const FLEE_MARGIN: Fixed = Fixed::from_bits(16 << 16);

/// Une porte à clé s'approche à la portée d'interaction moins cette marge (le jeu mesure du bord du collider).
const DOOR_REACH_INSET: Fixed = Fixed::from_bits(8 << 16);

/// Portée de pose minimale d'une bombe : sous cette distance au mur, le bot se tient déjà sur la bombe.
const BOMB_MIN_REACH: Fixed = Fixed::from_bits(4 << 16);

/// Un poseur de bombe plus proche d'un mur que la portée de pose plus cette marge est censé s'en charger (règle des pairs).
const PEER_COVER_MARGIN: Fixed = Fixed::from_bits(40 << 16);

/// Une case de fuite est sûre à cette distance au-delà du danger (`danger(bombe)`) de toute bombe posée.
const FLEE_CLEARANCE: Fixed = Fixed::from_bits(8 << 16);

/// Frames sans appel au-delà desquelles la tenue d'une réanimation recommence son cycle.
const SNAP_GAP_FRAMES: u32 = 2;

/// Frames sans pas au-delà desquelles une poursuite recommence.
const PURSUIT_GAP_FRAMES: u32 = 5;

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

/// Phase de patience d'un `explorateur` (m2-salles-murs-gungeon, D82) : le pilotage touche par touche,
/// décalé de 5 frames et avec de l'élan, peut tourner en cycle limite devant une ouverture de 32 px
/// (corps de 20 px) quel que soit le gain de correction (essais : avance de 5 frames, de 8 frames : chacun
/// débloque certaines graines et en bloque d'autres). Un bot resté à moins de [`PATIENCE_RADIUS`] de son
/// ancre pendant [`PATIENCE_FRAMES`] frames sans ennemi ni butin change de manière : **Settle** (aucune
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
const PATIENCE_RADIUS: Fixed = Fixed::from_bits(40 << 16);
/// Frames sans quitter l'ancre avant la première phase de repli.
const PATIENCE_FRAMES: u32 = 100;
/// Durée de l'arrêt (frames), puis du cap droit ; la séquence dure [`PATIENCE_CYCLE`].
const SETTLE_FRAMES: u32 = 15;
const PATIENCE_CYCLE: u32 = 60;
/// Écart transverse toléré pendant le cap droit (la marge d'une ouverture de 32 px pour un corps de 20 px).
const GO_TRANSVERSE: Fixed = Fixed::from_bits(6 << 16);

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
    /// Phase de patience (voir [`Patience`]).
    pub patience: Patience,
    /// [`ExplorerPolicy::lead`] : anticipation d'élan (`lead`) active.
    pub lead: bool,
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
    /// La salle se verrouille (combat, arène, boss) : seule une salle verrouillante fait tuer,
    /// donc avancer l'étage. Une autre (départ, récompense, boutique) ne vaut que le détour.
    pub locking: bool,
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
    /// La règle de `items::ItemDef::accepts` pour un consommable (`items::consumable_accepts`).
    pub fn accepts(&self, stock: u32, at_full_health: bool) -> bool {
        items::consumable_accepts(self.max_stack, self.refuse_when_full, stock, at_full_health)
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
    /// Coût de chemin au-delà duquel l'objet n'est plus un but (sous au sol : `Detour::shells`).
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
    synergy_table: Option<Res<'w, items::SynergyTable>>,
    enemy_health: Query<'w, 's, &'static Health, (With<Enemy>, With<Rollback>)>,
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
                        locking,
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
        let mut weapon_shelves = Vec::new();
        if policy.shopping {
            if matches!(policy.shop, ShopOrder::Nearest) {
                self.shopping_goals(&mut items, &mut weapon_shelves);
            } else {
                self.per_player_shopping_goals(&mut items, &mut weapon_shelves);
            }
        }
        if let Some(limit) = policy.detour.shells {
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
            .filter_map(|(_, inventory, ..)| inventory)
            .map(|inventory| inventory.consumable(game::loot::KEY_ITEM))
            .max()
            .unwrap_or(0);
        for (id, t, shelf, interactable) in order_iter!(self.shelves) {
            // M3-T8 (§73) : un présentoir à `cost` (marché du diable : vie maximale) n'est pas un achat
            // en douilles, son `price` vaut 0 ; sans politique d'isaac le bot ne le vise pas (T14 : `DevilPolicy`).
            if interactable && !shelf.sold && shelf.cost.is_none() && shelf.price <= wallet {
                items.push(ItemGoal::new(id.0, t.translation.truncate(), true, false));
                if let Some(weapon) = items::loot_weapon(&shelf.item_id) {
                    weapon_shelves.push((id.0, weapon.to_string()));
                }
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
    fn per_player_shopping_goals(
        &self,
        items: &mut Vec<ItemGoal>,
        weapon_shelves: &mut Vec<(usize, String)>,
    ) {
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
            if let Some(weapon) = items::loot_weapon(&shelf.item_id) {
                weapon_shelves.push((id.0, weapon.to_string()));
            }
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
        // Même ordre que `key_door_system` (`sorted_rooms` : position puis identifiant), trié une fois : la première
        // salle à clé dont la porte est proche est celle dont le jeu lit le prix.
        let rooms = sorted_rooms(&self.rooms);
        for (id, t, collider, interactable) in order_iter!(self.doors) {
            let Some(interactable) =
                interactable.filter(|i| i.interaction_type == InteractionType::KeyDoor)
            else {
                continue;
            };
            let p = t.translation.truncate();
            let Some((level, keys)) = rooms.iter().find_map(|(level, bounds, kind)| {
                let keys = key_lock_cost(table, kind, floor)?;
                door_near(bounds, p).then(|| (level.0.clone(), keys))
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
            goal.reach = (interactable.interaction_range - DOOR_REACH_INSET).max(Fixed::ZERO);
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
                if door_near(bounds, p) {
                    out.push(Rect::collider(p, collider));
                }
            }
        }
        // Ordre indépendant de celui de la requête des salles : (position, taille) croissantes.
        out.sort_by_key(|r| (r.min.x, r.min.y, r.max.x, r.max.y));
        out.dedup();
        out
    }

    /// Index de l'étage courant (`0` sans `FloorState`).
    pub fn floor_index(&self) -> u32 {
        self.floor.as_deref().map_or(0, |floor| floor.index)
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

    /// Empreinte de la vie de tous les ennemis (nombre et somme des vies courantes) : elle change à chaque coup au
    /// but, mort ou renfort (voir [`ExplorerJitter::engage`]).
    pub fn enemy_progress(&self) -> u64 {
        let (count, sum) = self
            .enemy_health
            .iter()
            .fold((0_u64, 0_u64), |(count, sum), health| {
                (
                    count + 1,
                    sum.wrapping_add(health.current.to_bits() as i64 as u64),
                )
            });
        (count << 48) ^ sum
    }

    /// Rayon de l'orbite des larmes du joueur `handle` (`Orbit` d'un objet ou d'une synergie tenus, composés comme au
    /// tir : `combat::projectile::compose`), `None` sans orbite.
    pub fn orbit_reach(&self, handle: usize) -> Option<Fixed> {
        let (items, synergies) = (self.item_table.as_deref()?, self.synergy_table.as_deref()?);
        let (_, inventory, ..) = self.holders.iter().find(|(p, ..)| p.handle == handle)?;
        let sources = items::projectile_sources(items, synergies, inventory?);
        combat::projectile::compose(&[], &sources)
            .orbit
            .map(|orbit| orbit.radius)
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
    /// Borne de détour des buts facultatifs de cette frame (`Detour::optional`, ou `Detour::last_stretch` quand plus
    /// aucune salle de combat ne reste ; objets, présentoirs, coffres et salles sans verrou). `None` : aucune.
    fn detour(&self, frame: &ExploreFrame) -> Option<u32> {
        let detour = self.policy.detour;
        if frame.rooms.iter().any(|room| room.locking && !room.boss) {
            detour.optional
        } else {
            detour.last_stretch.or(detour.optional)
        }
    }

    /// Portée de pose d'une bombe : l'explosion touche strictement à moins de son rayon.
    fn bomb_reach(&self) -> Option<Fixed> {
        let def = self.holder.bomb?;
        Some((def.radius - BOMB_STAND_MARGIN - self.holder.bomb_inset).max(BOMB_MIN_REACH))
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
            .any(|peer| zone.distance(*peer) <= reach + PEER_COVER_MARGIN)
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
    // Le jeu paie l'âme d'abord, refuse un reste qui tuerait ou dépasse la vie maximale : `Price::can_pay` (le même appel
    // que `game::shop`). Le seuil de la politique ne s'ajoute que si de la vie maximale part.
    let rest = (loss - holder.soul).max(Fixed::ZERO);
    Price::MaxHealth(loss).can_pay(
        0,
        &Inventory::default(),
        holder.health_max,
        holder.health,
        holder.soul,
    ) && (rest == Fixed::ZERO || holder.health > threshold)
}

/// Buts d'exploration triés par [`GoalKey`] (seulement ceux qu'un chemin atteint).
pub fn ordered_goals(
    field: &FlowField,
    frame: &ExploreFrame,
    ctx: &GoalContext,
) -> Vec<(GoalKey, Target)> {
    let mut goals: Vec<(GoalKey, Target)> = Vec::new();
    let detour = ctx.detour(frame);
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
            if item.max_cost.is_some_and(|limit| cost > limit)
                || detour.is_some_and(|limit| cost > limit)
            {
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
            // Une salle sans verrou n'est qu'un détour (voir `Detour::optional`) : lointaine, elle ne retarde ni les
            // combats ni le boss.
            if !room.locking && !room.boss && detour.is_some_and(|limit| cost > limit) {
                continue;
            }
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
    // Seulement un but que le champ réel n'atteint pas : un but atteignable (détour, sous au-delà de `Detour::shells`)
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
/// Un point de passage n'est tenu que ce nombre de frames : un bot qui ne peut pas l'atteindre (coin, corps que la
/// navigation ne voit pas) reprend la route, puis s'attache de nouveau après [`JITTER_FRAMES`].
pub const JITTER_HOLD_FRAMES: u32 = 60;

/// État anti-jitter d'un joueur.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jitter {
    anchor: FixedVec2,
    since: u32,
    /// But auquel le bot s'est attaché (`Step::target`).
    focus: Option<u64>,
    hold: Option<(u64, FixedVec2)>,
    /// Frame où le point de passage tenu a été pris.
    held_since: u32,
}

/// Un combat sans **progrès** pendant [`STALL_FRAMES`] frames est un blocage : les larmes s'arrêtent sur un coin de mur ou un
/// pilier, le bot tire sans fin vers un ennemi qui ne perd rien (graines 63, 86, 178 et 182 d'`isaac`, T11c). Le progrès
/// est l'empreinte de la vie des ennemis ([`ExplorerWorld::enemy_progress`]) : un coup au but, une mort ou un renfort
/// la change.
pub const STALL_FRAMES: u32 = 240;
/// Distance à laquelle le bot s'approche de l'ennemi d'un combat bloqué, par le chemin (presque au contact : la ligne
/// de tir change tant que le coin de mur n'est pas dépassé).
pub const ENGAGE_REACH: Fixed = Fixed::from_bits(8 << 16);
/// Frames d'approche avant de contourner l'ennemi, puis durée de chaque sens du contournement.
pub const STRAFE_FRAMES: u32 = 90;

/// État d'un combat suivi : l'empreinte de la vie des ennemis, depuis quand elle ne bouge plus, et depuis quand le bot
/// s'est engagé (blocage constaté).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stall {
    progress: u64,
    since: u32,
    engaged_at: Option<u32>,
}

/// Un joueur qui marche vers un but (hors salle) sans quitter un rayon de [`GIVE_UP_RADIUS`] pendant
/// [`GIVE_UP_FRAMES`] frames est contre un obstacle que la navigation ne voit pas (un tas de sous contre un mur qu'elle
/// croit libre : graine 103 d'`isaac` à deux bots, 600 frames de glissement le long d'un mur, deux buts alternés) : son
/// but du moment est **écarté** pour ce joueur. Les salles ne le sont jamais.
pub const GIVE_UP_FRAMES: u32 = 300;
/// Rayon de la poursuite sur place ([`GIVE_UP_FRAMES`]).
pub const GIVE_UP_RADIUS: Fixed = Fixed::from_bits(24 << 16);

/// Poursuite en cours : où le joueur est ancré, depuis quand, et la dernière frame vue.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pursuit {
    anchor: FixedVec2,
    since: u32,
    last: u32,
}

/// Réanimation tenue à portée sans bouger : au plus ce nombre de frames d'affilée (la réanimation du jeu dure moins), puis
/// [`SNAP_RELEASE_FRAMES`] frames où le pas d'approche reprend la main.
pub const SNAP_HOLD_FRAMES: u32 = 240;
/// Voir [`SNAP_HOLD_FRAMES`].
pub const SNAP_RELEASE_FRAMES: u32 = 60;

/// Anti-jitter des `explorateur` (`Oscillation::Hysteresis`) : **seconde mémoire du bot** après [`ExplorerVisits`],
/// hors simulation et hors rollback pour la même raison (elle ne sert qu'à produire des inputs que GGRS enregistre).
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct ExplorerJitter {
    /// Balancements de l'exploration, par joueur.
    pub sway: BTreeMap<usize, Jitter>,
    /// Blocages de combat ([`Self::engage`]), par joueur.
    pub stalls: BTreeMap<usize, Stall>,
    /// Poursuites en cours ([`Self::pursue`]), par joueur.
    pub pursuits: BTreeMap<usize, Pursuit>,
    /// Buts écartés `(joueur, identité du but)`.
    pub banned_goals: BTreeSet<(usize, u64)>,
    /// Tenue d'une réanimation à portée `(depuis, dernière frame)` ([`Self::snap_hold`]), par joueur.
    pub snaps: BTreeMap<usize, (u32, u32)>,
    /// Ancre `(position, frame d'arrivée)` par joueur ([`Self::patience`]).
    pub anchors: BTreeMap<usize, (FixedVec2, u32)>,
    /// Index de l'étage où cette mémoire a été constituée ([`Self::enter_floor`]).
    pub floor: Option<u32>,
}

impl ExplorerJitter {
    /// Phase de patience ([`Oscillation::Patience`]) du joueur `handle` à la frame `frame` ; `has_goal` : un cap d'exploration existe (sinon l'ancre
    /// est oubliée : attendre sans but n'est pas bloqué).
    pub fn patience(
        &mut self,
        handle: usize,
        frame: u32,
        position: FixedVec2,
        has_goal: bool,
    ) -> Patience {
        let entry = self.anchors.entry(handle).or_insert((position, frame));
        if !has_goal || entry.0.distance(&position) > PATIENCE_RADIUS {
            *entry = (position, frame);
            return Patience::Normal;
        }
        let stalled = frame.saturating_sub(entry.1);
        if stalled < PATIENCE_FRAMES {
            return Patience::Normal;
        }
        if (stalled - PATIENCE_FRAMES) % PATIENCE_CYCLE < SETTLE_FRAMES {
            Patience::Settle
        } else {
            Patience::Go
        }
    }

    /// But auquel le joueur `handle` s'est attaché (`explore_step` le passe en premier).
    pub fn focus(&self, handle: usize) -> Option<u64> {
        self.sway.get(&handle).and_then(|state| state.focus)
    }

    /// Entre dans l'étage `floor` : à un changement d'étage, les buts écartés et les poursuites sont oubliés (l'identité
    /// d'un mur ou d'une porte est sa zone : elle peut revenir à l'identique à l'étage suivant).
    pub fn enter_floor(&mut self, floor: u32) {
        if self.floor != Some(floor) {
            self.banned_goals.clear();
            self.pursuits.clear();
            self.stalls.clear();
            self.snaps.clear();
            self.floor = Some(floor);
        }
    }

    /// Buts écartés du joueur `handle`.
    pub fn banned(&self, handle: usize) -> BTreeSet<u64> {
        self.banned_goals
            .range((handle, 0)..=(handle, u64::MAX))
            .map(|(_, who)| *who)
            .collect()
    }

    /// Le joueur `handle` peut-il se figer à portée de la surface de réanimation à la frame `frame` ([`SNAP_HOLD_FRAMES`]
    /// frames de tenue, puis [`SNAP_RELEASE_FRAMES`] d'approche) ? La portée du jeu se mesure au cercle du collider, la
    /// navigation à sa boîte : si le jeu refuse l'Interaction à la limite, l'approche reprend la main au lieu de laisser le bot
    /// figé. Appelé à chaque frame où une surface de réanimation existe ; un trou de plus de 2 frames recommence le cycle.
    pub fn snap_hold(&mut self, handle: usize, frame: u32) -> bool {
        let state = self.snaps.entry(handle).or_insert((frame, frame));
        if frame.saturating_sub(state.1) > SNAP_GAP_FRAMES {
            *state = (frame, frame);
        }
        state.1 = frame;
        frame.saturating_sub(state.0) % (SNAP_HOLD_FRAMES + SNAP_RELEASE_FRAMES) < SNAP_HOLD_FRAMES
    }

    /// Note le pas `step` du joueur `handle` à `position`, frame `frame` : [`GIVE_UP_FRAMES`] frames sans quitter
    /// [`GIVE_UP_RADIUS`] (sans trou de plus de 5 frames) en marchant vers un but qui n'est pas une salle écartent ce but.
    pub fn pursue(&mut self, handle: usize, frame: u32, position: FixedVec2, step: &Step) {
        let fresh = Pursuit {
            anchor: position,
            since: frame,
            last: frame,
        };
        let state = self.pursuits.entry(handle).or_insert(fresh);
        if frame.saturating_sub(state.last) > PURSUIT_GAP_FRAMES
            || position.distance(&state.anchor) > GIVE_UP_RADIUS
        {
            *state = fresh;
        }
        state.last = frame;
        if frame.saturating_sub(state.since) >= GIVE_UP_FRAMES {
            if step.what != "room" {
                self.banned_goals.insert((handle, step.target));
            }
            state.since = frame;
        }
    }

    /// Combat du joueur `handle` (`in_combat` : un ennemi éveillé en vue ; `progress` : empreinte de la vie des
    /// ennemis) : `Some((ENGAGE_REACH, contournement))` quand l'empreinte ne bouge plus depuis [`STALL_FRAMES`] frames ;
    /// après [`STRAFE_FRAMES`] d'approche sans résultat il contourne l'ennemi (sens alterné toutes les
    /// [`STRAFE_FRAMES`]). Levé dès que l'empreinte change (un coup au but) ou à la fin du combat.
    pub fn engage(
        &mut self,
        handle: usize,
        frame: u32,
        progress: u64,
        in_combat: bool,
    ) -> Option<(Fixed, i8)> {
        if !in_combat {
            self.stalls.remove(&handle);
            return None;
        }
        let fresh = Stall {
            progress,
            since: frame,
            engaged_at: None,
        };
        let state = self.stalls.entry(handle).or_insert(fresh);
        if state.progress != progress {
            *state = fresh;
        }
        if state.engaged_at.is_none() && frame.saturating_sub(state.since) >= STALL_FRAMES {
            state.engaged_at = Some(frame);
        }
        let engaged_for = frame.saturating_sub(state.engaged_at?);
        let strafe = match engaged_for / STRAFE_FRAMES {
            0 => 0,
            turn if turn % 2 == 1 => 1,
            _ => -1,
        };
        Some((ENGAGE_REACH, strafe))
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
            self.sway.remove(&handle);
            return;
        }
        let fresh = Jitter {
            anchor: position,
            since: frame,
            focus: None,
            hold: None,
            held_since: frame,
        };
        let state = self.sway.entry(handle).or_insert(fresh);
        if position.distance(&state.anchor) > JITTER_RADIUS {
            *state = fresh;
        }
        if let Some((target, waypoint)) = state.hold {
            if target == step.target
                && frame.saturating_sub(state.held_since) < JITTER_HOLD_FRAMES
                && position.distance(&waypoint) > WAYPOINT_REACHED
                && clear(position, waypoint)
            {
                step.direction = crate::input::dead_zone(waypoint - position);
                return;
            }
            state.hold = None;
            state.since = frame;
        }
        if frame.saturating_sub(state.since) >= JITTER_FRAMES {
            state.focus = Some(step.target);
            state.hold = Some((step.target, position + step.direction));
            state.held_since = frame;
        }
    }
}

impl Target {
    /// Identité du but (voir [`Step::target`]).
    fn identity(&self) -> u64 {
        // Hachage de tous les bits (FNV-1a sur 64 bits) : deux buts distincts ne partagent pas une identité.
        let mix = |values: &[i64]| {
            values
                .iter()
                .fold(0xcbf2_9ce4_8422_2325_u64, |hash, value| {
                    (hash ^ *value as u64).wrapping_mul(0x0000_0100_0000_01b3)
                })
        };
        let rect = |tag: i64, r: &Rect| {
            mix(&[
                tag,
                r.min.x.to_bits() as i64,
                r.min.y.to_bits() as i64,
                r.max.x.to_bits() as i64,
                r.max.y.to_bits() as i64,
            ])
        };
        match self {
            Target::Room(zone) => rect(0, zone),
            Target::Bomb { zone, .. } => rect(1, zone),
            Target::Item(item) => mix(&[2, item.net_id as i64]),
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
    avoid: &BTreeSet<u64>,
) -> Option<Step> {
    let goals: Vec<Target> = {
        let field = nav.costs_from(position)?;
        let mut goals: Vec<Target> = ordered_goals(field, frame, ctx)
            .into_iter()
            .map(|(_, target)| target)
            .filter(|target| !avoid.contains(&target.identity()))
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
                    .all(|b| point(**p).distance(&b.position) >= danger(b) + FLEE_CLEARANCE)
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
            match explore.patience {
                Patience::Normal => {
                    let direction = if explore.lead {
                        lead(direction, view.velocity)
                    } else {
                        direction
                    };
                    crate::decide::steer(&mut input, direction);
                }
                Patience::Settle => {}
                Patience::Go => crate::decide::steer(&mut input, straight(direction)),
            }
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

    fn item(net_id: usize, x: f32) -> ItemGoal {
        ItemGoal::new(net_id, v(x, 0.0), true, false)
    }

    /// Politique de gungeon (`games/gungeon/assets/bot.ron`) : détour borné.
    fn bounded(portal_open: bool) -> GoalContext {
        GoalContext {
            portal_open,
            policy: ExplorerPolicy {
                detour: game::replay::Detour {
                    optional: Some(600),
                    last_stretch: Some(200),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..ctx()
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
        let goals = ordered_goals(&field(), &frame, &bounded(false));
        let ids: Vec<_> = goals.iter().map(|g| (key_of(g).1, key_of(g).2)).collect();
        assert_eq!(ids, vec![(0, 2), (1, 0)], "objet proche puis boss");
        // Même règle quand il reste des salles de combat, et après la mort du boss (portail).
        let frame = ExploreFrame {
            rooms: vec![room("a", 200.0, false), room("z", 1000.0, true)],
            items: vec![item(1, 800.0), item(2, 80.0)],
            ..Default::default()
        };
        let goals = ordered_goals(&field(), &frame, &bounded(false));
        assert_eq!(goals.len(), 2, "l'objet proche et la salle");
        let goals = ordered_goals(&field(), &frame, &bounded(true));
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
        let goals = ordered_goals(&field(), &with_fight, &bounded(false));
        assert_eq!(goals.len(), 2, "l'objet et la salle de combat");
        let boss_only = ExploreFrame {
            rooms: vec![room("z", 1000.0, true)],
            ..with_fight
        };
        let goals = ordered_goals(&field(), &boss_only, &bounded(false));
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
        let goals = ordered_goals(&field(), &frame, &bounded(false));
        let ids: Vec<_> = goals.iter().map(|g| key_of(g).3.clone()).collect();
        assert_eq!(ids, vec!["pres", "combat"]);
        // Plus qu'une salle sans verrou lointaine et le boss : le boss est le but.
        let frame = ExploreFrame {
            rooms: vec![passive("loin", 900.0), room("z", 600.0, true)],
            ..Default::default()
        };
        let goals = ordered_goals(&field(), &frame, &bounded(false));
        let ids: Vec<_> = goals.iter().map(|g| key_of(g).3.clone()).collect();
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
                locking: true,
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
                locking: true,
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

    fn pas(y: f32, target: u64) -> Step {
        Step {
            direction: v(0.0, y),
            interact: false,
            bomb: false,
            what: "room",
            target,
        }
    }

    /// Un bot qui veut avancer mais reste dans un rayon de 12 pendant 40 frames (deux buts dont l'ordre s'inverse) :
    /// il s'attache à son but du moment et tient son point de passage ; un vrai déplacement, une arrivée ou un autre but
    /// le libèrent. Avant 40 frames, rien n'est touché.
    #[test]
    fn l_anti_jitter_attache_le_bot_qui_balance_sur_place() {
        let mut jitter = ExplorerJitter::default();
        let libre = |_: FixedVec2, _: FixedVec2| true;
        for frame in 0..40 {
            let mut step = pas(
                if frame % 2 == 0 { 9.0 } else { -9.0 },
                u64::from(frame % 2),
            );
            let before = step;
            jitter.steady(0, frame, v(0.0, 0.0), &mut step, libre);
            assert_eq!(
                step, before,
                "frame {frame} : rien avant 40 frames sur place"
            );
            assert_eq!(jitter.focus(0), None);
        }
        let mut step = pas(9.0, 7);
        jitter.steady(0, 40, v(0.0, 0.0), &mut step, libre);
        assert_eq!(jitter.focus(0), Some(7), "s'attache au but du moment");
        // Le pas inverse que la route proposerait maintenant est remplacé par le point de passage tenu (0, 9).
        let mut step = pas(-9.0, 7);
        jitter.steady(0, 41, v(0.0, 2.0), &mut step, libre);
        assert_eq!(step.direction, v(0.0, 7.0), "vers (0, 9), pas vers le bas");
        // Segment bouché : le point de passage est lâché, le pas de la route est gardé.
        let mut step = pas(-9.0, 7);
        jitter.steady(0, 42, v(0.0, 2.0), &mut step, |_, _| false);
        assert_eq!(step.direction, v(0.0, -9.0));
        // Un vrai déplacement (plus de 12) repart de zéro.
        let mut step = pas(9.0, 7);
        jitter.steady(0, 43, v(0.0, 30.0), &mut step, libre);
        assert_eq!(jitter.focus(0), None);
        // Arrivé : l'état est oublié.
        let mut step = pas(0.0, 7);
        jitter.steady(0, 44, v(0.0, 30.0), &mut step, libre);
        assert!(jitter.sway.is_empty());
    }

    /// Un changement d'étage oublie les buts écartés (l'identité d'un mur est sa zone, qui peut revenir à l'identique) ;
    /// rester sur l'étage ne touche à rien.
    #[test]
    fn un_changement_d_etage_oublie_les_buts_ecartes() {
        let mut jitter = ExplorerJitter::default();
        jitter.enter_floor(0);
        jitter.banned_goals.insert((0, 42));
        jitter.enter_floor(0);
        assert_eq!(jitter.banned(0), BTreeSet::from([42]));
        jitter.enter_floor(1);
        assert!(jitter.banned(0).is_empty());
    }

    /// La tenue d'une réanimation à portée dure `SNAP_HOLD_FRAMES`, puis `SNAP_RELEASE_FRAMES` d'approche, puis recommence ;
    /// un trou de plus de 2 frames (autre but) recommence le cycle ; chaque joueur a son cycle.
    #[test]
    fn la_tenue_d_une_reanimation_est_bornee() {
        let mut jitter = ExplorerJitter::default();
        for frame in 100..100 + SNAP_HOLD_FRAMES {
            assert!(jitter.snap_hold(0, frame), "frame {frame}");
        }
        for frame in 100 + SNAP_HOLD_FRAMES..100 + SNAP_HOLD_FRAMES + SNAP_RELEASE_FRAMES {
            assert!(!jitter.snap_hold(0, frame), "relâche à {frame}");
        }
        assert!(
            jitter.snap_hold(0, 100 + SNAP_HOLD_FRAMES + SNAP_RELEASE_FRAMES),
            "nouveau cycle"
        );
        assert!(jitter.snap_hold(1, 5000), "un autre joueur");
        assert!(jitter.snap_hold(0, 9000), "un trou recommence le cycle");
    }

    /// Un joueur qui marche vers un but sans quitter 24 pendant 300 frames écarte son but du moment (jamais une
    /// salle) ; un joueur qui avance ou qu'un combat interrompt n'écarte rien.
    #[test]
    fn une_poursuite_sur_place_ecarte_le_but() {
        let mut jitter = ExplorerJitter::default();
        let step = Step {
            what: "shell",
            ..pas(5.0, 9)
        };
        for frame in 0..GIVE_UP_FRAMES {
            jitter.pursue(0, frame, v(0.0, (frame % 10) as f32), &step);
        }
        assert!(jitter.banned(0).is_empty());
        jitter.pursue(0, GIVE_UP_FRAMES, v(0.0, 3.0), &step);
        assert_eq!(jitter.banned(0), BTreeSet::from([9]));
        assert!(jitter.banned(1).is_empty(), "propre au joueur");
        // Une salle n'est jamais écartée.
        let mut room = Step {
            what: "room",
            ..pas(5.0, 4)
        };
        let mut other = ExplorerJitter::default();
        for frame in 0..=2 * GIVE_UP_FRAMES {
            room.target = 4;
            other.pursue(0, frame, v(0.0, 0.0), &room);
        }
        assert!(other.banned(0).is_empty());
        // Il avance : rien. Un combat (trou de plus de 5 frames) remet le compteur à zéro.
        let mut moving = ExplorerJitter::default();
        for frame in 0..4 * GIVE_UP_FRAMES {
            moving.pursue(0, frame, v(frame as f32, 0.0), &step);
        }
        assert!(moving.banned(0).is_empty());
        let mut fights = ExplorerJitter::default();
        for frame in (0..4 * GIVE_UP_FRAMES).filter(|f| f % 100 < 50) {
            fights.pursue(0, frame, v(0.0, 0.0), &step);
        }
        assert!(fights.banned(0).is_empty());
    }

    /// Un combat dont l'empreinte de vie ne bouge plus depuis 240 frames s'engage (le bot s'approche, puis contourne
    /// l'ennemi dans un sens puis dans l'autre) ; un coup au but (l'empreinte change) ou la fin du combat le lèvent.
    #[test]
    fn un_combat_bloque_s_engage_puis_se_leve() {
        let mut jitter = ExplorerJitter::default();
        for frame in 0..STALL_FRAMES {
            assert_eq!(jitter.engage(0, frame, 7, true), None, "frame {frame}");
        }
        assert_eq!(
            jitter.engage(0, STALL_FRAMES, 7, true),
            Some((ENGAGE_REACH, 0)),
            "d'abord l'approche"
        );
        assert_eq!(
            jitter.engage(0, STALL_FRAMES + STRAFE_FRAMES, 7, true),
            Some((ENGAGE_REACH, 1)),
            "puis le contournement"
        );
        assert_eq!(
            jitter.engage(0, STALL_FRAMES + 2 * STRAFE_FRAMES, 7, true),
            Some((ENGAGE_REACH, -1)),
            "dans l'autre sens"
        );
        assert_eq!(
            jitter.engage(0, STALL_FRAMES + 2 * STRAFE_FRAMES + 1, 8, true),
            None,
            "un coup au but : levé"
        );
        // Fin du combat : l'état est oublié, il faut de nouveau attendre.
        assert_eq!(jitter.engage(0, 1000, 8, false), None);
        assert_eq!(jitter.engage(0, 1001 + STALL_FRAMES, 8, true), None);
        // Un combat qui progresse (un coup au but toutes les 20 frames) ne s'engage jamais.
        let mut progressing = ExplorerJitter::default();
        for frame in 0..1200 {
            assert_eq!(
                progressing.engage(0, frame, u64::from(frame / 20), true),
                None,
                "frame {frame}"
            );
        }
    }

    /// Un point de passage que le bot n'atteint pas (corps invisible de la navigation) n'est tenu que
    /// `JITTER_HOLD_FRAMES` frames : ensuite le pas de la route reprend la main.
    #[test]
    fn la_garde_du_point_de_passage_est_bornee() {
        let mut jitter = ExplorerJitter::default();
        let libre = |_: FixedVec2, _: FixedVec2| true;
        let here = v(0.0, 0.0);
        for frame in 0..=JITTER_FRAMES {
            let mut step = pas(9.0, 5);
            jitter.steady(0, frame, here, &mut step, libre);
        }
        let mut step = pas(-9.0, 5);
        jitter.steady(0, JITTER_FRAMES + 1, here, &mut step, libre);
        assert!(step.direction.y > Fixed::ZERO, "le point de passage tient");
        let mut step = pas(-9.0, 5);
        jitter.steady(
            0,
            JITTER_FRAMES + JITTER_HOLD_FRAMES,
            here,
            &mut step,
            libre,
        );
        assert_eq!(
            step.direction,
            v(0.0, -9.0),
            "garde expirée : la route reprend"
        );
    }

    #[test]
    fn l_identite_d_un_but_distingue_salles_objets_et_bombes() {
        let zone = Rect {
            min: v(40.0, 40.0),
            max: v(88.0, 88.0),
        };
        let other = Rect {
            min: v(40.0, 128.0),
            max: v(88.0, 176.0),
        };
        let ids = [
            Target::Room(zone).identity(),
            Target::Room(other).identity(),
            Target::Bomb {
                zone,
                reach: fx(40.0),
            }
            .identity(),
            Target::Item(ItemGoal::new(7, v(0.0, 0.0), false, false)).identity(),
            Target::Item(ItemGoal::new(8, v(0.0, 0.0), false, false)).identity(),
        ];
        let distinct: std::collections::BTreeSet<_> = ids.iter().collect();
        assert_eq!(distinct.len(), ids.len(), "{ids:?}");
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
        let mut patience = ExplorerJitter::default();
        let at = |x: f32, y: f32| v(x, y);
        // En mouvement (ou sans but) : jamais de repli, l'ancre suit.
        for frame in 0..300 {
            let p = patience.patience(0, frame, at(frame as f32 * 3.0, 0.0), true);
            assert_eq!(p, Patience::Normal);
        }
        // Oscille sur place (±12 px) avec un but : Normal jusqu'à PATIENCE_FRAMES, puis arrêt, puis cap droit.
        let mut phases = Vec::new();
        for frame in 1000..1000 + PATIENCE_FRAMES + PATIENCE_CYCLE + 5 {
            let wobble = if frame % 20 < 10 { 12.0 } else { -12.0 };
            phases.push(patience.patience(0, frame, at(0.0, wobble), true));
        }
        let first = |phase| phases.iter().position(|p| *p == phase).unwrap() as u32;
        assert_eq!(first(Patience::Settle), PATIENCE_FRAMES);
        assert_eq!(first(Patience::Go), PATIENCE_FRAMES + SETTLE_FRAMES);
        // La séquence recommence (arrêt puis cap droit) tant que le bot ne bouge pas.
        assert_eq!(
            phases[(PATIENCE_FRAMES + PATIENCE_CYCLE) as usize],
            Patience::Settle
        );
        // Sans but, l'ancre est oubliée ; un autre joueur a sa propre ancre.
        assert_eq!(
            patience.patience(0, 2000, at(0.0, 0.0), false),
            Patience::Normal
        );
        assert_eq!(
            patience.patience(1, 2000, at(0.0, 0.0), true),
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
