use bevy::prelude::*;
use derive_more::{Deref, DerefMut};
use silkroad_agent_persistence::{
    CharacterReadError, CharacterWriteError, ListedCharacter, WorldJoinCharacter, WorldJoinError,
};
use tokio::sync::oneshot::Receiver;

#[derive(Component, Deref, DerefMut)]
#[component(storage = "SparseSet")]
pub(crate) struct CharactersLoading(pub(crate) Receiver<Result<Vec<ListedCharacter>, CharacterReadError>>);

#[derive(Component, Deref, DerefMut)]
#[component(storage = "SparseSet")]
pub(crate) struct CharacterJoining(pub(crate) Receiver<Result<WorldJoinCharacter, WorldJoinError>>);

#[derive(Component, Deref, DerefMut)]
#[component(storage = "SparseSet")]
pub(crate) struct CharacterCheckName(pub(crate) Receiver<Result<(String, bool), CharacterWriteError>>);

#[derive(Component, Deref, DerefMut)]
#[component(storage = "SparseSet")]
pub(crate) struct CharacterDelete(pub(crate) Receiver<Result<bool, CharacterWriteError>>);

#[derive(Component, Deref, DerefMut)]
#[component(storage = "SparseSet")]
pub(crate) struct CharacterCreate(pub(crate) Receiver<Result<(), CharacterWriteError>>);

#[derive(Component, Deref, DerefMut)]
#[component(storage = "SparseSet")]
pub(crate) struct CharacterRestore(pub(crate) Receiver<Result<bool, CharacterWriteError>>);

#[derive(Component, Default)]
pub(crate) struct CharacterSelect {
    pub(crate) characters: Option<Vec<ListedCharacter>>,
    pub(crate) checked_name: Option<String>,
}
