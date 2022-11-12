mod comp;
mod config;
mod db;
mod event;
mod ext;
mod game;
mod login;
mod net;
mod population;
mod server_plugin;
mod tasks;
mod world;

use crate::config::get_config;
use crate::db::server::ServerRegistration;
use crate::ext::DbPool;
use crate::game::GamePlugin;
use crate::login::LoginPlugin;
use crate::net::NetworkPlugin;
use crate::population::CapacityController;
use crate::population::LoginQueue;
use crate::server_plugin::ServerPlugin;
use crate::tasks::TaskCreator;
use crate::world::WorldPlugin;
use bevy_app::App;
use bevy_core::CorePlugin;
use bevy_time::TimePlugin;
use login::web::WebServer;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use silkroad_network::server::SilkroadServer;
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

    let capacity_manager = Arc::new(CapacityController::new(configuration.max_player_count));
    let queue = LoginQueue::new(capacity_manager.clone(), 30);

    let db_pool = runtime
        .block_on(configuration.database.create_pool())
        .expect("Should be able to create db pool");

    let token: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();

    runtime
        .block_on(ServerRegistration::setup(
            server_id,
            configuration.name.clone(),
            configuration.region.clone(),
            external_addr,
            configuration.rpc_port,
            token.clone(),
            db_pool.clone(),
        ))
        .expect("Should be able to register server");

    let _web_handle = runtime.spawn(
        WebServer::new(
            server_id,
            db_pool.clone(),
            queue.clone(),
            capacity_manager,
            token,
            configuration.rpc_port,
        )
        .run(),
    );

    let listen_addr = format!("{}:{}", configuration.listen_address, configuration.listen_port)
        .parse()
        .expect("Just created address should be in a valid format");
    let network = SilkroadServer::new(runtime.clone(), listen_addr).unwrap();

    info!("Listening for clients");
    App::new()
        .add_plugin(CorePlugin { ..Default::default() })
        .add_plugin(TimePlugin)
        .insert_resource::<DbPool>(db_pool.into())
        .insert_resource::<TaskCreator>(runtime.into())
        .add_plugin(ServerPlugin::new(configuration.game.clone(), server_id))
        .add_plugin(WorldPlugin)
        .add_plugin(NetworkPlugin::new(network))
        .add_plugin(LoginPlugin::new(queue))
        .add_plugin(GamePlugin)
        .run();
}
