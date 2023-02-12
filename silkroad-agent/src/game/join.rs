use crate::comp::net::Client;
use crate::comp::player::Player;
use crate::comp::sync::Synchronize;
use crate::comp::GameEntity;
use crate::config::GameConfig;
use crate::event::LoadingFinishedEvent;
use crate::game::daylight::DaylightCycle;
use bevy_ecs::prelude::*;
use silkroad_game_base::{Character, SpawningState};
use silkroad_protocol::character::CharacterStatsMessage;
use silkroad_protocol::chat::{ChatSource, ChatUpdate, TextCharacterInitialization};
use silkroad_protocol::world::{
    AliveState, CelestialUpdate, CharacterFinished, FriendListGroup, FriendListInfo, UpdatedState,
};
use tracing::debug;

pub(crate) fn load_finished(
    mut reader: EventReader<LoadingFinishedEvent>,
    settings: Res<GameConfig>,
    daycycle: Res<DaylightCycle>,
    mut query: Query<(&Client, &GameEntity, &mut Player, &mut Synchronize)>,
) {
    for event in reader.iter() {
        let (client, game_entity, mut player, mut sync) = match query.get_mut(event.0) {
            Ok(data) => data,
            _ => continue,
        };

        debug!(id = ?client.0.id(), "Finished loading.");
        player.character.state = SpawningState::Finished;
        sync.state.push(UpdatedState::Life(AliveState::Alive));
        send_character_stats(client, &player.character);
        send_text_initialization(client);
        let (hour, minute) = daycycle.time();
        client.send(CelestialUpdate {
            unique_id: game_entity.unique_id,
            moon_position: daycycle.moon(),
            hour,
            minute,
        });
        client.send(CharacterFinished::default());
        client.send(FriendListInfo {
            groups: vec![FriendListGroup::not_assigned()],
            friends: vec![],
        });

        if let Some(notice) = &settings.join_notice {
            client.send(ChatUpdate::new(ChatSource::Notice, notice.clone()));
        }
    }
}

fn send_character_stats(client: &Client, character: &Character) {
    client.send(CharacterStatsMessage::new(
        100,
        100,
        100,
        100,
        100,
        100,
        100,
        100,
        character.max_hp(),
        character.max_mp(),
        character.stats.strength(),
        character.stats.intelligence(),
    ));
}

fn send_text_initialization(client: &Client) {
    let mut characters = Vec::new();
    for i in 0x1d..0x8c {
        if i < 0x85 || i >= 0x89 {
            characters.push((i as u64) << 56);
        }
    }

    client.send(TextCharacterInitialization::new(characters));
}
