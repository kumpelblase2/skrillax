mod comp;
mod sys;

use crate::persist::sys::{attach_persistence, run_exit_persistence, run_persistence};
use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::prelude::*;
pub use comp::*;

pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(attach_persistence.in_base_set(CoreSet::PostUpdate))
            .add_system(run_persistence.in_base_set(CoreSet::Last))
            .add_system(run_exit_persistence.in_base_set(CoreSet::Last));
    }
}
