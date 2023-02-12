use cgmath::{Vector2, Vector3};
use rand::random;

pub trait Vector3Ext {
    fn to_flat_vec2(&self) -> Vector2<f32>;
}

impl Vector3Ext for Vector3<f32> {
    fn to_flat_vec2(&self) -> Vector2<f32> {
        Vector2::new(self.x, self.z)
    }
}

pub trait Vector2Ext<T> {
    fn random_in_radius(&self, radius: T) -> Self;
    fn with_height(&self, height: T) -> Vector3<T>;
}

impl Vector2Ext<f32> for Vector2<f32> {
    fn random_in_radius(&self, radius: f32) -> Self {
        let r = radius * random::<f32>().sqrt();
        let theta = random::<f32>() * 2.0 * std::f32::consts::PI;
        let x = self.x + r * theta.cos();
        let y = self.y + r * theta.sin();
        Vector2::new(x, y)
    }

    fn with_height(&self, height: f32) -> Vector3<f32> {
        Vector3::new(self.x, height, self.y)
    }
}

impl Vector2Ext<f64> for Vector2<f64> {
    fn random_in_radius(&self, radius: f64) -> Self {
        let r = radius * random::<f64>().sqrt();
        let theta = random::<f64>() * 2.0 * std::f64::consts::PI;
        let x = self.x + r * theta.cos();
        let y = self.y + r * theta.sin();
        Vector2::new(x, y)
    }

    fn with_height(&self, height: f64) -> Vector3<f64> {
        Vector3::new(self.x, height, self.y)
    }
}
