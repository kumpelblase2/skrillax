use bytes::BytesMut;
use clap::Parser;
use color_eyre::eyre::{Result, eyre};
use etherparse::{SlicedPacket, TransportSlice};
use pcap::Capture;
use silkroad_data::characterdata::load_character_map;
use silkroad_definitions::type_id::ObjectType;
use silkroad_protocol::auth::{
    AuthPacketRegistryExt, AuthRequest, AuthResponse, Disconnect, LogoutFinished, LogoutRequest, LogoutResponse,
    UnknownLargePacket,
};
use silkroad_protocol::character::{
    CharacterJoinRequest, CharacterJoinResponse, CharacterListRequest, CharacterListResponse,
    CharacterPacketRegistryExt, CharacterStatsMessage, FinishLoading, MacroStatus, UnknownPacket, UnknownPacket2,
};
use silkroad_protocol::chat::{ChatMessage, ChatPacketRegistryExt, TextCharacterInitialization};
use silkroad_protocol::combat::{
    CombatPacketRegistryExt, DidLevelUp, PerformAction, PerformActionResponse, PerformActionUpdate, ReceiveExperience,
};
use silkroad_protocol::community::{CommunityPacketRegistryExt, FriendListInfo};
use silkroad_protocol::gm::{GmCommand, GmPacketRegistryExt};
use silkroad_protocol::inventory::{
    ConsignmentList, ConsignmentResponse, InventoryOperation, InventoryOperationResult, InventoryPacketRegistryExt,
    OpenItemMall,
};
use silkroad_protocol::movement::{
    ChangeSpeed, EntityMovementInterrupt, MovementPacketRegistryExt, PlayerMovementRequest, PlayerMovementResponse,
    Rotation,
};
use silkroad_protocol::skill::{HotbarUpdate, LearnSkill, LevelUpMastery, SkillPacketRegistryExt};
use silkroad_protocol::spawn::{
    CharacterSpawn, CharacterSpawnEnd, CharacterSpawnStart, EntityDespawn, EntitySpawn, GroupEntitySpawnCount,
    GroupEntitySpawnData, GroupEntitySpawnEnd, GroupEntitySpawnStart, GroupEntityType, SpawnPacketRegistryExt,
    register_ref_id,
};
use silkroad_protocol::world::{
    AddQuestMarker, CelestialUpdate, CharacterFinished, CharacterPointsUpdate, EntityBarsUpdate, EntityUpdateState,
    GameGuideResponse, IncreaseInt, IncreaseStr, LevelUpEffect, LunarEventInfo, PlayerPickupAnimation, TargetEntity,
    TargetEntityResponse, UnTargetEntity, UpdateGameGuide, WeatherUpdate, WorldPacketRegistryExt,
};
use silkroad_protocol::{BasePacketRegistryExt, IdentityInformation, KeepAlive};
use skrillax_codec::SilkroadCodec;
use skrillax_packet::{IncomingPacket, IncomingPacketReframer, Packet, ReframingLimits, SerdeContext};
use skrillax_stream::handshake::{
    HandshakeAccepted, HandshakeChallenge, HandshakePacketRegistryExt, SecurityCapabilityCheck,
};
use skrillax_stream::registry::PacketRegistry;
use skrillax_stream::stream::DynamicPacket;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;
use tokio_util::codec::Decoder;
use tracing::{debug, error, info, warn};

mod security;

use security::CaptureSecurity;
use silkroad_gateway_protocol::{FrameworkStateRequest, FrameworkStateUpdate};

pub fn maybe_hex(s: &str) -> Result<u16, String> {
    const HEX_PREFIX: &str = "0x";
    const HEX_PREFIX_UPPER: &str = "0X";
    const HEX_PREFIX_LEN: usize = HEX_PREFIX.len();

    let result = if s.starts_with(HEX_PREFIX) || s.starts_with(HEX_PREFIX_UPPER) {
        u16::from_str_radix(&s[HEX_PREFIX_LEN..], 16)
    } else {
        s.parse::<u16>()
    };

    result.map_err(|e| format!("Failed to parse hex value: {}", e))
}

/// CLI tool to display packets from a pcap file filtered by gateway IP and port
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    /// Gateway IP address
    #[clap(long, default_value = "127.0.0.1")]
    gateway: String,

    /// Gateway port
    #[clap(long, default_value = "15779")]
    gateway_port: u16,

    /// Path to the pcap file to read
    #[clap(name = "PCAP_FILE")]
    pcap_file: PathBuf,

    #[clap(long, value_parser=maybe_hex)]
    opcode: Option<u16>,

    #[clap(long)]
    silkroad_dir: PathBuf,

    /// Worker threads used to recover the captured encryption key
    #[clap(long)]
    threads: Option<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialOrd, PartialEq)]
enum Direction {
    ServerToClient,
    ClientToServer,
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ServerToClient => f.write_str("S->C"),
            Self::ClientToServer => f.write_str("C->S"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CaptureFrame {
    number: u64,
    offset_us: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PacketMetadata {
    number: u64,
    frame_number: u64,
    capture_offset_us: i64,
    direction: Direction,
}

/// Numbers capture frames and logical packets, assigning each logical packet
/// the timestamp of the capture frame that completed it.
#[derive(Default)]
struct CaptureTimeline {
    frame_number: u64,
    packet_number: u64,
    first_timestamp_us: Option<i64>,
}

impl CaptureTimeline {
    fn next_frame(&mut self, timestamp_us: i64) -> CaptureFrame {
        self.frame_number += 1;
        let first_timestamp_us = *self.first_timestamp_us.get_or_insert(timestamp_us);

        CaptureFrame {
            number: self.frame_number,
            offset_us: timestamp_us.saturating_sub(first_timestamp_us),
        }
    }

    fn next_packet(&mut self, frame: CaptureFrame, direction: Direction) -> PacketMetadata {
        self.packet_number += 1;

        PacketMetadata {
            number: self.packet_number,
            frame_number: frame.number,
            capture_offset_us: frame.offset_us,
            direction,
        }
    }
}

struct CaptureSeconds(i64);

impl Display for CaptureSeconds {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let sign = if self.0 < 0 { "-" } else { "" };
        let micros = self.0.unsigned_abs();
        write!(f, "{sign}{}.{:06}s", micros / 1_000_000, micros % 1_000_000)
    }
}

struct PrettyDebug<'a, T: ?Sized>(&'a T);

impl<T: Debug + ?Sized> Display for PrettyDebug<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.0)
    }
}

struct PacketStream {
    frame_buffer: BytesMut,
    codec: SilkroadCodec,
    reframer: IncomingPacketReframer,
}

impl Default for PacketStream {
    fn default() -> Self {
        Self {
            frame_buffer: BytesMut::new(),
            codec: SilkroadCodec,
            reframer: IncomingPacketReframer::new(ReframingLimits::recommended()),
        }
    }
}

impl PacketStream {
    /// Adds one TCP payload and returns every logical packet completed by it.
    /// Incomplete frame and massive-packet data remains here for the next
    /// payload from this same direction.
    fn push(
        &mut self,
        direction: Direction,
        payload: &[u8],
        security: &mut CaptureSecurity,
    ) -> Result<Vec<IncomingPacket>> {
        self.frame_buffer.extend_from_slice(payload);
        let mut packets = Vec::new();

        loop {
            let Some(frame) = self.codec.decode(&mut self.frame_buffer)? else {
                break;
            };

            let packet = {
                let context = match direction {
                    Direction::ServerToClient => security.server_to_client_context(),
                    Direction::ClientToServer => security.client_to_server_context(),
                };
                self.reframer.push(&frame, context)?
            };

            if let Some(packet) = packet {
                match direction {
                    Direction::ServerToClient => security.observe_server_packet(&packet)?,
                    Direction::ClientToServer => security.observe_client_packet(&packet)?,
                }
                packets.push(packet);
            }
        }

        Ok(packets)
    }
}

struct PacketStreams {
    server_to_client: PacketStream,
    client_to_server: PacketStream,
    security: CaptureSecurity,
}

impl PacketStreams {
    fn new(brute_force_threads: usize) -> Self {
        Self {
            server_to_client: PacketStream::default(),
            client_to_server: PacketStream::default(),
            security: CaptureSecurity::new(brute_force_threads),
        }
    }

    fn push(&mut self, direction: Direction, payload: &[u8]) -> Result<Vec<IncomingPacket>> {
        match direction {
            Direction::ServerToClient => self.server_to_client.push(direction, payload, &mut self.security),
            Direction::ClientToServer => self.client_to_server.push(direction, payload, &mut self.security),
        }
    }
}

impl Default for PacketStreams {
    fn default() -> Self {
        Self::new(1)
    }
}

fn packet_matches(packet: &SlicedPacket, _ips: &[IpAddr], ports: &[u16]) -> bool {
    // let Some(InternetSlice::Ipv4(ipv4)) = &packet.net else {
    //     return false;
    // };

    // if ips.len() > 0
    //     && !ips
    //         .iter()
    //         .any(|ip| ipv4.header().destination_addr().eq(ip) || ipv4.header().source_addr().eq(ip))
    // {
    //     return false;
    // }

    let Some(TransportSlice::Tcp(tcp)) = &packet.transport else {
        return false;
    };

    ports
        .iter()
        .any(|port| tcp.source_port() == *port || tcp.destination_port() == *port)
}

fn main() -> Result<()> {
    // Capture-relative timestamps are attached to packet events. Suppress the
    // wall-clock processing timestamp so it cannot be mistaken for capture time.
    tracing_subscriber::fmt().without_time().with_target(false).init();
    color_eyre::install()?;

    let client_packet_registry = PacketRegistry::builder()
        .register_base_packets()
        .register::<FrameworkStateUpdate>()
        .register::<FrameworkStateRequest>()
        .register::<CelestialUpdate>()
        .register::<LunarEventInfo>()
        .register_passive_handshake()
        .register_auth_packets()
        .register_character_packets()
        .register_chat_packets()
        .register_combat_packets()
        .register_community_packets()
        .register_gm_packets()
        .register_inventory_packets()
        .register_movement_packets()
        .register_skill_packets()
        .register_spawn_packets()
        .register_world_packets()
        .build()
        .expect("Should be able to build registry.");

    let server_packet_registry = PacketRegistry::builder()
        .register_base_packets()
        .register_active_handshake()
        .register_auth_packets()
        .register_character_packets()
        .register_chat_packets()
        .register_combat_packets()
        .register_community_packets()
        .register_gm_packets()
        .register_inventory_packets()
        .register_movement_packets()
        .register_skill_packets()
        .register_spawn_packets()
        .register_world_packets()
        .build()
        .expect("Should be able to build registry.");

    // Parse command line arguments
    let cli = Cli::parse();

    let media_path = cli.silkroad_dir.join("Media.pk2");
    info!("Using silkroad dir {:?}", media_path);
    let pk2_file = pk2_sync::Pk2::open(media_path, "169841")?;
    let characters = load_character_map(&pk2_file)?;
    let mut object_map = HashMap::<u32, ObjectType>::new();
    characters.iter().for_each(|character| {
        if let Some(object_type) = ObjectType::from_type_id(&character.common.type_id) {
            object_map.insert(character.common.ref_id, object_type);
        }
    });

    register_ref_id(object_map);

    // Parse gateway IP
    let gateway_ip =
        IpAddr::from_str(&cli.gateway).map_err(|_| eyre!("Invalid gateway IP address: {}", cli.gateway))?;

    info!("Filtering packets for gateway {}:{}", gateway_ip, cli.gateway_port);

    // Open the pcap file
    info!("Opening pcap file: {:?}", cli.pcap_file);
    let mut cap = Capture::from_file(&cli.pcap_file).map_err(|e| eyre!("Failed to open pcap file: {}", e))?;

    let brute_force_threads = cli
        .threads
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, usize::from))
        .max(1);
    let mut streams = PacketStreams::new(brute_force_threads);
    let server_to_client_context = SerdeContext::default();
    let client_to_server_context = SerdeContext::default();
    let mut timeline = CaptureTimeline::default();

    // Process each TCP payload immediately. Each direction retains only its
    // own incomplete frame/logical packet while capture order moves on.
    while let Ok(captured_packet) = cap.next_packet() {
        let capture_timestamp_us = (captured_packet.header.ts.tv_sec as i64)
            .saturating_mul(1_000_000)
            .saturating_add(captured_packet.header.ts.tv_usec as i64);
        let frame = timeline.next_frame(capture_timestamp_us);

        let Ok(packet) = SlicedPacket::from_ethernet(captured_packet.data) else {
            continue;
        };

        if !packet_matches(&packet, &[], &[cli.gateway_port]) {
            continue;
        }

        let Some(TransportSlice::Tcp(tcp)) = &packet.transport else {
            panic!("Only TCP packets should have matched.")
        };

        let content = tcp.payload();
        let direction = if tcp.source_port() == cli.gateway_port {
            Direction::ServerToClient
        } else {
            Direction::ClientToServer
        };

        debug!(?direction, bytes = content.len(), "Processing TCP payload");
        let packets = match streams.push(direction, content) {
            Ok(packets) => packets,
            Err(e) => {
                error!(?direction, "Failed to process packet: {:?}", e);
                continue;
            },
        };
        let context = match direction {
            Direction::ServerToClient => &server_to_client_context,
            Direction::ClientToServer => &client_to_server_context,
        };

        let registry = match direction {
            Direction::ServerToClient => &client_packet_registry,
            Direction::ClientToServer => &server_packet_registry,
        };

        for packet in packets {
            let metadata = timeline.next_packet(frame, direction);
            process_packet(packet, registry, context, cli.opcode, metadata);
        }
    }

    Ok(())
}

fn process_packet(
    packet: IncomingPacket,
    registry: &PacketRegistry,
    context: &SerdeContext,
    opcode: Option<u16>,
    metadata: PacketMetadata,
) {
    let packet_opcode = packet.opcode();
    let packet_size = packet.data().len();
    let packet_span = tracing::info_span!(
        target: "packet",
        "packet",
        number = metadata.number,
        frame_number = metadata.frame_number,
        time = %CaptureSeconds(metadata.capture_offset_us),
        direction = %metadata.direction,
        opcode = format_args!("{packet_opcode:#06x}"),
        size = packet_size,
    );
    let _packet_span = packet_span.enter();

    let (consumed_bytes, parsed) = match registry.decode(packet_opcode, packet.data(), context) {
        Ok((consumed_bytes, parsed)) => (consumed_bytes, parsed),
        Err(e) => {
            error!(error = ?e, "failed to decode packet");
            return;
        },
    };

    if consumed_bytes != packet_size {
        warn!(
            consumed_bytes,
            remaining_bytes = packet_size.saturating_sub(consumed_bytes),
            "packet was not fully decoded",
        );
    }

    if parsed.opcode() == GroupEntitySpawnStart::ID {
        let Some(spawn) = parsed.as_packet::<GroupEntitySpawnStart>() else {
            return;
        };
        context.set::<GroupEntityType>(spawn.kind.into());
        context.set(GroupEntitySpawnCount(spawn.amount as usize));
    } else if parsed.opcode() == LevelUpEffect::ID {
        context.set(DidLevelUp);
    } else if parsed.opcode() == ReceiveExperience::ID {
        context.unset::<DidLevelUp>();
    }

    if opcode.is_none_or(|opcode| opcode == packet_opcode) {
        display_packet(parsed);
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

fn display_packet(packet: DynamicPacket) {
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
    use bytes::Bytes;
    use skrillax_codec::SilkroadFrame;

    fn serialize(frame: SilkroadFrame) -> Bytes {
        frame.serialize().expect("test frame should serialize")
    }

    fn frame(opcode: u16, data: &'static [u8]) -> Bytes {
        serialize(SilkroadFrame::Packet {
            count: 0,
            crc: 0,
            opcode,
            data: Bytes::from_static(data),
        })
    }

    #[test]
    fn capture_timeline_tracks_frame_and_logical_packet_timing() {
        let mut timeline = CaptureTimeline::default();

        // This frame establishes capture time zero but emits no logical packet.
        timeline.next_frame(1_000_000);
        let completion_frame = timeline.next_frame(1_250_000);
        let first = timeline.next_packet(completion_frame, Direction::ServerToClient);

        assert_eq!(
            first,
            PacketMetadata {
                number: 1,
                frame_number: 2,
                capture_offset_us: 250_000,
                direction: Direction::ServerToClient,
            }
        );

        let next_frame = timeline.next_frame(1_252_500);
        let second = timeline.next_packet(next_frame, Direction::ClientToServer);
        let third = timeline.next_packet(next_frame, Direction::ClientToServer);

        assert_eq!(second.number, 2);
        assert_eq!(second.frame_number, 3);
        assert_eq!(second.capture_offset_us, 252_500);
        assert_eq!(third.number, 3);
        assert_eq!(third.capture_offset_us, 252_500);
    }

    #[test]
    fn capture_time_is_formatted_in_seconds() {
        assert_eq!(CaptureSeconds(12_438_921).to_string(), "12.438921s");
        assert_eq!(CaptureSeconds(4_212).to_string(), "0.004212s");
    }

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
    fn interleaved_directions_keep_partial_frames_separate() {
        let server_frame = frame(0x1001, b"server");
        let client_frame = frame(0x2002, b"client");
        let split_at = server_frame.len() / 2;
        let mut streams = PacketStreams::default();
        let mut observed = Vec::new();

        observed.extend(
            streams
                .push(Direction::ServerToClient, &server_frame[..split_at])
                .unwrap()
                .into_iter()
                .map(|packet| (Direction::ServerToClient, packet)),
        );
        observed.extend(
            streams
                .push(Direction::ClientToServer, &client_frame)
                .unwrap()
                .into_iter()
                .map(|packet| (Direction::ClientToServer, packet)),
        );
        observed.extend(
            streams
                .push(Direction::ServerToClient, &server_frame[split_at..])
                .unwrap()
                .into_iter()
                .map(|packet| (Direction::ServerToClient, packet)),
        );

        assert_eq!(observed.len(), 2);
        assert_eq!(observed[0].0, Direction::ClientToServer);
        assert_eq!(observed[0].1.opcode(), 0x2002);
        assert_eq!(observed[0].1.data(), b"client");
        assert_eq!(observed[1].0, Direction::ServerToClient);
        assert_eq!(observed[1].1.opcode(), 0x1001);
        assert_eq!(observed[1].1.data(), b"server");
    }

    #[test]
    fn interleaved_directions_keep_incomplete_massive_packets_separate() {
        let header = serialize(SilkroadFrame::MassiveHeader {
            count: 0,
            crc: 0,
            contained_opcode: 0x1001,
            contained_count: 2,
        });
        let first_container = serialize(SilkroadFrame::MassiveContainer {
            count: 0,
            crc: 0,
            inner: Bytes::from_static(b"massive "),
        });
        let second_container = serialize(SilkroadFrame::MassiveContainer {
            count: 0,
            crc: 0,
            inner: Bytes::from_static(b"packet"),
        });
        let client_frame = frame(0x2002, b"client");
        let mut streams = PacketStreams::default();

        assert!(streams.push(Direction::ServerToClient, &header).unwrap().is_empty());
        let client_packets = streams.push(Direction::ClientToServer, &client_frame).unwrap();
        assert_eq!(client_packets.len(), 1);
        assert_eq!(client_packets[0].data(), b"client");
        assert!(
            streams
                .push(Direction::ServerToClient, &first_container)
                .unwrap()
                .is_empty()
        );

        let server_packets = streams.push(Direction::ServerToClient, &second_container).unwrap();
        assert_eq!(server_packets.len(), 1);
        assert_eq!(server_packets[0].opcode(), 0x1001);
        assert_eq!(server_packets[0].data(), b"massive packet");
    }

    #[test]
    fn one_capture_payload_emits_all_complete_packets_in_order() {
        let first = frame(0x1001, b"first");
        let second = frame(0x1002, b"second");
        let mut payload = BytesMut::new();
        payload.extend_from_slice(&first);
        payload.extend_from_slice(&second);

        let packets = PacketStreams::default()
            .push(Direction::ServerToClient, &payload)
            .unwrap();

        assert_eq!(packets.len(), 2);
        assert_eq!(packets[0].opcode(), 0x1001);
        assert_eq!(packets[0].data(), b"first");
        assert_eq!(packets[1].opcode(), 0x1002);
        assert_eq!(packets[1].data(), b"second");
    }
}
