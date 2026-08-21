use skrillax_packet::Packet;
use skrillax_serde::*;
use skrillax_stream::registry::PacketRegistryBuilder;

#[derive(Deserialize, Serialize, ByteSize, Copy, Clone, Packet, Debug)]
#[packet(opcode = 0x70A2)]
pub struct LevelUpMastery {
    pub mastery: u32,
    pub amount: u8,
}

#[derive(Serialize, ByteSize, Deserialize, Copy, Clone, Debug)]
#[silkroad(size = 2)]
pub enum LevelUpMasteryError {
    #[silkroad(value = 0x3802)]
    InsufficientSP,
    #[silkroad(value = 0x3804)]
    MasteryMaxLevel,
    #[silkroad(value = 0x3805)]
    ReachedTotalLimit,
}

#[derive(Serialize, ByteSize, Deserialize, Copy, Clone, Packet, Debug)]
#[packet(opcode = 0xB0A2)]
pub enum LevelUpMasteryResponse {
    #[silkroad(value = 1)]
    Success { mastery: u32, new_level: u8 },
    #[silkroad(value = 2)]
    Failure(LevelUpMasteryError),
}

#[derive(Deserialize, Serialize, ByteSize, Copy, Clone, Packet, Debug)]
#[packet(opcode = 0x70A1)]
pub struct LearnSkill(pub u32);

#[derive(Serialize, ByteSize, Deserialize, Copy, Clone, Packet, Debug)]
#[packet(opcode = 0xB0A1)]
pub enum LearnSkillResponse {
    #[silkroad(value = 1)]
    Success(u32),
    #[silkroad(value = 2)]
    Failure(LevelUpMasteryError), // TODO
}

#[derive(Clone, Copy, Serialize, ByteSize, Deserialize, Debug)]
pub struct MasteryData {
    pub id: u32,
    pub level: u8,
}

impl MasteryData {
    pub fn new(id: u32, level: u8) -> Self {
        MasteryData { id, level }
    }
}

#[derive(Clone, Copy, Serialize, ByteSize, Deserialize, Debug)]
pub struct SkillData {
    pub id: u32,
    pub flag: u8,
}

impl SkillData {
    pub fn new(id: u32, flag: u8) -> Self {
        SkillData { id, flag }
    }
}

#[derive(Packet, Deserialize, Serialize, ByteSize, Debug, Clone)]
#[packet(opcode = 0x7156)]
pub struct HotbarUpdate {
    pub size: u32, // not sure why this is needed?
    pub content: Vec<HotbarItem>,
}

#[derive(Deserialize, Serialize, ByteSize, Debug, Clone)]
pub struct HotbarItem {
    pub slot: u8,
    pub action_flag: u8,
    // The meaning of the `action_data` changes depending on the above `action_flag`
    pub action_data: u32,
}

pub trait SkillPacketRegistryExt {
    fn register_skill_packets(self) -> Self;
}

impl SkillPacketRegistryExt for PacketRegistryBuilder {
    fn register_skill_packets(self) -> Self {
        self.register::<LearnSkill>()
            .register::<LearnSkillResponse>()
            .register::<LevelUpMastery>()
            .register::<LevelUpMasteryResponse>()
            .register::<HotbarUpdate>()
    }
}
