extern crate core;

mod character_loader;
mod comp;
mod config;
mod db;
mod event;
mod executor;
pub(crate) mod ext;
mod game;
mod job_coordinator;
mod player_loader;
mod population;
mod resources;
mod settings;
mod sys;
mod time;
mod web;

use crate::character_loader::{CharacterLoader, CharacterLoaderFacade};
use crate::config::get_config;
use crate::executor::Executor;
use crate::game::Game;
use crate::job_coordinator::JobCoordinator;
use crate::population::capacity::CapacityController;
use crate::population::queue::LoginQueue;
use crate::settings::GameSettings;
use crate::web::WebServer;
use silkroad_network::server::SilkroadServer;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tracing::info;

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

    let capacity_manager = Arc::new(CapacityController::new(5));
    let queue = LoginQueue::new(capacity_manager.clone(), 30);

    let db_pool = runtime.block_on(configuration.database.create_pool()).unwrap();

    let _web_handle = runtime.spawn(
        WebServer::new(
            configuration.server_id,
            db_pool.clone(),
            queue.clone(),
            capacity_manager.clone(),
            external_addr,
            configuration.rpc_port.unwrap_or(1337),
        )
        .run(),
    );

    let mut coordinator = JobCoordinator::new(db_pool.clone(), configuration.server_id);
    runtime.block_on(coordinator.load());
    let character_loader =
        CharacterLoaderFacade::new(runtime.clone(), CharacterLoader::new(db_pool, configuration.server_id));

    let listen_addr = SocketAddr::from_str(&format!("0.0.0.0:{}", configuration.listen_port)).unwrap();
    let network = runtime.block_on(SilkroadServer::new(listen_addr)).unwrap();

    let game = Game::new(
        network,
        configuration.game.clone().unwrap_or_default().into(),
        queue,
        character_loader,
        coordinator,
    );
    let mut executor = Executor::new(game, 128);
    info!("Listening for clients");
    executor.run();
}
