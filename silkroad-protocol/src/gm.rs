use crate::EntityRarity;
use silkroad_serde::*;
use silkroad_serde_derive::*;

#[derive(Deserialize, ByteSize)]
#[silkroad(size = 2)]
pub enum GmCommand {
    #[silkroad(value = 0x0D)]
    BanUser { name: String },
    #[silkroad(value = 0x06)]
    SpawnMonster {
        ref_id: u32,
        amount: u8,
        rarity: EntityRarity,
    },
    #[silkroad(value = 0x0E)]
    Invisible,
    #[silkroad(value = 0x0F)]
    Invincible,
    #[silkroad(value = 0x07)]
    MakeItem { ref_id: u32, upgrade: u8 },
    #[silkroad(value = 0x0B)]
    KillMonster { unique_id: u32, unknown: u8 },
}

#[derive(Serialize, ByteSize)]
pub struct GmResponse {
    pub result: bool,
}
