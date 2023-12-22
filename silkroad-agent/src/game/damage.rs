use crate::agent::states::{Dead, StateTransitionQueue};
use crate::comp::damage::{DamageReceiver, Invincible};
use crate::comp::monster::Monster;
use crate::comp::net::Client;
use crate::comp::player::Player;
use crate::comp::{GameEntity, Health};
use crate::event::{DamageReceiveEvent, EntityDeath};
use crate::game::mind::Mind;
use bevy_ecs::prelude::*;
use silkroad_protocol::combat::{
    ActionType, DamageContent, DamageKind, DamageValue, PerEntityDamage, PerformActionError, PerformActionUpdate,
    SkillPartDamage,
};

pub(crate) fn handle_damage(
    mut reader: EventReader<DamageReceiveEvent>,
    mut receiver_query: Query<(
        &mut Health,
        &mut StateTransitionQueue,
        &mut DamageReceiver,
        Option<&Player>,
        Option<&Client>,
        Option<&Invincible>,
    )>,
    sender_query: Query<(&GameEntity, Option<&Client>)>,
    mut entity_died: EventWriter<EntityDeath>,
) {
    for damage_event in reader.read() {
        let Ok((mut health, mut controller, mut receiver, player, maybe_client, invincible)) =
            receiver_query.get_mut(damage_event.target.0)
        else {
            continue;
        };

        let (attacker, attacker_client) = sender_query
            .get(damage_event.source.0)
            .expect("Sender for damage event should exist");

        if health.is_dead() {
            // TODO: this might be wrong
            if let Some(client) = attacker_client {
                client.send(PerformActionUpdate::Error(PerformActionError::Completed))
            }
            continue;
        }

        let amount = if !invincible.is_some() { damage_event.amount } else { 0 };

        receiver.record_damage(attacker.unique_id, amount as u64);
        health.reduce(amount);
        let damage_data = if health.is_dead() {
            SkillPartDamage::KillingBlow(DamageValue::new(DamageKind::Standard, amount))
        } else {
            SkillPartDamage::Default(DamageValue::new(DamageKind::Standard, amount))
        };
        if let Some(client) = attacker_client {
            client.send(PerformActionUpdate::success(
                damage_event.attack.skill.ref_id,
                damage_event.source.1.unique_id,
                damage_event.target.1.unique_id,
                damage_event.attack.instance,
                ActionType::Attack {
                    damage: Some(DamageContent {
                        damage_instances: 1,
                        entities: vec![PerEntityDamage {
                            target: damage_event.target.1.unique_id,
                            damage: vec![damage_data],
                        }],
                    }),
                },
            ));
        } else if let Some(client) = maybe_client {
            client.send(PerformActionUpdate::success(
                damage_event.attack.skill.ref_id,
                damage_event.source.1.unique_id,
                damage_event.target.1.unique_id,
                damage_event.attack.instance,
                ActionType::Attack {
                    damage: Some(DamageContent {
                        damage_instances: 1,
                        entities: vec![PerEntityDamage {
                            target: damage_event.target.1.unique_id,
                            damage: vec![damage_data],
                        }],
                    }),
                },
            ));
        }

        if health.is_dead() {
            entity_died.send(EntityDeath {
                died: damage_event.target,
                killer: Some(damage_event.source),
            });
            let dead_state = if player.is_some() {
                Dead::new_player()
            } else {
                Dead::new_monster()
            };
            controller.request_transition(dead_state);
        }
    }
}

pub(crate) fn attack_player(mut query: Query<&mut Mind, With<Monster>>, mut events: EventReader<DamageReceiveEvent>) {
    for event in events.read() {
        if let Ok(mut mind) = query.get_mut(event.target.0) {
            if !mind.has_goal() {
                mind.attack(event.source)
            }
        }
    }
}
