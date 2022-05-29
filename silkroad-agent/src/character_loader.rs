use crate::db::character::{
    fetch_characters, fetch_characters_items, CharacterData, CharacterItem,
};
use bevy_ecs::entity::Entity;
use crossbeam_channel::{Receiver, TryRecvError};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tracing::trace_span;

#[derive(Clone)]
pub struct Character {
    pub(crate) character_data: CharacterData,
    pub(crate) items: Vec<CharacterItem>,
}

pub(crate) struct CharacterLoader {
    pool: PgPool,
    server_id: u16,
}

impl CharacterLoader {
    pub fn new(pool: PgPool, shard: u16) -> Self {
        CharacterLoader {
            pool,
            server_id: shard,
        }
    }

    pub(crate) async fn load_characters_sparse(&self, user_id: i32) -> Vec<Character> {
        let pool = self.pool.clone();
        let characters: Vec<CharacterData> = fetch_characters(&pool, user_id, self.server_id)
            .await
            .unwrap();

        let character_ids = characters.iter().map(|char| char.id).collect();
        let mut character_items = fetch_characters_items(&pool, character_ids).await.unwrap();

        let mut all_characters = Vec::new();

        for character in characters {
            let items = character_items.remove(&character.id).unwrap_or_default();

            all_characters.push(Character {
                character_data: character,
                items,
            });
        }

        all_characters
    }
}

pub(crate) struct CharacterLoaderFacade {
    runtime: Arc<Runtime>,
    loader: Arc<CharacterLoader>,
    callbacks: HashMap<Entity, Receiver<Vec<Character>>>,
}

impl CharacterLoaderFacade {
    pub fn new(runtime: Arc<Runtime>, loader: CharacterLoader) -> Self {
        CharacterLoaderFacade {
            runtime,
            loader: Arc::new(loader),
            callbacks: HashMap::new(),
        }
    }

    pub fn load_data(&mut self, entity: Entity, user_id: i32) {
        if self.callbacks.contains_key(&entity) {
            return;
        }

        let (tx, rx) = crossbeam_channel::bounded(1);
        let loader = self.loader.clone();
        self.runtime.spawn(async move {
            let id = user_id;
            let span = trace_span!("Load Characters", user = id);
            let _guard = span.enter();
            let chars = loader.load_characters_sparse(user_id).await;
            tx.send(chars).unwrap();
        });
        self.callbacks.insert(entity.clone(), rx);
    }

    pub fn characters(&mut self, entity: Entity) -> Option<Vec<Character>> {
        let receiver = self.callbacks.get(&entity)?;
        match receiver.try_recv() {
            Ok(characters) => {
                self.callbacks.remove(&entity);
                Some(characters)
            }
            Err(TryRecvError::Empty) => None,
            Err(_) => {
                todo!("handle error of character data receiver")
            }
        }
    }
}
