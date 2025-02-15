use crate::comp::monster::Monster;
use crate::comp::pos::Position;
use crate::comp::GameEntity;
use crate::config::get_config;
use crate::event::EntityDeath;
use crate::game::drop::SpawnDrop;
use crate::world::WorldData;
use bevy::prelude::*;
use rand::{thread_rng, Rng};
use silkroad_data::itemdata::RefItemData;
use silkroad_game_base::{Item, ItemTypeData};

const SMALL_GOLD_SIZE_MAX: u32 = 1000;
const MEDIUM_GOLD_SIZE_MAX: u32 = 5000;

pub(crate) fn get_gold_ref_id(amount: u32) -> &'static RefItemData {
    let items = WorldData::items();
    let item_ref = if amount < SMALL_GOLD_SIZE_MAX {
        items.find_id(1)
    } else if amount < MEDIUM_GOLD_SIZE_MAX {
        items.find_id(2)
    } else {
        items.find_id(3)
    };
    item_ref.expect("Gold item ref should exist")
}

pub(crate) fn drop_gold(
    mut death_events: EventReader<EntityDeath>,
    query: Query<(&GameEntity, &Position), With<Monster>>,
    mut drop_events: EventWriter<SpawnDrop>,
) {
    let characters = WorldData::characters();
    let gold = WorldData::gold();
    let config = get_config();
    for event in death_events.read() {
        if let Ok((game_entity, pos)) = query.get(event.died.0) {
            let Some(monster_data) = characters.find_id(game_entity.ref_id) else {
                continue;
            };

            let monster_level = monster_data.level;
            let gold_range = gold.get_for_level(monster_level);
            let amount = thread_rng().gen_range(gold_range);
            let amount = (config.game.drop.gold * amount as f32).floor() as u32;
            drop_events.send(SpawnDrop {
                item: Item {
                    reference: get_gold_ref_id(amount),
                    variance: None,
                    type_data: ItemTypeData::Gold { amount },
                },
                relative_position: pos.location(),
                owner: event.killer,
            });
        }
    }
}
