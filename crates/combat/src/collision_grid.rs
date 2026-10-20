//! Candidats de collision dérivés (T2.1, chantier B4b, `docs/taches.md`) : reconstruites à
//! partir de l'état rollback à des points précis de `GgrsSchedule`, jamais lues d'une frame
//! à l'autre. [`CollisionGrids`] n'est donc **ni rollback ni checksum** (`docs/taches.md` :
//! « donnée dérivée... reconstruite à chaque frame avant usage ») : elle ne participe jamais
//! au résultat de la simulation, seulement à sa vitesse. D15 remplace les cellules spatiales
//! de T2.1 par un balayage des mêmes entrées, aux mêmes points du schedule.
//!
//! Trois instantanés (les noms de grille de T2.1 sont gardés pour les consommateurs) :
//! - [`CollisionGrids::walls`] : murs (`Collider` + `Wall` + `Rollback`) seulement. Statique
//!   au sein d'une frame (les murs ne bougent jamais) et le plus souvent d'une frame à
//!   l'autre : reconstruite seulement quand le nombre de murs *avec collider* change (une
//!   porte qui s'ouvre retire son `Collider`, voir `crate::interaction`, donc change ce
//!   compte — une porte qui se referme le pose à nouveau).
//! - [`CollisionGrids::characters`] : tout le reste des colliders rollback (joueurs, ennemis,
//!   fenêtres, obstacles cassables, personnages de laboratoire du testbed, balles, hitbox de
//!   mêlée) — jamais les murs (grille séparée). Balles et hitbox y sont : l'ancienne
//!   `collider_query` de `move_characters` ne les excluait pas non plus, et une hitbox de
//!   mêlée d'ennemi (`CollisionLayer(enemy_layer)` tant qu'elle vit) comptait déjà dans son
//!   ralentissement « poussé par un ennemi » — les exclure aurait changé ce résultat (voir la
//!   doc de [`rebuild_character_grid_pre_movement`]). Les consommateurs qui ne veulent que de
//!   vraies cibles (balles, mêlée) filtrent déjà par leur propre `Query`
//!   (`With<Health>`/`With<Team>`, absents des balles/hitbox) au `.get()` par entité.
//!   Reconstruite deux fois par frame :
//!   - [`rebuild_character_grid_pre_movement`], avant `RollbackSystemSet::Movement` : ce que
//!     voyait l'ancien code à ce point (joueurs pas encore déplacés cette frame — sans effet
//!     sur `move_characters`, qui ignore les autres joueurs ; ennemis à leur position de fin
//!     de frame précédente, puisque `move_enemies` ne tourne que bien plus tard, dans
//!     `RollbackSystemSet::EnemyAI`) ;
//!   - [`rebuild_character_grid_post_movement`], après `Movement`, avant
//!     `RollbackSystemSet::Weapon` : les balles et la mêlée doivent voir les joueurs déjà
//!     déplacés cette frame, comme le faisait l'ancien code (une requête ECS directe,
//!     exécutée après `Movement`). Rien ne bouge plus les personnages entre ce point et la
//!     fin de `Weapon` (balles, mêlée), donc cette même image sert aux deux ; elle sert aussi,
//!     plus tard, à `move_enemies` (`RollbackSystemSet::EnemyAI`) pour la partie « fenêtres »
//!     seulement (positions de fenêtres inchangées depuis, filtrées par son propre `Query::get`
//!     qui rejette tout ce qui n'est pas une fenêtre — les entrées joueurs/ennemis, elles,
//!     périmées à ce point, ne sont jamais lues pour cet usage).
//! - [`CollisionGrids::targets`] (D97) : `characters` **sans les balles**, même ordre. Les balles
//!   et les projectiles composables ne testent que de vraies cibles (leur `Query` : `Without<Bullet>`,
//!   `Team`, `Health`) : chaque balle balayait aussi toutes les autres (432 au pic de
//!   `bench_tears`), soit balles × entités AABB par frame. Reconstruite avec `characters`
//!   (même passage, un seul tri).
//!
//! D15 : les instantanés sont de simples listes d'AABB, parcourues en balayage. Les entrées
//! gardent le `GgrsNetId.0` seul (le nom du `GgrsNetId` n'y sert jamais : le cloner allouait une
//! `String` par entité et par reconstruction) et sont **toujours triées par `net_id`** : les
//! requêtes gardent exactement le filtrage inclusif et l'ordre de T2.1 sans retrier ni allouer
//! (`iter_overlapping`). La grille de séparation des ennemis (`pathing`) reste indépendante de
//! ces collisions.

use crate::grid::Aabb;
use crate::weapons::Bullet;
use bevy::prelude::*;
use bevy_fixed::fixed_math::{self, FixedVec2, FixedVec3};
use bevy_ggrs::Rollback;
use utils::{net_id::GgrsNetId, order_iter};

use crate::collider::{Collider, ColliderShape, Wall};

/// Ressource dérivée (non rollback, non checksum) : voir la doc du module.
#[derive(Resource)]
pub struct CollisionGrids {
    pub walls: CollisionCandidates,
    /// Nombre de murs (`With<Wall>, With<Collider>, With<Rollback>`) et somme de leurs
    /// `GgrsNetId` vus à la dernière reconstruction de `walls` ; sert seulement à détecter un
    /// changement (porte ouverte/fermée, mur ajouté, et T1.8 : murs d'un niveau remplacés
    /// par ceux du suivant, souvent en même nombre — le nombre seul ne suffit plus), voir
    /// [`maybe_rebuild_wall_grid`].
    walls_signature: (usize, usize),
    pub characters: CollisionCandidates,
    /// `characters` sans les balles : les cibles que les balles et les projectiles peuvent
    /// toucher (leur propre `Query` exige `Without<Bullet>`, `Team` et `Health`). Une balle qui
    /// teste ses cibles ne balaie plus les centaines d'autres balles (D97, `docs/taches.md`).
    pub targets: CollisionCandidates,
}

/// Entrée d'un instantané : l'entité, son `GgrsNetId.0` (le nom du `GgrsNetId` ne sert jamais
/// ici, le cloner à chaque reconstruction allouait une `String` par entité et par frame) et son
/// AABB.
#[derive(Clone, Copy, Debug)]
pub struct Candidate {
    pub net_id: usize,
    pub entity: Entity,
    pub aabb: Aabb,
    bullet: bool,
}

/// Instantané des candidats : balayage des AABB, **toujours triés par `net_id`** (l'insertion
/// garde l'ordre, les reconstructions de frame trient une fois).
/// D15 : aucun découpage en cellules ni insertion dans un arbre pour les collisions.
#[derive(Default)]
pub struct CollisionCandidates {
    entries: Vec<Candidate>,
}

impl CollisionCandidates {
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Insère à sa place dans l'ordre des `net_id` (après les entrées de même `net_id`).
    pub fn insert(&mut self, net_id: usize, entity: Entity, aabb: Aabb) {
        let at = self.entries.partition_point(|entry| entry.net_id <= net_id);
        self.entries.insert(
            at,
            Candidate {
                net_id,
                entity,
                aabb,
                bullet: false,
            },
        );
    }

    /// Candidats qui chevauchent `aabb`, par `net_id` croissant, sans allocation. Chaque entrée
    /// apparaît une fois, y compris sur les bords inclus des AABB.
    pub fn iter_overlapping(&self, aabb: &Aabb) -> impl Iterator<Item = &Candidate> {
        let aabb = *aabb;
        self.entries
            .iter()
            .filter(move |entry| entry.aabb.overlaps(&aabb))
    }

    /// [`Self::iter_overlapping`] rassemblé (pour les appelants qui gardent la liste).
    pub fn query_aabb(&self, aabb: &Aabb) -> Vec<&Candidate> {
        self.iter_overlapping(aabb).collect()
    }
}

impl CollisionGrids {
    /// Reconstruit `characters` (tout sauf les murs) et `targets` (sans les balles) depuis
    /// `items` (`net_id`, entité, AABB, est une balle) : un seul tri, réutilisant les tampons.
    fn rebuild_characters(&mut self, items: impl Iterator<Item = (usize, Entity, Aabb, bool)>) {
        let entries = &mut self.characters.entries;
        entries.clear();
        entries.extend(items.map(|(net_id, entity, aabb, bullet)| Candidate {
            net_id,
            entity,
            aabb,
            bullet,
        }));
        entries.sort_unstable_by_key(|entry| entry.net_id);
        self.targets.entries.clear();
        self.targets
            .entries
            .extend(entries.iter().filter(|entry| !entry.bullet).copied());
    }
}

impl Default for CollisionGrids {
    fn default() -> Self {
        Self {
            walls: CollisionCandidates::default(),
            walls_signature: (0, 0),
            characters: CollisionCandidates::default(),
            targets: CollisionCandidates::default(),
        }
    }
}

/// AABB d'un collider positionné (même décalage `offset` que `crate::collider::is_colliding`).
/// Sert à la fois à insérer une entrée dans une grille et à construire la boîte de requête
/// d'un objet mobile (balle, hitbox de mêlée, personnage) à une position candidate.
pub fn collider_aabb(pos: &FixedVec3, collider: &Collider) -> Aabb {
    let center = FixedVec2::new(pos.x + collider.offset.x, pos.y + collider.offset.y);
    match collider.shape {
        ColliderShape::Circle { radius } => Aabb::from_circle(center, radius),
        ColliderShape::Rectangle { width, height } => Aabb::from_center_size(center, width, height),
    }
}

/// Union de deux AABB (plus petit rectangle qui contient les deux). Sert aux balles : la
/// boîte de requête couvre l'ancienne et la nouvelle position de la frame (voir
/// `docs/taches.md` T2.1), un sur-ensemble sûr du test précis `is_colliding` qui suit
/// toujours sur la position actuelle seule (résultat inchangé, voir le rapport de la tâche).
pub fn union_aabb(a: Aabb, b: Aabb) -> Aabb {
    Aabb {
        min: FixedVec2::new(a.min.x.min(b.min.x), a.min.y.min(b.min.y)),
        max: FixedVec2::new(a.max.x.max(b.max.x), a.max.y.max(b.max.y)),
    }
}

/// Reconstruit [`CollisionGrids::walls`] seulement quand l'ensemble des murs avec collider a
/// changé depuis la dernière frame (une porte qui s'ouvre/se ferme retire/pose son
/// `Collider`, voir `crate::interaction` ; T1.8 : passage de niveau du mode `Floors`, qui
/// remplace tous les murs — signature nombre + somme des net ids, voir
/// [`CollisionGrids::walls_signature`]). `.after(RollbackSystemSet::Interaction)` : voit
/// les portes ouvertes/fermées cette frame ; `.before(RollbackSystemSet::Movement)` : prêt
/// avant son premier consommateur (`move_characters`).
pub fn maybe_rebuild_wall_grid(
    mut grids: ResMut<CollisionGrids>,
    wall_query: Query<
        (&GgrsNetId, Entity, &fixed_math::FixedTransform3D, &Collider),
        (With<Wall>, With<Rollback>),
    >,
) {
    let signature = wall_query
        .iter()
        .fold((0usize, 0usize), |(count, sum), (net_id, ..)| {
            (count + 1, sum.wrapping_add(net_id.0))
        });
    if signature == grids.walls_signature {
        return;
    }
    grids.walls_signature = signature;
    grids.walls.clear();
    for (net_id, entity, transform, collider) in order_iter!(wall_query) {
        let aabb = collider_aabb(&transform.translation, collider);
        grids.walls.insert(net_id.0, entity, aabb);
    }
}

/// Avant `RollbackSystemSet::Movement` : voir la doc du module.
///
/// Contient tout ce qui a un `Collider` sauf les murs (grille séparée) — donc, comme
/// l'ancienne `collider_query` de `move_characters`, **aussi** les balles et les hitbox de
/// mêlée encore en vie d'une frame précédente. C'est nécessaire : une hitbox de mêlée
/// d'ennemi vit `duration_frames` avec `CollisionLayer(enemy_layer)`
/// (`weapons::melee::spawn_melee_hitbox`) et comptait déjà, avant T2.1, dans le
/// ralentissement « poussé par un ennemi » de `move_characters` (`count_enemy_collisions`
/// n'a jamais filtré par le marqueur `Enemy`, seulement par `CollisionLayer`). Les
/// consommateurs qui ne veulent que les vraies cibles (balles, mêlée) le font déjà via leur
/// propre `Query` (`With<Health>`/`With<Team>`, que balles et hitbox n'ont pas) au moment du
/// `.get()` par entité — inutile de filtrer ici, voir `crates/game/src/weapons/mod.rs` et
/// `crates/game/src/weapons/melee.rs`.
pub fn rebuild_character_grid_pre_movement(
    mut grids: ResMut<CollisionGrids>,
    query: CharacterQuery,
) {
    rebuild_characters(&mut grids, &query);
}

/// Après `RollbackSystemSet::Movement`, avant `RollbackSystemSet::Weapon` : voir la doc du
/// module et de [`rebuild_character_grid_pre_movement`] (même contenu : tout sauf les murs).
pub fn rebuild_character_grid_post_movement(
    mut grids: ResMut<CollisionGrids>,
    query: CharacterQuery,
) {
    rebuild_characters(&mut grids, &query);
}

type CharacterQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static GgrsNetId,
        Entity,
        &'static fixed_math::FixedTransform3D,
        &'static Collider,
        Has<Bullet>,
    ),
    (Without<Wall>, With<Rollback>),
>;

fn rebuild_characters(grids: &mut CollisionGrids, query: &CharacterQuery) {
    grids.rebuild_characters(
        query
            .iter()
            .map(|(net_id, entity, transform, collider, bullet)| {
                (
                    net_id.0,
                    entity,
                    collider_aabb(&transform.translation, collider),
                    bullet,
                )
            }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fixed::fixed_math::Fixed;

    /// D15 : mêmes candidats ET même ordre que les anciennes cellules, y compris aux bords,
    /// aux coordonnées négatives et pour une entrée couvrant plusieurs cellules.
    #[test]
    fn scan_matches_spatial_grid_in_order() {
        let mut scan = CollisionCandidates::default();
        let mut grid = crate::grid::SpatialGrid::new(Fixed::from_num(64));
        for i in (0..50u32).rev() {
            let center = FixedVec2::new(Fixed::from_num(i * 16), Fixed::from_num(-128));
            let aabb = Aabb::from_center_size(center, Fixed::from_num(128), Fixed::from_num(256));
            let net_id = GgrsNetId(i as usize, "candidate".into());
            let entity = Entity::from_raw_u32(i).unwrap();
            grid.insert(net_id.clone(), entity, aabb);
            scan.insert(net_id.0, entity, aabb);
        }
        for x in -5..55 {
            let center = FixedVec2::new(Fixed::from_num(x * 16), Fixed::ZERO);
            let query = Aabb::from_circle(center, Fixed::from_num(16));
            let scanned: Vec<_> = scan
                .query_aabb(&query)
                .into_iter()
                .map(|e| (e.net_id, e.entity))
                .collect();
            let swept: Vec<_> = grid
                .query_aabb(&query)
                .into_iter()
                .map(|e| (e.net_id.0, e.entity))
                .collect();
            assert_eq!(scanned, swept);
        }
        scan.clear();
        assert!(scan
            .query_aabb(&Aabb::from_circle(FixedVec2::ZERO, Fixed::ONE))
            .is_empty());
    }

    /// D97 : `targets` est `characters` sans les balles, dans le même ordre (`net_id`), quel que
    /// soit l'ordre d'itération de la requête.
    #[test]
    fn targets_are_characters_without_bullets_in_net_id_order() {
        let aabb = Aabb::from_circle(FixedVec2::ZERO, Fixed::from_num(8));
        let item = |net_id: usize, bullet: bool| {
            (
                net_id,
                Entity::from_raw_u32(net_id as u32).unwrap(),
                aabb,
                bullet,
            )
        };
        let mut grids = CollisionGrids::default();
        grids.rebuild_characters(
            [
                item(9, false),
                item(3, true),
                item(7, false),
                item(1, true),
                item(5, false),
            ]
            .into_iter(),
        );
        let ids = |candidates: &CollisionCandidates| {
            candidates
                .query_aabb(&aabb)
                .into_iter()
                .map(|e| e.net_id)
                .collect::<Vec<_>>()
        };
        assert_eq!(ids(&grids.characters), [1, 3, 5, 7, 9]);
        assert_eq!(ids(&grids.targets), [5, 7, 9]);
        // une reconstruction repart de zéro
        grids.rebuild_characters([item(2, false)].into_iter());
        assert_eq!(ids(&grids.characters), [2]);
        assert_eq!(ids(&grids.targets), [2]);
    }
}
