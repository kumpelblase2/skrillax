use crate::{parse_file, FileError, ParseError};
use pk2_sync::sync::Pk2;
use std::collections::HashMap;
use std::ops::{Deref, RangeInclusive};
use std::str::FromStr;

pub fn load_gold_map(pk2: &Pk2<impl std::io::Read + std::io::Seek>) -> Result<GoldMap, FileError> {
    let mut file = pk2.open_file("/server_dep/silkroad/textdata/levelgold.txt")?;
    let gold_lines: Vec<RefGold> = parse_file(&mut file)?;
    let map: HashMap<_, _> = gold_lines.into_iter().map(|gold| (gold.level, gold)).collect();
    Ok(GoldMap(map))
}

pub struct GoldMap(HashMap<u8, RefGold>);

impl Deref for GoldMap {
    type Target = HashMap<u8, RefGold>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl GoldMap {
    pub fn new(map: HashMap<u8, RefGold>) -> Self {
        Self(map)
    }

    pub fn range_for_level(&self, level: u8) -> Option<RangeInclusive<u32>> {
        if level == 0 {
            return Some(0..=0);
        }

        self.get(&level).map(|gold| gold.min..=gold.max)
    }

    pub fn get_for_level(&self, level: u8) -> RangeInclusive<u32> {
        self.range_for_level(level).unwrap_or_else(|| 0..=0)
    }
}

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
