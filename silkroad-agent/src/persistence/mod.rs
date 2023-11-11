use crate::comp::player::Player;
use crate::config::GameConfig;
use crate::event::ClientDisconnectedEvent;
use crate::ext::DbPool;
use crate::tasks::TaskCreator;
pub use apply::ApplyToDatabase;
use bevy_app::{App, Plugin, PostUpdate};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::*;
use bevy_ecs::ptr::Ptr;
use bevy_ecs::system::Commands;
use bevy_ecs_macros::Component;
use bevy_time::common_conditions::on_timer;
use futures::future::join_all;
use silkroad_game_base::{ChangeProvided, ChangeTracked, ToOptimizedChange};
use std::mem;
use std::ops::Deref;
use std::time::Duration;
use tracing::error;

mod apply;

#[derive(Component)]
pub struct Persistable;

#[derive(Component)]
struct PersistenceCollection<T: ChangeTracked + Component> {
    changes: Vec<T::ChangeItem>,
}

impl<T: ChangeTracked + Component> Default for PersistenceCollection<T> {
    fn default() -> Self {
        PersistenceCollection { changes: Vec::new() }
    }
}

struct PersistenceInfo {
    component: ComponentId,
    change_provider: fn(Ptr) -> Box<dyn ApplyToDatabase>,
}

#[derive(Resource, Default)]
struct PersistedComponents(Vec<PersistenceInfo>);

pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        let persist_interval = app
            .world
            .get_resource::<GameConfig>()
            .expect("Game config should exist.")
            .persist_interval;
        app.init_resource::<PersistedComponents>()
            .add_systems(PostUpdate, apply_changes_combined)
            .add_systems(
                PostUpdate,
                apply_changes_periodically.run_if(on_timer(Duration::from_secs(persist_interval))),
            );
    }
}

pub(crate) trait AppPersistanceExt {
    fn track_component<T: ChangeTracked + Component>(&mut self) -> &mut Self
    where
        T::ChangeItem: ApplyToDatabase;

    fn track_change_component<T: ChangeProvided + Component>(&mut self) -> &mut Self
    where
        T::Change: ApplyToDatabase;
}

impl AppPersistanceExt for App {
    fn track_component<T: ChangeTracked + Component>(&mut self) -> &mut Self
    where
        T::ChangeItem: ApplyToDatabase,
    {
        let persist_interval = self
            .world
            .get_resource::<GameConfig>()
            .expect("Game config should exist.")
            .persist_interval;
        self.add_systems(
            PostUpdate,
            (
                add_change_tracker::<T>,
                collect_changes::<T>,
                apply_changes_exit::<T>.after(collect_changes::<T>),
            ),
        )
        .add_systems(
            PostUpdate,
            apply_changes::<T>
                .run_if(on_timer(Duration::from_secs(persist_interval)))
                .after(collect_changes::<T>),
        );
        self
    }

    fn track_change_component<T: ChangeProvided + Component>(&mut self) -> &mut Self
    where
        T::Change: ApplyToDatabase,
    {
        let comp = self.world.init_component::<T>();
        let mut persistence_collection = self
            .world
            .get_resource_mut::<PersistedComponents>()
            .expect("Persistence plugin should be initialized.");

        persistence_collection.0.push(PersistenceInfo {
            component: comp,
            change_provider: |c| {
                let c: &T = unsafe { c.deref::<T>() };
                Box::new(c.as_change())
            },
        });
        self
    }
}

fn add_change_tracker<T: ChangeTracked + Component>(mut cmd: Commands, query: Query<Entity, Added<T>>) {
    for entity in query.iter() {
        let comp: PersistenceCollection<T> = PersistenceCollection::default();
        cmd.entity(entity).insert(comp);
    }
}

fn collect_changes<T: ChangeTracked + Component>(mut query: Query<(&mut T, &mut PersistenceCollection<T>)>) {
    for (mut change_source, mut change_collection) in query.iter_mut() {
        let mut changes = change_source.bypass_change_detection().changes();
        if !changes.is_empty() {
            change_collection.changes.append(&mut changes);
        }
    }
}

fn apply_changes<T: ChangeTracked + Component>(
    mut query: Query<(&Player, &mut PersistenceCollection<T>)>,
    task_creator: Res<TaskCreator>,
    pool: Res<DbPool>,
) where
    T::ChangeItem: ApplyToDatabase,
{
    for (player, mut changes) in query.iter_mut() {
        let changes = mem::take(&mut changes.changes);
        let optimized = changes.optimize();
        let character_id = player.character.id;
        let pool = pool.deref().deref().clone();
        task_creator.spawn(async move {
            let results = join_all(optimized.iter().map(|c| c.apply(character_id, &pool))).await;
            for e in results.into_iter().filter_map(|res| res.err()) {
                error!(error = %e, character_id = character_id, "Could not apply update");
            }
        });
    }
}

fn apply_changes_exit<T: ChangeTracked + Component>(
    mut query: Query<(&Player, &mut PersistenceCollection<T>)>,
    mut event_reader: EventReader<ClientDisconnectedEvent>,
    task_creator: Res<TaskCreator>,
    pool: Res<DbPool>,
) where
    T::ChangeItem: ApplyToDatabase,
{
    for event in event_reader.iter() {
        if let Ok((player, mut changes)) = query.get_mut(event.0) {
            let changes = mem::take(&mut changes.changes);
            let optimized = changes.optimize();
            let character_id = player.character.id;
            let pool = pool.deref().deref().clone();
            task_creator.spawn(async move {
                for change in optimized {
                    if let Err(e) = change.apply(character_id, &pool).await {
                        error!(error = %e, character_id = character_id, "Could not apply update");
                    }
                }
            });
        }
    }
}

fn apply_changes_combined(
    components: Res<PersistedComponents>,
    mut disconnections: EventReader<ClientDisconnectedEvent>,
    task_creator: Res<TaskCreator>,
    db_pool: Res<DbPool>,
    query: Query<(EntityRef, &Player)>,
) {
    if components.0.is_empty() {
        return;
    }

    for event in disconnections.iter() {
        let Ok((entity, player)) = query.get(event.0) else {
            continue;
        };

        let character_id = player.character.id;
        for config in components.0.iter() {
            let Some(ptr) = entity.get_by_id(config.component) else {
                continue;
            };

            let change = (config.change_provider)(ptr);
            let pool = db_pool.deref().deref().clone();

            task_creator.spawn(async move {
                if let Err(e) = change.apply(character_id, &pool).await {
                    error!(error = %e, character_id = character_id, "Could not apply update");
                }
            });
        }
    }
}

fn apply_changes_periodically(
    components: Res<PersistedComponents>,
    task_creator: Res<TaskCreator>,
    db_pool: Res<DbPool>,
    query: Query<(EntityRef, &Player)>,
) {
    if components.0.is_empty() {
        return;
    }

    for (entity, player) in query.iter() {
        let character_id = player.character.id;
        for config in components.0.iter() {
            let Some(ptr) = entity.get_by_id(config.component) else {
                continue;
            };

            let change = (config.change_provider)(ptr);
            let pool = db_pool.deref().deref().clone();

            task_creator.spawn(async move {
                if let Err(e) = change.apply(character_id, &pool).await {
                    error!(error = %e, character_id = character_id, "Could not apply update");
                }
            });
        }
    }
}
