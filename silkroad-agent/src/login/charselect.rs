use crate::agent::Agent;
use crate::comp::gold::GoldPouch;
use crate::comp::inventory::PlayerInventory;
use crate::comp::net::Client;
use crate::comp::player::{Player, PlayerBundle};
use crate::comp::pos::Position;
use crate::comp::visibility::Visibility;
use crate::comp::{GameEntity, Playing};
use crate::config::GameConfig;
use crate::db::character::{CharacterData, CharacterItem, DbRace};
use crate::ext::{DbPool, EntityIdPool};
use crate::input::LoginInput;
use crate::login::character_loader::DbCharacter;
use crate::login::job_distribution::JobDistribution;
use crate::login::{
    CharacterCheckName, CharacterCreate, CharacterDelete, CharacterRestore, CharacterSelect, CharactersLoading,
};
use crate::population::{LoginQueue, ReservationError};
use crate::server_plugin::ServerId;
use crate::tasks::TaskCreator;
use crate::world::WorldData;
use bevy_ecs::prelude::*;
use cgmath::Vector3;
use chrono::{TimeZone, Utc};
use silkroad_data::DataEntry;
use silkroad_game_base::{Heading, ItemTypeData, LocalPosition};
use silkroad_protocol::auth::{AuthResponse, AuthResult, AuthResultError};
use silkroad_protocol::character::{
    CharacterJoinResponse, CharacterListAction, CharacterListContent, CharacterListError, CharacterListRequestAction,
    CharacterListResponse, CharacterListResult, MacroStatus, UnknownPacket, UnknownPacket2, MACRO_POTION,
};
use silkroad_protocol::inventory::{
    BagContent, InventoryItemBindingData, InventoryItemContentData, InventoryItemData, RentInfo,
};
use silkroad_protocol::login::UnknownLargePacket;
use silkroad_protocol::skill::{MasteryData, SkillData};
use silkroad_protocol::spawn::{CharacterSpawn, CharacterSpawnEnd, CharacterSpawnStart, JobInformation};
use silkroad_protocol::world::{ActionState, AliveState, BodyState, EntityState};
use silkroad_protocol::SilkroadTime;
use tracing::debug;

pub(crate) fn handle_list_request(
    mut query: Query<(Entity, &Client, &Playing, &LoginInput, &mut CharacterSelect)>,
    task_creator: Res<TaskCreator>,
    pool: Res<DbPool>,
    mut cmd: Commands,
    job_distribution: Res<JobDistribution>,
    server_id: Res<ServerId>,
    settings: Res<GameConfig>,
) {
    for (entity, client, playing, input, mut character_list) in query.iter_mut() {
        for action in input.list.iter() {
            match action {
                CharacterListRequestAction::Create {
                    character_name,
                    ref_id,
                    scale,
                    chest,
                    pants,
                    boots,
                    weapon,
                } => {
                    if !can_create_character_with_name(&character_list, character_name) {
                        debug!(id = ?client.0.id(), "Tried to create character without checking name first.");
                        client.send(CharacterListResponse::new(
                            CharacterListAction::Create,
                            CharacterListResult::error(CharacterListError::InvalidCharacterData),
                        ));
                    }

                    let character = create_character_from(
                        playing.0.id,
                        server_id.0,
                        character_name.clone(),
                        *ref_id,
                        *scale,
                        *chest,
                        *pants,
                        *boots,
                        *weapon,
                    );
                    let task = task_creator.create_task(DbCharacter::create_character(character, pool.clone()));
                    cmd.entity(entity).insert(CharacterCreate(task));
                },
                CharacterListRequestAction::List => {
                    let receiver = task_creator.create_task(DbCharacter::load_characters_sparse(
                        playing.0.id,
                        server_id.0,
                        pool.clone(),
                    ));
                    cmd.entity(entity).insert(CharactersLoading(receiver));
                },
                CharacterListRequestAction::Delete { character_name } => {
                    if !has_user_character_with_name(&character_list, character_name) {
                        client.send(CharacterListResponse::new(
                            CharacterListAction::Delete,
                            CharacterListResult::error(CharacterListError::InvalidName),
                        ));
                        continue;
                    }

                    let task = task_creator.create_task(DbCharacter::start_delete_character(
                        playing.0.id,
                        character_name.clone(),
                        server_id.0,
                        settings.deletion_time,
                        pool.clone(),
                    ));
                    cmd.entity(entity).insert(CharacterDelete(task));
                },
                CharacterListRequestAction::CheckName { character_name } => {
                    character_list.checked_name = None;
                    let server_id = server_id.0;
                    let task = task_creator.create_task(CharacterData::check_name_available(
                        character_name.clone(),
                        server_id,
                        pool.clone(),
                    ));
                    cmd.entity(entity).insert(CharacterCheckName(task));
                },
                CharacterListRequestAction::Restore { character_name } => {
                    if !has_user_character_with_name(&character_list, character_name) {
                        client.send(CharacterListResponse::new(
                            CharacterListAction::Delete,
                            CharacterListResult::error(CharacterListError::InvalidName),
                        ));
                        continue;
                    }

                    let task = task_creator.create_task(DbCharacter::restore_character(
                        playing.0.id,
                        character_name.clone(),
                        server_id.0,
                        pool.clone(),
                    ));
                    cmd.entity(entity).insert(CharacterRestore(task));
                },
                CharacterListRequestAction::ShowJobSpread => {
                    let (hunter_perc, thief_perc) = job_distribution.spread();
                    send_job_spread(client, hunter_perc, thief_perc);
                },
                CharacterListRequestAction::AssignJob { .. } => {},
            }
        }
    }
}

pub(crate) fn handle_join(
    query: Query<(Entity, &Client, &LoginInput, &CharacterSelect, &Playing)>,
    mut cmd: Commands,
    mut allocator: ResMut<EntityIdPool>,
    settings: Res<GameConfig>,
) {
    for (entity, client, input, character_list, playing) in query.iter() {
        if let Some(ref join) = input.join {
            match character_list.characters {
                Some(ref characters) => {
                    let character = characters
                        .iter()
                        .find(|char| char.character_data.charname == join.character_name)
                        .unwrap();

                    let Some(character_data) =
                        WorldData::characters().find_id(character.character_data.character_type as u32)
                    else {
                        client.send(CharacterJoinResponse::error(CharacterListError::FailedToJoinWorld));
                        continue;
                    };

                    if character.character_data.deletion_end.is_some() {
                        client.send(CharacterJoinResponse::error(CharacterListError::InvalidName));
                        continue;
                    }

                    let mut player = Player::from_db_data(playing.0.clone(), &character.character_data);
                    let inventory = PlayerInventory::from_db(&character.items, 45);
                    let gold = GoldPouch::new(character.character_data.gold as u64);

                    player.character.masteries = character
                        .masteries
                        .iter()
                        .map(|mastery| (mastery.mastery_id as u32, mastery.level as u8))
                        .collect();

                    player.character.skills = character
                        .skills
                        .iter()
                        .map(|skill| (skill.skill_group_id as u32, skill.level as u8))
                        .collect();

                    let data = &character.character_data;

                    let pos =
                        LocalPosition((data.region as u16).into(), Vector3::new(data.x, data.y, data.z)).to_global();
                    let position = Position::new(pos, Heading::from(data.rotation as u16));

                    let agent = Agent::from_character_data(character_data);

                    let game_entity = GameEntity {
                        ref_id: data.character_type as u32,
                        unique_id: allocator.request_id().unwrap(),
                    };

                    client.send(CharacterJoinResponse::success());

                    send_spawn(client, &game_entity, &player, &inventory, &position, settings.max_level);

                    client.send(MacroStatus::Possible(
                        MACRO_POTION, /*| MACRO_HUNT | MACRO_SKILL*/
                        0,
                    ));
                    client.send(UnknownLargePacket::new());
                    client.send(UnknownPacket::new());
                    client.send(UnknownPacket2::new(game_entity.unique_id));

                    cmd.entity(entity)
                        .insert(PlayerBundle::new(
                            player,
                            game_entity,
                            inventory,
                            gold,
                            agent,
                            position.clone(),
                            Visibility::with_radius(500.),
                        ))
                        .remove::<CharacterSelect>()
                        .remove::<LoginInput>();
                },
                None => {
                    // TODO
                    client.send(CharacterJoinResponse::error(CharacterListError::ReachedCapacity));
                },
            }
        }
    }
}

pub(crate) fn handle_auth(
    query: Query<(Entity, &Client, &LoginInput), Without<Playing>>,
    mut cmd: Commands,
    login_queue: Res<LoginQueue>,
) {
    for (entity, client, input) in query.iter() {
        if let Some(ref auth) = input.auth {
            match login_queue.hand_in_reservation(auth.token) {
                Ok((token, user)) => {
                    debug!(id = ?client.0.id(), token = auth.token, "Accepted token");
                    cmd.entity(entity)
                        .insert(Playing(user, token))
                        .insert(CharacterSelect::default());
                    send_login_result(client, AuthResult::success());
                    break;
                },
                Err(err) => match err {
                    ReservationError::NoSuchToken | ReservationError::AlreadyHasReservation => {
                        send_login_result(client, AuthResult::error(AuthResultError::InvalidData));
                    },
                    ReservationError::NoSpotsAvailable | ReservationError::AllTokensTaken => {
                        send_login_result(client, AuthResult::error(AuthResultError::ServerFull));
                    },
                },
            }
        }
    }
}

fn send_login_result(client: &Client, result: AuthResult) {
    client.send(AuthResponse::new(result))
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
        if checked_name != name {
            return false;
        }
        true
    } else {
        false
    }
}

fn send_job_spread(client: &Client, hunters: u8, thieves: u8) {
    client.send(CharacterListResponse::new(
        CharacterListAction::ShowJobSpread,
        CharacterListResult::ok(CharacterListContent::jobspread(hunters, thieves)),
    ));
}

fn send_spawn(
    client: &Client,
    entity: &GameEntity,
    player: &Player,
    inventory: &PlayerInventory,
    position: &Position,
    max_level: u8,
) {
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

    let inventory_items = inventory
        .items()
        .map(|(slot, item)| InventoryItemData {
            slot: *slot,
            rent_data: RentInfo::Empty,
            item_id: item.reference.ref_id(),
            content_data: match item.type_data {
                ItemTypeData::Equipment { upgrade_level } => InventoryItemContentData::Equipment {
                    plus_level: upgrade_level,
                    variance: item.variance.unwrap_or_default(),
                    durability: 1,
                    magic: vec![],
                    bindings_1: InventoryItemBindingData::new(1, 0),
                    bindings_2: InventoryItemBindingData::new(2, 0),
                    bindings_3: InventoryItemBindingData::new(3, 0),
                    bindings_4: InventoryItemBindingData::new(4, 0),
                },
                ItemTypeData::Consumable { amount } => InventoryItemContentData::Expendable { stack_size: amount },
                _ => panic!("Missing inventory type representation."),
            },
        })
        .collect();

    let skill_data = WorldData::skills();

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
        Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap(),
        0,
        max_level,
        BagContent::new(inventory.size() as u8, inventory_items),
        BagContent::new(5, Vec::new()),
        player
            .character
            .masteries
            .iter()
            .map(|(mastery, level)| MasteryData {
                id: *mastery,
                level: *level,
            })
            .collect(),
        player
            .character
            .skills
            .iter()
            .flat_map(|(group, level)| {
                let skills_of_group = skill_data
                    .iter()
                    .filter(|skill_ref| skill_ref.group == *group)
                    .collect::<Vec<_>>();

                let max = skills_of_group
                    .iter()
                    .map(|skill_ref| skill_ref.level)
                    .max()
                    .unwrap_or(1);
                let reached_max = max == *level;

                skills_of_group
                    .into_iter()
                    .filter(|skill_ref| skill_ref.level <= *level)
                    .map(|skill_ref| SkillData {
                        id: skill_ref.ref_id,
                        enabled: !reached_max,
                    })
                    .collect::<Vec<_>>()
            })
            .collect(),
        Vec::new(),
        Vec::new(),
        entity.unique_id,
        position.as_protocol(),
        position.as_standing(),
        entity_state,
        character_data.name.clone(),
        JobInformation::empty(),
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
) -> DbCharacter {
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
        race: if ref_id > 2000 {
            DbRace::European
        } else {
            DbRace::Chinese
        },
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
    // TODO: properly pick masteries
    DbCharacter {
        character_data: character,
        items,
        masteries: vec![],
        skills: vec![],
    }
}
