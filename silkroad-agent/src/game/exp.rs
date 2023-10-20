use crate::comp::damage::DamageReceiver;
use crate::comp::exp::{Experienced, Leveled, SP};
use crate::comp::player::Player;
use crate::comp::pos::Position;
use crate::comp::{EntityReference, GameEntity, Health, Mana};
use crate::event::EntityDeath;
use crate::world::{EntityLookup, WorldData};
use bevy_ecs::prelude::*;
use tracing::warn;

const EXP_RECEIVE_RANGE_SQUARED: f32 = 1000.0 * 1000.0;

#[derive(Event)]
pub struct ReceiveExperienceEvent {
    pub source: Option<EntityReference>,
    pub target: EntityReference,
    pub exp: u64,
    pub sp: u64,
}

pub(crate) fn distribute_experience(
    mut death_events: EventReader<EntityDeath>,
    mut experience_writer: EventWriter<ReceiveExperienceEvent>,
    dead_query: Query<(&DamageReceiver, &Position)>,
    lookup: Res<EntityLookup>,
    receiver_query: Query<(&GameEntity, &Position)>,
) {
    for event in death_events.iter() {
        let Ok((damage_distribution, death_location)) = dead_query.get(event.died.0) else {
            continue;
        };

        for attacker_id in damage_distribution.all_attackers() {
            if let Some(((game_entity, position), target_entity)) = lookup
                .get_entity_for_id(attacker_id)
                .and_then(|entity| receiver_query.get(entity).ok().zip(Some(entity)))
            {
                if death_location.distance_to(position) <= EXP_RECEIVE_RANGE_SQUARED {
                    let event = ReceiveExperienceEvent {
                        source: Some(event.died),
                        target: EntityReference(target_entity, *game_entity),
                        exp: 100,
                        sp: 100,
                    };
                    experience_writer.send(event);
                }
            }
        }
    }
}

pub(crate) fn receive_experience(
    mut experience_events: EventReader<ReceiveExperienceEvent>,
    mut query: Query<(&mut Leveled, &mut Experienced, &mut SP)>,
) {
    let level_map = WorldData::levels();

    for event in experience_events.iter() {
        let Ok((mut level, mut experienced, mut sp)) = query.get_mut(event.target.0) else {
            continue;
        };

        if event.exp == 0 && event.sp == 0 {
            warn!("Somehow received 0 Exp AND 0 SP.");
            continue;
        }

        experienced.receive(event.exp, event.sp, event.source);
        let received_sp = experienced.convert_sp();
        if received_sp > 0 {
            sp.gain(received_sp);
        }

        while let Some(exp) = level_map.get_exp_for_level(level.current_level()) {
            if experienced.try_level_up(exp) {
                level.level_up();
            } else {
                break;
            }
        }
    }
}

pub(crate) fn reset_health_mana_on_level(
    mut query: Query<(&Player, &Leveled, &mut Health, &mut Mana), Changed<Leveled>>,
) {
    for (player, leveled, mut health, mut mana) in query.iter_mut() {
        if leveled.did_level() {
            health.upgrade(player.character.max_hp());
            mana.upgrade(player.character.max_mp());
        }
    }
}
