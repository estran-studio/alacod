//! Dessin des murs (m2-salles-murs-gungeon, `docs/conventions.md` §83) : **présentation seule**.
//!
//! Les gabarits LDtk ne portent aucune tuile de mur (la couche IntGrid `Walls` est écrite à la main ou
//! par script, et l'assembleur de salles la modifie ensuite : ouvertures murées), si bien que les murs
//! n'avaient aucune image. Le mécanisme est une **auto-tuile en présentation** : à l'apparition d'un
//! niveau, chaque case non nulle de la couche déclarée par `wall_art` dans `ui/hud.ron` reçoit un sprite
//! choisi d'après ses voisines (la carte assemblée, pas le gabarit d'origine). Les jeux qui ne déclarent
//! pas `wall_art` ne changent pas ; sans rendu (headless, `HudConfigHandle` absent), le système ne tourne pas.
//!
//! Vue de dessus avec **face de mur** côté sud : une case dont la voisine du sud est un sol montre sa
//! face (briques), les autres montrent le dessus du mur (« chapeau »), dont les bords qui touchent le sol
//! sont soulignés. Hors du niveau, une case compte comme un mur.
//!
//! Numérotation des tuiles de la feuille (`scripts/generer-sprites-gungeon.py`, `atlas_murs.png`) :
//!
//! | indices | tuile |
//! |---|---|
//! | `0..32` | dessus : `rebords + 8 × coins` ; rebords : bit 0 nord, bit 1 est, bit 2 ouest (voisine non mur) ; coins : bit 0 nord-est, bit 1 nord-ouest (angle rentrant : les deux voisines orthogonales sont des murs, la diagonale un sol) |
//! | `32..36` | face : `32 + (ouest non mur) + 2 × (est non mur)` |
//! | `36..` | tuiles annexes (`wall_art.cells` : une tuile fixe par valeur d'une autre couche IntGrid ; `isaac` : `36` = piques de `Surfaces`) |

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use game::ui::hud::{HudConfig, HudConfigHandle, WallArt};

use crate::game::floors::FloorSlots;

/// Nombre de tuiles de la feuille.
pub const WALL_TILES: u32 = 36;

/// Racine des sprites de murs d'un niveau (enfant du niveau : il hérite de sa visibilité).
#[derive(Component)]
pub struct WallArtRoot;

/// Tuile d'une case de mur ; `wall(dx, dy)` dit si la voisine à `(dx, dy)` est un mur (`dy` > 0 : vers le
/// sud, comme les lignes de l'IntGrid, le sol de la case de départ étant `wall(0, 0)` = vrai).
pub fn wall_tile(wall: impl Fn(i32, i32) -> bool) -> u32 {
    let (n, e, s, w) = (wall(0, -1), wall(1, 0), wall(0, 1), wall(-1, 0));
    if !s {
        return 32 + u32::from(!w) + 2 * u32::from(!e);
    }
    let rim = u32::from(!n) | (u32::from(!e) << 1) | (u32::from(!w) << 2);
    let corners = u32::from(n && e && !wall(1, -1)) | (u32::from(n && w && !wall(-1, -1)) << 1);
    rim + 8 * corners
}

/// Cases `(colonne, ligne)` non nulles de `csv` (largeur `w`, hauteur `h`) avec leur tuile.
pub fn wall_cells(csv: &[i32], w: i32, h: i32) -> Vec<((i32, i32), u32)> {
    // Une couche qui n'est pas une grille de `w × h` cases (couche d'un autre type, CSV vide) : aucun mur.
    if w <= 0 || h <= 0 || csv.len() != (w as usize) * (h as usize) {
        return Vec::new();
    }
    let at = |x: i32, y: i32| {
        if x < 0 || y < 0 || x >= w || y >= h {
            return true;
        }
        csv.get((y * w + x) as usize).copied().unwrap_or(0) != 0
    };
    let mut cells = Vec::new();
    for y in 0..h {
        for x in 0..w {
            if csv[(y * w + x) as usize] != 0 {
                cells.push(((x, y), wall_tile(|dx, dy| at(x + dx, y + dy))));
            }
        }
    }
    cells
}

/// Cases `(colonne, ligne)` de `csv` (largeur `w`, hauteur `h`) qui valent `value` (couche annexe : `WallArt::cells`).
pub fn value_cells(csv: &[i32], w: i32, h: i32, value: i32) -> Vec<(i32, i32)> {
    if w <= 0 || h <= 0 || csv.len() != (w as usize) * (h as usize) {
        return Vec::new();
    }
    (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .filter(|&(x, y)| csv[(y * w + x) as usize] == value)
        .collect()
}

/// Tuile de sol d'une case : une des `tiles`, choisie par une fonction entière de la position (aucun
/// aléa : la même case a la même tuile à chaque chargement et sur chaque machine). `tiles` vide : aucune.
pub fn floor_tile(x: i32, y: i32, tiles: &[u32]) -> Option<u32> {
    if tiles.is_empty() {
        return None;
    }
    let hash = (x as u32).wrapping_mul(0x9E37_79B1) ^ (y as u32).wrapping_mul(0x85EB_CA6B);
    let hash = (hash ^ (hash >> 15)).wrapping_mul(0x2C1B_3C6D);
    Some(tiles[(hash >> 8) as usize % tiles.len()])
}

/// Cases `(colonne, ligne)` qui ne sont **pas** des murs de `csv` (largeur `w`, hauteur `h`) avec leur tuile de
/// sol : le sol posé sous les cases vides de la couche des murs (`WallArt::floor`).
pub fn floor_cells(csv: &[i32], w: i32, h: i32, tiles: &[u32]) -> Vec<((i32, i32), u32)> {
    if w <= 0 || h <= 0 || csv.len() != (w as usize) * (h as usize) {
        return Vec::new();
    }
    let mut cells = Vec::new();
    for y in 0..h {
        for x in 0..w {
            if csv[(y * w + x) as usize] == 0 {
                if let Some(tile) = floor_tile(x, y, tiles) {
                    cells.push(((x, y), tile));
                }
            }
        }
    }
    cells
}

/// Image et découpage de la feuille, chargés une fois par chemin.
#[derive(Default)]
pub struct WallSheet {
    atlas: String,
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

#[allow(clippy::too_many_arguments)]
pub fn draw_walls(
    mut commands: Commands,
    hud: Res<HudConfigHandle>,
    configs: Res<Assets<HudConfig>>,
    server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    levels: Query<(Entity, &LevelIid, Option<&Children>)>,
    roots: Query<(), With<WallArtRoot>>,
    projects: Query<&LdtkProjectHandle>,
    project_assets: Res<Assets<LdtkProject>>,
    slots: FloorSlots,
    mut sheet: Local<WallSheet>,
) {
    let Some(art) = hud.get(&configs).and_then(|c| c.wall_art.as_ref()) else {
        return;
    };
    for (entity, iid, children) in &levels {
        // Les couches d'un niveau apparaissent avec ses enfants : sans eux, il n'est pas encore là.
        if children.is_none() {
            continue;
        }
        // Déjà dessiné : le niveau porte un enfant `WallArtRoot` (pas de marqueur à tenir à jour).
        if children.is_some_and(|children| children.iter().any(|child| roots.contains(child))) {
            continue;
        }
        let Some(project) = slots
            .world_of(entity)
            .and_then(|world| projects.get(world).ok())
            .and_then(|handle| project_assets.get(handle))
        else {
            continue;
        };
        let Some(level) = project.get_raw_level_by_iid(&iid.to_string()) else {
            continue;
        };
        if sheet.atlas != art.atlas {
            sheet.atlas = art.atlas.clone();
            sheet.image = server.load(art.atlas.clone());
            // Les tuiles annexes (cases, sol) prolongent la feuille au-delà des 36 tuiles de murs.
            let used = art
                .cells
                .iter()
                .map(|cell| cell.tile)
                .chain(
                    art.floor
                        .iter()
                        .flat_map(|floor| floor.tiles.iter().copied()),
                )
                .map(|tile| tile + 1)
                .fold(WALL_TILES, u32::max);
            sheet.layout = layouts.add(TextureAtlasLayout::from_grid(
                UVec2::splat(art.tile),
                art.columns.max(1),
                used.div_ceil(art.columns.max(1)),
                None,
                None,
            ));
        }
        let mut cells = Vec::new();
        for layer in level.layer_instances.iter().flatten() {
            let size = layer.grid_size as f32;
            // Origine du niveau en bas à gauche, y vers le haut : la ligne 0 de l'IntGrid est en haut.
            let place = |x: i32, y: i32, z: f32| {
                Vec3::new(
                    (x as f32 + 0.5) * size,
                    (layer.c_hei - 1 - y) as f32 * size + size / 2.0,
                    z,
                )
            };
            if layer.identifier == art.layer {
                for ((x, y), tile) in wall_cells(&layer.int_grid_csv, layer.c_wid, layer.c_hei) {
                    cells.push((tile, place(x, y, art.z), size));
                }
                if let Some(floor) = &art.floor {
                    for ((x, y), tile) in
                        floor_cells(&layer.int_grid_csv, layer.c_wid, layer.c_hei, &floor.tiles)
                    {
                        cells.push((tile, place(x, y, floor.z), size));
                    }
                }
            }
            for cell_art in art.cells.iter().filter(|c| c.layer == layer.identifier) {
                for (x, y) in value_cells(
                    &layer.int_grid_csv,
                    layer.c_wid,
                    layer.c_hei,
                    cell_art.value,
                ) {
                    cells.push((cell_art.tile, place(x, y, cell_art.z), size));
                }
            }
        }
        let image = sheet.image.clone();
        let layout = sheet.layout.clone();
        commands.entity(entity).with_children(|parent| {
            parent
                .spawn((WallArtRoot, Transform::default(), Visibility::default()))
                .with_children(|root| {
                    for (tile, position, size) in cells {
                        root.spawn((
                            Sprite {
                                custom_size: Some(Vec2::splat(size)),
                                ..Sprite::from_atlas_image(
                                    image.clone(),
                                    TextureAtlas {
                                        layout: layout.clone(),
                                        index: tile as usize,
                                    },
                                )
                            },
                            Transform::from_translation(position),
                        ));
                    }
                });
        });
    }
}

/// Cache les couches de métadonnées listées par `wall_art.hide_layers` (rendu IntGrid colorié de LDtk : les
/// ouvertures des gabarits, `LevelConnection`, seraient peintes en noir par-dessus le sol).
pub fn hide_metadata_layers(
    hud: Res<HudConfigHandle>,
    configs: Res<Assets<HudConfig>>,
    mut layers: Query<(&LayerMetadata, &mut Visibility), Without<WallArtRoot>>,
) {
    let Some(art) = hud.get(&configs).and_then(|c| c.wall_art.as_ref()) else {
        return;
    };
    if art.hide_layers.is_empty() {
        return;
    }
    for (layer, mut visibility) in &mut layers {
        if art.hide_layers.contains(&layer.identifier) && *visibility != Visibility::Hidden {
            *visibility = Visibility::Hidden;
        }
    }
}

/// Garde-fou pour le plugin : vrai si la configuration du HUD est chargée (rendu présent).
pub fn wall_art_ready(hud: Option<Res<HudConfigHandle>>) -> bool {
    hud.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(rows: &[&str]) -> (Vec<i32>, i32, i32) {
        let h = rows.len() as i32;
        let w = rows[0].len() as i32;
        let csv = rows
            .iter()
            .flat_map(|r| r.chars().map(|c| i32::from(c == '#')))
            .collect();
        (csv, w, h)
    }

    fn tile_at(rows: &[&str], x: i32, y: i32) -> u32 {
        let (csv, w, h) = grid(rows);
        wall_cells(&csv, w, h)
            .into_iter()
            .find(|(p, _)| *p == (x, y))
            .expect("case de mur")
            .1
    }

    const ROOM: &[&str] = &[
        "########", "#......#", "#......#", "..####..", "#......#", "########",
    ];

    #[test]
    fn la_face_est_au_sud_d_un_sol_et_le_dessus_ailleurs() {
        // (3, 0) : mur du haut, sol au sud, murs à l'est et à l'ouest : face pleine.
        assert_eq!(tile_at(ROOM, 3, 0), 32);
        // (3, 5) : mur du bas, hors niveau au sud (compte comme un mur) : dessus, rebord nord.
        assert_eq!(tile_at(ROOM, 3, 5), 1);
    }

    #[test]
    fn les_extremites_d_un_mur_ont_un_chapeau_lateral() {
        // (2, 3) : mur isolé de gauche, sol à l'ouest : face avec chapeau ouest.
        assert_eq!(tile_at(ROOM, 2, 3), 33);
        assert_eq!(tile_at(ROOM, 5, 3), 34);
        // Un poteau d'une case : chapeaux des deux côtés.
        let pillar = ["....", ".#..", "...."];
        assert_eq!(tile_at(&pillar, 1, 1), 35);
    }

    #[test]
    fn l_angle_rentrant_d_une_piece_est_marque() {
        // (0, 0) : coin haut gauche, voisines est et sud des murs ; (1, 1) est un sol : pas d'angle (le
        // sud est un mur, la diagonale sud-est n'est pas lue).
        assert_eq!(tile_at(ROOM, 0, 0), 0);
        // (0, 5), coin bas gauche : nord (0, 4) mur, est mur, nord-est (1, 4) sol : angle nord-est.
        assert_eq!(tile_at(ROOM, 0, 5), 8);
        // (7, 5), coin bas droit : angle nord-ouest.
        assert_eq!(tile_at(ROOM, 7, 5), 16);
    }

    #[test]
    fn une_couche_mal_formee_ne_donne_aucun_mur() {
        assert!(wall_cells(&[], 8, 8).is_empty());
        assert!(wall_cells(&[1, 1, 1], 2, 2).is_empty());
        assert!(wall_cells(&[1], 0, 1).is_empty());
    }

    #[test]
    fn un_mur_a_ras_du_niveau_voit_l_exterieur_comme_un_mur() {
        let (csv, w, h) = grid(&["###", "#.#", "###"]);
        let cells = wall_cells(&csv, w, h);
        assert_eq!(cells.len(), 8);
        // Toutes les tuiles sont dans la feuille.
        assert!(cells.iter().all(|(_, t)| *t < WALL_TILES));
    }

    #[test]
    fn le_sol_couvre_les_cases_vides_et_seulement_elles() {
        let (csv, w, h) = grid(ROOM);
        let floor = floor_cells(&csv, w, h, &[36, 37, 38, 39]);
        let walls = wall_cells(&csv, w, h);
        assert_eq!(floor.len() + walls.len(), (w * h) as usize);
        assert!(floor.iter().all(|(_, t)| (36..40).contains(t)));
        assert!(floor.iter().all(|(p, _)| walls.iter().all(|(q, _)| p != q)));
        // Une seule variante : toutes les cases la prennent ; aucune variante : pas de sol.
        assert!(floor_cells(&csv, w, h, &[7]).iter().all(|(_, t)| *t == 7));
        assert!(floor_cells(&csv, w, h, &[]).is_empty());
        assert!(floor_cells(&[1, 0], 3, 1, &[7]).is_empty());
    }

    #[test]
    fn la_tuile_de_sol_ne_depend_que_de_la_position() {
        let tiles = [36, 37, 38, 39];
        assert_eq!(floor_tile(5, 9, &tiles), floor_tile(5, 9, &tiles));
        // Les quatre variantes servent toutes sur une petite pièce (pas de damier dégénéré).
        let used: std::collections::BTreeSet<u32> = (0..12)
            .flat_map(|x| (0..8).map(move |y| (x, y)))
            .filter_map(|(x, y)| floor_tile(x, y, &tiles))
            .collect();
        assert_eq!(used.len(), 4);
    }

    #[test]
    fn une_couche_annexe_ne_dessine_que_la_valeur_demandee() {
        // Deux valeurs sur une grille 3 x 2 : seules les cases de la valeur 5 (piques) sortent, ligne par ligne.
        let csv = [0, 5, 0, 5, 5, 2];
        assert_eq!(value_cells(&csv, 3, 2, 5), vec![(1, 0), (0, 1), (1, 1)]);
        assert_eq!(value_cells(&csv, 3, 2, 2), vec![(2, 1)]);
        assert!(value_cells(&csv, 3, 2, 7).is_empty());
        // Une couche qui n'est pas une grille de `w × h` cases ne dessine rien.
        assert!(value_cells(&csv, 4, 2, 5).is_empty());
    }
}
