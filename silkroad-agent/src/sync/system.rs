use crate::agent::component::MovementState;
use crate::agent::state::{AgentState, Dead, Idle, MovementTarget, Moving, PickingUp, StateTransitionEvent};
use crate::comp::damage::Invincible;
use crate::comp::exp::{Experienced, Leveled, SP};
use crate::comp::gold::GoldPouch;
use crate::comp::mastery::MasteryKnowledge;
use crate::comp::net::Client;
use crate::comp::player::{Player, StatPoints};
use crate::comp::pos::Position;
use crate::comp::visibility::{Invisible, Visibility};
use crate::comp::{GameEntity, Health, Mana};
use crate::event::LoadingFinishedEvent;
use crate::game::exp::LevelUpEvent;
use crate::sync::{SynchronizationCollector, Update};
use bevy_ecs::prelude::*;
use silkroad_game_base::{Heading, LocalPosition, MovementSpeed};
use silkroad_protocol::character::CharacterStatsMessage;
use silkroad_protocol::combat::ReceiveExperience;
use silkroad_protocol::movement::{
    EntityMovementInterrupt, MovementDestination, MovementSource, MovementType, PlayerMovementResponse,
};
use silkroad_protocol::skill::LevelUpMasteryResponse;
use silkroad_protocol::world::{
    AliveState, BodyState, CharacterPointsUpdate, EntityBarUpdateSource, EntityBarUpdates, EntityBarsUpdate,
    EntityUpdateState, LevelUpEffect, PlayerPickupAnimation, UpdatedState,
};
use std::ops::Deref;
use tracing::debug;

pub(crate) fn synchronize_updates(
    mut update_collector: ResMut<SynchronizationCollector>,
    source_query: Query<(Option<&Client>, &Visibility)>,
    others: Query<&Client>,
) {
    for update in update_collector.collect_updates() {
        if let Ok((client, visibility)) = source_query.get(update.source) {
            if let Some(client) = client {
                if let Some(self_packet) = update.change_self {
                    client.send(self_packet);
                }
            }

            if let Some(other_packet) = update.change_others {
                for client in visibility
                    .entities_in_radius
                    .iter()
                    .map(|reference| others.get(reference.0))
                    .filter_map(|res| res.ok())
                {
                    client.send(other_packet.clone());
                }
            }
        }
    }
}

pub(crate) fn system_collect_bars_update(
    collector: Res<SynchronizationCollector>,
    mut query: Query<(Entity, &GameEntity, &Health, &Mana), Or<(Changed<Health>, Changed<Mana>)>>,
) {
    for (entity, game_entity, health, mana) in query.iter_mut() {
        match (health.collect_change(), mana.collect_change()) {
            (Some(change_hp), Some(change_mp)) if change_hp > 0 && change_mp > 0 => {
                let update = EntityBarsUpdate {
                    unique_id: game_entity.unique_id,
                    source: EntityBarUpdateSource::Regen,
                    updates: EntityBarUpdates::Both {
                        hp: health.current_health,
                        mp: mana.current_mana,
                    },
                };

                collector.send_update(Update {
                    source: entity,
                    change_self: Some(update.clone().into()),
                    change_others: Some(update.into()),
                })
            },
            (None, None) => continue,
            (change_hp, change_mp) => {
                if let Some(change) = change_hp {
                    let update = EntityBarsUpdate {
                        unique_id: game_entity.unique_id,
                        source: if change < 0 {
                            EntityBarUpdateSource::Damage
                        } else {
                            EntityBarUpdateSource::Regen
                        },
                        updates: EntityBarUpdates::HP {
                            amount: health.current_health,
                        },
                    };

                    collector.send_update(Update::update_all(entity, update));
                }

                if let Some(change) = change_mp {
                    let update = EntityBarsUpdate {
                        unique_id: game_entity.unique_id,
                        source: if change < 0 {
                            EntityBarUpdateSource::Damage
                        } else {
                            EntityBarUpdateSource::Regen
                        },
                        updates: EntityBarUpdates::MP {
                            amount: mana.current_mana,
                        },
                    };

                    collector.send_update(Update::update_all(entity, update));
                }
            },
        }
    }
}

pub(crate) fn system_collect_sp_update(
    collector: Res<SynchronizationCollector>,
    query: Query<(Entity, Ref<SP>), Changed<SP>>,
) {
    for (entity, sp) in query.iter() {
        if sp.is_added() {
            continue;
        }

        let sp = sp.into_inner();
        collector.send_update(Update {
            source: entity,
            change_self: Some(CharacterPointsUpdate::sp(sp.current()).into()),
            change_others: None,
        })
    }
}

pub(crate) fn system_collect_exp_update(
    collector: Res<SynchronizationCollector>,
    mut query: Query<(Entity, &Leveled, &Experienced), Changed<Experienced>>,
) {
    for (entity, level, exp) in query.iter_mut() {
        for event in exp.experience_gains() {
            collector.send_update(Update::self_update(
                entity,
                ReceiveExperience {
                    exp_origin: event.from.map(|source| source.1.unique_id).unwrap_or(0),
                    experience: event.exp,
                    sp: event.sp_exp,
                    unknown: 0,
                    new_level: event.trigged_level_up.then(|| level.current_level() as u16),
                },
            ));
        }
    }
}

pub(crate) fn system_collect_level_up(
    collector: Res<SynchronizationCollector>,
    mut level_up_events: EventReader<LevelUpEvent>,
    mut query: Query<(Entity, &GameEntity, Option<&Player>)>,
) {
    for event in level_up_events.read() {
        let Ok((entity, game_entity, maybe_player)) = query.get_mut(event.target.0) else {
            continue;
        };

        let animation = LevelUpEffect {
            entity: game_entity.unique_id,
        };

        collector.send_update(Update::update_all(entity, animation));

        if let Some(player) = maybe_player {
            let update = CharacterStatsMessage {
                phys_attack_min: 100,
                phys_attack_max: 100,
                mag_attack_min: 100,
                mag_attack_max: 100,
                phys_defense: 100,
                mag_defense: 100,
                hit_rate: 100,
                parry_rate: 100,
                max_hp: player.character.stats.max_health(event.level),
                max_mp: player.character.stats.max_mana(event.level),
                strength: player.character.stats.strength(),
                intelligence: player.character.stats.intelligence(),
            };

            collector.send_update(Update::self_update(entity, update));
        }
    }
}

pub(crate) fn collect_movement_transitions(
    mut state_events: EventReader<StateTransitionEvent>,
    collector: Res<SynchronizationCollector>,
    query: Query<(&GameEntity, &Position)>,
) {
    for event in state_events.read() {
        let Ok((game_entity, pos)) = query.get(event.entity) else {
            continue;
        };

        if matches!(event.from, AgentState::Moving(_)) && !matches!(event.to, AgentState::Moving(_)) {
            let update = create_movement_packet(
                game_entity,
                MovementUpdate::StopMove(pos.position().to_local(), pos.rotation()),
            );
            debug!("Sending movement start. {}", game_entity.unique_id);
            collector.send_update(Update::update_all(event.entity, update));
        }
    }
}

pub(crate) fn collect_movement_starts(
    collector: Res<SynchronizationCollector>,
    mut query: Query<(Entity, &GameEntity, &Position, &Moving), Or<(Added<Moving>, Changed<Moving>)>>,
) {
    for (entity, game_entity, pos, moving) in query.iter_mut() {
        let update = match moving.parameter {
            MovementTarget::Location(dest) => MovementUpdate::StartMove(pos.position().to_local(), dest.to_local()),
            MovementTarget::Direction(direction) => {
                MovementUpdate::StartMoveTowards(pos.position().to_local(), direction)
            },
        };
        debug!("Sending movement start. {}", game_entity.unique_id);
        let update = create_movement_packet(game_entity, update);
        collector.send_update(Update::update_all(entity, update));
    }
}

pub(crate) fn collect_movement_update(
    collector: Res<SynchronizationCollector>,
    mut query: Query<(Entity, &GameEntity, &Position), (Changed<Position>, With<Idle>)>,
) {
    for (entity, game_entity, pos) in query.iter_mut() {
        let update = if pos.did_move() {
            // We probably teleported or something else messed up us.
            let packet = EntityMovementInterrupt {
                entity_id: game_entity.unique_id,
                position: pos.as_protocol(),
            };
            debug!("Sending movement interrupt. {}", game_entity.unique_id);
            collector.send_update(Update::update_all(entity, packet));
            continue;
        } else if pos.did_rotate() {
            MovementUpdate::Turn(pos.rotation())
        } else {
            continue;
        };

        let packet = create_movement_packet(game_entity, update);
        collector.send_update(Update::update_all(entity, packet));
    }
}

fn create_movement_packet(entity: &GameEntity, update: MovementUpdate) -> PlayerMovementResponse {
    match update {
        MovementUpdate::StartMove(current, target) => PlayerMovementResponse::new(
            entity.unique_id,
            MovementDestination::location(target.0.id(), target.1.x as u16, target.1.y as u16, target.1.z as u16),
            Some(MovementSource::new(
                current.0.id(),
                (current.1.x * 10.) as u16,
                current.1.y * 10.,
                (current.1.z * 10.) as u16,
            )),
        ),
        MovementUpdate::StartMoveTowards(current, direction) => PlayerMovementResponse::new(
            entity.unique_id,
            MovementDestination::direction(true, direction.into()),
            Some(MovementSource::new(
                current.0.id(),
                (current.1.x * 10.) as u16,
                current.1.y * 10.,
                (current.1.z * 10.) as u16,
            )),
        ),
        MovementUpdate::StopMove(current, _heading) => PlayerMovementResponse::new(
            entity.unique_id,
            MovementDestination::location(
                current.0.id(),
                current.1.x as u16,
                current.1.y as u16,
                current.1.z as u16,
            ),
            None,
        ),
        MovementUpdate::Turn(heading) => PlayerMovementResponse::new(
            entity.unique_id,
            MovementDestination::direction(false, heading.into()),
            None,
        ),
    }
}

enum MovementUpdate {
    /// This entity has started moving from the given location towards the given target location.
    StartMove(LocalPosition, LocalPosition),
    /// This entity has started moving from the given location towards the given direction.
    StartMoveTowards(LocalPosition, Heading),
    /// This entity has finished its movement and stopped at the given location with the given rotation.
    StopMove(LocalPosition, Heading),
    /// This entity has turned and is now facing the given direction.
    Turn(Heading),
}

pub(crate) fn collect_movement_speed_change(
    collector: Res<SynchronizationCollector>,
    query: Query<(Entity, &GameEntity, Ref<MovementState>), Changed<MovementState>>,
) {
    for (entity, game_entity, state) in query.iter() {
        if state.is_added() {
            continue;
        }

        let state = state.into_inner();

        let update = EntityUpdateState {
            unique_id: game_entity.unique_id,
            update: UpdatedState::Movement(match state.deref() {
                MovementSpeed::Running | MovementSpeed::Berserk => MovementType::Running,
                MovementSpeed::Walking => MovementType::Walking,
            }),
        };
        collector.send_update(Update::update_all(entity, update));
    }
}

pub(crate) fn collect_pickup_animation(
    collector: Res<SynchronizationCollector>,
    query: Query<(Entity, &GameEntity, &Position, &PickingUp), Or<(Added<PickingUp>, Changed<PickingUp>)>>,
) {
    for (entity, game_entity, pos, pickup) in query.iter() {
        if let Some(cooldown) = &pickup.cooldown {
            // Maybe we could do better instead of relying on this being zero?
            if cooldown.elapsed().is_zero() {
                let update = PlayerPickupAnimation {
                    entity: game_entity.unique_id,
                    rotation: pos.rotation().into(),
                };
                collector.send_update(Update::update_all(entity, update));
            }
        }
    }
}

pub(crate) fn collect_deaths(
    collector: Res<SynchronizationCollector>,
    query: Query<(Entity, &GameEntity), Added<Dead>>,
) {
    for (entity, game_entity) in query.iter() {
        let update = EntityUpdateState::life(game_entity.unique_id, AliveState::Dead);
        collector.send_update(Update {
            source: entity,
            change_self: Some(update.into()),
            change_others: Some(update.into()),
        });
    }
}

pub(crate) fn collect_alives(
    collector: Res<SynchronizationCollector>,
    mut reader: EventReader<LoadingFinishedEvent>,
    query: Query<&GameEntity>,
) {
    for event in reader.read() {
        let Ok(game_entity) = query.get(event.0) else {
            continue;
        };
        let update = EntityUpdateState::life(game_entity.unique_id, AliveState::Alive);
        collector.send_update(Update {
            source: event.0,
            change_self: Some(update.into()),
            change_others: Some(update.into()),
        });
    }
}

pub(crate) fn collect_body_states(
    collector: Res<SynchronizationCollector>,
    invisible_query: Query<(Entity, &GameEntity), Added<Invisible>>,
    invincible_query: Query<(Entity, &GameEntity), Added<Invincible>>,
) {
    for (entity, game_entity) in invisible_query.iter() {
        let update = EntityUpdateState::body(game_entity.unique_id, BodyState::GMInvisible);
        collector.send_update(Update {
            source: entity,
            change_self: Some(update.into()),
            change_others: None, // TODO: we may need a way to include other players here
        });
    }

    for (entity, game_entity) in invincible_query.iter() {
        let update = EntityUpdateState::body(game_entity.unique_id, BodyState::GMInvincible);
        collector.send_update(Update {
            source: entity,
            change_self: Some(update.into()),
            change_others: Some(update.into()),
        });
    }
}

pub(crate) fn collect_stat_changes(
    collector: Res<SynchronizationCollector>,
    query: Query<(Entity, &Leveled, &StatPoints), Changed<StatPoints>>,
) {
    for (entity, level, stats) in query.iter() {
        if stats.has_spent_points() {
            collector.send_update(Update {
                source: entity,
                change_self: Some(
                    CharacterStatsMessage {
                        phys_attack_min: 100,
                        phys_attack_max: 100,
                        mag_attack_min: 100,
                        mag_attack_max: 100,
                        phys_defense: 100,
                        mag_defense: 100,
                        hit_rate: 100,
                        parry_rate: 100,
                        max_hp: stats.stats().max_health(level.current_level()),
                        max_mp: stats.stats().max_mana(level.current_level()),
                        strength: stats.stats().strength(),
                        intelligence: stats.stats().intelligence(),
                    }
                    .into(),
                ),
                change_others: None,
            })
        }

        if stats.has_gained_points() {
            collector.send_update(Update {
                source: entity,
                change_self: Some(CharacterPointsUpdate::StatPoints(stats.remaining_points()).into()),
                change_others: None,
            });
        }
    }
}

pub(crate) fn collect_gold_changes(
    collector: Res<SynchronizationCollector>,
    query: Query<(Entity, &GoldPouch), Changed<GoldPouch>>,
) {
    for (entity, gold) in query.iter() {
        collector.send_update(Update::self_update(
            entity,
            CharacterPointsUpdate::Gold {
                amount: gold.amount(),
                display: true, // TODO: figure out when this should true and when false
            },
        ));
    }
}

pub(crate) fn collect_mastery_changes(
    collector: Res<SynchronizationCollector>,
    query: Query<(Entity, &MasteryKnowledge), Changed<MasteryKnowledge>>,
) {
    for (entity, mastery) in query.iter() {
        for mastery_ref in mastery.updated() {
            collector.send_update(Update::self_update(
                entity,
                LevelUpMasteryResponse::Success {
                    mastery: *mastery_ref,
                    new_level: mastery.level_of(*mastery_ref).unwrap_or(1),
                },
            ));
        }
    }
}
