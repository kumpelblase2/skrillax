use crate::agent::goal::{AgentGoal, GoalTracker};
use crate::agent::state::Idle;
use crate::comp::monster::RandomStroll;
use crate::comp::pos::Position;
use crate::ext::Navmesh;
use bevy::prelude::*;
use rand::{thread_rng, Rng};
use silkroad_game_base::{GlobalLocation, Vector2Ext};
use std::time::Duration;

pub(crate) fn movement_monster(
    mut query: Query<(&mut RandomStroll, &mut GoalTracker, &Position), With<Idle>>,
    delta: Res<Time>,
    navmesh: Res<Navmesh>,
) {
    let delta = delta.delta();
    for (mut stroll, mut goal, pos) in query.iter_mut() {
        if goal.has_goal() {
            continue;
        }

        if stroll.check_timer.tick(delta).just_finished() {
            let new_location = GlobalLocation(stroll.origin.0.random_in_radius(stroll.radius));
            let new_y = navmesh.height_for(new_location).unwrap_or(pos.position().0.y);
            goal.switch_goal(AgentGoal::moving_to(new_location.with_y(new_y)));
            let next_move_duration = Duration::from_secs(thread_rng().gen_range(stroll.movement_timer_range.clone()));
            stroll.check_timer = Timer::new(next_move_duration, TimerMode::Once);
        }
    }
}
