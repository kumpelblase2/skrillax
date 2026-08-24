use super::LootRandom;
use silkroad_data::itemdata::RefItemData;
use silkroad_definitions::rarity::EntityRarityType;

/// Pre-resolved execution plan for a single monster species.
pub(crate) struct CompiledMonsterPlan {
    pub(crate) monster_level: u8,
    /// Resolved gold amount range for the monster's level, when any executed
    /// profile can generate gold.
    pub(crate) gold_range: Option<(u32, u32)>,
    /// Executed in order; either the base profile followed by an added
    /// profile, or a lone replacing profile.
    pub(crate) profiles: Vec<CompiledProfile>,
}

#[derive(Clone)]
pub(crate) struct CompiledProfile {
    pub(crate) rolls: Vec<CompiledRoll>,
}

#[derive(Clone)]
pub(crate) struct CompiledRoll {
    pub(crate) pool: CompiledPool,
    pub(crate) attempts: u32,
    pub(crate) chance: f64,
}

#[derive(Clone)]
pub(crate) struct CompiledPool {
    pub(crate) entries: Vec<CompiledGenerator>,
    /// Cumulative entry weights for weighted selection.
    pub(crate) cumulative_weights: Vec<u64>,
    pub(crate) total_weight: u64,
}

#[derive(Clone)]
pub(crate) enum CompiledGenerator {
    /// Generates a gold amount from the owning plan's level-dependent range.
    Gold,
    Item {
        reference: &'static RefItemData,
        amount: CompiledDistribution<u16>,
    },
    Equipment(EquipmentCandidates),
}

/// Equipment items grouped by their required level, restricted to a single
/// selector. Selection walks applicable level buckets proportionally to
/// their size.
#[derive(Clone)]
pub(crate) struct EquipmentCandidates {
    /// Inclusive required-level window relative to the killed monster.
    window: (i16, i16),
    /// Buckets sorted ascending by required level.
    by_level: Vec<EquipmentLevelBucket>,
    upgrade: CompiledDistribution<u8>,
}

#[derive(Clone)]
pub(crate) struct EquipmentLevelBucket {
    pub(crate) level: u8,
    pub(crate) items: Vec<&'static RefItemData>,
}

#[derive(Clone)]
pub(crate) enum CompiledDistribution<T: Copy> {
    Fixed(T),
    Uniform { min: T, max: T },
    Weighted { cumulative: Vec<(u64, T)>, total: u64 },
}

impl<T> CompiledDistribution<T>
where
    T: Copy + Ord + Into<u64> + TryFrom<u64>,
    <T as TryFrom<u64>>::Error: std::fmt::Debug,
{
    pub(crate) fn sample(&self, rng: &mut impl LootRandom) -> T {
        match self {
            CompiledDistribution::Fixed(value) => *value,
            CompiledDistribution::Uniform { min, max } => {
                let (min, max) = (*min, *max);
                // T is a small integer type in practice; sample through u64.
                let drawn = rng.below(max.into() - min.into() + 1);
                let value = min.into() + drawn;
                T::try_from(value).expect("sampled value within distribution bounds")
            },
            CompiledDistribution::Weighted { cumulative, total } => {
                let drawn = rng.below(*total);
                let index = cumulative.partition_point(|(threshold, _)| *threshold <= drawn);
                cumulative[index].1
            },
        }
    }
}

#[derive(Clone)]
pub(crate) struct CompiledModifier {
    pub(crate) attempts_multiplier: u32,
    pub(crate) chance_multiplier: f64,
    pub(crate) gold_amount_multiplier: f64,
    pub(crate) stack_amount_multiplier: f64,
    pub(crate) equipment_upgrade_bonus: CompiledDistribution<u8>,
}

#[derive(Default)]
pub(crate) struct CompiledModifiers {
    pub(crate) rarity: Vec<(EntityRarityType, CompiledModifier)>,
    pub(crate) party: Option<CompiledModifier>,
}

impl CompiledModifiers {
    pub(crate) fn for_kind(&self, kind: EntityRarityType) -> Option<&CompiledModifier> {
        self.rarity
            .iter()
            .find(|(candidate, _)| *candidate == kind)
            .map(|(_, modifier)| modifier)
    }

    pub(crate) fn party(&self) -> Option<&CompiledModifier> {
        self.party.as_ref()
    }
}

/// Upper bounds of the combined modifier effects any single death could
/// observe, used for the output-limit validation.
#[derive(Clone, Copy)]
pub(crate) struct WorstCaseFactors {
    pub(crate) attempts_multiplier: u64,
    pub(crate) chance_multiplier: f64,
    pub(crate) stack_amount_multiplier: f64,
}

impl WorstCaseFactors {
    pub(crate) fn from_modifiers(modifiers: &CompiledModifiers) -> Self {
        let mut worst = Self {
            attempts_multiplier: 1,
            chance_multiplier: 1.0,
            stack_amount_multiplier: 1.0,
        };

        let mut apply = |modifier: &CompiledModifier| {
            worst.attempts_multiplier *= u64::from(modifier.attempts_multiplier);
            worst.chance_multiplier *= modifier.chance_multiplier;
            worst.stack_amount_multiplier *= modifier.stack_amount_multiplier;
        };

        if let Some(kind_modifier) = modifiers.rarity.iter().map(|(_, modifier)| modifier).max_by(|a, b| {
            a.attempts_multiplier
                .cmp(&b.attempts_multiplier)
                .then(a.chance_multiplier.total_cmp(&b.chance_multiplier))
        }) {
            apply(kind_modifier);
        }

        if let Some(party) = modifiers.party() {
            apply(party);
        }

        worst
    }
}

impl EquipmentCandidates {
    pub(crate) fn new(
        window: (i16, i16),
        mut by_level: Vec<EquipmentLevelBucket>,
        upgrade: CompiledDistribution<u8>,
    ) -> Self {
        by_level.sort_by_key(|bucket| bucket.level);
        Self {
            window,
            by_level,
            upgrade,
        }
    }

    /// Whether any candidate's required level lies within the window for the
    /// given monster level. Used during compilation validation.
    pub(crate) fn has_match(&self, monster_level: u8) -> bool {
        let min_level = (i16::from(monster_level) + self.window.0).clamp(0, i16::from(u8::MAX)) as u8;
        let max_level = (i16::from(monster_level) + self.window.1).clamp(0, i16::from(u8::MAX)) as u8;
        self.by_level
            .iter()
            .any(|bucket| bucket.level >= min_level && bucket.level <= max_level)
    }

    pub(crate) fn sample_upgrade(&self, rng: &mut impl LootRandom) -> u8 {
        self.upgrade.sample(rng)
    }

    /// Selects an equipment item whose required level lies within the
    /// selector window relative to the killed monster's level. Buckets are
    /// weighted proportionally to their item count.
    pub(crate) fn select(&self, monster_level: u8, rng: &mut impl LootRandom) -> Option<&'static RefItemData> {
        let min_level = (i16::from(monster_level) + self.window.0).clamp(0, i16::from(u8::MAX)) as u8;
        let max_level = (i16::from(monster_level) + self.window.1).clamp(0, i16::from(u8::MAX)) as u8;
        let total: u64 = self
            .by_level
            .iter()
            .filter(|bucket| bucket.level >= min_level && bucket.level <= max_level)
            .map(|bucket| bucket.items.len() as u64)
            .sum();
        if total == 0 {
            return None;
        }

        let mut remaining = rng.below(total);
        for bucket in &self.by_level {
            if bucket.level < min_level || bucket.level > max_level {
                continue;
            }
            let count = bucket.items.len() as u64;
            if remaining < count {
                let index = rng.below(count) as usize;
                return Some(bucket.items[index]);
            }
            remaining -= count;
        }
        None
    }
}
