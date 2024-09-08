use bevy::{asset::load_internal_asset, prelude::*};

pub fn plugin(app: &mut App) {
    load_internal_asset!(app, SHADER_UTILS_NOISE, "./noise.wgsl", Shader::from_wgsl);
    load_internal_asset!(app, SHADER_UTILS_COMMON, "./common.wgsl", Shader::from_wgsl);
}

const SHADER_UTILS_NOISE: Handle<Shader> =
    Handle::weak_from_u128(0x0e78511e_e522_4bc3_aaa8_7d94ab2adcc2);

const SHADER_UTILS_COMMON: Handle<Shader> =
    Handle::weak_from_u128(0x32d7aff6_d67d_4a2a_a6d7_01139cbca5e0);
