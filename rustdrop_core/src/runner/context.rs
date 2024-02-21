use std::future::Future;

use tokio_util::task::TaskTracker;

use crate::Config;
#[derive(Default, Debug, Clone)]
pub struct Context {
    pub config: Config,
    tasks: TaskTracker,
}
impl Context {
    pub fn new(config: Config) -> Self {
        Self {
            tasks: TaskTracker::default(),
            config,
        }
    }
    pub fn spawn<F: Future<Output = ()> + Send + 'static>(&self, task: F) {
        self.tasks.spawn(task);
    }
    pub async fn shutdown(self) {
        self.tasks.close();
        self.tasks.wait().await;
    }
}
impl From<Config> for Context {
    fn from(value: Config) -> Self {
        Self::new(value)
    }
}
