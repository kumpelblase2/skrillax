use crate::{DataEntry, DataMap, FileError, ParseError};
use pk2::Pk2;
use std::str::FromStr;

pub fn load_mastery_map(pk2: &Pk2) -> Result<DataMap<RefMasteryData>, FileError> {
    DataMap::from(pk2, "/server_dep/silkroad/textdata/skillmasterydata.txt")
}

pub struct RefMasteryData {
    ref_id: u16,
    id: String,
    weapons: Vec<u8>,
}

impl DataEntry for RefMasteryData {
    fn ref_id(&self) -> u32 {
        self.ref_id as u32
    }

    fn code(&self) -> &str {
        &self.id
    }
}

impl FromStr for RefMasteryData {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elements = s.split('\t').collect::<Vec<&str>>();
        let weapon1: u8 = elements.get(8).ok_or(ParseError::MissingColumn(8))?.parse()?;
        let weapon2: u8 = elements.get(9).ok_or(ParseError::MissingColumn(9))?.parse()?;
        let weapon3: u8 = elements.get(10).ok_or(ParseError::MissingColumn(10))?.parse()?;
        let weapons = vec![weapon1, weapon2, weapon3]
            .into_iter()
            .filter(|a| *a != 0)
            .collect();
        Ok(Self {
            ref_id: elements.get(0).ok_or(ParseError::MissingColumn(0))?.parse()?,
            id: elements.get(3).ok_or(ParseError::MissingColumn(3))?.to_string(),
            weapons,
        })
    }
}
