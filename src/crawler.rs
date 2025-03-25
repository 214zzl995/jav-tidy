use std::path::PathBuf;

use anyhow::Ok;
use tokio::sync::mpsc;

use crate::config::AppConfig;



pub fn initial(config: &AppConfig, file_rx: mpsc::Receiver<PathBuf>) -> anyhow::Result<()> {

    Ok(())
}
