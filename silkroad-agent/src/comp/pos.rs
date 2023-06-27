use bevy_ecs_macros::Component;
use cgmath::{MetricSpace, Vector2};
use silkroad_game_base::{GlobalPosition, Heading};
use silkroad_protocol::world::{EntityMovementState, MovementType};

#[derive(Component, Clone, Copy)]
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
}
