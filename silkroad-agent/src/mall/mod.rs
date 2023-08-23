use crate::mall::event::MallOpenRequestEvent;
use crate::mall::system::{clean_tokens, open_mall};
use bevy_app::{App, Plugin, PostUpdate, Update};

mod db;
pub(crate) mod event;
mod system;

pub(crate) struct MallPlugin;

impl Plugin for MallPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MallOpenRequestEvent>()
            .add_systems(Update, open_mall)
            .add_systems(PostUpdate, clean_tokens);
    }
}
