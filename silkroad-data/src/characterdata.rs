use crate::type_id::TypeId;
use crate::{DataEntry, DataMap, FileError, ParseError};
use pk2::Pk2;
use std::str::FromStr;

pub fn load_character_map(pk2: &Pk2) -> Result<DataMap<RefCharacterData>, FileError> {
    DataMap::from(pk2, "/server_dep/silkroad/textdata/CharacterData.txt")
}

#[derive(Clone)]
pub struct RefCharacterData {
    pub ref_id: u32,
    // column 1
    pub id: String,
    // column 2
    pub type_id: TypeId,
    // column 9-12
    pub level: u8,
    // column 57
    pub exp: u32,
    // column 79
    pub hp: u32,
    // column 59
    pub base_range: u16,
    // column 50
    pub walk_speed: u32,
    // column 46
    pub run_speed: u32,
    // column 47
    pub berserk_speed: u32,
    // column 48
    pub aggressive: bool,
    // column 93,
    pub skills: Vec<u32>, // column 83-92
}

impl DataEntry for RefCharacterData {
    fn ref_id(&self) -> u32 {
        self.ref_id
    }

    fn code(&self) -> &str {
        &self.id
    }
}

impl FromStr for RefCharacterData {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elements = s.split('\t').collect::<Vec<&str>>();
        let aggressive: u8 = elements.get(93).ok_or(ParseError::MissingColumn(94))?.parse()?;
        let mut skills: Vec<u32> = Vec::new();
        for i in 83..=92 {
            let skill_id: u32 = elements.get(i).ok_or(ParseError::MissingColumn(i as u8))?.parse()?;
            if skill_id != 0 {
                skills.push(skill_id);
            }
        }
        Ok(Self {
            ref_id: elements.get(1).ok_or(ParseError::MissingColumn(1))?.parse()?,
            id: elements.get(2).ok_or(ParseError::MissingColumn(2))?.to_string(),
            type_id: TypeId(
                elements.get(9).ok_or(ParseError::MissingColumn(9))?.parse()?,
                elements.get(10).ok_or(ParseError::MissingColumn(10))?.parse()?,
                elements.get(11).ok_or(ParseError::MissingColumn(11))?.parse()?,
                elements.get(12).ok_or(ParseError::MissingColumn(12))?.parse()?,
            ),
            level: elements.get(57).ok_or(ParseError::MissingColumn(57))?.parse()?,
            exp: elements.get(79).ok_or(ParseError::MissingColumn(79))?.parse()?,
            hp: elements.get(59).ok_or(ParseError::MissingColumn(59))?.parse()?,
            walk_speed: elements.get(46).ok_or(ParseError::MissingColumn(46))?.parse()?,
            run_speed: elements.get(47).ok_or(ParseError::MissingColumn(47))?.parse()?,
            berserk_speed: elements.get(48).ok_or(ParseError::MissingColumn(48))?.parse()?,
            base_range: elements.get(50).ok_or(ParseError::MissingColumn(50))?.parse()?,
            aggressive: aggressive == 1,
            skills,
        })
    }
}
