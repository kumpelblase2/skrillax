use silkroad_definitions::Region;

pub trait GridRegion {
    fn with_grid_neighbours(&self) -> [Region; 9];
}

impl GridRegion for Region {
    fn with_grid_neighbours(&self) -> [Region; 9] {
        [
            Region::from_xy(self.x() - 1, self.y()),
            *self,
            Region::from_xy(self.x() + 1, self.y()),
            Region::from_xy(self.x() - 1, self.y() - 1),
            Region::from_xy(self.x(), self.y() - 1),
            Region::from_xy(self.x() + 1, self.y() - 1),
            Region::from_xy(self.x() - 1, self.y() + 1),
            Region::from_xy(self.x(), self.y() + 1),
            Region::from_xy(self.x() + 1, self.y() + 1),
        ]
    }
}
