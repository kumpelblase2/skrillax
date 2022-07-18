pub use crate::world::lookup::maintain_entities;
pub use crate::world::lookup::EntityLookup;
use crate::GameSettings;
use bevy_app::{App, CoreStage, Plugin};
use bevy_ecs::system::ResMut;
use id_pool::IdPool;
use pk2::Pk2;
use silkroad_data::gold::GoldMap;
use silkroad_data::level::LevelMap;
use silkroad_navmesh::NavmeshLoader;
use std::path::Path;

mod lookup;

const BLOWFISH_KEY: &str = "169841";

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        let data_location = &app.world.get_resource::<GameSettings>().unwrap().data_location;
        let location = Path::new(data_location);
        let data_file = location.join("Data.pk2");
        let data_pk2 = Pk2::open(data_file, BLOWFISH_KEY).unwrap();
        let media_file = location.join("Media.pk2");
        let media_pk2 = Pk2::open(media_file, BLOWFISH_KEY).unwrap();
        let levels = LevelMap::from(&media_pk2).unwrap();
        let gold = GoldMap::from(&media_pk2).unwrap();
        app.insert_resource(IdPool::new())
            .insert_resource(Ticks::default())
            .insert_resource(EntityLookup::new())
            .insert_resource(levels)
            .insert_resource(gold)
            .add_system_to_stage(CoreStage::First, update_ticks)
            .add_system_to_stage(CoreStage::First, maintain_entities)
            .insert_resource(NavmeshLoader::new(data_pk2));
    }
}

fn update_ticks(mut ticks: ResMut<Ticks>) {
    ticks.increase()
}

#[derive(Default)]
pub struct Ticks(pub u64);

impl Ticks {
    pub fn increase(&mut self) {
        self.0 += 1;
    }
}
