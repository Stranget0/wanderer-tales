use crate::gameplay::{
    map::{
        renderer::events::RenderCharacterEvent,
        spawner::{MapAddEvent, MapSubEvent},
        utils::{
            hex_layout::HexLayout,
            hex_map_item::{Biome, Height},
            hex_vector::FractionalHexVector,
        },
    },
    player::{
        components::{Character, HexPosition, HexPositionFractional, PlayerControllable},
        events::CharacterMovedEvent,
    },
};
use bevy::prelude::*;

use super::{
    components::{CameraOffset, SourceCameraFollow},
    renderers::traits::{CreateCharacterRenderBundle, CreateMapRenderBundle, RenderMapApi},
};

pub(crate) fn render_map<T: Bundle, R: CreateMapRenderBundle<T> + RenderMapApi + Component>(
    mut commands: Commands,
    mut render_map_event: EventReader<MapAddEvent>,
    mut layout_query: Query<(Entity, &HexLayout, &mut R)>,
    map_data_query: Query<(&HexPosition, &Biome, &Height)>,
) {
    for e in render_map_event.read() {
        for (layout_entity, layout, mut renderer) in layout_query.iter_mut() {
            let mut spawned_hexes = Vec::new();

            for source_entity in e.source_items.iter() {
                match map_data_query.get(*source_entity) {
                    Ok((pos, biome, height)) => {
                        let render_bundle =
                            renderer.create_map_render_bundle(layout, pos, biome, height);

                        let render_entity = commands.spawn(render_bundle).id();
                        let old_option = renderer.link_source_item(source_entity, &render_entity);

                        if let Some(old_rendered_entity) = old_option {
                            warn!("double render, replacing with new one");
                            commands.entity(old_rendered_entity).despawn();
                        }
                        spawned_hexes.push(render_entity);
                    }
                    Err(err) => {
                        error!("[renderer] entity get error: {}", err);
                        continue;
                    }
                };
            }

            commands.entity(layout_entity).push_children(&spawned_hexes);
        }
    }
}

pub(crate) fn fill_map<T: Bundle, R: CreateMapRenderBundle<T> + RenderMapApi + Component>(
    mut commands: Commands,
    mut layout_query: Query<(Entity, &HexLayout, &mut R)>,
    map_data_query: Query<(Entity, &HexPosition, &Biome, &Height)>,
) {
    for (layout_entity, layout, mut renderer) in layout_query.iter_mut() {
        for (source_entity, pos, biome, height) in map_data_query.iter() {
            if renderer.get_render_item(&source_entity).is_some() {
                continue;
            }
            let render_bundle = renderer.create_map_render_bundle(layout, pos, biome, height);
            let render_entity = commands.spawn(render_bundle).id();
            renderer.link_source_item(&source_entity, &render_entity);
            commands.entity(layout_entity).add_child(render_entity);
        }
    }
}

pub(crate) fn show_entity<E: Component>(mut commands: Commands, query: Query<Entity, With<E>>) {
    for entity in query.iter() {
        commands.entity(entity).insert(Visibility::Visible);
    }
}

pub(crate) fn despawn_map<R: RenderMapApi + Component>(
    mut commands: Commands,
    mut despawn_map_event: EventReader<MapSubEvent>,
    mut layout_query: Query<(Entity, &mut R)>,
) {
    for e in despawn_map_event.read() {
        for (layout_entity, mut render_map) in layout_query.iter_mut() {
            let mut children_to_remove: Vec<Entity> = Vec::new();
            for source_entity in e.source_items.iter() {
                if let Some(render_entity) = render_map.remove_render_item(source_entity) {
                    commands.entity(render_entity).despawn_recursive();
                    children_to_remove.push(render_entity);
                }
            }
            commands
                .entity(layout_entity)
                .remove_children(&children_to_remove);
        }
    }
}

pub(crate) fn render_character<
    T: Bundle,
    R: CreateCharacterRenderBundle<T> + RenderMapApi + Component,
>(
    mut commands: Commands,
    mut event: EventReader<RenderCharacterEvent>,
    mut layout_query: Query<(Entity, &HexLayout, &mut R)>,
) {
    for e in event.read() {
        for (layout_entity, layout, mut renderer) in layout_query.iter_mut() {
            let pos = layout.hex_to_pixel(&e.position.0);
            let render_entity = commands
                .spawn(renderer.create_character_render_bundle(&pos, e))
                .id();

            renderer.link_source_item(&e.source_entity, &render_entity);

            commands.entity(layout_entity).add_child(render_entity);
        }
    }
}

pub(crate) fn move_rendered_character<R: RenderMapApi + Component>(
    mut event: EventReader<CharacterMovedEvent>,
    mut transform_query: Query<&mut Transform>,
    layout_query: Query<(&HexLayout, &R)>,
) {
    for e in event.read() {
        for (layout, renderer) in layout_query.iter() {
            let render_entity_option = renderer.get_render_item(&e.source_entity);
            if let Some(render_entity) = render_entity_option {
                if let Ok(mut transform) = transform_query.get_mut(*render_entity) {
                    let delta = layout.hex_to_pixel(&e.delta_pos.0);
                    transform.translation.x += delta.x;
                    transform.translation.y += delta.y;
                }
            } else {
                error!("Could not get character render entity");
            }
        }
    }
}

pub(crate) fn synchronize_rendered_characters<R: RenderMapApi + Component>(
    mut player_render_query: Query<&mut Transform>,
    layout_query: Query<(&HexLayout, &R)>,
    character_query: Query<(Entity, &HexPositionFractional), With<Character>>,
) {
    for (layout, renderer) in layout_query.iter() {
        for (source_entity, pos) in character_query.iter() {
            match renderer.get_render_item(&source_entity) {
                Some(render_entity) => match player_render_query.get_mut(*render_entity) {
                    Ok(mut transform) => {
                        let pixel_pos = layout.hex_to_pixel(&pos.0);
                        transform.translation.x = pixel_pos.x;
                        transform.translation.y = pixel_pos.y;
                    }
                    Err(_) => error!("Character doesn't have transform to synchronize"),
                },
                None => warn!("No character render found to synchronize"),
            }
        }
    }
}

pub(crate) fn camera_follow<R: RenderMapApi + Component>(
    mut camera_query: Query<
        (&mut Transform, Option<&CameraOffset>, Option<&Camera3d>),
        With<Camera>,
    >,
    source_followed_query: Query<Entity, With<SourceCameraFollow>>,
    renderer_query: Query<&R>,
    transform_query: Query<&Transform, Without<Camera>>,
) {
    for renderer in renderer_query.iter() {
        for source_entity in source_followed_query.iter() {
            match renderer
                .get_render_item(&source_entity)
                .and_then(|entity| transform_query.get(*entity).ok())
            {
                Some(target_transform) => {
                    for (mut camera_transform, offset_option, camera_3d_option) in
                        camera_query.iter_mut()
                    {
                        let render_pos = Vec3::from_array(target_transform.translation.to_array());
                        let mut camera_pos =
                            Vec3::from_array(target_transform.translation.to_array());

                        if let Some(offset) = offset_option {
                            camera_pos += Vec3::new(offset.0, offset.1, offset.2);
                        }

                        camera_transform.translation.x = camera_pos.x;
                        camera_transform.translation.y = camera_pos.y;
                        camera_transform.translation.z = camera_pos.z;

                        if camera_3d_option.is_some() {
                            camera_transform.look_at(render_pos, Vec3::Y);
                        }
                    }
                }
                None => {
                    continue;
                }
            }
        }
    }
}
