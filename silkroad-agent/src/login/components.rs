use crate::login::character_loader::DbCharacter;
use bevy_ecs::prelude::*;
use derive_more::{Deref, DerefMut};
use tokio::sync::oneshot::Receiver;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub(crate) struct Login;

#[derive(Component, Deref, DerefMut)]
#[component(storage = "SparseSet")]
pub(crate) struct CharactersLoading(pub(crate) Receiver<Vec<DbCharacter>>);

#[derive(Component, Deref, DerefMut)]
#[component(storage = "SparseSet")]
pub(crate) struct CharacterCheckName(pub(crate) Receiver<(String, bool)>);

#[derive(Component, Deref, DerefMut)]
#[component(storage = "SparseSet")]
pub(crate) struct CharacterDelete(pub(crate) Receiver<bool>);

#[derive(Component, Deref, DerefMut)]
#[component(storage = "SparseSet")]
pub(crate) struct CharacterCreate(pub(crate) Receiver<()>);

#[derive(Component, Deref, DerefMut)]
#[component(storage = "SparseSet")]
pub(crate) struct CharacterRestore(pub(crate) Receiver<bool>);

#[derive(Component, Default)]
pub(crate) struct CharacterSelect {
    pub(crate) characters: Option<Vec<DbCharacter>>,
    pub(crate) checked_name: Option<String>,
}
