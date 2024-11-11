use bevy::prelude::*;

pub(crate) trait Vec3Ext: Copy {
    fn is_approx_zero(self) -> bool;
    fn horizontal(self) -> Vec3;
}
impl Vec3Ext for Vec3 {
    #[inline]
    fn is_approx_zero(self) -> bool {
        self.length_squared() < 1e-5
    }

    #[inline]
    fn horizontal(self) -> Vec3 {
        Vec3::new(self.x, 0., self.z)
    }
}
