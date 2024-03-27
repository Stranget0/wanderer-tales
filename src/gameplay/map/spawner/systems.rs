use self::hex_vector::iterators::HexVectorSpiral;
use super::resources::{HexToMapSourceEntity, SeedTable};
use crate::gameplay::components::*;
use crate::gameplay::map::utils::*;
use crate::gameplay::player::events::CharacterMovedEvent;
use crate::gameplay::{
    map::{components::SourceLayout, renderer::components::MeshType},
    player::components::Sight,
};
use crate::utils::UP;
use bevy::{
    hierarchy::{BuildChildren, DespawnRecursiveExt},
    prelude::*,
};
use itertools::Itertools;
use noise::core::perlin::perlin_2d;
use rand::Rng;

pub fn spawn_map_data(
    mut commands: Commands,
    mut hex_to_map_source_entity: ResMut<HexToMapSourceEntity>,
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
            let iter = HexVectorSpiral::new(&new_origin, from, to);
            spawn_map_part(
                &mut commands,
                &mut hex_to_map_source_entity,
                layout_entity,
                &seed_table,
                iter,
            )
        }
    }
}

pub fn despawn_map_data(
    mut commands: Commands,
    mut hex_to_map_source_entity: ResMut<HexToMapSourceEntity>,
    mut character_moved_event: EventReader<CharacterMovedEvent>,
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

        let iter = HexVectorSpiral::new(&origin, from, to);
        remove_map_part(&mut commands, iter, &mut hex_to_map_source_entity, |hex| {
            hex.distance_to(&new_origin) > sight
        });
    }
}

pub fn fill_map_data_on_sight(
    mut commands: Commands,
    mut hex_to_map_source_entity: ResMut<HexToMapSourceEntity>,
    sight_spawned: Query<(&HexPositionFractional, &Sight), Or<(Added<Sight>, Changed<Sight>)>>,
    layout_query: Query<Entity, With<SourceLayout>>,
    seed_table: Res<SeedTable>,
) {
    for (pos, sight) in sight_spawned.iter() {
        let origin: HexVector = (&pos.0).into();

        let from: u16 = 0;
        let to: u16 = sight.0;

        for layout_entity in layout_query.iter() {
            let hexes = HexVectorSpiral::new(&origin, from, to);

            spawn_map_part(
                &mut commands,
                &mut hex_to_map_source_entity,
                layout_entity,
                &seed_table,
                hexes,
            );
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
    mut height_changed_query: ParamSet<(
        Query<&HexPosition, Or<(Added<Height>, Changed<Height>)>>,
        Query<(&Height, &HexPosition)>,
        Query<&Height>,
    )>,
    hex_to_map_source_entity: Res<HexToMapSourceEntity>,
) {
    // First get hexes that spawner / changed their height.
    let mut hexes_to_iter = Vec::with_capacity(64);
    for changed_hex in height_changed_query.p0().iter() {
        // Their siblings need to be recalculated
        for i in 0..6 {
            if let Some(entity) = hex_to_map_source_entity
                .0
                .get(&changed_hex.0.get_sibling(i))
            {
                hexes_to_iter.push(entity);
            }
        }
    }

    // Map this into data
    let mut data_to_iter = Vec::with_capacity(64);
    for entity in hexes_to_iter {
        if let Ok((height, pos)) = height_changed_query.p1().get(*entity) {
            data_to_iter.push((*entity, pos.to_owned(), height.to_owned()));
        }
    }

    // Calculate Tile Differences
    for (entity, pos, height) in data_to_iter {
        let mut sibling_data = Vec::with_capacity(6);
        // Try to get differences
        for i in 0..6 {
            let hex = pos.0.get_sibling(i);

            let sibling_option = hex_to_map_source_entity.0.get(&hex).copied();

            if let Some(entity) = sibling_option {
                if let Ok(h) = height_changed_query.p2().get(entity) {
                    sibling_data.push((hex, h.0));
                    continue;
                }
            };
            break;
        }

        // Calculate minimal rotated mesh type if fine
        if sibling_data.len() == 6 {
            let height_diffs: [i8; 6] = sibling_data
                .iter()
                .map(|h| (h.1 as i16 - height.0 as i16) as i8)
                .collect_vec()
                .try_into()
                .unwrap();

            let minimal_cycle = LexigraphicalCycle::shiloah_minimal_rotation(&height_diffs);
            let mesh_rotation =
                Rotation::from_vec(UP * (minimal_cycle.rotation as f32 * 60.0).to_radians());
            let mesh_type = MeshType::HexMapTile(minimal_cycle.cycle);

            commands.entity(entity).insert((mesh_type, mesh_rotation));
        } else {
            // info!("FAILED TILE OFFSETS {:?}", pos);
        }
    }
}

fn spawn_map_part(
    commands: &mut Commands,
    hex_to_map_source_entity: &mut ResMut<HexToMapSourceEntity>,
    layout_entity: Entity,
    seed_table: &Res<SeedTable>,
    hexes: HexVectorSpiral,
) {
    let mut additive_entities: Vec<Entity> = vec![];

    for hex in hexes {
        if hex_to_map_source_entity.0.contains_key(&hex) {
            continue;
        }
        let bundle = create_map_tile_bundle(&hex, seed_table);

        let hex_entity = commands
            .spawn((bundle.clone(), Name::from("TileSource")))
            .id();
        hex_to_map_source_entity.0.insert(hex, hex_entity);
        additive_entities.push(hex_entity);
    }

    commands
        .entity(layout_entity)
        .push_children(&additive_entities);
}

fn remove_map_part<F: Fn(&HexVector) -> bool>(
    commands: &mut Commands<'_, '_>,
    iter: HexVectorSpiral<'_>,
    hex_to_map_source_entity: &mut ResMut<'_, HexToMapSourceEntity>,
    check: F,
) {
    let mut substractive_entities: Vec<Entity> = vec![];

    for hex in iter {
        if check(&hex) {
            let value = hex_to_map_source_entity.0.remove(&hex);
            if let Some(hex_entity) = value {
                substractive_entities.push(hex_entity);

                commands.entity(hex_entity).remove_parent();
            }
        }
    }

    for e in &substractive_entities {
        commands.entity(*e).despawn_recursive();
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
        mesh_type: MeshType::Debug,
        material_type,
    }
}
