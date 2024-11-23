use crate::prelude::*;
use avian3d::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(PhysicsPlugins::default());

    // #[cfg(feature = "dev")]
    // app.add_plugins(PhysicsDebugPlugin::default());
}

#[derive(PhysicsLayer)]
pub(crate) enum CollisionLayer {
    Player,
    Character,
    Terrain,
    CameraObstacle,
    Sensor,
}
