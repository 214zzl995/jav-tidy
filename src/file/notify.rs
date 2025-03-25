use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use notify::{Config, Error, Event, RecommendedWatcher, Watcher};
use parking_lot::Mutex;
use tokio::sync::mpsc;

use super::is_recycle_bin;

#[derive(Clone)]
pub struct SourceNotify(Arc<Mutex<SourceNotifyInner>>);

struct SourceNotifyInner {
    watcher: RecommendedWatcher,
    sources: Vec<PathBuf>,
}

impl SourceNotify {
    pub fn new(
        sources: &[PathBuf],
        return_tx: mpsc::Sender<PathBuf>,
        migrate_files_ext: &'static [&'static str],
    ) -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::unbounded_channel();
        let watcher = RecommendedWatcher::new(
            move |result: std::result::Result<Event, Error>| {
                tx.send(result).unwrap();
            },
            Config::default(),
        )?;

        let source_notify = SourceNotify(Arc::new(Mutex::new(SourceNotifyInner {
            watcher,
            sources: sources.to_owned(),
        })));

        source_notify.listen(return_tx, rx, migrate_files_ext)?;

        Ok(source_notify)
    }

    fn listen(
        &self,
        return_tx: mpsc::Sender<PathBuf>,
        mut rx: mpsc::UnboundedReceiver<Result<Event, Error>>,
        migrate_files_ext: &'static [&'static str],
    ) -> anyhow::Result<()> {
        tokio::spawn(async move {
            while let Some(Ok(event)) = rx.recv().await {
                let return_tx = return_tx.clone();
                tokio::spawn(async move {
                    match event.kind {
                        notify::EventKind::Create(_) => {
                            let path = event.paths.first().unwrap().clone();
                            if !path.is_file() {
                                return;
                            }
                            #[cfg(target_os = "windows")]
                            if is_recycle_bin(&path) {
                                return;
                            }
                            if super::is_migrate_files(
                                migrate_files_ext,
                                path.extension().unwrap().to_str().unwrap(),
                            ) {
                                return_tx.send(path.clone()).await.unwrap();
                            }
                        }
                        notify::EventKind::Remove(_) => {}
                        notify::EventKind::Modify(_) => {}
                        _ => {}
                    }
                });
            }
        });

        let mut source_notify = self.0.lock();

        let sources = source_notify.sources.clone();

        for source in sources {
            source_notify
                .watcher
                .watch(&source, ::notify::RecursiveMode::Recursive)?;
        }

        Ok(())
    }

    pub(super) fn _watch_source(&self, source: &Path) -> anyhow::Result<()> {
        let mut source_notify = self.0.lock();
        source_notify.sources.push(PathBuf::from(source));
        source_notify
            .watcher
            .watch(&source, ::notify::RecursiveMode::Recursive)?;

        Ok(())
    }

    pub(super) fn _unwatch_source(&self, source: &Path) -> anyhow::Result<()> {
        self.0.lock().sources.retain(|s| !s.eq(source));
        self.0.lock().watcher.unwatch(source)?;
        Ok(())
    }
}

impl SourceNotifyInner {}
