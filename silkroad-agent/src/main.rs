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
mod settings;
mod time;
mod world;

use crate::config::get_config;
use crate::game::GamePlugin;
use crate::login::LoginPlugin;
use crate::net::NetworkPlugin;
use crate::population::capacity::CapacityController;
use crate::population::queue::LoginQueue;
use crate::server_plugin::ServerPlugin;
use crate::settings::GameSettings;
use crate::world::WorldPlugin;
use bevy_app::App;
use bevy_core::CorePlugin;
use login::web::WebServer;
use silkroad_network::server::SilkroadServer;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tracing::info;

const DEFAULT_RPC_PORT: u16 = 1337;
const DEFAULT_LISTEN_ADDRESS: &str = "0.0.0.0";

fn main() {
    tracing_subscriber::fmt::init();

    let configuration = get_config();
    let external_addr = match &configuration.external_address {
        Some(addr) => SocketAddr::from_str(addr),
        None => SocketAddr::from_str(&format!("127.0.0.1:{}", configuration.listen_port)),
    }
    .unwrap();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(4)
        .thread_name("async-worker")
        .build()
        .unwrap();
    let runtime = Arc::new(runtime);

    let capacity_manager = Arc::new(CapacityController::new(configuration.max_player_count));
    let queue = LoginQueue::new(capacity_manager.clone(), 30);

    let db_pool = runtime.block_on(configuration.database.create_pool()).unwrap();

    let _web_handle = runtime.spawn(
        WebServer::new(
            configuration.server_id,
            db_pool.clone(),
            queue.clone(),
            capacity_manager.clone(),
            external_addr,
            configuration.rpc_port.unwrap_or(DEFAULT_RPC_PORT),
        )
        .run(),
    );

    let listen_address = configuration
        .listen_address
        .as_ref()
        .map(|addr| addr.as_ref())
        .unwrap_or(DEFAULT_LISTEN_ADDRESS);
    let listen_addr = format!("{}:{}", listen_address, configuration.listen_port)
        .parse()
        .unwrap();
    let network = SilkroadServer::new(runtime.clone(), listen_addr).unwrap();

    info!("Listening for clients");
    App::new()
        .add_plugin(CorePlugin)
        .insert_resource(db_pool)
        .insert_resource(runtime)
        .add_plugin(ServerPlugin::new(configuration.game.clone(), configuration.server_id))
        .add_plugin(WorldPlugin)
        .add_plugin(NetworkPlugin::new(network))
        .add_plugin(LoginPlugin::new(queue))
        .add_plugin(GamePlugin)
        .run();
}
