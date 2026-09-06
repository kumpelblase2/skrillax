use super::gold::{
    GOLD_ITEM_LARGE_REF_ID, GOLD_ITEM_MEDIUM_REF_ID, GOLD_ITEM_SMALL_REF_ID, GOLD_TIER_MEDIUM_MAX, GOLD_TIER_SMALL_MAX,
};
use super::*;
use silkroad_data::common::{RefCommon, RefOrigin};
use silkroad_data::datamap::DataEntry;
use silkroad_data::itemdata::{RefBiologicalType, RefItemData, RefItemRarity};
use silkroad_definitions::rarity::{EntityRarity, EntityRarityType};
use silkroad_definitions::TypeId;
use silkroad_game_base::{Item, ItemTypeData};
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

/// Deterministic RNG driven by pre-recorded draws. Panics when asked for
/// an unexpected draw so tests fail loudly instead of silently passing.
#[derive(Default)]
struct ScriptedRandom {
    below_values: VecDeque<u64>,
    probabilities: VecDeque<bool>,
    ranges: VecDeque<u32>,
}

impl ScriptedRandom {
    fn new(below: &[u64], probabilities: &[bool], ranges: &[u32]) -> Self {
        Self {
            below_values: below.iter().copied().collect(),
            probabilities: probabilities.iter().copied().collect(),
            ranges: ranges.iter().copied().collect(),
        }
    }
}

impl LootRandom for ScriptedRandom {
    fn below(&mut self, _bound: u64) -> u64 {
        self.below_values
            .pop_front()
            .expect("scripted random exhausted: unexpected 'below' draw")
    }

    fn probability(&mut self, _probability: f64) -> bool {
        self.probabilities
            .pop_front()
            .expect("scripted random exhausted: unexpected 'probability' draw")
    }

    fn range(&mut self, min: u32, max: u32) -> u32 {
        let value = self
            .ranges
            .pop_front()
            .expect("scripted random exhausted: unexpected 'range' draw");
        assert!(
            (min..=max).contains(&value),
            "scripted range draw {} outside {}..={}",
            value,
            min,
            max
        );
        value
    }
}

fn fake_stackable(ref_id: u32, max_stack: u16) -> &'static RefItemData {
    Box::leak(Box::new(RefItemData {
        common: RefCommon {
            service: true,
            ref_id,
            id: format!("ITEM_TEST_{}", ref_id),
            type_id: TypeId(3, 3, 1, 1),
            country: RefOrigin::Chinese,
            despawn_time: Duration::from_secs(30),
        },
        price: 100,
        rarity: RefItemRarity::General,
        can_drop: true,
        max_stack_size: max_stack,
        range: None,
        required_level: None,
        biological_type: RefBiologicalType::Both,
        params: [0; 4],
    }))
}

fn gold_items() -> GoldItemReferences {
    GoldItemReferences {
        small: fake_stackable(GOLD_ITEM_SMALL_REF_ID, 1),
        medium: fake_stackable(GOLD_ITEM_MEDIUM_REF_ID, 1),
        large: fake_stackable(GOLD_ITEM_LARGE_REF_ID, 1),
    }
}

fn fixed_pool(generator: CompiledGenerator) -> CompiledPool {
    CompiledPool {
        entries: vec![generator],
        cumulative_weights: vec![1],
        total_weight: 1,
    }
}

fn roll(pool: CompiledPool, attempts: u32, chance: f64) -> CompiledProfile {
    CompiledProfile {
        rolls: vec![CompiledRoll { pool, attempts, chance }],
    }
}

fn tables(plan: CompiledMonsterPlan, modifiers: CompiledModifiers, rate: f64) -> LootTables {
    LootTables::new(HashMap::from([(100u32, plan)]), gold_items(), modifiers, rate)
}

fn normal_plan(profiles: Vec<CompiledProfile>) -> CompiledMonsterPlan {
    CompiledMonsterPlan {
        monster_level: 10,
        gold_range: Some((40, 60)),
        profiles,
    }
}

fn normal_rarity() -> EntityRarity {
    EntityRarity::new(false, EntityRarityType::Normal)
}

#[test]
fn weighted_selection_boundaries() {
    // Weights 1 / 3: cumulative thresholds [1, 4].
    let pool = CompiledPool {
        entries: vec![
            CompiledGenerator::Item {
                reference: fake_stackable(50, 10),
                amount: CompiledDistribution::Fixed(1),
            },
            CompiledGenerator::Item {
                reference: fake_stackable(51, 10),
                amount: CompiledDistribution::Fixed(1),
            },
        ],
        cumulative_weights: vec![1, 4],
        total_weight: 4,
    };

    let tables = tables(
        normal_plan(vec![roll(pool.clone(), 1, 1.0)]),
        CompiledModifiers::default(),
        1.0,
    );
    let items = tables.generate(100, normal_rarity(), &mut ScriptedRandom::new(&[0], &[], &[]));
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].reference.ref_id(), 50);

    let items = tables.generate(100, normal_rarity(), &mut ScriptedRandom::new(&[1, 3], &[], &[]));
    assert_eq!(items[0].reference.ref_id(), 51);
}

#[test]
fn chance_success_and_failure_boundary() {
    let tables = tables(
        normal_plan(vec![roll(fixed_pool(CompiledGenerator::Gold), 1, 0.5)]),
        CompiledModifiers::default(),
        1.0,
    );

    let items = tables.generate(100, normal_rarity(), &mut ScriptedRandom::new(&[], &[true], &[40]));
    assert_eq!(items.len(), 1);

    let items = tables.generate(100, normal_rarity(), &mut ScriptedRandom::new(&[], &[false], &[]));
    assert!(items.is_empty());
}

#[test]
fn missing_gold_range_produces_no_gold() {
    let mut plan = normal_plan(vec![roll(fixed_pool(CompiledGenerator::Gold), 1, 1.0)]);
    plan.gold_range = None;
    let tables = tables(plan, CompiledModifiers::default(), 1.0);

    let items = tables.generate(100, normal_rarity(), &mut ScriptedRandom::default());
    assert!(items.is_empty());
}

#[test]
fn global_rate_produces_guaranteed_and_fractional_selections() {
    // Rate 2.5 with guaranteed chance: 2 guaranteed selections plus one
    // fractional selection with probability 0.5 per attempt.
    let tables = tables(
        normal_plan(vec![roll(fixed_pool(CompiledGenerator::Gold), 1, 1.0)]),
        CompiledModifiers::default(),
        2.5,
    );

    let items = tables.generate(
        100,
        normal_rarity(),
        &mut ScriptedRandom::new(&[], &[true], &[40, 40, 40]),
    );
    assert_eq!(items.len(), 3);

    let items = tables.generate(100, normal_rarity(), &mut ScriptedRandom::new(&[], &[false], &[40, 40]));
    assert_eq!(items.len(), 2);
}

#[test]
fn uniform_amount_endpoints_are_inclusive() {
    let tables = tables(
        normal_plan(vec![roll(
            fixed_pool(CompiledGenerator::Item {
                reference: fake_stackable(70, 100),
                amount: CompiledDistribution::Uniform { min: 2, max: 5 },
            }),
            1,
            1.0,
        )]),
        CompiledModifiers::default(),
        1.0,
    );

    let items = tables.generate(100, normal_rarity(), &mut ScriptedRandom::new(&[0], &[], &[]));
    match items[0].type_data {
        ItemTypeData::Consumable { amount } => assert_eq!(amount, 2),
        _ => panic!("expected consumable item"),
    }

    let items = tables.generate(100, normal_rarity(), &mut ScriptedRandom::new(&[3], &[], &[]));
    match items[0].type_data {
        ItemTypeData::Consumable { amount } => assert_eq!(amount, 5),
        _ => panic!("expected consumable item"),
    }
}

#[test]
fn gold_amount_is_sampled_scaled_and_tiered() {
    let mut modifiers = CompiledModifiers::default();
    modifiers.party = Some(CompiledModifier {
        attempts_multiplier: 1,
        chance_multiplier: 1.0,
        gold_amount_multiplier: 2.0,
        stack_amount_multiplier: 1.0,
        equipment_upgrade_bonus: CompiledDistribution::Fixed(0),
    });

    let tables = tables(
        normal_plan(vec![roll(fixed_pool(CompiledGenerator::Gold), 1, 1.0)]),
        modifiers,
        1.0,
    );

    // Draw 45 (range 40..=60), doubled by the party modifier to 90.
    let party_rarity = EntityRarity::new(true, EntityRarityType::Normal);
    let items = tables.generate(100, party_rarity, &mut ScriptedRandom::new(&[], &[], &[45]));
    assert_eq!(items.len(), 1);
    match items[0].type_data {
        ItemTypeData::Gold { amount } => assert_eq!(amount, 90),
        _ => panic!("expected gold item"),
    }
    assert_eq!(items[0].reference.ref_id(), GOLD_ITEM_SMALL_REF_ID);

    // Large amounts use the large gold tier.
    let item = tables.gold_item(9000);
    assert_eq!(item.reference.ref_id(), GOLD_ITEM_LARGE_REF_ID);
    let item = tables.gold_item(GOLD_TIER_SMALL_MAX);
    assert_eq!(item.reference.ref_id(), GOLD_ITEM_MEDIUM_REF_ID);
    let item = tables.gold_item(GOLD_TIER_MEDIUM_MAX);
    assert_eq!(item.reference.ref_id(), GOLD_ITEM_LARGE_REF_ID);
}

#[test]
fn upgrade_bonus_composition_applies_kind_then_party() {
    let mut modifiers = CompiledModifiers::default();
    modifiers.rarity.push((
        EntityRarityType::Champion,
        CompiledModifier {
            attempts_multiplier: 1,
            chance_multiplier: 1.0,
            gold_amount_multiplier: 1.0,
            stack_amount_multiplier: 1.0,
            equipment_upgrade_bonus: CompiledDistribution::Fixed(2),
        },
    ));
    modifiers.party = Some(CompiledModifier {
        attempts_multiplier: 1,
        chance_multiplier: 1.0,
        gold_amount_multiplier: 1.0,
        stack_amount_multiplier: 1.0,
        equipment_upgrade_bonus: CompiledDistribution::Fixed(1),
    });

    let candidates = EquipmentCandidates::new(
        (-10, 25),
        vec![EquipmentLevelBucket {
            level: 8,
            items: vec![fake_equipment(80)],
        }],
        CompiledDistribution::Fixed(3),
    );

    let champion_party = EntityRarity::new(true, EntityRarityType::Champion);
    let tables = tables(
        normal_plan(vec![roll(fixed_pool(CompiledGenerator::Equipment(candidates)), 1, 1.0)]),
        modifiers,
        1.0,
    );

    // Base upgrade 3 + kind bonus 2 + party bonus 1.
    let items = tables.generate(100, champion_party, &mut ScriptedRandom::new(&[0, 0], &[], &[]));
    assert_eq!(items.len(), 1);
    match items[0].type_data {
        ItemTypeData::Equipment { upgrade_level } => assert_eq!(upgrade_level, 6),
        _ => panic!("expected equipment item"),
    }
    assert!(items[0].variance.is_none());
}

fn fake_equipment(ref_id: u32) -> &'static RefItemData {
    let item = fake_stackable(ref_id, 1);
    Box::leak(Box::new(RefItemData {
        common: RefCommon {
            type_id: TypeId(3, 1, 6, 2),
            ..item.common.clone()
        },
        ..item.clone()
    }))
}

#[test]
fn identical_scripted_input_produces_identical_output() {
    let build_tables = || {
        let pool = CompiledPool {
            entries: vec![
                CompiledGenerator::Item {
                    reference: fake_stackable(60, 10),
                    amount: CompiledDistribution::Uniform { min: 1, max: 4 },
                },
                CompiledGenerator::Gold,
            ],
            cumulative_weights: vec![1, 2],
            total_weight: 2,
        };
        tables(normal_plan(vec![roll(pool, 2, 1.0)]), CompiledModifiers::default(), 1.0)
    };

    let script =
        |tables: &LootTables| tables.generate(100, normal_rarity(), &mut ScriptedRandom::new(&[0, 0, 1], &[], &[55]));

    let describe = |items: Vec<Item>| -> Vec<(u32, String)> {
        items
            .iter()
            .map(|item| {
                (
                    item.reference.ref_id(),
                    match item.type_data {
                        ItemTypeData::Consumable { amount } => format!("consumable {}", amount),
                        ItemTypeData::Gold { amount } => format!("gold {}", amount),
                        ItemTypeData::Equipment { upgrade_level } => format!("equipment {}", upgrade_level),
                        ItemTypeData::COS => String::from("cos"),
                    },
                )
            })
            .collect()
    };

    let first = describe(script(&build_tables()));
    let second = describe(script(&build_tables()));
    assert_eq!(first, second);
    assert_eq!(first.len(), 2);
    assert_eq!(first[1].1, "gold 55");
}

#[test]
fn unknown_monster_yields_no_items() {
    let tables = tables(
        normal_plan(vec![roll(fixed_pool(CompiledGenerator::Gold), 1, 1.0)]),
        CompiledModifiers::default(),
        1.0,
    );
    let items = tables.generate(9999, normal_rarity(), &mut ScriptedRandom::new(&[], &[], &[]));
    assert!(items.is_empty());
}

#[test]
fn generation_is_bounded_by_safety_limit() {
    // A configuration that would produce many drops must stop at the limit.
    let pool = CompiledPool {
        entries: vec![
            CompiledGenerator::Item {
                reference: fake_stackable(60, 10),
                amount: CompiledDistribution::Fixed(1),
            },
            CompiledGenerator::Gold,
        ],
        cumulative_weights: vec![1, 2],
        total_weight: 2,
    };
    let tables = tables(
        normal_plan(vec![roll(pool, 200, 1.0)]),
        CompiledModifiers::default(),
        1.0,
    );

    let mut rng = SystemRandom;
    let items = tables.generate(100, normal_rarity(), &mut rng);
    assert_eq!(items.len(), MAX_DROPS_PER_DEATH);
}
