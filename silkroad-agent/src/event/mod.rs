use bevy_ecs::prelude::*;

pub(crate) enum ServerEvent {
    ClientConnected,
    ClientDisconnected(Entity),
}
