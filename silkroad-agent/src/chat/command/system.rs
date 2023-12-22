use crate::agent::Agent;
use crate::comp::net::Client;
use crate::comp::player::Player;
use crate::comp::pos::Position;
use crate::comp::{EntityReference, GameEntity};
use crate::event::{PlayerCommandEvent, PlayerTeleportEvent};
use crate::ext::Navmesh;
use crate::game::exp::ReceiveExperienceEvent;
use crate::game::target::Target;
use crate::world::WorldData;
use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::*;
use silkroad_game_base::{GlobalLocation, GlobalPosition, LocalPosition, MovementSpeed};
use silkroad_protocol::chat::{ChatSource, ChatUpdate};
use silkroad_protocol::movement::ChangeSpeed;

fn format_location(pos: &LocalPosition) -> String {
    format!("X: {} | Y: {} | Z: {} | Region: {}", pos.1.x, pos.1.y, pos.1.z, pos.0)
}

fn command_response(message: String) -> ChatUpdate {
    ChatUpdate {
        source: ChatSource::Global {
            sender: "System".to_owned(),
        },
        message,
    }
}

pub(crate) fn handle_command(
    mut command_events: EventReader<PlayerCommandEvent>,
    mut query: Query<(
        Entity,
        &Client,
        &GameEntity,
        &Position,
        Option<&Target>,
        &mut Agent,
        &Player,
    )>,
    navmesh: Res<Navmesh>,
    target_query: Query<&Position>,
    mut teleport_events: EventWriter<PlayerTeleportEvent>,
    mut exp_events: EventWriter<ReceiveExperienceEvent>,
) {
    for event in command_events.read() {
        if let Ok((e, client, entity, pos, target, mut agent, player)) = query.get_mut(event.0) {
            if event.1.name == "pos" {
                let pos = pos.position().to_local();
                client.send(command_response(format_location(&pos)));
            } else if event.1.name == "gpos" {
                let pos = pos.position();
                client.send(command_response(format!("X: {} | Z: {}", pos.x, pos.z)));
            } else if event.1.name == "target" {
                let Some(target) = target else {
                    client.send(command_response("No target selected.".to_owned()));
                    continue;
                };

                let Ok(other_pos) = target_query.get(target.entity()) else {
                    client.send(command_response("Target does not exist.".to_owned()));
                    continue;
                };

                client.send(command_response(format_location(&other_pos.position().to_local())));
            } else if event.1.name == "movespeed" {
                let speed: f32 = event.1.args.first().and_then(|s| s.parse().ok()).unwrap_or(50.0);
                agent.set_speed(MovementSpeed::Running, speed);
                client.send(ChangeSpeed {
                    entity: entity.unique_id,
                    walk_speed: agent.get_speed_value(MovementSpeed::Walking),
                    running_speed: agent.get_speed_value(MovementSpeed::Running),
                });
            } else if event.1.name == "level" {
                let Some(target_level) = event.1.args.first().and_then(|s| s.parse::<u8>().ok()) else {
                    client.send(command_response("Invalid arguments".to_owned()));
                    continue;
                };

                let player_level = player.character.level;
                if target_level <= player_level {
                    client.send(command_response(
                        "Level needs to be higher than the current one.".to_owned(),
                    ));
                    return;
                }

                let total_required_exp: u64 = WorldData::levels()
                    .iter()
                    .filter(|(level, _)| *level >= player_level && *level < target_level)
                    .map(|(_, level)| level.exp)
                    .sum();

                if total_required_exp == 0 {
                    client.send(command_response("Level is not possible".to_owned()));
                    return;
                };

                let remaining = total_required_exp - player.character.exp;

                exp_events.send(ReceiveExperienceEvent {
                    source: None,
                    target: EntityReference(e, *entity),
                    exp: remaining,
                    sp: 0,
                });
            } else if event.1.name == "sp" {
                let Some(sp) = event.1.args.first().and_then(|s| s.parse::<u32>().ok()) else {
                    client.send(command_response("Invalid arguments".to_owned()));
                    continue;
                };

                exp_events.send(ReceiveExperienceEvent {
                    source: None,
                    target: EntityReference(e, *entity),
                    exp: 0,
                    sp: sp as u64 * 400,
                });
            } else if event.1.name == "tp" {
                let pos = if event.1.args.len() == 2 {
                    let x: f32 = event.1.args[0].parse().expect("Should be parseable as f32");
                    let z: f32 = event.1.args[1].parse().expect("Should be parseable as f32");
                    let location = GlobalLocation::from_ingame_location(x, z);
                    let height = navmesh.height_for(location).unwrap_or(0.0);
                    location.with_y(height)
                } else if event.1.args.len() == 3 {
                    let x: f32 = event.1.args[0].parse().expect("Should be parseable as f32");
                    let y: f32 = event.1.args[1].parse().expect("Should be parseable as f32");
                    let z: f32 = event.1.args[2].parse().expect("Should be parseable as f32");
                    GlobalPosition::from_ingame_position(x, y, z)
                } else {
                    client.send(command_response("Invalid arguments".to_owned()));
                    continue;
                };

                teleport_events.send(PlayerTeleportEvent(event.0, pos));
            }
        }
    }
}

pub(crate) fn handle_teleport(mut teleport_events: EventReader<PlayerTeleportEvent>, mut query: Query<&mut Position>) {
    for event in teleport_events.read() {
        if let Ok(mut pos) = query.get_mut(event.0) {
            pos.move_to(event.1);
        }
    }
}
