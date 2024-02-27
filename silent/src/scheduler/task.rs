use crate::scheduler::process_time::ProcessTime;
use anyhow::Result;
use serde::Serialize;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type JobToRun = dyn Fn() -> Result<()> + Send + Sync;
pub type JobToRunAsync = dyn Fn() -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync;

#[derive(Clone, Serialize)]
pub struct Task {
    pub id: String,
    pub process_time: ProcessTime,
    pub description: String,
    #[serde(skip)]
    action: Arc<JobToRun>,
    #[serde(skip)]
    action_async: Arc<JobToRunAsync>,
    #[serde(skip)]
    pub(crate) is_async: bool,
}

impl Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Task")
            .field("id", &self.id)
            .field("process_time", &self.process_time)
            .field("description", &self.description)
            .field("is_async", &self.is_async)
            .finish()
    }
}

impl Task {
    pub(crate) fn run(&self) -> Result<()> {
        match self.is_async {
            true => Err(anyhow::anyhow!("async task not support run")),
            false => match self.process_time.is_active() {
                true => (self.action.clone())(),
                false => Ok(()),
            },
        }
    }
    pub(crate) async fn run_async(&self) -> Result<()> {
        match self.is_async {
            true => match self.process_time.is_active() {
                true => (self.action_async.clone())().await,
                false => Ok(()),
            },
            false => Err(anyhow::anyhow!("sync task not support run_async")),
        }
    }

    pub fn create_with_action(
        id: String,
        process_time: ProcessTime,
        description: String,
        action: Arc<JobToRun>,
    ) -> Self {
        Self {
            id,
            process_time,
            description,
            action,
            action_async: Arc::new(|| Box::pin(async { Ok(()) })),
            is_async: false,
        }
    }

    pub fn create_with_action_async(
        id: String,
        process_time: ProcessTime,
        description: String,
        action_async: Arc<JobToRunAsync>,
    ) -> Self {
        Self {
            id,
            process_time,
            description,
            action: Arc::new(|| Ok(())),
            action_async,
            is_async: true,
        }
    }

    pub(crate) fn is_removable(&self) -> bool {
        match self.process_time {
            ProcessTime::Datetime(_) => self.process_time.is_active(),
            ProcessTime::Crontab(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::scheduler::process_time::ProcessTime;
    use crate::scheduler::task::Task;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_task() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        let mut task = Task::create_with_action(
            "test".to_string(),
            ProcessTime::try_from("9999-01-01T00:00:00Z".to_string()).unwrap(),
            "test".to_string(),
            Arc::new(move || {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }),
        );
        println!("{:?}", task);
        task.run().unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 0);
        task.process_time = ProcessTime::try_from("2023-01-01T00:00:00Z".to_string()).unwrap();
        task.run().unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_task_async_error() {
        let sync_task = Task::create_with_action(
            "test".to_string(),
            ProcessTime::try_from("2015-01-01T00:00:00Z".to_string()).unwrap(),
            "test".to_string(),
            Arc::new(move || Ok(())),
        );
        assert!(sync_task.run_async().await.is_err());
        assert!(sync_task.run().is_ok());
        assert!(sync_task.is_removable());
        let async_task = Task::create_with_action_async(
            "test".to_string(),
            ProcessTime::try_from("* * * * * * *".to_string()).unwrap(),
            "test".to_string(),
            Arc::new(move || Box::pin(async move { Ok(()) })),
        );
        assert!(async_task.run().is_err());
        assert!(async_task.run_async().await.is_ok());
        assert!(!async_task.is_removable())
    }

    #[tokio::test]
    async fn test_task_async() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        let mut task = Task::create_with_action_async(
            "test".to_string(),
            ProcessTime::try_from("9999-01-01T00:00:00Z".to_string()).unwrap(),
            "test".to_string(),
            Arc::new(move || {
                let counter_clone = counter_clone.clone();
                Box::pin(async move {
                    counter_clone.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
            }),
        );
        task.run_async().await.unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 0);
        task.process_time = ProcessTime::try_from("2023-01-01T00:00:00Z".to_string()).unwrap();
        task.run_async().await.unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
