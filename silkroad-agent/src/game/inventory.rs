use crate::comp::drop::{DropBundle, ItemDrop};
use crate::comp::net::InventoryInput;
use crate::comp::player::{Inventory, MoveError, Player, Race};
use crate::comp::pos::{GlobalLocation, GlobalPosition, Heading, Position};
use crate::comp::{drop, Client, GameEntity};
use crate::ext::{Vector2Ext, Vector3Ext};
use crate::game::gold::get_gold_ref_id;
use bevy_ecs::prelude::*;
use id_pool::IdPool;
use pk2::Pk2;
use rand::Rng;
use silkroad_data::type_id::{
    ObjectClothingPart, ObjectClothingType, ObjectConsumable, ObjectConsumableAmmo, ObjectEquippable, ObjectItem,
    ObjectJewelryType, ObjectRace, ObjectType, ObjectWeaponType,
};
use silkroad_navmesh::NavmeshLoader;
use silkroad_protocol::inventory::{
    InventoryOperationError, InventoryOperationRequest, InventoryOperationResponseData, InventoryOperationResult,
};
use silkroad_protocol::world::CharacterPointsUpdate;
use silkroad_protocol::ClientPacket;
use std::cmp::max;
use std::mem;

pub const GOLD_SLOT: u8 = 0xFE;

pub(crate) fn handle_inventory_input(
    mut query: Query<(
        Entity,
        &GameEntity,
        &Client,
        &mut InventoryInput,
        &mut Player,
        &Position,
    )>,
    mut commands: Commands,
    mut navmesh: ResMut<NavmeshLoader<Pk2>>,
    mut id_pool: ResMut<IdPool>,
) {
    for (entity, game_entity, client, mut input, mut player, position) in query.iter_mut() {
        for action in mem::take(&mut input.inputs) {
            match action {
                ClientPacket::InventoryOperation(op) => {
                    match op.data {
                        InventoryOperationRequest::DropGold { amount } => {
                            if amount > player.character.gold {
                                client.send(InventoryOperationResult::Error(InventoryOperationError::Indisposable));
                                continue;
                            }

                            if amount == 0 {
                                continue;
                            }

                            player.character.gold -= amount;

                            let drop_position = position.location.0.to_flat_vec2().random_in_radius(2.0);
                            let local_drop_pos = GlobalLocation(drop_position).to_local();
                            let target_region = local_drop_pos.0;
                            let drop_position = drop_position.with_height(
                                navmesh
                                    .load_navmesh(target_region)
                                    .unwrap()
                                    .heightmap()
                                    .height_at_position(local_drop_pos.1.x, local_drop_pos.1.y)
                                    .unwrap(),
                            );
                            let drop_id = id_pool.request_id().expect("Should be able to generate an id");
                            let rotation = rand::thread_rng().gen_range(0..360) as f32;

                            let item_ref = get_gold_ref_id(amount as u32);
                            commands.spawn().insert_bundle(DropBundle {
                                drop: ItemDrop {
                                    owner: None,
                                    item: drop::Item::Gold(amount as u32),
                                },
                                position: Position {
                                    location: GlobalPosition(drop_position),
                                    rotation: Heading(rotation),
                                },
                                game_entity: GameEntity {
                                    unique_id: drop_id,
                                    ref_id: item_ref.ref_id,
                                },
                                despawn: item_ref.despawn_time.into(),
                            });

                            client.send(InventoryOperationResult::Success(
                                InventoryOperationResponseData::DropGold { amount },
                            ));
                            client.send(CharacterPointsUpdate::Gold(player.character.gold, 0));
                        },
                        InventoryOperationRequest::PickupItem { unique_id } => {},
                        InventoryOperationRequest::Move { source, target, amount } => {
                            if let Some(source_item) = player.inventory.get_item_at(source) {
                                if Inventory::is_equipment_slot(target) {
                                    let type_id = source_item.reference.type_id;
                                    let object_type = ObjectType::from_type_id(&type_id)
                                        .expect("Iem to equip should have valid object type.");
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
                                        client.send(InventoryOperationResult::Error(
                                            InventoryOperationError::Indisposable,
                                        ));
                                        continue;
                                    }
                                }
                                match player.inventory.move_item(source, target, max(1, amount)) {
                                    Err(MoveError::Impossible) => {},
                                    Err(MoveError::ItemDoesNotExist) => {},
                                    Ok(amount_moved) => {
                                        client.send(InventoryOperationResult::Success(
                                            InventoryOperationResponseData::move_item(source, target, amount_moved),
                                        ));
                                    },
                                }
                            } else {
                                // TODO: use proper error code
                                client.send(InventoryOperationResult::Error(InventoryOperationError::Indisposable));
                            }
                        },
                        InventoryOperationRequest::DropItem { .. } => {},
                    }
                },
                _ => {},
            }
        }
    }
}

fn does_object_type_match_race(user_race: Race, obj_type: ObjectType) -> bool {
    match obj_type {
        ObjectType::Item(item) => match item {
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
            ObjectItem::Consumable(consumable) => match consumable {
                ObjectConsumable::Ammo(kind) => {
                    return match kind {
                        ObjectConsumableAmmo::Arrows => user_race == Race::Chinese,
                        ObjectConsumableAmmo::Bolts => user_race == Race::European,
                    }
                },
                _ => {},
            },
            _ => {},
        },
        _ => {},
    }
    false
}

fn does_object_type_match_slot(slot: u8, obj_type: ObjectType) -> bool {
    match obj_type {
        ObjectType::Item(item) => match item {
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
            ObjectItem::Consumable(consumable) => match consumable {
                ObjectConsumable::Ammo(_) => {
                    return slot == 7;
                },
                _ => {},
            },
            _ => {},
        },
        _ => {},
    }
    false
}
