use bevy::prelude::*;

use crate::gameplay::map::renderer::state::RendererState;
use crate::utils::*;

#[derive(Component)]
pub struct LocalGizmo;

pub fn draw_local_gizmos(
    mut gizmos: Gizmos,
    player: Query<&Transform, With<LocalGizmo>>,
    renderer: Res<State<RendererState>>,
) {
    for t in player.iter() {
        let multiplier = match renderer.get() {
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
