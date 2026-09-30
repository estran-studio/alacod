//! Grilles de collision dérivées (T2.1, chantier B4b, `docs/taches.md`) : reconstruites à
//! partir de l'état rollback à des points précis de `GgrsSchedule`, jamais lues d'une frame
//! à l'autre. [`CollisionGrids`] n'est donc **ni rollback ni checksum** (`docs/taches.md` :
//! « donnée dérivée... reconstruite à chaque frame avant usage ») : elle ne participe jamais
//! au résultat de la simulation, seulement à sa vitesse. Remplace les boucles « pour chaque
//! balle/personnage, pour chaque cible » par des requêtes sur `combat::grid::SpatialGrid`
//! (T1.8), déjà prouvé équivalent à la force brute par ses tests.
//!
//! Deux grilles :
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
//! Taille de cellule : 64 = 2 × 32, où 32 couvre le plus grand côté des colliders les plus
//! **courants** dans `characters` (joueur/ennemi 20×20 ; fenêtre jusqu'à 32×16, voir
//! `games/zombies/assets/exemples/test_map.ldtk`) — l'écrasante majorité des entrées à
//! chaque frame. Les entrées plus grandes et rares (hitbox de mêlée jusqu'à ~80 de diamètre
//! pour un balayage à l'épée ; murs fusionnés jusqu'à plusieurs centaines d'unités,
//! `map_ldtk::game::collider::generate_collision_rectangles`) restent correctes : elles
//! occupent simplement plusieurs cellules (`SpatialGrid::insert`/`query_aabb` gèrent ce cas,
//! T1.8 le prouve par équivalence avec la force brute), donc ce choix n'affecte que la
//! performance, jamais la justesse.

use bevy::prelude::*;
use bevy_fixed::fixed_math::{self, Fixed, FixedVec2, FixedVec3};
use bevy_ggrs::Rollback;
use combat::grid::{Aabb, SpatialGrid};
use utils::{net_id::GgrsNetId, order_iter};

use crate::collider::{Collider, ColliderShape, Wall};

/// Ressource dérivée (non rollback, non checksum) : voir la doc du module.
#[derive(Resource)]
pub struct CollisionGrids {
    pub walls: SpatialGrid,
    /// Nombre de murs (`With<Wall>, With<Collider>, With<Rollback>`) vu à la dernière
    /// reconstruction de `walls` ; sert seulement à détecter un changement (porte
    /// ouverte/fermée, mur ajouté), voir [`maybe_rebuild_wall_grid`].
    walls_collider_count: usize,
    pub characters: SpatialGrid,
}

/// Taille de cellule partagée par les deux grilles (voir la doc du module).
fn cell_size() -> Fixed {
    fixed_math::new(64.0)
}

impl Default for CollisionGrids {
    fn default() -> Self {
        Self {
            walls: SpatialGrid::new(cell_size()),
            walls_collider_count: 0,
            characters: SpatialGrid::new(cell_size()),
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

/// Reconstruit [`CollisionGrids::walls`] seulement quand le nombre de murs avec collider a
/// changé depuis la dernière frame (une porte qui s'ouvre/se ferme retire/pose son
/// `Collider`, voir `crate::interaction`). `.after(RollbackSystemSet::Interaction)` : voit
/// les portes ouvertes/fermées cette frame ; `.before(RollbackSystemSet::Movement)` : prêt
/// avant son premier consommateur (`move_characters`).
pub fn maybe_rebuild_wall_grid(
    mut grids: ResMut<CollisionGrids>,
    wall_query: Query<
        (&GgrsNetId, Entity, &fixed_math::FixedTransform3D, &Collider),
        (With<Wall>, With<Rollback>),
    >,
) {
    let count = wall_query.iter().count();
    if count == grids.walls_collider_count {
        return;
    }
    grids.walls_collider_count = count;
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
