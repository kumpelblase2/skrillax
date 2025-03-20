use crate::comp::monster::Monster;
use crate::comp::net::Client;
use crate::comp::GameEntity;
use crate::config::GameConfig;
use crate::event::{SpawnMonster, UniqueKilledEvent};
use crate::ext::NpcPositionList;
use bevy::prelude::*;
use rand::prelude::IteratorRandom;
use rand::{rng, Rng};
use silkroad_definitions::rarity::EntityRarityType;
use silkroad_game_base::NpcPosExt;
use silkroad_protocol::world::GameNotification;
use std::ops::RangeInclusive;
use std::time::Duration;
use tracing::{debug, warn};

pub(crate) fn unique_spawned(query: Query<(&GameEntity, &Monster), Added<Monster>>, notify: Query<&Client>) {
    for (entity, _) in query
        .iter()
        .filter(|(_, monster)| monster.rarity == EntityRarityType::Unique)
    {
        notify.iter().for_each(|client| {
            client.send(GameNotification::uniquespawned(entity.ref_id));
        });
    }
}

pub(crate) fn unique_killed(mut events: EventReader<UniqueKilledEvent>, notify: Query<&Client>) {
    for kill in events.read() {
        notify.iter().for_each(|client| {
            client.send(GameNotification::uniquekilled(kill.unique.ref_id, kill.player.clone()));
        });
    }
}

pub(crate) fn update_timers(
    time: Res<Time>,
    mut timers: ResMut<UniqueTimers>,
    npc_pos: Res<NpcPositionList>,
    mut writer: EventWriter<SpawnMonster>,
) {
    let delta = time.delta();
    let spawns = timers.update(delta);
    let mut rng = rng();
    for ref_id in spawns {
        let Some(position) = npc_pos.positions_of(ref_id).choose(&mut rng) else {
            warn!("Could not find a position to spawn unique with ref id {}", ref_id);
            continue;
        };

        let position = position.location().to_global().to_location();
        writer.send(SpawnMonster {
            ref_id,
            location: position,
            spawner: None,
            with_ai: true,
        });
    }
}

pub(crate) struct UniqueTimer {
    timer: Timer,
    range: RangeInclusive<usize>,
    unique_ref: u32,
}

#[derive(Resource)]
pub(crate) struct UniqueTimers {
    timers: Vec<UniqueTimer>,
}

impl UniqueTimers {
    pub(crate) fn update(&mut self, delta: Duration) -> Vec<u32> {
        let mut rng = rng();
        let mut to_spawn = Vec::new();
        for timer in self.timers.iter_mut() {
            if timer.timer.tick(delta).just_finished() {
                timer.timer = Timer::from_seconds(
                    rng.random_range(RangeInclusive::clone(&timer.range)) as f32 * 60.0,
                    TimerMode::Once,
                );
                to_spawn.push(timer.unique_ref);
            }
        }

        to_spawn
    }
}

pub(crate) fn setup_unique_timers(mut cmd: Commands, config: Res<GameConfig>) {
    let unique = &config.spawner.unique;
    let timers = vec![
        create_timer_for(unique.tiger_woman.spawn_range(), 1954),
        create_timer_for(unique.uruchi.spawn_range(), 1982),
        create_timer_for(unique.isyutaru.spawn_range(), 2002),
        create_timer_for(unique.bonelord.spawn_range(), 3810),
    ];
    debug!(
        "Settings spawn for tiger woman in {}min",
        timers[0].timer.remaining().as_secs() / 60
    );
    debug!(
        "Settings spawn for uruchi in {}min",
        timers[1].timer.remaining().as_secs() / 60
    );
    debug!(
        "Settings spawn for isyutaru in {}min",
        timers[2].timer.remaining().as_secs() / 60
    );
    debug!(
        "Settings spawn for bonelord in {}min",
        timers[3].timer.remaining().as_secs() / 60
    );
    cmd.insert_resource(UniqueTimers { timers });
}

fn create_timer_for(range: RangeInclusive<usize>, ref_id: u32) -> UniqueTimer {
    let time = rng().random_range(RangeInclusive::clone(&range));
    let timer = Timer::from_seconds(time as f32 * 60.0, TimerMode::Once);

    UniqueTimer {
        timer,
        range,
        unique_ref: ref_id,
    }
}
