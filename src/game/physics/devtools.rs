use crate::prelude::*;
use avian3d::prelude::*;

enum Flags {
    PhyiscsGizmos,
}

impl DebugFlagsExt for Flags {
    fn group(&self) -> &'static str {
        "Physics"
    }
    fn as_str(&self) -> &'static str {
        match self {
            Flags::PhyiscsGizmos => "Physics gizmos",
        }
    }
}

pub fn plugin(app: &mut App) {
    register_debug_flags(app, vec![Flags::PhyiscsGizmos]);
    app.add_plugins(PhysicsDebugPlugin::default())
        .add_systems(Update, sync_flags.run_if(debug_flags_changed));
}

fn sync_flags(mut gizmos: ResMut<GizmoConfigStore>, flags: Res<DebugFlags>) {
    let config = gizmos.config_mut::<PhysicsGizmos>().0;
    config.enabled = flags.get(&Flags::PhyiscsGizmos);
}
