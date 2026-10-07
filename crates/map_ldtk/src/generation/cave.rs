//! Caverne (T1.6, `docs/conventions.md` §21) : le niveau unique du gabarit LDtk est réécrit en
//! mémoire depuis `world::cave::generate`. Tout ce qui suit (colliders, navigation, spawns,
//! mode `Floors`) prend le chemin LDtk ordinaire.

use bevy::math::{IVec2, Vec2};
use bevy_ecs_ldtk::{
    ldtk::{FieldInstance, FieldValue, LdtkJson, RealEditorValue, Type},
    EntityInstance,
};

use world::{CaveConfig, CellGrid, CellKind, CELL_SIZE};

use crate::map_const;

/// Couche IntGrid des murs du gabarit (1 = `Wall` ou `Rock`), lue par les colliders et la
/// navigation comme pour toute carte.
pub const LAYER_WALLS: &str = "Walls";

/// Nombre de `PlayerSpawn` posés (index 0..4, un par joueur possible).
pub const CAVE_PLAYER_SPAWNS: usize = 4;

/// Identifiant stable du niveau de caverne (trié par iid au chargement des murs). Contient l'id
/// de la caverne (m1-integration-scenarios) : les cavernes d'une même séquence `Floors`
/// partagent la graine, leurs niveaux ne doivent pas partager l'iid.
pub fn cave_level_iid(id: &str, seed: i32) -> String {
    format!("cave-{id}-{seed}")
}

/// Grille de la caverne pour une graine de scénario (`map_seed`, même source que `Basic`).
pub fn cave_grid(seed: i32, config: &CaveConfig) -> CellGrid {
    world::generate(seed as u32 as u64, config)
}

/// Réécrit le premier niveau du gabarit : dimensions, IntGrid `Walls`, `PlayerSpawn{index}` et
/// `ZombieSpawn` ; les autres niveaux du gabarit sont retirés. Le niveau est placé en (0, 0)
/// monde, l'origine de `CellGrid` (`world_y = -px_hei` : bevy_ecs_ldtk place le bas du niveau
/// à `-(world_y + px_hei)`).
pub fn build_cave_ldtk(template: &LdtkJson, id: &str, seed: i32, config: &CaveConfig) -> LdtkJson {
    let grid = cave_grid(seed, config);
    let points = world::points_of_interest(
        &grid,
        CAVE_PLAYER_SPAWNS,
        config.enemy_spawns,
        config.spawn_clearance,
        config.nav_large,
    );
    let (w, h) = (grid.width as i32, grid.height as i32);

    let mut ldtk = template.clone();
    let mut level = ldtk
        .levels
        .first()
        .cloned()
        .expect("gabarit de caverne sans niveau");
    level.iid = cave_level_iid(id, seed);
    level.identifier = "Cave".into();
    level.px_wid = w * CELL_SIZE;
    level.px_hei = h * CELL_SIZE;
    level.world_x = 0;
    level.world_y = -level.px_hei;
    level.neighbours.clear();

    // Rangée LDtk r (haut en bas) <-> rangée de grille y = h - 1 - r (bas en haut)
    let csv: Vec<i32> = (0..h)
        .flat_map(|r| (0..w).map(move |x| (x, h - 1 - r)))
        .map(|(x, y)| i32::from(grid.get(x, y).is_some_and(CellKind::is_solid)))
        .collect();

    let mut entities = Vec::new();
    for (index, &(x, y)) in points.player_spawns.iter().enumerate() {
        entities.push(entity(
            template,
            map_const::ENTITY_PLAYER_SPAWN_LOCATION,
            (x as i32, h - 1 - y as i32),
            vec![(
                map_const::FIELD_PLAYER_SPAWN_INDEX_NAME,
                FieldValue::Int(Some(index as i32)),
            )],
            format!("{}-player-{index}", level.iid),
        ));
    }
    for (index, &(x, y)) in points.zombie_spawns.iter().enumerate() {
        let cell = (x as i32, h - 1 - y as i32);
        entities.push(entity(
            template,
            map_const::ENTITY_ZOMBIE_SPAWN_LOCATION,
            cell,
            vec![],
            format!("{}-zombie-{index}", level.iid),
        ));
        if !config.characters.is_empty() {
            let character = &config.characters[index % config.characters.len()];
            entities.push(entity(
                template,
                map_const::ENTITY_CHARACTER_SPAWN_LOCATION,
                cell,
                vec![
                    (
                        map_const::FIELD_CHARACTER_NAME,
                        FieldValue::String(Some(character.clone())),
                    ),
                    (
                        map_const::FIELD_TEAM_NAME,
                        FieldValue::String(Some("enemies".into())),
                    ),
                ],
                format!("{}-character-{index}", level.iid),
            ));
        }
    }

    for layer in level.layer_instances.iter_mut().flatten() {
        layer.c_wid = w;
        layer.c_hei = h;
        layer.auto_layer_tiles.clear();
        layer.grid_tiles.clear();
        layer.entity_instances.clear();
        if layer.layer_instance_type == Type::IntGrid {
            layer.int_grid_csv = if layer.identifier == LAYER_WALLS {
                csv.clone()
            } else {
                vec![0; (w * h) as usize]
            };
        }
        if layer.identifier == map_const::LAYER_ENTITY {
            layer.entity_instances = entities.clone();
        }
    }
    ldtk.levels = vec![level];
    ldtk
}

/// Instance d'entité du gabarit à la case LDtk `cell` (colonne, rangée depuis le haut).
fn entity(
    template: &LdtkJson,
    identifier: &str,
    cell: (i32, i32),
    fields: Vec<(&str, FieldValue)>,
    iid: String,
) -> EntityInstance {
    let def = template
        .defs
        .entities
        .iter()
        .find(|d| d.identifier == identifier)
        .unwrap_or_else(|| panic!("gabarit de caverne sans entité « {identifier} »"));
    let field_instances = fields
        .into_iter()
        .map(|(name, value)| {
            let field = def.field_defs.iter().find(|f| f.identifier == name);
            let editor = match &value {
                FieldValue::Int(Some(v)) => Some(RealEditorValue {
                    id: "V_Int".into(),
                    params: vec![serde_json::to_value(v).unwrap()],
                }),
                FieldValue::String(Some(v)) => Some(RealEditorValue {
                    id: "V_String".into(),
                    params: vec![serde_json::to_value(v).unwrap()],
                }),
                _ => None,
            };
            FieldInstance {
                identifier: name.to_string(),
                def_uid: field.map_or(0, |f| f.uid),
                field_instance_type: field
                    .map_or_else(|| "Int".to_string(), |f| f.field_definition_type.clone()),
                value,
                tile: None,
                real_editor_values: vec![editor],
            }
        })
        .collect();
    let px = IVec2::new(cell.0 * CELL_SIZE, cell.1 * CELL_SIZE);
    EntityInstance {
        identifier: identifier.into(),
        def_uid: def.uid,
        grid: IVec2::new(cell.0, cell.1),
        pivot: Vec2::new(def.pivot_x, def.pivot_y),
        tags: vec![],
        tile: def.tile_rect,
        smart_color: def.color,
        iid,
        width: def.width,
        height: def.height,
        field_instances,
        px,
        world_x: None,
        world_y: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::file::load_ldtk_json_file;
    use utils::get_crate_root_path;

    fn petite() -> CaveConfig {
        ron::from_str(
            &std::fs::read_to_string(get_crate_root_path!(
                "../../games/testbed/assets/caves/petite.ron"
            ))
            .unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn niveau_unique_intgrid_et_spawns() {
        let template = load_ldtk_json_file(get_crate_root_path!(
            "../../games/testbed/assets/caves/gabarit.ldtk"
        ))
        .unwrap();
        let config = petite();
        let ldtk = build_cave_ldtk(&template, "petite", 123456, &config);
        assert_eq!(ldtk.levels.len(), 1);
        let level = &ldtk.levels[0];
        assert_eq!((level.px_wid, level.px_hei), (48 * 16, 32 * 16));
        assert_eq!(level.world_y + level.px_hei, 0, "bas du niveau en y = 0");
        let grid = cave_grid(123456, &config);
        let walls = level
            .layer_instances
            .iter()
            .flatten()
            .find(|l| l.identifier == LAYER_WALLS)
            .unwrap();
        // Coin haut gauche LDtk = case (0, h - 1) de la grille : bordure
        assert_eq!(walls.int_grid_csv.len(), 48 * 32);
        assert_eq!(
            walls.int_grid_csv.iter().filter(|v| **v == 1).count(),
            grid.cells.iter().filter(|c| c.is_solid()).count()
        );
        let entities = &level
            .layer_instances
            .iter()
            .flatten()
            .find(|l| l.identifier == map_const::LAYER_ENTITY)
            .unwrap()
            .entity_instances;
        let players: Vec<_> = entities
            .iter()
            .filter(|e| e.identifier == map_const::ENTITY_PLAYER_SPAWN_LOCATION)
            .collect();
        assert_eq!(players.len(), CAVE_PLAYER_SPAWNS);
        for e in entities {
            let y = 31 - e.grid.y;
            assert_eq!(
                grid.get(e.grid.x, y),
                Some(CellKind::Floor),
                "{}",
                e.identifier
            );
        }
        assert_eq!(
            build_cave_ldtk(&template, "petite", 123456, &config).levels[0].iid,
            level.iid
        );
        // Deux cavernes de même graine : deux niveaux distincts.
        assert_eq!(level.iid, "cave-petite-123456");
        assert_ne!(
            build_cave_ldtk(&template, "autre", 123456, &config).levels[0].iid,
            level.iid
        );
    }
}
