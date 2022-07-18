use crate::ParseError;
use pk2::Pk2;
use std::io::Read;
use std::str::FromStr;

pub struct LevelMap {
    levels: Vec<RefLevel>,
}

impl LevelMap {
    pub fn from(pk2: &Pk2) -> LevelMap {
        let mut file = pk2.open_file("/server_dep/silkroad/textdata/LevelData.txt").unwrap();
        let mut full_string = String::new();
        file.read_to_string(&mut full_string).unwrap();
        let levels = full_string
            .lines()
            .filter(|line| line.len() > 0)
            .map(|line| RefLevel::from_str(line).unwrap())
            .collect();
        LevelMap { levels }
    }

    pub fn get_exp_for_level(&self, level: u8) -> Option<u64> {
        let index = (level - 1) as usize;
        self.levels.get(index).map(|level| level.exp)
    }
}

pub struct RefLevel {
    pub level: u8,
    pub exp: u64,
    pub exp_mastery: u32,
    pub mob_exp: u64,
    // Is this correct?
    pub job_exp_trader: i64,
    pub job_exp_thief: i64,
    pub job_exp_hunter: i64,
    pub pet_exp: u64,
    pub pet_stored_sp: u32,
}

impl FromStr for RefLevel {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elements = s.split('\t').collect::<Vec<&str>>();
        Ok(Self {
            level: elements.get(0).ok_or(ParseError::MissingColumn(0))?.parse()?,
            exp: elements.get(1).ok_or(ParseError::MissingColumn(1))?.parse()?,
            exp_mastery: elements.get(2).ok_or(ParseError::MissingColumn(2))?.parse()?,
            mob_exp: elements.get(5).ok_or(ParseError::MissingColumn(3))?.parse()?,
            job_exp_trader: elements.get(6).ok_or(ParseError::MissingColumn(4))?.parse()?,
            job_exp_thief: elements.get(7).ok_or(ParseError::MissingColumn(5))?.parse()?,
            job_exp_hunter: elements.get(8).ok_or(ParseError::MissingColumn(6))?.parse()?,
            pet_exp: elements.get(9).ok_or(ParseError::MissingColumn(7))?.parse()?,
            pet_stored_sp: elements.get(10).ok_or(ParseError::MissingColumn(8))?.parse()?,
        })
    }
}
