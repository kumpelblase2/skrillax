use crate::agent::states::{Idle, MovementGoal, Moving, StateTransitionQueue};
use crate::comp::monster::RandomStroll;
use crate::comp::pos::Position;
use crate::ext::Navmesh;
use bevy_ecs::prelude::*;
use bevy_time::Time;
use rand::random;
use silkroad_game_base::{GlobalLocation, Vector2Ext};

pub(crate) fn movement_monster(
    mut query: Query<(&mut RandomStroll, &mut StateTransitionQueue, &Position), With<Idle>>,
    delta: Res<Time>,
    navmesh: Res<Navmesh>,
) {
    let delta = delta.delta();
    for (mut stroll, mut transition, pos) in query.iter_mut() {
        if stroll.check_timer.finished() && random::<f32>() <= 0.1 {
            let new_location = GlobalLocation(stroll.origin.0.random_in_radius(stroll.radius));
            let new_y = navmesh.height_for(new_location).unwrap_or(pos.position().0.y);
            transition.request_transition(Moving(MovementGoal::Location(new_location.with_y(new_y))));
            stroll.check_timer.reset();
        } else {
            stroll.check_timer.tick(delta);
        }
    }
}
