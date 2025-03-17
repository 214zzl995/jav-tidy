use std::path::PathBuf;

use config::Config;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct AppConfig {
    migrate_files: Vec<String>,
    migrate_subtitles: bool,
    ignored_id_pattern: Vec<String>,
    capital: bool,
    input_dir: PathBuf,
    output_dir: PathBuf,
}

impl AppConfig {
    pub(crate) fn new(config_file: PathBuf) -> anyhow::Result<AppConfig> {
        let settings = Config::builder()
            .add_source(config::File::from(config_file))
            .add_source(config::Environment::with_prefix("JAVTIDY"))
            .build()
            .unwrap();

        Ok(settings.try_deserialize::<AppConfig>()?)
    }
}


