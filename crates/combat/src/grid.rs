use bevy::prelude::{Entity, Resource};
use bevy_fixed::fixed_math::{Fixed, FixedVec2, FixedWide};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utils::net_id::GgrsNetId;

/// Boîte englobante alignée aux axes (AABB) en fixed-point.
/// Les bords sont inclus dans le chevauchement : deux boîtes avec min=max se chevauchent.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Aabb {
    pub min: FixedVec2,
    pub max: FixedVec2,
}

impl Aabb {
    /// Crée une AABB centré en `center` avec les dimensions données.
    pub fn from_center_size(center: FixedVec2, width: Fixed, height: Fixed) -> Self {
        let two = Fixed::from_num(2);
        let half_width = width.saturating_div(two);
        let half_height = height.saturating_div(two);

        Self {
            min: FixedVec2::new(center.x - half_width, center.y - half_height),
            max: FixedVec2::new(center.x + half_width, center.y + half_height),
        }
    }

    /// Crée une AABB englobant un cercle de centre et rayon donnés.
    pub fn from_circle(center: FixedVec2, radius: Fixed) -> Self {
        Self {
            min: FixedVec2::new(center.x - radius, center.y - radius),
            max: FixedVec2::new(center.x + radius, center.y + radius),
        }
    }

    /// Retourne vrai si cette boîte chevauche `other`.
    /// Les bords sont inclus : deux boîtes qui se touchent exactement se chevauchent.
    pub fn overlaps(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }
}

/// Entrée dans la grille spatiale : une entité avec son AABB.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GridEntry {
    pub net_id: GgrsNetId,
    pub entity: Entity,
    pub aabb: Aabb,
}

/// Constante : taille de cellule par défaut (32 unités fixed-point).
pub const DEFAULT_CELL_SIZE: Fixed = Fixed::from_bits(32 << 16);

/// Grille spatiale déterministe pour requêtes rapides par zone.
/// Utilise BTreeMap pour l'ordre déterministe, pas de HashMap.
/// Les entrées sont insérées dans l'ordre où on les donne (l'appelant les fournira triées par net_id).
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct SpatialGrid {
    cell_size: Fixed,
    /// Map (cell_x, cell_y) -> indices des entrées dans `self.entries`
    cells: BTreeMap<(i32, i32), Vec<usize>>,
    /// Toutes les entrées, indexées pour la rapidité.
    entries: Vec<GridEntry>,
}

impl Default for SpatialGrid {
    fn default() -> Self {
        Self::new(DEFAULT_CELL_SIZE)
    }
}

impl SpatialGrid {
    /// Crée une grille vide avec la taille de cellule donnée.
    pub fn new(cell_size: Fixed) -> Self {
        Self {
            cell_size,
            cells: BTreeMap::new(),
            entries: Vec::new(),
        }
    }

    /// Vide la grille.
    pub fn clear(&mut self) {
        self.cells.clear();
        self.entries.clear();
    }

    /// Insère une entrée dans la grille.
    /// L'AABB peut couvrir plusieurs cellules ; l'entrée sera insérée dans chacune.
    pub fn insert(&mut self, net_id: GgrsNetId, entity: Entity, aabb: Aabb) {
        let entry_idx = self.entries.len();
        self.entries.push(GridEntry {
            net_id,
            entity,
            aabb,
        });

        // Calculer toutes les cellules couvertes par l'AABB.
        let cell_min = self.pos_to_cell(aabb.min);
        let cell_max = self.pos_to_cell(aabb.max);

        for cell_x in cell_min.0..=cell_max.0 {
            for cell_y in cell_min.1..=cell_max.1 {
                self.cells
                    .entry((cell_x, cell_y))
                    .or_insert_with(Vec::new)
                    .push(entry_idx);
            }
        }
    }

    /// Nombre d'entrées dans la grille.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Retourne true si la grille est vide.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Taille de cellule de cette grille.
    pub fn cell_size(&self) -> Fixed {
        self.cell_size
    }

    /// Requête par AABB : retourne toutes les entrées dont l'AABB chevauche `aabb`.
    /// Les résultats sont dédupliqués et triés par `net_id`.
    pub fn query_aabb(&self, aabb: &Aabb) -> Vec<&GridEntry> {
        let mut result_indices = std::collections::BTreeSet::new();

        let cell_min = self.pos_to_cell(aabb.min);
        let cell_max = self.pos_to_cell(aabb.max);

        for cell_x in cell_min.0..=cell_max.0 {
            for cell_y in cell_min.1..=cell_max.1 {
                if let Some(indices) = self.cells.get(&(cell_x, cell_y)) {
                    for &idx in indices {
                        if self.entries[idx].aabb.overlaps(aabb) {
                            result_indices.insert(idx);
                        }
                    }
                }
            }
        }

        let mut result: Vec<&GridEntry> = result_indices
            .iter()
            .map(|&idx| &self.entries[idx])
            .collect();
        result.sort_by_key(|entry| entry.net_id.0);
        result
    }

    /// Requête par cercle : retourne toutes les entrées dont l'AABB chevauche le cercle englobant,
    /// puis filtre précisément par test cercle-rectangle.
    /// Les résultats sont dédupliqués et triés par `net_id`.
    pub fn query_circle(&self, center: FixedVec2, radius: Fixed) -> Vec<&GridEntry> {
        let circle_aabb = Aabb::from_circle(center, radius);
        let mut result_indices = std::collections::BTreeSet::new();

        let cell_min = self.pos_to_cell(circle_aabb.min);
        let cell_max = self.pos_to_cell(circle_aabb.max);

        for cell_x in cell_min.0..=cell_max.0 {
            for cell_y in cell_min.1..=cell_max.1 {
                if let Some(indices) = self.cells.get(&(cell_x, cell_y)) {
                    for &idx in indices {
                        if circle_aabb.overlaps(&self.entries[idx].aabb) {
                            // Test précis cercle-rectangle
                            if self.circle_overlaps_aabb(center, radius, &self.entries[idx].aabb) {
                                result_indices.insert(idx);
                            }
                        }
                    }
                }
            }
        }

        let mut result: Vec<&GridEntry> = result_indices
            .iter()
            .map(|&idx| &self.entries[idx])
            .collect();
        result.sort_by_key(|entry| entry.net_id.0);
        result
    }

    /// Visite chaque paire d'entrées dont les AABBs se chevauchent, une seule fois,
    /// dans l'ordre `(net_id_a.0 < net_id_b.0)`.
    pub fn for_each_pair(&self, mut f: impl FnMut(&GridEntry, &GridEntry)) {
        let mut visited = std::collections::BTreeSet::new();

        for cell_indices in self.cells.values() {
            for &i in cell_indices {
                for &j in cell_indices {
                    if i == j {
                        continue;
                    }
                    let (a, b) = if self.entries[i].net_id.0 < self.entries[j].net_id.0 {
                        (i, j)
                    } else {
                        (j, i)
                    };

                    if visited.insert((a, b))
                        && self.entries[a].aabb.overlaps(&self.entries[b].aabb)
                    {
                        f(&self.entries[a], &self.entries[b]);
                    }
                }
            }
        }
    }

    // --- Helpers privés ---

    /// Convertit une position en coordonnées de cellule.
    /// Les coordonnées négatives sont supportées (floor division).
    fn pos_to_cell(&self, pos: FixedVec2) -> (i32, i32) {
        // Division en FixedWide (exacte : I64F32 contient I32F16), puis plancher.
        let cell_size_fw = FixedWide::from_num(self.cell_size);
        let x_fw = FixedWide::from_num(pos.x);
        let y_fw = FixedWide::from_num(pos.y);

        // `floor` et non troncature : -0.5 doit tomber dans la cellule -1.
        let cell_x_fw = x_fw.saturating_div(cell_size_fw).floor().to_num::<i32>();
        let cell_y_fw = y_fw.saturating_div(cell_size_fw).floor().to_num::<i32>();

        (cell_x_fw, cell_y_fw)
    }

    /// Test précis : vérifie si un cercle chevauche une AABB.
    /// Le cercle est défini par son centre et son rayon.
    fn circle_overlaps_aabb(&self, center: FixedVec2, radius: Fixed, aabb: &Aabb) -> bool {
        // Trouver le point le plus proche sur l'AABB au centre du cercle.
        let closest_x = center.x.max(aabb.min.x).min(aabb.max.x);
        let closest_y = center.y.max(aabb.min.y).min(aabb.max.y);

        let diff_x = center.x - closest_x;
        let diff_y = center.y - closest_y;

        // Calculer la distance au carré en FixedWide.
        let diff_x_fw = FixedWide::from_num(diff_x);
        let diff_y_fw = FixedWide::from_num(diff_y);
        let distance_sq_fw =
            diff_x_fw.saturating_mul(diff_x_fw) + diff_y_fw.saturating_mul(diff_y_fw);

        let radius_fw = FixedWide::from_num(radius);
        let radius_sq_fw = radius_fw.saturating_mul(radius_fw);

        distance_sq_fw <= radius_sq_fw
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Entity;

    /// Une entité de test valide (index seul) : la grille ne fait que la transporter.
    fn test_entity(i: u32) -> Entity {
        Entity::from_raw_u32(i).expect("index d'entité valide")
    }

    /// Simple LCG (Linear Congruential Generator) 64-bit pour les tests.
    struct Lcg64 {
        state: u64,
    }

    impl Lcg64 {
        fn new(seed: u64) -> Self {
            Self { state: seed }
        }

        /// Retourne le prochain nombre aléatoire en [0, u64::MAX].
        fn next(&mut self) -> u64 {
            const A: u64 = 6364136223846793005;
            const C: u64 = 1442695040888963407;
            self.state = self.state.wrapping_mul(A).wrapping_add(C);
            self.state
        }

        /// Retourne un nombre aléatoire en [min, max).
        fn range(&mut self, min: i32, max: i32) -> i32 {
            let range = (max - min) as u64;
            let r = (self.next() % range) as i32;
            min + r
        }
    }

    #[test]
    fn test_aabb_from_center_size() {
        let center = FixedVec2::new(Fixed::from_num(100), Fixed::from_num(200));
        let width = Fixed::from_num(40);
        let height = Fixed::from_num(60);
        let aabb = Aabb::from_center_size(center, width, height);

        assert_eq!(aabb.min.x, Fixed::from_num(80));
        assert_eq!(aabb.max.x, Fixed::from_num(120));
        assert_eq!(aabb.min.y, Fixed::from_num(170));
        assert_eq!(aabb.max.y, Fixed::from_num(230));
    }

    #[test]
    fn test_aabb_from_circle() {
        let center = FixedVec2::new(Fixed::from_num(100), Fixed::from_num(200));
        let radius = Fixed::from_num(50);
        let aabb = Aabb::from_circle(center, radius);

        assert_eq!(aabb.min.x, Fixed::from_num(50));
        assert_eq!(aabb.max.x, Fixed::from_num(150));
        assert_eq!(aabb.min.y, Fixed::from_num(150));
        assert_eq!(aabb.max.y, Fixed::from_num(250));
    }

    #[test]
    fn test_aabb_overlaps() {
        let aabb1 = Aabb {
            min: FixedVec2::new(Fixed::from_num(0), Fixed::from_num(0)),
            max: FixedVec2::new(Fixed::from_num(100), Fixed::from_num(100)),
        };

        let aabb2 = Aabb {
            min: FixedVec2::new(Fixed::from_num(50), Fixed::from_num(50)),
            max: FixedVec2::new(Fixed::from_num(150), Fixed::from_num(150)),
        };

        let aabb3 = Aabb {
            min: FixedVec2::new(Fixed::from_num(200), Fixed::from_num(200)),
            max: FixedVec2::new(Fixed::from_num(300), Fixed::from_num(300)),
        };

        assert!(aabb1.overlaps(&aabb2));
        assert!(aabb2.overlaps(&aabb1));
        assert!(!aabb1.overlaps(&aabb3));
    }

    #[test]
    fn test_grid_basic() {
        let mut grid = SpatialGrid::new(Fixed::from_num(32));

        let aabb = Aabb {
            min: FixedVec2::new(Fixed::from_num(0), Fixed::from_num(0)),
            max: FixedVec2::new(Fixed::from_num(10), Fixed::from_num(10)),
        };

        let net_id = GgrsNetId(1, "test".to_string());
        let entity = test_entity(1);

        grid.insert(net_id.clone(), entity, aabb);

        assert_eq!(grid.len(), 1);
        assert!(!grid.is_empty());

        grid.clear();
        assert!(grid.is_empty());
    }

    #[test]
    fn test_grid_query_aabb() {
        let mut grid = SpatialGrid::new(Fixed::from_num(32));

        // Insérer trois boîtes
        let aabb1 = Aabb {
            min: FixedVec2::new(Fixed::from_num(0), Fixed::from_num(0)),
            max: FixedVec2::new(Fixed::from_num(50), Fixed::from_num(50)),
        };
        let net_id1 = GgrsNetId(1, "a".to_string());
        grid.insert(net_id1.clone(), test_entity(1), aabb1);

        let aabb2 = Aabb {
            min: FixedVec2::new(Fixed::from_num(40), Fixed::from_num(40)),
            max: FixedVec2::new(Fixed::from_num(100), Fixed::from_num(100)),
        };
        let net_id2 = GgrsNetId(2, "b".to_string());
        grid.insert(net_id2.clone(), test_entity(2), aabb2);

        let aabb3 = Aabb {
            min: FixedVec2::new(Fixed::from_num(200), Fixed::from_num(200)),
            max: FixedVec2::new(Fixed::from_num(250), Fixed::from_num(250)),
        };
        let net_id3 = GgrsNetId(3, "c".to_string());
        grid.insert(net_id3.clone(), test_entity(3), aabb3);

        // Requête qui doit trouver les deux premières
        let query_box = Aabb {
            min: FixedVec2::new(Fixed::from_num(30), Fixed::from_num(30)),
            max: FixedVec2::new(Fixed::from_num(60), Fixed::from_num(60)),
        };
        let results = grid.query_aabb(&query_box);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].net_id.0, 1);
        assert_eq!(results[1].net_id.0, 2);
    }

    #[test]
    fn test_grid_query_circle() {
        let mut grid = SpatialGrid::new(Fixed::from_num(32));

        let aabb1 = Aabb {
            min: FixedVec2::new(Fixed::from_num(0), Fixed::from_num(0)),
            max: FixedVec2::new(Fixed::from_num(20), Fixed::from_num(20)),
        };
        let net_id1 = GgrsNetId(1, "a".to_string());
        grid.insert(net_id1.clone(), test_entity(1), aabb1);

        let aabb2 = Aabb {
            min: FixedVec2::new(Fixed::from_num(100), Fixed::from_num(100)),
            max: FixedVec2::new(Fixed::from_num(120), Fixed::from_num(120)),
        };
        let net_id2 = GgrsNetId(2, "b".to_string());
        grid.insert(net_id2.clone(), test_entity(2), aabb2);

        // Requête cercle au centre de la première boîte
        let center = FixedVec2::new(Fixed::from_num(10), Fixed::from_num(10));
        let radius = Fixed::from_num(20);
        let results = grid.query_circle(center, radius);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].net_id.0, 1);
    }

    #[test]
    fn test_grid_for_each_pair() {
        let mut grid = SpatialGrid::new(Fixed::from_num(32));

        // Insérer deux boîtes qui se chevauchent
        let aabb1 = Aabb {
            min: FixedVec2::new(Fixed::from_num(0), Fixed::from_num(0)),
            max: FixedVec2::new(Fixed::from_num(50), Fixed::from_num(50)),
        };
        let net_id1 = GgrsNetId(1, "a".to_string());
        grid.insert(net_id1.clone(), test_entity(1), aabb1);

        let aabb2 = Aabb {
            min: FixedVec2::new(Fixed::from_num(40), Fixed::from_num(40)),
            max: FixedVec2::new(Fixed::from_num(100), Fixed::from_num(100)),
        };
        let net_id2 = GgrsNetId(2, "b".to_string());
        grid.insert(net_id2.clone(), test_entity(2), aabb2);

        // Insérer une troisième boîte qui ne se chevauche pas
        let aabb3 = Aabb {
            min: FixedVec2::new(Fixed::from_num(200), Fixed::from_num(200)),
            max: FixedVec2::new(Fixed::from_num(250), Fixed::from_num(250)),
        };
        let net_id3 = GgrsNetId(3, "c".to_string());
        grid.insert(net_id3.clone(), test_entity(3), aabb3);

        let mut pairs = Vec::new();
        grid.for_each_pair(|a, b| {
            pairs.push((a.net_id.0, b.net_id.0));
        });

        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], (1, 2));
    }

    #[test]
    fn test_grid_large_box_multiple_cells() {
        let mut grid = SpatialGrid::new(Fixed::from_num(32));

        // Une boîte qui couvre plusieurs cellules
        let large_aabb = Aabb {
            min: FixedVec2::new(Fixed::from_num(10), Fixed::from_num(10)),
            max: FixedVec2::new(Fixed::from_num(100), Fixed::from_num(100)),
        };
        let net_id = GgrsNetId(1, "large".to_string());
        grid.insert(net_id.clone(), test_entity(1), large_aabb);

        // Requête qui devrait trouver la boîte
        let query_box = Aabb {
            min: FixedVec2::new(Fixed::from_num(50), Fixed::from_num(50)),
            max: FixedVec2::new(Fixed::from_num(60), Fixed::from_num(60)),
        };
        let results = grid.query_aabb(&query_box);

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_grid_negative_coordinates() {
        let mut grid = SpatialGrid::new(Fixed::from_num(32));

        let aabb = Aabb {
            min: FixedVec2::new(Fixed::from_num(-100), Fixed::from_num(-100)),
            max: FixedVec2::new(Fixed::from_num(-50), Fixed::from_num(-50)),
        };
        let net_id = GgrsNetId(1, "neg".to_string());
        grid.insert(net_id.clone(), test_entity(1), aabb);

        let query_box = Aabb {
            min: FixedVec2::new(Fixed::from_num(-80), Fixed::from_num(-80)),
            max: FixedVec2::new(Fixed::from_num(-60), Fixed::from_num(-60)),
        };
        let results = grid.query_aabb(&query_box);

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_results_sorted_by_net_id() {
        let mut grid = SpatialGrid::new(Fixed::from_num(32));

        // Insérer dans un ordre mélangé
        let aabb = Aabb {
            min: FixedVec2::new(Fixed::from_num(0), Fixed::from_num(0)),
            max: FixedVec2::new(Fixed::from_num(100), Fixed::from_num(100)),
        };

        grid.insert(GgrsNetId(3, "c".to_string()), test_entity(3), aabb);
        grid.insert(GgrsNetId(1, "a".to_string()), test_entity(1), aabb);
        grid.insert(GgrsNetId(2, "b".to_string()), test_entity(2), aabb);

        let results = grid.query_aabb(&aabb);

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].net_id.0, 1);
        assert_eq!(results[1].net_id.0, 2);
        assert_eq!(results[2].net_id.0, 3);
    }

    #[test]
    fn test_results_deduplicated() {
        let mut grid = SpatialGrid::new(Fixed::from_num(32));

        // Une boîte qui couvre plusieurs cellules apparaît plusieurs fois en interne
        let large_aabb = Aabb {
            min: FixedVec2::new(Fixed::from_num(0), Fixed::from_num(0)),
            max: FixedVec2::new(Fixed::from_num(100), Fixed::from_num(100)),
        };
        let net_id = GgrsNetId(1, "large".to_string());
        grid.insert(net_id.clone(), test_entity(1), large_aabb);

        let results = grid.query_aabb(&large_aabb);

        // Doit avoir une seule occurrence malgré la présence dans plusieurs cellules
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_equivalence_with_brute_force_aabb() {
        let mut grid = SpatialGrid::new(Fixed::from_num(32));
        let mut entries = Vec::new();

        let mut rng = Lcg64::new(42);

        // Générer 50 boîtes aléatoires (limiter pour éviter les problèmes de bits invalides dans les tests)
        for i in 0..50u32 {
            let pos_x = rng.range(-2000, 2000);
            let pos_y = rng.range(-2000, 2000);
            let width = rng.range(4, 120);
            let height = rng.range(4, 120);

            let center = FixedVec2::new(Fixed::from_num(pos_x), Fixed::from_num(pos_y));
            let aabb =
                Aabb::from_center_size(center, Fixed::from_num(width), Fixed::from_num(height));

            let net_id = GgrsNetId(i as usize, format!("box{}", i));
            let entity = test_entity(i);
            grid.insert(net_id.clone(), entity, aabb);
            entries.push((net_id, aabb));
        }

        // Lancer 200 requêtes AABB aléatoires
        rng = Lcg64::new(123);
        for _ in 0..200 {
            let query_x = rng.range(-2000, 2000);
            let query_y = rng.range(-2000, 2000);
            let query_w = rng.range(4, 120);
            let query_h = rng.range(4, 120);

            let center = FixedVec2::new(Fixed::from_num(query_x), Fixed::from_num(query_y));
            let query_box =
                Aabb::from_center_size(center, Fixed::from_num(query_w), Fixed::from_num(query_h));

            // Résultat de la grille
            let grid_results = grid.query_aabb(&query_box);
            let grid_ids: std::collections::BTreeSet<_> =
                grid_results.iter().map(|e| e.net_id.0).collect();

            // Résultat de la force brute
            let mut brute_ids = std::collections::BTreeSet::new();
            for (net_id, aabb) in &entries {
                if aabb.overlaps(&query_box) {
                    brute_ids.insert(net_id.0);
                }
            }

            assert_eq!(grid_ids, brute_ids, "Mismatch for query {:?}", query_box);
        }
    }

    #[test]
    fn test_equivalence_with_brute_force_circle() {
        let mut grid = SpatialGrid::new(Fixed::from_num(32));
        let mut entries = Vec::new();

        let mut rng = Lcg64::new(42);

        // Générer 50 boîtes aléatoires (limiter pour éviter les problèmes de bits invalides dans les tests)
        for i in 0..50u32 {
            let pos_x = rng.range(-2000, 2000);
            let pos_y = rng.range(-2000, 2000);
            let width = rng.range(4, 120);
            let height = rng.range(4, 120);

            let center = FixedVec2::new(Fixed::from_num(pos_x), Fixed::from_num(pos_y));
            let aabb =
                Aabb::from_center_size(center, Fixed::from_num(width), Fixed::from_num(height));

            let net_id = GgrsNetId(i as usize, format!("box{}", i));
            let entity = test_entity(i);
            grid.insert(net_id.clone(), entity, aabb);
            entries.push((net_id, aabb));
        }

        // Lancer 200 requêtes cercle aléatoires
        rng = Lcg64::new(123);
        for _ in 0..200 {
            let center_x = rng.range(-2000, 2000);
            let center_y = rng.range(-2000, 2000);
            let radius = rng.range(4, 120);

            let center = FixedVec2::new(Fixed::from_num(center_x), Fixed::from_num(center_y));
            let radius_fixed = Fixed::from_num(radius);

            // Résultat de la grille
            let grid_results = grid.query_circle(center, radius_fixed);
            let grid_ids: std::collections::BTreeSet<_> =
                grid_results.iter().map(|e| e.net_id.0).collect();

            // Résultat de la force brute
            let mut brute_ids = std::collections::BTreeSet::new();
            let circle_aabb = Aabb::from_circle(center, radius_fixed);
            for (net_id, aabb) in &entries {
                if circle_aabb.overlaps(aabb)
                    && circle_overlaps_aabb_brute(center, radius_fixed, aabb)
                {
                    brute_ids.insert(net_id.0);
                }
            }

            assert_eq!(
                grid_ids, brute_ids,
                "Mismatch for query circle center={:?}, radius={}",
                center, radius
            );
        }
    }

    #[test]
    fn test_equivalence_with_brute_force_for_each_pair() {
        let mut grid = SpatialGrid::new(Fixed::from_num(32));
        let mut entries = Vec::new();

        let mut rng = Lcg64::new(42);

        // Générer 100 boîtes aléatoires (moins pour éviter trop de paires)
        for i in 0..100u32 {
            let pos_x = rng.range(-2000, 2000);
            let pos_y = rng.range(-2000, 2000);
            let width = rng.range(4, 120);
            let height = rng.range(4, 120);

            let center = FixedVec2::new(Fixed::from_num(pos_x), Fixed::from_num(pos_y));
            let aabb =
                Aabb::from_center_size(center, Fixed::from_num(width), Fixed::from_num(height));

            let net_id = GgrsNetId(i as usize, format!("box{}", i));
            let entity = test_entity(i);
            grid.insert(net_id.clone(), entity, aabb);
            entries.push((net_id, aabb));
        }

        // Résultats de la grille
        let mut grid_pairs = Vec::new();
        grid.for_each_pair(|a, b| {
            grid_pairs.push((a.net_id.0, b.net_id.0));
        });
        grid_pairs.sort();

        // Résultats de la force brute
        let mut brute_pairs = Vec::new();
        for i in 0..entries.len() {
            for j in (i + 1)..entries.len() {
                if entries[i].1.overlaps(&entries[j].1) {
                    let a = entries[i].0 .0;
                    let b = entries[j].0 .0;
                    brute_pairs.push((a.min(b), a.max(b)));
                }
            }
        }
        brute_pairs.sort();

        assert_eq!(grid_pairs, brute_pairs);
    }

    #[test]
    fn test_grid_different_cell_sizes() {
        for cell_size_val in [8, 32, 256].iter() {
            let mut grid = SpatialGrid::new(Fixed::from_num(*cell_size_val));
            let mut entries = Vec::new();

            let mut rng = Lcg64::new(42);

            // Générer 50 boîtes aléatoires (limiter pour éviter les problèmes de bits invalides dans les tests)
            for i in 0..50u32 {
                let pos_x = rng.range(-2000, 2000);
                let pos_y = rng.range(-2000, 2000);
                let width = rng.range(4, 120);
                let height = rng.range(4, 120);

                let center = FixedVec2::new(Fixed::from_num(pos_x), Fixed::from_num(pos_y));
                let aabb =
                    Aabb::from_center_size(center, Fixed::from_num(width), Fixed::from_num(height));

                let net_id = GgrsNetId(i as usize, format!("box{}", i));
                let entity = test_entity(i);
                grid.insert(net_id.clone(), entity, aabb);
                entries.push((net_id, aabb));
            }

            // Lancer 200 requêtes AABB aléatoires
            rng = Lcg64::new(123);
            for _ in 0..200 {
                let query_x = rng.range(-2000, 2000);
                let query_y = rng.range(-2000, 2000);
                let query_w = rng.range(4, 120);
                let query_h = rng.range(4, 120);

                let center = FixedVec2::new(Fixed::from_num(query_x), Fixed::from_num(query_y));
                let query_box = Aabb::from_center_size(
                    center,
                    Fixed::from_num(query_w),
                    Fixed::from_num(query_h),
                );

                // Résultat de la grille
                let grid_results = grid.query_aabb(&query_box);
                let grid_ids: std::collections::BTreeSet<_> =
                    grid_results.iter().map(|e| e.net_id.0).collect();

                // Résultat de la force brute
                let mut brute_ids = std::collections::BTreeSet::new();
                for (net_id, aabb) in &entries {
                    if aabb.overlaps(&query_box) {
                        brute_ids.insert(net_id.0);
                    }
                }

                assert_eq!(
                    grid_ids, brute_ids,
                    "Mismatch for cell_size={}, query box {:?}",
                    cell_size_val, query_box
                );
            }
        }
    }

    /// Helper brute force pour tester le cercle-rectangle
    fn circle_overlaps_aabb_brute(center: FixedVec2, radius: Fixed, aabb: &Aabb) -> bool {
        let closest_x = center.x.max(aabb.min.x).min(aabb.max.x);
        let closest_y = center.y.max(aabb.min.y).min(aabb.max.y);

        let diff_x = center.x - closest_x;
        let diff_y = center.y - closest_y;

        let diff_x_fw = FixedWide::from_num(diff_x);
        let diff_y_fw = FixedWide::from_num(diff_y);
        let distance_sq_fw =
            diff_x_fw.saturating_mul(diff_x_fw) + diff_y_fw.saturating_mul(diff_y_fw);

        let radius_fw = FixedWide::from_num(radius);
        let radius_sq_fw = radius_fw.saturating_mul(radius_fw);

        distance_sq_fw <= radius_sq_fw
    }
}
