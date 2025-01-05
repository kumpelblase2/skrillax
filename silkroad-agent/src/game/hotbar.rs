use crate::comp::skill::Hotbar;
use crate::input::PlayerInput;
use bevy_ecs::prelude::*;

pub(crate) fn update_hotbar(mut query: Query<(&PlayerInput, &mut Hotbar)>) {
    for (input, mut hotbar) in query.iter_mut() {
        if let Some(hotbar_update) = input.hotbar.as_deref() {
            hotbar.update_entries(hotbar_update);
        }
    }
}
