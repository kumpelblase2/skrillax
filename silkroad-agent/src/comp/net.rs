use bevy_ecs::prelude::*;
use silkroad_protocol::gm::GmCommand;
use silkroad_protocol::ClientPacket;

#[derive(Component, Default)]
pub struct CharselectInput {
    pub inputs: Vec<ClientPacket>,
}

#[derive(Component, Default)]
pub struct MovementInput {
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
}
