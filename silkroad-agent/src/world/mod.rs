use crate::config::GameConfig;
use crate::ext::{EntityIdPool, Navmesh, NpcPositionList};
use crate::world::lookup::{collect_entities, maintain_entities};
use bevy::prelude::*;
pub use data::*;
pub use lookup::*;
use pk2_sync::sync::readonly::Pk2;
use silkroad_data::npc_pos::NpcPosition;
use silkroad_navmesh::builder::NavmeshBuilder;
use std::path::Path;

mod data;
mod lookup;
mod spawning;

const BLOWFISH_KEY: &str = "169841";

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        let data_location = &app
            .world()
            .get_resource::<GameConfig>()
            .expect("Game settings should exist")
            .data_location;
        let location = Path::new(data_location);
        let data_file = location.join("Data.pk2");
        let data_pk2 = Pk2::open_readonly(data_file, BLOWFISH_KEY).unwrap();
        let media_file = location.join("Media.pk2");
        let media_pk2 = Pk2::open_readonly(media_file, BLOWFISH_KEY).unwrap();
        WorldData::load_data_from(&media_pk2).expect("Should be able to load silkroad data");
        let npcs = NpcPosition::from(&media_pk2).unwrap();
        let navmesh = NavmeshBuilder::build_from(&data_pk2).expect("should be able to load navmesh from data.");
        app.insert_resource(EntityIdPool::default())
            .insert_resource(EntityLookup::default())
            .insert_resource::<NpcPositionList>(npcs.into())
            .add_systems(Startup, spawning::spawn_npcs)
            .add_systems(First, maintain_entities)
            .add_systems(Last, collect_entities)
            .add_systems(Update, spawning::spawn_monsters)
            .add_systems(Last, spawning::collect_monster_deaths)
            .insert_resource::<Navmesh>(navmesh.into());
    }
}
