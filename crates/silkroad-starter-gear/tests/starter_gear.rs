use silkroad_data::itemdata::RefItemData;
use silkroad_data::DataMap;
use silkroad_definitions::type_id::{ObjectClothingType, ObjectRace, ObjectWeaponType};
use silkroad_starter_gear::{StarterContext, StarterGear, StarterSelection};
use std::str::FromStr;

const ITEM_COLUMNS: usize = 125;

fn item_row(ref_id: u32, code: &str, type_id: (u8, u8, u8, u8), origin: u8, max_stack: u16) -> String {
    let mut columns = vec![String::from("0"); ITEM_COLUMNS];
    columns[0] = String::from("1");
    columns[1] = ref_id.to_string();
    columns[2] = code.to_string();
    let (t1, t2, t3, t4) = type_id;
    (columns[9], columns[10], columns[11], columns[12]) =
        (t1.to_string(), t2.to_string(), t3.to_string(), t4.to_string());
    columns[13] = String::from("300000");
    columns[14] = origin.to_string();
    columns[15] = String::from("0");
    columns[21] = String::from("1");
    columns[26] = String::from("1");
    columns[33] = String::from("1");
    columns[57] = max_stack.to_string();
    columns[58] = String::from("2");
    columns[94] = String::from("0");
    for index in [118, 120, 122, 124] {
        columns[index] = String::from("0");
    }
    columns.join("\t")
}

fn item_row_with_rarity(
    ref_id: u32,
    code: &str,
    type_id: (u8, u8, u8, u8),
    origin: u8,
    max_stack: u16,
    rarity: u8,
) -> String {
    let row = item_row(ref_id, code, type_id, origin, max_stack);
    let mut columns = row.split('\t').map(String::from).collect::<Vec<_>>();
    columns[15] = rarity.to_string();
    columns.join("\t")
}

fn item_row_with_level(
    ref_id: u32,
    code: &str,
    type_id: (u8, u8, u8, u8),
    origin: u8,
    max_stack: u16,
    required_level: u8,
) -> String {
    let row = item_row(ref_id, code, type_id, origin, max_stack);
    let mut columns = row.split('\t').map(String::from).collect::<Vec<_>>();
    columns[33] = required_level.to_string();
    columns.join("\t")
}

fn catalogue() -> &'static DataMap<RefItemData> {
    let items = vec![
        RefItemData::from_str(&item_row(10, "ITEM_ALL", (3, 3, 1, 1), 3, 50)).unwrap(),
        RefItemData::from_str(&item_row(11, "ITEM_EU", (3, 3, 1, 2), 1, 50)).unwrap(),
        RefItemData::from_str(&item_row(12, "ITEM_ROBE", (3, 3, 1, 3), 1, 50)).unwrap(),
        RefItemData::from_str(&item_row(13, "ITEM_STAFF", (3, 3, 1, 4), 1, 50)).unwrap(),
        RefItemData::from_str(&item_row(20, "ITEM_EU_ROBE_SHOULDER", (3, 1, 9, 2), 1, 1)).unwrap(),
        RefItemData::from_str(&item_row(21, "ITEM_EU_ROBE_LEG", (3, 1, 9, 4), 1, 1)).unwrap(),
        RefItemData::from_str(&item_row(22, "ITEM_EU_ROBE_FOOT", (3, 1, 9, 6), 1, 1)).unwrap(),
        RefItemData::from_str(&item_row(23, "ITEM_EU_STAFF", (3, 1, 6, 11), 1, 1)).unwrap(),
        RefItemData::from_str(&item_row(24, "ITEM_EU_BOLTS", (3, 3, 4, 2), 1, 500)).unwrap(),
        RefItemData::from_str(&item_row_with_level(25, "ITEM_HIGH_LEVEL", (3, 3, 1, 1), 3, 50, 2)).unwrap(),
        RefItemData::from_str(&item_row(26, "ITEM_HUGE_STACK", (3, 3, 1, 1), 3, 50_000)).unwrap(),
        RefItemData::from_str(&item_row(27, "ITEM_EU_ROBE_BODY", (3, 1, 9, 3), 1, 1)).unwrap(),
        RefItemData::from_str(&item_row(28, "ITEM_EU_ROBE_ARM", (3, 1, 9, 5), 1, 1)).unwrap(),
        RefItemData::from_str(&item_row_with_rarity(29, "ITEM_EU_RARE_STAFF", (3, 1, 6, 11), 1, 1, 2)).unwrap(),
        RefItemData::from_str(&item_row(30, "ITEM_EU_SHIELD", (3, 1, 4, 2), 1, 1)).unwrap(),
    ];
    Box::leak(Box::new(DataMap::new(items)))
}

#[test]
fn shipped_configuration_compiles_without_catalogue_specific_grants() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/starter-gear");
    let items = Box::leak(Box::new(DataMap::new(Vec::new())));

    StarterGear::load_and_compile(&directory, items).unwrap();
}

#[test]
fn character_receives_every_matching_starter_layer() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("base.ron"),
        r#"
(
    version: 1,
    sets: [
        (name: "all", applies_to: All, grants: [(code: "ITEM_ALL", amount: 2)]),
        (name: "eu", applies_to: Race(European), grants: [(code: "ITEM_EU")]),
        (name: "robe", applies_to: Clothing(Robe), grants: [(code: "ITEM_ROBE")]),
        (name: "staff", applies_to: Weapon(Staff), grants: [(code: "ITEM_STAFF")]),
    ],
)
"#,
    )
    .unwrap();

    let gear = StarterGear::load_and_compile(directory.path(), catalogue()).unwrap();
    let items = gear
        .resolve(StarterContext {
            race: ObjectRace::European,
            clothing: ObjectClothingType::Robe,
            weapon: ObjectWeaponType::Staff,
        })
        .unwrap();

    assert_eq!(
        items
            .iter()
            .map(|item| (item.reference_id, item.amount, item.slot))
            .collect::<Vec<_>>(),
        vec![(10, 2, 13), (11, 1, 14), (12, 1, 15), (13, 1, 16)]
    );
}

#[test]
fn matching_stackable_grants_are_combined_and_split_at_the_catalogue_limit() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("base.ron"),
        r#"
(
    version: 1,
    sets: [
        (name: "all", applies_to: All, grants: [(code: "ITEM_ALL", amount: 40)]),
        (name: "eu", applies_to: Race(European), grants: [(code: "ITEM_ALL", amount: 40)]),
    ],
)
"#,
    )
    .unwrap();

    let gear = StarterGear::load_and_compile(directory.path(), catalogue()).unwrap();
    let items = gear
        .resolve(StarterContext {
            race: ObjectRace::European,
            clothing: ObjectClothingType::Robe,
            weapon: ObjectWeaponType::Staff,
        })
        .unwrap();

    assert_eq!(
        items
            .iter()
            .map(|item| (item.reference_id, item.amount, item.slot))
            .collect::<Vec<_>>(),
        vec![(10, 50, 13), (10, 30, 14)]
    );
}

#[test]
fn rejects_a_grant_larger_than_the_catalogue_stack_limit() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("base.ron"),
        r#"
(
    version: 1,
    sets: [
        (name: "too-many", applies_to: All, grants: [(code: "ITEM_ALL", amount: 51)]),
    ],
)
"#,
    )
    .unwrap();

    let error = match StarterGear::load_and_compile(directory.path(), catalogue()) {
        Err(error) => error,
        Ok(_) => panic!("oversized starter stack unexpectedly compiled"),
    };
    assert!(error.to_string().contains("maximum stack size 50"), "{error}");
}

#[test]
fn preparing_a_selection_keeps_chosen_equipment_and_adds_matching_sets() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("base.ron"),
        r#"
(
    version: 1,
    sets: [
        (name: "all", applies_to: All, grants: [(code: "ITEM_ALL", amount: 2)]),
        (name: "staff", applies_to: Weapon(Staff), grants: [(code: "ITEM_STAFF")]),
    ],
)
"#,
    )
    .unwrap();

    let gear = StarterGear::load_and_compile(directory.path(), catalogue()).unwrap();
    let items = gear
        .prepare(StarterSelection {
            race: ObjectRace::European,
            chest: 20,
            pants: 21,
            boots: 22,
            weapon: 23,
        })
        .unwrap();

    assert_eq!(
        items
            .iter()
            .map(|item| (item.reference_id, item.amount, item.slot))
            .collect::<Vec<_>>(),
        vec![(20, 1, 1), (21, 1, 4), (22, 1, 5), (23, 1, 6), (10, 2, 13), (13, 1, 14)]
    );
}

#[test]
fn rejects_an_item_that_cannot_use_its_configured_equipment_slot() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("base.ron"),
        r#"
(
    version: 1,
    sets: [
        (
            name: "bad-slot",
            applies_to: All,
            grants: [(code: "ITEM_ALL", placement: Equip(SecondaryWeapon))],
        ),
    ],
)
"#,
    )
    .unwrap();

    let error = match StarterGear::load_and_compile(directory.path(), catalogue()) {
        Err(error) => error,
        Ok(_) => panic!("invalid equipped starter item unexpectedly compiled"),
    };
    assert!(error.to_string().contains("cannot be equipped in slot"), "{error}");
}

#[test]
fn rejects_configured_gear_that_overwrites_selected_equipment() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("base.ron"),
        r#"
(
    version: 1,
    sets: [
        (
            name: "replacement-weapon",
            applies_to: Weapon(Staff),
            grants: [(code: "ITEM_EU_STAFF", placement: Equip(Weapon))],
        ),
    ],
)
"#,
    )
    .unwrap();

    let error = match StarterGear::load_and_compile(directory.path(), catalogue()) {
        Err(error) => error,
        Ok(_) => panic!("starter configuration overwriting selected gear unexpectedly compiled"),
    };
    assert!(error.to_string().contains("occupied equipment slot 6"), "{error}");
}

#[test]
fn rejects_matching_sets_that_target_the_same_equipment_slot() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("base.ron"),
        r#"
(
    version: 1,
    sets: [
        (
            name: "first-eu-ammo",
            applies_to: Race(European),
            grants: [(code: "ITEM_EU_BOLTS", placement: Equip(SecondaryWeapon))],
        ),
        (
            name: "eu-ammo",
            applies_to: Race(European),
            grants: [(code: "ITEM_EU_BOLTS", placement: Equip(SecondaryWeapon))],
        ),
    ],
)
"#,
    )
    .unwrap();

    let error = match StarterGear::load_and_compile(directory.path(), catalogue()) {
        Err(error) => error,
        Ok(_) => panic!("colliding starter sets unexpectedly compiled"),
    };
    assert!(error.to_string().contains("equipment slot 7 more than once"), "{error}");
}

#[test]
fn rejects_race_specific_equipment_from_a_set_matching_both_races() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("base.ron"),
        r#"
(
    version: 1,
    sets: [
        (
            name: "wrong-race",
            applies_to: All,
            grants: [(code: "ITEM_EU_BOLTS", placement: Equip(SecondaryWeapon))],
        ),
    ],
)
"#,
    )
    .unwrap();

    let error = match StarterGear::load_and_compile(directory.path(), catalogue()) {
        Err(error) => error,
        Ok(_) => panic!("race-incompatible starter set unexpectedly compiled"),
    };
    assert!(error.to_string().contains("not usable by every race"), "{error}");
}

#[test]
fn rejects_configured_items_that_require_more_than_starter_level() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("base.ron"),
        r#"
(
    version: 1,
    sets: [
        (name: "too-high", applies_to: All, grants: [(code: "ITEM_HIGH_LEVEL")]),
    ],
)
"#,
    )
    .unwrap();

    let error = match StarterGear::load_and_compile(directory.path(), catalogue()) {
        Err(error) => error,
        Ok(_) => panic!("high-level starter item unexpectedly compiled"),
    };
    assert!(error.to_string().contains("requires level 2"), "{error}");
}

#[test]
fn rejects_upgrade_levels_for_non_equipment_items() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("base.ron"),
        r#"
(
    version: 1,
    sets: [
        (name: "upgraded-potion", applies_to: All, grants: [(code: "ITEM_ALL", upgrade: 1)]),
    ],
)
"#,
    )
    .unwrap();

    let error = match StarterGear::load_and_compile(directory.path(), catalogue()) {
        Err(error) => error,
        Ok(_) => panic!("upgraded non-equipment starter item unexpectedly compiled"),
    };
    assert!(
        error.to_string().contains("upgrade can only be used with equipment"),
        "{error}"
    );
}

#[test]
fn matching_sets_use_layer_order_instead_of_filename_order() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("a-weapon.ron"),
        r#"(version: 1, sets: [(name: "staff", applies_to: Weapon(Staff), grants: [(code: "ITEM_STAFF")])])"#,
    )
    .unwrap();
    std::fs::write(
        directory.path().join("z-all.ron"),
        r#"(version: 1, sets: [(name: "all", applies_to: All, grants: [(code: "ITEM_ALL")])])"#,
    )
    .unwrap();

    let gear = StarterGear::load_and_compile(directory.path(), catalogue()).unwrap();
    let items = gear
        .resolve(StarterContext {
            race: ObjectRace::European,
            clothing: ObjectClothingType::Robe,
            weapon: ObjectWeaponType::Staff,
        })
        .unwrap();

    assert_eq!(
        items.iter().map(|item| item.reference_id).collect::<Vec<_>>(),
        vec![10, 13]
    );
}

#[test]
fn rejects_a_context_with_mismatched_race_and_equipment_types() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("base.ron"), "(version: 1, sets: [])").unwrap();
    let gear = StarterGear::load_and_compile(directory.path(), catalogue()).unwrap();

    let result = gear.resolve(StarterContext {
        race: ObjectRace::Chinese,
        clothing: ObjectClothingType::Robe,
        weapon: ObjectWeaponType::Staff,
    });

    assert!(result.is_err());
}

#[test]
fn rejects_unknown_grant_fields_instead_of_applying_defaults() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("base.ron"),
        r#"(version: 1, sets: [(name: "typo", applies_to: All, grants: [(code: "ITEM_ALL", ammount: 20)])])"#,
    )
    .unwrap();

    assert!(StarterGear::load_and_compile(directory.path(), catalogue()).is_err());
}

#[test]
fn rejects_amounts_that_persistence_cannot_represent() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("base.ron"),
        r#"(version: 1, sets: [(name: "huge", applies_to: All, grants: [(code: "ITEM_HUGE_STACK", amount: 40000)])])"#,
    )
    .unwrap();

    let error = match StarterGear::load_and_compile(directory.path(), catalogue()) {
        Err(error) => error,
        Ok(_) => panic!("unpersistable starter amount unexpectedly compiled"),
    };
    assert!(error.to_string().contains("persistence limit 32767"), "{error}");
}

#[test]
fn semantic_chest_and_wrist_placements_map_to_the_item_part_slots() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("base.ron"),
        r#"
(
    version: 1,
    sets: [
        (
            name: "remaining-robe",
            applies_to: Race(European),
            grants: [
                (code: "ITEM_EU_ROBE_BODY", placement: Equip(ChestArmor)),
                (code: "ITEM_EU_ROBE_ARM", placement: Equip(WristArmor)),
            ],
        ),
    ],
)
"#,
    )
    .unwrap();

    let gear = StarterGear::load_and_compile(directory.path(), catalogue()).unwrap();
    let items = gear
        .resolve(StarterContext {
            race: ObjectRace::European,
            clothing: ObjectClothingType::Robe,
            weapon: ObjectWeaponType::Staff,
        })
        .unwrap();

    assert_eq!(items.iter().map(|item| item.slot).collect::<Vec<_>>(), vec![2, 3]);
}

#[test]
fn rejects_secondary_items_that_do_not_match_the_selected_weapon_type() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("base.ron"),
        r#"
(
    version: 1,
    sets: [
        (
            name: "staff-bolts",
            applies_to: Weapon(Staff),
            grants: [(code: "ITEM_EU_BOLTS", placement: Equip(SecondaryWeapon))],
        ),
    ],
)
"#,
    )
    .unwrap();

    let error = match StarterGear::load_and_compile(directory.path(), catalogue()) {
        Err(error) => error,
        Ok(_) => panic!("weapon-incompatible secondary item unexpectedly compiled"),
    };
    assert!(error.to_string().contains("secondary item is incompatible"), "{error}");
}

#[test]
fn rejects_nonstandard_client_selected_equipment() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("base.ron"), "(version: 1, sets: [])").unwrap();
    let gear = StarterGear::load_and_compile(directory.path(), catalogue()).unwrap();

    let error = gear
        .prepare(StarterSelection {
            race: ObjectRace::European,
            chest: 20,
            pants: 21,
            boots: 22,
            weapon: 29,
        })
        .unwrap_err();

    assert!(error.to_string().contains("ordinary creation equipment"), "{error}");
}

#[test]
fn european_caster_rods_can_receive_a_shield() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("base.ron"),
        r#"
(
    version: 1,
    sets: [
        (
            name: "warlock-shield",
            applies_to: Weapon(WarlockStaff),
            grants: [(code: "ITEM_EU_SHIELD", placement: Equip(SecondaryWeapon))],
        ),
    ],
)
"#,
    )
    .unwrap();

    let gear = StarterGear::load_and_compile(directory.path(), catalogue()).unwrap();
    let items = gear
        .resolve(StarterContext {
            race: ObjectRace::European,
            clothing: ObjectClothingType::Robe,
            weapon: ObjectWeaponType::WarlockStaff,
        })
        .unwrap();

    assert_eq!(items.iter().map(|item| item.slot).collect::<Vec<_>>(), vec![7]);
}
