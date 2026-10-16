use bevy::log::debug;
use std::rc::Rc;

use bevy::math::{IVec2, Vec2};
use bevy_ecs_ldtk::{
    ldtk::{FieldInstance, FieldValue, LdtkJson, Level, NeighbourLevel, RealEditorValue},
    EntityInstance,
};
use bevy_fixed::rng::RollbackRng;
use serde_json::Value;

use map::{
    game::entity::map::player_spawn::PlayerSpawnConfig,
    generation::{
        context::Side,
        entity::{
            character_spawn::CharacterSpawnConfig, door::DoorConfig, enemy_spawn::EnemySpawnConfig,
            location::EntityLocation, prop::PropConfig, soda_location::SodaLocationConfig,
            weapon_location::WeaponLocationConfig, window::WindowConfig,
        },
        room::{ConnectionTo, Room, RoomConnection},
        IMapGenerator,
    },
};

use crate::map_const::{self, LAYER_ENTITY};

#[derive(Debug, Clone)]
pub struct GeneratedRoom {
    level: Level,
    ldtk: Rc<LdtkJson>,
    /// Gabarit LDtk d'origine (`level.identifier` est remplacé par l'iid de la salle) :
    /// signature des cartes générées (D45, tests de diversité).
    pub template: String,
    /// Ouvertures du gabarit qu'aucune porte du plan ne prolonge : `(côté, première case, nombre de
    /// cases)`, lues dans les connexions finales de la salle (M3-T5, grille).
    unused_openings: Vec<(Side, usize, usize)>,
}

impl GeneratedRoom {
    /// Identifiant de la salle générée.
    pub fn iid(&self) -> &str {
        &self.level.iid
    }

    /// Identifiants des salles reliées à celle-ci par une porte du plan (arêtes de pose, boucles
    /// et portes de la salle secrète comprises ; contrôle des tests de grille, M3-T5).
    pub fn neighbour_iids(&self) -> Vec<String> {
        self.level
            .neighbours
            .iter()
            .map(|n| n.level_iid.clone())
            .collect()
    }

    /// Nombre de cases d'ouverture (`LevelConnection`) au bord de la salle que le mur ne ferme pas :
    /// trois par porte dans les gabarits de `isaac` (contrôle des tests de grille, M3-T5).
    pub fn open_cells(&self) -> usize {
        let Some(layers) = self.level.layer_instances.as_ref() else {
            return 0;
        };
        let find = |name: &str| layers.iter().find(|l| l.identifier == name);
        let (Some(openings), Some(walls)) = (find(map_const::LAYER_CONNECTION), find("Walls"))
        else {
            return 0;
        };
        openings
            .int_grid_csv
            .iter()
            .zip(&walls.int_grid_csv)
            .filter(|(opening, wall)| **opening == 1 && **wall == 0)
            .count()
    }

    /// Rectangle monde de la salle (`world_x`, `world_y`, largeur, hauteur) en pixels.
    pub fn world_rect(&self) -> (i32, i32, i32, i32) {
        (
            self.level.world_x,
            self.level.world_y,
            self.level.px_wid,
            self.level.px_hei,
        )
    }

    /// Entités de la couche `Entities` de la salle générée (M2-T11, vérifications des tests).
    pub fn entities(&self) -> impl Iterator<Item = &EntityInstance> {
        self.level
            .layer_instances
            .iter()
            .flatten()
            .filter(|layer| layer.identifier == LAYER_ENTITY)
            .flat_map(|layer| layer.entity_instances.iter())
    }

    pub fn create(ldtk_json: Rc<LdtkJson>, room: &Room) -> Self {
        let mut level = ldtk_json
            .levels
            .iter()
            .find(|item| item.identifier == room.level_def.level_id)
            .expect("failed to find level from original")
            .clone();

        level.iid = room.level_iid.clone();
        level.identifier = room.level_iid.clone();
        level.world_x = room.position.0;
        level.world_y = room.position.1;
        level.neighbours.clear();

        if !room.properties.is_empty() {
            for field in level.field_instances.iter_mut() {
                if let Some(value) = room.properties.get(field.identifier.as_str()) {
                    field.value = match value {
                        Value::Bool(b_value) => FieldValue::Bool(*b_value),
                        _ => FieldValue::String(None),
                    };
                    field.real_editor_values = vec![Some(RealEditorValue {
                        // TODO: do the correct mapping
                        id: "V_Bool".to_string(),
                        params: vec![value.clone()],
                    })]
                }
            }
        }
        level
            .layer_instances
            .as_mut()
            .unwrap()
            .iter_mut()
            .find(|x| x.identifier == LAYER_ENTITY)
            .unwrap()
            .entity_instances
            .clear();

        GeneratedRoom {
            level,
            ldtk: ldtk_json,
            template: room.level_def.level_id.clone(),
            unused_openings: room
                .level_def
                .connections
                .iter()
                .filter(|def| {
                    !matches!(
                        room.connections.get(def.index).and_then(|c| c.to.as_ref()),
                        Some(ConnectionTo::Room(_))
                    )
                })
                .map(|def| (def.side, def.starting_at, def.size))
                .collect(),
        }
    }
}

pub struct GeneratedMap {
    pub ldtk_json: Rc<LdtkJson>,
    pub generated_rooms: Vec<GeneratedRoom>,
}

pub fn _add_property_entity() {}

pub fn get_new_entity(
    rng: &mut RollbackRng,
    room: &GeneratedRoom,
    original_entity_identifier: &str,
    location: &EntityLocation,
    tile_size: (i32, i32),
    fields: Vec<(&str, FieldValue)>,
) -> EntityInstance {
    let entity = room
        .ldtk
        .defs
        .entities
        .iter()
        .find(|x| x.identifier == original_entity_identifier)
        .unwrap();

    let px = (
        location.position.0 * tile_size.0,
        location.position.1 * tile_size.1,
    );
    let world_px = (px.0 + room.level.world_x, px.1 + room.level.world_y);

    let identifiers = fields
        .iter()
        .map(|x| {
            // Try to find the field in the entity template
            let field_opt = entity.field_defs.iter().find(|fd| fd.identifier == x.0);

            let real_editor_value = match x.1.clone() {
                FieldValue::Int(v) => Some(("V_Int", serde_json::to_value(v).unwrap())),
                FieldValue::Bool(v) => Some(("V_Bool", serde_json::to_value(v).unwrap())),
                FieldValue::String(v) => v
                    .as_ref()
                    .map(|s| ("V_String", serde_json::to_value(s).unwrap())),
                _ => None,
            };

            let real_editor_value = real_editor_value.map(|v| RealEditorValue {
                id: v.0.to_string(),
                params: vec![v.1],
            });

            // If field exists in template, use its definition; otherwise create a synthetic one
            if let Some(field) = field_opt {
                FieldInstance {
                    identifier: field.identifier.clone(),
                    def_uid: field.uid,
                    field_instance_type: field.field_definition_type.clone(),
                    value: x.1.clone(),
                    tile: None,
                    real_editor_values: vec![real_editor_value],
                }
            } else {
                // Create a synthetic field instance for fields not in the template
                let field_type = match x.1 {
                    FieldValue::Int(_) => "Int",
                    FieldValue::Bool(_) => "Bool",
                    FieldValue::String(_) => "String",
                    _ => "String", // Default to String for unknown types
                };

                FieldInstance {
                    identifier: x.0.to_string(),
                    def_uid: 0, // Use 0 for synthetic fields
                    field_instance_type: field_type.to_string(),
                    value: x.1.clone(),
                    tile: None,
                    real_editor_values: vec![real_editor_value],
                }
            }
        })
        .collect();

    EntityInstance {
        identifier: original_entity_identifier.into(),
        def_uid: entity.uid,
        grid: IVec2::new(location.position.0, location.position.1),
        pivot: Vec2::new(entity.pivot_x, entity.pivot_y),
        tags: vec![],
        tile: entity.tile_rect,
        smart_color: entity.color,
        iid: rng.next_uuid(),
        width: location.size.0 * tile_size.0,
        height: location.size.1 * tile_size.1,
        field_instances: identifiers,
        px: IVec2::new(px.0, px.1),
        world_x: Some(world_px.0),
        world_y: Some(world_px.1),
    }
}

impl GeneratedMap {
    pub fn create(ldtk_json: LdtkJson) -> Self {
        GeneratedMap {
            ldtk_json: Rc::new(ldtk_json),
            generated_rooms: vec![],
        }
    }

    /// M2-T10 (mode `Floor`) : mure les ouvertures des salles qu'aucune salle voisine ne prolonge.
    /// Un étage de grammaire place des gabarits à plusieurs ouvertures dont certaines restent
    /// libres : sans cela, le joueur sortirait de l'étage. Une ouverture est une case de la couche
    /// `LevelConnection` au bord du niveau ; son côté est utilisé si la salle a un voisin de cette
    /// direction (`neighbours`). `Basic` ne l'appelle pas.
    pub fn close_unused_openings(&mut self) {
        for room in &mut self.generated_rooms {
            let used: Vec<String> = room
                .level
                .neighbours
                .iter()
                .map(|n| n.dir.clone())
                .collect();
            let Some(layers) = room.level.layer_instances.as_mut() else {
                continue;
            };
            let Some(opening_cells) = layers
                .iter()
                .find(|l| l.identifier == map_const::LAYER_CONNECTION)
                .map(|l| (l.c_wid, l.c_hei, l.int_grid_csv.clone()))
            else {
                continue;
            };
            let (w, h, openings) = opening_cells;
            let Some(walls) = layers.iter_mut().find(|l| l.identifier == "Walls") else {
                continue;
            };
            for (dir, cells) in [
                ("n", (0..w).map(|x| (x, 0)).collect::<Vec<_>>()),
                ("s", (0..w).map(|x| (x, h - 1)).collect()),
                ("w", (0..h).map(|y| (0, y)).collect()),
                ("e", (0..h).map(|y| (w - 1, y)).collect()),
            ] {
                if used.iter().any(|d| d == dir) {
                    continue;
                }
                for (x, y) in cells {
                    let i = (y * w + x) as usize;
                    if openings.get(i).copied().unwrap_or(0) == 1 {
                        walls.int_grid_csv[i] = 1;
                    }
                }
            }
        }
    }

    /// M3-T5 (disposition en grille) : mure exactement les ouvertures qu'aucune porte du plan ne
    /// prolonge. `close_unused_openings` raisonne par direction (un côté est « utilisé » dès qu'il a
    /// un voisin), insuffisant pour une grande salle qui a plusieurs ouvertures sur un côté dont
    /// seules certaines mènent quelque part. À appeler après `map_generation` : les connexions
    /// finales de chaque salle (portes de boucle et de salle secrète comprises) sont celles du plan.
    pub fn close_unused_openings_exact(&mut self) {
        for room in &mut self.generated_rooms {
            let Some(layers) = room.level.layer_instances.as_mut() else {
                continue;
            };
            let Some((w, h, openings)) = layers
                .iter()
                .find(|l| l.identifier == map_const::LAYER_CONNECTION)
                .map(|l| (l.c_wid, l.c_hei, l.int_grid_csv.clone()))
            else {
                continue;
            };
            let Some(walls) = layers.iter_mut().find(|l| l.identifier == "Walls") else {
                continue;
            };
            for (side, start, size) in &room.unused_openings {
                for t in *start as i32..(*start + *size) as i32 {
                    let (x, y) = match side {
                        Side::N => (t, 0),
                        Side::S => (t, h - 1),
                        Side::W => (0, t),
                        Side::E => (w - 1, t),
                    };
                    let i = (y * w + x) as usize;
                    if openings.get(i).copied().unwrap_or(0) == 1 {
                        walls.int_grid_csv[i] = 1;
                    }
                }
            }
        }
    }

    pub fn get_generated_map(&self) -> LdtkJson {
        let mut new_map: LdtkJson = (*self.ldtk_json).clone();

        new_map.levels = self
            .generated_rooms
            .iter()
            .enumerate()
            .map(|(i, x)| {
                let mut r = x.level.clone();
                r.identifier = format!("Level_{}", i);
                r
            })
            .collect();

        new_map
    }

    fn add_entity_to_level(
        &mut self,
        rng: &mut RollbackRng,
        location: &EntityLocation,
        entity_type: &str,
        fields: Vec<(&str, FieldValue)>,
    ) {
        let level = self
            .generated_rooms
            .iter_mut()
            .find(|x| x.level.iid == location.level_iid)
            .unwrap();

        let new_entity = get_new_entity(
            rng,
            level,
            entity_type,
            location,
            (
                self.ldtk_json.default_entity_width,
                self.ldtk_json.default_entity_height,
            ),
            fields,
        );

        level
            .level
            .layer_instances
            .as_mut()
            .unwrap()
            .iter_mut()
            .find(|x| x.identifier == map_const::LAYER_ENTITY)
            .unwrap()
            .entity_instances
            .push(new_entity);
    }
}

impl IMapGenerator for GeneratedMap {
    fn add_room(
        &mut self,
        _rng: &mut RollbackRng,
        room: &Room,
        connection_used: Option<&RoomConnection>,
        connected_to: Option<&RoomConnection>,
    ) {
        let mut generated_room = GeneratedRoom::create(self.ldtk_json.clone(), room);

        debug!(
            "adding room id={} type={:?} from_level={} position={} \n property={:?}",
            room.level_iid,
            room.level_def.level_type,
            room.level_def.level_id,
            room.position,
            room.properties
        );

        if let Some(connected_to) = connected_to {
            let connection_used = connection_used.unwrap();

            generated_room.level.neighbours.push(NeighbourLevel {
                level_iid: connected_to.level_iid.clone(),
                dir: connection_used.side.to_dir_str().into(),
                ..Default::default()
            });

            // find the other room and me as it's neighbours
            let linked_room = self
                .generated_rooms
                .iter_mut()
                .find(|r| r.level.iid == connected_to.level_iid)
                .unwrap();

            debug!("  connecting my side={:?} index={} with side={:?} index={} of room id={} from_level={} position={}x{}",
               connection_used.side, connection_used.index, connected_to.side, connected_to.index, connected_to.level_iid,
               connected_to.level_id, linked_room.level.world_x, linked_room.level.world_y,
            );

            linked_room.level.neighbours.push(NeighbourLevel {
                dir: connected_to.side.to_dir_str().into(),
                level_iid: room.level_iid.clone(),
                ..Default::default()
            })
        }

        self.generated_rooms.push(generated_room);
    }

    fn add_link(&mut self, a: &RoomConnection, b: &RoomConnection) {
        for (from, to) in [(a, b), (b, a)] {
            let room = self
                .generated_rooms
                .iter_mut()
                .find(|r| r.level.iid == from.level_iid)
                .expect("une porte de plus relie deux salles déjà ajoutées");
            room.level.neighbours.push(NeighbourLevel {
                level_iid: to.level_iid.clone(),
                dir: from.side.to_dir_str().into(),
                ..Default::default()
            });
        }
    }

    fn add_doors(&mut self, rng: &mut RollbackRng, doors: &Vec<(EntityLocation, DoorConfig)>) {
        debug!("Adding {} doors to map", doors.len());

        for (location, door) in doors.iter() {
            // Determine if door is horizontal or vertical based on size
            // If width > height, it's horizontal; otherwise vertical
            let door_type = if location.size.0 > location.size.1 {
                map_const::ENTITY_DOOR_HORIZONTAL_LOCATION
            } else {
                map_const::ENTITY_DOOR_VERTICAL_LOCATION
            };

            // Log door configuration
            let pairing_info = if let Some((paired_level, (px, py))) = &door.paired_door {
                format!(
                    "paired with door at ({}, {}) in level {}",
                    px, py, paired_level
                )
            } else {
                "unpaired".to_string()
            };

            debug!(
                "  Door at ({}, {}) in level {}: cost={}, electrify={}, interactable={}, {}",
                location.position.0,
                location.position.1,
                location.level_iid,
                door.cost,
                door.electrify,
                door.interactable,
                pairing_info
            );

            let mut fields = vec![
                (
                    map_const::FIELD_PRICE_NAME,
                    FieldValue::Int(Some(door.cost)),
                ),
                (
                    map_const::FIELD_ELECTRIFY_NAME,
                    FieldValue::Bool(door.electrify),
                ),
                (
                    map_const::FIELD_INTERACTABLE_NAME,
                    FieldValue::Bool(door.interactable),
                ),
            ];

            // Add paired door information if it exists
            if let Some((paired_level_iid, (paired_x, paired_y))) = &door.paired_door {
                fields.push((
                    map_const::FIELD_PAIRED_DOOR_X_NAME,
                    FieldValue::Int(Some(*paired_x)),
                ));
                fields.push((
                    map_const::FIELD_PAIRED_DOOR_Y_NAME,
                    FieldValue::Int(Some(*paired_y)),
                ));
                fields.push((
                    map_const::FIELD_PAIRED_DOOR_LEVEL_NAME,
                    FieldValue::String(Some(paired_level_iid.clone())),
                ));
            }

            self.add_entity_to_level(rng, location, door_type, fields);
        }

        debug!("Door generation complete\n");
    }

    fn add_windows(
        &mut self,
        rng: &mut RollbackRng,
        windows: &Vec<(EntityLocation, WindowConfig)>,
    ) {
        for (location, _) in windows.iter() {
            // Determine if window is horizontal or vertical based on size
            // If width > height, it's horizontal; otherwise vertical
            let window_type = if location.size.0 > location.size.1 {
                map_const::ENTITY_WINDOW_HORIZONTAL_LOCATION
            } else {
                map_const::ENTITY_WINDOW_VERTICAL_LOCATION
            };

            self.add_entity_to_level(rng, location, window_type, vec![]);
        }
    }

    fn add_player_spawns(
        &mut self,
        rng: &mut RollbackRng,
        player_spawns: &Vec<(EntityLocation, PlayerSpawnConfig)>,
    ) {
        for (location, spawn) in player_spawns.iter() {
            self.add_entity_to_level(
                rng,
                location,
                map_const::ENTITY_PLAYER_SPAWN_LOCATION,
                vec![(
                    map_const::FIELD_PLAYER_SPAWN_INDEX_NAME,
                    FieldValue::Int(Some(spawn.index as i32)),
                )],
            );
        }
    }

    fn add_enemy_spawns(
        &mut self,
        rng: &mut RollbackRng,
        enemy_spawns: &Vec<(EntityLocation, EnemySpawnConfig)>,
    ) {
        debug!("Adding {} enemy spawns to map", enemy_spawns.len());

        for (location, _spawn) in enemy_spawns.iter() {
            debug!(
                "  Enemy spawn at ({}, {}) in level {}",
                location.position.0, location.position.1, location.level_iid
            );

            self.add_entity_to_level(
                rng,
                location,
                map_const::ENTITY_ZOMBIE_SPAWN_LOCATION,
                vec![],
            );
        }

        debug!("Enemy spawn generation complete\n");
    }

    /// T2.9 (testbed) : contrairement aux portes/fenêtres (config recalculée ou vide),
    /// `character`/`team` sont des valeurs d'auteur : réécrites telles quelles.
    fn add_character_spawns(
        &mut self,
        rng: &mut RollbackRng,
        character_spawns: &Vec<(EntityLocation, CharacterSpawnConfig)>,
    ) {
        for (location, config) in character_spawns.iter() {
            let mut fields = vec![
                (
                    map_const::FIELD_CHARACTER_NAME,
                    FieldValue::String(Some(config.character.clone())),
                ),
                (
                    map_const::FIELD_TEAM_NAME,
                    FieldValue::String(config.team.clone()),
                ),
            ];
            // T1.5 : seulement si rempli (les cartes d'avant T1.5 gardent leurs entités).
            if let Some(variant) = &config.variant {
                fields.push((
                    map_const::FIELD_VARIANT_NAME,
                    FieldValue::String(Some(variant.clone())),
                ));
            }
            self.add_entity_to_level(
                rng,
                location,
                map_const::ENTITY_CHARACTER_SPAWN_LOCATION,
                fields,
            );
        }
    }

    /// T2.3, chantier C5 v1 : `weapon`/`price` sont des valeurs d'auteur, réécrites telles
    /// quelles (voir la doc de [`Self::add_character_spawns`]).
    fn add_weapon_locations(
        &mut self,
        rng: &mut RollbackRng,
        weapon_locations: &Vec<(EntityLocation, WeaponLocationConfig)>,
    ) {
        for (location, config) in weapon_locations.iter() {
            self.add_entity_to_level(
                rng,
                location,
                map_const::ENTITY_WEAPON_LOCATION,
                vec![
                    (
                        map_const::FIELD_WEAPON_NAME,
                        FieldValue::String(Some(config.weapon.clone())),
                    ),
                    (
                        map_const::FIELD_PRICE_NAME,
                        FieldValue::Int(Some(config.price as i32)),
                    ),
                ],
            );
        }
    }

    /// T2.3, chantier C5 v1 : `perk` est une valeur d'auteur, réécrite telle quelle.
    fn add_soda_locations(
        &mut self,
        rng: &mut RollbackRng,
        soda_locations: &Vec<(EntityLocation, SodaLocationConfig)>,
    ) {
        for (location, config) in soda_locations.iter() {
            self.add_entity_to_level(
                rng,
                location,
                map_const::ENTITY_SODA_LOCATION,
                vec![(
                    map_const::FIELD_PERK_NAME,
                    FieldValue::String(Some(config.perk.clone())),
                )],
            );
        }
    }

    /// M2-T11 : `prop` est une valeur d'auteur, réécrite telle quelle (champ seulement s'il est
    /// rempli : les cartes sans champ gardent leur entité nue).
    fn add_props(&mut self, rng: &mut RollbackRng, props: &Vec<(EntityLocation, PropConfig)>) {
        for (location, config) in props.iter() {
            let fields = if config.prop.is_empty() {
                vec![]
            } else {
                vec![(
                    map_const::FIELD_PROP_NAME,
                    FieldValue::String(Some(config.prop.clone())),
                )]
            };
            self.add_entity_to_level(rng, location, &config.entity, fields);
        }
    }
}
