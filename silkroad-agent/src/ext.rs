use cgmath::{Vector2, Vector3};
use std::future::Future;
use tokio::runtime::Runtime;
use tokio::sync::oneshot::Receiver;

pub(crate) trait Vector3Ext {
    fn to_flat_vec2(&self) -> Vector2<f32>;
}

impl Vector3Ext for Vector3<f32> {
    fn to_flat_vec2(&self) -> Vector2<f32> {
        Vector2::new(self.x, self.z)
    }
}

pub(crate) trait AsyncTaskCreate {
    fn create_task<F>(&self, task: F) -> Receiver<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send;
}

impl AsyncTaskCreate for Runtime {
    fn create_task<F>(&self, task: F) -> Receiver<F::Output>
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
