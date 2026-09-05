use crate::definition::{
    load, AppliesToDef, ClothingDef, EquipmentSlotDef, PlacementDef, RaceDef, StarterGearError, ValidationIssue,
    WeaponDef,
};
use crate::runtime::{CompiledGrant, CompiledSelector, CompiledSet, SecondaryKind, StarterContext, StarterGear};
use silkroad_data::itemdata::RefItemData;
use silkroad_data::DataMap;
use silkroad_definitions::type_id::{
    ObjectClothingType, ObjectConsumable, ObjectConsumableAmmo, ObjectEquippable, ObjectItem, ObjectRace, ObjectType,
    ObjectWeaponType,
};
use silkroad_game_base::{item_type_matches_equipment_slot, item_type_matches_race, Race};
use std::collections::{HashMap, HashSet};
use std::path::Path;

impl StarterGear {
    pub fn load_and_compile(directory: &Path, items: &'static DataMap<RefItemData>) -> Result<Self, StarterGearError> {
        let mut definitions = load(directory)?;
        definitions.sort_by(|left, right| {
            selector_layer(left.set.applies_to)
                .cmp(&selector_layer(right.set.applies_to))
                .then_with(|| left.set.name.cmp(&right.set.name))
        });
        let mut issues = Vec::new();
        let mut names = HashMap::new();
        let mut sets = Vec::new();

        for sourced in definitions {
            if let Some(original) = names.insert(sourced.set.name.clone(), sourced.path.clone()) {
                issues.push(ValidationIssue::new(
                    &sourced.path,
                    Some(sourced.set.name.clone()),
                    format!("duplicate set name; first defined in {}", original.display()),
                ));
                continue;
            }

            let applies_to = sourced.set.applies_to;
            let mut grants = Vec::new();
            for grant in sourced.set.grants {
                let Some(reference) = items.find_code(&grant.code) else {
                    issues.push(ValidationIssue::new(
                        &sourced.path,
                        Some(sourced.set.name.clone()),
                        format!("unknown item code '{}'", grant.code),
                    ));
                    continue;
                };
                if !reference.common.service {
                    issues.push(ValidationIssue::new(
                        &sourced.path,
                        Some(sourced.set.name.clone()),
                        format!("item '{}' is inactive", grant.code),
                    ));
                    continue;
                }
                let Some(object_type) = ObjectType::from_type_id(&reference.common.type_id) else {
                    issues.push(ValidationIssue::new(
                        &sourced.path,
                        Some(sourced.set.name.clone()),
                        format!("item '{}' has an unsupported object type", grant.code),
                    ));
                    continue;
                };
                if is_race_restricted(object_type)
                    && applicable_races(applies_to)
                        .into_iter()
                        .any(|race| !item_type_matches_race(race, object_type))
                {
                    issues.push(ValidationIssue::new(
                        &sourced.path,
                        Some(sourced.set.name.clone()),
                        format!("item '{}' is not usable by every race matched by the set", grant.code),
                    ));
                    continue;
                }
                if grant.upgrade > 0 && !matches!(object_type, ObjectType::Item(ObjectItem::Equippable(_))) {
                    issues.push(ValidationIssue::new(
                        &sourced.path,
                        Some(sourced.set.name.clone()),
                        format!("item '{}': upgrade can only be used with equipment", grant.code),
                    ));
                    continue;
                }
                if let Some(required_level) = reference.required_level.filter(|level| level.get() > 1) {
                    issues.push(ValidationIssue::new(
                        &sourced.path,
                        Some(sourced.set.name.clone()),
                        format!(
                            "item '{}' requires level {} but starters are level 1",
                            grant.code, required_level
                        ),
                    ));
                    continue;
                }
                if reference.common.ref_id > i32::MAX as u32 {
                    issues.push(ValidationIssue::new(
                        &sourced.path,
                        Some(sourced.set.name.clone()),
                        format!("item '{}' reference id exceeds the persistence limit", grant.code),
                    ));
                    continue;
                }
                if grant.amount > i16::MAX as u16 {
                    issues.push(ValidationIssue::new(
                        &sourced.path,
                        Some(sourced.set.name.clone()),
                        format!(
                            "item '{}' amount {} exceeds the persistence limit 32767",
                            grant.code, grant.amount
                        ),
                    ));
                    continue;
                }
                if grant.amount == 0 || grant.amount > reference.max_stack_size {
                    issues.push(ValidationIssue::new(
                        &sourced.path,
                        Some(sourced.set.name.clone()),
                        format!(
                            "item '{}' amount {} must be between 1 and its maximum stack size {}",
                            grant.code, grant.amount, reference.max_stack_size
                        ),
                    ));
                    continue;
                }
                let slot = match grant.placement {
                    PlacementDef::Bag => None,
                    PlacementDef::Equip(slot) => {
                        let slot = equipment_slot(slot);
                        if matches!(slot, 1 | 4 | 5 | 6) {
                            issues.push(ValidationIssue::new(
                                &sourced.path,
                                Some(sourced.set.name.clone()),
                                format!("item '{}' targets occupied equipment slot {}", grant.code, slot),
                            ));
                            continue;
                        }
                        if !item_type_matches_equipment_slot(slot, object_type) {
                            issues.push(ValidationIssue::new(
                                &sourced.path,
                                Some(sourced.set.name.clone()),
                                format!("item '{}' cannot be equipped in slot {}", grant.code, slot),
                            ));
                            continue;
                        }
                        Some(slot)
                    },
                };
                grants.push(CompiledGrant {
                    reference_id: reference.common.ref_id,
                    amount: grant.amount,
                    max_stack_size: reference.max_stack_size.min(i16::MAX as u16),
                    upgrade_level: grant.upgrade,
                    slot,
                    secondary: if slot == Some(7) {
                        secondary_kind(object_type)
                    } else {
                        None
                    },
                });
            }
            sets.push(CompiledSet {
                applies_to: selector(applies_to),
                grants,
            });
        }

        if !issues.is_empty() {
            return Err(StarterGearError::Validation(issues));
        }

        let gear = Self { sets, items };
        let mut context_errors = HashSet::new();
        for context in valid_contexts() {
            if let Err(error) = gear.resolve(context) {
                context_errors.insert(error.to_string());
            }
        }
        if context_errors.is_empty() {
            Ok(gear)
        } else {
            let mut context_errors = context_errors.into_iter().collect::<Vec<_>>();
            context_errors.sort();
            Err(StarterGearError::Validation(
                context_errors
                    .into_iter()
                    .map(|reason| ValidationIssue::new(directory, None, reason))
                    .collect(),
            ))
        }
    }
}

fn secondary_kind(object_type: ObjectType) -> Option<SecondaryKind> {
    match object_type {
        ObjectType::Item(ObjectItem::Equippable(ObjectEquippable::Shield(ObjectRace::Chinese))) => {
            Some(SecondaryKind::ChineseShield)
        },
        ObjectType::Item(ObjectItem::Equippable(ObjectEquippable::Shield(ObjectRace::European))) => {
            Some(SecondaryKind::EuropeanShield)
        },
        ObjectType::Item(ObjectItem::Consumable(ObjectConsumable::Ammo(ObjectConsumableAmmo::Arrows))) => {
            Some(SecondaryKind::Arrows)
        },
        ObjectType::Item(ObjectItem::Consumable(ObjectConsumable::Ammo(ObjectConsumableAmmo::Bolts))) => {
            Some(SecondaryKind::Bolts)
        },
        _ => None,
    }
}

fn selector_layer(selector: AppliesToDef) -> u8 {
    match selector {
        AppliesToDef::All => 0,
        AppliesToDef::Race(_) => 1,
        AppliesToDef::Clothing(_) => 2,
        AppliesToDef::Weapon(_) => 3,
    }
}

fn is_race_restricted(object_type: ObjectType) -> bool {
    matches!(
        object_type,
        ObjectType::Item(ObjectItem::Equippable(
            ObjectEquippable::Clothing(_, _)
                | ObjectEquippable::Shield(_)
                | ObjectEquippable::Jewelry(_, _)
                | ObjectEquippable::Weapon(_)
        )) | ObjectType::Item(ObjectItem::Consumable(ObjectConsumable::Ammo(_)))
    )
}

fn applicable_races(selector: AppliesToDef) -> Vec<Race> {
    match selector {
        AppliesToDef::All => vec![Race::Chinese, Race::European],
        AppliesToDef::Race(RaceDef::Chinese)
        | AppliesToDef::Clothing(ClothingDef::Garment | ClothingDef::Protector | ClothingDef::Armor)
        | AppliesToDef::Weapon(
            WeaponDef::Sword | WeaponDef::Blade | WeaponDef::Spear | WeaponDef::Glavie | WeaponDef::Bow,
        ) => vec![Race::Chinese],
        AppliesToDef::Race(RaceDef::European)
        | AppliesToDef::Clothing(ClothingDef::Robe | ClothingDef::LightArmor | ClothingDef::HeavyArmor)
        | AppliesToDef::Weapon(
            WeaponDef::OneHandSword
            | WeaponDef::TwoHandSword
            | WeaponDef::Axe
            | WeaponDef::WarlockStaff
            | WeaponDef::Staff
            | WeaponDef::Crossbow
            | WeaponDef::Dagger
            | WeaponDef::Harp
            | WeaponDef::ClericRod,
        ) => vec![Race::European],
    }
}

fn valid_contexts() -> Vec<StarterContext> {
    let chinese_clothing = [
        ObjectClothingType::Garment,
        ObjectClothingType::Protector,
        ObjectClothingType::Armor,
    ];
    let chinese_weapons = [
        ObjectWeaponType::Sword,
        ObjectWeaponType::Blade,
        ObjectWeaponType::Spear,
        ObjectWeaponType::Glavie,
        ObjectWeaponType::Bow,
    ];
    let european_clothing = [
        ObjectClothingType::Robe,
        ObjectClothingType::LightArmor,
        ObjectClothingType::HeavyArmor,
    ];
    let european_weapons = [
        ObjectWeaponType::OneHandSword,
        ObjectWeaponType::TwoHandSword,
        ObjectWeaponType::Axe,
        ObjectWeaponType::WarlockStaff,
        ObjectWeaponType::Staff,
        ObjectWeaponType::Crossbow,
        ObjectWeaponType::Dagger,
        ObjectWeaponType::Harp,
        ObjectWeaponType::ClericRod,
    ];

    let mut contexts = Vec::new();
    for clothing in chinese_clothing {
        for weapon in chinese_weapons {
            contexts.push(StarterContext {
                race: ObjectRace::Chinese,
                clothing,
                weapon,
            });
        }
    }
    for clothing in european_clothing {
        for weapon in european_weapons {
            contexts.push(StarterContext {
                race: ObjectRace::European,
                clothing,
                weapon,
            });
        }
    }
    contexts
}

fn selector(definition: AppliesToDef) -> CompiledSelector {
    match definition {
        AppliesToDef::All => CompiledSelector::All,
        AppliesToDef::Race(race) => CompiledSelector::Race(match race {
            RaceDef::Chinese => ObjectRace::Chinese,
            RaceDef::European => ObjectRace::European,
        }),
        AppliesToDef::Clothing(clothing) => CompiledSelector::Clothing(match clothing {
            ClothingDef::Garment => ObjectClothingType::Garment,
            ClothingDef::Protector => ObjectClothingType::Protector,
            ClothingDef::Armor => ObjectClothingType::Armor,
            ClothingDef::Robe => ObjectClothingType::Robe,
            ClothingDef::LightArmor => ObjectClothingType::LightArmor,
            ClothingDef::HeavyArmor => ObjectClothingType::HeavyArmor,
        }),
        AppliesToDef::Weapon(weapon) => CompiledSelector::Weapon(match weapon {
            WeaponDef::Sword => ObjectWeaponType::Sword,
            WeaponDef::Blade => ObjectWeaponType::Blade,
            WeaponDef::Spear => ObjectWeaponType::Spear,
            WeaponDef::Glavie => ObjectWeaponType::Glavie,
            WeaponDef::Bow => ObjectWeaponType::Bow,
            WeaponDef::OneHandSword => ObjectWeaponType::OneHandSword,
            WeaponDef::TwoHandSword => ObjectWeaponType::TwoHandSword,
            WeaponDef::Axe => ObjectWeaponType::Axe,
            WeaponDef::WarlockStaff => ObjectWeaponType::WarlockStaff,
            WeaponDef::Staff => ObjectWeaponType::Staff,
            WeaponDef::Crossbow => ObjectWeaponType::Crossbow,
            WeaponDef::Dagger => ObjectWeaponType::Dagger,
            WeaponDef::Harp => ObjectWeaponType::Harp,
            WeaponDef::ClericRod => ObjectWeaponType::ClericRod,
        }),
    }
}

fn equipment_slot(slot: EquipmentSlotDef) -> u8 {
    match slot {
        EquipmentSlotDef::HeadArmor => 0,
        EquipmentSlotDef::ShoulderArmor => 1,
        EquipmentSlotDef::ChestArmor => 2,
        EquipmentSlotDef::WristArmor => 3,
        EquipmentSlotDef::LegArmor => 4,
        EquipmentSlotDef::FootArmor => 5,
        EquipmentSlotDef::Weapon => 6,
        EquipmentSlotDef::SecondaryWeapon => 7,
        EquipmentSlotDef::Earring => 8,
        EquipmentSlotDef::Necklace => 9,
        EquipmentSlotDef::LeftRing => 10,
        EquipmentSlotDef::RightRing => 11,
    }
}
