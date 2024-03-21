use super::{FORWARD, UP};

pub fn to_3d_space(x: f32, y: f32, z: f32) -> [f32; 3] {
    let up = UP * z;
    let forward = FORWARD * y;
    let mut res = up + forward;
    res.x = x;

    [res.x, res.y, res.z]
}
