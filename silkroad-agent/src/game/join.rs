use crate::comp::exp::Leveled;
use crate::comp::net::Client;
use crate::comp::player::{Player, StatPoints};
use crate::comp::GameEntity;
use crate::config::GameConfig;
use crate::event::LoadingFinishedEvent;
use crate::game::daylight::DaylightCycle;
use bevy_ecs::prelude::*;
use silkroad_game_base::SpawningState;
use silkroad_protocol::character::CharacterStatsMessage;
use silkroad_protocol::chat::{ChatSource, ChatUpdate, TextCharacterInitialization};
use silkroad_protocol::community::{FriendListGroup, FriendListInfo};
use silkroad_protocol::world::{CelestialUpdate, CharacterFinished};
use tracing::debug;

pub(crate) fn load_finished(
    mut reader: EventReader<LoadingFinishedEvent>,
    settings: Res<GameConfig>,
    daycycle: Res<DaylightCycle>,
    mut query: Query<(&Client, &GameEntity, &mut Player, &Leveled, &StatPoints)>,
) {
    for event in reader.read() {
        let (client, game_entity, mut player, level, stat_points) = match query.get_mut(event.0) {
            Ok(data) => data,
            _ => continue,
        };

        debug!(id = ?client.0.id(), "Finished loading.");
        player.character.state = SpawningState::Finished;
        send_character_stats(client, stat_points, level.current_level());
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

fn send_character_stats(client: &Client, stat_points: &StatPoints, level: u8) {
    client.send(CharacterStatsMessage::new(
        100,
        100,
        100,
        100,
        100,
        100,
        100,
        100,
        stat_points.stats().max_health(level),
        stat_points.stats().max_mana(level),
        stat_points.stats().strength(),
        stat_points.stats().intelligence(),
    ));
}

fn send_text_initialization(client: &Client) {
    let mut characters = Vec::new();
    for i in 0x1d..0x8cu64 {
        if i < 0x85 || i >= 0x89 {
            characters.push(i << 56);
        }
    }

    client.send(TextCharacterInitialization::new(characters));
}
