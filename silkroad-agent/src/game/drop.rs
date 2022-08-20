use crate::comp::Despawn;
use bevy_core::Time;
use bevy_ecs::prelude::*;

pub(crate) fn tick_drop(mut cmd: Commands, time: Res<Time>, mut drops: Query<(Entity, &mut Despawn)>) {
    for (entity, mut despawn) in drops.iter_mut() {
        despawn.0.tick(time.delta());
        if despawn.0.finished() {
            cmd.entity(entity).despawn();
        }
    }
}
