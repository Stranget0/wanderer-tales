pub mod f32 {
    pub fn fmod(x: f32, y: f32) -> f32 {
        x - y * (x / y).floor()
    }
}
pub mod f64 {
    pub fn fmod(x: f64, y: f64) -> f64 {
        x - y * (x / y).floor()
    }
}
