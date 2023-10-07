use crate::comp::drop::{Drop, DropBundle};
use crate::comp::pos::Position;
use crate::comp::{Despawn, EntityReference, GameEntity};
use crate::ext::{EntityIdPool, Navmesh};
use bevy_ecs::prelude::*;
use bevy_time::Time;
use derive_more::Constructor;
use rand::Rng;
use silkroad_data::DataEntry;
use silkroad_game_base::{GlobalLocation, GlobalPosition, Heading, Item, Vector2Ext};

#[derive(Constructor, Event)]
pub(crate) struct SpawnDrop {
    pub item: Item,
    pub relative_position: GlobalLocation,
    pub owner: Option<EntityReference>,
}

pub(crate) fn tick_drop(mut cmd: Commands, time: Res<Time>, mut drops: Query<(Entity, &mut Despawn)>) {
    for (entity, mut despawn) in drops.iter_mut() {
        despawn.0.tick(time.delta());
        if despawn.0.finished() {
            cmd.entity(entity).despawn();
        }
    }
}

pub(crate) fn create_drops(
    mut reader: EventReader<SpawnDrop>,
    navmesh: Res<Navmesh>,
    mut id_gen: ResMut<EntityIdPool>,
    mut cmd: Commands,
) {
    for spawn in reader.iter() {
        let pos = random_position_around(&navmesh, spawn.relative_position, 2.0);
        let drop_id = id_gen.request_id().expect("Should be able to generate an id");
        let rotation = rand::thread_rng().gen_range(0..360) as f32;

        cmd.spawn(DropBundle {
            drop: Drop {
                owner: spawn.owner,
                item: spawn.item,
            },
            position: Position {
                location: pos,
                rotation: Heading(rotation),
            },
            game_entity: GameEntity {
                unique_id: drop_id,
                ref_id: spawn.item.reference.ref_id(),
            },
            despawn: spawn.item.reference.common.despawn_time.into(),
        });
    }
}

fn random_position_around(navmesh: &Navmesh, origin: GlobalLocation, radius: f32) -> GlobalPosition {
    let drop_position = origin.random_in_radius(radius);
    let local_drop_pos = GlobalLocation(drop_position).to_local();
    let drop_position = drop_position.with_height(navmesh.height_for(local_drop_pos).unwrap_or(0.0f32));
    GlobalPosition(drop_position)
}
