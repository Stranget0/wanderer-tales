use crate::prelude::*;
use bevy::{dev_tools::states::log_transitions, render::render_resource::ShaderImport};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (log_shader_load, log_transitions::<GameState>));
}

fn log_shader_load(mut asset_event: EventReader<AssetEvent<Shader>>, shaders: Res<Assets<Shader>>) {
    for event in asset_event.read() {
        let e = match event {
            AssetEvent::Added { id } => shaders.get(*id).map(|s| ("Added", s)),
            _ => None,
        };
        let Some((action, path, import_path)) = e.and_then(|e| {
            let ShaderImport::Custom(import_path) = e.1.import_path() else {
                return None;
            };
            if import_path.contains("wanderer_tales") {
                return Some((e.0, &e.1.path, import_path));
            }
            None
        }) else {
            continue;
        };

        info!("{} shader {} => {}", action, path, import_path);
    }
}
