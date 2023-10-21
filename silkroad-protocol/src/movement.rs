use silkroad_serde::*;
use std::fmt::{Display, Formatter};

#[derive(Clone, Eq, PartialEq, PartialOrd, Copy, Serialize, ByteSize)]
pub enum MovementType {
    #[silkroad(value = 0)]
    Running,
    #[silkroad(value = 1)]
    Walking,
}

#[derive(Copy, Clone, Serialize, Deserialize, ByteSize)]
pub enum MovementTarget {
    #[silkroad(value = 1)]
    TargetLocation { region: u16, x: u16, y: u16, z: u16 },
    #[silkroad(value = 0)]
    Direction { unknown: u8, angle: u16 },
}

impl MovementTarget {
    pub fn targetlocation(region: u16, x: u16, y: u16, z: u16) -> Self {
        MovementTarget::TargetLocation { region, x, y, z }
    }

    pub fn direction(unknown: u8, angle: u16) -> Self {
        MovementTarget::Direction { unknown, angle }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub enum EntityMovementState {
    #[silkroad(value = 1)]
    Moving {
        movement_type: MovementType,
        region: u16,
        x: u16,
        y: u16,
        z: u16,
    },
    #[silkroad(value = 0)]
    Standing {
        movement_type: MovementType,
        unknown: u8,
        angle: u16,
    },
}

impl EntityMovementState {
    pub fn moving(movement_type: MovementType, region: u16, x: u16, y: u16, z: u16) -> Self {
        EntityMovementState::Moving {
            movement_type,
            region,
            x,
            y,
            z,
        }
    }

    pub fn standing(movement_type: MovementType, unknown: u8, angle: u16) -> Self {
        EntityMovementState::Standing {
            movement_type,
            unknown,
            angle,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, ByteSize)]
pub enum MovementDestination {
    #[silkroad(value = 0)]
    Direction { moving: bool, heading: u16 },
    #[silkroad(value = 1)]
    Location { region: u16, x: u16, y: u16, z: u16 },
}

impl MovementDestination {
    pub fn direction(moving: bool, heading: u16) -> Self {
        MovementDestination::Direction { moving, heading }
    }

    pub fn location(region: u16, x: u16, y: u16, z: u16) -> Self {
        MovementDestination::Location { region, x, y, z }
    }
}

#[derive(Serialize, ByteSize, Copy, Clone)]
pub struct Position {
    pub region: u16,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    pub heading: u16,
}

impl Position {
    pub fn new(region: u16, pos_x: f32, pos_y: f32, pos_z: f32, heading: u16) -> Self {
        Position {
            region,
            pos_x,
            pos_y,
            pos_z,
            heading,
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct Location {
    pub region: u16,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} @ ({}|{}|{})", self.region, self.pos_x, self.pos_y, self.pos_z)
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct MovementSource {
    pub region: u16,
    pub x: u16,
    pub y: f32,
    pub z: u16,
}

impl MovementSource {
    pub fn new(region: u16, x: u16, y: f32, z: u16) -> Self {
        MovementSource { region, x, y, z }
    }
}

#[derive(Clone, Deserialize, ByteSize)]
pub struct PlayerMovementRequest {
    pub kind: MovementTarget,
}

#[derive(Clone, Serialize, ByteSize)]
pub struct PlayerMovementResponse {
    pub player_id: u32,
    pub destination: MovementDestination,
    pub source_position: Option<MovementSource>,
}

impl PlayerMovementResponse {
    pub fn new(player_id: u32, destination: MovementDestination, source_position: Option<MovementSource>) -> Self {
        PlayerMovementResponse {
            player_id,
            destination,
            source_position,
        }
    }
}

#[derive(Serialize, ByteSize, Copy, Clone)]
pub struct EntityMovementInterrupt {
    pub entity_id: u32,
    pub position: Position,
}

#[derive(Clone, Serialize, Deserialize, ByteSize)]
pub struct Rotation {
    pub heading: u16,
}

#[derive(Serialize, ByteSize, Copy, Clone)]
pub struct ChangeSpeed {
    pub entity: u32,
    pub walk_speed: f32,
    pub running_speed: f32,
}
