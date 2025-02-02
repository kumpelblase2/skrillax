use crate::agent::component::{Agent, MovementState};
use crate::agent::goal::GoalTracker;
use crate::agent::state::AgentStateQueue;
use crate::comp::damage::DamageReceiver;
use crate::comp::monster::{Monster, MonsterAiBundle, MonsterBundle, RandomStroll, SpawnedBy};
use crate::comp::pos::Position;
use crate::comp::visibility::Visibility;
use crate::comp::{GameEntity, Health};
use crate::event::SpawnMonster;
use crate::ext::{EntityIdPool, Navmesh};
use crate::world::WorldData;
use bevy_ecs::change_detection::{Res, ResMut};
use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::Commands;
use rand::{thread_rng, Rng};
use silkroad_definitions::rarity::EntityRarityType;
use silkroad_game_base::Heading;
use tracing::debug;

pub(crate) fn do_spawn_mobs(
    mut reader: EventReader<SpawnMonster>,
    mut cmd: Commands,
    mesh: Res<Navmesh>,
    mut id_pool: ResMut<EntityIdPool>,
) {
    let mut rng = thread_rng();
    let characters = WorldData::characters();
    for event in reader.read() {
        let character_def = characters
            .find_id(event.ref_id)
            .expect("Should have character definition for monster spawn.");
        let unique_id = id_pool.request_id().unwrap();
        let height = mesh.height_for(event.location).unwrap_or(0.0);
        let position = event.location.with_y(height);

        if character_def.rarity == EntityRarityType::Unique {
            debug!("Spawning {} at {}", character_def.common.id, position);
        }

        let mut spawning = cmd.spawn(MonsterBundle {
            monster: Monster {
                target: None,
                rarity: character_def.rarity,
            },
            health: Health::new(character_def.hp),
            position: Position::new(position, Heading(rng.gen())),
            entity: GameEntity {
                unique_id,
                ref_id: event.ref_id,
            },
            visibility: Visibility::with_radius(500.),
            spawner: event.spawner.unwrap_or(SpawnedBy::None),
            navigation: Agent::from_character_data(character_def),
            state_queue: AgentStateQueue::default(),
            movement_state: MovementState::default_monster(),
            damage_receiver: DamageReceiver::default(),
        });

        spawning.insert(MonsterAiBundle {
            stroll: RandomStroll::new(position.to_location(), 300., 10..60),
            goal: GoalTracker::default(),
        });
    }
}
