use bevy::{asset::load_internal_asset, prelude::*};

pub(super) fn plugin(app: &mut App) {
    load_internal_asset!(app, SHADER_UTILS_NOISE, "noise.wgsl", Shader::from_wgsl);
}

const SHADER_UTILS_NOISE: Handle<Shader> =
    Handle::weak_from_u128(0x0e78511e_e522_4bc3_aaa8_7d94ab2adcc2);
