use crate::comp::sync::Synchronize;
use bevy_ecs::prelude::*;
use bevy_time::{Time, Timer, TimerMode};
use silkroad_protocol::world::{AliveState, UpdatedState};
use std::time::Duration;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Dead {
    despawn: Option<Timer>,
}

impl Dead {
    pub fn new_player() -> Self {
        Dead { despawn: None }
    }

    pub fn new_monster() -> Self {
        Dead {
            despawn: Some(Timer::new(Duration::from_secs(5), TimerMode::Once)),
        }
    }

    pub fn should_despawn(&mut self, delta: Duration) -> bool {
        return if let Some(timer) = self.despawn.as_mut() {
            timer.tick(delta).just_finished()
        } else {
            false
        };
    }
}

pub fn broadcast_dead(mut query: Query<&mut Synchronize, Added<Dead>>) {
    for mut sync in query.iter_mut() {
        sync.state.push(UpdatedState::Life(AliveState::Dead))
    }
}

pub fn dead(mut cmd: Commands, mut query: Query<(Entity, &mut Dead)>, time: Res<Time>) {
    let delta = time.delta();
    for (entity, mut dead) in query.iter_mut() {
        if dead.should_despawn(delta) {
            cmd.entity(entity).despawn();
        }
    }
}
