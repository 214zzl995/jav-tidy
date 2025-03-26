use std::path::PathBuf;

use crate::{config::AppConfig, nfo::CrawlerNfo};
use anyhow::Ok;
use runner::{RunnerManger, TaskStatus};
use tokio::sync::mpsc;

pub fn initial(config: &AppConfig, file_rx: mpsc::Receiver<PathBuf>) -> anyhow::Result<()> {
    RunnerManger::new(
        config.thread_limit,
        true,
        |path: PathBuf| async move {
            CrawlerNfo::crawler(&path).await?;
            Ok(())
        },
        |task_id: String, status: TaskStatus| {
            
        },
    );
    Ok(())
}

impl CrawlerNfo {
    async fn crawler(path: &PathBuf) -> anyhow::Result<Self> {
        Ok(Self::default())
    }
}
