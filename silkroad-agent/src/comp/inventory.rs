use crate::persistence::ApplyToDatabase;
use crate::world::WorldData;
use async_trait::async_trait;
use bevy::prelude::*;
use silkroad_agent_persistence::CharacterWorldItem;
use silkroad_data::itemdata::RefItemData;
use silkroad_definitions::type_id::{ObjectItem, ObjectType};
use silkroad_game_base::{ChangeTracked, Inventory, InventoryChange, Item, ItemTypeData};
use sqlx::PgPool;
use std::ops::{Deref, DerefMut};

#[derive(Component)]
pub(crate) struct PlayerInventory {
    inventory: Inventory,
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

impl ChangeTracked for PlayerInventory {
    type ChangeItem = InventoryChange;

    fn changes(&mut self) -> Vec<Self::ChangeItem> {
        self.inventory.changes()
    }
}

#[async_trait]
impl ApplyToDatabase for InventoryChange {
    async fn apply(&self, character_id: u32, pool: &PgPool) -> Result<(), sqlx::Error> {
        match self {
            InventoryChange::AddItem { slot, item } => {
                sqlx::query!(
                    "INSERT INTO character_items(character_id, item_obj_id, upgrade_level, slot, variance, amount) VALUES($1, $2, $3, $4, $5, $6) ON CONFLICT(character_id, slot) DO UPDATE SET item_obj_id = EXCLUDED.item_obj_id, upgrade_level = EXCLUDED.upgrade_level, variance = EXCLUDED.variance, amount = EXCLUDED.amount",
                    character_id as i32,
                    item.reference.common.ref_id as i32,
                    item.type_data.upgrade_level().map(|a| a as i16).unwrap_or(0),
                    i16::from(u8::from(*slot)),
                    item.variance.map(|a| a as i64),
                    item.type_data.amount() as i16 // This should be fine, since we should never have gold inside an item slot
                ).execute(pool).await?;
            },
            InventoryChange::ChangeTypeData { slot, new_item, .. } => {
                sqlx::query!(
                    "UPDATE character_items SET upgrade_level = $1, amount = $2 WHERE character_id = $3 AND slot = $4",
                    new_item.upgrade_level().map(|a| a as i16).unwrap_or(0),
                    new_item.amount() as i16, // This should be fine, since we should never have gold inside an item slot
                    character_id as i32,
                    i16::from(u8::from(*slot)),
                )
                .execute(pool)
                .await?;
            },
            InventoryChange::MoveItem {
                source_slot,
                target_slot,
            } => {
                sqlx::query!(
                    "UPDATE character_items SET slot = $1 WHERE character_id = $2 AND slot = $3",
                    i16::from(u8::from(*target_slot)),
                    character_id as i32,
                    i16::from(u8::from(*source_slot)),
                )
                .execute(pool)
                .await?;
            },
            InventoryChange::RemoveItem { slot } => {
                sqlx::query!(
                    "DELETE FROM character_items WHERE character_id = $1 AND slot = $2",
                    character_id as i32,
                    i16::from(u8::from(*slot)),
                )
                .execute(pool)
                .await?;
            },
            InventoryChange::Swap {
                first_slot,
                second_slot,
            } => {
                sqlx::query!(
                    "UPDATE character_items SET slot = case slot when $2 then $3 when $3 then $2 end WHERE character_id = $1 AND slot in ($2, $3)",
                    character_id as i32,
                    i16::from(u8::from(*first_slot)),
                    i16::from(u8::from(*second_slot)),
                )
                .execute(pool)
                .await?;
            },
        }
        Ok(())
    }
}

impl PlayerInventory {
    fn from_loaded_inventory(items: &[CharacterWorldItem], size: usize) -> Option<Inventory> {
        let item_map = WorldData::items();
        let mut inventory = Inventory::new(size);

        for item in items {
            let item_def = item_map.find_id(item.reference_id)?;
            let type_data = Self::item_type_data_for(item_def, item)?;
            let slot = inventory.slot_from_raw(item.slot).ok()?;
            inventory
                .set_item(
                    slot,
                    Item {
                        reference: item_def,
                        variance: item.variance,
                        type_data,
                    },
                )
                .ok()?;
        }

        Some(inventory)
    }

    fn item_type_data_for(ref_data: &RefItemData, item: &CharacterWorldItem) -> Option<ItemTypeData> {
        let obj_type = ObjectType::from_type_id(&ref_data.common.type_id)?;
        if let ObjectType::Item(item_type) = obj_type {
            Some(match item_type {
                ObjectItem::Equippable(_) => ItemTypeData::Equipment {
                    upgrade_level: item.upgrade_level,
                },
                ObjectItem::Pet(_) => ItemTypeData::COS,
                _ => ItemTypeData::Consumable { amount: item.amount },
            })
        } else {
            None
        }
    }

    pub(crate) fn from_loaded(items: &[CharacterWorldItem], size: usize) -> Option<Self> {
        let inventory = Self::from_loaded_inventory(items, size)?;
        Some(PlayerInventory { inventory })
    }
}
