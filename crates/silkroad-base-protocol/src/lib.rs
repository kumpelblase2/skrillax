use skrillax_packet::Packet;
use skrillax_serde::*;
use skrillax_stream::registry::PacketRegistryBuilder;

#[derive(Clone, Serialize, ByteSize, Deserialize, Packet, Debug)]
#[packet(opcode = 0x2001)]
pub struct IdentityInformation {
    pub module_name: String,
    pub locality: u8,
    #[silkroad(when = "module_name == \"AgentServer\"")]
    pub port: Option<u16>,
}

impl IdentityInformation {
    pub fn new(module_name: String, locality: u8) -> Self {
        IdentityInformation {
            module_name,
            locality,
            port: None,
        }
    }

    pub fn new_agent(locality: u8, port: u16) -> Self {
        IdentityInformation {
            module_name: "AgentServer".to_string(),
            locality,
            port: Some(port),
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0x2002)]
pub struct KeepAlive;

pub trait BasePacketRegistryExt {
    fn register_base_packets(self) -> Self;
}

impl BasePacketRegistryExt for PacketRegistryBuilder {
    fn register_base_packets(self) -> Self {
        self.register::<IdentityInformation>().register::<KeepAlive>()
    }
}
