use crate::comp::player::Player;
use crate::comp::pos::Position;
use bevy_ecs::prelude::*;
use std::collections::HashSet;

#[derive(Default, Resource)]
pub(crate) struct PlayerActivity {
    pub(crate) set: HashSet<u16>,
}

pub(crate) fn reset_player_activity(mut activity: ResMut<PlayerActivity>) {
    activity.set.clear();
}

pub(crate) fn update_player_activity(mut activity: ResMut<PlayerActivity>, query: Query<&Position, With<Player>>) {
    query.iter().for_each(|pos| {
        activity.set.insert(pos.location.region().id());
    });
}
