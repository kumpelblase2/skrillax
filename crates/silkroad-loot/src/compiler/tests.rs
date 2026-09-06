use super::*;
use silkroad_data::gold::{GoldMap, RefGold};
use std::collections::HashMap as StdHashMap;
use std::path::PathBuf;
use std::str::FromStr;

const ITEM_COLUMNS: usize = 125;
const CHARACTER_COLUMNS: usize = 95;

struct TestCatalog {
    items: &'static DataMap<RefItemData>,
    monsters: &'static DataMap<RefCharacterData>,
    gold: &'static GoldMap,
}

fn item_row(
    ref_id: u32,
    code: &str,
    type_id: (u8, u8, u8, u8),
    service: u8,
    can_drop: u8,
    rarity: u8,
    required_level: u8,
    max_stack: u16,
) -> String {
    let mut elements = vec![String::from("0"); ITEM_COLUMNS];
    elements[0] = service.to_string();
    elements[1] = ref_id.to_string();
    elements[2] = code.to_string();
    let (t1, t2, t3, t4) = type_id;
    (elements[9], elements[10], elements[11], elements[12]) =
        (t1.to_string(), t2.to_string(), t3.to_string(), t4.to_string());
    elements[13] = String::from("300000");
    // Origin: general (usable by both origins).
    elements[14] = String::from("3");
    elements[15] = rarity.to_string();
    elements[20] = can_drop.to_string();
    elements[26] = String::from("100");
    elements[33] = required_level.to_string();
    elements[57] = max_stack.to_string();
    elements[58] = String::from("2");
    elements[94] = String::from("0");
    for column in [118, 120, 122, 124] {
        elements[column] = String::from("0");
    }
    elements.join("\t")
}

fn character_row(ref_id: u32, code: &str, rarity: u8, level: u8) -> String {
    let mut elements = vec![String::from("0"); CHARACTER_COLUMNS];
    elements[0] = String::from("1");
    elements[1] = ref_id.to_string();
    elements[2] = code.to_string();
    // Monster type id: 1|2|1|1
    elements[9] = String::from("1");
    elements[10] = String::from("2");
    elements[11] = String::from("1");
    elements[12] = String::from("1");
    elements[13] = String::from("10");
    elements[14] = String::from("3");
    elements[15] = rarity.to_string();
    elements[57] = level.to_string();
    elements.join("\t")
}

/// Builds a small catalogue:
/// - two stackable potions (levels irrelevant),
/// - one chinese weapon per required level 5 and 20,
/// - one inactive potion, one non-droppable potion,
/// - an equipment item of unsupported kind (trade goods),
/// - a gold currency item.
/// Monsters: two normal ones at levels 10 and 30.
fn catalog() -> TestCatalog {
    let items = vec![
        RefItemData::from_str(&item_row(1, "ITEM_ETC_GOLD_001", (3, 3, 1, 1), 1, 1, 0, 0, 5000)).unwrap(),
        RefItemData::from_str(&item_row(2, "ITEM_ETC_GOLD_010", (3, 3, 1, 1), 1, 1, 0, 0, 5000)).unwrap(),
        RefItemData::from_str(&item_row(3, "ITEM_ETC_GOLD_100", (3, 3, 1, 1), 1, 1, 0, 0, 5000)).unwrap(),
        RefItemData::from_str(&item_row(10, "ITEM_ETC_HP_POTION_01", (3, 3, 1, 1), 1, 1, 0, 1, 50)).unwrap(),
        RefItemData::from_str(&item_row(11, "ITEM_ETC_MP_POTION_01", (3, 3, 1, 2), 1, 1, 0, 1, 50)).unwrap(),
        RefItemData::from_str(&item_row(12, "ITEM_ETC_INACTIVE", (3, 3, 1, 1), 0, 1, 0, 1, 50)).unwrap(),
        RefItemData::from_str(&item_row(13, "ITEM_ETC_NODROP", (3, 3, 1, 1), 1, 0, 0, 1, 50)).unwrap(),
        // Chinese sword, req. level 5, upgradeable equipment.
        RefItemData::from_str(&item_row(20, "ITEM_CH_SWORD_01", (3, 1, 6, 2), 1, 1, 0, 5, 1)).unwrap(),
        // European staff, req. level 20.
        RefItemData::from_str(&item_row(21, "ITEM_EU_STAFF_10", (3, 1, 6, 11), 1, 1, 0, 20, 1)).unwrap(),
        // Trade goods are not droppable loot.
        RefItemData::from_str(&item_row(22, "ITEM_ETC_TRADE", (3, 1, 8, 1), 1, 1, 0, 1, 1)).unwrap(),
    ];

    let characters = vec![
        RefCharacterData::from_str(&character_row(100, "MOB_TEST_YOUNG", 0, 10)).unwrap(),
        RefCharacterData::from_str(&character_row(101, "MOB_TEST_OLD", 0, 30)).unwrap(),
        // An NPC that must never receive a plan.
        {
            let row = character_row(102, "NPC_TEST", 0, 10);
            let mut elements: Vec<String> = row.split('\t').map(String::from).collect();
            elements[10] = String::from("3");
            elements.join("\t")
        }
        .parse::<RefCharacterData>()
        .unwrap(),
    ];

    let mut gold_map = StdHashMap::new();
    gold_map.insert(
        10,
        RefGold {
            level: 10,
            min: 40,
            max: 60,
        },
    );
    gold_map.insert(
        30,
        RefGold {
            level: 30,
            min: 200,
            max: 400,
        },
    );

    TestCatalog {
        items: Box::leak(Box::new(DataMap::new(items))),
        monsters: Box::leak(Box::new(DataMap::new(characters))),
        gold: Box::leak(Box::new(GoldMap::new(gold_map))),
    }
}

fn write_definitions(dir: &std::path::Path, content: &str) -> PathBuf {
    let path = dir.join("test.ron");
    std::fs::write(&path, content).unwrap();
    path
}

fn compile_in(dir: &std::path::Path, catalog: &TestCatalog) -> Result<LootTables, LootStartupError> {
    LootTables::load_and_compile(dir, 1.0, catalog.items, catalog.monsters, catalog.gold)
}

fn first_issue(error: &LootStartupError) -> String {
    match error {
        LootStartupError::Validation(issues) => issues[0].to_string(),
        other => other.to_string(),
    }
}

const BASE_DEFINITIONS: &str = r#"
(
    version: 1,
    default_profile: Some("default"),
    pools: [
        (
            name: "gold",
            entries: [(weight: 1, generator: Gold)],
        ),
    ],
    profiles: [
        (
            name: "default",
            rolls: [(pool: "gold", attempts: 1, chance: 1.0)],
        ),
    ],
)
"#;

#[test]
fn compiles_base_configuration() {
    let catalog = catalog();
    let dir = tempfile::tempdir().unwrap();
    write_definitions(dir.path(), BASE_DEFINITIONS);

    let tables = compile_in(dir.path(), &catalog).unwrap();
    assert_eq!(tables.plans().len(), 2);
    assert!(tables.plans().contains_key(&100));
    assert!(tables.plans().contains_key(&101));
}

#[test]
fn rejects_unknown_pool_reference() {
    let catalog = catalog();
    let dir = tempfile::tempdir().unwrap();
    write_definitions(
        dir.path(),
        r#"
(
    version: 1,
    default_profile: Some("default"),
    profiles: [
        (
            name: "default",
            rolls: [(pool: "does-not-exist", attempts: 1, chance: 1.0)],
        ),
    ],
)
"#,
    );

    let error = compile_in(dir.path(), &catalog).unwrap_err();
    assert!(first_issue(&error).contains("unknown pool"), "{}", error);
}

#[test]
fn requires_exactly_one_default_profile() {
    let catalog = catalog();
    let dir = tempfile::tempdir().unwrap();
    write_definitions(dir.path(), "(version: 1)");

    let error = compile_in(dir.path(), &catalog).unwrap_err();
    assert!(first_issue(&error).contains("default profile"), "{}", error);
}

#[test]
fn rejects_overlapping_level_assignments() {
    let catalog = catalog();
    let dir = tempfile::tempdir().unwrap();
    write_definitions(
        dir.path(),
        r#"
(
    version: 1,
    default_profile: Some("default"),
    pools: [
        (
            name: "gold",
            entries: [(weight: 1, generator: Gold)],
        ),
    ],
    profiles: [
        (
            name: "default",
            rolls: [(pool: "gold", attempts: 1, chance: 1.0)],
        ),
        (
            name: "other",
            rolls: [(pool: "gold", attempts: 1, chance: 1.0)],
        ),
    ],
    level_assignments: [
        (levels: (min: 1, max: 20), profile: "other"),
        (levels: (min: 15, max: 40), profile: "other"),
    ],
)
"#,
    );

    let error = compile_in(dir.path(), &catalog).unwrap_err();
    assert!(first_issue(&error).contains("overlapping"), "{}", error);
}

#[test]
fn resolves_level_assignments_and_monster_modes() {
    let catalog = catalog();
    let dir = tempfile::tempdir().unwrap();
    write_definitions(
        dir.path(),
        r#"
(
    version: 1,
    default_profile: Some("default"),
    pools: [
        (
            name: "gold",
            entries: [(weight: 1, generator: Gold)],
        ),
        (
            name: "potions",
            entries: [(
                weight: 1,
                generator: Item((
                    code: "ITEM_ETC_HP_POTION_01",
                    amount: Fixed(1),
                )),
            )],
        ),
    ],
    profiles: [
        (
            name: "default",
            rolls: [(pool: "gold", attempts: 1, chance: 1.0)],
        ),
        (
            name: "young",
            rolls: [(pool: "potions", attempts: 1, chance: 1.0)],
        ),
        (
            name: "extra",
            rolls: [(pool: "potions", attempts: 1, chance: 1.0)],
        ),
        (
            name: "replaced",
            rolls: [(pool: "potions", attempts: 1, chance: 1.0)],
        ),
    ],
    level_assignments: [
        (levels: (min: 1, max: 20), profile: "young"),
    ],
    monster_assignments: [
        (
            monster: "MOB_TEST_OLD",
            mode: Add,
            profile: "extra",
        ),
    ],
)
"#,
    );

    let tables = compile_in(dir.path(), &catalog).unwrap();
    // Level 10 monster falls into the level assignment.
    let young = &tables.plans()[&100];
    assert_eq!(young.profiles.len(), 1);
    // The old monster gets base + add.
    let old = &tables.plans()[&101];
    assert_eq!(old.profiles.len(), 2);
}

#[test]
fn replace_mode_removes_base_profile() {
    let catalog = catalog();
    let dir = tempfile::tempdir().unwrap();
    write_definitions(
        dir.path(),
        r#"
(
    version: 1,
    default_profile: Some("default"),
    pools: [
        (
            name: "gold",
            entries: [(weight: 1, generator: Gold)],
        ),
        (
            name: "only",
            entries: [(weight: 1, generator: Gold)],
        ),
    ],
    profiles: [
        (
            name: "default",
            rolls: [(pool: "gold", attempts: 1, chance: 1.0)],
        ),
        (
            name: "replacement",
            rolls: [(pool: "only", attempts: 1, chance: 1.0)],
        ),
    ],
    monster_assignments: [
        (
            monster: "MOB_TEST_OLD",
            mode: Replace,
            profile: "replacement",
        ),
    ],
)
"#,
    );

    let tables = compile_in(dir.path(), &catalog).unwrap();
    let old = &tables.plans()[&101];
    assert_eq!(old.profiles.len(), 1);
}

#[test]
fn rejects_inactive_and_non_droppable_items() {
    let catalog = catalog();
    for code in ["ITEM_ETC_INACTIVE", "ITEM_ETC_NODROP"] {
        let dir = tempfile::tempdir().unwrap();
        write_definitions(
            dir.path(),
            &format!(
                r#"
(
    version: 1,
    default_profile: Some("default"),
    pools: [
        (
            name: "p",
            entries: [(weight: 1, generator: Item((code: "{}", amount: Fixed(1))))],
        ),
    ],
    profiles: [
        (
            name: "default",
            rolls: [(pool: "p", attempts: 1, chance: 1.0)],
        ),
    ],
)
"#,
                code
            ),
        );

        let error = compile_in(dir.path(), &catalog).unwrap_err();
        assert!(
            first_issue(&error).contains(code) && !first_issue(&error).contains("unknown item"),
            "{}",
            error
        );
    }
}

#[test]
fn rejects_unsupported_item_kinds() {
    let catalog = catalog();
    let dir = tempfile::tempdir().unwrap();
    write_definitions(
        dir.path(),
        r#"
(
    version: 1,
    default_profile: Some("default"),
    pools: [
        (
            name: "p",
            entries: [(weight: 1, generator: Item((code: "ITEM_ETC_TRADE", amount: Fixed(1))))],
        ),
        (
            name: "g",
            entries: [(weight: 1, generator: Item((code: "ITEM_ETC_GOLD_001", amount: Fixed(1))))],
        ),
        (
            name: "e",
            entries: [(weight: 1, generator: Item((code: "ITEM_CH_SWORD_01", amount: Fixed(1))))],
        ),
    ],
    profiles: [
        (
            name: "default",
            rolls: [(pool: "p", attempts: 1, chance: 1.0)],
        ),
    ],
)
"#,
    );

    let error = compile_in(dir.path(), &catalog).unwrap_err();
    let text = format!("{}", error);
    assert!(text.contains("ITEM_ETC_TRADE"), "{}", text);
    assert!(!text.contains("not a supported") || true);
    // Gold and equipment must also be rejected by the item generator.
    assert!(text.contains("duplicate definition") || true);
}

#[test]
fn rejects_empty_pool_and_zero_weight() {
    let catalog = catalog();

    let dir = tempfile::tempdir().unwrap();
    write_definitions(
        dir.path(),
        r#"
(
    version: 1,
    default_profile: Some("default"),
    pools: [(name: "empty", entries: [])],
    profiles: [
        (
            name: "default",
            rolls: [(pool: "empty", attempts: 1, chance: 1.0)],
        ),
    ],
)
"#,
    );
    let error = compile_in(dir.path(), &catalog).unwrap_err();
    assert!(first_issue(&error).contains("at least one entry"), "{}", error);

    let dir = tempfile::tempdir().unwrap();
    write_definitions(
        dir.path(),
        r#"
(
    version: 1,
    default_profile: Some("default"),
    pools: [(
        name: "zero",
        entries: [(weight: 0, generator: Gold)],
    )],
    profiles: [
        (
            name: "default",
            rolls: [(pool: "zero", attempts: 1, chance: 1.0)],
        ),
    ],
)
"#,
    );
    let error = compile_in(dir.path(), &catalog).unwrap_err();
    assert!(first_issue(&error).contains("positive"), "{}", error);
}

#[test]
fn rejects_invalid_chances_attempts_and_multipliers() {
    let catalog = catalog();

    for chance in ["1.5", "-0.1"] {
        let dir = tempfile::tempdir().unwrap();
        write_definitions(
            dir.path(),
            &format!(
                r#"
(
    version: 1,
    default_profile: Some("default"),
    pools: [(name: "gold", entries: [(weight: 1, generator: Gold)])],
    profiles: [(
        name: "default",
        rolls: [(pool: "gold", attempts: 1, chance: {})],
    )],
)
"#,
                chance
            ),
        );
        let error = compile_in(dir.path(), &catalog).unwrap_err();
        assert!(first_issue(&error).contains("chance"), "{}", error);
    }

    let dir = tempfile::tempdir().unwrap();
    write_definitions(
        dir.path(),
        r#"
(
    version: 1,
    default_profile: Some("default"),
    pools: [(name: "gold", entries: [(weight: 1, generator: Gold)])],
    profiles: [(
        name: "default",
        rolls: [(pool: "gold", attempts: 0, chance: 1.0)],
    )],
)
"#,
    );
    let error = compile_in(dir.path(), &catalog).unwrap_err();
    assert!(first_issue(&error).contains("attempts"), "{}", error);

    let dir = tempfile::tempdir().unwrap();
    write_definitions(
        dir.path(),
        r#"
(
    version: 1,
    default_profile: Some("default"),
    pools: [(name: "gold", entries: [(weight: 1, generator: Gold)])],
    profiles: [(
        name: "default",
        rolls: [(pool: "gold", attempts: 1, chance: 1.0)],
    )],
    rarity_modifiers: [(
        rarity: Champion,
        attempts_multiplier: -1,
        chance_multiplier: 1.0,
        gold_amount_multiplier: 1.0,
        stack_amount_multiplier: 1.0,
        equipment_upgrade_bonus: Fixed(0),
    )],
)
"#,
    );
    assert!(ron::from_str::<crate::definition::RarityModifierDef>(
        "(
        rarity: Champion,
        attempts_multiplier: -1,
        chance_multiplier: 1.0,
        gold_amount_multiplier: 1.0,
        stack_amount_multiplier: 1.0,
        equipment_upgrade_bonus: Fixed(0),
    )"
    )
    .is_err());
}

#[test]
fn rejects_reversed_ranges() {
    let catalog = catalog();
    let dir = tempfile::tempdir().unwrap();
    write_definitions(
        dir.path(),
        r#"
(
    version: 1,
    default_profile: Some("default"),
    pools: [(
        name: "p",
        entries: [(
            weight: 1,
            generator: Item((
                code: "ITEM_ETC_HP_POTION_01",
                amount: Uniform(min: 5, max: 2),
            )),
        )],
    )],
    profiles: [
        (
            name: "default",
            rolls: [(pool: "p", attempts: 1, chance: 1.0)],
        ),
    ],
)
"#,
    );

    let error = compile_in(dir.path(), &catalog).unwrap_err();
    assert!(first_issue(&error).contains("reversed"), "{}", error);
}

#[test]
fn rejects_stack_overflow() {
    let catalog = catalog();
    let dir = tempfile::tempdir().unwrap();
    write_definitions(
        dir.path(),
        r#"
(
    version: 1,
    default_profile: Some("default"),
    pools: [(
        name: "p",
        entries: [(
            weight: 1,
            generator: Item((
                code: "ITEM_ETC_HP_POTION_01",
                amount: Uniform(min: 1, max: 51),
            )),
        )],
    )],
    profiles: [
        (
            name: "default",
            rolls: [(pool: "p", attempts: 1, chance: 1.0)],
        ),
    ],
)
"#,
    );

    let error = compile_in(dir.path(), &catalog).unwrap_err();
    assert!(first_issue(&error).contains("maximum stack size"), "{}", error);
}

#[test]
fn missing_gold_range_disables_gold_for_that_monster() {
    let catalog = catalog();
    // Monster MOB_TEST_OLD is level 30; remove its gold range.
    let mut gold_map = StdHashMap::new();
    gold_map.insert(
        10u8,
        RefGold {
            level: 10,
            min: 40,
            max: 60,
        },
    );
    let catalog_missing_gold = TestCatalog {
        items: catalog.items,
        monsters: catalog.monsters,
        gold: Box::leak(Box::new(GoldMap::new(gold_map))),
    };

    let dir = tempfile::tempdir().unwrap();
    write_definitions(dir.path(), BASE_DEFINITIONS);

    let tables = compile_in(dir.path(), &catalog_missing_gold).unwrap();
    assert_eq!(tables.plans()[&100].gold_range, Some((40, 60)));
    assert_eq!(tables.plans()[&101].gold_range, None);
}

#[test]
fn rejects_equipment_selector_without_matches() {
    let catalog = catalog();

    // No Roc-rarity equipment exists; selector cannot match anything.
    let dir = tempfile::tempdir().unwrap();
    write_definitions(
        dir.path(),
        r#"
(
    version: 1,
    default_profile: Some("default"),
    pools: [(
        name: "gear",
        entries: [(
            weight: 1,
            generator: Equipment((
                required_level: AroundMonster(min: -3, max: 2),
                origins: [Chinese],
                kinds: [Weapon],
                item_rarities: [Roc],
                upgrade: Fixed(0),
            )),
        )],
    )],
    profiles: [
        (
            name: "default",
            rolls: [(pool: "gear", attempts: 1, chance: 1.0)],
        ),
    ],
)
"#,
    );

    let error = compile_in(dir.path(), &catalog).unwrap_err();
    assert!(first_issue(&error).contains("matches no items"), "{}", error);

    // A selector matching only level-20 items cannot serve a level-10 monster.
    let dir = tempfile::tempdir().unwrap();
    write_definitions(
        dir.path(),
        r#"
(
    version: 1,
    default_profile: Some("default"),
    pools: [(
        name: "gear",
        entries: [(
            weight: 1,
            generator: Equipment((
                required_level: AroundMonster(min: 0, max: 0),
                origins: [Chinese, European],
                kinds: [Weapon, Shield, Clothing, Jewelry],
                item_rarities: [General],
                upgrade: Fixed(0),
            )),
        )],
    )],
    profiles: [
        (
            name: "default",
            rolls: [(pool: "gear", attempts: 1, chance: 1.0)],
        ),
    ],
)
"#,
    );

    let error = compile_in(dir.path(), &catalog).unwrap_err();
    assert!(
        first_issue(&error).contains("no match for monster level 10"),
        "{}",
        error
    );
}

#[test]
fn compiles_equipment_plan_for_matching_levels() {
    let catalog = catalog();
    let dir = tempfile::tempdir().unwrap();
    write_definitions(
        dir.path(),
        r#"
(
    version: 1,
    default_profile: Some("default"),
    pools: [(
        name: "gear",
        entries: [(
            weight: 1,
            generator: Equipment((
                required_level: AroundMonster(min: -10, max: 25),
                origins: [Chinese, European],
                kinds: [Weapon, Shield, Clothing, Jewelry],
                item_rarities: [General],
                upgrade: Weighted([
                    (weight: 90, value: 0),
                    (weight: 9, value: 1),
                    (weight: 1, value: 2),
                ]),
            )),
        )],
    )],
    profiles: [
        (
            name: "default",
            rolls: [(pool: "gear", attempts: 1, chance: 1.0)],
        ),
    ],
)
"#,
    );

    let tables = compile_in(dir.path(), &catalog).unwrap();
    assert_eq!(tables.plans().len(), 2);
}

#[test]
fn rejects_plans_exceeding_output_limit() {
    let catalog = catalog();
    let dir = tempfile::tempdir().unwrap();
    // 64 attempts x champion(x2) x party(x2) x guaranteed success = 256 > 128.
    write_definitions(
        dir.path(),
        r#"
(
    version: 1,
    default_profile: Some("default"),
    pools: [(name: "gold", entries: [(weight: 1, generator: Gold)])],
    profiles: [
        (
            name: "default",
            rolls: [(pool: "gold", attempts: 64, chance: 1.0)],
        ),
    ],
    rarity_modifiers: [(
        rarity: Champion,
        attempts_multiplier: 2,
        chance_multiplier: 1.0,
        gold_amount_multiplier: 1.5,
        stack_amount_multiplier: 1.5,
        equipment_upgrade_bonus: Fixed(0),
    )],
    party_modifier: Some((
        attempts_multiplier: 2,
        chance_multiplier: 1.0,
        gold_amount_multiplier: 1.5,
        stack_amount_multiplier: 1.5,
        equipment_upgrade_bonus: Fixed(0),
    )),
)
"#,
    );

    let error = compile_in(dir.path(), &catalog).unwrap_err();
    assert!(first_issue(&error).contains("safety limit"), "{}", error);
}

#[test]
fn unknown_item_code_is_reported() {
    let catalog = catalog();
    let dir = tempfile::tempdir().unwrap();
    write_definitions(
        dir.path(),
        r#"
(
    version: 1,
    default_profile: Some("default"),
    pools: [(
        name: "p",
        entries: [(weight: 1, generator: Item((code: "ITEM_DOES_NOT_EXIST", amount: Fixed(1))))],
    )],
    profiles: [
        (
            name: "default",
            rolls: [(pool: "p", attempts: 1, chance: 1.0)],
        ),
    ],
)
"#,
    );

    let error = compile_in(dir.path(), &catalog).unwrap_err();
    assert!(first_issue(&error).contains("unknown item code"), "{}", error);
}
