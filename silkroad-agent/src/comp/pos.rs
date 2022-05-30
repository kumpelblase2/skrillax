use bevy_ecs_macros::Component;
use cgmath::{Vector2, Vector3};
use silkroad_navmesh::region::Region;
use silkroad_protocol::world::{EntityMovementState, MovementType};

#[derive(Copy, Clone)]
pub struct LocalLocation(pub Region, pub Vector2<f32>);

#[derive(Copy, Clone)]
pub struct GlobalLocation(pub Vector2<f32>);

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

    pub fn with_y(&self, y: f32) -> GlobalPosition {
        GlobalPosition(Vector3::new(self.0.x, y, self.0.y))
    }
}

#[derive(Copy, Clone)]
pub struct LocalPosition(pub Region, pub Vector3<f32>);

#[derive(Copy, Clone)]
pub struct GlobalPosition(pub Vector3<f32>);

impl LocalPosition {
    pub fn to_global(&self) -> GlobalPosition {
        let global_x = self.1.x + (self.0.x() as f32 * 1920.);
        let global_z = self.1.z + (self.0.y() as f32 * 1920.);
        GlobalPosition(Vector3::new(global_x, self.1.y, global_z))
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
        Heading(percent * 360.)
    }
}

impl Into<u16> for Heading {
    fn into(self) -> u16 {
        let percentage = 360.0 / self.0;
        (percentage * (u16::MAX as f32)) as u16
    }
}

#[derive(Component, Clone)]
pub(crate) struct Position {
    pub location: GlobalPosition,
    pub rotation: Heading,
}

impl Position {
    pub fn as_protocol(&self) -> silkroad_protocol::world::Position {
        let local = self.location.to_local();
        silkroad_protocol::world::Position {
            region: local.0.id(),
            pos_x: local.1.x,
            pos_y: local.1.y,
            pos_z: local.1.z,
            heading: self.rotation.into(),
        }
    }

    pub fn as_movement(&self) -> silkroad_protocol::world::EntityMovementState {
        let local = self.location.to_local();
        EntityMovementState::Moving {
            movement_type: MovementType::Running,
            region: local.0.id(),
            x: local.1.x as u16,
            y: local.1.y as u16,
            z: local.1.z as u16,
        }
    }
}
