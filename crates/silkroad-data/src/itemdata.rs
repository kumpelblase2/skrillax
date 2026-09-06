use crate::common::RefCommon;
use crate::{DataEntry, DataMap, FileError, ParseError};
use num_enum::TryFromPrimitive;
use pk2_sync::sync::Pk2;
use std::fmt::{Debug, Formatter};
use std::num::{NonZeroU16, NonZeroU8};
use std::str::FromStr;

pub fn load_item_map(pk2: &Pk2<impl std::io::Read + std::io::Seek>) -> Result<DataMap<RefItemData>, FileError> {
    DataMap::from(pk2, "/server_dep/silkroad/textdata/ItemData.txt")
}

#[derive(TryFromPrimitive, Copy, Clone, Eq, PartialEq, Debug)]
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
    pub rarity: RefItemRarity,
    pub can_drop: bool,
    pub max_stack_size: u16,
    pub range: Option<NonZeroU16>,
    pub required_level: Option<NonZeroU8>,
    pub biological_type: RefBiologicalType,
    pub params: [isize; 4],
}

impl Debug for RefItemData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RefItemData")
            .field("ref_id", &self.common.ref_id)
            .field("id", &self.common.id)
            .finish()
    }
}

impl PartialEq for RefItemData {
    fn eq(&self, other: &Self) -> bool {
        self.ref_id() == other.ref_id()
    }
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
        let rarity: u8 = elements.get(15).ok_or(ParseError::MissingColumn(15))?.parse()?;
        let can_drop = elements.get(21).ok_or(ParseError::MissingColumn(21))?.parse::<u8>()?;
        if can_drop > 1 {
            return Err(ParseError::InvalidBoolean(can_drop));
        }

        Ok(Self {
            common,
            price: elements.get(26).ok_or(ParseError::MissingColumn(26))?.parse()?,
            rarity: RefItemRarity::try_from(rarity)?,
            can_drop: can_drop == 1,
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

#[cfg(test)]
mod test {
    use super::*;

    const COLUMN_COUNT: usize = 125;

    fn item_row() -> Vec<String> {
        let mut elements = vec![String::from("0"); COLUMN_COUNT];
        elements[0] = String::from("1"); // service
        elements[1] = String::from("55"); // ref id
        elements[2] = String::from("ITEM_ETC_HP_POTION_01");
        elements[9] = String::from("3"); // TypeID1
        elements[10] = String::from("3"); // TypeID2
        elements[11] = String::from("1"); // TypeID3
        elements[12] = String::from("1"); // TypeID4
        elements[13] = String::from("300000"); // despawn time
        elements[14] = String::from("0"); // country
        elements[26] = String::from("150"); // price
        elements[33] = String::from("5"); // required level
        elements[57] = String::from("50"); // max stack size
        elements[58] = String::from("2"); // biological type
        elements[94] = String::from("0"); // range
        for column in [118, 120, 122, 124] {
            elements[column] = String::from("0");
        }
        elements
    }

    fn parse_row(elements: Vec<String>) -> Result<RefItemData, ParseError> {
        let joined = elements.join("\t");
        RefItemData::from_str(&joined)
    }

    #[test]
    fn parse_item_metadata() {
        let mut row = item_row();
        row[15] = String::from("2");
        row[21] = String::from("1");

        let item = parse_row(row).unwrap();
        assert!(item.common.service);
        assert_eq!(item.rarity, RefItemRarity::Seal);
        assert!(item.can_drop);
        assert_eq!(item.max_stack_size, 50);
    }

    #[test]
    fn inactive_item_without_drop_support() {
        let mut row = item_row();
        row[0] = String::from("0");
        row[15] = String::from("0");
        row[21] = String::from("0");

        let item = parse_row(row).unwrap();
        assert!(!item.common.service);
        assert_eq!(item.rarity, RefItemRarity::General);
        assert!(!item.can_drop);
    }

    #[test]
    fn reject_invalid_rarity_and_flags() {
        let mut row = item_row();
        row[15] = String::from("4");
        assert!(parse_row(row.clone()).is_err());

        row[15] = String::from("0");
        row[21] = String::from("2");
        assert!(parse_row(row).is_err());
    }
}
