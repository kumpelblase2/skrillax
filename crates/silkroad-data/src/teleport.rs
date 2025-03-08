use crate::common::RefCommon;
use crate::{parse_file, DataEntry, DataMap, FileError, ParseError};
use pk2::Pk2;
use silkroad_definitions::Region;
use std::collections::HashMap;
use std::num::NonZeroU16;
use std::str::FromStr;

pub fn load_teleport_map(pk2: &Pk2) -> Result<HashMap<u16, TeleportLocation>, FileError> {
    let mut file = pk2.open_file("/server_dep/silkroad/textdata/TeleportData.txt")?;
    let teleports: Vec<TeleportLocation> = parse_file(&mut file)?;
    let map: HashMap<_, _> = teleports.into_iter().map(|gold| (gold.ref_id, gold)).collect();
    Ok(map)
}

pub fn load_teleport_links(pk2: &Pk2) -> Result<Vec<TeleportLink>, FileError> {
    let mut file = pk2.open_file("/server_dep/silkroad/textdata/TeleportLink.txt")?;
    let teleports: Vec<TeleportLink> = parse_file(&mut file)?;
    Ok(teleports)
}

pub fn load_teleport_buildings(pk2: &Pk2) -> Result<DataMap<TeleportBuilding>, FileError> {
    let mut file = pk2.open_file("/server_dep/silkroad/textdata/TeleportBuilding.txt")?;
    let buildings: Vec<TeleportBuilding> = parse_file(&mut file)?;
    Ok(DataMap::new(buildings))
}

pub struct TeleportLocation {
    pub active: bool,
    pub ref_id: u16,
    pub code: String,
    pub npc_id: Option<NonZeroU16>,
    pub spawn_region: Region,
    pub spawn_x: i16,
    pub spawn_y: i16,
    pub spawn_z: i16,
    pub spawn_angle: u16,
    pub respawn_point: bool,
}

impl FromStr for TeleportLocation {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elements = s.split('\t').collect::<Vec<&str>>();

        let region: i16 = elements.get(5).ok_or(ParseError::MissingColumn(5))?.parse()?;
        let npc_id: u16 = elements.get(3).ok_or(ParseError::MissingColumn(3))?.parse()?;
        let active: u8 = elements.get(0).ok_or(ParseError::MissingColumn(0))?.parse()?;
        let respawn_point: u8 = elements.get(10).ok_or(ParseError::MissingColumn(10))?.parse()?;
        Ok(Self {
            active: active == 1,
            ref_id: elements.get(1).ok_or(ParseError::MissingColumn(1))?.parse()?,
            code: elements.get(2).ok_or(ParseError::MissingColumn(2))?.to_string(),
            npc_id: NonZeroU16::new(npc_id),
            spawn_region: (region as u16).into(),
            spawn_x: elements.get(6).ok_or(ParseError::MissingColumn(6))?.parse()?,
            spawn_y: elements.get(7).ok_or(ParseError::MissingColumn(7))?.parse()?,
            spawn_z: elements.get(8).ok_or(ParseError::MissingColumn(8))?.parse()?,
            spawn_angle: elements.get(9).ok_or(ParseError::MissingColumn(9))?.parse()?,
            respawn_point: respawn_point == 1,
        })
    }
}

pub struct TeleportLink {
    pub active: bool,
    pub source_id: u16,
    pub target_id: u16,
    pub cost: u64,
    pub minimum_level: u8,
}

impl FromStr for TeleportLink {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elements = s.split('\t').collect::<Vec<&str>>();

        let active: u8 = elements.get(0).ok_or(ParseError::MissingColumn(0))?.parse()?;
        Ok(Self {
            active: active == 1,
            source_id: elements.get(1).ok_or(ParseError::MissingColumn(1))?.parse()?,
            target_id: elements.get(2).ok_or(ParseError::MissingColumn(2))?.parse()?,
            cost: elements.get(3).ok_or(ParseError::MissingColumn(3))?.parse()?,
            minimum_level: elements.get(21).ok_or(ParseError::MissingColumn(21))?.parse()?,
        })
    }
}

pub struct TeleportBuilding {
    pub common: RefCommon,
    pub region: Region,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl FromStr for TeleportBuilding {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elements = s.split('\t').collect::<Vec<&str>>();

        let region: i16 = elements.get(41).ok_or(ParseError::MissingColumn(41))?.parse()?;
        Ok(Self {
            common: RefCommon::from_columns(&elements)?,
            region: (region as u16).into(),
            x: elements.get(43).ok_or(ParseError::MissingColumn(43))?.parse()?,
            y: elements.get(44).ok_or(ParseError::MissingColumn(44))?.parse()?,
            z: elements.get(45).ok_or(ParseError::MissingColumn(45))?.parse()?,
        })
    }
}

impl DataEntry for TeleportBuilding {
    fn ref_id(&self) -> u32 {
        self.common.ref_id
    }

    fn code(&self) -> &str {
        &self.common.id
    }
}
