use crate::comp::player::Player;
use crate::comp::Client;
use crate::event::PlayerLevelUp;
use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::*;
use silkroad_data::level::LevelMap;
use std::cmp::max;

pub(crate) fn notify_levelup(
    mut levelups: EventReader<PlayerLevelUp>,
    levels: Res<LevelMap>,
    mut query: Query<(&Client, &mut Player)>,
) {
    for levelup in levelups.iter() {
        if let Ok((_client, mut player)) = query.get_mut(levelup.0) {
            player.character.level = player.character.level + 1;
            player.character.max_level = max(player.character.level, player.character.max_level);
            player.character.exp = max(
                levels.get_exp_for_level(player.character.level).unwrap_or(0),
                player.character.exp,
            );
            // client.send();
        }
    }
}
