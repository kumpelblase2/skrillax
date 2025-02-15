use crate::comp::player::Player;
use crate::comp::pos::Position;
use bevy::prelude::*;
use silkroad_definitions::Region;
use std::collections::HashSet;

#[derive(Default, Resource)]
pub(crate) struct PlayerActivity {
    set: HashSet<Region>,
}

impl PlayerActivity {
    pub(crate) fn is_region_active(&self, region: &Region) -> bool {
        self.set.contains(region)
    }

    pub(crate) fn active_regions(&self) -> impl Iterator<Item = Region> + '_ {
        self.set.iter().copied()
    }
}

pub(crate) fn update_player_activity(mut activity: ResMut<PlayerActivity>, query: Query<&Position, With<Player>>) {
    activity.set = query.iter().map(|pos| pos.position().region()).collect();
}
