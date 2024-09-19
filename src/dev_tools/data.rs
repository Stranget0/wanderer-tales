use crate::prelude::*;
use bevy::render::mesh::VertexAttributeValues;

#[derive(Component, Clone)]
pub struct DebugNormals(pub Vec<(Vec3, Vec3)>);

impl DebugNormals {
    pub fn from_mesh(mesh: &Mesh) -> DebugNormals {
        let mut debug_normals = Vec::new();
        if let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        {
            if let Some(VertexAttributeValues::Float32x3(normals)) =
                mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
            {
                for (i, position) in positions.iter().enumerate() {
                    let pos = Vec3::new(position[0], position[1], position[2]);
                    let normal = Vec3::new(normals[i][0], normals[i][1], normals[i][2]);

                    debug_normals.push((pos, pos + normal));
                }
            }
        }
        DebugNormals(debug_normals)
    }

    pub fn apply_transform(&mut self, transform: &Transform) {
        for (pos, normal) in &mut self.0 {
            *pos = transform.transform_point(*pos);
            *normal = transform.transform_point(*normal);
        }
    }
}
