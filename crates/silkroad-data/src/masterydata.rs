use crate::{parse_file, DataEntry, DataMap, FileError, ParseError};
use pk2_sync::sync::Pk2;
use std::num::NonZeroU8;
use std::str::FromStr;

pub fn load_mastery_map(pk2: &Pk2<impl std::io::Read + std::io::Seek>) -> Result<DataMap<RefMasteryData>, FileError> {
    let mut file = pk2.open_file("/server_dep/silkroad/textdata/skillmasterydata.txt")?;
    let levels: Vec<RefMasteryData> = parse_file(&mut file)?;
    let map = levels
        .into_iter()
        .filter(|mastery| mastery.secondary.is_none() || mastery.secondary.unwrap().get() == 1)
        .collect();
    Ok(DataMap::new(map))
}

pub struct RefMasteryData {
    pub ref_id: u16,
    pub secondary: Option<NonZeroU8>,
    pub id: String,
    pub weapons: Vec<u8>,
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
        let secondary: i8 = elements.get(1).ok_or(ParseError::MissingColumn(1))?.parse()?;
        let weapon1: u8 = elements.get(8).ok_or(ParseError::MissingColumn(8))?.parse()?;
        let weapon2: u8 = elements.get(9).ok_or(ParseError::MissingColumn(9))?.parse()?;
        let weapon3: u8 = elements.get(10).ok_or(ParseError::MissingColumn(10))?.parse()?;
        let weapons = vec![weapon1, weapon2, weapon3]
            .into_iter()
            .filter(|a| *a != 0)
            .collect();
        Ok(Self {
            ref_id: elements.get(0).ok_or(ParseError::MissingColumn(0))?.parse()?,
            secondary: if secondary > 0 {
                NonZeroU8::new(secondary as u8)
            } else {
                None
            },
            id: elements.get(3).ok_or(ParseError::MissingColumn(3))?.to_string(),
            weapons,
        })
    }
}
