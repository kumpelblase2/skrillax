use std::collections::HashSet;
use std::sync::OnceLock;

static EQUIPMENT_IDS: OnceLock<HashSet<u32>> = OnceLock::new();

pub fn initialize_equipment_ids<T: Into<HashSet<u32>>>(equipment_ids: T) {
    let _ = EQUIPMENT_IDS.set(equipment_ids.into());
}

pub fn is_equipment_id(id: u32) -> bool {
    EQUIPMENT_IDS.get().map(|set| set.contains(&id)).unwrap_or(false)
}
