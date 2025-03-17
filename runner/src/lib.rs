use dashmap::DashMap;
use futures_util::{future, Future, FutureExt};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::{broadcast, mpsc, watch, Notify, Semaphore};

mod task;
pub use task::{Task, TaskStatus, TASK_RUNTIME};

extern crate uuid;

pub trait EventHandler: Send + 'static {
    fn handle_event(&self, task_id: String, status: TaskStatus);
}

impl<F> EventHandler for F
where
    F: Fn(String, TaskStatus) + Send + 'static,
{
    fn handle_event(&self, task_id: String, status: TaskStatus) {
        (self)(task_id, status);
    }
}

pub trait ExecuteHandler<T, R>: Send + 'static {
    fn run(&self, data: T) -> impl Future<Output = Result<(), R>> + Send;
}

impl<FF, F, T, R> ExecuteHandler<T, R> for FF
where
    FF: Fn(T) -> F + Send + 'static + Sync,
    F: Future<Output = Result<(), R>> + Send,
    T: Send + Sync + Clone + 'static,
    R: Into<Box<dyn Error + Send + Sync>> + 'static,
{
    async fn run(&self, data: T) -> Result<(), R> {
        (self)(data).await
    }
}

#[derive(Clone)]
pub struct RunnerManger<T> {
    inner: Arc<Inner<T>>,
}

struct Inner<T> {
    status: Mutex<PoolStatus>,
    tasks: DashMap<String, Task<T>>,
    semaphore: Arc<Semaphore>,
    no_task: AtomicBool,
    add_task_notify: Notify,
    pool_status_notify: Notify,
    task_finish_notify: Arc<Notify>,
    pool_status_channel: watch::Sender<PoolStatus>,
    notification_channel: mpsc::UnboundedSender<(String, TaskLog)>,
    pause_channel: broadcast::Sender<()>,
}

#[derive(Eq, Debug, Hash, PartialEq, Clone, Deserialize, Serialize)]
#[repr(u8)]
pub enum PoolStatus {
    Running,
    Pause,
    PauseLoading,
}

#[derive(Debug)]
pub enum TaskLog {
    StatusChange(task::TaskStatus, String),
    ScheduleChange(u8, String),
}

impl<T> RunnerManger<T>
where
    T: Send + Sync + Clone + 'static,
{
    pub fn new<F, E, R>(permits: usize, auto_run: bool, execute: F, task_status_event: E) -> Self
    where
        F: ExecuteHandler<T, R> + Send + Sync + 'static,
        E: EventHandler + Send + Sync + 'static,
        R: Into<Box<dyn Error + Send + Sync>> + Display + 'static,
    {
        let status = if auto_run {
            PoolStatus::Running
        } else {
            PoolStatus::Pause
        };

        let (notification_tx, notification_rx) = mpsc::unbounded_channel();
        let (pause_tx, _) = broadcast::channel(1);
        let (pool_status_tx, _) = watch::channel(status.clone());

        let task_pool = RunnerManger {
            inner: Arc::new(Inner {
                status: Mutex::new(status),
                pool_status_notify: Notify::new(),
                semaphore: Arc::new(Semaphore::new(permits)),
                add_task_notify: Notify::new(),
                no_task: AtomicBool::new(false),
                tasks: DashMap::new(),
                notification_channel: notification_tx,
                pause_channel: pause_tx,
                pool_status_channel: pool_status_tx,
                task_finish_notify: Arc::new(Notify::new()),
            }),
        };

        tokio::spawn(
            task_pool
                .clone()
                .run_task_log(notification_rx, task_status_event),
        );

        tokio::spawn(task_pool.clone().run_with_task_type(execute));

        task_pool
    }

    async fn run_with_task_type<F, R>(self, execute: F) -> anyhow::Result<()>
    where
        F: ExecuteHandler<T, R> + Send + Sync + 'static,
        R: Into<Box<dyn Error + Send + Sync>> + Display + 'static,
    {
        let semaphore = self.inner.semaphore.clone();
        let execute = Arc::new(execute);
        loop {
            let permit = semaphore.clone().acquire_owned().await?;

            let pool_status = self.get_pool_status();
            if pool_status.eq(&PoolStatus::Pause) || pool_status.eq(&PoolStatus::PauseLoading) {
                self.inner.pool_status_notify.notified().await;
            }

            let next_task = match self.lock_task().await {
                Some(task) => task,
                None => {
                    // If the task is fetched and returned, the software has ended.
                    continue;
                }
            };

            let task_completion = async move {
                drop(permit);
            };
            let task_runner = next_task
                .1
                .run(
                    next_task.0,
                    self.inner.notification_channel.clone(),
                    self.inner.task_finish_notify.clone(),
                    self.inner.pause_channel.subscribe(),
                    execute.clone(),
                )
                .then(move |result| {
                    future::join(future::ready(result), task_completion).map(|(result, _)| result)
                });

            tokio::spawn(task_runner);
        }
    }

    async fn run_task_log<E>(
        self,
        mut rx: mpsc::UnboundedReceiver<(String, TaskLog)>,
        task_status_event: E,
    ) where
        E: EventHandler + Send + Sync + 'static,
    {
        while let Some(log) = rx.recv().await {
            match log.1 {
                TaskLog::StatusChange(status, msg) => {
                    self.set_task_status(&log.0, &status, msg).await;
                    task_status_event.handle_event(log.0, status);
                }
                TaskLog::ScheduleChange(schedule, msg) => {
                    self.set_task_schedule(&log.0, schedule, msg).await;
                }
            }
        }
    }

    pub fn get_all_task(&self) -> Vec<(String, Task<T>)> {
        self.inner
            .tasks
            .iter()
            .map(|t| (t.key().clone(), t.value().clone()))
            .collect()
    }

    pub fn add_task(&self, id: &str, task: &Task<T>) -> anyhow::Result<()> {
        self.inner.tasks.insert(id.to_string(), task.clone());

        if self
            .inner
            .no_task
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            self.inner.add_task_notify.notify_one();
        }

        Ok(())
    }

    pub fn init_tasks(&self, task: Vec<(String, Task<T>)>) {
        for t in task {
            self.inner.tasks.insert(t.0, t.1);
        }

        if self
            .inner
            .no_task
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            self.inner.add_task_notify.notify_one();
        }
    }

    pub fn remove_task(&self, id: &str) {
        self.inner.tasks.remove(id);
    }

    pub fn get_pool_status(&self) -> PoolStatus {
        self.inner.status.lock().clone()
    }

    pub async fn pause(&self) -> anyhow::Result<()> {
        self.set_pool_status(PoolStatus::PauseLoading)?;
        loop {
            self.inner.task_finish_notify.notified().await;
            if self.inner.pause_channel.receiver_count() == 0 {
                self.set_pool_status(PoolStatus::Pause)?;
                break;
            }
        }
        Ok(())
    }

    pub fn resume(&self) -> anyhow::Result<()> {
        self.inner.pool_status_notify.notify_waiters();
        self.set_pool_status(PoolStatus::Running)?;
        Ok(())
    }

    pub async fn force_pause(&self) -> anyhow::Result<()> {
        // Forced stops are executed very quickly, so the wait state is not sent. Only as a marker not to continue to the next task
        *self.inner.status.lock() = PoolStatus::PauseLoading;
        self.inner.pause_channel.send(()).unwrap();
        loop {
            self.inner.task_finish_notify.notified().await;
            if self.inner.pause_channel.receiver_count() == 0 {
                self.set_pool_status(PoolStatus::Pause)?;
                break;
            }
        }
        Ok(())
    }

    fn set_pool_status(&self, status: PoolStatus) -> anyhow::Result<()> {
        *self.inner.status.lock() = status.clone();
        if !self.inner.pool_status_channel.is_closed() {
            self.inner.pool_status_channel.send(status)?;
        }
        Ok(())
    }

    pub fn watch_pool_status(&self) -> watch::Receiver<PoolStatus> {
        self.inner.pool_status_channel.subscribe()
    }

    fn next_task(&self) -> Option<(String, Task<T>)>
    where
        T: Send + Sync + 'static,
    {
        self.inner.tasks.iter_mut().find_map(|mut t| {
            if t.value().status.eq(&task::TaskStatus::Wait) {
                // Directly updating the state prevents the next thread from continuing to get the
                t.value_mut().status = task::TaskStatus::Running;
                Some((t.key().clone(), t.value().clone()))
            } else {
                None
            }
        })
    }

    async fn lock_task(&self) -> Option<(String, Task<T>)>
    where
        T: Send + Sync + 'static,
    {
        match { self.next_task() } {
            Some(task) => Some(task),
            None => {
                self.inner
                    .no_task
                    .store(true, std::sync::atomic::Ordering::Relaxed);
                self.inner.add_task_notify.notified().await;
                self.next_task()
            }
        }
    }

    pub fn change_task_status(&self, id: &str, status: task::TaskStatus) {
        if let Some(mut task) = self.inner.tasks.get_mut(id) {
            task.status = status;
        }
    }

    pub(crate) async fn set_task_status(&self, id: &str, status: &task::TaskStatus, msg: String) {
        if let Some(mut task) = self.inner.tasks.get_mut(id) {
            task.status = status.clone();
            task.last_log = msg;
        }
    }

    pub(crate) async fn set_task_schedule(&self, id: &str, schedule: u8, msg: String) {
        if let Some(mut task) = self.inner.tasks.get_mut(id) {
            task.schedule = schedule;
            task.last_log = msg;
        }
    }
}

#[test]
fn tsts() {
    #[derive(Clone)]
    struct TaskTest {
        data: String,
    }

    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async move {
        let runner = RunnerManger::new(
            6,
            true,
            |data: TaskTest| async move {
                println!("{}", data.data);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                Err(anyhow::anyhow!("test"))
            },
            |task_id, status| match status {
                TaskStatus::Fail => log::error!("status_event:{:?} {}", status, task_id),
                _ => log::info!("status_event:{:?} {}", status, task_id),
            },
        );

        for _ in 0..5 {
            let task = Task::new(
                TaskTest {
                    data: "test".to_string(),
                },
                task::TaskStatus::Wait,
                "test".to_string(),
            );
            let id = uuid::Uuid::new_v4().to_string();
            runner.add_task(&id, &task).unwrap();
        }

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        match runner.pause().await {
            Ok(_) => println!("pause"),
            Err(e) => println!("pause error:{}", e),
        };
    });
}
