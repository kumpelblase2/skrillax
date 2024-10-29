use crate::movement::Location;
use skrillax_packet::Packet;
use skrillax_protocol::{define_inbound_protocol, define_outbound_protocol};
use skrillax_serde::*;
use std::fmt::{Display, Formatter};

#[derive(Deserialize, Serialize, ByteSize, Copy, Clone, Debug)]
pub enum ActionTarget {
    #[silkroad(value = 0)]
    None,
    #[silkroad(value = 1)]
    Entity(u32),
    #[silkroad(value = 2)]
    Area(Location),
}

impl Display for ActionTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionTarget::None => write!(f, "None"),
            ActionTarget::Entity(id) => write!(f, "Entity({id})"),
            ActionTarget::Area(loc) => write!(f, "Location({loc})"),
        }
    }
}

impl ActionTarget {
    pub fn self_target() -> Self {
        ActionTarget::Entity(0)
    }
}

#[derive(Deserialize, Serialize, ByteSize, Copy, Clone, Debug)]
pub enum DoActionType {
    #[silkroad(value = 1)]
    Attack { target: ActionTarget },
    #[silkroad(value = 2)]
    PickupItem { target: ActionTarget },
    #[silkroad(value = 4)]
    UseSkill { ref_id: u32, target: ActionTarget },
    #[silkroad(value = 5)]
    CancelBuff { ref_id: u32, target: ActionTarget },
}

#[derive(Deserialize, Copy, Clone, Packet, Debug, Serialize, ByteSize)]
#[packet(opcode = 0x7074)]
pub enum PerformAction {
    #[silkroad(value = 1)]
    Do(DoActionType),
    #[silkroad(value = 2)]
    Stop,
}

#[derive(Serialize, ByteSize, Deserialize, Copy, Clone, Debug)]
pub enum DoActionResponseCode {
    #[silkroad(value = 1)]
    Success,
    #[silkroad(value = 3)]
    Failure(u16),
}

#[derive(Serialize, ByteSize, Deserialize, Copy, Clone, Debug, Packet)]
#[packet(opcode = 0xB074)]
pub enum PerformActionResponse {
    #[silkroad(value = 1)]
    Do(DoActionResponseCode),
    #[silkroad(value = 2)]
    Stop(PerformActionError),
}

#[derive(Serialize, ByteSize, Deserialize, Clone, Debug)]
pub struct DamageContent {
    pub damage_instances: u8,
    #[silkroad(list_type = "length")]
    pub entities: Vec<PerEntityDamage>,
}

#[derive(Serialize, ByteSize, Clone, Deserialize, Debug)]
pub struct PerEntityDamage {
    pub target: u32,
    #[silkroad(list_type = "none")]
    pub damage: Vec<SkillPartDamage>,
}

#[derive(Serialize, ByteSize, Copy, Clone, Deserialize, Debug)]
pub enum DamageKind {
    #[silkroad(value = 1)]
    Standard,
    #[silkroad(value = 2)]
    Critical,
}

#[derive(Serialize, ByteSize, Copy, Clone, Deserialize, Debug)]
pub struct DamageValue {
    pub kind: DamageKind,
    pub amount: u32,
    pub unknown: u16,
    // 0x0
    pub unknown_2: u8, // 0x0
}

impl DamageValue {
    pub fn new(kind: DamageKind, amount: u32) -> Self {
        Self {
            kind,
            amount,
            unknown: 0,
            unknown_2: 0,
        }
    }
}

// Maybe this should be a bitflag instead?
#[derive(Serialize, ByteSize, Copy, Clone, Deserialize, Debug)]
pub enum SkillPartDamage {
    #[silkroad(value = 0)]
    Default(DamageValue),
    #[silkroad(value = 0x80)]
    KillingBlow(DamageValue),
    #[silkroad(value = 0x08)]
    Abort,
}

#[derive(Serialize, ByteSize, Deserialize, Copy, Clone, Debug)]
pub enum PerformActionError {
    #[silkroad(value = 0x00)]
    Completed,
    #[silkroad(value = 0x01)]
    Obstacle,
    #[silkroad(value = 0x03)]
    NotLearned,
    #[silkroad(value = 0x04)]
    InsufficientMP,
    #[silkroad(value = 0x05)]
    Cooldown,
    #[silkroad(value = 0x06)]
    InvalidTarget,
    #[silkroad(value = 0x07)]
    InvalidDistance,
    #[silkroad(value = 0x0C)]
    BuffsIntersect,
    #[silkroad(value = 0x0D)]
    InvalidWeapon,
    #[silkroad(value = 0x0E)]
    InsufficientAmmunition,
    #[silkroad(value = 0x0F)]
    WeaponBroken,
    #[silkroad(value = 0x10)]
    ObstacleInPath,
    #[silkroad(value = 0x11)]
    Untargetable,
    #[silkroad(value = 0x13)]
    InsufficientHP,
}

#[derive(Serialize, ByteSize, Deserialize, Clone, Debug)]
pub enum ActionType {
    #[silkroad(value = 0)]
    None,
    #[silkroad(value = 1)]
    Attack { damage: Option<DamageContent> },
    #[silkroad(value = 8)]
    Teleport,
}

#[derive(Serialize, ByteSize, Deserialize, Clone, Packet, Debug)]
#[packet(opcode = 0xB070)]
pub enum PerformActionUpdate {
    #[silkroad(value = 1)]
    Success {
        unknown: u16, // 0x3002 | 0x3000
        skill_id: u32,
        source: u32,
        instance: u32,
        unknown_4: u32, // (0x27ef2b , 0x47c1f) 261713 0?
        target: u32,
        kind: ActionType,
    },
    #[silkroad(value = 2)]
    Failure(PerformActionError),
}

impl PerformActionUpdate {
    pub fn success(skill_id: u32, source: u32, target: u32, instance: u32, kind: ActionType) -> Self {
        PerformActionUpdate::Success {
            unknown: 0x3002,
            skill_id,
            source,
            instance,
            unknown_4: 0,
            target,
            kind,
        }
    }
}

#[derive(Serialize, ByteSize, Copy, Clone, Packet, Debug)]
#[packet(opcode = 0x3056)]
pub struct ReceiveExperience {
    /// Unique ID of the entity that provided the experience
    pub exp_origin: u32,
    /// The amount of experience points
    pub experience: u64,
    /// the amount of skill experience points
    pub sp: u64,
    // Some kind of flag for reading additional data (either 4 or 8 bytes)
    pub unknown: u8,
    /// If the player reached a new level thanks to this experience and what the new level is
    #[silkroad(size = 0)]
    pub new_level: Option<u16>,
}

define_inbound_protocol! { CombatClientProtocol =>
    PerformAction
}

define_outbound_protocol! { CombatServerProtocol =>
    PerformActionResponse,
    PerformActionUpdate,
    ReceiveExperience
}
