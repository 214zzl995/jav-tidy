use std::path::{Path, PathBuf};

use config::Config;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct AppConfig {
    pub migrate_files: Vec<String>,
    migrate_subtitles: bool,
    ignored_id_pattern: Vec<String>,
    capital: bool,
    pub input_dir: PathBuf,
    output_dir: PathBuf,
    pub thread_limit: usize,
    pub template_priority: Vec<String>,
    #[serde(default = "default_maximum_fetch_count")]
    pub maximum_fetch_count: usize,
}

fn default_maximum_fetch_count() -> usize {
    3
}

impl AppConfig {
    pub(crate) fn new(config_file: &Path) -> anyhow::Result<Self> {
        let settings = Config::builder()
            .add_source(config::File::from(config_file))
            .add_source(config::Environment::with_prefix("JAVTIDY"))
            .build()
            .unwrap();

        let config: AppConfig = settings.try_deserialize()?;

        Ok(config)
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

    pub fn is_useing_template(&self, template: &str) -> bool {
        self.template_priority
            .iter()
            .any(|t| t == template)
    }

    pub fn get_template_index(&self, template: &str) -> Option<usize> {
        self.template_priority
            .iter()
            .position(|t| t == template)
    }

    pub fn clean_file_name(&self, file_name: &str) -> String {
        let mut name = file_name.to_string();
        if self.capital {
            name = name.to_lowercase();
        }
        for pattern in &self.ignored_id_pattern {
            name = name.replace(pattern, "");
        }
        name.to_uppercase()
    }
}
