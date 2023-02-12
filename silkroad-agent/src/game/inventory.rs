use crate::comp::inventory::PlayerInventory;
use crate::comp::net::Client;
use crate::comp::player::Player;
use crate::comp::pos::Position;
use crate::game::drop::SpawnDrop;
use crate::game::gold::get_gold_ref_id;
use crate::input::PlayerInput;
use bevy_ecs::prelude::*;
use silkroad_data::type_id::{
    ObjectClothingPart, ObjectClothingType, ObjectConsumable, ObjectConsumableAmmo, ObjectEquippable, ObjectItem,
    ObjectJewelryType, ObjectRace, ObjectType, ObjectWeaponType,
};
use silkroad_game_base::{Inventory, Item, ItemTypeData, MoveError, Race};
use silkroad_protocol::inventory::{
    InventoryOperationError, InventoryOperationRequest, InventoryOperationResponseData, InventoryOperationResult,
};
use silkroad_protocol::world::CharacterPointsUpdate;
use std::cmp::max;

pub(crate) fn handle_inventory_input(
    mut query: Query<(&Client, &Player, &PlayerInput, &mut PlayerInventory, &Position)>,
    mut item_spawn: EventWriter<SpawnDrop>,
) {
    for (client, player, input, mut inventory, position) in query.iter_mut() {
        if let Some(ref action) = input.inventory {
            match action.data {
                InventoryOperationRequest::DropGold { amount } => {
                    if amount > inventory.gold {
                        client.send(InventoryOperationResult::Error(InventoryOperationError::NotEnoughGold));
                        continue;
                    }

                    if amount == 0 {
                        continue;
                    }

                    inventory.gold -= amount;

                    let item_ref = get_gold_ref_id(amount as u32);
                    item_spawn.send(SpawnDrop::new(
                        Item {
                            reference: item_ref,
                            variance: None,
                            type_data: ItemTypeData::Gold { amount: amount as u32 },
                        },
                        position.location.to_location(),
                        None,
                    ));

                    client.send(InventoryOperationResult::Success(
                        InventoryOperationResponseData::DropGold { amount },
                    ));
                    client.send(CharacterPointsUpdate::Gold {
                        amount: player.character.gold,
                        display: false,
                    });
                    client.send(InventoryOperationResult::Success(
                        InventoryOperationResponseData::DropGold { amount },
                    ))
                },
                InventoryOperationRequest::PickupItem { unique_id } => {},
                InventoryOperationRequest::Move { source, target, amount } => {
                    if let Some(source_item) = inventory.get_item_at(source) {
                        if Inventory::is_equipment_slot(target) {
                            let type_id = source_item.reference.common.type_id;
                            let object_type = ObjectType::from_type_id(&type_id)
                                .expect("Item to equip should have valid object type.");
                            let fits = does_object_type_match_slot(target, object_type)
                                && source_item
                                    .reference
                                    .required_level
                                    .map(|val| val.get() <= player.character.level)
                                    .unwrap_or(true)
                                && does_object_type_match_race(player.character.race, object_type);
                            // TODO: check if equipment requirement sex matches
                            //  check if required masteries matches
                            if !fits {
                                // TODO: Use more appropriate error code
                                client.send(InventoryOperationResult::Error(InventoryOperationError::Indisposable));
                                continue;
                            }
                        }
                        match inventory.move_item(source, target, max(1, amount)) {
                            Err(MoveError::Impossible) => {},
                            Err(MoveError::ItemDoesNotExist) => {},
                            Err(MoveError::NotStackable) => {},
                            Ok(amount_moved) => {
                                client.send(InventoryOperationResult::Success(
                                    InventoryOperationResponseData::move_item(source, target, amount_moved),
                                ));
                            },
                        }
                    } else {
                        client.send(InventoryOperationResult::Error(InventoryOperationError::InvalidTarget));
                    }
                },
                InventoryOperationRequest::DropItem { .. } => {},
            }
        }
    }
}

fn does_object_type_match_race(user_race: Race, obj_type: ObjectType) -> bool {
    if let ObjectType::Item(item) = obj_type {
        match item {
            ObjectItem::Equippable(equipment) => match equipment {
                ObjectEquippable::Clothing(kind, _) => {
                    return match kind {
                        ObjectClothingType::Garment | ObjectClothingType::Protector | ObjectClothingType::Armor => {
                            user_race == Race::Chinese
                        },
                        ObjectClothingType::Robe | ObjectClothingType::LightArmor | ObjectClothingType::HeavyArmor => {
                            user_race == Race::European
                        },
                    }
                },
                ObjectEquippable::Shield(race) | ObjectEquippable::Jewelry(race, _) => {
                    return match race {
                        ObjectRace::Chinese => user_race == Race::Chinese,
                        ObjectRace::European => user_race == Race::European,
                    }
                },
                ObjectEquippable::Weapon(kind) => {
                    return match kind {
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
                    }
                },
                _ => {},
            },
            ObjectItem::Consumable(ObjectConsumable::Ammo(kind)) => {
                return match kind {
                    ObjectConsumableAmmo::Arrows => user_race == Race::Chinese,
                    ObjectConsumableAmmo::Bolts => user_race == Race::European,
                }
            },
            _ => {},
        }
    }
    false
}

fn does_object_type_match_slot(slot: u8, obj_type: ObjectType) -> bool {
    if let ObjectType::Item(item) = obj_type {
        match item {
            ObjectItem::Equippable(equipment) => match equipment {
                ObjectEquippable::Clothing(_, part) => {
                    return match part {
                        ObjectClothingPart::Head => slot == 0,
                        ObjectClothingPart::Shoulder => slot == 1,
                        ObjectClothingPart::Body => slot == 2,
                        ObjectClothingPart::Leg => slot == 4,
                        ObjectClothingPart::Arm => slot == 3,
                        ObjectClothingPart::Foot => slot == 5,
                        ObjectClothingPart::Any => false,
                    }
                },
                ObjectEquippable::Shield(_) => {
                    return slot == 7;
                },
                ObjectEquippable::Jewelry(_, kind) => {
                    return match kind {
                        ObjectJewelryType::Earring => slot == 8,
                        ObjectJewelryType::Necklace => slot == 9,
                        ObjectJewelryType::Ring => slot == 11 || slot == 10,
                    }
                },
                ObjectEquippable::Weapon(_) => {
                    return slot == 6;
                },
                _ => {},
            },
            ObjectItem::Consumable(ObjectConsumable::Ammo(_)) => {
                return slot == 7;
            },
            _ => {},
        }
    }
    false
}
