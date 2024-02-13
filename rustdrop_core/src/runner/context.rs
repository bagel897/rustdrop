use std::future::Future;

use tokio::task::JoinSet;

use crate::Config;
#[derive(Default)]
pub struct Context {
    pub config: Config,
    tasks: JoinSet<()>,
}
impl Clone for Context {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            tasks: JoinSet::new(),
        }
    }
}
impl Context {
    pub fn new(config: Config) -> Self {
        Self {
            tasks: JoinSet::default(),
            config,
        }
    }
    pub fn spawn<F: Future<Output = ()> + Send + 'static>(&mut self, task: F, name: &str) {
        let builder = self.tasks.build_task().name(name);
        builder.spawn(task).unwrap();
    }
    pub async fn clean_shutdown(&mut self) {
        while let Some(task) = self.tasks.join_next().await {
            task.unwrap();
        }
    }
    pub async fn shutdown(mut self) {
        self.tasks.shutdown().await;
    }
}
impl From<Config> for Context {
    fn from(value: Config) -> Self {
        Self::new(value)
    }
}
