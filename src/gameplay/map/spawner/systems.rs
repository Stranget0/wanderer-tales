use bevy::{
    hierarchy::{BuildChildren, DespawnRecursiveExt},
    prelude::*,
};
use itertools::Itertools;
use noise::core::perlin::perlin_2d;
use rand::Rng;

use crate::{
    gameplay::{
        map::{
            components::SourceLayout,
            renderer::components::{MeshType, RenderGroup},
            utils::{
                hex_layout::{get_corner_heights, get_hex_corner_3d, get_hex_corner_z},
                hex_map_item::{Biome, Height, HexMapTileBundle, TileHeight},
                hex_vector::{iterators::HexVectorSpiral, HexVector},
                lexigraphical_cycle::LexigraphicalCycle,
            },
        },
        player::{
            components::{HexPosition, HexPositionFractional, PlayerRoot, Rotation},
            events::{CharacterMovedEvent, PlayerWithSightSpawnedEvent},
        },
    },
    utils::positive_modulo,
};

use super::{
    events::MapAddEvent,
    resources::{HexToMapSourceEntity, SeedTable},
    MapSubEvent,
};

const MAP_RENDER_GROUPS: [RenderGroup; 2] = [RenderGroup::Gameplay3D, RenderGroup::PreviewMap2D];

pub fn spawn_map_data(
    mut commands: Commands,
    mut hex_to_map_source_entity: ResMut<HexToMapSourceEntity>,
    mut render_event: EventWriter<MapAddEvent>,
    mut character_moved_event: EventReader<CharacterMovedEvent>,
    layout_query: Query<Entity, With<SourceLayout>>,
    seed_table: Res<SeedTable>,
) {
    for e in character_moved_event.read() {
        let origin: HexVector = (&e.pos.0 - &e.delta_pos).into();
        let new_origin: HexVector = (&e.pos.0).into();
        let distance = origin.distance_to(&new_origin);
        if is_moved_event_irrelevant(e) || distance < 1 {
            continue;
        }

        let sight = e.sight.as_ref().unwrap().0;
        let from: u16 = sight;
        let to: u16 = sight.saturating_sub(distance);

        for layout_entity in layout_query.iter() {
            let mut additive_entities: Vec<Entity> = vec![];
            let mut additive_hexes: Vec<(Entity, HexMapTileBundle)> = vec![];

            for hex in HexVectorSpiral::new(&new_origin, from, to) {
                if hex_to_map_source_entity.0.contains_key(&hex) {
                    continue;
                }

                let bundle = create_map_tile_bundle(&hex, &seed_table);
                let hex_entity = commands
                    .spawn((bundle.clone(), Name::new("TileSource")))
                    .id();
                hex_to_map_source_entity.0.insert(hex, hex_entity);
                additive_entities.push(hex_entity);
                additive_hexes.push((hex_entity, bundle));
            }

            commands
                .entity(layout_entity)
                .push_children(&additive_entities);

            render_event.send(MapAddEvent {
                source_items: additive_hexes,
                render_groups: MAP_RENDER_GROUPS.to_vec(),
            });
        }
    }
}

pub fn despawn_map_data(
    mut commands: Commands,
    mut hex_to_map_source_entity: ResMut<HexToMapSourceEntity>,
    mut render_sub_event: EventWriter<MapSubEvent>,
    mut character_moved_event: EventReader<CharacterMovedEvent>,
    layout_query: Query<Entity, With<SourceLayout>>,
) {
    for e in character_moved_event.read() {
        let origin: HexVector = (&e.pos.0 - &e.delta_pos).into();
        let new_origin: HexVector = (&e.pos.0).into();
        let distance = origin.distance_to(&new_origin);
        if is_moved_event_irrelevant(e) || distance < 1 {
            continue;
        }

        let sight = e.sight.as_ref().unwrap().0;
        let from: u16 = sight;
        let to: u16 = sight.saturating_sub(distance);

        for layout_entity in layout_query.iter() {
            let mut substractive_entities: Vec<Entity> = vec![];

            for hex in HexVectorSpiral::new(&origin, from, to) {
                if hex.distance_to(&new_origin) > sight {
                    let value = hex_to_map_source_entity.0.remove(&hex);
                    if let Some(hex_entity) = value {
                        substractive_entities.push(hex_entity);

                        commands
                            .entity(layout_entity)
                            .remove_children(&[hex_entity]);
                    }
                }
            }

            for e in &substractive_entities {
                commands.entity(*e).despawn_recursive();
            }

            render_sub_event.send(MapSubEvent {
                source_items: substractive_entities,
                render_groups: MAP_RENDER_GROUPS.to_vec(),
            });
        }
    }
}

pub fn init_map_data(
    mut commands: Commands,
    mut hex_to_map_source_entity: ResMut<HexToMapSourceEntity>,
    mut render_event: EventWriter<MapAddEvent>,
    mut character_spawned: EventReader<PlayerWithSightSpawnedEvent>,
    layout_query: Query<Entity, With<SourceLayout>>,
    seed_table: Res<SeedTable>,
) {
    for e in character_spawned.read() {
        let origin: HexVector = (&e.pos.0).into();
        let sight = e.sight.0;

        let from: u16 = 0;
        let to: u16 = sight;

        for layout_entity in layout_query.iter() {
            let mut additive_entities: Vec<Entity> = vec![];
            let mut additive_hexes: Vec<(Entity, HexMapTileBundle)> = vec![];

            for hex in HexVectorSpiral::new(&origin, from, to) {
                if hex_to_map_source_entity.0.contains_key(&hex) {
                    continue;
                }
                let bundle = create_map_tile_bundle(&hex, &seed_table);

                let hex_entity = commands
                    .spawn((bundle.clone(), Name::from("TileSource")))
                    .id();
                hex_to_map_source_entity.0.insert(hex, hex_entity);
                additive_entities.push(hex_entity);
                additive_hexes.push((hex_entity, bundle));
            }

            commands
                .entity(layout_entity)
                .push_children(&additive_entities);

            render_event.send(MapAddEvent {
                source_items: additive_hexes,
                render_groups: MAP_RENDER_GROUPS.to_vec(),
            });
        }
    }
}

pub fn clear_map_data(mut commands: Commands, layout_query: Query<Entity, With<SourceLayout>>) {
    for e in layout_query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn add_hex_tile_offsets(
    mut commands: Commands,
    tile_query: Query<(Entity, &Height, &HexPosition), With<Biome>>,
    player_query: Query<&HexPositionFractional, With<PlayerRoot>>,
    hex_to_map_source_entity: Res<HexToMapSourceEntity>,
    mut has_logged: Local<bool>,
) {
    let player_pos_o = player_query.get_single();
    if player_pos_o.is_err() {
        return;
    }

    let player_pos = &player_pos_o.unwrap().0;
    for (entity, height, pos) in tile_query.iter() {
        let mut sibling_data = Vec::with_capacity(6);
        for i in 0..6 {
            let hex = pos.0.get_sibling(i);

            let height_option = hex_to_map_source_entity
                .0
                .get(&hex)
                .and_then(|entity| tile_query.get(*entity).ok())
                .map(|(_, h, _)| h);

            if let Some(h) = height_option {
                sibling_data.push((hex, h));
            } else {
                break;
            };
        }

        if sibling_data.len() == 6 {
            let height_diffs: [i8; 6] = sibling_data
                .iter()
                .map(|h| (h.1 .0 as i16 - height.0 as i16) as i8)
                .collect_vec()
                .try_into()
                .unwrap();

            if pos.0 == HexVector::from(player_pos) && !*has_logged {
                *has_logged = true;

                let corners = (0..6)
                    .map(|index| get_hex_corner_z(get_corner_heights(&height_diffs, index)))
                    .collect_vec();

                let debug = (0..6)
                    .map(|i| {
                        let [a, b] = get_corner_heights(&height_diffs, i);
                        let str = format!(
                            "{:?} ({:?}) [{:?} {:?} {:?}] -> {:.1}",
                            sibling_data[i].1,
                            sibling_data[i].0,
                            height.0,
                            height.0 as i16 + *a as i16,
                            height.0 as i16 + *b as i16,
                            height.0 as f32 + corners[i]
                        );

                        str
                    })
                    .collect_vec();

                println!("{:#?}", debug);
                // .reduce(|acc, curr| {
                //     let str = format!("{}\n{}", acc, curr);
                //     // &str[..]
                // })
                // .unwrap();
            }
            // let minimal_cycle =
            //     LexigraphicalCycle::shiloah_minimal_rotation(&height_diffs.try_into().unwrap());
            let mesh_rotation =
                // Rotation(UP * ((minimal_cycle.rotation + 5) as f32 * 60.0).to_radians());
                Rotation(Vec3::ZERO);
            // let mesh_type = MeshType::HexMapTile(minimal_cycle.cycle);
            let mesh_type = MeshType::HexMapTile(height_diffs);

            commands.entity(entity).insert((mesh_type, mesh_rotation));
        }
    }
}

fn get_height_offset(hex: &HexVector, seed_table: &Res<SeedTable>) -> f32 {
    let compounds = [
        (1.0, noise(1., hex, seed_table)),
        // (0.3, noise(3., hex, seed_table)),
        // (noise(0.1, hex, seed_table)),
    ];
    let mut h: f64 = 0.0;
    let mut max = 0.0;
    for c in compounds {
        max += c.0;
        h += c.1 * c.0;
    }

    // normalize to [-1., 1.]
    h /= max;
    h as f32
}

fn get_height_midpoint(hex: &HexVector, seed_table: &Res<SeedTable>) -> u8 {
    let compounds = [
        (1.0, 0.0),
        (0.3, noise(1., hex, seed_table)),
        // (0.3, noise(3., hex, seed_table)),
        // (noise(0.1, hex, seed_table)),
    ];
    let mut h: f64 = 0.0;
    let mut max = 0.0;
    for c in compounds {
        max += c.0;
        h += c.1 * c.0;
    }

    // normalize to [0., 1.]
    h = ((h / max) + 1.0) / 2.0;

    let h_u8: u8 = (h * 255.0) as u8;

    h_u8
}

fn noise(zoom: f64, hex: &HexVector, seed_table: &Res<SeedTable>) -> f64 {
    let from = f64::from(i16::MAX) * 0.001 / zoom;
    let point = [f64::from(hex.0) / from, f64::from(hex.1) / from];

    perlin_2d(point, &seed_table.table)
}

fn get_biome(_hex: &HexVector) -> Biome {
    let mut rng = rand::thread_rng();
    let x: f32 = rng.gen();
    match x.round() as u8 {
        0 => Biome::Forest,
        _ => Biome::Grass,
    }
}

fn is_moved_event_irrelevant(e: &CharacterMovedEvent) -> bool {
    !e.is_player_controllable || e.sight.is_none()
}

fn create_map_tile_bundle(hex: &HexVector, seed_table: &Res<SeedTable>) -> HexMapTileBundle {
    let height = TileHeight {
        midpoint: get_height_midpoint(hex, seed_table),
        offset: get_height_offset(hex, seed_table),
    };
    let material_type = height.get_material();

    HexMapTileBundle {
        biome: get_biome(hex),
        height: Height(height.get_height()),
        tile_height: height,
        pos: HexPosition(hex.clone()),
        material_type,
    }
}
