use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

/// The currently supported loot definition schema version.
pub(crate) const SUPPORTED_VERSION: u32 = 1;

/// Serde-facing root of a single loot `.ron` file.
///
/// Files may contribute any subset of definitions; they are merged without
/// order-based precedence. Vectors of named records are used instead of maps
/// so duplicate names can be detected reliably.
#[derive(Deserialize, Debug)]
pub(crate) struct LootFileV1 {
    pub(crate) version: u32,
    #[serde(default)]
    pub(crate) default_profile: Option<String>,
    #[serde(default)]
    pub(crate) pools: Vec<NamedPoolDef>,
    #[serde(default)]
    pub(crate) profiles: Vec<NamedProfileDef>,
    #[serde(default)]
    pub(crate) level_assignments: Vec<LevelAssignmentDef>,
    #[serde(default)]
    pub(crate) monster_assignments: Vec<MonsterAssignmentDef>,
    #[serde(default)]
    pub(crate) rarity_modifiers: Vec<RarityModifierDef>,
    #[serde(default)]
    pub(crate) party_modifier: Option<PartyModifierDef>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct NamedPoolDef {
    pub(crate) name: String,
    pub(crate) entries: Vec<WeightedEntryDef>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct WeightedEntryDef {
    pub(crate) weight: u32,
    pub(crate) generator: GeneratorDef,
}

#[derive(Deserialize, Debug)]
pub(crate) enum GeneratorDef {
    Gold,
    Item(ItemGeneratorDef),
    Equipment(EquipmentSelectorDef),
}

#[derive(Deserialize, Debug)]
pub(crate) struct ItemGeneratorDef {
    pub(crate) code: String,
    pub(crate) amount: DistributionDef<u16>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct NamedProfileDef {
    pub(crate) name: String,
    pub(crate) rolls: Vec<RollDef>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct RollDef {
    pub(crate) pool: String,
    pub(crate) attempts: u32,
    pub(crate) chance: f64,
}

#[derive(Deserialize, Debug)]
pub(crate) struct LevelAssignmentDef {
    /// Inclusive monster level range this profile replaces the default for.
    pub(crate) levels: InclusiveRangeDef<u8>,
    pub(crate) profile: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct MonsterAssignmentDef {
    pub(crate) monster: String,
    pub(crate) mode: AssignmentMode,
    pub(crate) profile: String,
}

#[derive(Copy, Clone, Deserialize, Debug, Eq, PartialEq)]
pub(crate) enum AssignmentMode {
    Add,
    Replace,
}

#[derive(Deserialize, Debug)]
pub(crate) struct RarityModifierDef {
    pub(crate) rarity: RarityKindDef,
    pub(crate) attempts_multiplier: u32,
    pub(crate) chance_multiplier: f64,
    pub(crate) gold_amount_multiplier: f64,
    pub(crate) stack_amount_multiplier: f64,
    pub(crate) equipment_upgrade_bonus: DistributionDef<u8>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct PartyModifierDef {
    pub(crate) attempts_multiplier: u32,
    pub(crate) chance_multiplier: f64,
    pub(crate) gold_amount_multiplier: f64,
    pub(crate) stack_amount_multiplier: f64,
    pub(crate) equipment_upgrade_bonus: DistributionDef<u8>,
}

#[derive(Copy, Clone, Deserialize, Debug, Eq, PartialEq, Hash)]
pub(crate) enum RarityKindDef {
    Normal,
    Champion,
    Unique,
    Giant,
    Titan,
    Elite,
    Strong,
    Unique2,
}

#[derive(Deserialize, Debug)]
pub(crate) struct EquipmentSelectorDef {
    /// Required level window relative to the killed monster's level, inclusive
    /// on both ends. May be negative to select lower-level equipment.
    pub(crate) required_level: AroundMonsterDef,
    pub(crate) origins: Vec<OriginDef>,
    pub(crate) kinds: Vec<EquipmentKindDef>,
    pub(crate) item_rarities: Vec<ItemRarityDef>,
    pub(crate) upgrade: DistributionDef<u8>,
}

#[derive(Copy, Clone, Deserialize, Debug)]
#[serde(rename = "AroundMonster")]
pub(crate) struct AroundMonsterDef {
    pub(crate) min: i16,
    pub(crate) max: i16,
}

#[derive(Copy, Clone, Deserialize, Debug, Eq, PartialEq)]
pub(crate) enum OriginDef {
    Chinese,
    European,
}

#[derive(Copy, Clone, Deserialize, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentKindDef {
    Weapon,
    Shield,
    Clothing,
    Jewelry,
}

#[derive(Copy, Clone, Deserialize, Debug, Eq, PartialEq)]
pub(crate) enum ItemRarityDef {
    General,
    Blue,
    Seal,
    Set,
    Roc,
    Legend,
}

#[derive(Clone, Deserialize, Debug)]
pub(crate) enum DistributionDef<T> {
    Fixed(T),
    Uniform { min: T, max: T },
    Weighted(Vec<WeightedValueDef<T>>),
}

#[derive(Clone, Deserialize, Debug)]
pub(crate) struct WeightedValueDef<T> {
    pub(crate) weight: u32,
    pub(crate) value: T,
}

#[derive(Copy, Clone, Deserialize, Debug)]
pub(crate) struct InclusiveRangeDef<T> {
    pub(crate) min: T,
    pub(crate) max: T,
}

/// Location a single definition originated from.
#[derive(Clone, Debug)]
pub(crate) struct DefinitionSource {
    pub(crate) path: PathBuf,
    /// Index of the record within its file (for diagnostics).
    pub(crate) index: usize,
}

#[derive(Debug)]
pub(crate) struct NamedDefinition<T> {
    pub(crate) source: DefinitionSource,
    pub(crate) value: T,
}

impl<T> NamedDefinition<T> {
    fn new(path: &Path, index: usize, value: T) -> Self {
        Self {
            source: DefinitionSource {
                path: path.to_path_buf(),
                index,
            },
            value,
        }
    }
}

/// All definitions merged across every discovered file.
#[derive(Debug, Default)]
pub(crate) struct MergedDefinitions {
    pub(crate) default_profile: Option<NamedDefinition<String>>,
    pub(crate) pools: Vec<NamedDefinition<NamedPoolDef>>,
    pub(crate) profiles: Vec<NamedDefinition<NamedProfileDef>>,
    pub(crate) level_assignments: Vec<NamedDefinition<LevelAssignmentDef>>,
    pub(crate) monster_assignments: Vec<NamedDefinition<MonsterAssignmentDef>>,
    pub(crate) rarity_modifiers: Vec<NamedDefinition<RarityModifierDef>>,
    pub(crate) party_modifier: Option<NamedDefinition<PartyModifierDef>>,
}

/// A single collected validation failure with its origin.
#[derive(Debug)]
pub struct LootValidationIssue {
    pub(crate) source: PathBuf,
    pub(crate) kind: &'static str,
    pub(crate) name: Option<String>,
    pub(crate) reason: String,
    /// Where the colliding original was defined, for duplicate errors.
    pub(crate) original_source: Option<PathBuf>,
}

impl std::fmt::Display for LootValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ", self.kind)?;
        match &self.name {
            Some(name) => write!(f, "'{}' ", name)?,
            None => write!(f, "? ")?,
        }
        write!(f, "in {:?}: {}", self.source.display(), self.reason)?;
        if let Some(original) = &self.original_source {
            write!(f, " (first defined in {:?})", original.display())?;
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LootStartupError {
    #[error("Could not read loot definitions from {path:?}: {source}")]
    DirectoryIo { path: PathBuf, source: std::io::Error },
    #[error("Could not parse loot definitions in {path:?}: {source}")]
    FileParse {
        path: PathBuf,
        source: ron::error::SpannedError,
    },
    #[error("Invalid loot definitions:\n{}", .0.iter().map(|issue| issue.to_string()).collect::<Vec<_>>().join("\n"))]
    Validation(Vec<LootValidationIssue>),
}

/// Discovers and parses all `.ron` files below `directory` (recursively,
/// sorted lexically for deterministic diagnostics) and merges them into a
/// single model, rejecting duplicates.
pub(crate) fn load_definitions(directory: &Path) -> Result<MergedDefinitions, LootStartupError> {
    let paths = discover_files(directory).map_err(|source| LootStartupError::DirectoryIo {
        path: directory.to_path_buf(),
        source,
    })?;

    if paths.is_empty() {
        return Err(LootStartupError::Validation(vec![LootValidationIssue {
            source: directory.to_path_buf(),
            kind: "directory",
            name: None,
            reason: String::from("contains no loot definition files"),
            original_source: None,
        }]));
    }

    let mut files = Vec::with_capacity(paths.len());
    for path in paths {
        let content = fs::read_to_string(&path).map_err(|source| LootStartupError::DirectoryIo {
            path: path.clone(),
            source,
        })?;
        let file: LootFileV1 = ron::from_str(&content).map_err(|source| LootStartupError::FileParse {
            path: path.clone(),
            source,
        })?;
        files.push((path, file));
    }

    merge(files)
}

fn discover_files(directory: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    discover_recursive(directory, &mut files)?;
    files.sort();
    Ok(files)
}

fn discover_recursive(directory: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            discover_recursive(&path, files)?;
        } else if path.extension().is_some_and(|extension| extension == "ron") {
            files.push(path);
        }
    }
    Ok(())
}

fn merge(files: Vec<(PathBuf, LootFileV1)>) -> Result<MergedDefinitions, LootStartupError> {
    let mut issues = Vec::new();
    let mut merged = MergedDefinitions::default();

    for (path, file) in files {
        if file.version != SUPPORTED_VERSION {
            issues.push(LootValidationIssue {
                source: path.clone(),
                kind: "file",
                name: None,
                reason: format!(
                    "unsupported schema version {}, expected {}",
                    file.version, SUPPORTED_VERSION
                ),
                original_source: None,
            });
            continue;
        }

        merge_file(&path, file, &mut merged, &mut issues);
    }

    if issues.is_empty() {
        Ok(merged)
    } else {
        Err(LootStartupError::Validation(issues))
    }
}

fn merge_file(path: &Path, file: LootFileV1, merged: &mut MergedDefinitions, issues: &mut Vec<LootValidationIssue>) {
    if let Some(default_profile) = file.default_profile {
        if let Some(existing) = &merged.default_profile {
            issues.push(LootValidationIssue {
                source: path.to_path_buf(),
                kind: "default profile",
                name: Some(default_profile),
                reason: String::from("more than one default profile declared"),
                original_source: Some(existing.source.path.clone()),
            });
        } else {
            merged.default_profile = Some(NamedDefinition::new(path, 0, default_profile));
        }
    }

    for (index, pool) in file.pools.into_iter().enumerate() {
        if let Some(existing) = merged.pools.iter().find(|other| other.value.name == pool.name) {
            issues.push(duplicate("pool", &pool.name, path, existing));
        } else {
            merged.pools.push(NamedDefinition::new(path, index, pool));
        }
    }

    for (index, profile) in file.profiles.into_iter().enumerate() {
        if let Some(existing) = merged.profiles.iter().find(|other| other.value.name == profile.name) {
            issues.push(duplicate("profile", &profile.name, path, existing));
        } else {
            merged.profiles.push(NamedDefinition::new(path, index, profile));
        }
    }

    for (index, assignment) in file.level_assignments.into_iter().enumerate() {
        merged
            .level_assignments
            .push(NamedDefinition::new(path, index, assignment));
    }

    for (index, assignment) in file.monster_assignments.into_iter().enumerate() {
        if let Some(existing) = merged
            .monster_assignments
            .iter()
            .find(|other| other.value.monster == assignment.monster)
        {
            issues.push(duplicate("monster assignment", &assignment.monster, path, existing));
        } else {
            merged
                .monster_assignments
                .push(NamedDefinition::new(path, index, assignment));
        }
    }

    for (index, modifier) in file.rarity_modifiers.into_iter().enumerate() {
        if let Some(existing) = merged
            .rarity_modifiers
            .iter()
            .find(|other| other.value.rarity == modifier.rarity)
        {
            issues.push(duplicate(
                "rarity modifier",
                &format!("{:?}", modifier.rarity),
                path,
                existing,
            ));
        } else {
            merged
                .rarity_modifiers
                .push(NamedDefinition::new(path, index, modifier));
        }
    }

    if let Some(party_modifier) = file.party_modifier {
        if let Some(existing) = &merged.party_modifier {
            issues.push(LootValidationIssue {
                source: path.to_path_buf(),
                kind: "party modifier",
                name: None,
                reason: String::from("more than one party modifier declared"),
                original_source: Some(existing.source.path.clone()),
            });
        } else {
            merged.party_modifier = Some(NamedDefinition::new(path, 0, party_modifier));
        }
    }
}

fn duplicate<T>(kind: &'static str, name: &str, path: &Path, existing: &NamedDefinition<T>) -> LootValidationIssue {
    LootValidationIssue {
        source: path.to_path_buf(),
        kind,
        name: Some(name.to_string()),
        reason: String::from("duplicate definition"),
        original_source: Some(existing.source.path.clone()),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    fn write_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, content).unwrap();
        path
    }

    const VALID_FILE: &str = r#"
(
    version: 1,
    default_profile: Some("world-default"),
    pools: [
        (
            name: "gold",
            entries: [(weight: 1, generator: Gold)],
        ),
    ],
    profiles: [
        (
            name: "world-default",
            rolls: [(pool: "gold", attempts: 1, chance: 1.0)],
        ),
    ],
)
"#;

    #[test]
    fn load_single_valid_file() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "base.ron", VALID_FILE);

        let merged = load_definitions(dir.path()).unwrap();
        assert_eq!(merged.pools.len(), 1);
        assert_eq!(merged.pools[0].value.name, "gold");
        assert_eq!(merged.profiles.len(), 1);
        assert_eq!(merged.default_profile.as_ref().unwrap().value, "world-default");
    }

    #[test]
    fn load_multiple_files() {
        let subdir = tempfile::tempdir().unwrap();
        let nested = subdir.path().join("nested");
        std::fs::create_dir(&nested).unwrap();

        write_file(subdir.path(), "a.ron", VALID_FILE);
        write_file(
            &nested,
            "extra.ron",
            r#"
(
    version: 1,
    pools: [
        (
            name: "consumables",
            entries: [(
                weight: 1,
                generator: Item((
                    code: "ITEM_ETC_HP_POTION_01",
                    amount: Uniform(min: 1, max: 3),
                )),
            )],
        ),
    ],
)
"#,
        );

        let merged = load_definitions(subdir.path()).unwrap();
        assert_eq!(merged.pools.len(), 2);
        assert!(merged.pools.iter().any(|pool| pool.value.name == "consumables"));
    }

    #[test]
    fn reject_unsupported_version() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "future.ron", "(version: 2)");

        let error = load_definitions(dir.path()).unwrap_err();
        assert!(matches!(error, LootStartupError::Validation(_)));
        assert!(error.to_string().contains("unsupported schema version 2"));
    }

    #[test]
    fn detect_duplicate_pool_across_files() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "a.ron", VALID_FILE);
        write_file(dir.path(), "b.ron", VALID_FILE);

        let error = load_definitions(dir.path()).unwrap_err();
        let LootStartupError::Validation(issues) = error else {
            panic!("expected validation issues");
        };
        assert!(issues
            .iter()
            .any(|issue| issue.kind == "pool" && issue.original_source.is_some()));
    }

    #[test]
    fn duplicate_detection_is_order_independent() {
        // Reversing the lexical discovery order must produce the same
        // duplicate error regardless of which file comes first.
        let issues_of = |first: &str, second: &str| {
            let dir = tempfile::tempdir().unwrap();
            write_file(dir.path(), first, VALID_FILE);
            write_file(dir.path(), second, VALID_FILE);
            let LootStartupError::Validation(issues) = load_definitions(dir.path()).unwrap_err() else {
                panic!("expected validation issues");
            };
            let mut descriptions: Vec<String> = issues
                .iter()
                .map(|issue| format!("{}|{:?}", issue.kind, issue.name))
                .collect();
            descriptions.sort();
            descriptions
        };

        assert_eq!(issues_of("a.ron", "b.ron"), issues_of("z.ron", "b.ron"));
    }

    #[test]
    fn detect_multiple_defaults() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "a.ron", VALID_FILE);
        write_file(dir.path(), "b.ron", VALID_FILE);

        let LootStartupError::Validation(issues) = load_definitions(dir.path()).unwrap_err() else {
            panic!("expected validation issues");
        };
        assert!(issues.iter().any(|issue| issue.kind == "default profile"));
    }

    #[test]
    fn detect_duplicate_monster_assignment_and_rarity_modifier() {
        let content = r#"
(
    version: 1,
    monster_assignments: [
        (
            monster: "MOB_CH_TIGERWOMAN",
            mode: Add,
            profile: "extra",
        ),
    ],
    rarity_modifiers: [
        (
            rarity: Champion,
            attempts_multiplier: 2,
            chance_multiplier: 1.0,
            gold_amount_multiplier: 1.5,
            stack_amount_multiplier: 1.5,
            equipment_upgrade_bonus: Fixed(0),
        ),
    ],
)
"#;
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "a.ron", content);
        write_file(dir.path(), "b.ron", content);

        let LootStartupError::Validation(issues) = load_definitions(dir.path()).unwrap_err() else {
            panic!("expected validation issues");
        };
        assert!(issues.iter().any(|issue| issue.kind == "monster assignment"));
        assert!(issues.iter().any(|issue| issue.kind == "rarity modifier"));
    }

    #[test]
    fn detect_multiple_party_modifiers() {
        let party_modifier = r#"party_modifier: Some((
                attempts_multiplier: 1,
                chance_multiplier: 1.5,
                gold_amount_multiplier: 1.5,
                stack_amount_multiplier: 1.5,
                equipment_upgrade_bonus: Fixed(0),
            ))"#;
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "a.ron", &format!("(version: 1, {party_modifier})"));
        write_file(dir.path(), "b.ron", &format!("(version: 1, {party_modifier})"));

        let LootStartupError::Validation(issues) = load_definitions(dir.path()).unwrap_err() else {
            panic!("expected validation issues");
        };
        assert!(issues.iter().any(|issue| issue.kind == "party modifier"));
    }

    #[test]
    fn report_parse_error_with_location() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "broken.ron", "(version: 1, pools: [");

        let error = load_definitions(dir.path()).unwrap_err();
        let LootStartupError::FileParse { source, .. } = error else {
            panic!("expected parse error");
        };
        let message = source.to_string();
        assert!(message.contains(':'), "expected line/column info in {}", message);
    }

    #[test]
    fn missing_directory_is_io_error() {
        let error = load_definitions(Path::new("/definitely/not/here")).unwrap_err();
        assert!(matches!(error, LootStartupError::DirectoryIo { .. }));
    }

    #[test]
    fn empty_directory_is_validation_error() {
        let dir = tempfile::tempdir().unwrap();
        let error = load_definitions(dir.path()).unwrap_err();
        assert!(error.to_string().contains("no loot definition files"));
    }

    /// The shipped default definitions must always remain loadable.
    #[test]
    fn shipped_base_definitions_parse() {
        let content = include_str!("../../../configs/loot/base.ron");
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "base.ron", content);

        let merged = load_definitions(dir.path()).unwrap();
        assert_eq!(merged.pools[0].value.name, "gold");
        assert_eq!(merged.default_profile.unwrap().value, "world-default");
    }
}
