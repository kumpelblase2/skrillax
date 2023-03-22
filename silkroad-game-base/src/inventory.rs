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
}

#[derive(Copy, Clone)]
pub enum ItemTypeData {
    Equipment { upgrade_level: u8 },
    COS,
    Consumable { amount: u16 },
    Gold { amount: u32 },
}

pub struct Inventory {
    size: usize,
    items: HashMap<u8, Item>, // TODO: wouldn't this make more sense as an array of N size?
}

impl Inventory {
    pub fn new(size: usize) -> Self {
        assert!(size > 0xC, "Minimum Inventory size is 12");
        Inventory {
            size,
            items: HashMap::new(),
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

    fn update_stack_size(item: &mut Item, amount: i16) -> Result<(), MoveError> {
        item.type_data = match item.type_data {
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

    pub fn move_item(&mut self, source: u8, target: u8, amount: u16) -> Result<u16, MoveError> {
        if let Some(mut source_item) = self.items.remove(&source) {
            if let Some(mut target_item) = self.items.remove(&target) {
                if source_item.reference.ref_id() == target_item.reference.ref_id()
                    && source_item.reference.max_stack_size > 1
                {
                    let available_on_target_stack = target_item.reference.max_stack_size - target_item.stack_size();
                    if available_on_target_stack == 0 {
                        return Ok(0);
                    }

                    if available_on_target_stack >= amount {
                        Self::update_stack_size(&mut target_item, amount as i16)?;
                        self.items.insert(target, target_item);
                    } else {
                        Self::update_stack_size(&mut target_item, available_on_target_stack as i16)?;
                        Self::update_stack_size(&mut source_item, -(available_on_target_stack as i16))?;

                        self.items.insert(source, source_item);
                        self.items.insert(target, target_item);
                        return Ok(available_on_target_stack);
                    }
                } else {
                    self.items.insert(target, source_item);
                    self.items.insert(source, target_item);
                }
            } else {
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

    pub fn add_item(&mut self, mut item: Item) -> Option<u8> {
        if item.reference.max_stack_size > 1 {
            for i in self.non_equipment_slots() {
                if let Some(mut existing) = self.items.get_mut(&i) {
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
                                    existing.type_data = ItemTypeData::Consumable { amount: sum_amount };
                                    Some(i)
                                } else {
                                    existing.type_data = ItemTypeData::Consumable {
                                        amount: item.reference.max_stack_size,
                                    };
                                    let remaining = sum_amount - item.reference.max_stack_size;
                                    if let Some(free_slot) = self.empty_slot() {
                                        item.type_data = ItemTypeData::Consumable { amount: remaining };
                                        self.set_item(free_slot, item);
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
        }

        if let Some(free_slot) = self.empty_slot() {
            self.items.insert(free_slot, item);
            return Some(free_slot);
        }
        None
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Inventory::new(45)
    }
}

pub enum MoveError {
    ItemDoesNotExist,
    NotStackable,
    Impossible,
}
