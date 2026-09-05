use crate::{Change, ChangeTracked, MergeResult, Race};
use silkroad_data::itemdata::RefItemData;
use silkroad_data::DataEntry;
pub use silkroad_definitions::inventory::EquipmentSlot;
use silkroad_definitions::type_id::{
    ObjectClothingPart, ObjectClothingType, ObjectConsumable, ObjectConsumableAmmo, ObjectEquippable, ObjectItem,
    ObjectJewelryType, ObjectRace, ObjectType, ObjectWeaponType,
};
use std::collections::HashMap;

const EQUIPMENT_SLOT_COUNT: u8 = 13;
const FIRST_BAG_SLOT: u8 = EQUIPMENT_SLOT_COUNT;
const LAST_ITEM_SLOT: u8 = 0xFD;

/// A zero-based position in the bag section of the main inventory.
///
/// This deliberately does not expose the protocol's `13`-based representation.
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct BagSlot(u8);

impl BagSlot {
    pub const MAX_INDEX: u8 = LAST_ITEM_SLOT - FIRST_BAG_SLOT;

    pub const fn new(index: u8) -> Option<Self> {
        if index <= Self::MAX_INDEX {
            Some(Self(index))
        } else {
            None
        }
    }

    pub const fn index(self) -> u8 {
        self.0
    }

    const fn from_raw(raw: u8) -> Option<Self> {
        if raw >= FIRST_BAG_SLOT && raw <= LAST_ITEM_SLOT {
            Some(Self(raw - FIRST_BAG_SLOT))
        } else {
            None
        }
    }

    const fn as_raw(self) -> u8 {
        self.0 + FIRST_BAG_SLOT
    }
}

/// A semantic slot in the character's main inventory.
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum MainInventorySlot {
    Equipment(EquipmentSlot),
    Bag(BagSlot),
}

impl MainInventorySlot {
    /// Decodes the representation shared by the protocol and current database schema.
    pub fn from_raw(raw: u8) -> Result<Self, InvalidInventorySlot> {
        if raw < EQUIPMENT_SLOT_COUNT {
            return Ok(Self::Equipment(
                raw.try_into().expect("every equipment slot value has a definition"),
            ));
        }

        BagSlot::from_raw(raw)
            .map(Self::Bag)
            .ok_or(InvalidInventorySlot::ReservedOrInvalid { raw })
    }

    /// Encodes this slot for the protocol or current database schema.
    pub const fn as_raw(self) -> u8 {
        match self {
            Self::Equipment(slot) => slot as u8,
            Self::Bag(slot) => slot.as_raw(),
        }
    }
}

impl From<EquipmentSlot> for MainInventorySlot {
    fn from(slot: EquipmentSlot) -> Self {
        Self::Equipment(slot)
    }
}

impl From<MainInventorySlot> for u8 {
    fn from(slot: MainInventorySlot) -> Self {
        slot.as_raw()
    }
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum InvalidInventorySlot {
    #[error("raw inventory slot {raw:#04X} is reserved or invalid")]
    ReservedOrInvalid { raw: u8 },
    #[error("inventory slot {raw:#04X} is outside inventory size {size}")]
    OutsideInventory { raw: u8, size: usize },
}

/// Whether an item kind is allowed in a specific equipment slot.
pub fn item_type_matches_equipment_slot(slot: EquipmentSlot, object_type: ObjectType) -> bool {
    let ObjectType::Item(item) = object_type else {
        return false;
    };
    match item {
        ObjectItem::Equippable(equipment) => match equipment {
            ObjectEquippable::Clothing(_, part) => match part {
                ObjectClothingPart::Head => slot == EquipmentSlot::HeadArmor,
                ObjectClothingPart::Shoulder => slot == EquipmentSlot::ShoulderArmor,
                ObjectClothingPart::Body => slot == EquipmentSlot::ChestArmor,
                ObjectClothingPart::Leg => slot == EquipmentSlot::LegArmor,
                ObjectClothingPart::Arm => slot == EquipmentSlot::WristArmor,
                ObjectClothingPart::Foot => slot == EquipmentSlot::FootArmor,
                ObjectClothingPart::Any => false,
            },
            ObjectEquippable::Shield(_) => slot == EquipmentSlot::SecondaryWeapon,
            ObjectEquippable::Jewelry(_, kind) => match kind {
                ObjectJewelryType::Earring => slot == EquipmentSlot::Earring,
                ObjectJewelryType::Necklace => slot == EquipmentSlot::Necklace,
                ObjectJewelryType::Ring => slot == EquipmentSlot::LeftRing || slot == EquipmentSlot::RightRing,
            },
            ObjectEquippable::Weapon(_) => slot == EquipmentSlot::Weapon,
            _ => false,
        },
        ObjectItem::Consumable(ObjectConsumable::Ammo(_)) => slot == EquipmentSlot::SecondaryWeapon,
        _ => false,
    }
}

/// Whether an item kind may be used by a character race.
pub fn item_type_matches_race(user_race: Race, object_type: ObjectType) -> bool {
    let ObjectType::Item(item) = object_type else {
        return false;
    };
    match item {
        ObjectItem::Equippable(equipment) => match equipment {
            ObjectEquippable::Clothing(kind, _) => match kind {
                ObjectClothingType::Garment | ObjectClothingType::Protector | ObjectClothingType::Armor => {
                    user_race == Race::Chinese
                },
                ObjectClothingType::Robe | ObjectClothingType::LightArmor | ObjectClothingType::HeavyArmor => {
                    user_race == Race::European
                },
            },
            ObjectEquippable::Shield(race) | ObjectEquippable::Jewelry(race, _) => match race {
                ObjectRace::Chinese => user_race == Race::Chinese,
                ObjectRace::European => user_race == Race::European,
            },
            ObjectEquippable::Weapon(kind) => match kind {
                ObjectWeaponType::Sword
                | ObjectWeaponType::Blade
                | ObjectWeaponType::Spear
                | ObjectWeaponType::Glavie
                | ObjectWeaponType::Bow => user_race == Race::Chinese,
                ObjectWeaponType::OneHandSword
                | ObjectWeaponType::TwoHandSword
                | ObjectWeaponType::Axe
                | ObjectWeaponType::WarlockStaff
                | ObjectWeaponType::Staff
                | ObjectWeaponType::Crossbow
                | ObjectWeaponType::Dagger
                | ObjectWeaponType::Harp
                | ObjectWeaponType::ClericRod => user_race == Race::European,
                _ => false,
            },
            _ => false,
        },
        ObjectItem::Consumable(ObjectConsumable::Ammo(kind)) => match kind {
            ObjectConsumableAmmo::Arrows => user_race == Race::Chinese,
            ObjectConsumableAmmo::Bolts => user_race == Race::European,
        },
        _ => false,
    }
}

#[derive(Copy, Clone)]
pub struct Item {
    pub reference: &'static RefItemData,
    pub variance: Option<u64>,
    pub type_data: ItemTypeData,
}

impl Item {
    pub fn stack_size(&self) -> u16 {
        match &self.type_data {
            ItemTypeData::Consumable { amount, .. } => *amount,
            _ => 1,
        }
    }

    pub fn is_max_stacked(&self) -> bool {
        self.stack_size() >= self.reference.max_stack_size
    }

    pub fn upgrade_level(&self) -> u8 {
        match &self.type_data {
            ItemTypeData::Equipment { upgrade_level } => *upgrade_level,
            _ => 0,
        }
    }

    pub fn change_stack_size(&mut self, amount: i16) -> Result<(), MoveError> {
        self.type_data = match self.type_data {
            ItemTypeData::Consumable { amount: old_amount } => {
                // This is annoying because checked_add_signed is still in nightly
                if amount < 0 {
                    let to_subtract = amount.unsigned_abs();
                    if to_subtract > old_amount {
                        return Err(MoveError::Impossible);
                    } else {
                        ItemTypeData::Consumable {
                            amount: old_amount - to_subtract,
                        }
                    }
                } else {
                    ItemTypeData::Consumable {
                        amount: old_amount.checked_add(amount as u16).ok_or(MoveError::Impossible)?,
                    }
                }
            },
            _ => return Err(MoveError::NotStackable),
        };
        Ok(())
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ItemTypeData {
    Equipment { upgrade_level: u8 },
    COS,
    Consumable { amount: u16 },
    Gold { amount: u32 },
}

impl ItemTypeData {
    pub fn upgrade_level(&self) -> Option<u8> {
        match self {
            ItemTypeData::Equipment { upgrade_level } => Some(*upgrade_level),
            _ => None,
        }
    }

    pub fn amount(&self) -> u32 {
        match self {
            ItemTypeData::Consumable { amount } => u32::from(*amount),
            ItemTypeData::Gold { amount } => *amount,
            _ => 1,
        }
    }
}

pub enum InventoryChange {
    AddItem {
        slot: MainInventorySlot,
        item: Item,
    },
    ChangeTypeData {
        slot: MainInventorySlot,
        old_item: ItemTypeData,
        new_item: ItemTypeData,
    },
    MoveItem {
        source_slot: MainInventorySlot,
        target_slot: MainInventorySlot,
    },
    RemoveItem {
        slot: MainInventorySlot,
    },
    Swap {
        first_slot: MainInventorySlot,
        second_slot: MainInventorySlot,
    },
}

impl Change for InventoryChange {
    fn merge(self, other: Self) -> MergeResult<InventoryChange> {
        match &self {
            InventoryChange::AddItem { slot, item } => match &other {
                InventoryChange::AddItem {
                    slot: new_slot,
                    item: new_item,
                } if *slot == *new_slot => {
                    // This should actually never happen, because it would mean the item got overwritten.
                    MergeResult::Merged(other)
                },
                InventoryChange::MoveItem {
                    source_slot,
                    target_slot,
                } if *source_slot == *slot => MergeResult::Merged(InventoryChange::AddItem {
                    item: *item,
                    slot: *target_slot,
                }),
                InventoryChange::RemoveItem { slot: removed_slot } if *slot == *removed_slot => MergeResult::Cancelled,
                InventoryChange::ChangeTypeData {
                    slot: changed_slot,
                    old_item,
                    new_item,
                } if *slot == *changed_slot && item.type_data == *old_item => {
                    MergeResult::Merged(InventoryChange::AddItem {
                        slot: *slot,
                        item: Item {
                            reference: item.reference,
                            variance: item.variance,
                            type_data: *new_item,
                        },
                    })
                },
                InventoryChange::Swap {
                    first_slot,
                    second_slot,
                } if *first_slot == *slot || *second_slot == *slot => {
                    // The swap operation cannot really be merged with anything
                    // else and thus we want to prevent more checks by making
                    // them incompatible.
                    MergeResult::Incompatible(self, other)
                },
                _ => MergeResult::Unchanged(self, other),
            },
            InventoryChange::RemoveItem { slot } => match &other {
                InventoryChange::AddItem { slot: new_slot, .. } if *slot == *new_slot => {
                    // We _could_ handle this better if we knew the old item by checking if it matches
                    // but we won't bother for now
                    MergeResult::Incompatible(self, other)
                },
                InventoryChange::RemoveItem { slot: other_slot } if *slot == *other_slot => {
                    // This should never happen - just like two items added on the same slot.
                    // We should've have either encountered an add or a move before
                    MergeResult::Incompatible(self, other)
                },
                InventoryChange::MoveItem { source_slot, .. } if *slot == *source_slot => {
                    // Again something that should never happen - we cannot move an empty slot.
                    MergeResult::Incompatible(self, other)
                },
                InventoryChange::ChangeTypeData { slot: changed_slot, .. } if *slot == *changed_slot => {
                    // Again something that should never happen - we cannot change an empty slot.
                    MergeResult::Incompatible(self, other)
                },
                InventoryChange::Swap {
                    first_slot,
                    second_slot,
                } if *first_slot == *slot || *second_slot == *slot => {
                    // see above.
                    MergeResult::Incompatible(self, other)
                },
                _ => MergeResult::Unchanged(self, other),
            },
            InventoryChange::MoveItem {
                source_slot,
                target_slot,
            } => match &other {
                InventoryChange::AddItem { slot, .. } if *target_slot == *slot => {
                    // Should never happen
                    MergeResult::Incompatible(self, other)
                },
                InventoryChange::MoveItem {
                    source_slot: new_source,
                    target_slot: new_target,
                } if *target_slot == *new_source => MergeResult::Merged(InventoryChange::MoveItem {
                    source_slot: *source_slot,
                    target_slot: *new_target,
                }),
                InventoryChange::RemoveItem { slot } if *slot == *target_slot => {
                    MergeResult::Merged(InventoryChange::RemoveItem { slot: *source_slot })
                },
                InventoryChange::Swap {
                    first_slot,
                    second_slot,
                } if *second_slot == *target_slot
                    || *first_slot == *target_slot
                    || *second_slot == *source_slot
                    || *first_slot == *source_slot =>
                {
                    // We cannot merge a move followed by a swap.
                    MergeResult::Incompatible(self, other)
                },
                _ => MergeResult::Unchanged(self, other),
            },
            InventoryChange::ChangeTypeData {
                slot,
                new_item,
                old_item,
            } => match &other {
                InventoryChange::ChangeTypeData {
                    slot: other_slot,
                    new_item: changed_new,
                    old_item: changed_old,
                } if *slot == *other_slot => {
                    if *new_item == *changed_old {
                        MergeResult::Merged(InventoryChange::ChangeTypeData {
                            slot: *slot,
                            old_item: *old_item,
                            new_item: *changed_new,
                        })
                    } else {
                        MergeResult::Incompatible(self, other)
                    }
                },
                InventoryChange::MoveItem { source_slot, .. } if *source_slot == *slot => {
                    MergeResult::Incompatible(self, other)
                },
                InventoryChange::RemoveItem { slot: removed_slot } if *slot == *removed_slot => {
                    MergeResult::Merged(other)
                },
                InventoryChange::Swap {
                    first_slot,
                    second_slot,
                } if *first_slot == *slot || *second_slot == *slot => {
                    // see above
                    MergeResult::Incompatible(self, other)
                },
                _ => MergeResult::Unchanged(self, other),
            },
            InventoryChange::Swap {
                first_slot,
                second_slot,
            } => match &other {
                InventoryChange::AddItem { slot, .. }
                | InventoryChange::ChangeTypeData { slot, .. }
                | InventoryChange::RemoveItem { slot, .. }
                    if *slot == *first_slot || *slot == *second_slot =>
                {
                    // This would mean the item would be overwritten - shouldn't happen
                    MergeResult::Incompatible(self, other)
                },
                InventoryChange::MoveItem {
                    source_slot,
                    target_slot,
                } if *first_slot == *source_slot
                    || *first_slot == *target_slot
                    || *second_slot == *source_slot
                    || *second_slot == *target_slot =>
                {
                    MergeResult::Incompatible(self, other)
                },
                InventoryChange::Swap {
                    first_slot: other_first_slot,
                    second_slot: other_second_slot,
                } => {
                    if (*first_slot == *other_first_slot && *second_slot == *other_first_slot)
                        || (*first_slot == *other_second_slot && *second_slot == *other_first_slot)
                    {
                        MergeResult::Cancelled
                    } else if *first_slot == *other_first_slot
                        || *first_slot == *other_second_slot
                        || *second_slot == *other_first_slot
                        || *second_slot == *other_second_slot
                    {
                        MergeResult::Incompatible(self, other)
                    } else {
                        MergeResult::Unchanged(self, other)
                    }
                },
                _ => MergeResult::Unchanged(self, other),
            },
        }
    }
}

pub struct Inventory {
    size: usize,
    // TODO: wouldn't this make more sense as an array of N size?
    items: HashMap<MainInventorySlot, Item>,
    changes: Vec<InventoryChange>,
}

impl Inventory {
    pub fn new(size: usize) -> Self {
        assert!(
            size > usize::from(EQUIPMENT_SLOT_COUNT),
            "inventory must contain at least one bag slot"
        );
        assert!(
            size <= usize::from(LAST_ITEM_SLOT) + 1,
            "inventory is too large for the protocol"
        );
        Inventory {
            size,
            items: HashMap::new(),
            changes: Vec::new(),
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    /// Decodes and validates a protocol or persistence slot for this inventory.
    pub fn slot_from_raw(&self, raw: u8) -> Result<MainInventorySlot, InvalidInventorySlot> {
        let slot = MainInventorySlot::from_raw(raw)?;
        self.ensure_slot(slot)?;
        Ok(slot)
    }

    fn ensure_slot(&self, slot: MainInventorySlot) -> Result<(), InvalidInventorySlot> {
        let raw = slot.as_raw();
        if usize::from(raw) < self.size {
            Ok(())
        } else {
            Err(InvalidInventorySlot::OutsideInventory { raw, size: self.size })
        }
    }

    pub fn get_item_at(&self, slot: MainInventorySlot) -> Option<&Item> {
        self.items.get(&slot)
    }

    pub fn equipment_items(&self) -> impl Iterator<Item = (EquipmentSlot, &Item)> {
        self.items.iter().filter_map(|(slot, item)| match slot {
            MainInventorySlot::Equipment(slot) => Some((*slot, item)),
            MainInventorySlot::Bag(_) => None,
        })
    }

    pub fn items(&self) -> impl Iterator<Item = (MainInventorySlot, &Item)> {
        self.items.iter().map(|(slot, item)| (*slot, item))
    }

    pub fn weapon(&self) -> Option<&Item> {
        self.get_equipment_item(EquipmentSlot::Weapon)
    }

    fn bag_slots(&self) -> impl Iterator<Item = MainInventorySlot> {
        (FIRST_BAG_SLOT..self.size as u8)
            .map(|raw| MainInventorySlot::from_raw(raw).expect("inventory size excludes reserved protocol slots"))
    }

    fn empty_slot(&self) -> Option<MainInventorySlot> {
        self.bag_slots().find(|slot| !self.items.contains_key(slot))
    }

    pub fn move_item(
        &mut self,
        source: MainInventorySlot,
        target: MainInventorySlot,
        amount: u16,
    ) -> Result<u16, MoveError> {
        self.ensure_slot(source)?;
        self.ensure_slot(target)?;

        if let Some(mut source_item) = self.items.remove(&source) {
            if let Some(mut target_item) = self.items.remove(&target) {
                if source_item.reference.ref_id() == target_item.reference.ref_id()
                    && source_item.reference.max_stack_size > 1
                {
                    let available_on_target_stack = target_item.reference.max_stack_size - target_item.stack_size();
                    if available_on_target_stack == 0 {
                        self.items.insert(source, source_item);
                        self.items.insert(target, target_item);
                        return Ok(0);
                    }

                    let moved = amount.min(available_on_target_stack).min(source_item.stack_size());
                    let old_type_data = target_item.type_data;
                    target_item.change_stack_size(i16::try_from(moved).map_err(|_| MoveError::Impossible)?)?;
                    let new_type_data = target_item.type_data;
                    if moved == source_item.stack_size() {
                        self.changes.push(InventoryChange::RemoveItem { slot: source });
                    } else {
                        let old_data = source_item.type_data;
                        source_item.change_stack_size(-i16::try_from(moved).map_err(|_| MoveError::Impossible)?)?;
                        let new_data = source_item.type_data;
                        self.changes.push(InventoryChange::ChangeTypeData {
                            slot: source,
                            old_item: old_data,
                            new_item: new_data,
                        });
                        self.items.insert(source, source_item);
                    }
                    self.changes.push(InventoryChange::ChangeTypeData {
                        slot: target,
                        old_item: old_type_data,
                        new_item: new_type_data,
                    });
                    self.items.insert(target, target_item);
                    return Ok(moved);
                } else {
                    self.changes.push(InventoryChange::Swap {
                        first_slot: source,
                        second_slot: target,
                    });
                    self.items.insert(target, source_item);
                    self.items.insert(source, target_item);
                }
            } else {
                self.changes.push(InventoryChange::MoveItem {
                    source_slot: source,
                    target_slot: target,
                });
                self.items.insert(target, source_item);
            }
        } else {
            return Err(MoveError::ItemDoesNotExist);
        }

        Ok(amount)
    }

    pub fn get_equipment_item(&self, slot: EquipmentSlot) -> Option<&Item> {
        self.items.get(&slot.into())
    }

    pub fn set_item(&mut self, slot: MainInventorySlot, item: Item) -> Result<(), InvalidInventorySlot> {
        self.ensure_slot(slot)?;
        self.items.insert(slot, item);
        Ok(())
    }

    fn find_slots_matching(&self, item: Item) -> impl Iterator<Item = MainInventorySlot> + '_ {
        self.items
            .iter()
            .filter(move |(_, existing)| existing.reference == item.reference && existing.variance == item.variance)
            .map(|(slot, _)| slot)
            .copied()
    }

    pub fn add_item(&mut self, mut item: Item) -> Option<MainInventorySlot> {
        if item.reference.max_stack_size > 1 {
            for i in self.find_slots_matching(item).collect::<Vec<_>>() {
                let free_slot = self.empty_slot();
                let existing = self.items.get_mut(&i).expect("The matching slot should have an item");
                if !existing.is_max_stacked() && existing.reference.ref_id() == item.reference.ref_id() {
                    return match (existing.type_data, item.type_data) {
                        (
                            ItemTypeData::Consumable {
                                amount: existing_amount,
                            },
                            ItemTypeData::Consumable { amount: added_amount },
                        ) => {
                            let sum_amount = existing_amount.checked_add(added_amount)?;
                            if sum_amount <= item.reference.max_stack_size {
                                let old_data = existing.type_data;
                                let new_data = ItemTypeData::Consumable { amount: sum_amount };
                                existing.type_data = new_data;
                                self.changes.push(InventoryChange::ChangeTypeData {
                                    slot: i,
                                    old_item: old_data,
                                    new_item: new_data,
                                });
                                Some(i)
                            } else {
                                let remaining = sum_amount - item.reference.max_stack_size;
                                if let Some(free_slot) = free_slot {
                                    let old_data = existing.type_data;
                                    let new_data = ItemTypeData::Consumable {
                                        amount: item.reference.max_stack_size,
                                    };
                                    existing.type_data = new_data;
                                    self.changes.push(InventoryChange::ChangeTypeData {
                                        slot: i,
                                        old_item: old_data,
                                        new_item: new_data,
                                    });
                                    item.type_data = ItemTypeData::Consumable { amount: remaining };
                                    self.set_item(free_slot, item)
                                        .expect("empty slots are valid for this inventory");
                                    self.changes.push(InventoryChange::AddItem { slot: free_slot, item });
                                    Some(free_slot)
                                } else {
                                    None
                                }
                            }
                        },
                        _ => {
                            // This *should* not happen. If two ref_ids match, they should have the same type data!
                            None
                        },
                    };
                }
            }
        }

        if let Some(free_slot) = self.empty_slot() {
            self.items.insert(free_slot, item);
            self.changes.push(InventoryChange::AddItem { slot: free_slot, item });
            return Some(free_slot);
        }
        None
    }

    pub fn remove_item(&mut self, item: Item) -> Result<u16, MoveError> {
        let mut to_remove = item.stack_size();
        let mut removed = 0;
        for i in self.find_slots_matching(item).collect::<Vec<_>>() {
            let existing = self
                .items
                .get_mut(&i)
                .expect("Item should still exist just after checking");
            if to_remove > 1 {
                if existing.stack_size() > to_remove {
                    let old_data = existing.type_data;
                    existing.change_stack_size(-(to_remove as i16))?;
                    self.changes.push(InventoryChange::ChangeTypeData {
                        slot: i,
                        old_item: old_data,
                        new_item: existing.type_data,
                    });
                    removed += to_remove;
                    to_remove = 0;
                    break;
                } else {
                    removed += existing.stack_size();
                    to_remove = to_remove.saturating_sub(existing.stack_size());
                    self.items.remove(&i);
                    self.changes.push(InventoryChange::RemoveItem { slot: i });
                }
            } else {
                self.items.remove(&i);
                self.changes.push(InventoryChange::RemoveItem { slot: i });
                removed += 1;
                break;
            }
        }
        Ok(removed)
    }
}

impl ChangeTracked for Inventory {
    type ChangeItem = InventoryChange;

    fn changes(&mut self) -> Vec<InventoryChange> {
        std::mem::take(&mut self.changes)
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Inventory::new(45)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MoveError {
    #[error("item does not exist")]
    ItemDoesNotExist,
    #[error("item is not stackable")]
    NotStackable,
    #[error("inventory operation is impossible")]
    Impossible,
    #[error(transparent)]
    InvalidSlot(#[from] InvalidInventorySlot),
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ToOptimizedChange;
    use once_cell::sync::Lazy;
    use silkroad_data::common::{RefCommon, RefOrigin};
    use silkroad_data::itemdata::RefBiologicalType;
    use silkroad_definitions::type_id::{ObjectConsumable, ObjectConsumableRecovery, ObjectItem, ObjectType};
    use std::ops::Deref;

    static FIRST_ITEM_DATA: Lazy<RefItemData> = Lazy::new(|| RefItemData {
        common: RefCommon {
            service: true,
            ref_id: 1,
            id: "TestItem".to_string(),
            type_id: ObjectType::Item(ObjectItem::Consumable(ObjectConsumable::Recovery(
                ObjectConsumableRecovery::HP,
            )))
            .type_id(),
            country: RefOrigin::Chinese,
            despawn_time: Default::default(),
        },
        price: 100,
        rarity: silkroad_data::itemdata::RefItemRarity::General,
        can_drop: true,
        max_stack_size: 50,
        range: None,
        required_level: None,
        biological_type: RefBiologicalType::Both,
        params: [0, 0, 0, 0],
    });

    static SECOND_ITEM_DATA: Lazy<RefItemData> = Lazy::new(|| RefItemData {
        common: RefCommon {
            service: true,
            ref_id: 2,
            id: "TestItem2".to_string(),
            type_id: ObjectType::Item(ObjectItem::Consumable(ObjectConsumable::Recovery(
                ObjectConsumableRecovery::HP,
            )))
            .type_id(),
            country: RefOrigin::Chinese,
            despawn_time: Default::default(),
        },
        price: 100,
        rarity: silkroad_data::itemdata::RefItemRarity::General,
        can_drop: true,
        max_stack_size: 50,
        range: None,
        required_level: None,
        biological_type: RefBiologicalType::Both,
        params: [0, 0, 0, 0],
    });

    fn bag(index: u8) -> MainInventorySlot {
        MainInventorySlot::Bag(BagSlot::new(index).unwrap())
    }

    #[test]
    fn main_inventory_slots_translate_at_the_raw_seam() {
        assert_eq!(
            MainInventorySlot::from_raw(0).unwrap(),
            MainInventorySlot::Equipment(EquipmentSlot::HeadArmor)
        );
        assert_eq!(
            MainInventorySlot::from_raw(2).unwrap(),
            MainInventorySlot::Equipment(EquipmentSlot::ChestArmor)
        );
        assert_eq!(MainInventorySlot::from_raw(13).unwrap(), bag(0));
        assert_eq!(MainInventorySlot::from_raw(45).unwrap(), bag(32));
        assert_eq!(u8::from(bag(0)), 13);
        assert_eq!(u8::from(bag(32)), 45);
        assert_eq!(
            MainInventorySlot::from_raw(0xFE),
            Err(InvalidInventorySlot::ReservedOrInvalid { raw: 0xFE })
        );
        assert_eq!(
            MainInventorySlot::from_raw(0xFF),
            Err(InvalidInventorySlot::ReservedOrInvalid { raw: 0xFF })
        );
    }

    #[test]
    fn inventory_validates_slots_against_its_size() {
        let inventory = Inventory::new(14);
        assert_eq!(inventory.slot_from_raw(13), Ok(bag(0)));
        assert_eq!(
            inventory.slot_from_raw(14),
            Err(InvalidInventorySlot::OutsideInventory { raw: 14, size: 14 })
        );
    }

    #[test]
    pub fn simple_inventory_tracking() {
        let mut inv = Inventory::default();

        let reference = FIRST_ITEM_DATA.deref();
        let slot = inv
            .add_item(Item {
                variance: None,
                reference,
                type_data: ItemTypeData::Consumable { amount: 5 },
            })
            .unwrap();

        let other_slot = inv
            .add_item(Item {
                variance: None,
                reference,
                type_data: ItemTypeData::Consumable { amount: 5 },
            })
            .unwrap();

        assert_eq!(slot, other_slot);
        let changes = inv.changes();
        assert_eq!(2, changes.len());
        let mut optimized = changes.optimize();
        assert_eq!(1, optimized.len());
        assert!(matches!(
            optimized.pop().unwrap(),
            InventoryChange::AddItem { slot: changed_slot, .. } if changed_slot == slot
        ));
    }

    #[test]
    pub fn test_different_items() {
        let mut inv = Inventory::default();

        let first_item = FIRST_ITEM_DATA.deref();
        let second_item = SECOND_ITEM_DATA.deref();

        let first_slot = inv
            .add_item(Item {
                reference: first_item,
                variance: None,
                type_data: ItemTypeData::Consumable { amount: 5 },
            })
            .unwrap();
        let second_slot = inv
            .add_item(Item {
                reference: second_item,
                variance: None,
                type_data: ItemTypeData::Consumable { amount: 5 },
            })
            .unwrap();
        assert_ne!(first_slot, second_slot);

        let changes = inv.changes();
        assert_eq!(2, changes.len());
        let optimized = changes.optimize();
        assert_eq!(2, optimized.len());
    }

    #[test]
    fn inventory_regressions() {
        let reference = FIRST_ITEM_DATA.deref();
        let item = |amount| Item {
            variance: None,
            reference,
            type_data: ItemTypeData::Consumable { amount },
        };

        let mut inv = Inventory::default();
        inv.set_item(bag(0), item(10)).unwrap();
        inv.set_item(bag(1), item(45)).unwrap();
        assert_eq!(5, inv.move_item(bag(0), bag(1), 5).unwrap());
        assert_eq!(5, inv.get_item_at(bag(0)).unwrap().stack_size());

        let mut full = Inventory::new(14);
        full.set_item(bag(0), item(45)).unwrap();
        assert_eq!(None, full.add_item(item(10)));
        assert_eq!(45, full.get_item_at(bag(0)).unwrap().stack_size());
        assert!(full.changes().is_empty());

        let mut overflow = item(u16::MAX);
        assert!(overflow.change_stack_size(1).is_err());
        assert_eq!(u16::MAX, overflow.stack_size());

        let mut remove = Inventory::default();
        remove.set_item(bag(0), item(10)).unwrap();
        remove.remove_item(item(4)).unwrap();
        assert_eq!(6, remove.get_item_at(bag(0)).unwrap().stack_size());
        assert!(matches!(
            remove.changes().as_slice(),
            [InventoryChange::ChangeTypeData { slot, .. }] if *slot == bag(0)
        ));
    }

    #[test]
    pub fn test_remove_item() {
        let mut inv = Inventory::default();

        let item_ref = FIRST_ITEM_DATA.deref();
        let item = Item {
            variance: None,
            reference: item_ref,
            type_data: ItemTypeData::Consumable { amount: 5 },
        };
        let slot = inv.add_item(item).unwrap();
        let _ = inv.changes(); // consume the changes
        inv.remove_item(item).unwrap();
        assert_eq!(0, inv.items().count());
        let mut changes = inv.changes();
        assert_eq!(1, changes.len());
        assert!(matches!(
            changes.pop().unwrap(),
            InventoryChange::RemoveItem { slot: removed_slot } if removed_slot == slot
        ));
    }
}
