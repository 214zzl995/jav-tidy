use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use notify::{Config, Error, Event, EventKind, RecommendedWatcher, Watcher};
use tokio::sync::{mpsc, RwLock};

#[cfg(target_os = "windows")]
use super::is_recycle_bin;

/// 高性能文件监控器
///
/// 特性：
/// - 批量处理文件事件，减少任务创建开销
/// - 智能去重，避免重复处理相同文件
/// - 延迟处理，减少频繁的小文件操作
/// - 专注单一目录监控，简化架构
#[derive(Clone)]
pub struct SourceNotify {
    inner: Arc<SourceNotifyInner>,
}

struct SourceNotifyInner {
    watcher: RwLock<RecommendedWatcher>,
    allowed_extensions: HashSet<String>,
}

/// 事件处理器配置
struct EventHandlerConfig {
    /// 批处理大小 - 每批最多处理的文件数量
    batch_size: usize,
    /// 批处理延迟 - 等待更多事件的时间（毫秒）
    batch_delay_ms: u64,
    /// 去重窗口大小 - 防止短时间内重复处理同一文件
    dedup_window: usize,
}

impl Default for EventHandlerConfig {
    fn default() -> Self {
        Self {
            batch_size: 50,
            batch_delay_ms: 100,
            dedup_window: 1000,
        }
    }
}

impl SourceNotify {
    /// 创建新的文件监控器
    ///
    /// # 参数
    /// - `sources`: 要监控的目录列表
    /// - `return_tx`: 文件路径发送通道
    /// - `migrate_files_ext`: 允许的文件扩展名列表
    ///
    /// # 返回
    /// 返回监控器实例或错误
    pub fn new(
        sources: &[PathBuf],
        return_tx: mpsc::Sender<PathBuf>,
        migrate_files_ext: &'static [&'static str],
    ) -> anyhow::Result<Self> {
        // 创建事件通道
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        // 创建观察器
        log::info!("创建文件系统监控器...");
        let watcher = RecommendedWatcher::new(
            move |result: Result<Event, Error>| {
                match &result {
                    Ok(event) => {
                        log::debug!("接收到文件系统事件: {:?}", event);
                    }
                    Err(e) => {
                        log::error!("文件系统事件错误: {}", e);
                    }
                }
                if let Err(e) = event_tx.send(result) {
                    log::warn!("发送文件事件失败: {}", e);
                }
            },
            Config::default(),
        )?;

        // 预处理扩展名为HashSet，提高查找效率
        let allowed_extensions: HashSet<String> = migrate_files_ext
            .iter()
            .map(|ext| ext.to_lowercase())
            .collect();
        
        log::info!("配置监控文件扩展名: {:?}", allowed_extensions);

        let source_notify = SourceNotify {
            inner: Arc::new(SourceNotifyInner {
                watcher: RwLock::new(watcher),
                allowed_extensions,
            }),
        };

        // 启动事件处理器
        source_notify.start_event_handler(return_tx, event_rx)?;

        // 在后台任务中开始监控目录
        let inner_clone = Arc::clone(&source_notify.inner);
        let sources_to_watch = sources.to_vec();
        tokio::spawn(async move {
            if let Err(e) = Self::initialize_watching(&inner_clone, &sources_to_watch).await {
                log::error!("初始化文件监控失败: {}", e);
            }
        });

        Ok(source_notify)
    }

    /// 初始化监控（内部异步方法）
    async fn initialize_watching(
        inner: &Arc<SourceNotifyInner>,
        sources: &[PathBuf],
    ) -> anyhow::Result<()> {
        let mut watcher = inner.watcher.write().await;

        for source in sources {
            if let Err(e) = watcher.watch(source, notify::RecursiveMode::Recursive) {
                log::error!("无法监控目录 {}: {}", source.display(), e);
                return Err(anyhow::anyhow!("监控目录失败: {}", e));
            } else {
                log::info!("开始监控目录: {}", source.display());
            }
        }

        Ok(())
    }

    /// 启动事件处理器
    fn start_event_handler(
        &self,
        return_tx: mpsc::Sender<PathBuf>,
        mut event_rx: mpsc::UnboundedReceiver<Result<Event, Error>>,
    ) -> anyhow::Result<()> {
        let inner = Arc::clone(&self.inner);
        let config = EventHandlerConfig::default();

        log::info!("启动文件事件处理器，批处理配置: 大小={}, 延迟={}ms", 
                  config.batch_size, config.batch_delay_ms);

        tokio::spawn(async move {
            let mut pending_files = Vec::with_capacity(config.batch_size);
            let mut recent_files = std::collections::VecDeque::with_capacity(config.dedup_window);

            loop {
                // 收集一批事件或等待超时
                let batch_complete = Self::collect_event_batch(
                    &mut event_rx,
                    &mut pending_files,
                    &mut recent_files,
                    &inner,
                    &config,
                )
                .await;

                // 处理收集到的文件
                if !pending_files.is_empty() {
                    log::debug!("处理文件批次，包含 {} 个文件", pending_files.len());
                    Self::process_file_batch(&return_tx, &mut pending_files).await;
                }

                // 如果通道已关闭且没有更多事件，退出循环
                if !batch_complete && pending_files.is_empty() {
                    break;
                }
            }

            log::info!("文件监控事件处理器已停止");
        });

        Ok(())
    }

    /// 收集一批事件
    async fn collect_event_batch(
        event_rx: &mut mpsc::UnboundedReceiver<Result<Event, Error>>,
        pending_files: &mut Vec<PathBuf>,
        recent_files: &mut std::collections::VecDeque<PathBuf>,
        inner: &Arc<SourceNotifyInner>,
        config: &EventHandlerConfig,
    ) -> bool {
        let start_time = std::time::Instant::now();

        while pending_files.len() < config.batch_size {
            let timeout = Duration::from_millis(config.batch_delay_ms);
            let remaining_time = timeout.saturating_sub(start_time.elapsed());

            match tokio::time::timeout(remaining_time, event_rx.recv()).await {
                Ok(Some(Ok(event))) => {
                    log::debug!("处理文件系统事件: kind={:?}, paths={:?}", event.kind, event.paths);
                    Self::process_single_event(event, pending_files, recent_files, inner, config);
                }
                Ok(Some(Err(e))) => {
                    log::warn!("文件监控事件错误: {}", e);
                }
                Ok(None) => {
                    // 通道已关闭
                    return false;
                }
                Err(_) => {
                    // 超时 - 批处理完成
                    break;
                }
            }
        }

        true
    }

    /// 处理单个文件事件
    fn process_single_event(
        event: Event,
        pending_files: &mut Vec<PathBuf>,
        recent_files: &mut std::collections::VecDeque<PathBuf>,
        inner: &Arc<SourceNotifyInner>,
        config: &EventHandlerConfig,
    ) {
        // 只处理文件创建事件
        if !matches!(event.kind, EventKind::Create(_)) {
            log::debug!("忽略非创建事件: {:?}", event.kind);
            return;
        }

        for path in event.paths {
            log::debug!("检查文件: {}", path.display());
            
            // 基本过滤
            if !path.is_file() {
                log::debug!("跳过非文件: {}", path.display());
                continue;
            }

            #[cfg(target_os = "windows")]
            if is_recycle_bin(&path) {
                log::debug!("跳过回收站文件: {}", path.display());
                continue;
            }

            // 检查扩展名
            if !Self::is_allowed_file(&path, &inner.allowed_extensions) {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    log::debug!("跳过不支持的文件扩展名 '{}': {}", ext, path.display());
                } else {
                    log::debug!("跳过无扩展名文件: {}", path.display());
                }
                continue;
            }

            // 去重检查
            if recent_files.contains(&path) {
                log::debug!("跳过重复文件: {}", path.display());
                continue;
            }

            // 添加到待处理列表
            log::info!("发现新的待处理文件: {}", path.display());
            pending_files.push(path.clone());

            // 维护去重窗口
            recent_files.push_back(path);
            if recent_files.len() > config.dedup_window {
                recent_files.pop_front();
            }
        }
    }

    /// 检查文件是否为允许的类型
    fn is_allowed_file(path: &Path, allowed_extensions: &HashSet<String>) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| allowed_extensions.contains(&ext.to_lowercase()))
            .unwrap_or(false)
    }

    /// 批量处理文件
    async fn process_file_batch(
        return_tx: &mpsc::Sender<PathBuf>,
        pending_files: &mut Vec<PathBuf>,
    ) {
        for file_path in pending_files.drain(..) {
            if let Err(e) = return_tx.send(file_path.clone()).await {
                log::error!("发送文件路径失败 {}: {}", file_path.display(), e);
                // 如果发送失败，说明接收方已关闭，应该停止处理
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_is_allowed_file() {
        let mut allowed = HashSet::new();
        allowed.insert("mp4".to_string());
        allowed.insert("mkv".to_string());

        assert!(SourceNotify::is_allowed_file(
            Path::new("test.mp4"),
            &allowed
        ));
        assert!(SourceNotify::is_allowed_file(
            Path::new("test.MP4"),
            &allowed
        ));
        assert!(!SourceNotify::is_allowed_file(
            Path::new("test.txt"),
            &allowed
        ));
        assert!(!SourceNotify::is_allowed_file(Path::new("test"), &allowed));
    }
}
