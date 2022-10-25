use crate::world::WorldData;
use silkroad_data::itemdata::RefItemData;

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
