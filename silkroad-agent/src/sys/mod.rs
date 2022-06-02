use crate::resources::Ticks;
use bevy_core::Time;
use bevy_ecs::system::ResMut;

pub mod charselect;
pub mod in_game;
pub mod login;
pub mod movement;
pub mod net;
pub mod visibility;

pub(crate) fn update_time(mut time: ResMut<Time>, mut ticks: ResMut<Ticks>) {
    time.update();
    ticks.increase();
}
