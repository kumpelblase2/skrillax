use crate::comp::drop::ItemDrop;
use bevy_core::Time;
use bevy_ecs::prelude::*;

pub(crate) fn tick_drop(mut cmd: Commands, time: Res<Time>, mut drops: Query<(Entity, &mut ItemDrop)>) {
    for (entity, mut drop) in drops.iter_mut() {
        drop.despawn_timer.tick(time.delta());
        if drop.despawn_timer.finished() {
            cmd.entity(entity).despawn();
        }
    }
}
