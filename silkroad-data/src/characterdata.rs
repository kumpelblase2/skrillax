use crate::type_id::TypeId;
use crate::{list_files, parse_file, FileError, ParseError};
use pk2::Pk2;
use std::str::FromStr;

pub struct CharacterMap {
    character_data: Vec<RefCharacterData>,
}

impl CharacterMap {
    pub fn from(pk2: &Pk2) -> Result<CharacterMap, FileError> {
        let mut file = pk2.open_file("/server_dep/silkroad/textdata/CharacterData.txt")?;
        let character_lines = list_files(&mut file)?;
        let all_characters: Vec<RefCharacterData> = character_lines
            .into_iter()
            .filter(|name| name.len() > 0)
            .map(|filename| format!("/server_dep/silkroad/textdata/{}", filename))
            .map(|filename| {
                let mut file = pk2.open_file(&filename).unwrap();
                parse_file(&mut file).unwrap()
            })
            .flatten()
            .collect();

        Ok(CharacterMap::new(all_characters))
    }

    pub fn new(character_data: Vec<RefCharacterData>) -> Self {
        Self { character_data }
    }

    pub fn find_id(&self, id: u32) -> Option<&RefCharacterData> {
        self.character_data.iter().find(|data| data.ref_id == id)
    }
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
    pub walk_speed: u32,
    // column 46
    pub run_speed: u32,
    // column 47
    pub berserk_speed: u32,
    // column 48
    pub aggressive: bool, // column 93
}

impl FromStr for RefCharacterData {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elements = s.split('\t').collect::<Vec<&str>>();
        let aggressive: u8 = elements.get(93).ok_or(ParseError::MissingColumn(94))?.parse()?;
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
            aggressive: aggressive == 1,
        })
    }
}
