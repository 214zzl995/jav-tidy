use std::path::PathBuf;

mod notify;

use anyhow::Ok;
use notify::SourceNotify;
use tokio::sync::mpsc;

use crate::config::AppConfig;

pub async fn initial(
    config: &AppConfig,
    return_tx: mpsc::Sender<PathBuf>,
) -> anyhow::Result<SourceNotify> {
    let migrate_files_ext = config.get_migrate_files_ext();

    let return_tx_notify = return_tx.clone();

    let source_notify = SourceNotify::new(
        &[config.input_dir.clone()],
        return_tx_notify,
        migrate_files_ext,
    )?;

    let input_dir = config.input_dir.clone();
    tokio::spawn(full_scan(input_dir, return_tx, migrate_files_ext));

    Ok(source_notify)
}

async fn full_scan(
    source: PathBuf,
    return_tx: mpsc::Sender<PathBuf>,
    migrate_files_ext: &'static [&'static str],
) -> anyhow::Result<()> {
    for entry in walkdir::WalkDir::new(source) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let path = entry.path();
            #[cfg(target_os = "windows")]
            if is_recycle_bin(path) {
                continue;
            }
            if is_migrate_files(
                migrate_files_ext,
                path.extension().unwrap().to_str().unwrap(),
            ) {
                return_tx.send(path.to_owned()).await?;
            }
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
pub(in crate::file) fn is_recycle_bin(path: &Path) -> bool {
    path.components()
        .nth(2)
        .is_some_and(|com| com.as_os_str().to_str().unwrap_or("").eq("$RECYCLE.BIN"))
}

pub(in crate::file) fn is_migrate_files(
    migrate_files_ext: &'static [&'static str],
    ext: &str,
) -> bool {
    migrate_files_ext.iter().any(|ext1| ext == *ext1)
}
