use crate::world::lookup::collect_entities;
use crate::world::lookup::maintain_entities;
use bevy_app::{App, CoreStage, Plugin};
use id_pool::IdPool;
use pk2::Pk2;
use silkroad_data::npc_pos::NpcPosition;
use silkroad_navmesh::NavmeshLoader;
use std::path::Path;

mod data;
mod lookup;
mod spawning;

use crate::config::GameConfig;
pub use data::*;
pub use lookup::*;

const BLOWFISH_KEY: &str = "169841";

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        let data_location = &app
            .world
            .get_resource::<GameConfig>()
            .expect("Game settings should exist")
            .data_location;
        let location = Path::new(data_location);
        let data_file = location.join("Data.pk2");
        let data_pk2 = Pk2::open(data_file, BLOWFISH_KEY).unwrap();
        let media_file = location.join("Media.pk2");
        let media_pk2 = Pk2::open(media_file, BLOWFISH_KEY).unwrap();
        WorldData::load_data_from(&media_pk2).expect("Should be able to load silkroad data");
        let npcs = NpcPosition::from(&media_pk2).unwrap();
        app.insert_resource(IdPool::new())
            .insert_resource(EntityLookup::default())
            .insert_resource(npcs)
            .add_startup_system(spawning::spawn_npcs)
            .add_system_to_stage(CoreStage::First, maintain_entities)
            .add_system_to_stage(CoreStage::Last, collect_entities)
            .add_system(spawning::spawn_monsters)
            .insert_resource(NavmeshLoader::new(data_pk2));
    }
}
