use cgmath::{Deg, InnerSpace, Vector2, Vector3};
use silkroad_definitions::Region;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Deref};

#[derive(Copy, Clone)]
pub struct LocalLocation(pub Region, pub Vector2<f32>);

#[derive(Copy, Clone)]
pub struct GlobalLocation(pub Vector2<f32>);

impl Add<Vector2<f32>> for GlobalLocation {
    type Output = Self;

    fn add(self, rhs: Vector2<f32>) -> Self::Output {
        GlobalLocation(self.0 + rhs)
    }
}

impl Deref for GlobalLocation {
    type Target = Vector2<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<GlobalLocation> for LocalLocation {
    fn from(value: GlobalLocation) -> Self {
        value.to_local()
    }
}

impl From<LocalLocation> for GlobalLocation {
    fn from(value: LocalLocation) -> Self {
        value.to_global()
    }
}

impl LocalLocation {
    pub fn to_global(&self) -> GlobalLocation {
        let global_x = self.1.x + (self.0.x() as f32 * 1920.);
        let global_z = self.1.y + (self.0.y() as f32 * 1920.);
        GlobalLocation(Vector2::new(global_x, global_z))
    }
}

impl GlobalLocation {
    pub fn to_local(&self) -> LocalLocation {
        let region_x = (self.0.x / 1920.) as u8;
        let region_y = (self.0.y / 1920.) as u8;
        let region = Region::from_xy(region_x, region_y);
        let local_x = self.0.x % 1920.;
        let local_z = self.0.y % 1920.;
        LocalLocation(region, Vector2::new(local_x, local_z))
    }

    pub fn from_ingame_location(x: f32, z: f32) -> GlobalLocation {
        let x = x * 10.0 + (0x87 as f32 * 1920.0);
        let z = z * 10.0 + (0x5C as f32 * 1920.0);
        GlobalLocation(Vector2::new(x, z))
    }

    pub fn with_y(&self, y: f32) -> GlobalPosition {
        GlobalPosition(Vector3::new(self.0.x, y, self.0.y))
    }
}

impl Display for GlobalLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}", self.x, self.y)
    }
}

#[derive(Copy, Clone)]
pub struct LocalPosition(pub Region, pub Vector3<f32>);

impl Display for LocalPosition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}|{} @ {}", self.1.x, self.1.y, self.1.z, self.0)
    }
}

impl LocalPosition {
    pub fn to_global(&self) -> GlobalPosition {
        let global_x = self.1.x + (self.0.x() as f32 * 1920.);
        let global_z = self.1.z + (self.0.y() as f32 * 1920.);
        GlobalPosition(Vector3::new(global_x, self.1.y, global_z))
    }
}

#[derive(Copy, Clone)]
pub struct GlobalPosition(pub Vector3<f32>);

impl Deref for GlobalPosition {
    type Target = Vector3<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Add<Vector3<f32>> for GlobalPosition {
    type Output = Self;

    fn add(self, rhs: Vector3<f32>) -> Self::Output {
        GlobalPosition(self.0 + rhs)
    }
}

impl Display for GlobalPosition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}|{}", self.0.x, self.0.y, self.0.z)
    }
}

impl GlobalPosition {
    pub fn to_local(&self) -> LocalPosition {
        let region_x = (self.0.x / 1920.) as u8;
        let region_y = (self.0.z / 1920.) as u8;
        let region = Region::from_xy(region_x, region_y);
        let local_x = self.0.x % 1920.;
        let local_z = self.0.z % 1920.;
        LocalPosition(region, Vector3::new(local_x, self.0.y, local_z))
    }

    pub fn region(&self) -> Region {
        let region_x = (self.0.x / 1920.) as u8;
        let region_y = (self.0.z / 1920.) as u8;
        Region::from_xy(region_x, region_y)
    }

    pub fn to_location(&self) -> GlobalLocation {
        GlobalLocation(Vector2::new(self.0.x, self.0.z))
    }

    pub fn from_ingame_position(x: f32, y: f32, z: f32) -> GlobalPosition {
        let x = x * 10.0 + (0x87 as f32 * 1920.0);
        let z = z * 10.0 + (0x5C as f32 * 1920.0);
        GlobalPosition(Vector3::new(x, y, z))
    }
}

impl From<GlobalPosition> for LocalPosition {
    fn from(global: GlobalPosition) -> Self {
        global.to_local()
    }
}

impl From<LocalPosition> for GlobalPosition {
    fn from(local: LocalPosition) -> Self {
        local.to_global()
    }
}

#[derive(Copy, Clone)]
pub struct Heading(pub f32);

impl From<u16> for Heading {
    fn from(heading: u16) -> Self {
        if heading == 0 {
            return Heading(0.);
        }
        let percent = (heading as f32) / (u16::MAX as f32);
        Heading(360. - percent * 360.)
    }
}

impl From<Vector2<f32>> for Heading {
    fn from(value: Vector2<f32>) -> Self {
        Heading(Deg::from(value.angle(Vector2::unit_x())).0)
    }
}

impl From<Heading> for u16 {
    fn from(heading: Heading) -> Self {
        if heading.0 == 0.0 {
            return 0;
        }
        let percentage = (360. - heading.0) / 360.0;
        (percentage * (u16::MAX as f32)) as u16
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_convert_global() {
        let global = GlobalLocation::from_ingame_location(6047.0, 1144.0);
        let local = global.to_local();

        assert_eq!(local.0, Region::from(24998));
        assert_eq!(local.1.x, 950.0);
        assert_eq!(local.1.y, 1840.0);
    }
}
