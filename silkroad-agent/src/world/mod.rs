use crate::config::GameConfig;
use crate::ext::{EntityIdPool, Navmesh, NpcPositionList};
use crate::game::loot::LootResource;
use crate::world::lookup::{collect_entities, maintain_entities};
use bevy::prelude::*;
pub use data::*;
pub use lookup::*;
use pk2_sync::sync::readonly::Pk2;
use silkroad_data::npc_pos::NpcPosition;
use silkroad_definitions::type_id::{ObjectItem, ObjectType};
use silkroad_loot::LootTables;
use silkroad_navmesh::builder::NavmeshBuilder;
use silkroad_protocol::runtime::initialize_equipment_ids;
use silkroad_protocol::spawn::register_ref_id;
use std::collections::{HashMap, HashSet};
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

        let loot_config = app
            .world()
            .get_resource::<GameConfig>()
            .expect("Game settings should exist")
            .loot
            .clone();
        let loot_tables = LootTables::load_and_compile(
            Path::new(&loot_config.directory),
            loot_config.rate,
            WorldData::items(),
            WorldData::characters(),
            WorldData::gold(),
        )
        .unwrap_or_else(|error| panic!("Invalid loot configuration: {error}"));

        let npcs = NpcPosition::from(&media_pk2).unwrap();
        let navmesh = NavmeshBuilder::build_from(&data_pk2).expect("should be able to load navmesh from data.");

        // Technically, this is not necessary to be set, because we'd never read items from the client.
        // But to be safe, we'll set it anyway.
        let equipment_items = WorldData::items()
            .iter()
            .filter(|item| {
                matches!(
                    ObjectType::from_type_id(&item.common.type_id),
                    Some(ObjectType::Item(ObjectItem::Equippable(_)))
                )
            })
            .map(|item| item.common.ref_id)
            .collect::<HashSet<_>>();
        initialize_equipment_ids(equipment_items);

        let mut object_map = HashMap::<u32, ObjectType>::new();
        WorldData::characters().iter().for_each(|character| {
            if let Some(object_type) = ObjectType::from_type_id(&character.common.type_id) {
                object_map.insert(character.common.ref_id, object_type);
            }
        });

        register_ref_id(object_map);

        app.insert_resource(LootResource::new(loot_tables))
            .insert_resource(EntityIdPool::default())
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
