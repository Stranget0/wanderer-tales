use bevy::prelude::*;

use crate::gameplay::map::renderer::renderers::RenderMapApi;
use crate::gameplay::map::renderer::state::RendererState;
use crate::utils::*;

#[derive(Component)]
pub struct LocalGizmoSource;

pub fn draw_local_gizmos<R: Component + RenderMapApi>(
    mut gizmos: Gizmos,
    targets_query: Query<Entity, With<LocalGizmoSource>>,
    transform_query: Query<&Transform>,
    renderer_state: Res<State<RendererState>>,
    renderer_query: Query<&R>,
) {
    for source_entity in targets_query.iter() {
        for renderer in renderer_query.iter() {
            if let Some(t) = renderer
                .get_render_item(&source_entity)
                .and_then(|render_entity| transform_query.get(*render_entity).ok())
            {
                let multiplier = match renderer_state.get() {
                    RendererState::TwoDimension => {
                        continue;
                    }
                    _ => 1.0,
                };
                let offset = Vec3::from_array(t.translation.to_array());
                gizmos.arrow(
                    offset,
                    offset + t.rotation.mul_vec3(Vec3::X) * multiplier,
                    Color::RED,
                );
                gizmos.arrow(
                    offset,
                    offset + t.rotation.mul_vec3(FORWARD) * multiplier,
                    Color::GREEN,
                );
                gizmos.arrow(
                    offset,
                    offset + t.rotation.mul_vec3(UP) * multiplier,
                    Color::BLUE,
                );
            }
        }
    }
}
