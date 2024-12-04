use skrillax_packet::Packet;
use skrillax_protocol::define_protocol;
use skrillax_serde::*;

#[derive(Clone, Serialize, ByteSize, Deserialize, Packet, Debug)]
#[packet(opcode = 0x2001)]
pub struct IdentityInformation {
    pub module_name: String,
    pub locality: u8,
}

impl IdentityInformation {
    pub fn new(module_name: String, locality: u8) -> Self {
        IdentityInformation { module_name, locality }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0x2002)]
pub struct KeepAlive;

define_protocol! { BaseProtocol =>
    IdentityInformation,
    KeepAlive
}
