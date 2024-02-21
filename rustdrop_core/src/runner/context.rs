use std::future::Future;

use rand::thread_rng;
use tokio_util::task::TaskTracker;

use crate::{core::bits::EndpointInfo, Config};
#[derive(Debug, Clone)]
pub struct Context {
    pub config: Config,
    tasks: TaskTracker,
    pub endpoint_info: EndpointInfo,
}
impl Context {
    pub fn new(config: Config) -> Self {
        let mut rng = thread_rng();
        let endpoint_info = EndpointInfo::new(&config, &mut rng);
        Self {
            tasks: TaskTracker::default(),
            config,
            endpoint_info,
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
