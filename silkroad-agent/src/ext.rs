use bevy_ecs_macros::Resource;
use derive_more::{Deref, DerefMut, From};
use id_pool::IdPool;
use pk2::Pk2;
use silkroad_data::npc_pos::NpcPosition;
use silkroad_game_base::LocalLocation;
use silkroad_navmesh::NavmeshLoader;
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

#[derive(Resource, Deref, DerefMut, From)]
pub struct Navmesh(NavmeshLoader<Pk2>);

impl Navmesh {
    pub fn height_for<T: Into<LocalLocation>>(&mut self, location: T) -> Option<f32> {
        let local = location.into();
        self.0
            .load_navmesh(local.0)
            .ok()
            .and_then(|mesh| mesh.heightmap().height_at_position(local.1.x, local.1.y))
    }
}

#[derive(Resource, Deref, DerefMut, From)]
pub struct ServerResource(SilkroadServer);

#[derive(Resource, Deref, DerefMut, From)]
pub struct NpcPositionList(Vec<NpcPosition>);

#[derive(Default, Resource)]
pub struct ActionIdCounter(AtomicU32);

impl ActionIdCounter {
    pub fn next(&self) -> u32 {
        self.0.fetch_add(1, Ordering::Relaxed)
    }
}
