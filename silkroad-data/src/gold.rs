use crate::{parse_file, FileError, ParseError};
use pk2::Pk2;
use std::ops::RangeInclusive;
use std::str::FromStr;

pub struct GoldMap {
    gold_levels: Vec<RefGold>,
}

impl GoldMap {
    pub fn from(pk2: &Pk2) -> Result<GoldMap, FileError> {
        let mut file = pk2.open_file("/server_dep/silkroad/textdata/levelgold.txt")?;
        let gold_lines = parse_file(&mut file)?;
        Ok(GoldMap::new(gold_lines))
    }

    pub fn new(gold_levels: Vec<RefGold>) -> Self {
        Self { gold_levels }
    }

    pub fn get_for_level(&self, level: u8) -> RangeInclusive<u32> {
        if level == 0 {
            return 0..=0;
        }

        self.gold_levels
            .iter()
            .find(|gold| gold.level == level)
            .map(|level| level.min..=level.max)
            .unwrap_or_else(|| {
                let last = self
                    .gold_levels
                    .last()
                    .expect("We should always have at least one entry.");
                last.min..=last.max
            })
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
