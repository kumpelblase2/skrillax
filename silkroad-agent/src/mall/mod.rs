use crate::mall::event::MallOpenRequestEvent;
use crate::mall::system::{clean_tokens, open_mall};
use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::prelude::*;

mod db;
pub(crate) mod event;
mod system;

pub(crate) struct MallPlugin;

impl Plugin for MallPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MallOpenRequestEvent>()
            .add_system(open_mall.in_base_set(CoreSet::Update))
            .add_system(clean_tokens.in_base_set(CoreSet::PostUpdate));
    }
}
