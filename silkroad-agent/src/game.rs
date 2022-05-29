use crate::event::ServerEvent;
use crate::resources::{CurrentTime, Delta, Ticks};
use crate::settings::GameSettings;
use crate::sys::charselect::charselect;
use crate::sys::in_game::in_game;
use crate::sys::login::login;
use crate::sys::movement::movement;
use crate::sys::net::{accept, disconnected, receive};
use crate::{CharacterLoaderFacade, JobCoordinator, LoginQueue};
use bevy_ecs::event::Events;
use bevy_ecs::schedule::{Schedule, Stage, SystemStage};
use bevy_ecs::system::SystemState;
use bevy_ecs::world::World;
use pk2::Pk2;
use silkroad_navmesh::NavmeshLoader;
use silkroad_network::server::SilkroadServer;
use std::path::Path;
use std::time::Instant;

pub(crate) struct Game {
    world: World,
    schedule: Schedule,
}

const BLOWFISH_KEY: &'static str = "169841";

impl Game {
    pub fn new(
        network: SilkroadServer,
        settings: GameSettings,
        login_queue: LoginQueue,
        character_loader: CharacterLoaderFacade,
        job_coordinator: JobCoordinator,
    ) -> Self {
        Game {
            world: Self::setup_world(network, character_loader, job_coordinator, login_queue, settings),
            schedule: Self::setup_schedule(),
        }
    }

    fn setup_world(
        network: SilkroadServer,
        character_loader: CharacterLoaderFacade,
        job_coordinator: JobCoordinator,
        login_queue: LoginQueue,
        settings: GameSettings,
    ) -> World {
        let mut world = World::new();
        world.insert_resource(Ticks::default());
        world.insert_resource(Delta::default());
        world.insert_resource(CurrentTime::default());
        world.insert_resource(settings);
        world.insert_resource(Events::<ServerEvent>::default());
        world.insert_resource(network);
        world.insert_resource(character_loader);
        world.insert_resource(job_coordinator);
        world.insert_resource(login_queue);
        let silkroad_folder = Path::new("/home/tim/Games/silkroad-online-2/drive_c/Program Files (x86)/Silkroad");
        let data_file = silkroad_folder.join("Data.pk2");
        world.insert_resource(NavmeshLoader::new(Pk2::open(data_file, BLOWFISH_KEY).unwrap()));

        world
    }

    fn setup_schedule() -> Schedule {
        let mut schedule = Schedule::default();

        schedule.add_stage(
            "incoming-net",
            SystemStage::parallel()
                .with_system(accept)
                .with_system(receive)
                .with_system(disconnected),
        );
        schedule.add_stage(
            "handle-packet",
            SystemStage::parallel()
                .with_system(login)
                .with_system(charselect)
                .with_system(in_game),
        );
        schedule.add_stage("world", SystemStage::parallel().with_system(movement));

        schedule
    }

    pub(crate) fn tick(&mut self, delta: f64) {
        let current_time = Instant::now();
        self.world.get_resource_mut::<Ticks>().unwrap().increase();
        self.world.get_resource_mut::<Delta>().unwrap().0 = delta;
        self.world.get_resource_mut::<CurrentTime>().unwrap().0 = current_time;
        self.schedule.run(&mut self.world);
    }
}
