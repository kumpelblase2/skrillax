use super::generators::rarity_kind;
use super::LootCompiler;
use crate::definition::{DefinitionSource, DistributionDef, LootValidationIssue};
use crate::runtime::{CompiledModifier, CompiledModifiers};

impl LootCompiler<'_> {
    pub(super) fn compile_modifiers(&self, issues: &mut Vec<LootValidationIssue>) -> CompiledModifiers {
        let mut modifiers = CompiledModifiers::default();
        for named in &self.definitions.rarity_modifiers {
            if let Some(compiled) = self.compile_modifier(
                &named.value.attempts_multiplier,
                &named.value.chance_multiplier,
                &named.value.gold_amount_multiplier,
                &named.value.stack_amount_multiplier,
                &named.value.equipment_upgrade_bonus,
                named.source.clone(),
                issues,
            ) {
                modifiers.rarity.push((rarity_kind(named.value.rarity), compiled));
            }
        }

        if let Some(party) = &self.definitions.party_modifier {
            if let Some(compiled) = self.compile_modifier(
                &party.value.attempts_multiplier,
                &party.value.chance_multiplier,
                &party.value.gold_amount_multiplier,
                &party.value.stack_amount_multiplier,
                &party.value.equipment_upgrade_bonus,
                party.source.clone(),
                issues,
            ) {
                modifiers.party = Some(compiled);
            }
        }

        modifiers
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_modifier(
        &self,
        attempts_multiplier: &u32,
        chance_multiplier: &f64,
        gold_amount_multiplier: &f64,
        stack_amount_multiplier: &f64,
        upgrade_bonus: &DistributionDef<u8>,
        source: DefinitionSource,
        issues: &mut Vec<LootValidationIssue>,
    ) -> Option<CompiledModifier> {
        if *attempts_multiplier == 0 {
            issues.push(Self::issue_at(
                source.clone(),
                "modifier",
                None,
                String::from("attempts multiplier must be positive"),
            ));
            return None;
        }

        for (label, multiplier) in [
            ("chance", *chance_multiplier),
            ("gold amount", *gold_amount_multiplier),
            ("stack amount", *stack_amount_multiplier),
        ] {
            if !multiplier.is_finite() || multiplier < 0.0 {
                issues.push(Self::issue_at(
                    source.clone(),
                    "modifier",
                    None,
                    format!(
                        "{} multiplier must be finite and non-negative, was {}",
                        label, multiplier
                    ),
                ));
                return None;
            }
        }

        let upgrade = self.compile_distribution(upgrade_bonus, &source, "upgrade bonus", issues)?;

        Some(CompiledModifier {
            attempts_multiplier: *attempts_multiplier,
            chance_multiplier: *chance_multiplier,
            gold_amount_multiplier: *gold_amount_multiplier,
            stack_amount_multiplier: *stack_amount_multiplier,
            equipment_upgrade_bonus: upgrade,
        })
    }
}
