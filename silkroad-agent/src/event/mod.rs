use bevy_ecs::prelude::*;
use cgmath::Vector3;

pub(crate) struct ClientConnectedEvent(pub Entity);

pub(crate) struct ClientDisconnectedEvent(pub Entity);

pub(crate) enum ChatEvent {
    RegionalChat {
        sender: Entity,
        sender_unique_id: u32,
        position: Vector3<f32>,
        message: String,
    },
    PrivateChat {
        sender: String,
        target: String,
        message: String,
    },
    Command {
        sender: Entity,
        message: String,
    },
}

pub(crate) struct PlayerLevelUp(pub Entity, pub u8);
