use bevy_ecs::prelude::*;
use silkroad_protocol::auth::{AuthRequest, LogoutRequest};
use silkroad_protocol::character::{CharacterJoinRequest, CharacterListRequestAction};
use silkroad_protocol::chat::ChatMessage;
use silkroad_protocol::combat::PerformAction;
use silkroad_protocol::gm::GmCommand;
use silkroad_protocol::inventory::InventoryOperation;
use silkroad_protocol::world::{MovementTarget, Rotation, TargetEntity, UnTargetEntity};
use std::mem;
use silkroad_protocol::skill::LevelUpMastery;

#[derive(Component, Default)]
pub(crate) struct PlayerInput {
    pub logout: Option<LogoutRequest>,
    pub target: Option<TargetEntity>,
    pub untarget: Option<UnTargetEntity>,
    pub chat: Vec<ChatMessage>,
    pub action: Option<PerformAction>,
    pub movement: Option<MovementTarget>,
    pub rotation: Option<Rotation>,
    pub inventory: Option<InventoryOperation>,
    pub gm: Option<GmCommand>,
    pub mastery: Option<LevelUpMastery>
}

impl PlayerInput {
    pub(crate) fn reset(&mut self) {
        mem::take(self);
    }
}

#[derive(Component, Default)]
pub(crate) struct LoginInput {
    pub list: Vec<CharacterListRequestAction>,
    pub join: Option<CharacterJoinRequest>,
    pub auth: Option<AuthRequest>,
}

impl LoginInput {
    pub(crate) fn reset(&mut self) {
        mem::take(self);
    }
}
