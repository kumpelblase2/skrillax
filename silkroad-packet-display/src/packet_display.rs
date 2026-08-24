use silkroad_gateway_protocol::{
    FrameworkStateRequest, FrameworkStateUpdate, GatewayNoticeRequest, GatewayNoticeResponse, LoginRequest,
    LoginResponse, PasscodeRequiredResponse, PatchRequest, PatchResponse, PingServerRequest, PingServerResponse,
    SecurityCodeInput, SecurityCodeResponse, ShardListRequest, ShardListResponse,
};
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
    CheckTradesAllowed, ConsignmentList, ConsignmentResponse, InventoryOperation, InventoryOperationResult,
    OpenItemMall, TradesAllowedResponse,
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
    GameGuideResponse, GuildMatchingList, IncreaseInt, IncreaseStr, LevelUpEffect, LunarEventInfo,
    PlayerPickupAnimation, TargetEntity, TargetEntityResponse, UnTargetEntity, UpdateGameGuide, WeatherUpdate,
};
use silkroad_protocol::{IdentityInformation, KeepAlive};
use skrillax_stream::handshake::{HandshakeAccepted, HandshakeChallenge, SecurityCapabilityCheck};
use skrillax_stream::stream::DynamicPacket;
use std::fmt::{Debug, Display, Formatter};
use tracing::{info, warn};

struct PrettyDebug<'a, T: ?Sized>(&'a T);

pub(crate) struct HexDump<'a>(pub(crate) &'a [u8]);

impl Display for HexDump<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            return f.write_str("<empty>");
        }

        for (line, chunk) in self.0.chunks(16).enumerate() {
            if line > 0 {
                f.write_str("\n")?;
            }

            write!(f, "{:08x}  ", line * 16)?;
            for column in 0..16 {
                if column == 8 {
                    f.write_str(" ")?;
                }
                if let Some(byte) = chunk.get(column) {
                    write!(f, "{byte:02x} ")?;
                } else {
                    f.write_str("   ")?;
                }
            }

            f.write_str(" |")?;
            for byte in chunk {
                let character = if byte.is_ascii_graphic() || *byte == b' ' {
                    char::from(*byte)
                } else {
                    '.'
                };
                write!(f, "{character}")?;
            }
            f.write_str("|")?;
        }

        Ok(())
    }
}

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
            LevelUpEffect,
            PatchRequest,
            PatchResponse,
            LoginRequest,
            LoginResponse,
            PasscodeRequiredResponse,
            SecurityCodeResponse,
            GuildMatchingList,
            GatewayNoticeRequest,
            GatewayNoticeResponse,
            PingServerRequest,
            PingServerResponse,
            ShardListRequest,
            ShardListResponse,
            SecurityCodeInput,
            CheckTradesAllowed,
            TradesAllowedResponse
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

    #[test]
    fn hex_dump_displays_offsets_hex_bytes_and_ascii() {
        let bytes = b"Silkroad\0packet bytes!";

        assert_eq!(
            HexDump(bytes).to_string(),
            concat!(
                "00000000  53 69 6c 6b 72 6f 61 64  00 70 61 63 6b 65 74 20  |Silkroad.packet |\n",
                "00000010  62 79 74 65 73 21                                 |bytes!|",
            )
        );
    }

    #[test]
    fn hex_dump_identifies_an_empty_payload() {
        assert_eq!(HexDump(&[]).to_string(), "<empty>");
    }
}
