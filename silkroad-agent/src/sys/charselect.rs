use crate::character_loader::Character;
use crate::comp::monster::Monster;
use crate::comp::player::{Agent, Buffed, Inventory, MovementState, Player};
use crate::comp::pos::{Heading, LocalPosition, Position};
use crate::comp::visibility::Visibility;
use crate::comp::{CharacterSelect, Client, NetworkedEntity, Playing};
use crate::db::character::CharacterItem;
use crate::job_coordinator::JobCoordinator;
use crate::time::AsSilkroadTime;
use crate::{CharacterLoaderFacade, GameSettings};
use bevy_ecs::prelude::*;
use cgmath::Vector3;
use chrono::{TimeZone, Utc};
use silkroad_protocol::character::{
    CharacterJoinRequest, CharacterJoinResponse, CharacterJoinResult, CharacterListAction, CharacterListContent,
    CharacterListEntry, CharacterListEquippedItem, CharacterListError, CharacterListRequest,
    CharacterListRequestAction, CharacterListResponse, CharacterListResult,
};
use silkroad_protocol::world::{
    ActionState, AliveState, BodyState, CharacterSpawn, CharacterSpawnEnd, CharacterSpawnStart, EntityMovementState,
    EntityRarity, EntityState, JobType, MovementType,
};
use silkroad_protocol::ClientPacket;

pub(crate) fn charselect(
    mut cmd: Commands,
    mut character_loader: ResMut<CharacterLoaderFacade>,
    settings: Res<GameSettings>,
    job_coordinator: Res<JobCoordinator>,
    mut query: Query<(Entity, &mut Client, &mut CharacterSelect, &Playing)>,
) {
    for (entity, mut client, mut character_list, playing) in query.iter_mut() {
        let mut character_list: &mut CharacterSelect = &mut character_list;
        while let Some(packet) = client.1.pop_front() {
            match packet {
                ClientPacket::CharacterListRequest(CharacterListRequest { action }) => match action {
                    CharacterListRequestAction::Create { .. } => {},
                    CharacterListRequestAction::List => {
                        character_loader.load_data(entity, playing.0.id);
                    },
                    CharacterListRequestAction::Delete { .. } => {},
                    CharacterListRequestAction::CheckName { .. } => {},
                    CharacterListRequestAction::Restore { .. } => {},
                    CharacterListRequestAction::ShowJobSpread => {
                        let (hunter_perc, thief_perc) = job_coordinator.spread();
                        send_job_spread(&client, hunter_perc, thief_perc);
                    },
                    CharacterListRequestAction::AssignJob { .. } => {},
                },
                ClientPacket::CharacterJoinRequest(CharacterJoinRequest { character_name }) => {
                    match character_list.0 {
                        Some(ref characters) => {
                            let character = characters
                                .iter()
                                .find(|char| char.character_data.charname == character_name.as_ref())
                                .unwrap();

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
                                rotation: Heading(0.0),
                            };

                            let agent = Agent {
                                id: 1,
                                ref_id: data.character_type as u32,
                                movement_speed: 50.0,
                                movement_state: MovementState::Standing,
                                movement_target: None,
                            };

                            client.send(CharacterJoinResponse::new(CharacterJoinResult::Success));

                            send_spawn(&client, &player, &agent, &position, settings.max_level);

                            cmd.entity(entity)
                                .insert(player)
                                .insert(agent)
                                .insert(position.clone())
                                .insert(Buffed {})
                                .insert(Visibility::with_radius(50.))
                                .remove::<CharacterSelect>();
                        },
                        None => {
                            client.send(CharacterJoinResponse::new(CharacterJoinResult::Error {
                                error: CharacterListError::ReachedCapacity, // TODO
                            }));
                        },
                    }
                },
                _ => {},
            }
        }

        if let Some(characters) = character_loader.characters(entity) {
            send_character_list(&client, &characters);
            character_list.0 = Some(characters);
        }
    }
}

fn send_character_list(client: &Client, character_list: &Vec<Character>) {
    let response = CharacterListResponse::new(
        CharacterListAction::List,
        CharacterListResult::Ok {
            content: CharacterListContent::Characters {
                characters: character_list.iter().map(|chara| from_character(chara)).collect(),
                job: 0,
            },
        },
    );
    client.send(response);
}

fn from_character(character: &Character) -> CharacterListEntry {
    let data = &character.character_data;
    CharacterListEntry {
        ref_id: data.character_type as u32,
        name: data.charname.clone(),
        unknown_1: 0,
        unknown_2: 0,
        scale: data.scale as u8,
        level: data.levels as u8,
        exp: data.exp as u64,
        sp: data.sp as u32,
        strength: data.strength as u16,
        intelligence: data.intelligence as u16,
        stat_points: data.stat_points as u16,
        hp: data.current_hp as u32,
        mp: data.current_mp as u32,
        remaining_deletion_time: None,
        region: data.region as u16,
        last_logout: data.last_logout.map(|time| time.as_silkroad_time()).unwrap_or(0),
        guild_member_class: 0,
        guild_rename_required: None,
        academy_member_class: 0,
        equipped_items: character.items.iter().map(|item| from_item(item)).collect(),
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

fn send_spawn(client: &Client, player: &Player, agent: &Agent, position: &Position, max_level: u8) {
    client.send(CharacterSpawnStart);

    let character_data = &player.character;

    client.send(CharacterSpawn::new(
        Utc::now().as_silkroad_time(),
        agent.ref_id,
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
        Vec::new(),
        5,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        agent.id, // TODO
        position.as_protocol(),
        0,
        position.rotation.into(),
        EntityState {
            alive: AliveState::Spawning,
            unknown1: 0,
            action_state: ActionState::None,
            body_state: BodyState::None,
            unknown2: 0,
            walk_speed: 16.0,
            run_speed: 50.0,
            berserk_speed: 100.0,
            active_buffs: vec![],
        },
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
        2,
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
