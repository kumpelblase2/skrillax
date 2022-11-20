use crate::auth::*;
use crate::character::*;
use crate::chat::*;
use crate::combat::*;
use crate::error::ProtocolError;
use crate::general::*;
use crate::gm::*;
use crate::login::*;
use crate::world::*;
use bytes::Bytes;

pub mod auth;
pub mod character;
pub mod chat;
pub mod combat;
pub mod error;
pub mod general;
pub mod gm;
pub mod inventory;
pub mod login;
pub mod world;

pub use silkroad_serde::SilkroadTime;

use crate::inventory::*;

macro_rules! client_packets {
    ($($opcode:literal => $name:ident),*) => {
        pub enum ClientPacket {
            $($name($name)),*
        }

        impl ClientPacket {
            pub fn deserialize(opcode: u16, data: Bytes) -> Result<ClientPacket, ProtocolError> {
                match opcode {
                    $($opcode => Ok(ClientPacket::$name(data.try_into()?)),)*
                    _ => Err(ProtocolError::UnknownOpcode(opcode)),
                }
            }
        }

        $(
            impl From<$name> for ClientPacket {
                fn from(other: $name) -> Self {
                    ClientPacket::$name(other)
                }
            }
        )*
    };
}

client_packets! {
    0x7007 => CharacterListRequest,
    0x7001 => CharacterJoinRequest,
    0x34c6 => FinishLoading,
    0x750E => ConsignmentList,
    0x7021 => PlayerMovementRequest,
    0x7302 => AddFriend,
    0x7310 => CreateFriendGroup,
    0x7304 => DeleteFriend,
    0x7024 => Rotation,
    0x7045 => TargetEntity,
    0x704B => UnTargetEntity,
    0x7034 => InventoryOperation,
    0x7025 => ChatMessage,
    0x6100 => PatchRequest,
    0x610A => LoginRequest,
    0x6117 => SecurityCodeInput,
    0x6104 => GatewayNoticeRequest,
    0x6107 => PingServerRequest,
    0x6101 => ShardListRequest,
    0x2001 => IdentityInformation,
    0x2002 => KeepAlive,
    0x5000 => HandshakeChallenge,
    0x9000 => HandshakeAccepted,
    0x6103 => AuthRequest,
    0x7005 => LogoutRequest,
    0x7010 => GmCommand,
    0x755D => OpenItemMall,
    0x7074 => PerformAction,
    0x70EA => UpdateGameGuide
}

macro_rules! server_packets {
    ($($opcode:literal => $name:ident),*) => {
        pub enum ServerPacket {
            $($name($name)),*
        }

        impl ServerPacket {
            pub fn into_serialize(self) -> (u16, Bytes) {
                match self {
                    $(ServerPacket::$name(data) => ($opcode, data.into()),)*
                }
            }
        }

        $(
            impl From<$name> for ServerPacket {
                fn from(other: $name) -> Self {
                    ServerPacket::$name(other)
                }
            }
        )*
    };
}

server_packets! {
    0xB007 => CharacterListResponse,
    0xB001 => CharacterJoinResponse,
    0x303D => CharacterStatsMessage,
    0x3601 => UnknownPacket,
    0xB602 => UnknownPacket2,
    0x3020 => CelestialUpdate,
    0x34f2 => LunarEventInfo,
    0x34A5 => CharacterSpawnStart,
    0x3013 => CharacterSpawn,
    0x34A6 => CharacterSpawnEnd,
    0x3077 => CharacterFinished,
    0x3016 => EntityDespawn,
    0x3015 => EntitySpawn,
    0x3017 => GroupEntitySpawnStart,
    0x3019 => GroupEntitySpawnData,
    0x3018 => GroupEntitySpawnEnd,
    0xB50E => ConsignmentResponse,
    0x3809 => WeatherUpdate,
    0x3305 => FriendListInfo,
    0x300C => GameNotification,
    0xB021 => PlayerMovementResponse,
    0x30BF => EntityUpdateState,
    0xB045 => TargetEntityResponse,
    0xB04B => UnTargetEntityResponse,
    0x3535 => TextCharacterInitialization,
    0x3555 => MacroStatus,
    0x3026 => ChatUpdate,
    0xB025 => ChatMessageResponse,
    0xA100 => PatchResponse,
    0xA10A => LoginResponse,
    0xA117 => SecurityCodeResponse,
    0xA104 => GatewayNoticeResponse,
    0xA107 => PingServerResponse,
    0xA101 => ShardListResponse,
    0x2116 => PasscodeRequiredResponse,
    0xA117 => PasscodeResponse,
    0x210E => QueueUpdate,
    0x2001 => IdentityInformation,
    0x5000 => SecuritySetup,
    0xA103 => AuthResponse,
    0xB005 => LogoutResponse,
    0x300A => LogoutFinished,
    0x2212 => Disconnect,
    0x3057 => EntityBarsUpdate,
    0xB034 => InventoryOperationResult,
    0xB010 => GmResponse,
    0xB55D => OpenItemMallResponse,
    0xB074 => PerformActionResponse,
    0xB070 => PerformActionUpdate,
    0x304E => CharacterPointsUpdate,
    0xB0EA => GameGuideResponse,
    0xB023 => EntityMovementInterrupt
}

impl ServerPacket {
    pub fn is_massive(&self) -> bool {
        matches!(self, Self::PatchResponse(_) | Self::GatewayNoticeResponse(_))
    }

    pub fn is_encrypted(&self) -> bool {
        matches!(self, Self::LoginResponse(_))
    }
}
