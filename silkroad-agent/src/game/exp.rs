use crate::comp::damage::DamageReceiver;
use crate::comp::net::Client;
use crate::comp::player::Player;
use crate::comp::pos::Position;
use crate::comp::sync::Synchronize;
use crate::comp::{EntityReference, GameEntity, Health};
use crate::event::EntityDeath;
use crate::world::{EntityLookup, WorldData};
use bevy_ecs::prelude::*;
use silkroad_protocol::character::CharacterStatsMessage;
use silkroad_protocol::world::{
    CharacterPointsUpdate, EntityBarUpdateSource, EntityBarUpdates, EntityBarsUpdate, ReceiveExperience,
};
use tracing::debug;

const SP_EXP_PER_SP: u32 = 400;
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
    mut query: Query<(&GameEntity, &Client, &mut Player, &mut Health, &mut Synchronize)>,
) {
    let level_map = WorldData::levels();

    for event in experience_events.iter() {
        let Ok((entity, client, mut player, mut health, mut sync)) = query.get_mut(event.target.0) else {
            continue;
        };

        player.character.exp += event.exp;
        player.character.sp_exp += event.sp as u32;
        let received_sp = player.character.sp_exp / SP_EXP_PER_SP;
        player.character.sp += received_sp;
        player.character.sp_exp %= SP_EXP_PER_SP;
        let mut levels_increased = 0u16;

        while let Some(exp) = level_map.get_exp_for_level(player.character.level) {
            if exp < player.character.exp {
                levels_increased += 1;
                player.character.increase_level();
                player.character.exp -= exp;
            } else {
                break;
            }
        }

        client.send(ReceiveExperience {
            exp_origin: event.source.map(|source| source.1.unique_id).unwrap_or(0),
            experience: event.exp,
            sp: event.sp,
            unknown: 0,
            new_level: (levels_increased > 0).then_some(player.character.level as u16),
        });

        if levels_increased > 0 {
            debug!(
                player = player.character.name,
                "Levelled up! New level: {}", player.character.level
            );

            health.max_health = player.character.max_hp();
            health.current_health = health.max_health;

            sync.did_level = true;

            client.send(CharacterStatsMessage {
                phys_attack_min: 100,
                phys_attack_max: 100,
                mag_attack_min: 100,
                mag_attack_max: 100,
                phys_defense: 100,
                mag_defense: 100,
                hit_rate: 100,
                parry_rate: 100,
                max_hp: player.character.stats.max_health(player.character.level),
                max_mp: player.character.stats.max_mana(player.character.level),
                strength: player.character.stats.strength(),
                intelligence: player.character.stats.intelligence(),
            });

            client.send(EntityBarsUpdate {
                unique_id: entity.unique_id,
                source: EntityBarUpdateSource::LevelUp,
                updates: EntityBarUpdates::Both {
                    hp: health.current_health,
                    mp: player.character.max_mp(),
                },
            })
        }

        if received_sp > 0 {
            client.send(CharacterPointsUpdate::sp(player.character.sp));
        }
    }
}
