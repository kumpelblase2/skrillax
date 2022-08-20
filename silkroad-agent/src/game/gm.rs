use crate::comp::drop::{DropBundle, Item, ItemDrop};
use crate::comp::monster::{Monster, MonsterBundle, RandomStroll, SpawnedBy};
use crate::comp::net::GmInput;
use crate::comp::player::Agent;
use crate::comp::pos::Position;
use crate::comp::sync::Synchronize;
use crate::comp::visibility::Visibility;
use crate::comp::{Client, Despawn, EntityReference, GameEntity, Health};
use crate::world::{EntityLookup, CHARACTERS, ITEMS};
use bevy_core::Timer;
use bevy_ecs::prelude::*;
use id_pool::IdPool;
use silkroad_data::type_id::{ObjectConsumable, ObjectConsumableCurrency, ObjectItem, ObjectType};
use silkroad_protocol::gm::{GmCommand, GmResponse};
use silkroad_protocol::world::{BodyState, UpdatedState};
use silkroad_protocol::ServerPacket;
use std::mem::take;
use std::time::Duration;

pub(crate) fn handle_gm_commands(
    mut query: Query<(Entity, &GameEntity, &Client, &Position, &mut GmInput, &mut Synchronize)>,
    mut commands: Commands,
    mut id_pool: ResMut<IdPool>,
    lookup: Res<EntityLookup>,
) {
    for (entity, game_entity, client, position, mut input, mut sync) in query.iter_mut() {
        for command in take(&mut input.inputs) {
            // FIXME: send response
            match command {
                GmCommand::BanUser { .. } => {},
                GmCommand::SpawnMonster { ref_id, amount, rarity } => {
                    let character_def = CHARACTERS.get().unwrap().find_id(ref_id).unwrap();
                    for _ in 0..amount {
                        let unique_id = id_pool.request_id().unwrap();
                        // FIXME: `SpawnedBy` doesn't really make sense here.
                        let bundle = MonsterBundle {
                            monster: Monster { target: None, rarity },
                            health: Health::new(character_def.hp),
                            position: position.clone(),
                            entity: GameEntity { unique_id, ref_id },
                            visibility: Visibility::with_radius(100.),
                            spawner: SpawnedBy { spawner: entity },
                            navigation: Agent::new(character_def.run_speed as f32),
                            sync: Default::default(),
                            stroll: RandomStroll::new(position.location.to_location(), 100., Duration::from_secs(1)),
                        };
                        commands.spawn().insert_bundle(bundle);
                    }
                },
                GmCommand::MakeItem { ref_id, upgrade } => {
                    let item = ITEMS.get().unwrap().find_id(ref_id).unwrap();
                    let unique_id = id_pool.request_id().unwrap();
                    let object_type = ObjectType::from_type_id(&item.type_id).unwrap();
                    let item_type = if matches!(object_type, ObjectType::Item(ObjectItem::Equippable(_))) {
                        Item::Equipment { upgrade }
                    } else if matches!(
                        object_type,
                        ObjectType::Item(ObjectItem::Consumable(ObjectConsumable::Currency(
                            ObjectConsumableCurrency::Gold
                        )))
                    ) {
                        Item::Gold(1)
                    } else {
                        Item::Consumable(1)
                    };
                    let bundle = DropBundle {
                        drop: ItemDrop {
                            owner: Some(EntityReference(entity, *game_entity)),
                            item: item_type,
                        },
                        position: position.clone(),
                        game_entity: GameEntity { unique_id, ref_id },
                        despawn: Despawn(Timer::new(item.despawn_time, false)),
                    };
                    commands.spawn().insert_bundle(bundle);
                },
                GmCommand::Invincible => {
                    sync.state.push(UpdatedState::Body(BodyState::GMInvincible));
                },
                GmCommand::Invisible => {
                    sync.state.push(UpdatedState::Body(BodyState::GMInvisible));
                },
                GmCommand::KillMonster { unique_id, .. } => {},
            }
        }
    }
}
