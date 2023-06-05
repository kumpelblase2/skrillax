use crate::ParseError;
use num_enum_derive::TryFromPrimitive;
use std::str::FromStr;

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, TryFromPrimitive)]
pub enum EntityRarity {
    Normal = 0,
    Champion,
    Unique = 3,
    Giant,
    Titan,
    Elite,
    Elite2,
    Unique2,
}

impl FromStr for EntityRarity {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let number: u8 = s.parse()?;
        Ok(EntityRarity::try_from(number)?)
    }
}
