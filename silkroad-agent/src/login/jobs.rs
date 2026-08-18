use crate::comp::net::Client;
use crate::comp::Playing;
use crate::login::{
    CharacterCheckName, CharacterCreate, CharacterDelete, CharacterRestore, CharacterSelect, CharactersLoading,
};
use bevy::prelude::*;
use chrono::Utc;
use silkroad_agent_persistence::{CharacterListItem, ListedCharacter};
use silkroad_protocol::character::{
    CharacterListAction, CharacterListContent, CharacterListEntry, CharacterListEquippedItem, CharacterListError,
    CharacterListResponse, CharacterListResult, TimeInformation,
};
use silkroad_protocol::PackedSilkroadTime;
use tokio::sync::oneshot::error::TryRecvError;
use tracing::warn;

pub(crate) fn handle_character_list_received(
    mut query: Query<(Entity, &Client, &Playing, &mut CharacterSelect, &mut CharactersLoading)>,
    mut cmd: Commands,
) {
    for (entity, client, playing, mut character_list, mut loading) in query.iter_mut() {
        match loading.try_recv() {
            Ok(Ok(characters)) => {
                send_character_list(client, &characters);
                character_list.characters = Some(characters);
            },
            Ok(Err(error)) => {
                warn!(id = playing.0.id, ?error, "Could not load characters");
                send_error(
                    client,
                    CharacterListAction::List,
                    CharacterListError::CouldntConnectToServer,
                );
            },
            Err(TryRecvError::Empty) => continue,
            Err(error) => warn!(id = playing.0.id, ?error, "Character loading task was cancelled"),
        }
        cmd.entity(entity).remove::<CharactersLoading>();
    }
}

fn send_character_list(client: &Client, character_list: &[ListedCharacter]) {
    let characters = character_list.iter().map(from_character).collect();
    let response = CharacterListResponse::new(
        CharacterListAction::List,
        CharacterListResult::ok(CharacterListContent::characters(characters, 0)),
    );
    client.send(response);
}

fn from_character(character: &ListedCharacter) -> CharacterListEntry {
    let last_logout = character
        .last_logout
        .and_then(|time| time.try_into().ok())
        .unwrap_or(PackedSilkroadTime::zero());
    let playtime_information = character
        .deletion_end
        .map(|end| (end - Utc::now()).num_minutes().max(0) as u32)
        .map(|remaining| TimeInformation::deleting(last_logout, remaining))
        .unwrap_or_else(|| TimeInformation::playable(last_logout));

    CharacterListEntry {
        ref_id: character.reference_id,
        name: character.name.clone(),
        unknown: String::new(),
        scale: character.scale,
        level: character.level,
        exp: character.experience,
        sp: character.skill_points,
        strength: character.strength,
        intelligence: character.intelligence,
        stat_points: character.stat_points,
        hp: character.current_hp,
        mp: character.current_mp,
        region: character.region,
        playtime_info: playtime_information,
        guild_member_class: 0,
        guild_rename_required: None,
        academy_member_class: 0,
        equipped_items: character.equipped_items.iter().map(from_item).collect(),
        avatar_items: Vec::new(),
    }
}

fn from_item(item: &CharacterListItem) -> CharacterListEquippedItem {
    CharacterListEquippedItem::new(item.reference_id, item.upgrade_level)
}

pub(crate) fn handle_character_name_check(
    mut query: Query<(Entity, &Client, &Playing, &mut CharacterSelect, &mut CharacterCheckName)>,
    mut cmd: Commands,
) {
    for (entity, client, playing, mut character_list, mut check) in query.iter_mut() {
        match check.try_recv() {
            Ok(Ok((name, available))) => {
                let result = if available {
                    CharacterListResult::ok(CharacterListContent::Empty)
                } else {
                    CharacterListResult::error(CharacterListError::NameAlreadyUsed)
                };
                character_list.checked_name = Some(name);
                client.send(CharacterListResponse::new(CharacterListAction::CheckName, result));
            },
            Ok(Err(error)) => {
                warn!(id = playing.0.id, %error, "Could not check character name");
                send_error(
                    client,
                    CharacterListAction::CheckName,
                    CharacterListError::CouldntConnectToServer,
                );
            },
            Err(TryRecvError::Empty) => continue,
            Err(error) => warn!(id = playing.0.id, ?error, "Character name-check task was cancelled"),
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
            Ok(Ok(())) => client.send(CharacterListResponse::new(
                CharacterListAction::Create,
                CharacterListResult::ok(CharacterListContent::Empty),
            )),
            Ok(Err(error)) => {
                warn!(id = playing.0.id, %error, "Could not create character");
                send_error(
                    client,
                    CharacterListAction::Create,
                    CharacterListError::CouldntCreateCharacter,
                );
            },
            Err(TryRecvError::Empty) => continue,
            Err(error) => warn!(id = playing.0.id, ?error, "Character creation task was cancelled"),
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
            Ok(Ok(true)) => client.send(CharacterListResponse::new(
                CharacterListAction::Delete,
                CharacterListResult::ok(CharacterListContent::Empty),
            )),
            Ok(Ok(false)) => send_error(client, CharacterListAction::Delete, CharacterListError::InvalidName),
            Ok(Err(error)) => {
                warn!(id = playing.0.id, %error, "Could not delete character");
                send_error(
                    client,
                    CharacterListAction::Delete,
                    CharacterListError::CouldntConnectToServer,
                );
            },
            Err(TryRecvError::Empty) => continue,
            Err(error) => warn!(id = playing.0.id, ?error, "Character deletion task was cancelled"),
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
            Ok(Ok(true)) => client.send(CharacterListResponse::new(
                CharacterListAction::Restore,
                CharacterListResult::ok(CharacterListContent::Empty),
            )),
            Ok(Ok(false)) => send_error(client, CharacterListAction::Restore, CharacterListError::InvalidName),
            Ok(Err(error)) => {
                warn!(id = playing.0.id, %error, "Could not restore character");
                send_error(
                    client,
                    CharacterListAction::Restore,
                    CharacterListError::CouldntConnectToServer,
                );
            },
            Err(TryRecvError::Empty) => continue,
            Err(error) => warn!(id = playing.0.id, ?error, "Character restoration task was cancelled"),
        }
        cmd.entity(entity).remove::<CharacterRestore>();
    }
}

fn send_error(client: &Client, action: CharacterListAction, error: CharacterListError) {
    client.send(CharacterListResponse::new(action, CharacterListResult::error(error)));
}
