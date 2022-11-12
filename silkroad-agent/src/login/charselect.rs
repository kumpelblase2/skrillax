use crate::comp::net::{CharselectInput, Client, GmInput, InputBundle};
use crate::comp::player::{Agent, Inventory, Player, PlayerBundle};
use crate::comp::pos::{Heading, LocalPosition, Position};
use crate::comp::visibility::Visibility;
use crate::comp::{CharacterSelect, GameEntity, Playing};
use crate::config::GameConfig;
use crate::db::character::{CharacterData, CharacterItem};
use crate::ext::{DbPool, EntityIdPool};
use crate::login::character_loader::Character;
use crate::login::job_distribution::JobDistribution;
use crate::server_plugin::ServerId;
use crate::tasks::TaskCreator;
use bevy_ecs::prelude::*;
use cgmath::Vector3;
use chrono::{TimeZone, Utc};
use silkroad_data::DataEntry;
use silkroad_protocol::character::{
    CharacterJoinRequest, CharacterJoinResponse, CharacterListAction, CharacterListContent, CharacterListEntry,
    CharacterListEquippedItem, CharacterListError, CharacterListRequest, CharacterListRequestAction,
    CharacterListResponse, CharacterListResult, MacroStatus, TimeInformation, MACRO_HUNT, MACRO_POTION, MACRO_SKILL,
};
use silkroad_protocol::inventory::{InventoryItemBindingData, InventoryItemContentData, InventoryItemData, RentInfo};
use silkroad_protocol::world::{
    ActionState, AliveState, BodyState, CharacterSpawn, CharacterSpawnEnd, CharacterSpawnStart, EntityState, JobType,
};
use silkroad_protocol::{ClientPacket, SilkroadTime};
use std::mem::take;
use tokio::sync::oneshot::error::TryRecvError;
use tracing::{debug, warn};

pub(crate) fn charselect(
    settings: Res<GameConfig>,
    job_distribution: Res<JobDistribution>,
    pool: Res<DbPool>,
    task_creator: Res<TaskCreator>,
    server_id: Res<ServerId>,
    mut cmd: Commands,
    mut allocator: ResMut<EntityIdPool>,
    mut query: Query<(Entity, &Client, &mut CharselectInput, &mut CharacterSelect, &Playing)>,
) {
    for (entity, client, mut charselect_inputs, mut character_list, playing) in query.iter_mut() {
        for packet in take(&mut charselect_inputs.inputs) {
            match packet {
                ClientPacket::CharacterListRequest(CharacterListRequest { action }) => match action {
                    CharacterListRequestAction::Create {
                        character_name,
                        ref_id,
                        scale,
                        chest,
                        pants,
                        boots,
                        weapon,
                    } => {
                        if !can_create_character_with_name(&character_list, &character_name) {
                            debug!(id = ?client.0.id(), "Tried to create character without checking name first.");
                            client.send(CharacterListResponse::new(
                                CharacterListAction::Create,
                                CharacterListResult::error(CharacterListError::CloudntCreateCharacter),
                            ));
                        }

                        let character = create_character_from(
                            playing.0.id,
                            server_id.0,
                            character_name,
                            ref_id,
                            scale,
                            chest,
                            pants,
                            boots,
                            weapon,
                        );
                        let task = task_creator.create_task(Character::create_character(character, pool.clone()));
                        character_list.character_create = Some(task);
                    },
                    CharacterListRequestAction::List => {
                        if character_list.character_receiver.is_some() {
                            continue;
                        }

                        let receiver = task_creator.create_task(Character::load_characters_sparse(
                            playing.0.id,
                            server_id.0,
                            pool.clone(),
                        ));
                        character_list.character_receiver = Some(receiver);
                    },
                    CharacterListRequestAction::CheckName { character_name } => {
                        if character_list.character_name_check.is_none() {
                            character_list.checked_name = Some(character_name.clone());
                            let server_id = server_id.0;
                            let task = task_creator.create_task(CharacterData::check_name_available(
                                character_name,
                                server_id,
                                pool.clone(),
                            ));
                            character_list.character_name_check = Some(task);
                        }
                    },
                    CharacterListRequestAction::Delete { character_name } => {
                        if !has_user_character_with_name(&character_list, &character_name) {
                            client.send(CharacterListResponse::new(
                                CharacterListAction::Delete,
                                CharacterListResult::error(CharacterListError::InvalidName),
                            ));
                            continue;
                        }

                        let task = task_creator.create_task(Character::start_delete_character(
                            playing.0.id,
                            character_name,
                            server_id.0,
                            settings.deletion_time,
                            pool.clone(),
                        ));
                        character_list.character_delete_task = Some(task);
                    },
                    CharacterListRequestAction::Restore { character_name } => {
                        if !has_user_character_with_name(&character_list, &character_name) {
                            client.send(CharacterListResponse::new(
                                CharacterListAction::Delete,
                                CharacterListResult::error(CharacterListError::InvalidName),
                            ));
                            continue;
                        }

                        let task = task_creator.create_task(Character::restore_character(
                            playing.0.id,
                            character_name,
                            server_id.0,
                            pool.clone(),
                        ));
                        character_list.character_restore = Some(task);
                    },
                    CharacterListRequestAction::ShowJobSpread => {
                        let (hunter_perc, thief_perc) = job_distribution.spread();
                        send_job_spread(client, hunter_perc, thief_perc);
                    },
                    CharacterListRequestAction::AssignJob { .. } => {},
                },
                ClientPacket::CharacterJoinRequest(CharacterJoinRequest { character_name }) => {
                    match character_list.characters {
                        Some(ref characters) => {
                            let character = characters
                                .iter()
                                .find(|char| char.character_data.charname == character_name.as_ref())
                                .unwrap();

                            if character.character_data.deletion_end.is_some() {
                                client.send(CharacterJoinResponse::error(CharacterListError::InvalidName));
                                continue;
                            }

                            let player = Player {
                                user: playing.0.clone(),
                                character: crate::comp::player::Character::from_db_character(&character.character_data),
                                inventory: Inventory::from(&character.items, 45),
                                logout: None,
                            };

                            let data = &character.character_data;

                            let position = Position {
                                location: LocalPosition(
                                    (data.region as u16).into(),
                                    Vector3::new(data.x, data.y, data.z),
                                )
                                .to_global(),
                                rotation: Heading::from(data.rotation as u16),
                            };

                            let agent = Agent::new(50.0);

                            let game_entity = GameEntity {
                                ref_id: data.character_type as u32,
                                unique_id: allocator.request_id().unwrap(),
                            };

                            client.send(CharacterJoinResponse::success());

                            send_spawn(client, &game_entity, &player, &position, settings.max_level);

                            client.send(MacroStatus::Possible(MACRO_POTION | MACRO_HUNT | MACRO_SKILL, 0));

                            let is_gm = player.character.gm;

                            let mut spawn_cmd = cmd.entity(entity);
                            spawn_cmd
                                .insert(PlayerBundle::new(
                                    player,
                                    game_entity,
                                    agent,
                                    position.clone(),
                                    Visibility::with_radius(500.),
                                ))
                                .insert(InputBundle::default())
                                .remove::<CharacterSelect>();

                            if is_gm {
                                spawn_cmd.insert(GmInput::default());
                            }
                        },
                        None => {
                            // TODO
                            client.send(CharacterJoinResponse::error(CharacterListError::ReachedCapacity));
                        },
                    }
                },
                _ => {},
            }
        }

        if let Some(receiver) = character_list.character_receiver.as_mut() {
            match receiver.try_recv() {
                Ok(characters) => {
                    send_character_list(client, &characters);
                    character_list.characters = Some(characters);
                    character_list.character_receiver = None;
                },
                Err(TryRecvError::Empty) => {},
                Err(e) => {
                    warn!(id = playing.0.id, "Error when loading characters. {:?}", e);
                    character_list.character_receiver = None;
                },
            }
        }

        if let Some(receiver) = character_list.character_name_check.as_mut() {
            match receiver.try_recv() {
                Ok(available) => {
                    let result = if available {
                        CharacterListResult::ok(CharacterListContent::Empty)
                    } else {
                        character_list.checked_name = None;
                        CharacterListResult::error(CharacterListError::NameAlreadyUsed)
                    };
                    client.send(CharacterListResponse::new(CharacterListAction::CheckName, result));
                    character_list.character_name_check = None;
                },
                Err(TryRecvError::Empty) => {},
                Err(_) => {
                    warn!(id = playing.0.id, "Error when checking name.");
                    character_list.character_name_check = None;
                },
            }
        }

        if let Some(receiver) = character_list.character_create.as_mut() {
            match receiver.try_recv() {
                Ok(_) => {
                    client.send(CharacterListResponse::new(
                        CharacterListAction::Create,
                        CharacterListResult::ok(CharacterListContent::Empty),
                    ));
                    character_list.character_create = None;
                },
                Err(TryRecvError::Empty) => {},
                Err(_) => {
                    warn!(id = playing.0.id, "Error when creating character.");
                    character_list.character_create = None;
                },
            }
        }

        if let Some(receiver) = character_list.character_delete_task.as_mut() {
            match receiver.try_recv() {
                Ok(true) => {
                    client.send(CharacterListResponse::new(
                        CharacterListAction::Delete,
                        CharacterListResult::ok(CharacterListContent::Empty),
                    ));
                    character_list.character_delete_task = None;
                },
                Err(TryRecvError::Empty) => {},
                _ => {
                    character_list.character_delete_task = None;
                    todo!("Send error to client.")
                },
            }
        }

        if let Some(receiver) = character_list.character_restore.as_mut() {
            match receiver.try_recv() {
                Ok(result) => {
                    if result {
                        client.send(CharacterListResponse::new(
                            CharacterListAction::Restore,
                            CharacterListResult::ok(CharacterListContent::Empty),
                        ));
                    } else {
                        client.send(CharacterListResponse::new(
                            CharacterListAction::Restore,
                            CharacterListResult::error(CharacterListError::InvalidName), // TODO: use a better error
                        ));
                    }
                    character_list.character_restore = None;
                },
                Err(TryRecvError::Empty) => {},
                Err(_) => {
                    warn!(id = playing.0.id, "Error when restoring a character.");
                    character_list.character_restore = None;
                },
            }
        }
    }
}

fn has_user_character_with_name(charselect: &CharacterSelect, character_name: &str) -> bool {
    charselect
        .characters
        .as_ref()
        .map(|chars| {
            chars
                .iter()
                .any(|character| character.character_data.charname == character_name)
        })
        .unwrap_or(false)
}

fn can_create_character_with_name(charselect: &CharacterSelect, name: &str) -> bool {
    if let Some(ref checked_name) = charselect.checked_name {
        if charselect.character_name_check.is_some() || checked_name != name {
            return false;
        }
        true
    } else {
        false
    }
}

fn send_character_list(client: &Client, character_list: &[Character]) {
    let characters = character_list.iter().map(from_character).collect();
    let response = CharacterListResponse::new(
        CharacterListAction::List,
        CharacterListResult::ok(CharacterListContent::characters(characters, 0)),
    );
    client.send(response);
}

fn from_character(character: &Character) -> CharacterListEntry {
    let data = &character.character_data;
    let last_logout = data
        .last_logout
        .map(SilkroadTime::from)
        .unwrap_or_else(|| SilkroadTime::default());
    let target_deletion_date = data.deletion_end;
    let playtime_information = target_deletion_date
        .map(|end| end - Utc::now())
        .map(|dur| dur.num_minutes() as u32)
        .map(|remaining| TimeInformation::deleting(last_logout, remaining))
        .unwrap_or_else(|| TimeInformation::playable(last_logout));
    CharacterListEntry {
        ref_id: data.character_type as u32,
        name: data.charname.clone(),
        unknown: String::new(),
        scale: data.scale as u8,
        level: data.level as u8,
        exp: data.exp as u64,
        sp: data.sp as u32,
        strength: data.strength as u16,
        intelligence: data.intelligence as u16,
        stat_points: data.stat_points as u16,
        hp: data.current_hp as u32,
        mp: data.current_mp as u32,
        region: data.region as u16,
        playtime_info: playtime_information,
        guild_member_class: 0,
        guild_rename_required: None,
        academy_member_class: 0,
        equipped_items: character.items.iter().map(from_item).collect(),
        avatar_items: Vec::new(),
    }
}

fn from_item(item: &CharacterItem) -> CharacterListEquippedItem {
    CharacterListEquippedItem::new(item.item_obj_id as u32, item.upgrade_level as u8)
}

fn send_job_spread(client: &Client, hunters: u8, thieves: u8) {
    client.send(CharacterListResponse::new(
        CharacterListAction::ShowJobSpread,
        CharacterListResult::ok(CharacterListContent::jobspread(hunters, thieves)),
    ));
}

fn send_spawn(client: &Client, entity: &GameEntity, player: &Player, position: &Position, max_level: u8) {
    client.send(CharacterSpawnStart);

    let character_data = &player.character;

    let entity_state = EntityState {
        alive: AliveState::Spawning,
        unknown1: 0,
        action_state: ActionState::None,
        body_state: BodyState::None,
        unknown2: 0,
        walk_speed: 16.0,
        run_speed: 50.0,
        berserk_speed: 100.0,
        active_buffs: vec![],
    };

    let inventory_items = player
        .inventory
        .items()
        .map(|(slot, item)| InventoryItemData {
            slot: *slot,
            rent_data: RentInfo::Empty,
            item_id: item.reference.ref_id(),
            content_data: InventoryItemContentData::Equipment {
                plus_level: item.upgrade_level,
                variance: item.variance.unwrap_or_default(),
                durability: 1,
                magic: vec![],
                bindings_1: InventoryItemBindingData::new(1, 0),
                bindings_2: InventoryItemBindingData::new(2, 0),
                bindings_3: InventoryItemBindingData::new(3, 0),
                bindings_4: InventoryItemBindingData::new(4, 0),
            },
        })
        .collect();

    client.send(CharacterSpawn::new(
        SilkroadTime::default(),
        entity.ref_id,
        character_data.scale,
        character_data.level,
        character_data.max_level,
        character_data.exp,
        character_data.sp_exp,
        character_data.gold,
        character_data.sp,
        character_data.stat_points,
        character_data.berserk_points,
        character_data.current_hp,
        character_data.current_mp,
        character_data.beginner_mark,
        0,
        0,
        0,
        0,
        0,
        0x4,
        Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
        0,
        max_level,
        player.inventory.size() as u8,
        inventory_items,
        5,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        entity.unique_id,
        position.as_protocol(),
        0,
        position.rotation.into(),
        entity_state,
        character_data.name.clone(),
        String::new(),
        JobType::None, // TODO
        1,
        0,
        0,
        0,
        0,
        false,
        0,
        0xFF,
        player.user.id as u32,
        character_data.gm,
        Vec::new(),
        0,
        1,
        1,
        2,
        Vec::new(),
    ));

    client.send(CharacterSpawnEnd);
}

pub(crate) fn create_character_from(
    user_id: i32,
    server_id: u16,
    character_name: String,
    ref_id: u32,
    scale: u8,
    chest: u32,
    pants: u32,
    boots: u32,
    weapon: u32,
) -> Character {
    let character = CharacterData {
        id: 0,
        user_id,
        server_id: server_id as i32,
        charname: character_name,
        character_type: ref_id as i32,
        scale: scale as i16,
        level: 1,
        max_level: 1,
        exp: 0,
        sp: 0,
        sp_exp: 0,
        strength: 20,
        intelligence: 20,
        stat_points: 0,
        current_hp: 200,
        current_mp: 200,
        deletion_end: None,
        x: 739.,
        y: 37.4519,
        z: 1757.,
        rotation: 0,
        region: 24998,
        berserk_points: 0,
        gold: 5000000,
        beginner_mark: true,
        gm: false,
        last_logout: None,
    };

    let items = vec![
        CharacterItem {
            id: 0,
            character_id: 0,
            item_obj_id: chest as i32,
            upgrade_level: 0,
            variance: None,
            slot: 1,
            amount: 1,
        },
        CharacterItem {
            id: 0,
            character_id: 0,
            item_obj_id: pants as i32,
            upgrade_level: 0,
            variance: None,
            slot: 4,
            amount: 1,
        },
        CharacterItem {
            id: 0,
            character_id: 0,
            item_obj_id: boots as i32,
            upgrade_level: 0,
            variance: None,
            slot: 5,
            amount: 1,
        },
        CharacterItem {
            id: 0,
            character_id: 0,
            item_obj_id: weapon as i32,
            upgrade_level: 0,
            variance: None,
            slot: 6,
            amount: 1,
        },
    ];
    Character {
        character_data: character,
        items,
    }
}
