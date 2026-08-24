use crate::comp::monster::Monster;
use crate::comp::pos::Position;
use crate::comp::GameEntity;
use crate::event::EntityDeath;
use crate::game::drop::SpawnDrop;
use bevy::prelude::*;
use silkroad_definitions::rarity::EntityRarity;
use silkroad_game_base::Item;
use silkroad_loot::{LootTables, SystemRandom};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Agent-owned ECS resource adapting the framework-independent loot tables to
/// Bevy systems.
#[derive(Resource)]
pub(crate) struct LootResource(LootTables);

impl LootResource {
    pub(crate) fn new(tables: LootTables) -> Self {
        Self(tables)
    }

    pub(crate) fn generate(&self, monster_ref_id: u32, rarity: EntityRarity) -> Vec<Item> {
        self.0.generate(monster_ref_id, rarity, &mut SystemRandom)
    }

    pub(crate) fn has_plan(&self, monster_ref_id: u32) -> bool {
        self.0.has_plan(monster_ref_id)
    }

    /// Shared gold drop factory for death drops and player-initiated drops.
    pub(crate) fn gold_item(&self, amount: u32) -> Item {
        self.0.gold_item(amount)
    }
}

/// Generates loot for every dead monster and emits one `SpawnDrop` per item
/// through the existing drop pipeline.
pub(crate) fn generate_loot(
    mut deaths: MessageReader<EntityDeath>,
    monsters: Query<(&GameEntity, &Monster, &Position)>,
    tables: Res<LootResource>,
    mut drops: MessageWriter<SpawnDrop>,
) {
    for event in deaths.read() {
        let Ok((game_entity, monster, position)) = monsters.get(event.died.0) else {
            continue;
        };

        let items = tables.generate(game_entity.ref_id, monster.rarity);
        if items.is_empty() && !tables.has_plan(game_entity.ref_id) {
            warn_unknown_monster(game_entity.ref_id);
            continue;
        }

        for item in items {
            drops.write(SpawnDrop {
                item,
                relative_position: position.location(),
                owner: event.killer,
            });
        }
    }
}

/// Warns about monsters without a loot plan at most once every 30 seconds.
fn warn_unknown_monster(ref_id: u32) {
    static LAST_WARNED_MILLIS: AtomicU64 = AtomicU64::new(0);
    const WARN_INTERVAL_MILLIS: u64 = 30_000;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0);
    let last = LAST_WARNED_MILLIS.load(Ordering::Relaxed);
    if now.saturating_sub(last) > WARN_INTERVAL_MILLIS
        && LAST_WARNED_MILLIS.compare_exchange(last, now, Ordering::Relaxed, Ordering::Relaxed) == Ok(last)
    {
        tracing::warn!(ref_id, "Dead monster has no loot plan; no loot was generated");
    }
}

#[cfg(test)]
fn fake_gold_item(ref_id: u32) -> silkroad_data::itemdata::RefItemData {
    use silkroad_data::common::{RefCommon, RefOrigin};
    use silkroad_data::itemdata::{RefBiologicalType, RefItemData, RefItemRarity};
    use silkroad_definitions::TypeId;
    use std::time::Duration;

    RefItemData {
        common: RefCommon {
            service: true,
            ref_id,
            id: format!("ITEM_ETC_GOLD_{ref_id:03}"),
            type_id: TypeId(3, 3, 1, 1),
            country: RefOrigin::Chinese,
            despawn_time: Duration::from_secs(30),
        },
        price: 0,
        rarity: RefItemRarity::General,
        can_drop: true,
        max_stack_size: 1,
        range: None,
        required_level: None,
        biological_type: RefBiologicalType::Both,
        params: [0; 4],
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::comp::EntityReference;
    use silkroad_data::characterdata::RefCharacterData;
    use silkroad_data::common::{RefCommon, RefOrigin};
    use silkroad_data::datamap::{DataEntry, DataMap};
    use silkroad_data::gold::{GoldMap, RefGold};
    use silkroad_definitions::rarity::{EntityRarity, EntityRarityType};
    use silkroad_definitions::TypeId;
    use silkroad_game_base::{GlobalPosition, Heading, ItemTypeData};
    use std::collections::HashMap;
    use std::time::Duration;

    /// Minimal fixture tables compiled through the crate's public interface.
    fn fixture_tables() -> LootTables {
        let items = Box::leak(Box::new(DataMap::new(vec![
            fake_gold_item(1),
            fake_gold_item(2),
            fake_gold_item(3),
        ])));
        let monsters = Box::leak(Box::new(DataMap::new(vec![RefCharacterData {
            common: RefCommon {
                service: true,
                ref_id: 100,
                id: String::from("MOB_TEST"),
                type_id: TypeId(1, 2, 1, 1),
                country: RefOrigin::General,
                despawn_time: Duration::from_secs(10),
            },
            rarity: EntityRarity::new(false, EntityRarityType::Normal),
            level: 10,
            exp: 0,
            hp: 100,
            walk_speed: 0,
            run_speed: 0,
            berserk_speed: 0,
            base_range: 0,
            pickup_range: None,
            aggressive: false,
            skills: Vec::new(),
        }])));
        let gold = Box::leak(Box::new(GoldMap::new(HashMap::from([(
            10,
            RefGold {
                level: 10,
                min: 50,
                max: 50,
            },
        )]))));
        let definitions = tempfile::tempdir().unwrap();
        std::fs::write(
            definitions.path().join("test.ron"),
            r#"(
                version: 1,
                default_profile: Some("default"),
                pools: [(name: "gold", entries: [(weight: 1, generator: Gold)])],
                profiles: [(
                    name: "default",
                    rolls: [(pool: "gold", attempts: 1, chance: 1.0)],
                )],
            )"#,
        )
        .unwrap();

        LootTables::load_and_compile(definitions.path(), 1.0, items, monsters, gold).unwrap()
    }

    #[derive(Default)]
    struct RecordedDrop {
        ref_id: u32,
        amount: Option<u32>,
        location: (f32, f32),
        has_owner: bool,
    }

    #[derive(Resource, Default)]
    struct RecordedDrops(Vec<RecordedDrop>);

    fn record_drops(mut reader: MessageReader<SpawnDrop>, mut recorded: ResMut<RecordedDrops>) {
        for drop in reader.read() {
            recorded.0.push(RecordedDrop {
                ref_id: drop.item.reference.ref_id(),
                amount: match drop.item.type_data {
                    ItemTypeData::Gold { amount } => Some(amount),
                    _ => None,
                },
                location: (drop.relative_position.0.x, drop.relative_position.0.y),
                has_owner: drop.owner.is_some(),
            });
        }
    }

    #[test]
    fn guaranteed_gold_pool_creates_exactly_one_death_drop() {
        let mut app = App::new();
        app.add_message::<EntityDeath>()
            .add_message::<SpawnDrop>()
            .insert_resource(LootResource::new(fixture_tables()))
            .insert_resource(RecordedDrops::default())
            .add_systems(Update, (generate_loot, record_drops).chain());

        let monster = app
            .world_mut()
            .spawn((
                GameEntity {
                    unique_id: 7,
                    ref_id: 100,
                },
                Monster {
                    target: None,
                    rarity: EntityRarity::new(false, EntityRarityType::Normal),
                },
                Position::new(GlobalPosition(cgmath::Vector3::new(1.0, 2.0, 3.0)), Heading(0.0)),
            ))
            .id();

        app.world_mut().write_message(EntityDeath {
            died: EntityReference(
                monster,
                GameEntity {
                    unique_id: 7,
                    ref_id: 100,
                },
            ),
            killer: None,
        });

        app.update();

        let drops = &app.world().resource::<RecordedDrops>().0;
        assert_eq!(drops.len(), 1, "exactly one drop should be created per death");
        assert_eq!(drops[0].ref_id, 1);
        assert_eq!(drops[0].amount, Some(50));
        assert_eq!((drops[0].location.0, drops[0].location.1), (1.0, 3.0));
        assert!(!drops[0].has_owner);

        // A second update must not duplicate the drop.
        app.update();
        let drops = &app.world().resource::<RecordedDrops>().0;
        assert_eq!(drops.len(), 1);
    }

    #[test]
    fn resource_gold_factory_uses_compiled_tiers() {
        let loot = LootResource::new(fixture_tables());
        assert_eq!(loot.gold_item(500).reference.ref_id(), 1);
        assert_eq!(loot.gold_item(9000).reference.ref_id(), 3);
    }
}
