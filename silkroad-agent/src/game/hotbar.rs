use crate::comp::skill::Hotbar;
use crate::input::PlayerInputEvent;
use bevy::prelude::*;
use silkroad_protocol::skill::HotbarUpdate;

pub(crate) fn update_hotbar(mut query: Query<&mut Hotbar>, mut reader: MessageReader<PlayerInputEvent<HotbarUpdate>>) {
    for event in reader.read() {
        let Ok(mut hotbar) = query.get_mut(event.player) else {
            continue;
        };
        hotbar.update_entries(&event.input.content);
    }
}
