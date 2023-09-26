use crate::comp::net::Client;
use crate::comp::player::Player;
use crate::input::PlayerInput;
use bevy_ecs::prelude::*;
use silkroad_game_base::StatType;
use silkroad_protocol::character::CharacterStatsMessage;
use silkroad_protocol::world::{IncreaseIntResponse, IncreaseStrResponse};
use std::mem::take;

pub(crate) fn increase_stats(mut query: Query<(&mut PlayerInput, &mut Player, &Client)>) {
    for (mut input, mut player, client) in query.iter_mut() {
        for stat_increase in take(&mut input.increase_stats) {
            if player.character.stat_points == 0 {
                match stat_increase {
                    StatType::STR => client.send(IncreaseStrResponse::Error(0)),
                    StatType::INT => client.send(IncreaseIntResponse::Error(0)),
                }
                continue;
            }

            player.character.stat_points -= 1;

            match stat_increase {
                StatType::STR => {
                    player.character.stats.increase_strength(1);
                    client.send(IncreaseStrResponse::Success)
                },
                StatType::INT => {
                    player.character.stats.increase_intelligence(1);
                    client.send(IncreaseIntResponse::Success)
                },
            }

            client.send(CharacterStatsMessage {
                phys_attack_min: 100,
                phys_attack_max: 100,
                mag_attack_min: 100,
                mag_attack_max: 100,
                phys_defense: 100,
                mag_defense: 100,
                hit_rate: 100,
                parry_rate: 100,
                max_hp: player.character.stats.max_health(player.character.level),
                max_mp: player.character.stats.max_mana(player.character.level),
                strength: player.character.stats.strength(),
                intelligence: player.character.stats.intelligence(),
            });
        }
    }
}
