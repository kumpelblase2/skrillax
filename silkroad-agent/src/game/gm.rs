use crate::comp::drop::{DropBundle, ItemDrop};
use crate::comp::monster::{Monster, MonsterBundle, RandomStroll, SpawnedBy};
use crate::comp::net::GmInput;
use crate::comp::player::Agent;
use crate::comp::pos::Position;
use crate::comp::sync::Synchronize;
use crate::comp::visibility::Visibility;
use crate::comp::{Client, EntityReference, GameEntity, Health};
use crate::world::EntityLookup;
use bevy_core::Timer;
use bevy_ecs::prelude::*;
use id_pool::IdPool;
use silkroad_data::characterdata::CharacterMap;
use silkroad_data::itemdata::ItemMap;
use silkroad_protocol::gm::{GmCommand, GmResponse};
use silkroad_protocol::world::{BodyState, UpdatedState};
use silkroad_protocol::ServerPacket;
use std::mem::take;
use std::time::Duration;

pub(crate) fn handle_gm_commands(
    mut query: Query<(Entity, &GameEntity, &Client, &Position, &mut GmInput, &mut Synchronize)>,
    mut commands: Commands,
    characters: Res<CharacterMap>,
    items: Res<ItemMap>,
    mut id_pool: ResMut<IdPool>,
    mut lookup: ResMut<EntityLookup>,
) {
    for (entity, game_entity, client, position, mut input, mut sync) in query.iter_mut() {
        for command in take(&mut input.inputs) {
            // FIXME: send response
            match command {
                GmCommand::BanUser { .. } => {},
                GmCommand::SpawnMonster { ref_id, amount, rarity } => {
                    let character_def = characters.find_id(ref_id).unwrap();
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
                        let spawned = commands.spawn().insert_bundle(bundle).id();
                        lookup.add_entity(unique_id, spawned);
                    }
                },
                GmCommand::MakeItem { ref_id, amount } => {
                    let item = items.find_id(ref_id).unwrap();
                    let unique_id = id_pool.request_id().unwrap();
                    let bundle = DropBundle {
                        drop: ItemDrop {
                            despawn_timer: Timer::new(item.despawn_time, false),
                            owner: Some(EntityReference(entity, *game_entity)),
                            amount: amount as u32,
                        },
                        position: position.clone(),
                        game_entity: GameEntity { unique_id, ref_id },
                    };
                    let spawned = commands.spawn().insert_bundle(bundle).id();
                    lookup.add_entity(unique_id, spawned);
                },
                GmCommand::Invincible => {
                    sync.state.push(UpdatedState::Body(BodyState::GMInvincible));
                },
                GmCommand::Invisible => {
                    sync.state.push(UpdatedState::Body(BodyState::GMInvisible));
                },
            }
        }
    }
}
