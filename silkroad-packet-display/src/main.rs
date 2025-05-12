use bytes::BytesMut;
use clap::Parser;
use color_eyre::eyre::{Result, eyre};
use etherparse::{SlicedPacket, TransportSlice};
use log::{error, warn};
use pcap::Capture;
use silkroad_data::characterdata::load_character_map;
use silkroad_definitions::type_id::ObjectType;
use silkroad_protocol::auth::{AuthPacketRegistryExt, AuthRequest, LogoutRequest, UnknownLargePacket};
use silkroad_protocol::character::{
    CharacterJoinRequest, CharacterListRequest, CharacterPacketRegistryExt, FinishLoading,
};
use silkroad_protocol::chat::{ChatMessage, ChatPacketRegistryExt};
use silkroad_protocol::combat::{CombatPacketRegistryExt, DidLevelUp, PerformAction, ReceiveExperience};
use silkroad_protocol::community::CommunityPacketRegistryExt;
use silkroad_protocol::gm::{GmCommand, GmPacketRegistryExt};
use silkroad_protocol::inventory::{ConsignmentList, InventoryOperation, InventoryPacketRegistryExt, OpenItemMall};
use silkroad_protocol::movement::{MovementPacketRegistryExt, PlayerMovementRequest, Rotation};
use silkroad_protocol::skill::{HotbarUpdate, LearnSkill, LevelUpMastery, SkillPacketRegistryExt};
use silkroad_protocol::spawn::{
    GroupEntitySpawnCount, GroupEntitySpawnData, GroupEntitySpawnEnd, GroupEntitySpawnStart, GroupEntityType,
    SpawnPacketRegistryExt, register_ref_id,
};
use silkroad_protocol::world::{
    GameGuideResponse, IncreaseInt, IncreaseStr, LevelUpEffect, TargetEntity, UnTargetEntity, UpdateGameGuide,
    WorldPacketRegistryExt,
};
use silkroad_protocol::{BasePacketRegistryExt, IdentityInformation};
use skrillax_codec::{SilkroadCodec, SilkroadFrame};
use skrillax_packet::{FromFrames, IncomingPacket, Packet, ReframingError, SecurityContext, SerdeContext};
use skrillax_stream::handshake::{
    HandshakeAccepted, HandshakeChallenge, HandshakePacketRegistryExt, SecurityCapabilityCheck,
};
use skrillax_stream::registry::PacketRegistry;
use skrillax_stream::stream::DynamicPacket;
use std::collections::HashMap;
use std::fmt::Debug;
use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;
use tokio_util::codec::Decoder;
use tracing::{debug, info};

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
}

#[derive(Eq, PartialOrd, PartialEq)]
enum Direction {
    ServerToClient,
    ClientToServer,
}

fn packet_matches(packet: &SlicedPacket, ips: &[IpAddr], ports: &[u16]) -> bool {
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
    // Initialize tracing
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let client_packet_registry = PacketRegistry::builder()
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

    // Create buffers for each direction
    let mut server_to_client_buffer = BytesMut::new();
    let mut client_to_server_buffer = BytesMut::new();

    // Process each packet
    while let Ok(packet) = cap.next_packet() {
        let Ok(packet) = SlicedPacket::from_ethernet(packet.data) else {
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

        // Accumulate packet data into the appropriate buffer
        match direction {
            Direction::ServerToClient => {
                server_to_client_buffer.extend_from_slice(content);
                debug!("Added {} bytes to server-to-client buffer", content.len());
            },
            Direction::ClientToServer => {
                client_to_server_buffer.extend_from_slice(content);
                debug!("Added {} bytes to client-to-server buffer", content.len());
            },
        }
    }

    let mut codec = SilkroadCodec;
    let mut server_to_client_frames: Vec<SilkroadFrame> = Vec::new();
    loop {
        let next_frame = codec.decode(&mut server_to_client_buffer);
        match next_frame {
            Ok(Some(f)) => {
                server_to_client_frames.push(f);
            },
            Ok(None) => {
                break;
            },
            Err(e) => {
                error!("Encountered error while decoding: {}", e);
                break;
            },
        }
    }

    let mut frames = &server_to_client_frames[..];
    let context = SerdeContext::default();

    while !frames.is_empty() {
        let parsed = IncomingPacket::from_frames(frames, SecurityContext::default());
        let (packet, remaining) = match parsed {
            Ok(skrillax_packet) => skrillax_packet,
            Err(ReframingError::Incomplete(missing)) => {
                warn!("Incomplete packet: missing {:?}", missing);
                break;
            },
            Err(e) => {
                error!("Failed to parse packet: {}", e);
                frames = &frames[1..];
                continue;
            },
        };

        frames = remaining;

        let Ok((remaining_bytes, parsed)) = client_packet_registry.decode(packet.opcode(), packet.data(), &context)
        else {
            error!("Failed to parse packet: {:4x}", packet.opcode());
            continue;
        };
        if remaining_bytes != 0 {
            warn!("Packet was not fully decoded: {}", remaining_bytes);
        }

        // let parsed = FullClientProtocol::create_from(packet.opcode(), packet.data(), context.clone());
        if parsed.opcode() == GroupEntitySpawnStart::ID {
            let Some(spawn) = parsed.as_packet::<GroupEntitySpawnStart>() else {
                continue;
            };
            context.set::<GroupEntityType>(spawn.kind.into());
            context.set(GroupEntitySpawnCount(spawn.amount as usize));
        } else if parsed.opcode() == LevelUpEffect::ID {
            context.set(DidLevelUp);
        } else if parsed.opcode() == ReceiveExperience::ID {
            context.unset::<DidLevelUp>();
        }

        if cli.opcode.is_some() && cli.opcode.unwrap() != packet.opcode() {
            continue;
        }

        let _ = display_packet(parsed);
    }
    // Here you would process the accumulated buffers
    // For now, we just log the buffer sizes

    Ok(())
}

macro_rules! handle_packets {
    ($packet:expr, [$( $ty:ty ),*]) => {{
        match $packet.opcode() {
            $(
                <$ty>::ID => {
                    let inner = $packet.into_packet::<$ty>()?;
                    println!("{:?}", inner);
                }
            )*
            unhandled => {
                println!("Unhandled packet opcode: {:4x}", unhandled);
            }
        }
    }};
}

fn display_packet(packet: DynamicPacket) -> Result<(), DynamicPacket> {
    handle_packets!(
        packet,
        [
            // HandshakeChallenge,
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
            UnknownLargePacket
        ]
    );
    Ok(())
}
