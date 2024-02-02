use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use tokio_util::{sync::CancellationToken, task::TaskTracker};

use crate::{Config, UiHandle};
#[derive(Default)]
pub struct Application<U: UiHandle> {
    pub cancel: CancellationToken,
    pub config: Arc<Config>,
    ui: Arc<Mutex<U>>,
    pub tracker: TaskTracker,
}
impl<U: UiHandle> Clone for Application<U> {
    fn clone(&self) -> Self {
        Self {
            cancel: self.cancel.clone(),
            config: self.config.clone(),
            ui: self.ui.clone(),
            tracker: self.tracker.clone(),
        }
    }
}
impl<U: UiHandle> Application<U> {
    pub fn ui(&self) -> Result<MutexGuard<'_, U>, PoisonError<MutexGuard<'_, U>>> {
        self.ui.lock()
    }
    pub fn child_token(&self) -> CancellationToken {
        self.cancel.child_token()
    }
    pub async fn shutdown(self) {
        self.cancel.cancel();
        self.tracker.close();
        self.tracker.wait().await;
    }
}
impl<U: UiHandle + From<Config>> From<Config> for Application<U> {
    fn from(value: Config) -> Self {
        Self {
            cancel: CancellationToken::default(),
            tracker: TaskTracker::default(),
            ui: Arc::new(Mutex::new(U::from(value.clone()))),
            config: Arc::new(value),
        }
    }
}
