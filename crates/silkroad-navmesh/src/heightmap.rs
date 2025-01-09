use std::cmp::min;

/// A heightmap describes the height of points in a given area.
///
/// While the heightmap is generally comprised of a grid with a certain
/// density, the map can provide the height of any point on it by
/// interpolating it with the four nearest tiles.
pub struct Heightmap<'a> {
    map: &'a [f32],
    size: usize,
    tile_size: usize,
}

impl Heightmap<'_> {
    /// Creates a new heightmap
    ///
    /// Builds a new heightmap from the grid specified by `data`, which is expected to
    /// be of size `size * size`. The data array is supposed to be a flat representation
    /// of a two-dimensional structure where a point `(1, 2)` is at index `2 * size + 1`.
    /// `tile_size` specifies the distance between two neighbouring indices, e.g. `(2,0)`
    /// and `(3,0)`. In other words, it represents the resolution of the heightmap.
    pub(crate) fn new(data: &[f32], size: usize, tile_size: usize) -> Heightmap {
        Heightmap {
            map: data,
            size,
            tile_size,
        }
    }

    /// The total size along one axis
    ///
    /// The size consists of the tile size and the grid size.
    fn max_size(&self) -> usize {
        self.size * self.tile_size
    }

    /// Calculates the effective height for a specific location on the heightmap. The heightmap
    /// only contains the location for specific values and therefor the effective height of a
    /// given location needs to be calculated with respect to the nearest given height values.
    ///
    /// This is done by doing a bi-linear interpolation of the nearest four locations that are
    /// close to the target location. As such, the point needs to be inside the grid, for the
    /// calculation to work. If the point is outside of the grid, e.g. if `x < 0`, [None]
    /// will be returned.
    pub fn height_at_position(&self, x: f32, y: f32) -> Option<f32> {
        if x < 0. || x > self.max_size() as f32 || y < 0. || y > self.max_size() as f32 {
            return None;
        }

        let size = self.tile_size as f32;
        let left_corner_x = (x / size).floor() as usize;
        let top_corner_y = (y / size).floor() as usize;

        let right_corner_x = min(left_corner_x + 1, self.size - 1);
        let bottom_corner_y = min(top_corner_y + 1, self.size - 1);

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_empty_map() {
        let data = Box::new([0.0f32, 0.0, 0.0, 0.0]);
        let map = Heightmap::new(data.as_slice(), 2, 1);

        assert_eq!(2, map.max_size());
        assert_eq!(Some(0.0f32), map.height_at_position(0.0, 0.0));
        assert_eq!(Some(0.0f32), map.height_at_position(1.0, 0.0));
        assert_eq!(Some(0.0f32), map.height_at_position(0.0, 1.0));
        assert_eq!(Some(0.0f32), map.height_at_position(1.9999, 1.9999));
    }

    #[test]
    pub fn test_middle() {
        let data = Box::new([0.0f32, 0.0, 0.0, 10.0]);
        let map = Heightmap::new(data.as_slice(), 2, 1);

        assert_eq!(Some(0.0f32), map.height_at_position(0.0, 0.0));
        // we're in the middle between [0,1] and [1,1], where [0,1] is at height 0 and [1,1] is at height 10
        // thus, we should be at 5 - the middle between these two
        assert_eq!(Some(5.0), map.height_at_position(0.5, 1.0));
        // Now we've moved a little bit further towards [0,2] compared to the previous location.
        assert_eq!(Some(2.5), map.height_at_position(0.5, 0.5));
    }
}
