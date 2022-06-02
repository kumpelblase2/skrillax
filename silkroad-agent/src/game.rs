use crate::event::ServerEvent;
use crate::id_allocator::IdAllocator;
use crate::resources::Ticks;
use crate::settings::GameSettings;
use crate::sys::charselect::charselect;
use crate::sys::in_game::in_game;
use crate::sys::login::login;
use crate::sys::movement::movement;
use crate::sys::net::{accept, disconnected, receive};
use crate::sys::update_time;
use crate::sys::visibility::{player_visibility_update, visibility};
use crate::{CharacterLoaderFacade, JobCoordinator, LoginQueue};
use bevy_core::Time;
use bevy_ecs::event::Events;
use bevy_ecs::schedule::{Schedule, Stage, SystemStage};
use bevy_ecs::world::World;
use pk2::Pk2;
use silkroad_navmesh::NavmeshLoader;
use silkroad_network::server::SilkroadServer;
use std::path::Path;

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
        world.insert_resource(Time::default());
        world.insert_resource(Events::<ServerEvent>::default());
        world.insert_resource(network);
        world.insert_resource(IdAllocator::new());
        world.insert_resource(character_loader);
        world.insert_resource(job_coordinator);
        world.insert_resource(login_queue);
        let silkroad_folder = Path::new(&settings.data_location);
        let data_file = silkroad_folder.join("Data.pk2");
        world.insert_resource(NavmeshLoader::new(Pk2::open(data_file, BLOWFISH_KEY).unwrap()));
        world.insert_resource(settings);

        world
    }

    fn setup_schedule() -> Schedule {
        let mut schedule = Schedule::default();

        schedule.add_stage("base", SystemStage::single(update_time));

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
        schedule.add_stage(
            "world",
            SystemStage::parallel()
                .with_system(visibility)
                .with_system(player_visibility_update)
                .with_system(movement),
        );

        schedule
    }

    pub(crate) fn tick(&mut self) {
        self.schedule.run(&mut self.world);
    }
}
