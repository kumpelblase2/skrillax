use crate::{Change, ChangeTracked, MergeResult};
use silkroad_data::itemdata::RefItemData;
use silkroad_data::DataEntry;
use std::collections::hash_map::Iter;
use std::collections::HashMap;

pub const WEAPON_SLOT: u8 = 6;
pub const GOLD_SLOT: u8 = 0xFE;

#[derive(Copy, Clone, Eq, PartialOrd, PartialEq)]
pub enum EquipmentSlot {
    HeadArmor,
    ShoulderArmor,
    WristArmor,
    ChestArmor,
    LegArmor,
    FootArmor,
    Weapon,
    SecondaryWeapon,
    Earring,
    Necklace,
    LeftRing,
    RightRing,
    Special,
}

impl From<EquipmentSlot> for u8 {
    fn from(slot: EquipmentSlot) -> u8 {
        match slot {
            EquipmentSlot::Weapon => WEAPON_SLOT,
            EquipmentSlot::SecondaryWeapon => 7,
            EquipmentSlot::HeadArmor => 0,
            EquipmentSlot::ShoulderArmor => 1,
            EquipmentSlot::WristArmor => 3,
            EquipmentSlot::ChestArmor => 2,
            EquipmentSlot::LegArmor => 4,
            EquipmentSlot::FootArmor => 5,
            EquipmentSlot::Earring => 8,
            EquipmentSlot::Necklace => 9,
            EquipmentSlot::LeftRing => 10,
            EquipmentSlot::RightRing => 11,
            EquipmentSlot::Special => 12,
        }
    }
}

impl TryFrom<u8> for EquipmentSlot {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let value = match value {
            0 => EquipmentSlot::HeadArmor,
            1 => EquipmentSlot::ShoulderArmor,
            2 => EquipmentSlot::ChestArmor,
            3 => EquipmentSlot::WristArmor,
            4 => EquipmentSlot::LegArmor,
            5 => EquipmentSlot::FootArmor,
            6 => EquipmentSlot::Weapon,
            7 => EquipmentSlot::SecondaryWeapon,
            8 => EquipmentSlot::Earring,
            9 => EquipmentSlot::Necklace,
            10 => EquipmentSlot::LeftRing,
            11 => EquipmentSlot::RightRing,
            12 => EquipmentSlot::Special,
            _ => return Err(()),
        };
        Ok(value)
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
                        amount: old_amount + amount as u16,
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
    fn merge(self, other: Self) -> Result<Self, (Self, Self)> {
        match &self {
            ItemTypeData::Consumable { amount } => match &other {
                ItemTypeData::Consumable { amount: new_amount } => Ok(ItemTypeData::Consumable {
                    amount: *amount + *new_amount,
                }),
                _ => Err((self, other)),
            },
            ItemTypeData::Gold { amount } => match &other {
                ItemTypeData::Gold { amount: new_amount } => Ok(ItemTypeData::Gold {
                    amount: *amount + *new_amount,
                }),
                _ => Err((self, other)),
            },
            _ => Err((self, other)),
        }
    }
}

pub enum InventoryChange {
    AddItem {
        slot: u8,
        item: Item,
    },
    ChangeTypeData {
        slot: u8,
        old_item: ItemTypeData,
        new_item: ItemTypeData,
    },
    MoveItem {
        source_slot: u8,
        target_slot: u8,
    },
    RemoveItem {
        slot: u8,
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
                InventoryChange::MoveItem {
                    source_slot,
                    target_slot,
                } if *source_slot == *slot => MergeResult::Incompatible(self, other),
                InventoryChange::RemoveItem { slot: removed_slot } if *slot == *removed_slot => {
                    MergeResult::Merged(other)
                },
                _ => MergeResult::Unchanged(self, other),
            },
        }
    }
}

pub struct Inventory {
    size: usize,
    // TODO: wouldn't this make more sense as an array of N size?
    items: HashMap<u8, Item>,
    changes: Vec<InventoryChange>,
}

impl Inventory {
    pub fn new(size: usize) -> Self {
        assert!(size > 0xC, "Minimum Inventory size is 12");
        Inventory {
            size,
            items: HashMap::new(),
            changes: Vec::new(),
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn get_item_at(&self, slot: u8) -> Option<&Item> {
        self.items.get(&slot)
    }

    pub fn equipment_items(&self) -> impl Iterator<Item = (&u8, &Item)> {
        self.items.iter().filter(|(index, _)| Self::is_equipment_slot(**index))
    }

    pub fn items(&self) -> Iter<u8, Item> {
        self.items.iter()
    }

    pub fn weapon(&self) -> Option<&Item> {
        self.items.get(&WEAPON_SLOT)
    }

    fn non_equipment_slots(&self) -> impl Iterator<Item = u8> {
        (0u8..(self.size as u8)).filter(|index| !Self::is_equipment_slot(*index))
    }

    fn empty_slot(&self) -> Option<u8> {
        self.non_equipment_slots().find(|slot| !self.items.contains_key(slot))
    }

    pub fn move_item(&mut self, source: u8, target: u8, amount: u16) -> Result<u16, MoveError> {
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

                    if available_on_target_stack >= amount {
                        let old_type_data = target_item.type_data;
                        target_item.change_stack_size(amount as i16)?;
                        let new_type_data = target_item.type_data;
                        self.changes.push(InventoryChange::RemoveItem { slot: source });
                        self.changes.push(InventoryChange::ChangeTypeData {
                            slot: target,
                            old_item: old_type_data,
                            new_item: new_type_data,
                        });
                        self.items.insert(target, target_item);
                    } else {
                        let old_data = target_item.type_data;
                        target_item.change_stack_size(available_on_target_stack as i16)?;
                        let new_data = target_item.type_data;
                        self.changes.push(InventoryChange::ChangeTypeData {
                            slot: target,
                            old_item: old_data,
                            new_item: new_data,
                        });
                        let old_data = source_item.type_data;
                        source_item.change_stack_size(-(available_on_target_stack as i16))?;
                        let new_data = source_item.type_data;
                        self.changes.push(InventoryChange::ChangeTypeData {
                            slot: source,
                            old_item: old_data,
                            new_item: new_data,
                        });

                        self.items.insert(source, source_item);
                        self.items.insert(target, target_item);
                        return Ok(available_on_target_stack);
                    }
                } else {
                    // TODO: how will I reflect this in the changes? we can't do two moves
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

    pub fn is_equipment_slot(slot: u8) -> bool {
        slot <= 0xCu8
    }

    pub fn get_equipment_item(&self, slot: EquipmentSlot) -> Option<&Item> {
        self.items.get(&slot.into())
    }

    pub fn set_item(&mut self, slot: u8, item: Item) {
        self.items.insert(slot, item);
    }

    fn find_slots_matching(&self, item: Item) -> impl Iterator<Item = u8> + '_ {
        self.items
            .iter()
            .filter(move |(slot, existing)| existing.reference == item.reference && existing.variance == item.variance)
            .map(|(slot, _)| slot)
            .copied()
    }

    pub fn add_item(&mut self, mut item: Item) -> Option<u8> {
        if item.reference.max_stack_size > 1 {
            for i in self.find_slots_matching(item).collect::<Vec<_>>() {
                let mut existing = self.items.get_mut(&i).expect("The matching slot should have an item");
                if !existing.is_max_stacked() && existing.reference.ref_id() == item.reference.ref_id() {
                    return match (existing.type_data, item.type_data) {
                        (
                            ItemTypeData::Consumable {
                                amount: existing_amount,
                            },
                            ItemTypeData::Consumable { amount: added_amount },
                        ) => {
                            let sum_amount = existing_amount + added_amount;
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
                                let remaining = sum_amount - item.reference.max_stack_size;
                                if let Some(free_slot) = self.empty_slot() {
                                    item.type_data = ItemTypeData::Consumable { amount: remaining };
                                    self.set_item(free_slot, item);
                                    self.changes.push(InventoryChange::AddItem { slot: free_slot, item });
                                    Some(free_slot)
                                } else {
                                    // We should possibly undo the previous change to update the size.
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
            let mut existing = self
                .items
                .get_mut(&i)
                .expect("Item should still exist just after checking");
            if to_remove > 1 {
                if existing.stack_size() > to_remove {
                    existing.change_stack_size(-(to_remove as i16))?;
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
                to_remove = 0;
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

#[derive(Debug)]
pub enum MoveError {
    ItemDoesNotExist,
    NotStackable,
    Impossible,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ToOptimizedChange;
    use once_cell::sync::Lazy;
    use silkroad_data::common::{RefCommon, RefOrigin};
    use silkroad_data::itemdata::RefBiologicalType;
    use silkroad_data::{ObjectConsumable, ObjectConsumableRecovery, ObjectItem, ObjectType};
    use std::ops::Deref;

    static FIRST_ITEM_DATA: Lazy<RefItemData> = Lazy::new(|| RefItemData {
        common: RefCommon {
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
        max_stack_size: 50,
        range: None,
        required_level: None,
        biological_type: RefBiologicalType::Both,
        params: [0, 0, 0, 0],
    });

    static SECOND_ITEM_DATA: Lazy<RefItemData> = Lazy::new(|| RefItemData {
        common: RefCommon {
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
        max_stack_size: 50,
        range: None,
        required_level: None,
        biological_type: RefBiologicalType::Both,
        params: [0, 0, 0, 0],
    });

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
            InventoryChange::AddItem { slot, .. }
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
        assert_eq!(0, inv.items().len());
        let mut changes = inv.changes();
        assert_eq!(1, changes.len());
        assert!(matches!(changes.pop().unwrap(), InventoryChange::RemoveItem { slot }));
    }
}
