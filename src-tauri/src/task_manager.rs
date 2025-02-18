use std::future::Future;

use tokio_util::{sync::CancellationToken, task::TaskTracker};

#[derive(Debug, Clone, Default)]
pub struct TaskManager {
    pub tracker: TaskTracker,
    pub token: CancellationToken,
}

impl TaskManager {
    #[allow(unused)]
    pub fn spawn_result_task(
        &self,
        fut: impl Future<Output = anyhow::Result<()>> + Send + 'static,
    ) {
        let token = self.token.clone();
        self.tracker.spawn(async move {
            tokio::select! {
                () = token.cancelled() => {
                    return;
                }
                res = fut => {
                    if let Err(err) = res {
                        tracing::error!("Task error: {err:?}");
                    }
                }
            }
        });
    }

    #[allow(unused)]
    pub fn spawn_task(&self, fut: impl Future<Output = ()> + Send + 'static) {
        let token = self.token.clone();
        self.tracker.spawn(async move {
            tokio::select! {
                () = token.cancelled() => {
                    return;
                }
                () = fut => {
                }
            }
        });
    }

    pub async fn shutdown(&mut self) {
        self.token.cancel();
        self.tracker.close();
        self.tracker.wait().await;
    }
}
