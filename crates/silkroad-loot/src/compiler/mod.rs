mod distributions;
mod generators;
mod modifiers;
mod plans;
mod profiles;

use crate::definition::{load_definitions, DefinitionSource, LootStartupError, LootValidationIssue, MergedDefinitions};
use crate::runtime::{CompiledProfile, GoldItemReferences, LootTables};
use silkroad_data::characterdata::RefCharacterData;
use silkroad_data::datamap::DataMap;
use silkroad_data::gold::GoldMap;
use silkroad_data::itemdata::RefItemData;
use std::collections::HashMap;
use std::path::Path;

impl LootTables {
    /// Loads every loot definition below the configured directory, validates
    /// them against the Media catalogue, and compiles them into immutable
    /// runtime tables. Any invalid definition aborts startup.
    pub fn load_and_compile(
        directory: &Path,
        rate: f64,
        items: &'static DataMap<RefItemData>,
        monsters: &'static DataMap<RefCharacterData>,
        gold: &'static GoldMap,
    ) -> Result<Self, LootStartupError> {
        let definitions = load_definitions(directory)?;
        LootCompiler::new(definitions, directory, rate, items, monsters, gold).compile()
    }
}

struct LootCompiler<'a> {
    definitions: MergedDefinitions,
    directory: &'a Path,
    rate: f64,
    items: &'static DataMap<RefItemData>,
    monsters: &'static DataMap<RefCharacterData>,
    gold: &'static GoldMap,
}

/// A compiled profile together with its originating definition, so later
/// stages can attribute diagnostics.
struct CompiledProfileEntry {
    profile: CompiledProfile,
    source: DefinitionSource,
}

impl<'a> LootCompiler<'a> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        definitions: MergedDefinitions,
        directory: &'a Path,
        rate: f64,
        items: &'static DataMap<RefItemData>,
        monsters: &'static DataMap<RefCharacterData>,
        gold: &'static GoldMap,
    ) -> Self {
        Self {
            definitions,
            directory,
            rate,
            items,
            monsters,
            gold,
        }
    }

    fn compile(self) -> Result<LootTables, LootStartupError> {
        let mut issues = Vec::new();

        self.validate_rate(&mut issues);
        let gold_items = self.resolve_gold_items(&mut issues);
        let modifiers = self.compile_modifiers(&mut issues);
        let default_profile = self.require_default_profile(&mut issues);
        let profiles = self.compile_all_profiles(&mut issues);
        self.validate_level_assignments(&mut issues);
        self.validate_monster_assignments(&profiles, &mut issues);

        let plans = match (&default_profile, &gold_items) {
            (Some(default), Ok(_)) => self.build_plans(default, &profiles, &modifiers, &mut issues),
            _ => HashMap::new(),
        };

        if issues.is_empty() {
            Ok(LootTables::new(
                plans,
                gold_items.expect("checked above"),
                modifiers,
                self.rate,
            ))
        } else {
            Err(LootStartupError::Validation(issues))
        }
    }

    fn source_path(&self) -> std::path::PathBuf {
        self.directory.to_path_buf()
    }

    fn issue(&self, kind: &'static str, name: Option<String>, reason: String) -> LootValidationIssue {
        LootValidationIssue {
            source: self.source_path(),
            kind,
            name,
            reason,
            original_source: None,
        }
    }

    fn issue_at(
        source: DefinitionSource,
        kind: &'static str,
        name: Option<String>,
        reason: String,
    ) -> LootValidationIssue {
        LootValidationIssue {
            source: source.path.clone(),
            kind,
            name,
            reason,
            original_source: None,
        }
    }

    fn validate_rate(&self, issues: &mut Vec<LootValidationIssue>) {
        if !self.rate.is_finite() || self.rate < 0.0 {
            issues.push(self.issue(
                "configuration",
                Some(String::from("rate")),
                format!("global loot rate must be finite and non-negative, was {}", self.rate),
            ));
        }
    }

    fn resolve_gold_items(&self, issues: &mut Vec<LootValidationIssue>) -> Result<GoldItemReferences, ()> {
        GoldItemReferences::resolve(self.items).map_err(|missing| {
            issues.push(self.issue(
                "catalogue",
                Some(String::from("gold items")),
                format!(
                    "gold item references are missing from the media catalogue: {:?}",
                    missing
                ),
            ));
        })
    }

    fn require_default_profile(&self, issues: &mut Vec<LootValidationIssue>) -> Option<String> {
        match &self.definitions.default_profile {
            Some(default) => Some(default.value.clone()),
            None => {
                issues.push(self.issue(
                    "default profile",
                    None,
                    String::from("exactly one default profile must be declared"),
                ));
                None
            },
        }
    }
}

#[cfg(test)]
mod tests;
