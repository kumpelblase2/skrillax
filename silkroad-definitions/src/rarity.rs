use byteorder::ReadBytesExt;
use bytes::{BufMut, BytesMut};
use num_enum_derive::{IntoPrimitive, TryFromPrimitive};
#[cfg(feature = "serde")]
use silkroad_serde::{ByteSize, Deserialize, SerializationError, Serialize};
use std::io::Read;

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum EntityRarityType {
    Normal = 0,
    Champion,
    UnknownCos,
    Unique,
    Giant,
    Titan,
    Elite,
    Strong,
    Unique2,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct EntityRarity {
    party: bool,
    kind: EntityRarityType,
}

impl PartialEq<EntityRarityType> for EntityRarity {
    fn eq(&self, other: &EntityRarityType) -> bool {
        self.kind == *other
    }
}

impl From<EntityRarityType> for EntityRarity {
    fn from(kind: EntityRarityType) -> Self {
        EntityRarity { party: false, kind }
    }
}

impl Default for EntityRarity {
    fn default() -> Self {
        EntityRarity {
            party: false,
            kind: EntityRarityType::Normal,
        }
    }
}

impl TryFrom<u8> for EntityRarity {
    type Error = <EntityRarityType as TryFrom<u8>>::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let is_party = (value & 0x10) != 0;
        let rarity_type_value = !0x10 & value;

        let kind: EntityRarityType = rarity_type_value.try_into()?;

        Ok(EntityRarity { party: is_party, kind })
    }
}

impl From<EntityRarity> for u8 {
    fn from(value: EntityRarity) -> Self {
        (value.party as u8) << 4 | (value.kind as u8)
    }
}

#[cfg(feature = "serde")]
impl ByteSize for EntityRarity {
    fn byte_size(&self) -> usize {
        1
    }
}

#[cfg(feature = "serde")]
impl Serialize for EntityRarity {
    fn write_to(&self, writer: &mut BytesMut) {
        writer.put_u8((*self).into())
    }
}

#[cfg(feature = "serde")]
impl Deserialize for EntityRarity {
    fn read_from<T: Read + ReadBytesExt>(reader: &mut T) -> Result<Self, SerializationError>
    where
        Self: Sized,
    {
        let data = reader.read_u8()?;
        EntityRarity::try_from(data).map_err(|_| SerializationError::UnknownVariation(data as usize, "EntityRarity"))
    }
}
