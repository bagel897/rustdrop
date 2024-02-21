use std::future::Future;

use tokio::task::Builder;
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
    pub fn spawn<F: Future<Output = ()> + Send + 'static>(&self, task: F, name: &str) {
        Builder::new()
            .name(name)
            .spawn(self.tasks.track_future(task))
            .unwrap();
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
