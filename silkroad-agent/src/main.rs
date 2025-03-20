#![allow(clippy::type_complexity)]

mod agent;
mod chat;
mod cmd;
mod comp;
mod config;
mod db;
mod event;
mod ext;
mod game;
mod input;
mod login;
mod mall;
mod net;
mod persistence;
mod population;
mod protocol;
mod server_plugin;
mod sync;
mod tasks;
mod world;

use crate::agent::AgentPlugin;
use crate::cmd::CommandPlugin;
use crate::config::get_config;
use crate::db::server::ServerRegistration;
use crate::ext::DbPool;
use crate::game::GamePlugin;
use crate::input::ReceivePlugin;
use crate::login::LoginPlugin;
use crate::mall::MallPlugin;
use crate::net::NetworkPlugin;
use crate::persistence::PersistencePlugin;
use crate::population::{CapacityController, LoginQueue};
use crate::server_plugin::ServerPlugin;
use crate::sync::SynchronizationPlugin;
use crate::tasks::TaskCreator;
use crate::world::WorldPlugin;
use bevy::prelude::*;
use bevy::time::TimePlugin;
use login::web::WebServer;
use rand::distr::Alphanumeric;
use rand::{rng, Rng};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tracing::info;

fn main() {
    tracing_subscriber::fmt::init();

    let configuration = get_config();
    let server_id = configuration.server_id;
    let external_addr = match &configuration.external_address {
        Some(addr) => SocketAddr::from_str(addr),
        None => format!("127.0.0.1:{}", configuration.listen_port).parse(),
    }
    .expect("External address should be 'ip:port'.");

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(4)
        .thread_name("async-worker")
        .build()
        .expect("Should be able to create tokio runtime");
    let runtime = Arc::new(runtime);

    let capacity_manager = CapacityController::new(configuration.max_player_count);
    let queue = LoginQueue::new(capacity_manager.clone(), 30);

    let db_pool = runtime
        .block_on(configuration.database.create_pool())
        .expect("Should be able to create db pool");

    let token: String = rng().sample_iter(&Alphanumeric).take(30).map(char::from).collect();

    runtime
        .block_on(ServerRegistration::setup(
            server_id,
            configuration.name.clone(),
            configuration.region.clone(),
            external_addr,
            configuration.rpc_address.clone(),
            configuration.rpc_port,
            token.clone(),
            db_pool.clone(),
        ))
        .expect("Should be able to register server");

    let _web_handle = runtime.spawn(WebServer::run(
        server_id,
        db_pool.clone(),
        queue.clone(),
        capacity_manager,
        token,
        configuration.rpc_port,
    ));

    let listen_addr = format!("{}:{}", configuration.listen_address, configuration.listen_port)
        .parse()
        .expect("Just created address should be in a valid format");

    info!("Listening for clients");
    App::new()
        .add_plugins(TimePlugin)
        .add_plugins(TaskPoolPlugin::default())
        .insert_resource::<TaskCreator>(runtime.clone().into())
        .insert_resource::<DbPool>(db_pool.into())
        .add_plugins(ServerPlugin::new(configuration.game.clone(), server_id))
        .add_plugins(NetworkPlugin::new(listen_addr, runtime))
        .add_plugins(ReceivePlugin)
        .add_plugins(SynchronizationPlugin)
        .add_plugins(AgentPlugin)
        .add_plugins(PersistencePlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(LoginPlugin::new(queue))
        .add_plugins(GamePlugin)
        .add_plugins(MallPlugin)
        .add_plugins(CommandPlugin)
        .run();
}
