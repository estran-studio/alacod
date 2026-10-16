//! Candidats de collision dérivés (T2.1, chantier B4b, `docs/taches.md`) : reconstruites à
//! partir de l'état rollback à des points précis de `GgrsSchedule`, jamais lues d'une frame
//! à l'autre. [`CollisionGrids`] n'est donc **ni rollback ni checksum** (`docs/taches.md` :
//! « donnée dérivée... reconstruite à chaque frame avant usage ») : elle ne participe jamais
//! au résultat de la simulation, seulement à sa vitesse. D15 remplace les cellules spatiales
//! de T2.1 par un balayage des mêmes entrées, aux mêmes points du schedule.
//!
//! Deux instantanés (les noms de grille de T2.1 sont gardés pour les consommateurs) :
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
//!
//! D15 : les deux instantanés sont de simples listes d'AABB, parcourues en balayage.
//! Les requêtes gardent exactement le filtrage inclusif et le tri par `net_id` de T2.1.
//! La grille de séparation des ennemis (`pathing`) reste indépendante de ces collisions.

use crate::grid::{Aabb, GridEntry};
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
}

/// Instantané des candidats : balayage des AABB, puis tri par `net_id`.
/// D15 : aucun découpage en cellules ni insertion dans un arbre pour les collisions.
#[derive(Default)]
pub struct CollisionCandidates {
    entries: Vec<GridEntry>,
}

impl CollisionCandidates {
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn insert(&mut self, net_id: GgrsNetId, entity: Entity, aabb: Aabb) {
        self.entries.push(GridEntry {
            net_id,
            entity,
            aabb,
        });
    }

    /// Chaque entrée apparaît une fois, y compris sur les bords inclus des AABB.
    pub fn query_aabb(&self, aabb: &Aabb) -> Vec<&GridEntry> {
        let mut result: Vec<_> = self
            .entries
            .iter()
            .filter(|entry| entry.aabb.overlaps(aabb))
            .collect();
        result.sort_by_key(|entry| entry.net_id.0);
        result
    }
}

impl Default for CollisionGrids {
    fn default() -> Self {
        Self {
            walls: CollisionCandidates::default(),
            walls_signature: (0, 0),
            characters: CollisionCandidates::default(),
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
        grids.walls.insert(net_id.clone(), entity, aabb);
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
    query: Query<
        (&GgrsNetId, Entity, &fixed_math::FixedTransform3D, &Collider),
        (Without<Wall>, With<Rollback>),
    >,
) {
    grids.characters.clear();
    for (net_id, entity, transform, collider) in order_iter!(query) {
        let aabb = collider_aabb(&transform.translation, collider);
        grids.characters.insert(net_id.clone(), entity, aabb);
    }
}

/// Après `RollbackSystemSet::Movement`, avant `RollbackSystemSet::Weapon` : voir la doc du
/// module et de [`rebuild_character_grid_pre_movement`] (même contenu : tout sauf les murs).
pub fn rebuild_character_grid_post_movement(
    mut grids: ResMut<CollisionGrids>,
    query: Query<
        (&GgrsNetId, Entity, &fixed_math::FixedTransform3D, &Collider),
        (Without<Wall>, With<Rollback>),
    >,
) {
    grids.characters.clear();
    for (net_id, entity, transform, collider) in order_iter!(query) {
        let aabb = collider_aabb(&transform.translation, collider);
        grids.characters.insert(net_id.clone(), entity, aabb);
    }
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
            scan.insert(net_id, entity, aabb);
        }
        for x in -5..55 {
            let center = FixedVec2::new(Fixed::from_num(x * 16), Fixed::ZERO);
            let query = Aabb::from_circle(center, Fixed::from_num(16));
            let ids = |entries: Vec<&GridEntry>| {
                entries
                    .into_iter()
                    .map(|e| (e.net_id.0, e.entity))
                    .collect::<Vec<_>>()
            };
            assert_eq!(ids(scan.query_aabb(&query)), ids(grid.query_aabb(&query)));
        }
        scan.clear();
        assert!(scan
            .query_aabb(&Aabb::from_circle(FixedVec2::ZERO, Fixed::ONE))
            .is_empty());
    }
}
