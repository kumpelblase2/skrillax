use mint::Vector2;
use sr_formats::jmxvnvm::JmxNvm;
use std::cmp::min;

pub struct WorldSpace;

const TILE_WIDTH: f32 = 20.0;
const TILE_HEIGHT: f32 = 20.0;

pub trait NavmeshExt {
    fn height_at_position<T: Into<Vector2<f32>>>(&self, position: T) -> f32;

    fn height_of_vertex(&self, x: usize, y: usize) -> f32;

    fn vertex_count_width(&self) -> usize;
    fn vertex_count_height(&self) -> usize;
}

impl NavmeshExt for JmxNvm {
    fn height_at_position<T: Into<Vector2<f32>>>(&self, position: T) -> f32 {
        let position = position.into();
        // TODO this should return a result, because the vector could be outside of the grid
        let left_corner_x = (position.x / TILE_WIDTH).floor() as usize;
        let top_corner_y = (position.y / TILE_HEIGHT).floor() as usize;

        let right_corner_x = min(left_corner_x + 1, self.vertex_count_width());
        let bottom_corner_y = min(top_corner_y + 1, self.vertex_count_height());

        let top_left_height = self.height_of_vertex(left_corner_x, top_corner_y);
        let top_right_height = self.height_of_vertex(right_corner_x, top_corner_y);
        let bottom_left_height = self.height_of_vertex(left_corner_x, bottom_corner_y);
        let bottom_right_height = self.height_of_vertex(right_corner_x, bottom_corner_y);

        let x_diff = (position.x - (left_corner_x as f32 * TILE_WIDTH)) / TILE_WIDTH;
        let y_diff = (position.y - (top_corner_y as f32 * TILE_HEIGHT)) / TILE_HEIGHT;

        let interpolate_y_1 = top_left_height + (bottom_left_height - top_left_height) * y_diff;
        let interpolate_y_2 = top_right_height + (bottom_right_height - top_right_height) * y_diff;
        let interpolate = interpolate_y_1 + (interpolate_y_2 - interpolate_y_1) * x_diff;
        interpolate
    }

    fn height_of_vertex(&self, x: usize, y: usize) -> f32 {
        self.height_map[x + y * self.vertex_count_width()]
    }

    fn vertex_count_width(&self) -> usize {
        96
    }

    fn vertex_count_height(&self) -> usize {
        96
    }
}
