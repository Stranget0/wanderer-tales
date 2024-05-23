use bevy::{asset::load_internal_asset, prelude::*};
pub struct MyShadersPlugin;

impl Plugin for MyShadersPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            TERRAIN_SHADER_HANDLE,
            "../../assets/shaders/utils_deform.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SHADER_COMMON_HANDLE,
            "../../assets/shaders/utils_common.wgsl",
            Shader::from_wgsl
        );
    }
}

const TERRAIN_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x0a1fb958_f676_4fd7_81e4_b6b16699d170);

const SHADER_COMMON_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x0e78511e_e522_4bc3_aaa8_7d94ab2adcc2);
