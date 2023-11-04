use bevy_ecs_macros::Component;
use cgmath::{MetricSpace, Vector2};
use silkroad_game_base::{GlobalLocation, GlobalPosition, Heading};
use silkroad_protocol::movement::{EntityMovementState, MovementType};

#[derive(Component, Copy, Clone)]
pub(crate) struct Position {
    location: GlobalPosition,
    rotation: Heading,
    has_rotated: bool,
    has_moved: bool,
}

impl Position {
    pub fn new(pos: GlobalPosition, rotation: Heading) -> Self {
        Position {
            location: pos,
            rotation,
            has_rotated: false,
            has_moved: false,
        }
    }

    pub fn as_protocol(&self) -> silkroad_protocol::movement::Position {
        let local = self.location.to_local();
        silkroad_protocol::movement::Position {
            region: local.0.id(),
            pos_x: local.1.x,
            pos_y: local.1.y,
            pos_z: local.1.z,
            heading: self.rotation.into(),
        }
    }

    pub fn as_movement(&self) -> EntityMovementState {
        let local = self.location.to_local();
        EntityMovementState::Moving {
            movement_type: MovementType::Running,
            region: local.0.id(),
            x: local.1.x as u16,
            y: local.1.y as u16,
            z: local.1.z as u16,
        }
    }

    pub fn as_standing(&self) -> EntityMovementState {
        EntityMovementState::Standing {
            movement_type: MovementType::Walking,
            unknown: 0,
            angle: self.rotation.into(),
        }
    }

    pub fn distance_to(&self, other: &Position) -> f32 {
        let my_vec2 = Vector2::new(self.location.0.x, self.location.0.z);
        let other_vec2 = Vector2::new(other.location.0.x, other.location.0.z);
        my_vec2.distance2(other_vec2)
    }

    pub fn position(&self) -> GlobalPosition {
        self.location
    }

    pub fn location(&self) -> GlobalLocation {
        self.location.to_location()
    }

    pub fn rotation(&self) -> Heading {
        self.rotation
    }

    pub fn rotate(&mut self, heading: Heading) {
        self.has_rotated = true;
        self.rotation = heading;
    }

    pub fn move_to(&mut self, new_pos: GlobalPosition) {
        self.location = new_pos;
        self.has_moved = true;
    }

    pub fn update(&mut self, pos: GlobalPosition, heading: Heading) {
        self.move_to(pos);
        self.rotate(heading);
    }

    pub fn reset(&mut self) {
        self.has_moved = false;
        self.has_rotated = false;
    }

    pub fn did_rotate(&self) -> bool {
        self.has_rotated
    }

    pub fn did_move(&self) -> bool {
        self.has_moved
    }
}
