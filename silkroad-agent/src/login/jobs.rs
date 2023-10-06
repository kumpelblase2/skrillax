use crate::comp::net::Client;
use crate::comp::Playing;
use crate::db::character::CharacterItem;
use crate::login::character_loader::DbCharacter;
use crate::login::{
    CharacterCheckName, CharacterCreate, CharacterDelete, CharacterRestore, CharacterSelect, CharactersLoading,
};
use bevy_ecs::prelude::*;
use chrono::Utc;
use silkroad_protocol::character::{
    CharacterListAction, CharacterListContent, CharacterListEntry, CharacterListEquippedItem, CharacterListError,
    CharacterListResponse, CharacterListResult, TimeInformation,
};
use silkroad_protocol::SilkroadTime;
use tokio::sync::oneshot::error::TryRecvError;
use tracing::warn;

pub(crate) fn handle_character_list_received(
    mut query: Query<(Entity, &Client, &Playing, &mut CharacterSelect, &mut CharactersLoading)>,
    mut cmd: Commands,
) {
    for (entity, client, playing, mut character_list, mut loading) in query.iter_mut() {
        match loading.try_recv() {
            Ok(characters) => {
                send_character_list(client, &characters);
                character_list.characters = Some(characters);
            },
            Err(TryRecvError::Empty) => continue,
            Err(e) => {
                warn!(id = playing.0.id, "Error when loading characters. {:?}", e);
            },
        }
        cmd.entity(entity).remove::<CharactersLoading>();
    }
}

fn send_character_list(client: &Client, character_list: &[DbCharacter]) {
    let characters = character_list.iter().map(from_character).collect();
    let response = CharacterListResponse::new(
        CharacterListAction::List,
        CharacterListResult::ok(CharacterListContent::characters(characters, 0)),
    );
    client.send(response);
}

fn from_character(character: &DbCharacter) -> CharacterListEntry {
    let data = &character.character_data;
    let last_logout = data.last_logout.map(SilkroadTime::from).unwrap_or_default();
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
        equipped_items: character
            .items
            .iter()
            .filter(|item| item.slot < 13)
            .map(from_item)
            .collect(),
        avatar_items: Vec::new(),
    }
}

fn from_item(item: &CharacterItem) -> CharacterListEquippedItem {
    CharacterListEquippedItem::new(item.item_obj_id as u32, item.upgrade_level as u8)
}

pub(crate) fn handle_character_name_check(
    mut query: Query<(Entity, &Client, &Playing, &mut CharacterSelect, &mut CharacterCheckName)>,
    mut cmd: Commands,
) {
    for (entity, client, playing, mut character_list, mut check) in query.iter_mut() {
        match check.try_recv() {
            Ok((name, available)) => {
                let result = if available {
                    CharacterListResult::ok(CharacterListContent::Empty)
                } else {
                    CharacterListResult::error(CharacterListError::NameAlreadyUsed)
                };
                character_list.checked_name = Some(name);
                client.send(CharacterListResponse::new(CharacterListAction::CheckName, result));
            },
            Err(TryRecvError::Empty) => continue,
            Err(e) => {
                warn!(id = playing.0.id, "Error when checking for name. {:?}", e);
            },
        }
        cmd.entity(entity).remove::<CharacterCheckName>();
    }
}

pub(crate) fn handle_character_create(
    mut query: Query<(Entity, &Client, &Playing, &mut CharacterCreate)>,
    mut cmd: Commands,
) {
    for (entity, client, playing, mut create) in query.iter_mut() {
        match create.try_recv() {
            Ok(_) => {
                client.send(CharacterListResponse::new(
                    CharacterListAction::Create,
                    CharacterListResult::ok(CharacterListContent::Empty),
                ));
            },
            Err(TryRecvError::Empty) => continue,
            Err(e) => {
                warn!(id = playing.0.id, "Error when creating character. {:?}", e);
            },
        }
        cmd.entity(entity).remove::<CharacterCreate>();
    }
}

pub(crate) fn handle_character_delete(
    mut query: Query<(Entity, &Client, &Playing, &mut CharacterDelete)>,
    mut cmd: Commands,
) {
    for (entity, client, playing, mut delete) in query.iter_mut() {
        match delete.try_recv() {
            Ok(success) => {
                if success {
                    client.send(CharacterListResponse::new(
                        CharacterListAction::Delete,
                        CharacterListResult::ok(CharacterListContent::Empty),
                    ));
                } else {
                    // TODO
                }
            },
            Err(TryRecvError::Empty) => continue,
            Err(e) => {
                warn!(id = playing.0.id, "Error when deleting character. {:?}", e);
            },
        }
        cmd.entity(entity).remove::<CharacterDelete>();
    }
}

pub(crate) fn handle_character_restore(
    mut query: Query<(Entity, &Client, &Playing, &mut CharacterRestore)>,
    mut cmd: Commands,
) {
    for (entity, client, playing, mut restore) in query.iter_mut() {
        match restore.try_recv() {
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
            },
            Err(TryRecvError::Empty) => continue,
            Err(e) => {
                warn!(id = playing.0.id, "Error when restoring character. {:?}", e);
            },
        }
        cmd.entity(entity).remove::<CharacterRestore>();
    }
}
