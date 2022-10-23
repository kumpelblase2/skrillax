use crate::ParseError;
use crate::TypeId;
use num_enum_derive::TryFromPrimitive;
use std::str::FromStr;
use std::time::Duration;

#[derive(Clone)]
pub struct RefCommon {
    pub ref_id: u32,
    // column 1
    pub id: String,
    // column 2
    pub type_id: TypeId,
    // column 9-12
    pub country: RefOrigin,
    // column 14
    pub despawn_time: Duration, // column 13
}

impl RefCommon {
    pub fn from_columns(elements: &[&str]) -> Result<Self, ParseError> {
        Ok(Self {
            ref_id: elements.get(1).ok_or(ParseError::MissingColumn(1))?.parse()?,
            id: elements.get(2).ok_or(ParseError::MissingColumn(2))?.to_string(),
            type_id: TypeId(
                elements.get(9).ok_or(ParseError::MissingColumn(9))?.parse()?,
                elements.get(10).ok_or(ParseError::MissingColumn(10))?.parse()?,
                elements.get(11).ok_or(ParseError::MissingColumn(11))?.parse()?,
                elements.get(12).ok_or(ParseError::MissingColumn(12))?.parse()?,
            ),
            despawn_time: Duration::from_millis(elements.get(13).ok_or(ParseError::MissingColumn(13))?.parse()?),
            country: elements.get(14).ok_or(ParseError::MissingColumn(14))?.parse()?,
        })
    }
}

#[derive(TryFromPrimitive, Copy, Clone)]
#[repr(u8)]
pub enum RefOrigin {
    Chinese = 0,
    European = 1,
    General = 3,
}

impl FromStr for RefOrigin {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: u8 = s.parse()?;
        Ok(RefOrigin::try_from(value)?)
    }
}
