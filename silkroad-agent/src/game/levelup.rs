use crate::comp::player::Player;
use crate::comp::Client;
use crate::event::PlayerLevelUp;
use crate::world::LEVELS;
use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::*;
use std::cmp::max;

pub(crate) fn notify_levelup(mut levelups: EventReader<PlayerLevelUp>, mut query: Query<(&Client, &mut Player)>) {
    for levelup in levelups.iter() {
        if let Ok((_client, mut player)) = query.get_mut(levelup.0) {
            player.character.level += 1;
            player.character.max_level = max(player.character.level, player.character.max_level);
            player.character.exp = max(
                LEVELS
                    .get()
                    .unwrap()
                    .get_exp_for_level(player.character.level)
                    .unwrap_or(0),
                player.character.exp,
            );
            // client.send();
        }
    }
}
