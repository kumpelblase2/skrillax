use crate::world::lookup::collect_entities;
pub use crate::world::lookup::maintain_entities;
pub use crate::world::lookup::EntityLookup;
use crate::GameSettings;
use bevy_app::{App, CoreStage, Plugin};
use bevy_ecs::system::ResMut;
use id_pool::IdPool;
use once_cell::sync::OnceCell;
use pk2::Pk2;
use silkroad_data::characterdata::{load_character_map, RefCharacterData};
use silkroad_data::gold::{load_gold_map, GoldMap};
use silkroad_data::itemdata::{load_item_map, RefItemData};
use silkroad_data::level::{load_level_map, LevelMap};
use silkroad_data::npc_pos::NpcPosition;
use silkroad_data::skilldata::{load_skill_map, RefSkillData};
use silkroad_data::DataMap;
use silkroad_navmesh::NavmeshLoader;
use std::path::Path;

mod lookup;
mod spawning;

pub static ITEMS: OnceCell<DataMap<RefItemData>> = OnceCell::new();
pub static CHARACTERS: OnceCell<DataMap<RefCharacterData>> = OnceCell::new();
pub static SKILLS: OnceCell<DataMap<RefSkillData>> = OnceCell::new();
pub static LEVELS: OnceCell<LevelMap> = OnceCell::new();
pub static GOLD: OnceCell<GoldMap> = OnceCell::new();

const BLOWFISH_KEY: &str = "169841";

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        let data_location = &app
            .world
            .get_resource::<GameSettings>()
            .expect("Game settings should exist")
            .data_location;
        let location = Path::new(data_location);
        let data_file = location.join("Data.pk2");
        let data_pk2 = Pk2::open(data_file, BLOWFISH_KEY).unwrap();
        let media_file = location.join("Media.pk2");
        let media_pk2 = Pk2::open(media_file, BLOWFISH_KEY).unwrap();
        let levels = load_level_map(&media_pk2).unwrap();
        let gold = load_gold_map(&media_pk2).unwrap();
        let characters = load_character_map(&media_pk2).unwrap();
        let items = load_item_map(&media_pk2).unwrap();
        let npcs = NpcPosition::from(&media_pk2).unwrap();
        let skills = load_skill_map(&media_pk2).unwrap();
        let _ = LEVELS.set(levels);
        let _ = GOLD.set(gold);
        let _ = CHARACTERS.set(characters);
        let _ = ITEMS.set(items);
        let _ = SKILLS.set(skills);
        app.insert_resource(IdPool::new())
            .insert_resource(Ticks::default())
            .insert_resource(EntityLookup::new())
            .insert_resource(npcs)
            .add_startup_system(spawning::spawn_npcs)
            .add_system_to_stage(CoreStage::First, update_ticks)
            .add_system_to_stage(CoreStage::First, maintain_entities)
            .add_system_to_stage(CoreStage::Last, collect_entities)
            .add_system(spawning::spawn_monsters)
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
