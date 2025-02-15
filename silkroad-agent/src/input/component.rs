use bevy::prelude::*;
use silkroad_game_base::StatType;
use silkroad_protocol::auth::{AuthRequest, LogoutRequest};
use silkroad_protocol::character::{CharacterJoinRequest, CharacterListRequestAction};
use silkroad_protocol::chat::ChatClientProtocol;
use silkroad_protocol::combat::PerformAction;
use silkroad_protocol::gm::GmCommand;
use silkroad_protocol::inventory::InventoryOperation;
use silkroad_protocol::movement::{MovementTarget, Rotation};
use silkroad_protocol::skill::{HotbarItem, LearnSkill, LevelUpMastery};
use silkroad_protocol::world::{TargetEntity, UnTargetEntity};
use std::mem;

#[derive(Component, Default)]
pub(crate) struct PlayerInput {
    pub logout: Option<LogoutRequest>,
    pub target: Option<TargetEntity>,
    pub untarget: Option<UnTargetEntity>,
    pub chat: Vec<ChatClientProtocol>,
    pub action: Option<PerformAction>,
    pub movement: Option<MovementTarget>,
    pub rotation: Option<Rotation>,
    pub inventory: Option<InventoryOperation>,
    pub gm: Option<GmCommand>,
    pub mastery: Option<LevelUpMastery>,
    pub skill_add: Option<LearnSkill>,
    pub increase_stats: Vec<StatType>,
    pub hotbar: Option<Vec<HotbarItem>>,
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
