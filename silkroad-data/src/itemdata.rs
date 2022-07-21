use crate::type_id::TypeId;
use crate::{list_files, parse_file, FileError, ParseError};
use num_enum::TryFromPrimitive;
use pk2::Pk2;
use std::str::FromStr;
use std::time::Duration;

pub struct ItemMap {
    item_data: Vec<RefItemData>,
}

impl ItemMap {
    pub fn from(pk2: &Pk2) -> Result<ItemMap, FileError> {
        let mut file = pk2.open_file("/server_dep/silkroad/textdata/ItemData.txt")?;
        let item_lines = list_files(&mut file)?;
        let all_items: Vec<RefItemData> = item_lines
            .into_iter()
            .filter(|name| name.len() > 0)
            .map(|filename| format!("/server_dep/silkroad/textdata/{}", filename))
            .map(|filename| {
                let mut file = pk2.open_file(&filename).unwrap();
                parse_file(&mut file).unwrap()
            })
            .flatten()
            .collect();

        Ok(ItemMap::new(all_items))
    }

    pub fn new(item_data: Vec<RefItemData>) -> Self {
        Self { item_data }
    }
}

#[derive(TryFromPrimitive, Copy, Clone)]
#[repr(u8)]
pub enum RefItemCountry {
    Chinese = 0,
    European = 1,
    General = 3,
}

impl FromStr for RefItemCountry {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: u8 = s.parse()?;
        Ok(RefItemCountry::try_from(value)?)
    }
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

#[derive(Clone)]
pub struct RefItemData {
    ref_id: u32,
    id: String,
    type_id: TypeId,
    despawn_time: Duration,
    country: RefItemCountry,
    price: u64,
    params: [isize; 4],
}

impl FromStr for RefItemData {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elements = s.split('\t').collect::<Vec<&str>>();
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
            price: elements.get(26).ok_or(ParseError::MissingColumn(26))?.parse()?,
            params: [
                elements.get(118).ok_or(ParseError::MissingColumn(118))?.parse()?,
                elements.get(120).ok_or(ParseError::MissingColumn(120))?.parse()?,
                elements.get(122).ok_or(ParseError::MissingColumn(122))?.parse()?,
                elements.get(124).ok_or(ParseError::MissingColumn(124))?.parse()?,
            ],
        })
    }
}
