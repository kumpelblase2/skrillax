use super::{CompiledProfileEntry, LootCompiler};
use crate::definition::{
    DefinitionSource, LootValidationIssue, NamedPoolDef, NamedProfileDef, RollDef, WeightedEntryDef,
};
use crate::runtime::{CompiledPool, CompiledProfile, CompiledRoll};
use std::collections::HashMap;

impl LootCompiler<'_> {
    pub(super) fn compile_all_profiles(
        &self,
        issues: &mut Vec<LootValidationIssue>,
    ) -> HashMap<String, CompiledProfileEntry> {
        let pools: HashMap<&str, &NamedPoolDef> = self
            .definitions
            .pools
            .iter()
            .map(|named| (named.value.name.as_str(), &named.value))
            .collect();
        let pool_sources: HashMap<&str, &DefinitionSource> = self
            .definitions
            .pools
            .iter()
            .map(|named| (named.value.name.as_str(), &named.source))
            .collect();

        let mut compiled = HashMap::new();
        for named in &self.definitions.profiles {
            let NamedProfileDef { name, rolls } = &named.value;
            if rolls.is_empty() {
                issues.push(Self::issue_at(
                    named.source.clone(),
                    "profile",
                    Some(name.clone()),
                    String::from("profile must contain at least one roll"),
                ));
                continue;
            }

            let mut compiled_rolls = Vec::with_capacity(rolls.len());
            for roll in rolls {
                if let Some(compiled_roll) = self.compile_roll(roll, &pools, &pool_sources, &named.source, issues) {
                    compiled_rolls.push(compiled_roll);
                }
            }

            compiled.insert(
                name.clone(),
                CompiledProfileEntry {
                    profile: CompiledProfile { rolls: compiled_rolls },
                    source: named.source.clone(),
                },
            );
        }
        compiled
    }

    fn compile_roll(
        &self,
        roll: &RollDef,
        pools: &HashMap<&str, &NamedPoolDef>,
        pool_sources: &HashMap<&str, &DefinitionSource>,
        profile_source: &DefinitionSource,
        issues: &mut Vec<LootValidationIssue>,
    ) -> Option<CompiledRoll> {
        if roll.attempts == 0 {
            issues.push(Self::issue_at(
                profile_source.clone(),
                "roll",
                Some(format!("pool '{}'", roll.pool)),
                String::from("attempts must be positive"),
            ));
            return None;
        }

        if !roll.chance.is_finite() || !(0.0..=1.0).contains(&roll.chance) {
            issues.push(Self::issue_at(
                profile_source.clone(),
                "roll",
                Some(format!("pool '{}'", roll.pool)),
                format!("chance must be within 0..=1, was {}", roll.chance),
            ));
            return None;
        }

        let pool_def = pools.get(roll.pool.as_str()).or_else(|| {
            issues.push(Self::issue_at(
                profile_source.clone(),
                "roll",
                Some(format!("pool '{}'", roll.pool)),
                String::from("references an unknown pool"),
            ));
            None
        })?;

        let pool_source = pool_sources.get(roll.pool.as_str())?;
        let pool = self.compile_pool(pool_def, pool_source, issues)?;
        Some(CompiledRoll {
            pool,
            attempts: roll.attempts,
            chance: roll.chance,
        })
    }

    fn compile_pool(
        &self,
        pool: &NamedPoolDef,
        source: &DefinitionSource,
        issues: &mut Vec<LootValidationIssue>,
    ) -> Option<CompiledPool> {
        if pool.entries.is_empty() {
            issues.push(Self::issue_at(
                source.clone(),
                "pool",
                Some(pool.name.clone()),
                String::from("pool must contain at least one entry"),
            ));
            return None;
        }

        let mut entries = Vec::with_capacity(pool.entries.len());
        let mut cumulative_weights = Vec::with_capacity(pool.entries.len());
        let mut cumulative = 0u64;
        for WeightedEntryDef { weight, generator } in &pool.entries {
            if *weight == 0 {
                issues.push(Self::issue_at(
                    source.clone(),
                    "pool entry",
                    Some(pool.name.clone()),
                    String::from("entry weight must be positive"),
                ));
                return None;
            }

            cumulative = match cumulative.checked_add(u64::from(*weight)) {
                Some(value) => value,
                None => {
                    issues.push(Self::issue_at(
                        source.clone(),
                        "pool",
                        Some(pool.name.clone()),
                        String::from("cumulative entry weights overflow"),
                    ));
                    return None;
                },
            };
            cumulative_weights.push(cumulative);
            entries.push(self.compile_generator(generator, source, &pool.name, issues)?);
        }

        Some(CompiledPool {
            entries,
            cumulative_weights,
            total_weight: cumulative,
        })
    }
}
