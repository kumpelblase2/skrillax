use cgmath::Vector2;
use rand::random;
use std::f32::consts::PI;

pub(crate) fn random_point_in_circle(center: Vector2<f32>, radius: f32) -> Vector2<f32> {
    let r = radius * random::<f32>().sqrt();
    let theta = random::<f32>() * 2.0 * PI;
    let x = center.x + r * theta.cos();
    let y = center.y + r * theta.sin();
    Vector2::new(x, y)
}
