use crate::comp::net::Client;
use crate::comp::player::StatPoints;
use crate::input::PlayerInputEvent;
use bevy::prelude::*;
use silkroad_protocol::world::{IncreaseInt, IncreaseIntResponse, IncreaseStr, IncreaseStrResponse};

pub(crate) fn increase_stats_int(
    mut query: Query<(&mut StatPoints, &Client)>,
    mut reader: MessageReader<PlayerInputEvent<IncreaseInt>>,
) {
    for event in reader.read() {
        let Ok((mut stat_points, client)) = query.get_mut(event.player) else {
            continue;
        };
        if stat_points.remaining_points() == 0 {
            client.send(IncreaseIntResponse::Failure(0));
            continue;
        }
        stat_points.spend_int();
        client.send(IncreaseIntResponse::Success);
    }
}

pub(crate) fn increase_stats_str(
    mut query: Query<(&mut StatPoints, &Client)>,
    mut reader: MessageReader<PlayerInputEvent<IncreaseStr>>,
) {
    for event in reader.read() {
        let Ok((mut stat_points, client)) = query.get_mut(event.player) else {
            continue;
        };
        if stat_points.remaining_points() == 0 {
            client.send(IncreaseStrResponse::Failure(0));
            continue;
        }
        stat_points.spend_str();
        client.send(IncreaseStrResponse::Success);
    }
}
