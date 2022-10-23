use crate::common::RefCommon;
use crate::{DataEntry, DataMap, FileError, ParseError};
use num_enum::TryFromPrimitive;
use pk2::Pk2;
use std::num::{NonZeroU16, NonZeroU8};
use std::str::FromStr;

pub fn load_item_map(pk2: &Pk2) -> Result<DataMap<RefItemData>, FileError> {
    DataMap::from(pk2, "/server_dep/silkroad/textdata/ItemData.txt")
}

#[derive(TryFromPrimitive)]
#[repr(u8)]
pub enum RefItemRarity {
    General = 0,
    Blue = 1,
    Seal = 2,
    Set = 3,
    Roc = 6,
    Legend = 8,
}

#[derive(TryFromPrimitive, Copy, Clone)]
#[repr(u8)]
pub enum RefBiologicalType {
    Female = 0,
    Male = 1,
    Both = 2,
    Pet1 = 3,
    Pet2 = 4,
    Pet3 = 5,
}

impl FromStr for RefBiologicalType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: u8 = s.parse()?;
        Ok(RefBiologicalType::try_from(value)?)
    }
}

#[derive(Clone)]
pub struct RefItemData {
    pub common: RefCommon,
    pub price: u64,
    pub max_stack_size: u16,
    pub range: Option<NonZeroU16>,
    pub required_level: Option<NonZeroU8>,
    pub biological_type: RefBiologicalType,
    pub params: [isize; 4],
}

impl DataEntry for RefItemData {
    fn ref_id(&self) -> u32 {
        self.common.ref_id
    }

    fn code(&self) -> &str {
        &self.common.id
    }
}

impl FromStr for RefItemData {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elements = s.split('\t').collect::<Vec<&str>>();
        let common = RefCommon::from_columns(&elements)?;
        let range: u16 = elements.get(94).ok_or(ParseError::MissingColumn(94))?.parse()?;
        let required_level: u8 = elements.get(33).ok_or(ParseError::MissingColumn(33))?.parse()?;
        Ok(Self {
            common,
            price: elements.get(26).ok_or(ParseError::MissingColumn(26))?.parse()?,
            params: [
                elements.get(118).ok_or(ParseError::MissingColumn(118))?.parse()?,
                elements.get(120).ok_or(ParseError::MissingColumn(120))?.parse()?,
                elements.get(122).ok_or(ParseError::MissingColumn(122))?.parse()?,
                elements.get(124).ok_or(ParseError::MissingColumn(124))?.parse()?,
            ],
            range: NonZeroU16::new(range),
            required_level: NonZeroU8::new(required_level),
            biological_type: elements.get(58).ok_or(ParseError::MissingColumn(58))?.parse()?,
            max_stack_size: elements.get(57).ok_or(ParseError::MissingColumn(57))?.parse()?,
        })
    }
}
