use std::{path::PathBuf, sync::OnceLock};

use config::Config;
use serde::Deserialize;

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct AppConfig {
    pub migrate_files: Vec<String>,
    migrate_subtitles: bool,
    ignored_id_pattern: Vec<String>,
    capital: bool,
    pub input_dir: PathBuf,
    output_dir: PathBuf,
}

impl AppConfig {
    pub(crate) fn initial(config_file: PathBuf) -> anyhow::Result<()> {
        let settings = Config::builder()
            .add_source(config::File::from(config_file))
            .add_source(config::Environment::with_prefix("JAVTIDY"))
            .build()
            .unwrap();

        let config: AppConfig = settings.try_deserialize()?;

        CONFIG
            .set(config)
            .map_err(|_| anyhow::anyhow!("Failed to set config"))?;

        Ok(())
    }

    pub fn get_migrate_files_ext(&self) -> &'static [&'static str] {
        let leaked_strs: Vec<&'static str> = self
            .migrate_files
            .clone()
            .into_iter()
            .map(|s| Box::leak(s.into_boxed_str()) as &'static str)
            .collect();

        Box::leak(leaked_strs.into_boxed_slice())
    }
}

pub fn get_config() -> &'static AppConfig {
    CONFIG.get().expect("Config not initialized")
}
