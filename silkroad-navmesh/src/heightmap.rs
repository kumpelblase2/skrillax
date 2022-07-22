use std::cmp::min;

pub struct Heightmap {
    map: Box<[f32]>,
    size: usize,
    tile_size: usize,
}

impl Heightmap {
    pub(crate) fn new(data: Box<[f32]>, size: usize, tile_size: usize) -> Heightmap {
        Heightmap {
            map: data,
            size,
            tile_size,
        }
    }

    fn max_size(&self) -> usize {
        self.size * self.tile_size
    }

    pub fn height_at_position(&self, x: f32, y: f32) -> Option<f32> {
        if x < 0. || x > self.max_size() as f32 || y < 0. || y > self.max_size() as f32 {
            return None;
        }

        let size = self.tile_size as f32;
        let left_corner_x = (x / size).floor() as usize;
        let top_corner_y = (y / size).floor() as usize;

        let right_corner_x = min(left_corner_x + 1, self.size);
        let bottom_corner_y = min(top_corner_y + 1, self.size);

        let top_left_height = self.height_of_vertex(left_corner_x, top_corner_y);
        let top_right_height = self.height_of_vertex(right_corner_x, top_corner_y);
        let bottom_left_height = self.height_of_vertex(left_corner_x, bottom_corner_y);
        let bottom_right_height = self.height_of_vertex(right_corner_x, bottom_corner_y);

        let x_diff = (x - (left_corner_x as f32 * size)) / size;
        let y_diff = (y - (top_corner_y as f32 * size)) / size;

        let interpolate_y_1 = top_left_height + (bottom_left_height - top_left_height) * y_diff;
        let interpolate_y_2 = top_right_height + (bottom_right_height - top_right_height) * y_diff;
        let interpolate = interpolate_y_1 + (interpolate_y_2 - interpolate_y_1) * x_diff;
        Some(interpolate)
    }

    fn height_of_vertex(&self, x: usize, y: usize) -> f32 {
        self.map[x + y * self.size]
    }
}
