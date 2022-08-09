use crate::ParseError;
use std::str::FromStr;

pub struct RefMasteryData {
    ref_id: u16,
    id: String,
    weapons: Vec<u8>,
}

impl FromStr for RefMasteryData {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elements = s.split('\t').collect::<Vec<&str>>();
        let weapon1: u8 = elements.get(9).ok_or(ParseError::MissingColumn(9))?.parse()?;
        let weapon2: u8 = elements.get(10).ok_or(ParseError::MissingColumn(10))?.parse()?;
        let weapon3: u8 = elements.get(11).ok_or(ParseError::MissingColumn(11))?.parse()?;
        let weapons = vec![weapon1, weapon2, weapon3]
            .into_iter()
            .filter(|a| *a != 0)
            .collect();
        Ok(Self {
            ref_id: elements.get(0).ok_or(ParseError::MissingColumn(0))?.parse()?,
            id: elements.get(4).ok_or(ParseError::MissingColumn(4))?.to_string(),
            weapons,
        })
    }
}
