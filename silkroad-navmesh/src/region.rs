use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Region(u16);

impl Display for Region {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#04x}", self.0)
    }
}

impl Region {
    pub fn from_xy(x: u8, y: u8) -> Self {
        let x = x as u16;
        let y = y as u16;
        Region((y << 8) | x)
    }

    pub fn y(&self) -> u8 {
        (self.0 >> 8) as u8
    }

    pub fn x(&self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    pub fn id(&self) -> u16 {
        self.0
    }

    pub fn is_dungeon(&self) -> bool {
        (self.0 & 0x8000) != 0
    }

    pub fn neighbours(&self) -> Vec<Region> {
        let mut regions = Vec::new();
        regions.push(Region::from_xy(self.x() - 1, self.y()));
        regions.push(Region::from_xy(self.x(), self.y()));
        regions.push(Region::from_xy(self.x() + 1, self.y()));
        regions.push(Region::from_xy(self.x() - 1, self.y() - 1));
        regions.push(Region::from_xy(self.x(), self.y() - 1));
        regions.push(Region::from_xy(self.x() + 1, self.y() - 1));
        regions.push(Region::from_xy(self.x() - 1, self.y() + 1));
        regions.push(Region::from_xy(self.x(), self.y() + 1));
        regions.push(Region::from_xy(self.x() + 1, self.y() + 1));
        regions
    }
}

impl From<u16> for Region {
    fn from(id: u16) -> Self {
        Region(id)
    }
}
