use silkroad_data::itemdata::{RefItemData, RefItemRarity};
use silkroad_data::DataMap;
use silkroad_definitions::type_id::{
    ObjectClothingPart, ObjectClothingType, ObjectEquippable, ObjectItem, ObjectRace, ObjectType, ObjectWeaponType,
};
use silkroad_game_base::{item_type_matches_race, Race};

pub const FIRST_BAG_SLOT: u8 = 13;
pub const STARTER_INVENTORY_SIZE: u8 = 45;

pub struct StarterContext {
    pub race: ObjectRace,
    pub clothing: ObjectClothingType,
    pub weapon: ObjectWeaponType,
}

pub struct StarterSelection {
    pub race: ObjectRace,
    pub chest: u32,
    pub pants: u32,
    pub boots: u32,
    pub weapon: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StarterItem {
    pub reference_id: u32,
    pub upgrade_level: u8,
    pub amount: u16,
    pub slot: u8,
}

pub(crate) struct CompiledSet {
    pub(crate) applies_to: CompiledSelector,
    pub(crate) grants: Vec<CompiledGrant>,
}

pub(crate) enum CompiledSelector {
    All,
    Race(ObjectRace),
    Clothing(ObjectClothingType),
    Weapon(ObjectWeaponType),
}

pub(crate) struct CompiledGrant {
    pub(crate) reference_id: u32,
    pub(crate) amount: u16,
    pub(crate) max_stack_size: u16,
    pub(crate) upgrade_level: u8,
    pub(crate) slot: Option<u8>,
    pub(crate) secondary: Option<SecondaryKind>,
}

pub(crate) enum SecondaryKind {
    ChineseShield,
    EuropeanShield,
    Arrows,
    Bolts,
}

impl SecondaryKind {
    fn matches(&self, weapon: ObjectWeaponType) -> bool {
        match self {
            Self::ChineseShield => matches!(weapon, ObjectWeaponType::Sword | ObjectWeaponType::Blade),
            Self::EuropeanShield => matches!(
                weapon,
                ObjectWeaponType::OneHandSword | ObjectWeaponType::WarlockStaff | ObjectWeaponType::ClericRod
            ),
            Self::Arrows => weapon == ObjectWeaponType::Bow,
            Self::Bolts => weapon == ObjectWeaponType::Crossbow,
        }
    }
}

struct PendingGrant<'a> {
    reference_id: u32,
    amount: u32,
    max_stack_size: u16,
    upgrade_level: u8,
    slot: Option<u8>,
    secondary: Option<&'a SecondaryKind>,
}

pub struct StarterGear {
    pub(crate) sets: Vec<CompiledSet>,
    pub(crate) items: &'static DataMap<RefItemData>,
}

impl StarterGear {
    pub fn prepare(&self, selection: StarterSelection) -> Result<Vec<StarterItem>, StarterGearSelectionError> {
        let clothing = self.selected_clothing(selection.chest, ObjectClothingPart::Shoulder, selection.race)?;
        let pants = self.selected_clothing(selection.pants, ObjectClothingPart::Leg, selection.race)?;
        let boots = self.selected_clothing(selection.boots, ObjectClothingPart::Foot, selection.race)?;
        if pants != clothing || boots != clothing {
            return Err(StarterGearSelectionError::Invalid(
                "selected clothing pieces do not use the same clothing family".into(),
            ));
        }
        let weapon = self.selected_weapon(selection.weapon, selection.race)?;

        let mut result = vec![
            selected_item(selection.chest, 1),
            selected_item(selection.pants, 4),
            selected_item(selection.boots, 5),
            selected_item(selection.weapon, 6),
        ];
        result.extend(
            self.resolve(StarterContext {
                race: selection.race,
                clothing,
                weapon,
            })
            .map_err(StarterGearSelectionError::Resolve)?,
        );
        Ok(result)
    }

    fn selected_clothing(
        &self,
        reference_id: u32,
        expected_part: ObjectClothingPart,
        race: ObjectRace,
    ) -> Result<ObjectClothingType, StarterGearSelectionError> {
        let item = self.selected_item(reference_id)?;
        let object_type = ObjectType::from_type_id(&item.common.type_id);
        let Some(ObjectType::Item(ObjectItem::Equippable(ObjectEquippable::Clothing(kind, part)))) = object_type else {
            return Err(StarterGearSelectionError::Invalid(format!(
                "selected item '{}' is not clothing",
                item.common.id
            )));
        };
        if part != expected_part || !item_type_matches_race(game_race(race), object_type.unwrap()) {
            return Err(StarterGearSelectionError::Invalid(format!(
                "selected clothing '{}' is incompatible with the requested slot or race",
                item.common.id
            )));
        }
        Ok(kind)
    }

    fn selected_weapon(
        &self,
        reference_id: u32,
        race: ObjectRace,
    ) -> Result<ObjectWeaponType, StarterGearSelectionError> {
        let item = self.selected_item(reference_id)?;
        let object_type = ObjectType::from_type_id(&item.common.type_id);
        let Some(ObjectType::Item(ObjectItem::Equippable(ObjectEquippable::Weapon(kind)))) = object_type else {
            return Err(StarterGearSelectionError::Invalid(format!(
                "selected item '{}' is not a weapon",
                item.common.id
            )));
        };
        if !item_type_matches_race(game_race(race), object_type.unwrap()) {
            return Err(StarterGearSelectionError::Invalid(format!(
                "selected weapon '{}' is incompatible with the character race",
                item.common.id
            )));
        }
        Ok(kind)
    }

    fn selected_item(&self, reference_id: u32) -> Result<&RefItemData, StarterGearSelectionError> {
        let item = self.items.find_id(reference_id).ok_or_else(|| {
            StarterGearSelectionError::Invalid(format!("selected item reference {reference_id} is unknown"))
        })?;
        if !item.common.service || item.required_level.is_some_and(|level| level.get() > 1) {
            return Err(StarterGearSelectionError::Invalid(format!(
                "selected item '{}' is inactive or requires a level above 1",
                item.common.id
            )));
        }
        if !item.can_drop || item.rarity != RefItemRarity::General {
            return Err(StarterGearSelectionError::Invalid(format!(
                "selected item '{}' is not ordinary creation equipment",
                item.common.id
            )));
        }
        Ok(item)
    }

    pub fn resolve(&self, context: StarterContext) -> Result<Vec<StarterItem>, StarterGearResolveError> {
        if !clothing_matches_race(context.clothing, context.race) || !weapon_matches_race(context.weapon, context.race)
        {
            return Err(StarterGearResolveError::UnsupportedContext);
        }

        let mut pending: Vec<PendingGrant<'_>> = Vec::new();
        for set in &self.sets {
            if !set.applies_to.matches(&context) {
                continue;
            }
            for grant in &set.grants {
                if grant.slot.is_none() && grant.max_stack_size > 1 {
                    if let Some(existing) = pending.iter_mut().find(|existing| {
                        existing.slot.is_none()
                            && existing.reference_id == grant.reference_id
                            && existing.upgrade_level == grant.upgrade_level
                    }) {
                        existing.amount += u32::from(grant.amount);
                        continue;
                    }
                }
                pending.push(PendingGrant {
                    reference_id: grant.reference_id,
                    amount: u32::from(grant.amount),
                    max_stack_size: grant.max_stack_size,
                    upgrade_level: grant.upgrade_level,
                    slot: grant.slot,
                    secondary: grant.secondary.as_ref(),
                });
            }
        }

        let mut next_bag_slot = FIRST_BAG_SLOT;
        let mut occupied_equipment = [false; FIRST_BAG_SLOT as usize];
        for slot in [1usize, 4, 5, 6] {
            occupied_equipment[slot] = true;
        }
        let mut result = Vec::new();
        for grant in pending {
            if grant
                .secondary
                .is_some_and(|secondary| !secondary.matches(context.weapon))
            {
                return Err(StarterGearResolveError::IncompatibleSecondary);
            }
            let mut remaining = grant.amount;
            while remaining > 0 {
                let slot = match grant.slot {
                    Some(slot) if occupied_equipment[usize::from(slot)] => {
                        return Err(StarterGearResolveError::DuplicateEquipmentSlot(slot));
                    },
                    Some(slot) => {
                        occupied_equipment[usize::from(slot)] = true;
                        slot
                    },
                    None if next_bag_slot < STARTER_INVENTORY_SIZE => {
                        let slot = next_bag_slot;
                        next_bag_slot += 1;
                        slot
                    },
                    None => return Err(StarterGearResolveError::InventoryFull),
                };
                let amount = remaining.min(u32::from(grant.max_stack_size));
                result.push(StarterItem {
                    reference_id: grant.reference_id,
                    upgrade_level: grant.upgrade_level,
                    amount: amount as u16,
                    slot,
                });
                remaining -= amount;
            }
        }

        Ok(result)
    }
}

impl CompiledSelector {
    fn matches(&self, context: &StarterContext) -> bool {
        match self {
            Self::All => true,
            Self::Race(race) => *race == context.race,
            Self::Clothing(clothing) => *clothing == context.clothing,
            Self::Weapon(weapon) => *weapon == context.weapon,
        }
    }
}

fn game_race(race: ObjectRace) -> Race {
    match race {
        ObjectRace::Chinese => Race::Chinese,
        ObjectRace::European => Race::European,
    }
}

fn selected_item(reference_id: u32, slot: u8) -> StarterItem {
    StarterItem {
        reference_id,
        upgrade_level: 0,
        amount: 1,
        slot,
    }
}

fn clothing_matches_race(clothing: ObjectClothingType, race: ObjectRace) -> bool {
    matches!(
        (clothing, race),
        (
            ObjectClothingType::Garment | ObjectClothingType::Protector | ObjectClothingType::Armor,
            ObjectRace::Chinese
        ) | (
            ObjectClothingType::Robe | ObjectClothingType::LightArmor | ObjectClothingType::HeavyArmor,
            ObjectRace::European
        )
    )
}

fn weapon_matches_race(weapon: ObjectWeaponType, race: ObjectRace) -> bool {
    matches!(
        (weapon, race),
        (
            ObjectWeaponType::Sword
                | ObjectWeaponType::Blade
                | ObjectWeaponType::Spear
                | ObjectWeaponType::Glavie
                | ObjectWeaponType::Bow,
            ObjectRace::Chinese
        ) | (
            ObjectWeaponType::OneHandSword
                | ObjectWeaponType::TwoHandSword
                | ObjectWeaponType::Axe
                | ObjectWeaponType::WarlockStaff
                | ObjectWeaponType::Staff
                | ObjectWeaponType::Crossbow
                | ObjectWeaponType::Dagger
                | ObjectWeaponType::Harp
                | ObjectWeaponType::ClericRod,
            ObjectRace::European
        )
    )
}

#[derive(Debug, thiserror::Error)]
pub enum StarterGearSelectionError {
    #[error("invalid selected starter gear: {0}")]
    Invalid(String),
    #[error(transparent)]
    Resolve(#[from] StarterGearResolveError),
}

#[derive(Debug, thiserror::Error)]
pub enum StarterGearResolveError {
    #[error("starter gear context combines incompatible race, clothing, or weapon types")]
    UnsupportedContext,
    #[error("starter gear does not fit in the character inventory")]
    InventoryFull,
    #[error("starter secondary item is incompatible with the selected weapon type")]
    IncompatibleSecondary,
    #[error("starter gear targets equipment slot {0} more than once")]
    DuplicateEquipmentSlot(u8),
}
