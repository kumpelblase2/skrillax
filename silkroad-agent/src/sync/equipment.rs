use crate::comp::inventory::PlayerInventory;
use crate::comp::GameEntity;
use crate::sync::{SynchronizationCollector, Update};
use bevy::prelude::*;
use silkroad_definitions::inventory::EquipmentSlot;
use silkroad_protocol::world::{CharacterEquipmentRemove, CharacterEquipmentUpdate};

const EQUIPMENT_SLOT_COUNT: usize = EquipmentSlot::Special as usize + 1;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct SynchronizedEquipmentItem {
    item_id: u32,
    upgrade_level: u8,
}

/// The equipment state most recently projected to nearby clients.
#[derive(Component, Clone, Debug, Eq, PartialEq)]
pub(crate) struct LastSynchronizedEquipment {
    items: [Option<SynchronizedEquipmentItem>; EQUIPMENT_SLOT_COUNT],
}

impl LastSynchronizedEquipment {
    pub(crate) fn from_inventory(inventory: &PlayerInventory) -> Self {
        let mut items = [None; EQUIPMENT_SLOT_COUNT];
        for (slot, item) in inventory.equipment_items() {
            items[slot as usize] = Some(SynchronizedEquipmentItem {
                item_id: item.reference.common.ref_id,
                upgrade_level: item.upgrade_level(),
            });
        }
        Self { items }
    }

    fn changes_to(&self, current: &Self) -> Vec<EquipmentChange> {
        let mut changes = Vec::new();

        for (raw_slot, (previous, current)) in self.items.iter().zip(&current.items).enumerate() {
            if previous == current {
                continue;
            }

            let slot = EquipmentSlot::try_from(raw_slot as u8)
                .expect("every synchronized equipment index should have a slot definition");
            if let Some(previous) = previous {
                changes.push(EquipmentChange::Remove {
                    slot,
                    item_id: previous.item_id,
                });
            }
            if let Some(current) = current {
                changes.push(EquipmentChange::Update {
                    slot,
                    item_id: current.item_id,
                    upgrade_level: current.upgrade_level,
                });
            }
        }

        changes
    }
}

#[derive(Debug, Eq, PartialEq)]
enum EquipmentChange {
    Remove {
        slot: EquipmentSlot,
        item_id: u32,
    },
    Update {
        slot: EquipmentSlot,
        item_id: u32,
        upgrade_level: u8,
    },
}

pub(crate) fn collect_equipment_changes(
    collector: Res<SynchronizationCollector>,
    mut query: Query<(Entity, &GameEntity, &PlayerInventory, &mut LastSynchronizedEquipment), Changed<PlayerInventory>>,
) {
    for (entity, game_entity, inventory, mut previous) in query.iter_mut() {
        let current = LastSynchronizedEquipment::from_inventory(inventory);

        for change in previous.changes_to(&current) {
            match change {
                EquipmentChange::Remove { slot, item_id } => {
                    collector.send_update(Update::update_all(
                        entity,
                        CharacterEquipmentRemove {
                            entity: game_entity.unique_id,
                            slot: slot as u8,
                            item_id,
                        },
                    ));
                },
                EquipmentChange::Update {
                    slot,
                    item_id,
                    upgrade_level,
                } => {
                    collector.send_update(Update::update_all(
                        entity,
                        CharacterEquipmentUpdate {
                            entity: game_entity.unique_id,
                            slot: slot as u8,
                            item_id,
                            upgrade_level,
                        },
                    ));
                },
            }
        }

        *previous = current;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn equipment(items: &[(EquipmentSlot, u32, u8)]) -> LastSynchronizedEquipment {
        let mut equipment = LastSynchronizedEquipment {
            items: [None; EQUIPMENT_SLOT_COUNT],
        };
        for &(slot, item_id, upgrade_level) in items {
            equipment.items[slot as usize] = Some(SynchronizedEquipmentItem { item_id, upgrade_level });
        }
        equipment
    }

    #[test]
    fn newly_equipped_item_produces_an_update() {
        let previous = equipment(&[]);
        let current = equipment(&[(EquipmentSlot::Weapon, 42, 3)]);

        assert_eq!(
            previous.changes_to(&current),
            vec![EquipmentChange::Update {
                slot: EquipmentSlot::Weapon,
                item_id: 42,
                upgrade_level: 3,
            }]
        );
    }

    #[test]
    fn unequipped_item_produces_a_remove() {
        let previous = equipment(&[(EquipmentSlot::Weapon, 42, 3)]);
        let current = equipment(&[]);

        assert_eq!(
            previous.changes_to(&current),
            vec![EquipmentChange::Remove {
                slot: EquipmentSlot::Weapon,
                item_id: 42,
            }]
        );
    }

    #[test]
    fn replacement_removes_the_previous_item_before_updating() {
        let previous = equipment(&[(EquipmentSlot::Weapon, 42, 3)]);
        let current = equipment(&[(EquipmentSlot::Weapon, 84, 5)]);

        assert_eq!(
            previous.changes_to(&current),
            vec![
                EquipmentChange::Remove {
                    slot: EquipmentSlot::Weapon,
                    item_id: 42,
                },
                EquipmentChange::Update {
                    slot: EquipmentSlot::Weapon,
                    item_id: 84,
                    upgrade_level: 5,
                },
            ]
        );
    }

    #[test]
    fn unchanged_final_state_produces_no_updates() {
        let previous = equipment(&[(EquipmentSlot::Weapon, 42, 3)]);
        let current = equipment(&[(EquipmentSlot::Weapon, 42, 3)]);

        assert!(previous.changes_to(&current).is_empty());
    }
}
