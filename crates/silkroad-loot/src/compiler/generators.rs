use super::distributions::distribution_maximum;
use super::LootCompiler;
use crate::definition::{
    DefinitionSource, EquipmentKindDef, EquipmentSelectorDef, GeneratorDef, ItemRarityDef, LootValidationIssue,
    OriginDef, RarityKindDef,
};
use crate::runtime::{CompiledGenerator, EquipmentCandidates, EquipmentLevelBucket};
use silkroad_data::itemdata::{RefItemData, RefItemRarity};
use silkroad_definitions::rarity::EntityRarityType;
use silkroad_definitions::type_id::{ObjectEquippable, ObjectItem, ObjectType};
use std::collections::HashMap;

impl LootCompiler<'_> {
    pub(super) fn compile_generator(
        &self,
        generator: &GeneratorDef,
        source: &DefinitionSource,
        pool_name: &str,
        issues: &mut Vec<LootValidationIssue>,
    ) -> Option<CompiledGenerator> {
        match generator {
            GeneratorDef::Gold => Some(CompiledGenerator::Gold),
            GeneratorDef::Item(item) => {
                let reference = self.items.find_code(&item.code).or_else(|| {
                    issues.push(Self::issue_at(
                        source.clone(),
                        "pool entry",
                        Some(pool_name.to_string()),
                        format!("unknown item code '{}'", item.code),
                    ));
                    None
                })?;

                if !reference.common.service {
                    issues.push(Self::issue_at(
                        source.clone(),
                        "pool entry",
                        Some(pool_name.to_string()),
                        format!("item '{}' is inactive", item.code),
                    ));
                    return None;
                }

                if !reference.can_drop {
                    issues.push(Self::issue_at(
                        source.clone(),
                        "pool entry",
                        Some(pool_name.to_string()),
                        format!("item '{}' cannot be dropped", item.code),
                    ));
                    return None;
                }

                if !is_supported_stackable(reference) {
                    issues.push(Self::issue_at(
                        source.clone(),
                        "pool entry",
                        Some(pool_name.to_string()),
                        format!("item '{}' is not a supported droppable stackable item", item.code),
                    ));
                    return None;
                }

                let amount = self.compile_distribution(&item.amount, source, "amount", issues)?;
                let maximum = distribution_maximum(&item.amount);
                if maximum < 1 {
                    issues.push(Self::issue_at(
                        source.clone(),
                        "pool entry",
                        Some(pool_name.to_string()),
                        String::from("item amount must be at least 1"),
                    ));
                    return None;
                }

                if u32::from(maximum) > u32::from(reference.max_stack_size) {
                    issues.push(Self::issue_at(
                        source.clone(),
                        "pool entry",
                        Some(pool_name.to_string()),
                        format!(
                            "item amount {} exceeds the maximum stack size {} of '{}'",
                            maximum, reference.max_stack_size, item.code
                        ),
                    ));
                    return None;
                }

                Some(CompiledGenerator::Item { reference, amount })
            },
            GeneratorDef::Equipment(selector) => self.compile_equipment(selector, source, pool_name, issues),
        }
    }

    fn compile_equipment(
        &self,
        selector: &EquipmentSelectorDef,
        source: &DefinitionSource,
        pool_name: &str,
        issues: &mut Vec<LootValidationIssue>,
    ) -> Option<CompiledGenerator> {
        let context = |reason: String| {
            Self::issue_at(
                source.clone(),
                "pool entry",
                Some(pool_name.to_string()),
                format!("invalid equipment selector: {}", reason),
            )
        };

        if selector.required_level.min > selector.required_level.max {
            issues.push(context(format!(
                "required level window is reversed ({}..={})",
                selector.required_level.min, selector.required_level.max
            )));
            return None;
        }

        if selector.origins.is_empty() || selector.kinds.is_empty() || selector.item_rarities.is_empty() {
            issues.push(context(String::from(
                "origin, kind, and rarity filters must not be empty",
            )));
            return None;
        }

        let upgrade = self.compile_distribution(&selector.upgrade, source, "upgrade", issues)?;

        let mut by_level: HashMap<u8, Vec<&'static RefItemData>> = HashMap::new();
        for item in self.items.iter() {
            if !item.common.service || !item.can_drop {
                continue;
            }

            let Some(equippable) = as_supported_equipment(item) else {
                continue;
            };

            if !selector.kinds.iter().any(|kind| matches_kind(*kind, &equippable)) {
                continue;
            }

            if !selector.origins.iter().any(|origin| origin_matches(origin, item)) {
                continue;
            }

            let mapped_rarity = item_rarity(item.rarity);
            if !selector.item_rarities.contains(&mapped_rarity) {
                continue;
            }

            let Some(required_level) = item.required_level else {
                continue;
            };

            by_level.entry(required_level.get()).or_default().push(item);
        }

        if by_level.is_empty() {
            issues.push(context(String::from(
                "selector matches no items in the media catalogue",
            )));
            return None;
        }

        let candidates = EquipmentCandidates::new(
            (selector.required_level.min, selector.required_level.max),
            by_level
                .into_iter()
                .map(|(level, items)| EquipmentLevelBucket { level, items })
                .collect(),
            upgrade,
        );

        Some(CompiledGenerator::Equipment(candidates))
    }
}

pub(super) fn rarity_kind(kind: RarityKindDef) -> EntityRarityType {
    match kind {
        RarityKindDef::Normal => EntityRarityType::Normal,
        RarityKindDef::Champion => EntityRarityType::Champion,
        RarityKindDef::Unique => EntityRarityType::Unique,
        RarityKindDef::Giant => EntityRarityType::Giant,
        RarityKindDef::Titan => EntityRarityType::Titan,
        RarityKindDef::Elite => EntityRarityType::Elite,
        RarityKindDef::Strong => EntityRarityType::Strong,
        RarityKindDef::Unique2 => EntityRarityType::Unique2,
    }
}

fn item_rarity(rarity: RefItemRarity) -> ItemRarityDef {
    match rarity {
        RefItemRarity::General => ItemRarityDef::General,
        RefItemRarity::Blue => ItemRarityDef::Blue,
        RefItemRarity::Seal => ItemRarityDef::Seal,
        RefItemRarity::Set => ItemRarityDef::Set,
        RefItemRarity::Roc => ItemRarityDef::Roc,
        RefItemRarity::Legend => ItemRarityDef::Legend,
    }
}

fn matches_kind(kind: EquipmentKindDef, equippable: &ObjectEquippable) -> bool {
    matches!(
        (kind, equippable),
        (EquipmentKindDef::Weapon, ObjectEquippable::Weapon(_))
            | (EquipmentKindDef::Shield, ObjectEquippable::Shield(_))
            | (EquipmentKindDef::Clothing, ObjectEquippable::Clothing(_, _))
            | (EquipmentKindDef::Jewelry, ObjectEquippable::Jewelry(_, _))
    )
}

fn origin_matches(origin: &OriginDef, item: &RefItemData) -> bool {
    match (origin, item.common.country) {
        (_, silkroad_data::common::RefOrigin::General) => true,
        (OriginDef::Chinese, silkroad_data::common::RefOrigin::Chinese) => true,
        (OriginDef::European, silkroad_data::common::RefOrigin::European) => true,
        _ => false,
    }
}

/// Whether the item can be represented as a runtime stackable/expendable drop.
/// Gold, pet/COS, trade goods, avatars, and other unsupported kinds are
/// rejected.
fn is_supported_stackable(item: &RefItemData) -> bool {
    let Some(ObjectType::Item(ObjectItem::Consumable(consumable))) = ObjectType::from_type_id(&item.common.type_id)
    else {
        return false;
    };

    use silkroad_definitions::type_id::{ObjectConsumable, ObjectConsumableCurrency};
    !matches!(
        consumable,
        ObjectConsumable::Currency(ObjectConsumableCurrency::Gold) | ObjectConsumable::Pet(_)
    )
}

/// Returns the broad supported equipment classification of an item, if any.
fn as_supported_equipment(item: &RefItemData) -> Option<ObjectEquippable> {
    match ObjectType::from_type_id(&item.common.type_id) {
        Some(ObjectType::Item(ObjectItem::Equippable(equippable))) => match equippable {
            ObjectEquippable::Weapon(_)
            | ObjectEquippable::Shield(_)
            | ObjectEquippable::Clothing(_, _)
            | ObjectEquippable::Jewelry(_, _) => Some(equippable),
            _ => None,
        },
        _ => None,
    }
}
