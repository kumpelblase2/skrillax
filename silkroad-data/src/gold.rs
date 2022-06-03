use crate::ParseError;
use std::str::FromStr;

pub struct RefGold {
    pub level: u8,
    pub min: u32,
    pub max: u32,
}

impl FromStr for RefGold {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elements = s.split('\t').collect::<Vec<&str>>();
        Ok(Self {
            level: elements.get(0).ok_or(ParseError::MissingColumn(0))?.parse()?,
            min: elements.get(1).ok_or(ParseError::MissingColumn(1))?.parse()?,
            max: elements.get(2).ok_or(ParseError::MissingColumn(2))?.parse()?,
        })
    }
}
