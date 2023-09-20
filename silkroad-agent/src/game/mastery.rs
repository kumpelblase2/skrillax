use crate::comp::net::Client;
use crate::comp::player::Player;
use crate::input::PlayerInput;
use crate::world::WorldData;
use bevy_ecs::prelude::*;
use silkroad_protocol::skill::{LevelUpMasteryError, LevelUpMasteryResponse};
use silkroad_protocol::world::CharacterPointsUpdate;

pub(crate) fn handle_mastery_levelup(mut query: Query<(&Client, &mut Player, &mut PlayerInput)>) {
    let masteries = WorldData::masteries();
    let levels = WorldData::levels();
    for (client, mut player, mut input) in query.iter_mut() {
        if let Some(mastery_levelup) = input.mastery.take() {
            let Some(mastery) = masteries.find_id(mastery_levelup.mastery) else {
                client.send(LevelUpMasteryResponse::Error(LevelUpMasteryError::InsufficientSP)); // TODO
                continue;
            };

            let current_level = player
                .character
                .masteries
                .iter()
                .find(|(mastery, _)| mastery.ref_id == mastery_levelup.mastery as u16)
                .map(|(_, level)| level)
                .copied()
                .unwrap_or(0);

            let required_sp = levels.get_mastery_sp_for_level(current_level).unwrap_or(0);

            if player.character.sp < required_sp {
                client.send(LevelUpMasteryResponse::Error(LevelUpMasteryError::InsufficientSP));
                continue;
            }

            if required_sp > 0 {
                player.character.sp -= required_sp;
                client.send(CharacterPointsUpdate::SP {
                    amount: player.character.sp,
                    display: false,
                });
            }

            let next_level = current_level.checked_add(1).unwrap_or(u8::MAX);
            if let Some(position) = player
                .character
                .masteries
                .iter()
                .position(|(mastery, _)| mastery.ref_id == mastery_levelup.mastery as u16)
            {
                player.character.masteries[position] = (mastery, next_level);
            } else {
                player.character.masteries.push((mastery, next_level));
            };
            client.send(LevelUpMasteryResponse::Success {
                mastery: mastery_levelup.mastery,
                new_level: next_level,
            });
        }
    }
}
