use std::{
    future::Future,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
};

use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

use crate::{Config, UiHandle};
#[derive(Default)]
pub struct Application<U: UiHandle> {
    pub cancel: CancellationToken,
    pub config: Config,
    ui: Arc<Mutex<U>>,
    tasks: JoinSet<()>,
}
impl<U: UiHandle> Clone for Application<U> {
    fn clone(&self) -> Self {
        Self {
            cancel: self.cancel.clone(),
            config: self.config.clone(),
            ui: self.ui.clone(),
            tasks: JoinSet::new(),
        }
    }
}
impl<U: UiHandle> Application<U> {
    pub fn spawn<F: Future<Output = ()> + Send + 'static>(&mut self, task: F, name: &str) {
        let builder = self.tasks.build_task().name(name);
        builder.spawn(task).unwrap();
    }
    pub fn ui(&self) -> Result<MutexGuard<'_, U>, PoisonError<MutexGuard<'_, U>>> {
        self.ui.lock()
    }
    pub fn child_token(&self) -> CancellationToken {
        self.cancel.child_token()
    }
    pub async fn shutdown(mut self) {
        self.cancel.cancel();
        self.tasks.shutdown().await;
    }
}
impl<U: UiHandle + From<Config>> From<Config> for Application<U> {
    fn from(value: Config) -> Self {
        Self {
            cancel: CancellationToken::default(),
            tasks: JoinSet::default(),
            ui: Arc::new(Mutex::new(U::from(value.clone()))),
            config: value,
        }
    }
}
