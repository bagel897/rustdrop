use std::{future::Future, sync::Arc};

use tokio::{
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;

use crate::{Config, UiHandle};
#[derive(Default)]
pub struct Application<U: UiHandle> {
    pub cancel: CancellationToken,
    pub config: Config,
    ui: Arc<RwLock<U>>,
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
    pub fn new(ui: U, config: Config) -> Self {
        Self {
            cancel: CancellationToken::default(),
            tasks: JoinSet::default(),
            ui: Arc::new(RwLock::new(ui)),
            config,
        }
    }
    pub fn spawn<F: Future<Output = ()> + Send + 'static>(&mut self, task: F, name: &str) {
        let builder = self.tasks.build_task().name(name);
        builder.spawn(task).unwrap();
    }
    pub async fn ui_write(&self) -> RwLockWriteGuard<'_, U> {
        self.ui.write().await
    }
    pub async fn ui(&self) -> RwLockReadGuard<'_, U> {
        self.ui.read().await
    }
    pub fn child_token(&self) -> CancellationToken {
        self.cancel.child_token()
    }
    pub async fn shutdown(mut self) {
        self.cancel.cancel();
        self.tasks.shutdown().await;
    }
    pub(crate) fn ui_ref(&self) -> Arc<RwLock<U>> {
        return self.ui.clone();
    }
}
impl<U: UiHandle + From<Config>> From<Config> for Application<U> {
    fn from(value: Config) -> Self {
        Self::new(U::from(value.clone()), value)
    }
}
