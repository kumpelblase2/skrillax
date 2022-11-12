pub(crate) mod drop;
pub(crate) mod monster;
pub(crate) mod net;
pub(crate) mod npc;
pub(crate) mod player;
pub(crate) mod pos;
pub(crate) mod stats;
pub(crate) mod sync;
pub(crate) mod visibility;

use crate::db::user::ServerUser;
use crate::login::character_loader::Character;
use crate::population::capacity::PlayingToken;
use bevy_ecs::prelude::*;
use bevy_time::{Timer, TimerMode};
use std::time::Duration;
use tokio::sync::oneshot::Receiver;

#[derive(Component)]
pub(crate) struct Login;

#[derive(Component, Default)]
pub(crate) struct CharacterSelect {
    pub(crate) characters: Option<Vec<Character>>,
    pub(crate) character_receiver: Option<Receiver<Vec<Character>>>,
    pub(crate) character_name_check: Option<Receiver<bool>>,
    pub(crate) character_delete_task: Option<Receiver<bool>>,
    pub(crate) checked_name: Option<String>,
    pub(crate) character_create: Option<Receiver<()>>,
    pub(crate) character_restore: Option<Receiver<bool>>,
}

#[derive(Component, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct GameEntity {
    pub unique_id: u32,
    pub ref_id: u32,
}

#[derive(Component)]
pub(crate) struct Playing(pub(crate) ServerUser, pub(crate) PlayingToken);

#[derive(Component)]
pub(crate) struct Health {
    pub current_health: u32,
    pub max_health: u32,
}

impl Health {
    pub fn new(max_health: u32) -> Self {
        Self {
            current_health: max_health,
            max_health,
        }
    }
}

#[derive(Hash, Copy, Clone, Eq, PartialEq)]
pub struct EntityReference(pub Entity, pub(crate) GameEntity);

#[derive(Clone, Component)]
pub struct Despawn(pub Timer);

impl Despawn {
    pub fn despawn_after_seconds(seconds: u64) -> Despawn {
        Despawn(Timer::from_seconds(seconds as f32, TimerMode::Once))
    }
}

impl From<Duration> for Despawn {
    fn from(duration: Duration) -> Self {
        Despawn(Timer::new(duration, TimerMode::Once))
    }
}
