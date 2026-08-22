use bytes::BytesMut;
use clap::Parser;
use color_eyre::eyre::{Result, eyre};
use etherparse::{SlicedPacket, TransportSlice};
use pcap::Capture;
use silkroad_data::characterdata::load_character_map;
use silkroad_definitions::type_id::ObjectType;
use silkroad_protocol::BasePacketRegistryExt;
use silkroad_protocol::auth::AuthPacketRegistryExt;
use silkroad_protocol::character::CharacterPacketRegistryExt;
use silkroad_protocol::chat::ChatPacketRegistryExt;
use silkroad_protocol::combat::{CombatPacketRegistryExt, DidLevelUp, ReceiveExperience};
use silkroad_protocol::community::CommunityPacketRegistryExt;
use silkroad_protocol::gm::GmPacketRegistryExt;
use silkroad_protocol::inventory::InventoryPacketRegistryExt;
use silkroad_protocol::movement::MovementPacketRegistryExt;
use silkroad_protocol::skill::SkillPacketRegistryExt;
use silkroad_protocol::spawn::{
    GroupEntitySpawnCount, GroupEntitySpawnStart, GroupEntityType, SpawnPacketRegistryExt, register_ref_id,
};
use silkroad_protocol::world::{CelestialUpdate, LevelUpEffect, LunarEventInfo, WorldPacketRegistryExt};
use skrillax_codec::SilkroadCodec;
use skrillax_packet::{IncomingPacket, IncomingPacketReframer, Packet, ReframingLimits, SerdeContext};
use skrillax_stream::handshake::HandshakePacketRegistryExt;
use skrillax_stream::registry::PacketRegistry;
use skrillax_stream::stream::DynamicPacket;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use tokio_util::codec::Decoder;
use tracing::{debug, error, info, warn};

mod capture_filter;
mod packet_display;
mod security;

use capture_filter::{CaptureFilter, ConnectionKey, Direction};
use packet_display::display_packet;
use security::CaptureSecurity;
use silkroad_gateway_protocol::{FrameworkStateRequest, FrameworkStateUpdate, LoginResponse, LoginResult};

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

/// CLI tool to display gateway packets and follow the agent endpoint announced at login
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    /// Gateway port
    #[clap(long, default_value = "15779")]
    gateway_port: u16,

    /// Agent port to follow without waiting for a successful login response
    #[clap(long)]
    agent_port: Option<u16>,

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

struct ConnectionState {
    streams: PacketStreams,
    server_to_client_context: SerdeContext,
    client_to_server_context: SerdeContext,
}

impl ConnectionState {
    fn new(brute_force_threads: usize) -> Self {
        Self {
            streams: PacketStreams::new(brute_force_threads),
            server_to_client_context: SerdeContext::default(),
            client_to_server_context: SerdeContext::default(),
        }
    }
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
        .register::<LoginResponse>()
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

    let mut packet_filter = CaptureFilter::new(cli.gateway_port, cli.agent_port);
    match cli.agent_port {
        Some(agent_port) => info!(
            gateway_port = cli.gateway_port,
            agent_port, "Filtering gateway and agent packets by port"
        ),
        None => info!(
            gateway_port = cli.gateway_port,
            "Filtering gateway packets until a successful login identifies an agent endpoint"
        ),
    }

    // Open the pcap file
    info!("Opening pcap file: {:?}", cli.pcap_file);
    let mut cap = Capture::from_file(&cli.pcap_file).map_err(|e| eyre!("Failed to open pcap file: {}", e))?;

    let brute_force_threads = cli
        .threads
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, usize::from))
        .max(1);
    let mut connections = HashMap::<ConnectionKey, ConnectionState>::new();
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

        let Some(matched) = packet_filter.match_packet(&packet) else {
            continue;
        };
        let Some(TransportSlice::Tcp(tcp)) = &packet.transport else {
            unreachable!("only TCP packets can match the capture filter")
        };

        let direction = matched.direction;
        let connection = connections
            .entry(matched.connection)
            .or_insert_with(|| ConnectionState::new(brute_force_threads));
        let content = tcp.payload();

        debug!(?direction, bytes = content.len(), "Processing TCP payload");
        let packets = match connection.streams.push(direction, content) {
            Ok(packets) => packets,
            Err(e) => {
                error!(?direction, "Failed to process packet: {:?}", e);
                continue;
            },
        };
        let context = match direction {
            Direction::ServerToClient => &connection.server_to_client_context,
            Direction::ClientToServer => &connection.client_to_server_context,
        };

        let registry = match direction {
            Direction::ServerToClient => &client_packet_registry,
            Direction::ClientToServer => &server_packet_registry,
        };

        for packet in packets {
            let metadata = timeline.next_packet(frame, direction);
            if let Some(agent_endpoint) = process_packet(packet, registry, context, cli.opcode, metadata)
                && packet_filter.add_agent_endpoint(agent_endpoint)
            {
                info!(%agent_endpoint, "Following agent endpoint announced by successful login");
            }
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
) -> Option<SocketAddr> {
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
            return None;
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
        let spawn = parsed.as_packet::<GroupEntitySpawnStart>()?;
        context.set::<GroupEntityType>(spawn.kind.into());
        context.set(GroupEntitySpawnCount(spawn.amount as usize));
    } else if parsed.opcode() == LevelUpEffect::ID {
        context.set(DidLevelUp);
    } else if parsed.opcode() == ReceiveExperience::ID {
        context.unset::<DidLevelUp>();
    }

    let agent_endpoint = successful_login_endpoint(&parsed);

    if opcode.is_none_or(|opcode| opcode == packet_opcode) {
        display_packet(parsed);
    }

    agent_endpoint
}

fn successful_login_endpoint(packet: &DynamicPacket) -> Option<SocketAddr> {
    let response = packet.as_packet::<LoginResponse>()?;
    let LoginResult::Success {
        agent_ip, agent_port, ..
    } = &response.result
    else {
        return None;
    };

    match agent_ip.parse::<IpAddr>() {
        Ok(agent_ip) => Some(SocketAddr::new(agent_ip, *agent_port)),
        Err(error) => {
            warn!(agent_ip, error = %error, "successful login contained an invalid agent IP address");
            None
        },
    }
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
    fn cli_defaults_to_gateway_port_without_an_agent_port() {
        let cli = Cli::try_parse_from(["packet-display", "--silkroad-dir", "/tmp/silkroad", "capture.pcap"]).unwrap();

        assert_eq!(cli.gateway_port, 15779);
        assert_eq!(cli.agent_port, None);
    }

    #[test]
    fn successful_login_supplies_the_agent_endpoint() {
        let packet = DynamicPacket::from(LoginResponse::new(LoginResult::success(
            42,
            "10.0.0.2".to_owned(),
            15780,
        )));

        assert_eq!(
            successful_login_endpoint(&packet),
            Some("10.0.0.2:15780".parse().unwrap())
        );
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
