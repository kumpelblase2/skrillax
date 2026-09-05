use crate::comp::exp::Leveled;
use crate::comp::gold::GoldPouch;
use crate::comp::inventory::PlayerInventory;
use crate::comp::net::Client;
use crate::comp::player::CharacterRace;
use crate::comp::pos::Position;
use crate::game::drop::SpawnDrop;
use crate::game::loot::LootResource;
use crate::input::PlayerInputEvent;
use bevy::prelude::*;
use silkroad_definitions::type_id::ObjectType;
use silkroad_game_base::{item_type_matches_equipment_slot, item_type_matches_race, MainInventorySlot, MoveError};
use silkroad_protocol::inventory::{
    InventoryOperation, InventoryOperationError, InventoryOperationRequest, InventoryOperationResponseData,
    InventoryOperationResult,
};
use std::cmp::max;

pub(crate) fn handle_inventory_input(
    mut query: Query<(
        &Client,
        &Leveled,
        &CharacterRace,
        &mut PlayerInventory,
        &mut GoldPouch,
        &Position,
    )>,
    mut item_spawn: MessageWriter<SpawnDrop>,
    mut reader: MessageReader<PlayerInputEvent<InventoryOperation>>,
    loot: Res<LootResource>,
) {
    for event in reader.read() {
        let Ok((client, level, race, mut inventory, mut gold, position)) = query.get_mut(event.player) else {
            continue;
        };

        match event.input.data {
            InventoryOperationRequest::DropGold { amount } => {
                if amount > gold.amount() {
                    client.send(InventoryOperationResult::Failure(
                        InventoryOperationError::NotEnoughGold,
                    ));
                    continue;
                }

                if amount == 0 {
                    continue;
                }

                gold.spend(amount);

                item_spawn.write(SpawnDrop::new(loot.gold_item(amount as u32), position.location(), None));

                client.send(InventoryOperationResult::Success(
                    InventoryOperationResponseData::DropGold { amount },
                ));
            },
            InventoryOperationRequest::PickupItem { unique_id } => {},
            InventoryOperationRequest::Move { source, target, amount } => {
                let (Ok(source_slot), Ok(target_slot)) =
                    (inventory.slot_from_raw(source), inventory.slot_from_raw(target))
                else {
                    client.send(InventoryOperationResult::Failure(
                        InventoryOperationError::InvalidTarget,
                    ));
                    continue;
                };

                if let Some(source_item) = inventory.get_item_at(source_slot) {
                    if let MainInventorySlot::Equipment(target_equipment_slot) = target_slot {
                        let type_id = source_item.reference.common.type_id;
                        let object_type =
                            ObjectType::from_type_id(&type_id).expect("Item to equip should have valid object type.");
                        let fits = item_type_matches_equipment_slot(target_equipment_slot, object_type)
                            && source_item
                                .reference
                                .required_level
                                .map(|val| val.get() <= level.current_level())
                                .unwrap_or(true)
                            && item_type_matches_race(race.inner(), object_type);
                        // TODO: check if equipment requirement sex matches
                        //  check if required masteries matches
                        if !fits {
                            // TODO: Use more appropriate error code
                            client.send(InventoryOperationResult::Failure(InventoryOperationError::Indisposable));
                            continue;
                        }
                    }
                    match inventory.move_item(source_slot, target_slot, max(1, amount)) {
                        Err(MoveError::Impossible) => {},
                        Err(MoveError::ItemDoesNotExist) => {},
                        Err(MoveError::NotStackable) => {},
                        Err(MoveError::InvalidSlot(_)) => {},
                        Ok(amount_moved) => {
                            client.send(InventoryOperationResult::Success(
                                InventoryOperationResponseData::move_item(
                                    source_slot.into(),
                                    target_slot.into(),
                                    amount_moved,
                                ),
                            ));
                        },
                    }
                } else {
                    client.send(InventoryOperationResult::Failure(
                        InventoryOperationError::InvalidTarget,
                    ));
                }
            },
            InventoryOperationRequest::DropItem { .. } => {},
        }
    }
}
