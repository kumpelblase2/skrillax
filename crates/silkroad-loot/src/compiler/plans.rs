use super::{CompiledProfileEntry, LootCompiler};
use crate::definition::{AssignmentMode, DefinitionSource, LevelAssignmentDef, LootValidationIssue};
use crate::runtime::{
    CompiledGenerator, CompiledModifiers, CompiledMonsterPlan, CompiledProfile, WorstCaseFactors, MAX_DROPS_PER_DEATH,
};
use silkroad_data::characterdata::RefCharacterData;
use silkroad_definitions::type_id::{ObjectMonster, ObjectType};
use std::collections::HashMap;
use std::path::Path;

impl LootCompiler<'_> {
    pub(super) fn validate_level_assignments(&self, issues: &mut Vec<LootValidationIssue>) {
        let profile_names: Vec<&str> = self
            .definitions
            .profiles
            .iter()
            .map(|profile| profile.value.name.as_str())
            .collect();

        let mut sorted: Vec<&LevelAssignmentDef> = self
            .definitions
            .level_assignments
            .iter()
            .map(|named| &named.value)
            .collect();
        sorted.sort_by_key(|assignment| assignment.levels.min);

        for window in sorted.windows(2) {
            let (first, second) = (window[0], window[1]);
            if second.levels.min <= first.levels.max {
                issues.push(self.issue(
                    "level assignment",
                    Some(format!("{}..={}", first.levels.min, first.levels.max)),
                    format!(
                        "overlapping level assignment ({}..={})",
                        second.levels.min, second.levels.max
                    ),
                ));
            }
        }

        for named in &self.definitions.level_assignments {
            if named.value.levels.min > named.value.levels.max {
                issues.push(Self::issue_at(
                    named.source.clone(),
                    "level assignment",
                    None,
                    format!(
                        "level range is reversed ({}..={})",
                        named.value.levels.min, named.value.levels.max
                    ),
                ));
            }

            if !profile_names.contains(&named.value.profile.as_str()) {
                issues.push(Self::issue_at(
                    named.source.clone(),
                    "level assignment",
                    None,
                    format!("references unknown profile '{}'", named.value.profile),
                ));
            }
        }
    }

    pub(super) fn validate_monster_assignments(
        &self,
        profiles: &HashMap<String, CompiledProfileEntry>,
        issues: &mut Vec<LootValidationIssue>,
    ) {
        for named in &self.definitions.monster_assignments {
            if self.monsters.find_code(&named.value.monster).is_none() {
                issues.push(Self::issue_at(
                    named.source.clone(),
                    "monster assignment",
                    Some(named.value.monster.clone()),
                    String::from("references an unknown monster code"),
                ));
            }

            if !profiles.contains_key(&named.value.profile) {
                issues.push(Self::issue_at(
                    named.source.clone(),
                    "monster assignment",
                    Some(named.value.monster.clone()),
                    format!("references unknown profile '{}'", named.value.profile),
                ));
            }
        }
    }

    pub(super) fn build_plans(
        &self,
        default_profile: &str,
        profiles: &HashMap<String, CompiledProfileEntry>,
        modifiers: &CompiledModifiers,
        issues: &mut Vec<LootValidationIssue>,
    ) -> HashMap<u32, CompiledMonsterPlan> {
        let mut plans = HashMap::new();

        for character in self.monsters.iter() {
            if !is_monster(character) {
                continue;
            }

            let level_assignment = self
                .definitions
                .level_assignments
                .iter()
                .find(|named| named.value.levels.min <= character.level && character.level <= named.value.levels.max)
                .map(|named| named.value.profile.as_str());

            let base_profile = level_assignment.unwrap_or(default_profile);
            let assignment = self
                .definitions
                .monster_assignments
                .iter()
                .find(|named| named.value.monster == character.common.id);

            let (executed, executed_source) = match assignment {
                Some(assignment) if assignment.value.mode == AssignmentMode::Replace => (
                    vec![assignment.value.profile.as_str()],
                    profiles
                        .get(&assignment.value.profile)
                        .map(|entry| entry.source.clone()),
                ),
                Some(assignment) => (
                    vec![base_profile, assignment.value.profile.as_str()],
                    profiles.get(base_profile).map(|entry| entry.source.clone()),
                ),
                None => (
                    vec![base_profile],
                    profiles.get(base_profile).map(|entry| entry.source.clone()),
                ),
            };

            if executed.iter().any(|name| !profiles.contains_key(*name)) {
                // Already reported by earlier validations.
                continue;
            }

            let compiled_profiles: Vec<CompiledProfile> =
                executed.iter().map(|name| profiles[*name].profile.clone()).collect();

            let uses_gold = compiled_profiles
                .iter()
                .flat_map(|profile| profile.rolls.iter())
                .any(|roll| {
                    roll.pool
                        .entries
                        .iter()
                        .any(|generator| matches!(generator, CompiledGenerator::Gold))
                });

            let gold_range = if uses_gold {
                match self.gold.range_for_level(character.level) {
                    Some(range) if range.start() <= range.end() => Some((*range.start(), *range.end())),
                    Some(range) => {
                        issues.push(Self::issue_at(
                            source_or_config(self.directory, executed_source.as_ref()),
                            "generator",
                            Some(format!("gold for {}", character.common.id)),
                            format!(
                                "gold range for level {} is reversed ({}..={})",
                                character.level,
                                range.start(),
                                range.end()
                            ),
                        ));
                        None
                    },
                    None => {
                        issues.push(Self::issue_at(
                            source_or_config(self.directory, executed_source.as_ref()),
                            "generator",
                            Some(format!("gold for {}", character.common.id)),
                            format!("no gold range is defined for monster level {}", character.level),
                        ));
                        None
                    },
                }
            } else {
                None
            };

            self.validate_equipment_matches(character, &compiled_profiles, executed_source.as_ref(), issues);

            self.validate_output_limit(
                character,
                &compiled_profiles,
                executed_source.as_ref(),
                modifiers,
                issues,
            );

            plans.insert(
                character.common.ref_id,
                CompiledMonsterPlan {
                    monster_level: character.level,
                    gold_range,
                    profiles: compiled_profiles,
                },
            );
        }

        plans
    }

    /// Validates that every equipment generator in an executed profile can
    /// produce output for this specific monster. Gold ranges are validated
    /// while resolving the plan's gold range.
    fn validate_equipment_matches(
        &self,
        character: &RefCharacterData,
        profiles: &[CompiledProfile],
        source: Option<&DefinitionSource>,
        issues: &mut Vec<LootValidationIssue>,
    ) {
        for profile in profiles {
            for roll in &profile.rolls {
                for generator in &roll.pool.entries {
                    if let CompiledGenerator::Equipment(candidates) = generator {
                        if !candidates.has_match(character.level) {
                            issues.push(Self::issue_at(
                                source_or_config(self.directory, source),
                                "generator",
                                Some(format!("equipment for {}", character.common.id)),
                                format!("equipment selector has no match for monster level {}", character.level),
                            ));
                        }
                    }
                }
            }
        }
    }

    /// Rejects configurations whose worst-case output for a single death
    /// exceeds the safety limit.
    fn validate_output_limit(
        &self,
        character: &RefCharacterData,
        profiles: &[CompiledProfile],
        source: Option<&DefinitionSource>,
        modifiers: &CompiledModifiers,
        issues: &mut Vec<LootValidationIssue>,
    ) {
        let factors = WorstCaseFactors::from_modifiers(modifiers);
        let bound = output_upper_bound(profiles, self.rate, factors);
        if bound > MAX_DROPS_PER_DEATH as u64 {
            issues.push(Self::issue_at(
                source_or_config(self.directory, source),
                "plan",
                Some(character.common.id.clone()),
                format!(
                    "worst-case output of {} drops exceeds the safety limit of {}",
                    bound, MAX_DROPS_PER_DEATH
                ),
            ));
        }
    }
}

fn source_or_config(directory: &Path, source: Option<&DefinitionSource>) -> DefinitionSource {
    source.cloned().unwrap_or_else(|| DefinitionSource {
        path: directory.to_path_buf(),
        index: 0,
    })
}

/// Upper bound of generated drops for one death across the given profiles.
fn output_upper_bound(profiles: &[CompiledProfile], rate: f64, factors: WorstCaseFactors) -> u64 {
    let mut total = 0u64;
    for profile in profiles {
        for roll in &profile.rolls {
            let expected = (roll.chance * rate * factors.chance_multiplier).clamp(0.0, f64::from(u32::MAX));
            let successes_per_attempt = expected.ceil() as u64;
            total = total.saturating_add(
                (u64::from(roll.attempts))
                    .saturating_mul(factors.attempts_multiplier)
                    .saturating_mul(successes_per_attempt),
            );
        }
    }
    total
}

fn is_monster(character: &RefCharacterData) -> bool {
    use silkroad_definitions::type_id::{ObjectEntity, ObjectNonPlayer};
    matches!(
        ObjectType::from_type_id(&character.common.type_id),
        Some(ObjectType::Entity(ObjectEntity::NonPlayer(ObjectNonPlayer::Monster(
            ObjectMonster::General
        ))))
    )
}
