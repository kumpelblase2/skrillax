use cgmath::{Vector2, Vector3};

pub(crate) trait Vector3Ext {
    fn to_flat_vec2(&self) -> Vector2<f32>;
}

impl Vector3Ext for Vector3<f32> {
    fn to_flat_vec2(&self) -> Vector2<f32> {
        Vector2::new(self.x, self.z)
    }
}
