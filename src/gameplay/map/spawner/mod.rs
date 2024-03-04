use bevy::{
    hierarchy::{BuildChildren, DespawnRecursiveExt},
    math::vec2,
    prelude::{SpatialBundle, *},
    utils::HashMap,
};
use rand::Rng;

use super::{
    events::{MapAddEvent, MoveSightEvent},
    utils::{
        hex_layout::HexLayout,
        hex_map_item::{Biome, Height, HexMapItemBundle},
        hex_vector::{iterators::HexVectorSpiral, HexVector},
        layout_orientation::POINTY_TOP_ORIENTATION,
    },
};

#[derive(Resource)]
pub struct MapData {
    pub hex_to_entity: HashMap<HexVector, Entity>,
}

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
        for hex in HexVectorSpiral::new(&origin, from, to) {
            if map_data.hex_to_entity.contains_key(&hex) {
                continue;
            }

            let bundle = HexMapItemBundle {
                biome: get_biome(&hex),
                height: get_height(&hex),
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

pub fn despawn_map_data(mut commands: Commands, layout: Query<Entity, With<HexLayout>>) {
    let controls = commands.entity(layout.single());
    controls.despawn_recursive();
}

fn get_height(_hex: &HexVector) -> Height {
    let mut rng = rand::thread_rng();
    let x: f32 = rng.gen();
    Height((x * 50.0) as u8)
}

fn get_biome(_hex: &HexVector) -> Biome {
    let mut rng = rand::thread_rng();
    let x: f32 = rng.gen();
    match x.round() as u8 {
        0 => Biome::Forest,
        _ => Biome::Grass,
    }
}
