use crate::comp::drop::{DropBundle, ItemDrop};
use crate::comp::net::InventoryInput;
use crate::comp::player::Player;
use crate::comp::pos::{GlobalLocation, GlobalPosition, Heading, Position};
use crate::comp::{drop, Client, GameEntity};
use crate::ext::{Vector2Ext, Vector3Ext};
use crate::game::gold::get_gold_ref_id;
use bevy_ecs::prelude::*;
use id_pool::IdPool;
use pk2::Pk2;
use rand::Rng;
use silkroad_navmesh::NavmeshLoader;
use silkroad_protocol::inventory::{
    InventoryOperationError, InventoryOperationRequest, InventoryOperationResponseData, InventoryOperationResult,
};
use silkroad_protocol::world::CharacterPointsUpdate;
use silkroad_protocol::{ClientPacket, ServerPacket};
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
                ClientPacket::InventoryOperation(op) => match op.data {
                    InventoryOperationRequest::DropGold { amount } => {
                        if amount > player.character.gold {
                            client.send(ServerPacket::InventoryOperationResult(
                                InventoryOperationResult::Error {
                                    error: InventoryOperationError::Indisposable,
                                    slot: GOLD_SLOT,
                                },
                            ));
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

                        client.send(ServerPacket::InventoryOperationResult(
                            InventoryOperationResult::Success(InventoryOperationResponseData::DropGold { amount }),
                        ));
                        client.send(ServerPacket::CharacterPointsUpdate(CharacterPointsUpdate::Gold(
                            player.character.gold,
                            0,
                        )));
                    },
                    InventoryOperationRequest::PickupItem { unique_id } => {},
                    InventoryOperationRequest::Move { source, target, amount } => {},
                },
                _ => {},
            }
        }
    }
}
