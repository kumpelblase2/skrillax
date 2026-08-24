use super::{
    CompiledGenerator, CompiledModifier, CompiledModifiers, CompiledMonsterPlan, CompiledPool, GoldItemReferences,
    LootRandom,
};
use silkroad_definitions::rarity::EntityRarity;
use silkroad_game_base::{Item, ItemTypeData};
use std::collections::HashMap;

/// Defensive upper bound of generated drops per monster death. Configurations
/// able to exceed this limit are rejected during startup compilation.
pub(crate) const MAX_DROPS_PER_DEATH: usize = 128;

/// Immutable, precompiled runtime representation of all loot definitions.
pub struct LootTables {
    plans: HashMap<u32, CompiledMonsterPlan>,
    gold_items: GoldItemReferences,
    modifiers: CompiledModifiers,
    rate: f64,
}

impl std::fmt::Debug for LootTables {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LootTables")
            .field("plan_count", &self.plans.len())
            .field("rate", &self.rate)
            .finish()
    }
}

impl LootTables {
    pub(crate) fn new(
        plans: HashMap<u32, CompiledMonsterPlan>,
        gold_items: GoldItemReferences,
        modifiers: CompiledModifiers,
        rate: f64,
    ) -> Self {
        Self {
            plans,
            gold_items,
            modifiers,
            rate,
        }
    }

    pub fn has_plan(&self, monster_ref_id: u32) -> bool {
        self.plans.contains_key(&monster_ref_id)
    }

    #[cfg(test)]
    pub(crate) fn plans(&self) -> &HashMap<u32, CompiledMonsterPlan> {
        &self.plans
    }

    /// Constructs a gold drop item with the appropriate visual tier.
    pub fn gold_item(&self, amount: u32) -> Item {
        self.gold_items.construct_item(amount)
    }

    /// Generates the drops for a dead monster. Unknown monster reference ids
    /// yield no loot; callers are expected to warn about them.
    pub fn generate(&self, monster_ref_id: u32, rarity: EntityRarity, rng: &mut impl LootRandom) -> Vec<Item> {
        let Some(plan) = self.plans.get(&monster_ref_id) else {
            return Vec::new();
        };

        let kind_modifier = self.modifiers.for_kind(rarity.kind());
        let party_modifier = rarity.is_party().then(|| self.modifiers.party()).flatten();

        let attempts_multiplier =
            kind_modifier.map_or(1, |m| m.attempts_multiplier) * party_modifier.map_or(1, |m| m.attempts_multiplier);
        let chance_multiplier =
            kind_modifier.map_or(1.0, |m| m.chance_multiplier) * party_modifier.map_or(1.0, |m| m.chance_multiplier);
        let gold_multiplier = kind_modifier.map_or(1.0, |m| m.gold_amount_multiplier)
            * party_modifier.map_or(1.0, |m| m.gold_amount_multiplier);
        let stack_multiplier = kind_modifier.map_or(1.0, |m| m.stack_amount_multiplier)
            * party_modifier.map_or(1.0, |m| m.stack_amount_multiplier);

        let mut items = Vec::new();
        'outer: for profile in &plan.profiles {
            for roll in &profile.rolls {
                // Per attempt: expected selections scaled by global rate and
                // chance modifiers, split into guaranteed selections plus one
                // probabilistic selection for the fractional remainder.
                let expected = roll.chance * self.rate * chance_multiplier;
                let guaranteed = expected.floor();
                let remainder = expected - guaranteed;
                for _ in 0..roll.attempts.saturating_mul(attempts_multiplier) {
                    for _ in 0..(guaranteed as usize) {
                        if items.len() >= MAX_DROPS_PER_DEATH {
                            break 'outer;
                        }
                        self.execute(
                            plan,
                            &roll.pool,
                            kind_modifier,
                            party_modifier,
                            gold_multiplier,
                            stack_multiplier,
                            rng,
                            &mut items,
                        );
                    }
                    if remainder > 0.0 && rng.probability(remainder) {
                        if items.len() >= MAX_DROPS_PER_DEATH {
                            break 'outer;
                        }
                        self.execute(
                            plan,
                            &roll.pool,
                            kind_modifier,
                            party_modifier,
                            gold_multiplier,
                            stack_multiplier,
                            rng,
                            &mut items,
                        );
                    }
                }
            }
        }
        items
    }

    #[allow(clippy::too_many_arguments)]
    fn execute(
        &self,
        plan: &CompiledMonsterPlan,
        pool: &CompiledPool,
        kind_modifier: Option<&CompiledModifier>,
        party_modifier: Option<&CompiledModifier>,
        gold_multiplier: f64,
        stack_multiplier: f64,
        rng: &mut impl LootRandom,
        items: &mut Vec<Item>,
    ) {
        let index = if pool.entries.len() > 1 {
            let drawn = rng.below(pool.total_weight);
            pool.cumulative_weights.partition_point(|threshold| *threshold <= drawn)
        } else {
            0
        };
        match &pool.entries[index] {
            CompiledGenerator::Gold => {
                let Some((min_amount, max_amount)) = plan.gold_range else {
                    return;
                };
                let amount = if min_amount == max_amount {
                    min_amount
                } else {
                    rng.range(min_amount, max_amount)
                };
                let amount = scale_amount(amount, gold_multiplier);
                items.push(self.gold_items.construct_item(amount));
            },
            CompiledGenerator::Item { reference, amount } => {
                let amount = scale_amount(u32::from(amount.sample(rng)), stack_multiplier)
                    .min(u32::from(reference.max_stack_size)) as u16;
                items.push(Item {
                    reference,
                    variance: None,
                    type_data: ItemTypeData::Consumable { amount },
                });
            },
            CompiledGenerator::Equipment(candidates) => {
                let Some(reference) = candidates.select(plan.monster_level, rng) else {
                    return;
                };
                let mut upgrade = candidates.sample_upgrade(rng);
                for modifier in [kind_modifier, party_modifier].into_iter().flatten() {
                    upgrade = upgrade.saturating_add(modifier.equipment_upgrade_bonus.sample(rng));
                }
                items.push(Item {
                    reference,
                    variance: None,
                    type_data: ItemTypeData::Equipment { upgrade_level: upgrade },
                });
            },
        }
    }
}

fn scale_amount(amount: u32, multiplier: f64) -> u32 {
    let scaled = (f64::from(amount) * multiplier).floor();
    scaled.clamp(0.0, f64::from(u32::MAX)) as u32
}
