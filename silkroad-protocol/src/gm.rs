use silkroad_definitions::rarity::EntityRarity;
use silkroad_serde::*;

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

#[derive(Serialize, ByteSize, Clone)]
#[silkroad(size = 2)]
pub enum GmSuccessResult {
    #[silkroad(value = 1)]
    Message(String),
    #[silkroad(value = 4)]
    EntityIds { player: u32, mob: u32, item: u32 },
    #[silkroad(value = 0x15)]
    EventScriptRegisterOk,
    #[silkroad(value = 0x16)]
    EventScriptUnRegisterOk,
    #[silkroad(value = 0x21)]
    SiegeManagerOk,
    #[silkroad(value = 0x31)]
    ClearInventoryOk,
    #[silkroad(value = 0x38)]
    CheckMacroUserOk,
}

#[derive(Serialize, ByteSize, Clone)]
pub enum GmResponseResult {
    #[silkroad(value = 1)]
    Success(GmSuccessResult),
    #[silkroad(value = 0)]
    Error,
}

#[derive(Serialize, ByteSize, Clone)]
pub struct GmResponse {
    pub result: GmResponseResult,
}

impl GmResponse {
    pub fn success_message(message: String) -> Self {
        GmResponse {
            result: GmResponseResult::Success(GmSuccessResult::Message(message)),
        }
    }

    pub fn print_entity_ids(player_id: u32, mob_id: u32, item_id: u32) -> Self {
        GmResponse {
            result: GmResponseResult::Success(GmSuccessResult::EntityIds {
                player: player_id,
                mob: mob_id,
                item: item_id,
            }),
        }
    }
}
