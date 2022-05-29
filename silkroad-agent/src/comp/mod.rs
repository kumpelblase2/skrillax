pub(crate) mod player;
pub(crate) mod pos;
pub(crate) mod stats;
pub(crate) mod visibility;

use crate::character_loader::Character;
use crate::db::user::ServerUser;
use crate::population::capacity::PlayingToken;
use bevy_ecs::prelude::*;
use cgmath::Vector3;
use pos::GlobalPosition;
use silkroad_navmesh::region::Region;
use silkroad_network::stream::Stream;
use silkroad_protocol::{ClientPacket, ServerPacket};
use std::collections::VecDeque;
use std::time::Instant;

#[derive(Component)]
pub(crate) struct Login;

#[derive(Component)]
pub(crate) struct LastAction(pub(crate) Instant);

#[derive(Component)]
pub(crate) struct CharacterSelect(pub(crate) Option<Vec<Character>>);

#[derive(Component)]
pub(crate) struct Client(pub(crate) Stream, pub(crate) VecDeque<ClientPacket>);

impl Client {
    pub fn send<T: Into<ServerPacket>>(&self, packet: T) {
        // We specifically ignore the error here because we'll handle the client being disconnected
        // at the end of the game tick. This means we might do some unnecessary things, but that's ok
        // for now. The upside is that this means there's a single point where we handle such errors.
        let _ = self.0.send(packet);
    }
}

#[derive(Component)]
pub(crate) struct Playing(pub(crate) ServerUser, pub(crate) PlayingToken);

#[derive(Component)]
pub(crate) struct NetworkedEntity(pub u32);

#[derive(Component)]
pub(crate) struct Health(f32);

#[derive(Component)]
pub(crate) struct Monster {
    pub target: Option<Entity>,
}
