use bevy_ecs_macros::Resource;
use std::future::Future;
use std::ops::Deref;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::oneshot::Receiver;

#[derive(Resource)]
pub(crate) struct TaskCreator(Arc<Runtime>);

impl From<Arc<Runtime>> for TaskCreator {
    fn from(runtime: Arc<Runtime>) -> Self {
        TaskCreator(runtime)
    }
}

impl Clone for TaskCreator {
    fn clone(&self) -> Self {
        TaskCreator(self.0.clone())
    }
}

impl Deref for TaskCreator {
    type Target = Runtime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TaskCreator {
    pub fn create_task<F>(&self, task: F) -> Receiver<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send,
    {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.spawn(async move {
            let result = task.await;
            let _ = sender.send(result);
        });
        receiver
    }
}
