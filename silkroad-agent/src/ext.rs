use bevy_ecs_macros::Resource;
use derive_more::{Deref, DerefMut, From};
use id_pool::IdPool;
use silkroad_data::npc_pos::NpcPosition;
use silkroad_game_base::LocalLocation;
use silkroad_navmesh::GlobalNavmesh;
use silkroad_network::server::SilkroadServer;
use sqlx::PgPool;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Resource, Deref, DerefMut, From)]
pub struct EntityIdPool(IdPool);

impl Default for EntityIdPool {
    fn default() -> Self {
        EntityIdPool(IdPool::new())
    }
}

#[derive(Resource, Deref, DerefMut, From)]
pub struct DbPool(PgPool);

#[derive(Resource, Deref, From)]
pub struct Navmesh(GlobalNavmesh);

impl Navmesh {
    fn height_for_location(&self, local: LocalLocation) -> Option<f32> {
        self.0
            .mesh_for(local.0)
            .and_then(|mesh| mesh.heightmap().height_at_position(local.1.x, local.1.y))
    }

    pub fn height_for<T: Into<LocalLocation>>(&self, location: T) -> Option<f32> {
        let local = location.into();
        self.height_for_location(local)
    }
}

#[derive(Resource, Deref, DerefMut, From)]
pub struct ServerResource(SilkroadServer);

#[derive(Resource, Deref, DerefMut, From)]
pub struct NpcPositionList(Vec<NpcPosition>);

impl NpcPositionList {
    pub fn positions_of(&self, ref_id: u32) -> impl Iterator<Item = &NpcPosition> {
        self.0.iter().filter(move |pos| pos.npc_id == ref_id)
    }
}

#[derive(Default, Resource)]
pub struct ActionIdCounter(AtomicU32);

impl ActionIdCounter {
    pub fn next(&self) -> u32 {
        self.0.fetch_add(1, Ordering::Relaxed)
    }
}
