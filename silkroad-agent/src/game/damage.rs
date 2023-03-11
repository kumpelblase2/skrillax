use crate::agent::states::{Dead, StateTransitionQueue};
use crate::comp::monster::Monster;
use crate::comp::net::Client;
use crate::comp::player::Player;
use crate::comp::sync::Synchronize;
use crate::comp::Health;
use crate::event::DamageReceiveEvent;
use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::*;
use silkroad_protocol::combat::{
    ActionType, DamageContent, DamageKind, DamageValue, PerEntityDamage, PerformActionUpdate, SkillPartDamage,
};
use silkroad_protocol::world::{AliveState, UpdatedState};

pub(crate) fn handle_damage(
    mut reader: EventReader<DamageReceiveEvent>,
    mut receiver_query: Query<(
        &mut Health,
        &mut Synchronize,
        &mut StateTransitionQueue,
        Option<&Player>,
        Option<&Monster>,
    )>,
    mut sender_query: Query<Option<&Client>>,
    mut cmd: Commands,
) {
    for damage_event in reader.iter() {
        let Ok((mut health, mut synchronize, mut controller, player, monster)) = receiver_query
                .get_mut(damage_event.target.0) else {
            continue;
        };

        let attacker_client = sender_query
            .get(damage_event.source.0)
            .expect("Sender for damage event should exist");

        health.reduce(damage_event.amount);
        synchronize.health = Some(health.current_health);
        let damage_data = if health.is_dead() {
            SkillPartDamage::KillingBlow(DamageValue::new(DamageKind::Standard, damage_event.amount))
        } else {
            SkillPartDamage::Default(DamageValue::new(DamageKind::Standard, damage_event.amount))
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
        }

        if health.is_dead() {
            // TODO: this should be done by the state transition
            synchronize.state.push(UpdatedState::Life(AliveState::Dead));
            let dead_state = if player.is_some() {
                Dead::new_player()
            } else {
                Dead::new_monster()
            };
            controller.request_transition(dead_state);
        }
    }
}
