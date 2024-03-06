use bevy::{
    hierarchy::{BuildChildren, DespawnRecursiveExt},
    math::vec2,
    prelude::{SpatialBundle, *},
};
use noise::core::perlin::perlin_2d;
use rand::Rng;

use self::resources::{MapData, SeedTable};

use super::{
    events::{MapAddEvent, MoveSightEvent},
    utils::{
        hex_layout::HexLayout,
        hex_map_item::{Biome, Height, HexMapItemBundle},
        hex_vector::{iterators::HexVectorSpiral, HexVector},
        layout_orientation::POINTY_TOP_ORIENTATION,
    },
};

pub mod resources;

pub fn spawn_layout(mut commands: Commands) {
    let layout: HexLayout = HexLayout {
        orientation: POINTY_TOP_ORIENTATION,
        size: vec2(5.0, 5.0),
        origin: vec2(0.0, 0.0),
    };

    commands.spawn((
        layout,
        SpatialBundle {
            ..Default::default()
        },
    ));
}

pub fn spawn_map_data(
    mut commands: Commands,
    layout_query: Query<(Entity, &HexLayout)>,
    seed_table: Res<SeedTable>,
    mut map_data: ResMut<MapData>,
    mut origin_event: EventReader<MoveSightEvent>,
    mut render_event: EventWriter<MapAddEvent>,
) {
    let layout_res = layout_query.get_single();
    if layout_res.is_err() {
        return;
    };
    let (layout_entity, layout) = layout_res.unwrap();

    for move_map_event in origin_event.read() {
        let origin = origin_from_event(layout, move_map_event);
        let new_origin: HexVector = new_origin_from_event(layout, move_map_event);
        let distance = origin.distance_to(&new_origin);

        let is_unnecessary = get_is_unnecessary(distance, move_map_event);
        if is_unnecessary {
            continue;
        }

        let mut additive_hexes: Vec<Entity> = vec![];

        let from: u16 = move_map_event.sight;
        let to: u16 = move_map_event.sight - distance;
        for hex in HexVectorSpiral::new(&new_origin, from, to) {
            if map_data.hex_to_entity.contains_key(&hex) {
                continue;
            }

            let bundle = HexMapItemBundle {
                biome: get_biome(&hex),
                height: Height {
                    midpoint: get_height_midpoint(&hex, &seed_table),
                    offset: get_height_offset(&hex, &seed_table),
                },
                pos: hex.clone(),
            };

            let hex_entity = commands.spawn(bundle).id();

            map_data.hex_to_entity.insert(hex, hex_entity);

            let mut layout_controls = commands.entity(layout_entity);
            layout_controls.add_child(hex_entity);
            additive_hexes.push(hex_entity);
        }

        render_event.send(MapAddEvent(additive_hexes));
    }
}

pub fn despawn_map_data(
    mut commands: Commands,
    layout_query: Query<(Entity, &HexLayout)>,
    mut map_data: ResMut<MapData>,
    mut origin_event: EventReader<MoveSightEvent>,
) {
    let layout_res = layout_query.get_single();
    if layout_res.is_err() {
        return;
    };
    let (layout_entity, layout) = layout_res.unwrap();

    for move_map_event in origin_event.read() {
        let origin = origin_from_event(layout, move_map_event);
        let new_origin: HexVector = new_origin_from_event(layout, move_map_event);
        let distance = origin.distance_to(&new_origin);

        if distance < 1 {
            continue;
        }

        let mut substractive_hexes: Vec<Entity> = vec![];

        let from: u16 = move_map_event.sight;
        let to: u16 = move_map_event.sight - distance;
        for hex in HexVectorSpiral::new(&origin, from, to) {
            if hex.distance_to(&new_origin) > move_map_event.sight {
                let value = map_data.hex_to_entity.remove(&hex);
                if let Some(hex_entity) = value {
                    substractive_hexes.push(hex_entity);
                }
            }
        }
        commands
            .entity(layout_entity)
            .remove_children(&substractive_hexes);

        for e in substractive_hexes {
            commands.entity(e).despawn();
        }
    }
}

fn get_is_unnecessary(distance: u16, move_map_event: &MoveSightEvent) -> bool {
    distance < 1 && !move_map_event.force_render
}

fn new_origin_from_event(layout: &HexLayout, move_map_event: &MoveSightEvent) -> HexVector {
    layout.pixel_to_hex(move_map_event.pos).into()
}

fn origin_from_event(layout: &HexLayout, move_map_event: &MoveSightEvent) -> HexVector {
    let origin: HexVector = layout
        .pixel_to_hex(move_map_event.pos - move_map_event.delta_pos)
        .into();
    origin
}

pub fn clear_map_data(mut commands: Commands, layout: Query<Entity, With<HexLayout>>) {
    let controls = commands.entity(layout.single());
    controls.despawn_recursive();
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

    // ((res + 1. / 2.) * f64::from(u8::MAX)) as u8
}

fn get_biome(_hex: &HexVector) -> Biome {
    let mut rng = rand::thread_rng();
    let x: f32 = rng.gen();
    match x.round() as u8 {
        0 => Biome::Forest,
        _ => Biome::Grass,
    }
}
