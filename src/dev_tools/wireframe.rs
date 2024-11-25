use crate::prelude::*;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};

enum Flags {
    Wireframe,
}

impl DebugFlagsExt for Flags {
    fn group(&self) -> &'static str {
        "Wireframe"
    }
    fn as_str(&self) -> &'static str {
        match self {
            Flags::Wireframe => "Wireframe",
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    register_debug_flags(app, vec![Flags::Wireframe]);

    app.add_plugins(WireframePlugin)
        .insert_resource(WireframeConfig {
            default_color: Color::srgb(0.5, 0.5, 0.5),
            ..Default::default()
        })
        .add_systems(Update, sync_flags.run_if(debug_flags_changed));
}

fn sync_flags(mut wireframe: ResMut<WireframeConfig>, flags: Res<DebugFlags>) {
    wireframe.global = flags.get(&Flags::Wireframe);
}
