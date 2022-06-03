use crate::world::id_allocator::IdAllocator;
use crate::GameSettings;
use bevy_app::{App, CoreStage, Plugin};
use bevy_ecs::system::ResMut;
use pk2::Pk2;
use silkroad_navmesh::NavmeshLoader;
use std::path::Path;

pub mod id_allocator;

const BLOWFISH_KEY: &str = "169841";

pub(crate) struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        let data_location = &app.world.get_resource::<GameSettings>().unwrap().data_location;
        let location = Path::new(data_location);
        let data_file = location.join("Data.pk2");

        app.insert_resource(IdAllocator::new())
            .insert_resource(Ticks::default())
            .add_system_to_stage(CoreStage::First, update_ticks)
            .insert_resource(NavmeshLoader::new(Pk2::open(data_file, BLOWFISH_KEY).unwrap()));
    }
}

fn update_ticks(mut ticks: ResMut<Ticks>) {
    ticks.increase()
}

#[derive(Default)]
pub(crate) struct Ticks(pub u64);

impl Ticks {
    pub(crate) fn increase(&mut self) {
        self.0 += 1;
    }
}
