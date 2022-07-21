use crate::comp::npc::NPC;
use crate::comp::pos::{Heading, LocalPosition, Position};
use crate::comp::GameEntity;
pub use crate::world::lookup::maintain_entities;
pub use crate::world::lookup::EntityLookup;
use crate::GameSettings;
use bevy_app::{App, CoreStage, Plugin};
use bevy_ecs::prelude::*;
use bevy_ecs::system::ResMut;
use cgmath::Vector3;
use id_pool::IdPool;
use pk2::Pk2;
use silkroad_data::characterdata::CharacterMap;
use silkroad_data::gold::GoldMap;
use silkroad_data::itemdata::ItemMap;
use silkroad_data::level::LevelMap;
use silkroad_data::npc_pos::NpcPosition;
use silkroad_data::type_id::{ObjectEntity, ObjectNonPlayer, ObjectType};
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
        let characters = CharacterMap::from(&media_pk2).unwrap();
        let items = ItemMap::from(&media_pk2).unwrap();
        let npcs = NpcPosition::from(&media_pk2).unwrap();
        app.insert_resource(IdPool::new())
            .insert_resource(Ticks::default())
            .insert_resource(EntityLookup::new())
            .insert_resource(levels)
            .insert_resource(gold)
            .insert_resource(characters)
            .insert_resource(items)
            .insert_resource(npcs)
            .add_startup_system(spawn_npcs)
            .add_system_to_stage(CoreStage::First, update_ticks)
            .add_system_to_stage(CoreStage::First, maintain_entities)
            .insert_resource(NavmeshLoader::new(data_pk2));
    }
}

fn update_ticks(mut ticks: ResMut<Ticks>) {
    ticks.increase()
}

fn spawn_npcs(
    npc_spawns: Res<Vec<NpcPosition>>,
    character_data: Res<CharacterMap>,
    mut commands: Commands,
    mut id_pool: ResMut<IdPool>,
) {
    for spawn in npc_spawns.iter() {
        let character_data = character_data
            .find_id(spawn.npc_id)
            .expect("Could not find character data for NPC to spawn.");
        let type_id =
            ObjectType::from_type_id(&character_data.type_id).expect("Could not create type id from type 4-tuple.");
        if matches!(
            type_id,
            ObjectType::Entity(ObjectEntity::NonPlayer(ObjectNonPlayer::NPC(_)))
        ) {
            let game_entity = GameEntity {
                unique_id: id_pool.request_id().expect("Should have ID available for NPC"),
                ref_id: spawn.npc_id,
            };
            let npc = NPC {};
            let position = Position {
                location: LocalPosition(spawn.region.into(), Vector3::new(spawn.x, spawn.y, spawn.z)).to_global(),
                rotation: Heading(0.0),
            };
            commands.spawn().insert(game_entity).insert(npc).insert(position);
        }
    }
}

#[derive(Default)]
pub struct Ticks(pub u64);

impl Ticks {
    pub fn increase(&mut self) {
        self.0 += 1;
    }
}
