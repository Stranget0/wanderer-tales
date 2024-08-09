use bevy::{
    prelude::*,
    render::{mesh::*, render_asset::RenderAssetUsages},
};

use crate::screen::Screen;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Playing), spawn_map);
}

fn spawn_map(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn((
        StateScoped(Screen::Playing),
        PbrBundle {
            mesh: asset_server.add(create_subdivided_plane(10, 10.0)),
            material: asset_server.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.7, 0.6),
                ..Default::default()
            }),
            ..Default::default()
        },
    ));
}

fn create_subdivided_plane(subdivisions: u32, size: f32) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );

    let num_vertices_per_side = subdivisions + 1;
    let num_vertices = (num_vertices_per_side * num_vertices_per_side) as usize;
    let num_indices = (subdivisions * subdivisions * 6) as usize;

    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    let mut uvs = Vec::with_capacity(num_vertices);
    let mut indices = Vec::with_capacity(num_indices);

    for y in 0..=subdivisions {
        for x in 0..=subdivisions {
            let fx = x as f32 / subdivisions as f32 * size;
            let fy = y as f32 / subdivisions as f32 * size;

            positions.push([fx - 0.5, 0.0, fy - 0.5]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([fx, fy]);
        }
    }

    for y in 0..subdivisions {
        for x in 0..subdivisions {
            let i = y * (subdivisions + 1) + x;
            indices.push(i);
            indices.push(i + subdivisions + 1);
            indices.push(i + 1);

            indices.push(i + 1);
            indices.push(i + subdivisions + 1);
            indices.push(i + subdivisions + 2);
        }
    }

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));
    mesh.insert_indices(Indices::U32(indices));

    mesh
}
