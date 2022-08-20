use crate::Location;
use silkroad_serde::*;
use silkroad_serde_derive::*;

#[derive(Deserialize)]
pub enum ActionTarget {
    #[silkroad(value = 0)]
    None,
    #[silkroad(value = 1)]
    Entity(u32),
    #[silkroad(value = 2)]
    Area(Location),
}

impl ActionTarget {
    pub fn self_target() -> Self {
        ActionTarget::Entity(0)
    }
}

#[derive(Deserialize)]
pub enum DoActionType {
    #[silkroad(value = 1)]
    Attack { target: ActionTarget },
    #[silkroad(value = 4)]
    UseSkill { ref_id: u32, target: ActionTarget },
    #[silkroad(value = 5)]
    CancelBuff { ref_id: u32, target: ActionTarget },
}

#[derive(Deserialize)]
pub enum PerformAction {
    #[silkroad(value = 1)]
    Do(DoActionType),
    #[silkroad(value = 2)]
    Stop,
}

#[derive(Serialize, ByteSize)]
pub enum PerformActionResponse {
    #[silkroad(value = 1)]
    Do(u8),
    #[silkroad(value = 2)]
    Stop(u8),
}

#[derive(Serialize, ByteSize)]
pub struct DamageContent {
    #[silkroad(list_type = "length")]
    pub instances: Vec<DamageInstance>,
}

#[derive(Serialize, ByteSize)]
pub struct DamageInstance {
    pub damage_count: u8,
    pub unknown: u8,
    // 0x1
    pub target: u32,
    #[silkroad(list_type = "none")]
    pub damage: Vec<SkillPartDamage>,
}

#[derive(Serialize, ByteSize)]
pub enum DamageKind {
    #[silkroad(value = 1)]
    Standard,
    #[silkroad(value = 2)]
    Critical,
}

#[derive(Serialize, ByteSize)]
pub enum SkillPartDamage {
    #[silkroad(value = 0)]
    Default {
        kind: DamageKind,
        amount: u32,
        unknown_2: u16,
        // 0x0
        unknown_3: u8, // 0x0
    },
    #[silkroad(value = 0x80)]
    KillingBlow {
        kind: DamageKind,
        amount: u32,
        unknown_2: u16,
        // 0x0
        unknown_3: u8, // 0x0
    },
    #[silkroad(value = 0x08)]
    Empty,
}

#[derive(Serialize, ByteSize)]
pub enum PerformActionError {
    #[silkroad(value = 0x06)]
    InvalidTarget,
    #[silkroad(value = 0x05)]
    Cooldown,
    #[silkroad(value = 0x1)]
    Obstacle,
    #[silkroad(value = 0x0E)]
    InsufficientAmmunition,
    #[silkroad(value = 0x0C)]
    BuffsIntersect,
}

#[derive(Serialize, ByteSize)]
pub enum ActionType {
    #[silkroad(value = 0)]
    None,
    #[silkroad(value = 1)]
    Attack,
    #[silkroad(value = 8)]
    Teleport,
}

#[derive(Serialize, ByteSize)]
pub enum PerformActionUpdate {
    #[silkroad(value = 1)]
    Success {
        unknown: u8,
        // 0x2
        unknown_2: u8,
        // 0x30,
        skill_id: u32,
        source: u32,
        unknown_3: u32,
        // 0x76034
        unknown_4: u32,
        // (0x27ef2b , 0x47c1f) 261713
        target: u32,
        kind: ActionType,
        unknown_5: u8,
        // ??
        #[silkroad(size = 0)]
        damage: Option<DamageContent>,
    },
    #[silkroad(value = 2)]
    Error(PerformActionError),
}

impl PerformActionUpdate {
    pub fn success(skill_id: u32, source: u32, target: u32, kind: ActionType, damage: Option<DamageContent>) -> Self {
        PerformActionUpdate::Success {
            unknown: 0x2,
            unknown_2: 0x30,
            skill_id,
            source,
            unknown_3: 0x076034,
            unknown_4: 0x27ef2b,
            target,
            kind,
            unknown_5: 0,
            damage,
        }
    }
}
