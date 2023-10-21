use crate::comp::exp::{Experienced, Leveled};
use crate::comp::pos::Position;
use crate::comp::{Health, Mana};
use bevy_ecs::change_detection::DetectChangesMut;
use bevy_ecs::prelude::Query;

pub(crate) trait Reset {
    fn reset(&mut self);
}

pub(crate) fn reset_tracked_entities(
    mut query: Query<(
        Option<&mut Position>,
        Option<&mut Health>,
        Option<&mut Mana>,
        Option<&mut Experienced>,
        Option<&mut Leveled>,
    )>,
) {
    for (mut pos, mut health, mut mana, mut exp, mut level) in query.iter_mut() {
        if let Some(pos) = &mut pos {
            pos.bypass_change_detection().reset();
        }

        if let Some(health) = &mut health {
            health.bypass_change_detection().reset();
        }

        if let Some(mana) = &mut mana {
            mana.bypass_change_detection().reset();
        }

        if let Some(exp) = &mut exp {
            exp.bypass_change_detection().reset();
        }

        if let Some(leveled) = &mut level {
            leveled.bypass_change_detection().reset();
        }
    }
}
