use std::{error::Error, fmt::Display, sync::Arc};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, Notify};

use crate::{ExecuteHandler, TaskLog};

tokio::task_local! {
   pub static TASK_RUNTIME: (String, mpsc::UnboundedSender<(String, TaskLog)>);
}

#[derive(Clone, Debug, Hash)]
pub struct Task<T> {
    pub status: TaskStatus,
    pub schedule: u8,
    pub last_log: String,
    pub create_at: DateTime<Local>,
    pub metadata: T,
}

#[derive(Eq, Hash, PartialEq, Clone, Debug, Deserialize, Serialize)]
#[repr(u8)]
pub enum TaskStatus {
    Wait,
    Running,
    Fail,
    Success,
    Pause,
}

impl From<u8> for TaskStatus {
    fn from(status: u8) -> Self {
        match status {
            0 => TaskStatus::Wait,
            1 => TaskStatus::Running,
            2 => TaskStatus::Fail,
            3 => TaskStatus::Success,
            4 => TaskStatus::Pause,
            _ => TaskStatus::Wait,
        }
    }
}

#[macro_export]
macro_rules! schedule {
    ($schedule:expr,$($arg:tt)*) => {
        #[allow(unreachable_code)]
        $crate::TASK_RUNTIME.with(|(id, tx)| {
            tx.send((id.clone(), $crate::TaskLog::ScheduleChange($schedule, format!($($arg)*)))).unwrap();
        });
    }
}

impl<T> Task<T>
where
    T: Send + Sync + 'static + Clone,
{
    pub fn new(task: T, initial_status: TaskStatus, last_log: String) -> Self {
        Task {
            status: initial_status,
            schedule: 0,
            last_log,
            create_at: Local::now(),
            metadata: task,
        }
    }

    pub(crate) async fn run<F, R>(
        self,
        task_id: String,
        log_tx: mpsc::UnboundedSender<(String, TaskLog)>,
        task_finish_notify: Arc<Notify>,
        mut pause_rx: broadcast::Receiver<()>,
        execute: Arc<F>,
    ) -> anyhow::Result<()>
    where
        F: ExecuteHandler<T, R> + Send + Sync + 'static,
        R: Into<Box<dyn Error + Send + Sync>> + Display + 'static,
    {
        log_tx.send((
            task_id.clone(),
            TaskLog::StatusChange(TaskStatus::Running, "Task is running".to_string()),
        ))?;

        let task_run = TASK_RUNTIME.scope((task_id.clone(), log_tx.clone()), async move {
            execute.run(self.metadata).await
        });

        tokio::select! {
            res = task_run => {
                match res {
                    Ok(_) => {
                        log_tx.send((task_id.clone(), TaskLog::StatusChange(TaskStatus::Success, "Successful task execution".to_string()))).unwrap();
                    }
                    Err(err) => {
                        log_tx.send((task_id.clone(), TaskLog::StatusChange(TaskStatus::Fail, format!("Task execution error, error reason: {}", err)))).unwrap();
                    }
                }
            }
            _ = pause_rx.recv() => {
                if !log_tx.is_closed() {
                   log_tx.send((task_id.clone(), TaskLog::StatusChange(TaskStatus::Fail, "Forced cessation of the mission".to_string()))).unwrap();
                }
            }
        }

        task_finish_notify.notify_one();

        Ok(())
    }
}
