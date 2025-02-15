use crate::agent::component::Agent;
use crate::comp::net::Client;
use crate::comp::player::{Player, StatPoints};
use crate::comp::pos::Position;
use crate::comp::{EntityReference, GameEntity};
use crate::ext::Navmesh;
use crate::game::exp::ReceiveExperienceEvent;
use crate::game::target::Target;
use crate::world::WorldData;
use bevy::app::MainScheduleOrder;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use gumdrop::Options;
use silkroad_game_base::{GlobalLocation, GlobalPosition, MovementSpeed};
use silkroad_protocol::chat::{ChatSource, ChatUpdate};
use silkroad_protocol::movement::ChangeSpeed;
use std::fmt::Display;
use tracing::{info, warn};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Sender {
    Player(Entity),
    System,
}

#[derive(Event)]
struct CommandInvocation<T> {
    sender: Sender,
    args: T,
}

enum CommandOutcome {
    Success(Option<String>),
    InvalidCommand(String),
    InvalidArguments(String),
    ExecutionFailure(String),
}

impl CommandOutcome {
    fn is_positive(&self) -> bool {
        matches!(self, CommandOutcome::Success(_))
    }
}

impl Display for CommandOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            CommandOutcome::Success(message) => message.clone().unwrap_or(String::new()),
            CommandOutcome::InvalidCommand(cmd) => format!("The command {} does not exist.", cmd),
            CommandOutcome::InvalidArguments(failure) => format!("The specified arguments were invalid. {}", failure),
            CommandOutcome::ExecutionFailure(failure) => format!("Command execution failed: {}", failure),
        };
        write!(f, "{}", str)
    }
}

#[derive(Event)]
struct CommandResult {
    receiver: Sender,
    outcome: CommandOutcome,
}

#[derive(Event)]
struct IncomingCommand {
    sender: Sender,
    command: String,
}

#[derive(ScheduleLabel, Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct CommandSchedule;

pub struct CommandPlugin;

impl Plugin for CommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CommandInvocation<AddStatPoints>>()
            .add_event::<IncomingCommand>()
            .add_event::<CommandResult>()
            .add_event::<CommandInvocation<AddStatPoints>>()
            .add_event::<CommandInvocation<AddSkillPoints>>()
            .add_event::<CommandInvocation<ChangeLevel>>()
            .add_event::<CommandInvocation<AlterMovespeed>>()
            .add_event::<CommandInvocation<PrintPos>>()
            .add_event::<CommandInvocation<PrintTarget>>()
            .add_event::<CommandInvocation<TeleportArgs>>()
            .add_systems(
                CommandSchedule,
                (
                    parse_commands,
                    (
                        handle_stat_points,
                        handle_sp_points,
                        handle_level,
                        handle_movespeed,
                        handle_print_pos,
                        handle_print_target,
                        handle_teleport,
                    ),
                    output_results,
                )
                    .chain(),
            );
        let mut schedule_order = app.world_mut().resource_mut::<MainScheduleOrder>();
        schedule_order.insert_before(Update, CommandSchedule);
    }
}

pub trait CommandExecutionExt {
    fn enqueue_chat_command(&mut self, sender: Sender, command: String);
}

impl CommandExecutionExt for Commands<'_, '_> {
    fn enqueue_chat_command(&mut self, sender: Sender, command: String) {
        self.send_event(IncomingCommand { sender, command });
    }
}

#[derive(Debug, Options)]
#[options(help = "Silkroad ingame commands")]
struct SilkroadCommand {
    #[options(command)]
    inner: Option<SilkroadCommands>,
}

#[derive(Debug, Options, PartialEq)]
enum SilkroadCommands {
    #[options(help = "Add more stat points")]
    StatPoints(AddStatPoints),
    #[options(help = "Add more skill points")]
    SkillPoints(AddSkillPoints),
    #[options(help = "Increase the character level")]
    Level(ChangeLevel),
    #[options(help = "Change the movement speed")]
    Movespeed(AlterMovespeed),
    #[options(help = "Prints the current position")]
    Pos(PrintPos),
    #[options(help = "Prints the current position of the target")]
    Target(PrintTarget),
    #[options(help = "Teleports to the given position")]
    Tp(TeleportArgs),
    #[options(help = "Show the help output")]
    Help(Help),
}

#[derive(Debug, Options, PartialEq)]
struct Help {}

fn parse_commands(mut incoming: EventReader<IncomingCommand>, mut cmds: Commands) {
    for incoming in incoming.read() {
        let command_args: Vec<&str> = incoming.command.split(' ').collect();
        let cmd = match SilkroadCommand::parse_args_default(&command_args) {
            Ok(cmd) => cmd,
            Err(err) => {
                cmds.send_event(CommandResult {
                    receiver: incoming.sender,
                    outcome: CommandOutcome::InvalidArguments(err.to_string()),
                });
                continue;
            },
        };

        let help_requested = cmd.help_requested();
        let Some(inner) = cmd.inner else {
            cmds.send_event(CommandResult {
                receiver: incoming.sender,
                outcome: CommandOutcome::Success(Some(SilkroadCommand::usage().to_string())),
            });
            continue;
        };

        if help_requested || matches!(inner, SilkroadCommands::Help(_)) {
            cmds.send_event(CommandResult {
                receiver: incoming.sender,
                outcome: CommandOutcome::Success(Some(SilkroadCommand::usage().to_string())),
            });
            continue;
        }

        match inner {
            SilkroadCommands::StatPoints(args) => {
                cmds.send_event(CommandInvocation {
                    sender: incoming.sender,
                    args,
                });
            },
            SilkroadCommands::SkillPoints(args) => {
                cmds.send_event(CommandInvocation {
                    sender: incoming.sender,
                    args,
                });
            },
            SilkroadCommands::Level(args) => {
                cmds.send_event(CommandInvocation {
                    sender: incoming.sender,
                    args,
                });
            },
            SilkroadCommands::Movespeed(args) => {
                cmds.send_event(CommandInvocation {
                    sender: incoming.sender,
                    args,
                });
            },
            SilkroadCommands::Pos(args) => {
                cmds.send_event(CommandInvocation {
                    sender: incoming.sender,
                    args,
                });
            },
            SilkroadCommands::Target(args) => {
                cmds.send_event(CommandInvocation {
                    sender: incoming.sender,
                    args,
                });
            },
            SilkroadCommands::Tp(args) => {
                cmds.send_event(CommandInvocation {
                    sender: incoming.sender,
                    args,
                });
            },
            SilkroadCommands::Help(_) => {
                unreachable!("Help should have already been handled above.")
            },
        }
    }
}

fn output_results(mut results: EventReader<CommandResult>, sender_query: Query<&Client>) {
    for result in results.read() {
        if let Sender::Player(player) = result.receiver {
            let Ok(client) = sender_query.get(player) else {
                continue;
            };

            client.send(ChatUpdate {
                source: ChatSource::system(),
                message: result.outcome.to_string(),
            });
        } else if result.outcome.is_positive() {
            info!("{}", result.outcome);
        } else {
            warn!("{}", result.outcome);
        }
    }
}

#[derive(Options, Debug, PartialEq, Copy, Clone)]
struct AddStatPoints {
    #[options(short = "s")]
    str: bool,
    #[options(short = "i")]
    int: bool,
    #[options(free)]
    amount: u16,
}

fn handle_stat_points(
    mut invocations: EventReader<CommandInvocation<AddStatPoints>>,
    mut results: EventWriter<CommandResult>,
    mut player_query: Query<&mut StatPoints>,
) {
    for add_stats in invocations.read() {
        let Sender::Player(player) = add_stats.sender else {
            results.send(CommandResult {
                receiver: add_stats.sender,
                outcome: CommandOutcome::ExecutionFailure("This command can only be used by a player.".to_string()),
            });
            continue;
        };

        if let Ok(mut stats) = player_query.get_mut(player) {
            let args = add_stats.args;
            if !args.str && !args.int {
                stats.gain_points(args.amount);
            } else {
                if args.str {
                    stats.gain_points(args.amount);
                    stats.spend_str_points(args.amount);
                }
                if args.int {
                    stats.gain_points(args.amount);
                    stats.spend_int_points(args.amount);
                }
            }
        } else {
            results.send(CommandResult {
                receiver: add_stats.sender,
                outcome: CommandOutcome::ExecutionFailure(
                    "The given entity does not have stat points or does not exist.".to_string(),
                ),
            });
        }
    }
}

#[derive(Options, Debug, PartialEq)]
struct AddSkillPoints {
    #[options(free)]
    amount: u64,
}

fn handle_sp_points(
    mut invocations: EventReader<CommandInvocation<AddSkillPoints>>,
    mut results: EventWriter<CommandResult>,
    query: Query<&GameEntity>,
    mut experience_events: EventWriter<ReceiveExperienceEvent>,
) {
    for add_stats in invocations.read() {
        let Sender::Player(player) = add_stats.sender else {
            results.send(CommandResult {
                receiver: add_stats.sender,
                outcome: CommandOutcome::ExecutionFailure("This command can only be used by a player.".to_string()),
            });
            continue;
        };

        experience_events.send(ReceiveExperienceEvent {
            source: None,
            target: EntityReference(player, *query.get(player).unwrap()),
            exp: 0,
            sp: add_stats.args.amount * 400,
        });
    }
}

#[derive(Options, Debug, PartialEq)]
struct ChangeLevel {
    #[options(free)]
    target_level: u8,
}

fn handle_level(
    mut invocations: EventReader<CommandInvocation<ChangeLevel>>,
    mut results: EventWriter<CommandResult>,
    mut query: Query<(&mut Player, &GameEntity)>,
    mut experience_events: EventWriter<ReceiveExperienceEvent>,
) {
    for change_level in invocations.read() {
        let Sender::Player(player_entity) = change_level.sender else {
            results.send(CommandResult {
                receiver: change_level.sender,
                outcome: CommandOutcome::ExecutionFailure("This command can only be used by a player.".to_string()),
            });
            continue;
        };

        let target_level = change_level.args.target_level;
        let (player, entity_ref) = query.get_mut(player_entity).unwrap();
        let player_level = player.character.level;
        if target_level <= player_level {
            results.send(CommandResult {
                receiver: change_level.sender,
                outcome: CommandOutcome::ExecutionFailure("Level needs to be higher than the current one.".to_string()),
            });
            continue;
        }

        let total_required_exp: u64 = WorldData::levels()
            .iter()
            .filter(|(level, _)| *level >= player_level && *level < target_level)
            .map(|(_, level)| level.exp)
            .sum();

        if total_required_exp == 0 {
            results.send(CommandResult {
                receiver: change_level.sender,
                outcome: CommandOutcome::ExecutionFailure("Level is impossible.".to_string()),
            });
            continue;
        };

        let remaining = total_required_exp - player.character.exp;

        experience_events.send(ReceiveExperienceEvent {
            source: None,
            target: EntityReference(player_entity, *entity_ref),
            exp: remaining,
            sp: 0,
        });
    }
}

#[derive(Options, Debug, PartialEq)]
struct AlterMovespeed {
    #[options(free)]
    speed: f32,
}

fn handle_movespeed(
    mut invocations: EventReader<CommandInvocation<AlterMovespeed>>,
    mut results: EventWriter<CommandResult>,
    mut query: Query<(&mut Agent, &Client, &GameEntity)>,
) {
    for change_speed in invocations.read() {
        let Sender::Player(player_entity) = change_speed.sender else {
            results.send(CommandResult {
                receiver: change_speed.sender,
                outcome: CommandOutcome::ExecutionFailure("This command can only be used by a player.".to_string()),
            });
            continue;
        };

        let (mut agent, client, entity_ref) = query.get_mut(player_entity).unwrap();
        let speed = change_speed.args.speed;
        agent.set_speed(MovementSpeed::Running, speed);
        client.send(ChangeSpeed {
            entity: entity_ref.unique_id,
            walk_speed: agent.get_speed_value(MovementSpeed::Walking),
            running_speed: agent.get_speed_value(MovementSpeed::Running),
        });
    }
}

#[derive(Options, Debug, PartialEq)]
struct PrintPos {
    #[options(short = "g")]
    global: bool,
}

fn handle_print_pos(
    mut invocations: EventReader<CommandInvocation<PrintPos>>,
    mut results: EventWriter<CommandResult>,
    query: Query<&Position>,
) {
    for change_speed in invocations.read() {
        let Sender::Player(player_entity) = change_speed.sender else {
            results.send(CommandResult {
                receiver: change_speed.sender,
                outcome: CommandOutcome::ExecutionFailure("This command can only be used by a player.".to_string()),
            });
            continue;
        };

        let pos = query.get(player_entity).unwrap();
        results.send(CommandResult {
            receiver: change_speed.sender,
            outcome: CommandOutcome::Success(Some(format_position(pos, change_speed.args.global))),
        });
    }
}

#[derive(Options, Debug, PartialEq)]
struct PrintTarget {
    #[options(short = "g")]
    global: bool,
}

fn handle_print_target(
    mut invocations: EventReader<CommandInvocation<PrintTarget>>,
    mut results: EventWriter<CommandResult>,
    query: Query<Option<&Target>>,
    query_pos: Query<&Position>,
) {
    for change_speed in invocations.read() {
        let Sender::Player(player_entity) = change_speed.sender else {
            results.send(CommandResult {
                receiver: change_speed.sender,
                outcome: CommandOutcome::ExecutionFailure("This command can only be used by a player.".to_string()),
            });
            continue;
        };

        let Some(target) = query.get(player_entity).unwrap() else {
            results.send(CommandResult {
                receiver: change_speed.sender,
                outcome: CommandOutcome::Success(Some("No target selected.".to_string())),
            });
            continue;
        };

        let target_pos = query_pos.get(target.entity()).unwrap();

        results.send(CommandResult {
            receiver: change_speed.sender,
            outcome: CommandOutcome::Success(Some(format_position(target_pos, change_speed.args.global))),
        });
    }
}

fn format_position(pos: &Position, global: bool) -> String {
    if global {
        let pos = pos.position().to_local();
        format!("X: {} | Y: {} | Z: {} | Region: {}", pos.1.x, pos.1.y, pos.1.z, pos.0)
    } else {
        let pos = pos.position();
        format!("X: {} | Z: {}", pos.x, pos.z)
    }
}

#[derive(Options, Debug, PartialEq)]
struct TeleportArgs {
    #[options(free)]
    x: f32,
    #[options(free)]
    y: f32,
    #[options(free)]
    z: Option<f32>,
}

fn handle_teleport(
    mut invocations: EventReader<CommandInvocation<TeleportArgs>>,
    mut results: EventWriter<CommandResult>,
    navmesh: Res<Navmesh>,
    mut position: Query<&mut Position>,
) {
    for change_speed in invocations.read() {
        let Sender::Player(player_entity) = change_speed.sender else {
            results.send(CommandResult {
                receiver: change_speed.sender,
                outcome: CommandOutcome::ExecutionFailure("This command can only be used by a player.".to_string()),
            });
            continue;
        };

        let target = if change_speed.args.z.is_some() {
            GlobalPosition::from_ingame_position(change_speed.args.x, change_speed.args.y, change_speed.args.z.unwrap())
        } else {
            let location = GlobalLocation::from_ingame_location(change_speed.args.x, change_speed.args.y);
            let height = navmesh.height_for(location).unwrap_or(0.0);
            location.with_y(height)
        };

        let mut position = position.get_mut(player_entity).unwrap();
        position.move_to(target);
    }
}
