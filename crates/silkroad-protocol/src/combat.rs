use crate::combat::SkillCastType::Attack;
use crate::movement::Location;
use bytes::BytesMut;
use skrillax_packet::Packet;
use skrillax_serde::__internal::byteorder::{LittleEndian, ReadBytesExt};
use skrillax_serde::*;
use skrillax_stream::registry::PacketRegistryBuilder;
use std::fmt::{Display, Formatter};
use std::io::Read;

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

#[derive(Serialize, ByteSize, Clone, Debug)]
pub struct DamageContent {
    pub damage_instances: u8,
    pub entities: Vec<PerEntityDamage>,
}

#[derive(Copy, Clone)]
struct DamageInstances(u8);

impl Deserialize for DamageContent {
    fn read_from<T: Read + ReadBytesExt>(reader: &mut T, ctx: &SerdeContext) -> Result<Self, SerializationError>
    where
        Self: Sized,
    {
        let damage_instances = reader.read_u8()?;
        ctx.set(DamageInstances(damage_instances));
        let length = reader.read_u8()?;
        let mut entities = Vec::with_capacity(length.into());
        for _i in 0..length {
            let per_entity = PerEntityDamage::read_from(reader, ctx)?;
            entities.push(per_entity);
        }
        ctx.unset::<DamageInstances>();

        Ok(DamageContent {
            damage_instances,
            entities,
        })
    }
}

#[derive(Clone, Debug)]
pub struct PerEntityDamage {
    pub target: u32,
    pub damage: Vec<SkillPartDamage>,
}

impl ByteSize for PerEntityDamage {
    fn byte_size(&self) -> usize {
        4 + self.damage.iter().map(|d| d.byte_size()).sum::<usize>()
    }
}

impl Serialize for PerEntityDamage {
    fn write_to(&self, writer: &mut BytesMut, ctx: &SerdeContext) -> Result<(), SerializationError> {
        self.target.write_to(writer, ctx)?;
        for damage in &self.damage {
            damage.write_to(writer, ctx)?;
        }
        Ok(())
    }
}

impl Deserialize for PerEntityDamage {
    fn read_from<T: Read + ReadBytesExt>(reader: &mut T, ctx: &SerdeContext) -> Result<Self, SerializationError>
    where
        Self: Sized,
    {
        let target = reader.read_u32::<LittleEndian>()?;
        let size = ctx.get::<DamageInstances>().map(|instances| instances.0).unwrap_or(0);
        let mut damage = Vec::with_capacity(size.into());
        for _i in 0..size {
            let per_entity = SkillPartDamage::read_from(reader, ctx)?;
            damage.push(per_entity);
        }

        Ok(PerEntityDamage { target, damage })
    }
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
    #[silkroad(value = 1)]
    Damage(DamageValue),
    #[silkroad(value = 0x02)]
    Block,
    #[silkroad(value = 0x04)]
    KnockDown(DamageValue),
    #[silkroad(value = 0x05)]
    Knockback(DamageValue),
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
    Attack { damage: DamageContent },
    #[silkroad(value = 8)]
    Teleport,
}

#[derive(Serialize, ByteSize, Deserialize, Copy, Clone, Debug)]
pub enum SkillCastType {
    #[silkroad(value = 0)]
    Buff,
    #[silkroad(value = 2)]
    Attack,
}

#[derive(Serialize, ByteSize, Deserialize, Clone, Packet, Debug)]
#[packet(opcode = 0xB070)]
pub enum PerformActionUpdate {
    #[silkroad(value = 1)]
    Success {
        cast_type: SkillCastType,
        unknown: u8, // 0x30
        skill_id: u32,
        source: u32,
        instance: u32,
        unknown_4: u32, // (0x27ef2b , 0x47c1f) 261713 0?
        target: u32,
        unknown_5: u8,
        kind: ActionType,
    },
    #[silkroad(value = 2)]
    Failure(PerformActionError),
}

impl PerformActionUpdate {
    pub fn success(skill_id: u32, source: u32, target: u32, instance: u32, kind: ActionType) -> Self {
        PerformActionUpdate::Success {
            cast_type: Attack,
            unknown: 0x30,
            skill_id,
            source,
            instance,
            unknown_4: 0,
            target,
            unknown_5: 1,
            kind,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct DidLevelUp;

#[derive(Deserialize, Serialize, ByteSize, Copy, Clone, Packet, Debug)]
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
    #[silkroad(when = "ctx.get::<DidLevelUp>().is_some()")]
    pub new_level: Option<u16>,
}

pub trait CombatPacketRegistryExt {
    fn register_combat_packets(self) -> Self;
}

impl CombatPacketRegistryExt for PacketRegistryBuilder {
    fn register_combat_packets(self) -> Self {
        self.register::<PerformAction>()
            .register::<PerformActionResponse>()
            .register::<PerformActionUpdate>()
            .register::<ReceiveExperience>()
    }
}
