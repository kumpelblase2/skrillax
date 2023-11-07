use crate::comp::net::Client;
use crate::comp::player::StatPoints;
use crate::input::PlayerInput;
use bevy_ecs::prelude::*;
use silkroad_game_base::StatType;
use silkroad_protocol::world::{IncreaseIntResponse, IncreaseStrResponse};
use std::mem::take;

pub(crate) fn increase_stats(mut query: Query<(&mut PlayerInput, &mut StatPoints, &Client)>) {
    for (mut input, mut stat_points, client) in query.iter_mut() {
        for stat_increase in take(&mut input.increase_stats) {
            if stat_points.remaining_points() == 0 {
                match stat_increase {
                    StatType::STR => client.send(IncreaseStrResponse::Error(0)),
                    StatType::INT => client.send(IncreaseIntResponse::Error(0)),
                }
                continue;
            }

            match stat_increase {
                StatType::STR => {
                    stat_points.spend_str();
                    client.send(IncreaseStrResponse::Success)
                },
                StatType::INT => {
                    stat_points.spend_int();
                    client.send(IncreaseIntResponse::Success)
                },
            }
        }
    }
}
