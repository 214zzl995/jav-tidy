use std::path::PathBuf;

use crate::{config::AppConfig, nfo::MovieNfoCrawler};
use anyhow::Ok;
use runner::{RunnerManger, TaskStatus};
use tokio::sync::mpsc;

pub fn initial(config: &AppConfig, file_rx: mpsc::Receiver<PathBuf>) -> anyhow::Result<()> {
    RunnerManger::new(
        config.thread_limit,
        true,
        |path: PathBuf| async move {
            MovieNfoCrawler::crawler(&path).await?;
            Ok(())
        },
        |task_id: String, status: TaskStatus| {},
    );
    Ok(())
}

impl MovieNfoCrawler {
    async fn crawler(path: &PathBuf) -> anyhow::Result<Self> {
        Ok(Self::default())
    }
}

struct Test(String);

impl TryFrom<String> for Test {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Test(value))
    }
}
