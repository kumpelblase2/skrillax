use silkroad_serde::*;

#[derive(Clone, Deserialize, ByteSize)]
pub enum InventoryOperationRequest {
    #[silkroad(value = 0x00)]
    Move { source: u8, target: u8, amount: u16 },
    #[silkroad(value = 0x0A)]
    DropGold { amount: u64 },
    #[silkroad(value = 0x06)]
    PickupItem { unique_id: u32 },
    #[silkroad(value = 0x07)]
    DropItem { slot: u8 },
}

impl InventoryOperationRequest {
    pub fn dropgold(amount: u64) -> Self {
        InventoryOperationRequest::DropGold { amount }
    }
}

#[derive(Clone, Serialize, Deserialize, ByteSize)]
#[silkroad(size = 4)]
pub enum RentInfo {
    #[silkroad(value = 0)]
    Empty,
    #[silkroad(value = 1)]
    First { can_delete: u16, start: u64, end: u64 },
    #[silkroad(value = 2)]
    Second {
        can_delete: u16,
        can_recharge: u16,
        meter_rate: u32,
    },
    #[silkroad(value = 3)]
    Third {
        can_delete: u16,
        can_recharge: u16,
        start: u64,
        end: u64,
        pickup: u64,
    },
}

impl RentInfo {
    pub fn first(can_delete: u16, start: u64, end: u64) -> Self {
        RentInfo::First { can_delete, start, end }
    }

    pub fn second(can_delete: u16, can_recharge: u16, meter_rate: u32) -> Self {
        RentInfo::Second {
            can_delete,
            can_recharge,
            meter_rate,
        }
    }

    pub fn third(can_delete: u16, can_recharge: u16, start: u64, end: u64, pickup: u64) -> Self {
        RentInfo::Third {
            can_delete,
            can_recharge,
            start,
            end,
            pickup,
        }
    }
}

#[derive(Clone, Serialize, ByteSize)]
#[silkroad(size = 0)]
pub enum ItemPickupData {
    Gold {
        amount: u32,
    },
    Item {
        rent: RentInfo,
        ref_id: u32,
        content: InventoryItemContentData,
    },
    General,
}

impl ItemPickupData {
    pub fn gold(amount: u32) -> Self {
        ItemPickupData::Gold { amount }
    }

    pub fn item(rent: RentInfo, ref_id: u32, content: InventoryItemContentData) -> Self {
        ItemPickupData::Item { rent, ref_id, content }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub enum InventoryOperationResponseData {
    #[silkroad(value = 0x00)]
    UpdateSlots {
        source_slot: u8,
        target_slot: u8,
        amount: u16,
        unknown: Option<u8>,
    },
    #[silkroad(value = 0x0A)]
    DropGold { amount: u64 },
    #[silkroad(value = 0x06)]
    PickupItem { slot: u8, item: ItemPickupData },
    #[silkroad(value = 0x0e)]
    AddedByServer {
        slot: u8,
        unknown: u8,
        data: ItemPickupData,
    },
}

impl InventoryOperationResponseData {
    pub fn dropgold(amount: u64) -> Self {
        InventoryOperationResponseData::DropGold { amount }
    }

    pub fn pickupitem(slot: u8, item: ItemPickupData) -> Self {
        InventoryOperationResponseData::PickupItem { slot, item }
    }

    pub fn move_item(source: u8, dest: u8, amount: u16) -> Self {
        InventoryOperationResponseData::UpdateSlots {
            source_slot: source,
            target_slot: dest,
            amount,
            unknown: None,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Copy, Serialize, ByteSize)]
#[silkroad(size = 2)]
pub enum ConsignmentErrorCode {
    #[silkroad(value = 0x700D)]
    NotEnoughGold,
}

#[derive(Clone, Serialize, ByteSize)]
pub enum ConsignmentResult {
    #[silkroad(value = 1)]
    Success { items: Vec<ConsignmentItem> },
    #[silkroad(value = 2)]
    Error { code: ConsignmentErrorCode },
}

impl ConsignmentResult {
    pub fn success(items: Vec<ConsignmentItem>) -> Self {
        ConsignmentResult::Success { items }
    }

    pub fn error(code: ConsignmentErrorCode) -> Self {
        ConsignmentResult::Error { code }
    }
}

#[derive(Clone, Serialize, ByteSize)]
#[silkroad(size = 0)]
pub enum InventoryItemContentData {
    Equipment {
        plus_level: u8,
        variance: u64,
        durability: u32,
        magic: Vec<InventoryItemMagicData>,
        bindings_1: InventoryItemBindingData,
        bindings_2: InventoryItemBindingData,
        bindings_3: InventoryItemBindingData,
        bindings_4: InventoryItemBindingData,
    },
    Expendable {
        stack_size: u16,
    },
}

impl InventoryItemContentData {
    pub fn equipment(
        plus_level: u8,
        variance: u64,
        durability: u32,
        magic: Vec<InventoryItemMagicData>,
        bindings_1: InventoryItemBindingData,
        bindings_2: InventoryItemBindingData,
        bindings_3: InventoryItemBindingData,
        bindings_4: InventoryItemBindingData,
    ) -> Self {
        InventoryItemContentData::Equipment {
            plus_level,
            variance,
            durability,
            magic,
            bindings_1,
            bindings_2,
            bindings_3,
            bindings_4,
        }
    }

    pub fn expendable(stack_size: u16) -> Self {
        InventoryItemContentData::Expendable { stack_size }
    }
}

#[derive(Copy, Clone, Serialize, ByteSize)]
#[silkroad(size = 2)]
pub enum InventoryOperationError {
    #[silkroad(value = 0x03)]
    InvalidTarget,
    #[silkroad(value = 0x1807)]
    InventoryFull,
    #[silkroad(value = 0x1808)]
    SomethingMallLevelLimit,
    // ??? TODO
    #[silkroad(value = 0x180B)]
    CannotBeStored,
    #[silkroad(value = 0x180E)]
    EquipItemErr,
    // ??? TODO
    #[silkroad(value = 0x180F)]
    NotEnoughGold,
    #[silkroad(value = 0x1810)]
    TooLowLevel,
    #[silkroad(value = 0x1812)]
    LockedByOthers,
    #[silkroad(value = 0x1816)]
    DifferentSex,
    #[silkroad(value = 0x181E)]
    Busy,
    #[silkroad(value = 0x1826)]
    Timeout,
    #[silkroad(value = 0x182B)]
    ExchangeCancelled,
    #[silkroad(value = 0x182F)]
    DifferentRegion,
    #[silkroad(value = 0x1830)]
    MoreStrengthRequired,
    #[silkroad(value = 0x1831)]
    MoreIntellectRequired,
    #[silkroad(value = 0x1838)]
    Indisposable,
    #[silkroad(value = 0x1839)]
    CannotBePicked,
    #[silkroad(value = 0x183A)]
    CannotTrade,
    #[silkroad(value = 0x183e)]
    Unusable,
    #[silkroad(value = 0x184A)]
    InsufficientCoins,
    #[silkroad(value = 0x18DC)]
    RequiresSpecialtyBag,
}

#[derive(Clone, Serialize, ByteSize)]
pub enum InventoryOperationResult {
    #[silkroad(value = 2)]
    Error(InventoryOperationError),
    #[silkroad(value = 1)]
    Success(InventoryOperationResponseData),
}

impl InventoryOperationResult {
    const GOLD_SLOT: u8 = 0xFE;

    pub fn success_gain_gold(amount: u32) -> Self {
        InventoryOperationResult::Success(InventoryOperationResponseData::PickupItem {
            slot: Self::GOLD_SLOT,
            item: ItemPickupData::Gold { amount },
        })
    }

    pub fn success_gain_item(slot: u8, ref_id: u32, content: InventoryItemContentData) -> Self {
        InventoryOperationResult::Success(InventoryOperationResponseData::PickupItem {
            slot,
            item: ItemPickupData::Item {
                rent: RentInfo::Empty,
                ref_id,
                content,
            },
        })
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct JobBagContent {
    pub items: Vec<InventoryItemData>,
}

impl JobBagContent {
    pub fn new(items: Vec<InventoryItemData>) -> Self {
        JobBagContent { items }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct InventoryItemData {
    pub slot: u8,
    pub rent_data: RentInfo,
    pub item_id: u32,
    pub content_data: InventoryItemContentData,
}

impl InventoryItemData {
    pub fn new(slot: u8, rent_data: RentInfo, item_id: u32, content_data: InventoryItemContentData) -> Self {
        InventoryItemData {
            slot,
            rent_data,
            item_id,
            content_data,
        }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct InventoryAvatarItemData;

#[derive(Clone, Serialize, ByteSize)]
pub struct InventoryItemMagicData;

#[derive(Clone, Serialize, ByteSize)]
pub struct InventoryItemBindingData {
    pub kind: u8,
    pub value: u8,
}

impl InventoryItemBindingData {
    pub fn new(kind: u8, value: u8) -> Self {
        InventoryItemBindingData { kind, value }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct CharacterSpawnItemData {
    pub item_id: u32,
    pub upgrade_level: u8,
}

impl CharacterSpawnItemData {
    pub fn new(item_id: u32, upgrade_level: u8) -> Self {
        CharacterSpawnItemData { item_id, upgrade_level }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct ConsignmentItem {
    pub personal_id: u32,
    pub status: u8,
    pub ref_item_id: u32,
    pub sell_count: u32,
    pub price: u64,
    pub deposit: u64,
    pub fee: u64,
    pub end_date: u32,
}

impl ConsignmentItem {
    pub fn new(
        personal_id: u32,
        status: u8,
        ref_item_id: u32,
        sell_count: u32,
        price: u64,
        deposit: u64,
        fee: u64,
        end_date: u32,
    ) -> Self {
        ConsignmentItem {
            personal_id,
            status,
            ref_item_id,
            sell_count,
            price,
            deposit,
            fee,
            end_date,
        }
    }
}

#[derive(Clone, Deserialize, ByteSize)]
pub struct ConsignmentList;

#[derive(Clone, Serialize, ByteSize)]
pub struct ConsignmentResponse {
    pub result: ConsignmentResult,
}

impl ConsignmentResponse {
    pub fn new(result: ConsignmentResult) -> Self {
        ConsignmentResponse { result }
    }

    pub fn success_empty() -> Self {
        ConsignmentResponse {
            result: ConsignmentResult::Success { items: vec![] },
        }
    }
}

#[derive(Clone, Deserialize, ByteSize)]
pub struct InventoryOperation {
    pub data: InventoryOperationRequest,
}

#[derive(Clone, Deserialize, ByteSize)]
pub struct OpenItemMall;

#[derive(Serialize, ByteSize)]
pub struct OpenItemMallResponse(pub OpenItemMallResult);

#[derive(Serialize, ByteSize)]
pub enum OpenItemMallResult {
    #[silkroad(value = 2)]
    Error,
    #[silkroad(value = 1)]
    Success { jid: u32, token: String },
}
