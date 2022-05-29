use bytes::Bytes;
use crate::error::ProtocolError;
use crate::character::*;
use crate::world::*;
use crate::chat::*;
use crate::login::*;
use crate::general::*;
use crate::auth::*;

pub mod error;

pub mod character;

pub mod world;

pub mod chat;

pub mod login;

pub mod general;

pub mod auth;

pub enum ClientPacket {
    CharacterListRequest(CharacterListRequest),
    CharacterJoinRequest(CharacterJoinRequest),
    FinishLoading(FinishLoading),
    ConsignmentList(ConsignmentList),
    PlayerMovementRequest(PlayerMovementRequest),
    AddFriend(AddFriend),
    CreateFriendGroup(CreateFriendGroup),
    DeleteFriend(DeleteFriend),
    Rotation(Rotation),
    ChatMessage(ChatMessage),
    PatchRequest(PatchRequest),
    LoginRequest(LoginRequest),
    SecurityCodeInput(SecurityCodeInput),
    GatewayNoticeRequest(GatewayNoticeRequest),
    PingServerRequest(PingServerRequest),
    ShardListRequest(ShardListRequest),
    IdentityInformation(IdentityInformation),
    KeepAlive(KeepAlive),
    HandshakeChallenge(HandshakeChallenge),
    HandshakeAccepted(HandshakeAccepted),
    AuthRequest(AuthRequest),
    LogoutRequest(LogoutRequest),
}

impl ClientPacket {
    pub fn deserialize(opcode: u16, data: Bytes) -> Result<ClientPacket, ProtocolError> {
        match opcode {
            0x7007 => Ok(ClientPacket::CharacterListRequest(data.try_into()?)),
            0x7001 => Ok(ClientPacket::CharacterJoinRequest(data.try_into()?)),
            0x34c6 => Ok(ClientPacket::FinishLoading(data.try_into()?)),
            0x750E => Ok(ClientPacket::ConsignmentList(data.try_into()?)),
            0x7021 => Ok(ClientPacket::PlayerMovementRequest(data.try_into()?)),
            0x7302 => Ok(ClientPacket::AddFriend(data.try_into()?)),
            0x7310 => Ok(ClientPacket::CreateFriendGroup(data.try_into()?)),
            0x7304 => Ok(ClientPacket::DeleteFriend(data.try_into()?)),
            0x7024 => Ok(ClientPacket::Rotation(data.try_into()?)),
            0x7025 => Ok(ClientPacket::ChatMessage(data.try_into()?)),
            0x6100 => Ok(ClientPacket::PatchRequest(data.try_into()?)),
            0x610A => Ok(ClientPacket::LoginRequest(data.try_into()?)),
            0x6117 => Ok(ClientPacket::SecurityCodeInput(data.try_into()?)),
            0x6104 => Ok(ClientPacket::GatewayNoticeRequest(data.try_into()?)),
            0x6107 => Ok(ClientPacket::PingServerRequest(data.try_into()?)),
            0x6101 => Ok(ClientPacket::ShardListRequest(data.try_into()?)),
            0x2001 => Ok(ClientPacket::IdentityInformation(data.try_into()?)),
            0x2002 => Ok(ClientPacket::KeepAlive(data.try_into()?)),
            0x5000 => Ok(ClientPacket::HandshakeChallenge(data.try_into()?)),
            0x9000 => Ok(ClientPacket::HandshakeAccepted(data.try_into()?)),
            0x6103 => Ok(ClientPacket::AuthRequest(data.try_into()?)),
            0x7005 => Ok(ClientPacket::LogoutRequest(data.try_into()?)),
            _ => Err(ProtocolError::UnknownOpcode(opcode))
        }
    }
}

pub enum ServerPacket {
    CharacterListResponse(CharacterListResponse),
    CharacterJoinResponse(CharacterJoinResponse),
    CharacterStatsMessage(CharacterStatsMessage),
    UnknownPacket(UnknownPacket),
    UnknownPacket2(UnknownPacket2),
    CelestialUpdate(CelestialUpdate),
    LunarEventInfo(LunarEventInfo),
    CharacterSpawnStart(CharacterSpawnStart),
    CharacterSpawn(CharacterSpawn),
    CharacterSpawnEnd(CharacterSpawnEnd),
    CharacterFinished(CharacterFinished),
    EntityDespawn(EntityDespawn),
    EntitySpawn(EntitySpawn),
    GroupEntitySpawnStart(GroupEntitySpawnStart),
    GroupEntitySpawnData(GroupEntitySpawnData),
    GroupEntitySpawnEnd(GroupEntitySpawnEnd),
    ConsignmentResponse(ConsignmentResponse),
    WeatherUpdate(WeatherUpdate),
    FriendListInfo(FriendListInfo),
    GameNotification(GameNotification),
    PlayerMovementResponse(PlayerMovementResponse),
    EntityUpdateState(EntityUpdateState),
    TextCharacterInitialization(TextCharacterInitialization),
    ChatUpdate(ChatUpdate),
    ChatMessageResponse(ChatMessageResponse),
    PatchResponse(PatchResponse),
    LoginResponse(LoginResponse),
    SecurityCodeResponse(SecurityCodeResponse),
    GatewayNoticeResponse(GatewayNoticeResponse),
    PingServerResponse(PingServerResponse),
    ShardListResponse(ShardListResponse),
    PasscodeRequiredResponse(PasscodeRequiredResponse),
    PasscodeResponse(PasscodeResponse),
    QueueUpdate(QueueUpdate),
    IdentityInformation(IdentityInformation),
    ServerInfoSeed(ServerInfoSeed),
    ServerStateSeed(ServerStateSeed),
    SecuritySetup(SecuritySetup),
    AuthResponse(AuthResponse),
    LogoutResponse(LogoutResponse),
    LogoutFinished(LogoutFinished),
    Disconnect(Disconnect),
}

impl ServerPacket {
    pub fn into_serialize(self) -> (u16, Bytes) {
        match self {
            ServerPacket::CharacterListResponse(data) => (0xB007, data.into()),
            ServerPacket::CharacterJoinResponse(data) => (0xB001, data.into()),
            ServerPacket::CharacterStatsMessage(data) => (0x303D, data.into()),
            ServerPacket::UnknownPacket(data) => (0x3601, data.into()),
            ServerPacket::UnknownPacket2(data) => (0xB602, data.into()),
            ServerPacket::CelestialUpdate(data) => (0x3020, data.into()),
            ServerPacket::LunarEventInfo(data) => (0x34f2, data.into()),
            ServerPacket::CharacterSpawnStart(data) => (0x34A5, data.into()),
            ServerPacket::CharacterSpawn(data) => (0x3013, data.into()),
            ServerPacket::CharacterSpawnEnd(data) => (0x34A6, data.into()),
            ServerPacket::CharacterFinished(data) => (0x3077, data.into()),
            ServerPacket::EntityDespawn(data) => (0x3016, data.into()),
            ServerPacket::EntitySpawn(data) => (0x3015, data.into()),
            ServerPacket::GroupEntitySpawnStart(data) => (0x3017, data.into()),
            ServerPacket::GroupEntitySpawnData(data) => (0x3019, data.into()),
            ServerPacket::GroupEntitySpawnEnd(data) => (0x3018, data.into()),
            ServerPacket::ConsignmentResponse(data) => (0xB50E, data.into()),
            ServerPacket::WeatherUpdate(data) => (0x3809, data.into()),
            ServerPacket::FriendListInfo(data) => (0x3305, data.into()),
            ServerPacket::GameNotification(data) => (0x300C, data.into()),
            ServerPacket::PlayerMovementResponse(data) => (0xB021, data.into()),
            ServerPacket::EntityUpdateState(data) => (0x30BF, data.into()),
            ServerPacket::TextCharacterInitialization(data) => (0x3535, data.into()),
            ServerPacket::ChatUpdate(data) => (0x3026, data.into()),
            ServerPacket::ChatMessageResponse(data) => (0xB025, data.into()),
            ServerPacket::PatchResponse(data) => (0xA100, data.into()),
            ServerPacket::LoginResponse(data) => (0xA10A, data.into()),
            ServerPacket::SecurityCodeResponse(data) => (0xA117, data.into()),
            ServerPacket::GatewayNoticeResponse(data) => (0xA104, data.into()),
            ServerPacket::PingServerResponse(data) => (0xA107, data.into()),
            ServerPacket::ShardListResponse(data) => (0xA101, data.into()),
            ServerPacket::PasscodeRequiredResponse(data) => (0x2116, data.into()),
            ServerPacket::PasscodeResponse(data) => (0xA117, data.into()),
            ServerPacket::QueueUpdate(data) => (0x210E, data.into()),
            ServerPacket::IdentityInformation(data) => (0x2001, data.into()),
            ServerPacket::ServerInfoSeed(data) => (0x2005, data.into()),
            ServerPacket::ServerStateSeed(data) => (0x6005, data.into()),
            ServerPacket::SecuritySetup(data) => (0x5000, data.into()),
            ServerPacket::AuthResponse(data) => (0xA103, data.into()),
            ServerPacket::LogoutResponse(data) => (0xB005, data.into()),
            ServerPacket::LogoutFinished(data) => (0x300A, data.into()),
            ServerPacket::Disconnect(data) => (0x2212, data.into()),
        }
    }

    pub fn is_massive(&self) -> bool {
        matches!(self, Self::PatchResponse(_) | Self::GatewayNoticeResponse(_) | Self::ServerInfoSeed(_) | Self::ServerStateSeed(_))
    }

    pub fn is_encrypted(&self) -> bool {
        matches!(self, Self::LoginResponse(_))
    }
}