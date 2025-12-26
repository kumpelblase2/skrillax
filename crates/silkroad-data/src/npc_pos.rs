use crate::{parse_file, FileError, ParseError};
use pk2_sync::sync::Pk2;
use std::str::FromStr;

pub struct NpcPosition {
    pub npc_id: u32,
    pub region: u16,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl NpcPosition {
    pub fn from(pk2: &Pk2<impl std::io::Read + std::io::Seek>) -> Result<Vec<NpcPosition>, FileError> {
        let mut file = pk2.open_file("/server_dep/silkroad/textdata/NpcPos.txt")?;
        parse_file(&mut file)
    }
}

impl FromStr for NpcPosition {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elements = s.split('\t').collect::<Vec<&str>>();
        let region: i16 = elements.get(1).ok_or(ParseError::MissingColumn(1))?.parse()?;
        Ok(Self {
            npc_id: elements.get(0).ok_or(ParseError::MissingColumn(0))?.parse()?,
            region: region as u16,
            x: elements.get(2).ok_or(ParseError::MissingColumn(2))?.parse()?,
            y: elements.get(3).ok_or(ParseError::MissingColumn(3))?.parse()?,
            z: elements.get(4).ok_or(ParseError::MissingColumn(4))?.parse()?,
        })
    }
}
