use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) const SUPPORTED_VERSION: u32 = 1;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StarterGearFileV1 {
    pub(crate) version: u32,
    #[serde(default)]
    pub(crate) sets: Vec<StarterSetDef>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StarterSetDef {
    pub(crate) name: String,
    pub(crate) applies_to: AppliesToDef,
    pub(crate) grants: Vec<GrantDef>,
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub(crate) enum AppliesToDef {
    All,
    Race(RaceDef),
    Clothing(ClothingDef),
    Weapon(WeaponDef),
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub(crate) enum RaceDef {
    Chinese,
    European,
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub(crate) enum ClothingDef {
    Garment,
    Protector,
    Armor,
    Robe,
    LightArmor,
    HeavyArmor,
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub(crate) enum WeaponDef {
    Sword,
    Blade,
    Spear,
    Glavie,
    Bow,
    OneHandSword,
    TwoHandSword,
    Axe,
    WarlockStaff,
    Staff,
    Crossbow,
    Dagger,
    Harp,
    ClericRod,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GrantDef {
    pub(crate) code: String,
    #[serde(default = "default_amount")]
    pub(crate) amount: u16,
    #[serde(default)]
    pub(crate) upgrade: u8,
    #[serde(default)]
    pub(crate) placement: PlacementDef,
}

fn default_amount() -> u16 {
    1
}

#[derive(Copy, Clone, Debug, Default, Deserialize)]
pub(crate) enum PlacementDef {
    #[default]
    Bag,
    Equip(EquipmentSlotDef),
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub(crate) enum EquipmentSlotDef {
    HeadArmor,
    ShoulderArmor,
    WristArmor,
    ChestArmor,
    LegArmor,
    FootArmor,
    Weapon,
    SecondaryWeapon,
    Earring,
    Necklace,
    LeftRing,
    RightRing,
}

#[derive(Debug)]
pub(crate) struct SourcedSet {
    pub(crate) path: PathBuf,
    pub(crate) set: StarterSetDef,
}

pub(crate) fn load(directory: &Path) -> Result<Vec<SourcedSet>, StarterGearError> {
    let mut paths = Vec::new();
    discover(directory, &mut paths).map_err(|source| StarterGearError::DirectoryIo {
        path: directory.to_path_buf(),
        source,
    })?;
    paths.sort();

    if paths.is_empty() {
        return Err(StarterGearError::Validation(vec![ValidationIssue::new(
            directory,
            None,
            "contains no starter gear definition files",
        )]));
    }

    let mut sets = Vec::new();
    for path in paths {
        let content = fs::read_to_string(&path).map_err(|source| StarterGearError::DirectoryIo {
            path: path.clone(),
            source,
        })?;
        let file: StarterGearFileV1 = ron::from_str(&content).map_err(|source| StarterGearError::FileParse {
            path: path.clone(),
            source: Box::new(source),
        })?;
        if file.version != SUPPORTED_VERSION {
            return Err(StarterGearError::Validation(vec![ValidationIssue::new(
                &path,
                None,
                format!(
                    "unsupported schema version {}; expected {}",
                    file.version, SUPPORTED_VERSION
                ),
            )]));
        }
        sets.extend(file.sets.into_iter().map(|set| SourcedSet {
            path: path.clone(),
            set,
        }));
    }
    Ok(sets)
}

fn discover(directory: &Path, output: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            discover(&path, output)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("ron") {
            output.push(path);
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct ValidationIssue {
    pub path: PathBuf,
    pub set: Option<String>,
    pub reason: String,
}

impl ValidationIssue {
    pub(crate) fn new(path: &Path, set: Option<String>, reason: impl Into<String>) -> Self {
        Self {
            path: path.to_path_buf(),
            set,
            reason: reason.into(),
        }
    }
}

impl std::fmt::Display for ValidationIssue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.set {
            Some(set) => write!(
                formatter,
                "starter set '{set}' in {}: {}",
                self.path.display(),
                self.reason
            ),
            None => write!(formatter, "starter gear in {}: {}", self.path.display(), self.reason),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StarterGearError {
    #[error("could not read starter gear definitions from {path}: {source}")]
    DirectoryIo { path: PathBuf, source: std::io::Error },
    #[error("could not parse starter gear definitions in {path}: {source}")]
    FileParse {
        path: PathBuf,
        source: Box<ron::error::SpannedError>,
    },
    #[error("invalid starter gear definitions:\n{}", .0.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n"))]
    Validation(Vec<ValidationIssue>),
}
