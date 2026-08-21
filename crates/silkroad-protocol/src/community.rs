use skrillax_packet::Packet;
use skrillax_serde::*;
use skrillax_stream::registry::PacketRegistryBuilder;

#[derive(Clone, Serialize, ByteSize, Deserialize, Debug)]
pub struct GuildInformation {
    pub name: String,
    pub id: u32,
    pub member: String,
    pub last_icon_rev: u32,
    pub union_id: u32,
    pub last_union_icon_rev: u32,
    pub is_friendly: u8,
    pub siege_unknown: u8,
}

impl GuildInformation {
    pub fn new(
        name: String,
        id: u32,
        member: String,
        last_icon_rev: u32,
        union_id: u32,
        last_union_icon_rev: u32,
        is_friendly: u8,
    ) -> Self {
        GuildInformation {
            name,
            id,
            member,
            last_icon_rev,
            union_id,
            last_union_icon_rev,
            is_friendly,
            siege_unknown: 0,
        }
    }

    pub fn empty() -> Self {
        Self::new("".to_string(), 0, "".to_string(), 0, 0, 0, 0)
    }
}

impl Default for GuildInformation {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Debug)]
pub struct FriendListGroup {
    pub id: u16,
    pub name: String,
}

impl FriendListGroup {
    pub fn new(id: u16, name: String) -> Self {
        FriendListGroup { id, name }
    }

    pub fn not_assigned() -> Self {
        Self::new(0, "N/A".to_string())
    }
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Debug)]
pub struct FriendListEntry {
    pub char_id: u32,
    pub name: String,
    pub char_model: u32,
    pub group_id: u16,
    pub offline: bool,
}

impl FriendListEntry {
    pub fn new(char_id: u32, name: String, char_model: u32, group_id: u16, offline: bool) -> Self {
        FriendListEntry {
            char_id,
            name,
            char_model,
            group_id,
            offline,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0x3305)]
pub struct FriendListInfo {
    pub groups: Vec<FriendListGroup>,
    pub friends: Vec<FriendListEntry>,
}

impl FriendListInfo {
    pub fn new(groups: Vec<FriendListGroup>, friends: Vec<FriendListEntry>) -> Self {
        FriendListInfo { groups, friends }
    }
}

#[derive(Clone, Deserialize, ByteSize, Serialize, Packet, Debug)]
#[packet(opcode = 0x7302)]
pub struct AddFriend {
    pub name: String,
}

#[derive(Clone, Deserialize, ByteSize, Serialize, Packet, Debug)]
#[packet(opcode = 0x7310)]
pub struct CreateFriendGroup {
    pub name: String,
}

#[derive(Clone, Deserialize, ByteSize, Serialize, Packet, Debug)]
#[packet(opcode = 0x7304)]
pub struct DeleteFriend {
    pub friend_character_id: u32,
}
pub trait CommunityPacketRegistryExt {
    fn register_community_packets(self) -> Self;
}

impl CommunityPacketRegistryExt for PacketRegistryBuilder {
    fn register_community_packets(self) -> Self {
        self.register::<FriendListInfo>()
            .register::<AddFriend>()
            .register::<CreateFriendGroup>()
            .register::<DeleteFriend>()
    }
}
