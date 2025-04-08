use std::path::{Path, PathBuf};

use crate::{config::AppConfig, nfo::MovieNfoCrawler};
use anyhow::{Context, Ok};
use crawler_template::Template;
use tokio::sync::mpsc;

pub fn initial(template_path:&Path, config: &AppConfig, file_rx: mpsc::Receiver<PathBuf>) -> anyhow::Result<()> {

    let templates = get_templates(template_path, config)
        .with_context(|| format!("get template from {}", template_path.display()))?;

    

    
    Ok(())
}

impl MovieNfoCrawler {
    async fn crawler(path: &PathBuf) -> anyhow::Result<Self> {
        Ok(Self::default())
    }
}

fn get_templates(
    path: &Path,
    config: &AppConfig,
) -> anyhow::Result<Vec<Template<MovieNfoCrawler>>> {
    let mut templates = vec![None; config.template_priority.len()];

    for entry in path.read_dir()? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name = file_name.to_str().unwrap();

        if entry.path().is_file()
            && entry.path().extension() == Some("yaml".as_ref())
            && config.is_useing_template(file_name)
        {
            let yaml = std::fs::read_to_string(entry.path())?;
            let index = config.get_template_index(file_name).unwrap();
            let template = Template::from_yaml(&yaml)?;

            templates[index] = Some(template);
        }
    }

    for (i, template) in templates.iter().enumerate() {
        if template.is_none() {
            return Err(anyhow::anyhow!(
                "template {} not found",
                config.template_priority[i]
            ));
        }
    }

    Ok(templates
        .into_iter()
        .map(|t| t.unwrap())
        .collect::<Vec<_>>())
}
