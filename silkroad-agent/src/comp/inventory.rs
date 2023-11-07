use crate::db::character::CharacterItem;
use crate::sync::Reset;
use crate::world::WorldData;
use bevy_ecs::prelude::*;
use silkroad_data::itemdata::RefItemData;
use silkroad_definitions::type_id::{ObjectItem, ObjectType};
use silkroad_game_base::{ChangeProvided, Inventory, Item, ItemTypeData};
use std::ops::{Deref, DerefMut};

#[derive(Component)]
pub(crate) struct PlayerInventory {
    inventory: Inventory,
    gold: u64,
    gold_changed: bool,
}

impl PlayerInventory {
    pub(crate) fn has_changed_gold(&self) -> bool {
        self.gold_changed
    }

    pub(crate) fn gold(&self) -> u64 {
        self.gold
    }
}

impl Deref for PlayerInventory {
    type Target = Inventory;

    fn deref(&self) -> &Self::Target {
        &self.inventory
    }
}

impl DerefMut for PlayerInventory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inventory
    }
}

impl Reset for PlayerInventory {
    fn reset(&mut self) {
        self.gold_changed = false;
    }
}

#[derive(Copy, Clone)]
pub(crate) struct GoldChange(pub u64);

impl ChangeProvided for PlayerInventory {
    type Change = GoldChange;

    fn as_change(&self) -> Self::Change {
        GoldChange(self.gold)
    }
}

impl PlayerInventory {
    fn from_db_inventory(items: &[CharacterItem], size: usize) -> Inventory {
        let item_map = WorldData::items();
        let mut inventory = Inventory::new(size);

        for item in items {
            let item_def = item_map.find_id(item.item_obj_id as u32).unwrap();

            inventory.set_item(
                item.slot as u8,
                Item {
                    reference: item_def,
                    variance: item.variance.map(|v| v as u64),
                    type_data: Self::item_type_data_for(item_def, item).unwrap(),
                },
            );
        }

        inventory
    }

    fn item_type_data_for(ref_data: &RefItemData, item: &CharacterItem) -> Option<ItemTypeData> {
        let obj_type = ObjectType::from_type_id(&ref_data.common.type_id).unwrap();
        if let ObjectType::Item(item_type) = obj_type {
            let res = match item_type {
                ObjectItem::Equippable(_) => ItemTypeData::Equipment {
                    upgrade_level: item.upgrade_level as u8,
                },
                ObjectItem::Pet(_) => ItemTypeData::COS,
                _ => ItemTypeData::Consumable {
                    amount: item.amount as u16,
                },
            };
            Some(res)
        } else {
            None
        }
    }

    pub(crate) fn from_db(items: &[CharacterItem], size: usize, gold: u64) -> Self {
        let inventory = Self::from_db_inventory(items, size);
        PlayerInventory {
            inventory,
            gold,
            gold_changed: false,
        }
    }

    pub fn gain_gold(&mut self, amount: u64) {
        self.gold = self.gold.saturating_add(amount);
        self.gold_changed = true;
    }

    pub fn spend_gold(&mut self, amount: u64) {
        self.gold = self.gold.saturating_sub(amount);
        self.gold_changed = true;
    }
}
