use bevy_ecs::prelude::*;
use silkroad_network::stream::Stream;
use silkroad_protocol::gm::GmCommand;
use silkroad_protocol::{ClientPacket, ServerPacket};
use std::time::Instant;

#[derive(Component, Default)]
pub struct CharselectInput {
    pub inputs: Vec<ClientPacket>,
}

#[derive(Component, Default)]
pub struct MovementInput {
    pub inputs: Vec<ClientPacket>,
}

#[derive(Component, Default)]
pub struct InventoryInput {
    pub inputs: Vec<ClientPacket>,
}

#[derive(Component, Default)]
pub struct ChatInput {
    pub inputs: Vec<ClientPacket>,
}

#[derive(Component, Default)]
pub struct WorldInput {
    pub inputs: Vec<ClientPacket>,
}

#[derive(Component, Default)]
pub struct GmInput {
    pub inputs: Vec<GmCommand>,
}

#[derive(Bundle, Default)]
pub struct InputBundle {
    movement: MovementInput,
    chat: ChatInput,
    world: WorldInput,
    inventory: InventoryInput,
}

#[derive(Component)]
pub(crate) struct LastAction(pub(crate) Instant);

#[derive(Component)]
pub(crate) struct Client(pub(crate) Stream);

impl Client {
    pub fn send<T: Into<ServerPacket>>(&self, packet: T) {
        // We specifically ignore the error here because we'll handle the client being disconnected
        // at the end of the game tick. This means we might do some unnecessary things, but that's ok
        // for now. The upside is that this means there's a single point where we handle such errors.
        let _ = self.0.send(packet);
    }
}
