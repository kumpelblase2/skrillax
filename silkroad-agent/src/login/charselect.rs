use crate::agent::component::Agent;
use crate::comp::gold::GoldPouch;
use crate::comp::inventory::PlayerInventory;
use crate::comp::net::Client;
use crate::comp::player::{Player, PlayerBundle};
use crate::comp::pos::Position;
use crate::comp::skill::Hotbar;
use crate::comp::visibility::Visibility;
use crate::comp::{GameEntity, Playing};
use crate::config::GameConfig;
use crate::ext::{CharacterPersistenceResource, EntityIdPool};
use crate::input::PlayerInputEvent;
use crate::login::job_distribution::JobDistribution;
use crate::login::{
    CharacterCheckName, CharacterCreate, CharacterDelete, CharacterJoining, CharacterRestore, CharacterSelect,
    CharactersLoading,
};
use crate::population::{LoginQueue, ReservationError};
use crate::server_plugin::ServerId;
use crate::tasks::TaskCreator;
use crate::world::WorldData;
use bevy::prelude::*;
use cgmath::Vector3;
use chrono::{TimeZone, Utc};
use silkroad_agent_persistence::{CharacterOwner, CharacterRace, NewCharacter, NewCharacterItem, WorldJoinError};
use silkroad_data::DataEntry;
use silkroad_game_base::{Heading, ItemTypeData, LocalPosition};
use silkroad_protocol::auth::{AuthRequest, AuthResponse, AuthResult, AuthResultError, UnknownLargePacket};
use silkroad_protocol::character::{
    CharacterJoinRequest, CharacterJoinResponse, CharacterListAction, CharacterListContent, CharacterListError,
    CharacterListRequest, CharacterListRequestAction, CharacterListResponse, CharacterListResult, MacroStatus,
    UnknownPacket, UnknownPacket2, MACRO_POTION,
};
use silkroad_protocol::inventory::{
    BagContent, EquipmentItemContentData, ExpendableItemContentData, InventoryItemBindingData, InventoryItemData,
    ItemContentData, RentInfo,
};
use silkroad_protocol::skill::{HotbarItem, MasteryData, SkillData};
use silkroad_protocol::spawn::{CharacterSpawn, CharacterSpawnEnd, CharacterSpawnStart, JobInformation};
use silkroad_protocol::world::{ActionState, AliveState, BodyState, EntityState};
use silkroad_protocol::PackedSilkroadTime;
use tokio::sync::oneshot::error::TryRecvError;
use tracing::{debug, warn};

pub(crate) fn handle_list_request(
    mut query: Query<(Entity, &Client, &Playing, &mut CharacterSelect)>,
    task_creator: Res<TaskCreator>,
    character_persistence: Res<CharacterPersistenceResource>,
    mut cmd: Commands,
    job_distribution: Res<JobDistribution>,
    server_id: Res<ServerId>,
    settings: Res<GameConfig>,
    mut reader: MessageReader<PlayerInputEvent<CharacterListRequest>>,
) {
    for event in reader.read() {
        let (entity, client, playing, mut character_list) = query.get_mut(event.player).unwrap();
        match &event.input.action {
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
                        CharacterListResult::error(CharacterListError::InvalidCharacterData),
                    ));
                    continue;
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
                let character_persistence = (*character_persistence).clone();
                let task = task_creator.create_task(async move { character_persistence.create(character).await });
                cmd.entity(entity).insert(CharacterCreate(task));
            },
            CharacterListRequestAction::List => {
                let owner = CharacterOwner {
                    user_id: playing.0.id,
                    server_id: server_id.0,
                };
                let character_persistence = (*character_persistence).clone();
                let receiver =
                    task_creator.create_task(async move { character_persistence.character_list(owner).await });
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

                let owner = CharacterOwner {
                    user_id: playing.0.id,
                    server_id: server_id.0,
                };
                let name = character_name.clone();
                let deletion_minutes = settings.deletion_time;
                let character_persistence = (*character_persistence).clone();
                let task = task_creator.create_task(async move {
                    character_persistence
                        .start_deletion(owner, &name, deletion_minutes)
                        .await
                });
                cmd.entity(entity).insert(CharacterDelete(task));
            },
            CharacterListRequestAction::CheckName { character_name } => {
                character_list.checked_name = None;
                let server_id = server_id.0;
                let name = character_name.clone();
                let character_persistence = (*character_persistence).clone();
                let task = task_creator.create_task(async move {
                    let available = character_persistence.is_name_available(server_id, &name).await?;
                    Ok((name, available))
                });
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

                let owner = CharacterOwner {
                    user_id: playing.0.id,
                    server_id: server_id.0,
                };
                let name = character_name.clone();
                let character_persistence = (*character_persistence).clone();
                let task = task_creator.create_task(async move { character_persistence.restore(owner, &name).await });
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

pub(crate) fn handle_join(
    query: Query<(Entity, &Playing, Option<&CharacterJoining>), With<CharacterSelect>>,
    character_persistence: Res<CharacterPersistenceResource>,
    task_creator: Res<TaskCreator>,
    server_id: Res<ServerId>,
    mut cmd: Commands,
    mut reader: MessageReader<PlayerInputEvent<CharacterJoinRequest>>,
) {
    for event in reader.read() {
        let Ok((entity, playing, joining)) = query.get(event.player) else {
            continue;
        };
        if joining.is_some() {
            continue;
        }

        let owner = CharacterOwner {
            user_id: playing.0.id,
            server_id: server_id.0,
        };
        let name = event.input.character_name.clone();
        let character_persistence = (*character_persistence).clone();
        let receiver = task_creator.create_task(async move { character_persistence.world_join(owner, &name).await });
        cmd.entity(entity).insert(CharacterJoining(receiver));
    }
}

pub(crate) fn handle_character_join_received(
    mut query: Query<(Entity, &Client, &Playing, &mut CharacterJoining)>,
    mut cmd: Commands,
    mut allocator: ResMut<EntityIdPool>,
    settings: Res<GameConfig>,
) {
    for (entity, client, playing, mut joining) in query.iter_mut() {
        let loaded = match joining.try_recv() {
            Ok(Ok(character)) => character,
            Ok(Err(WorldJoinError::NotFound)) => {
                client.send(CharacterJoinResponse::error(CharacterListError::FailedToJoinWorld));
                cmd.entity(entity).remove::<CharacterJoining>();
                continue;
            },
            Ok(Err(WorldJoinError::PendingDeletion)) => {
                client.send(CharacterJoinResponse::error(CharacterListError::InvalidName));
                cmd.entity(entity).remove::<CharacterJoining>();
                continue;
            },
            Ok(Err(error)) => {
                warn!(
                    user_id = playing.0.id,
                    ?error,
                    "Could not load character for world join"
                );
                client.send(CharacterJoinResponse::error(CharacterListError::FailedToJoinWorld));
                cmd.entity(entity).remove::<CharacterJoining>();
                continue;
            },
            Err(TryRecvError::Empty) => continue,
            Err(error) => {
                warn!(
                    user_id = playing.0.id,
                    ?error,
                    "Character world-join task was cancelled"
                );
                client.send(CharacterJoinResponse::error(CharacterListError::FailedToJoinWorld));
                cmd.entity(entity).remove::<CharacterJoining>();
                continue;
            },
        };

        let Some(character_data) = WorldData::characters().find_id(loaded.reference_id) else {
            warn!(
                reference_id = loaded.reference_id,
                "Character references unknown world data"
            );
            client.send(CharacterJoinResponse::error(CharacterListError::FailedToJoinWorld));
            cmd.entity(entity).remove::<CharacterJoining>();
            continue;
        };
        let Some(inventory) = PlayerInventory::from_loaded(&loaded.items, 45) else {
            warn!(
                character_id = loaded.id,
                "Character inventory contains unknown item data"
            );
            client.send(CharacterJoinResponse::error(CharacterListError::FailedToJoinWorld));
            cmd.entity(entity).remove::<CharacterJoining>();
            continue;
        };
        let Some(unique_id) = allocator.request_id() else {
            client.send(CharacterJoinResponse::error(CharacterListError::ReachedCapacity));
            cmd.entity(entity).remove::<CharacterJoining>();
            continue;
        };

        let player = Player::from_loaded(playing.0.clone(), &loaded);
        let gold = GoldPouch::new(loaded.gold);
        let hotbar = Hotbar::from_list(&loaded.hotbar);
        let location = loaded.location;
        let position = Position::new(
            LocalPosition(
                (location.region).into(),
                Vector3::new(location.x, location.y, location.z),
            )
            .to_global(),
            Heading::from(location.heading),
        );
        let agent = Agent::from_character_data(character_data);
        let game_entity = GameEntity {
            ref_id: loaded.reference_id,
            unique_id,
        };

        client.send(CharacterJoinResponse::success());
        send_spawn(
            client,
            &game_entity,
            &player,
            &inventory,
            &position,
            settings.max_level,
            &hotbar,
        );
        client.send(MacroStatus::Possible(MACRO_POTION, 0));
        client.send(UnknownLargePacket::known());
        client.send(UnknownPacket::new());
        client.send(UnknownPacket2::new(game_entity.unique_id));

        cmd.entity(entity)
            .insert(PlayerBundle::new(
                player,
                game_entity,
                inventory,
                gold,
                agent,
                position,
                Visibility::with_radius(500.),
                hotbar,
            ))
            .remove::<CharacterSelect>()
            .remove::<CharacterJoining>();
    }
}

pub(crate) fn handle_auth(
    query: Query<(Entity, &Client), Without<Playing>>,
    mut cmd: Commands,
    login_queue: Res<LoginQueue>,
    mut reader: MessageReader<PlayerInputEvent<AuthRequest>>,
) {
    for event in reader.read() {
        let (entity, client) = query.get(event.player).unwrap();
        match login_queue.hand_in_reservation(event.input.token) {
            Ok((token, user)) => {
                debug!(id = ?client.0.id(), token = event.input.token, "Accepted token");
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

fn send_login_result(client: &Client, result: AuthResult) {
    client.send(AuthResponse::new(result))
}

fn has_user_character_with_name(charselect: &CharacterSelect, character_name: &str) -> bool {
    charselect
        .characters
        .as_ref()
        .map(|chars| chars.iter().any(|character| character.name == character_name))
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
    hotbar: &Hotbar,
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
            content_data: match item.type_data {
                ItemTypeData::Equipment { upgrade_level } => ItemContentData::new_equipment(
                    item.reference.ref_id(),
                    EquipmentItemContentData::new(
                        upgrade_level,
                        item.variance.unwrap_or_default(),
                        1,
                        vec![],
                        InventoryItemBindingData::new(1, 0),
                        InventoryItemBindingData::new(2, 0),
                        InventoryItemBindingData::new(3, 0),
                        InventoryItemBindingData::new(4, 0),
                    ),
                ),
                ItemTypeData::Consumable { amount } => {
                    ItemContentData::new_expendable(item.reference.ref_id(), ExpendableItemContentData::new(amount))
                },
                _ => panic!("Missing inventory type representation."),
            },
        })
        .collect();

    let skill_data = WorldData::skills();

    client.send(CharacterSpawn::new(
        PackedSilkroadTime::zero(),
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
                        flag: !reached_max as u8,
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
        false,
        0,
        0xFF,
        player.user.id as u32,
        character_data.gm,
        hotbar
            .used_entries()
            .map(|(slot, kind, data)| HotbarItem {
                slot,
                action_flag: kind,
                action_data: data,
            })
            .collect(),
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
) -> NewCharacter {
    let item = |reference_id, slot| NewCharacterItem {
        reference_id,
        upgrade_level: 0,
        variance: None,
        slot,
        amount: 1,
    };

    NewCharacter {
        owner: CharacterOwner { user_id, server_id },
        name: character_name,
        race: if ref_id > 2000 {
            CharacterRace::European
        } else {
            CharacterRace::Chinese
        },
        reference_id: ref_id,
        scale,
        level: 1,
        max_level: 1,
        strength: 20,
        intelligence: 20,
        stat_points: 0,
        current_hp: 200,
        current_mp: 200,
        x: 739.,
        y: 37.4519,
        z: 1757.,
        region: 24998,
        beginner_mark: true,
        gold: 5_000_000,
        items: vec![item(chest, 1), item(pants, 4), item(boots, 5), item(weapon, 6)],
    }
}
