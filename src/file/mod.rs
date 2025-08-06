#[cfg(target_os = "windows")]
use std::path::Path;
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
    log::info!("初始化文件监控系统...");
    let migrate_files_ext = config.get_migrate_files_ext();
    log::debug!("支持的文件扩展名: {:?}", migrate_files_ext);

    let return_tx_notify = return_tx.clone();

    log::info!("创建文件监控器，监控输入目录: {}", config.input_dir.display());
    let source_notify = SourceNotify::new(
        &[config.input_dir.clone()],
        return_tx_notify,
        migrate_files_ext,
    )?;

    let input_dir = config.input_dir.clone();
    log::info!("启动初始全目录扫描任务: {}", input_dir.display());
    tokio::spawn(full_scan(input_dir, return_tx, migrate_files_ext));

    log::info!("文件监控系统初始化完成");
    Ok(source_notify)
}

async fn full_scan(
    source: PathBuf,
    return_tx: mpsc::Sender<PathBuf>,
    migrate_files_ext: &'static [&'static str],
) -> anyhow::Result<()> {
    log::info!("开始全目录扫描: {}", source.display());
    let mut file_count = 0;
    let mut matched_count = 0;
    
    for entry in walkdir::WalkDir::new(&source) {
        let entry = entry?;
        if entry.file_type().is_file() {
            file_count += 1;
            let path = entry.path();
            log::debug!("扫描文件: {}", path.display());
            
            #[cfg(target_os = "windows")]
            if is_recycle_bin(path) {
                log::debug!("跳过回收站文件: {}", path.display());
                continue;
            }
            
            if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                if is_migrate_files(migrate_files_ext, extension) {
                    matched_count += 1;
                    log::info!("发现匹配文件: {}", path.display());
                    return_tx.send(path.to_owned()).await?;
                } else {
                    log::debug!("跳过不匹配扩展名 '{}' 的文件: {}", extension, path.display());
                }
            } else {
                log::debug!("跳过无扩展名文件: {}", path.display());
            }
        }
    }
    
    log::info!("全目录扫描完成: 总文件数={}, 匹配文件数={}", file_count, matched_count);
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
    let matches = migrate_files_ext.contains(&ext);
    log::debug!("扩展名匹配检查: '{}' 在 {:?} 中 = {}", ext, migrate_files_ext, matches);
    matches
}
