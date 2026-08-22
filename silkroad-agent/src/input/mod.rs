mod events;
mod system;

use crate::input::system::{handle_identity_information, handle_packet_inputs};
use bevy::prelude::*;
pub use events::PlayerInputEvent;
use silkroad_protocol::auth::{AuthRequest, LogoutRequest};
use silkroad_protocol::character::{CharacterJoinRequest, CharacterListRequest, FinishLoading};
use silkroad_protocol::chat::ChatMessage;
use silkroad_protocol::combat::PerformAction;
use silkroad_protocol::gm::GmCommand;
use silkroad_protocol::inventory::{ConsignmentList, InventoryOperation, OpenItemMall};
use silkroad_protocol::movement::{PlayerMovementRequest, Rotation};
use silkroad_protocol::skill::{HotbarUpdate, LearnSkill, LevelUpMastery};
use silkroad_protocol::world::{
    GameGuideResponse, IncreaseInt, IncreaseStr, TargetEntity, UnTargetEntity, UpdateGameGuide,
};
use silkroad_protocol::IdentityInformation;

pub(crate) struct ReceivePlugin;

impl Plugin for ReceivePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, handle_packet_inputs)
            .add_systems(Update, handle_identity_information)
            .add_message::<PlayerInputEvent<ChatMessage>>()
            .add_message::<PlayerInputEvent<IncreaseInt>>()
            .add_message::<PlayerInputEvent<IncreaseStr>>()
            .add_message::<PlayerInputEvent<FinishLoading>>()
            .add_message::<PlayerInputEvent<GmCommand>>()
            .add_message::<PlayerInputEvent<TargetEntity>>()
            .add_message::<PlayerInputEvent<PerformAction>>()
            .add_message::<PlayerInputEvent<UnTargetEntity>>()
            .add_message::<PlayerInputEvent<UpdateGameGuide>>()
            .add_message::<PlayerInputEvent<CharacterListRequest>>()
            .add_message::<PlayerInputEvent<CharacterJoinRequest>>()
            .add_message::<PlayerInputEvent<AuthRequest>>()
            .add_message::<PlayerInputEvent<LogoutRequest>>()
            .add_message::<PlayerInputEvent<GameGuideResponse>>()
            .add_message::<PlayerInputEvent<HotbarUpdate>>()
            .add_message::<PlayerInputEvent<LearnSkill>>()
            .add_message::<PlayerInputEvent<LevelUpMastery>>()
            .add_message::<PlayerInputEvent<InventoryOperation>>()
            .add_message::<PlayerInputEvent<OpenItemMall>>()
            .add_message::<PlayerInputEvent<ConsignmentList>>()
            .add_message::<PlayerInputEvent<PlayerMovementRequest>>()
            .add_message::<PlayerInputEvent<IdentityInformation>>()
            .add_message::<PlayerInputEvent<Rotation>>();
    }
}
