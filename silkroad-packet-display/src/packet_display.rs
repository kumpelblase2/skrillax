use silkroad_gateway_protocol::{FrameworkStateRequest, FrameworkStateUpdate, LoginResponse};
use silkroad_protocol::auth::{
    AuthRequest, AuthResponse, Disconnect, LogoutFinished, LogoutRequest, LogoutResponse, UnknownLargePacket,
};
use silkroad_protocol::character::{
    CharacterJoinRequest, CharacterJoinResponse, CharacterListRequest, CharacterListResponse, CharacterStatsMessage,
    FinishLoading, MacroStatus, UnknownPacket, UnknownPacket2,
};
use silkroad_protocol::chat::{ChatMessage, TextCharacterInitialization};
use silkroad_protocol::combat::{PerformAction, PerformActionResponse, PerformActionUpdate, ReceiveExperience};
use silkroad_protocol::community::FriendListInfo;
use silkroad_protocol::gm::GmCommand;
use silkroad_protocol::inventory::{
    ConsignmentList, ConsignmentResponse, InventoryOperation, InventoryOperationResult, OpenItemMall,
};
use silkroad_protocol::movement::{
    ChangeSpeed, EntityMovementInterrupt, PlayerMovementRequest, PlayerMovementResponse, Rotation,
};
use silkroad_protocol::skill::{HotbarUpdate, LearnSkill, LevelUpMastery};
use silkroad_protocol::spawn::{
    CharacterSpawn, CharacterSpawnEnd, CharacterSpawnStart, EntityDespawn, EntitySpawn, GroupEntitySpawnData,
    GroupEntitySpawnEnd, GroupEntitySpawnStart,
};
use silkroad_protocol::world::{
    AddQuestMarker, CelestialUpdate, CharacterFinished, CharacterPointsUpdate, EntityBarsUpdate, EntityUpdateState,
    GameGuideResponse, IncreaseInt, IncreaseStr, LevelUpEffect, LunarEventInfo, PlayerPickupAnimation, TargetEntity,
    TargetEntityResponse, UnTargetEntity, UpdateGameGuide, WeatherUpdate,
};
use silkroad_protocol::{IdentityInformation, KeepAlive};
use skrillax_stream::handshake::{HandshakeAccepted, HandshakeChallenge, SecurityCapabilityCheck};
use skrillax_stream::stream::DynamicPacket;
use std::fmt::{Debug, Display, Formatter};
use tracing::{info, warn};

struct PrettyDebug<'a, T: ?Sized>(&'a T);

impl<T: Debug + ?Sized> Display for PrettyDebug<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.0)
    }
}

fn short_packet_type_name(type_name: &str) -> &str {
    type_name.rsplit("::").next().unwrap_or(type_name)
}

fn log_decoded_packet<T: Debug + ?Sized>(packet_type: &str, packet: &T) {
    info!(
        target: "packet",
        packet_type,
        packet = %PrettyDebug(packet),
        "decoded packet",
    );
}

macro_rules! handle_packets {
    ($packet:expr, [$first_ty:ty $(, $ty:ty)* $(,)?]) => {{
        let packet_type = short_packet_type_name($packet.packet_type_name());
        if let Ok(inner) = $packet.try_as_packet::<$first_ty>() {
            log_decoded_packet(packet_type, inner);
        } $(
            else if let Ok(inner) = $packet.try_as_packet::<$ty>() {
                log_decoded_packet(packet_type, inner);
            }
        )* else {
            warn!(
                target: "packet",
                packet_type,
                "decoded packet has no display handler",
            );
        }
    }};
}

/// Logs a decoded packet using its concrete packet representation.
pub(crate) fn display_packet(packet: DynamicPacket) {
    handle_packets!(
        packet,
        [
            KeepAlive,
            CharacterJoinResponse,
            HandshakeChallenge,
            HandshakeAccepted,
            SecurityCapabilityCheck,
            IdentityInformation,
            ChatMessage,
            IncreaseInt,
            IncreaseStr,
            FinishLoading,
            GmCommand,
            TargetEntity,
            PerformAction,
            FrameworkStateUpdate,
            FrameworkStateRequest,
            LoginResponse,
            UnTargetEntity,
            UpdateGameGuide,
            CharacterListRequest,
            CharacterJoinRequest,
            AuthRequest,
            LogoutRequest,
            GameGuideResponse,
            HotbarUpdate,
            LearnSkill,
            LevelUpMastery,
            InventoryOperation,
            OpenItemMall,
            ConsignmentList,
            PlayerMovementRequest,
            Rotation,
            GroupEntitySpawnStart,
            GroupEntitySpawnData,
            GroupEntitySpawnEnd,
            UnknownLargePacket,
            AuthResponse,
            CharacterListResponse,
            CharacterSpawnStart,
            CharacterSpawn,
            CharacterSpawnEnd,
            CelestialUpdate,
            MacroStatus,
            LunarEventInfo,
            UnknownPacket,
            UnknownPacket2,
            CharacterStatsMessage,
            EntityDespawn,
            AddQuestMarker,
            EntityUpdateState,
            TextCharacterInitialization,
            FriendListInfo,
            TargetEntityResponse,
            PerformActionResponse,
            PerformActionUpdate,
            EntityBarsUpdate,
            CharacterPointsUpdate,
            ReceiveExperience,
            WeatherUpdate,
            CharacterFinished,
            PlayerMovementResponse,
            ConsignmentResponse,
            ChangeSpeed,
            LogoutResponse,
            LogoutFinished,
            Disconnect,
            EntitySpawn,
            EntityMovementInterrupt,
            PlayerPickupAnimation,
            InventoryOperationResult,
            LevelUpEffect
        ]
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pretty_debug_uses_alternate_debug_formatting() {
        #[allow(dead_code)]
        #[derive(Debug)]
        struct ExamplePacket {
            value: u8,
        }

        assert_eq!(
            format!("{}", PrettyDebug(&ExamplePacket { value: 7 })),
            "ExamplePacket {\n    value: 7,\n}"
        );
    }

    #[test]
    fn packet_type_names_do_not_include_module_paths() {
        assert_eq!(
            short_packet_type_name("silkroad_protocol::movement::PlayerMovementResponse"),
            "PlayerMovementResponse"
        );
    }
}
