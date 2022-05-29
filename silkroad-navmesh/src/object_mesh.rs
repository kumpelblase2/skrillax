use mint::{ColumnMatrix4, Vector2, Vector3};

pub(crate) struct ObjectMesh {
    bounding_box: (Vector2<f32>, Vector2<f32>),
    vertices: Box<[Vector3<f32>]>,
    triangles: Box<[(usize, usize, usize)]>,
    translation_matrix: ColumnMatrix4<f32>,
}
