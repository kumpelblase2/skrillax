use super::LootCompiler;
use crate::definition::{DefinitionSource, DistributionDef, LootValidationIssue};
use crate::runtime::CompiledDistribution;

impl LootCompiler<'_> {
    pub(super) fn compile_distribution<T>(
        &self,
        distribution: &DistributionDef<T>,
        source: &DefinitionSource,
        label: &str,
        issues: &mut Vec<LootValidationIssue>,
    ) -> Option<CompiledDistribution<T>>
    where
        T: Copy + Ord + Into<u64> + std::fmt::Debug,
    {
        let source = source.clone();
        let context =
            move |reason: String| Self::issue_at(source.clone(), "distribution", Some(label.to_string()), reason);

        match distribution {
            DistributionDef::Fixed(value) => Some(CompiledDistribution::Fixed(*value)),
            DistributionDef::Uniform { min, max } => {
                if min > max {
                    issues.push(context(format!("range is reversed ({:?}..={:?})", min, max)));
                    return None;
                }
                Some(CompiledDistribution::Uniform { min: *min, max: *max })
            },
            DistributionDef::Weighted(values) => {
                if values.is_empty() {
                    issues.push(context(String::from("weighted distribution must not be empty")));
                    return None;
                }

                let mut cumulative_vec = Vec::with_capacity(values.len());
                let mut cumulative = 0u64;
                for value in values {
                    if value.weight == 0 {
                        issues.push(context(String::from("weighted entry must have a positive weight")));
                        return None;
                    }
                    cumulative = match cumulative.checked_add(u64::from(value.weight)) {
                        Some(next) => next,
                        None => {
                            issues.push(context(String::from("cumulative weights overflow")));
                            return None;
                        },
                    };
                    cumulative_vec.push((cumulative, value.value));
                }
                Some(CompiledDistribution::Weighted {
                    cumulative: cumulative_vec,
                    total: cumulative,
                })
            },
        }
    }
}

pub(super) fn distribution_maximum<T: Copy + Ord>(distribution: &DistributionDef<T>) -> T {
    match distribution {
        DistributionDef::Fixed(value) => *value,
        DistributionDef::Uniform { max, .. } => *max,
        DistributionDef::Weighted(values) => values
            .iter()
            .map(|value| value.value)
            .max()
            .expect("weighted distributions are non-empty"),
    }
}
