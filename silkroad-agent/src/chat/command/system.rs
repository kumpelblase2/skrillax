use crate::agent::Agent;
use crate::comp::net::Client;
use crate::comp::pos::Position;
use crate::comp::GameEntity;
use crate::event::{PlayerCommandEvent, PlayerTeleportEvent};
use crate::ext::Navmesh;
use crate::game::target::Target;
use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::*;
use silkroad_game_base::{GlobalLocation, GlobalPosition, LocalPosition, MovementSpeed};
use silkroad_protocol::chat::{ChatSource, ChatUpdate};
use silkroad_protocol::world::{ChangeSpeed, EntityMovementInterrupt};
use std::ops::Deref;

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
    mut query: Query<(&Client, &GameEntity, &Position, Option<&Target>, &mut Agent)>,
    mut navmesh: ResMut<Navmesh>,
    target_query: Query<&Position>,
    mut teleport_events: EventWriter<PlayerTeleportEvent>,
) {
    for event in command_events.iter() {
        if let Ok((client, entity, pos, target, mut agent)) = query.get_mut(event.0) {
            if event.1.name == "pos" {
                let pos = pos.location.to_local();
                client.send(command_response(format_location(&pos)));
            } else if event.1.name == "gpos" {
                let pos = pos.location;
                client.send(command_response(format!("X: {} | Z: {}", pos.x, pos.z)));
            } else if event.1.name == "target" {
                let Some(target) = target else {
                    client.send(command_response( "No target selected.".to_owned()));
                    continue;
                };

                let Ok(other_pos) = target_query.get(target.entity()) else {
                    client.send(command_response(  "Target does not exist.".to_owned()));
                    continue;
                };

                client.send(command_response(format_location(&other_pos.location.to_local())));
            } else if event.1.name == "movespeed" {
                let speed: f32 = event.1.args.first().map(|s| s.parse().unwrap_or(50.0)).unwrap_or(50.0);
                agent.set_speed(MovementSpeed::Running, speed);
                client.send(ChangeSpeed {
                    entity: entity.unique_id,
                    walk_speed: agent.get_speed_value(MovementSpeed::Walking),
                    running_speed: agent.get_speed_value(MovementSpeed::Running),
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

pub(crate) fn handle_teleport(
    mut teleport_events: EventReader<PlayerTeleportEvent>,
    mut query: Query<(&Client, &GameEntity, &mut Position)>,
) {
    for event in teleport_events.iter() {
        if let Ok((client, game_entity, mut pos)) = query.get_mut(event.0) {
            pos.location = event.1;
            client.send(EntityMovementInterrupt {
                entity_id: game_entity.unique_id,
                position: pos.as_protocol(),
            });
        }
    }
}
