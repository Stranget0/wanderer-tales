use bevy::{
    prelude::*,
    render::{mesh::*, render_asset::*},
};

pub fn create_subdivided_plane<F: Fn(f32, f32) -> (f32, [f32; 3])>(
    subdivisions: u32,
    size: f32,
    height_function: F,
) -> Mesh {
    let subdivisions_less = subdivisions - 1;
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );

    let num_vertices_per_side = subdivisions_less + 1;
    let num_vertices = (num_vertices_per_side * num_vertices_per_side) as usize;
    let num_indices = (subdivisions_less * subdivisions_less * 6) as usize;

    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    let mut uvs = Vec::with_capacity(num_vertices);
    let mut indices = Vec::with_capacity(num_indices);

    for y in 0..=subdivisions_less {
        for x in 0..=subdivisions_less {
            let u = x as f32 / subdivisions_less as f32;
            let v = y as f32 / subdivisions_less as f32;
            let x = (u - 0.5) * size;
            let z = (v - 0.5) * size;

            let (y, normal) = height_function(x, z);

            positions.push([x, y, z]);
            normals.push(normal);
            uvs.push([u, v]);
        }
    }

    for y in 0..subdivisions_less {
        for x in 0..subdivisions_less {
            let i = y * (subdivisions_less + 1) + x;
            indices.push(i);
            indices.push(i + subdivisions_less + 1);
            indices.push(i + 1);

            indices.push(i + 1);
            indices.push(i + subdivisions_less + 1);
            indices.push(i + subdivisions_less + 2);
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
