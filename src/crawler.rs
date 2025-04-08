use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

use crate::{
    config::AppConfig,
    nfo::{MovieNfo, MovieNfoCrawler},
};
use anyhow::Context;
use crawler_template::Template;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::sync::mpsc;

type Templates = Arc<Vec<(String, Template<MovieNfoCrawler>)>>;

pub fn initial(
    template_path: &Path,
    config: &AppConfig,
    file_rx: mpsc::Receiver<PathBuf>,
    multi_progress: MultiProgress,
) -> anyhow::Result<()> {
    let templates = Arc::new(
        get_templates(template_path, config)
            .with_context(|| format!("get template from {}", template_path.display()))?,
    );

    Ok(())
}

async fn crawler(
    crawler_name: &str,
    process: &ProgressBar,
    templates: Templates,
    app_config: &Arc<AppConfig>,
) -> anyhow::Result<MovieNfo> {
    let mut succecc_nfo = vec![];

    for (template_name, template) in templates.iter() {
        process.set_message(format!("正在使用: {} 模版爬取数据", template_name));
        let mut init_params = HashMap::new();
        init_params.insert("crawl_name", crawler_name.to_string());

        match template.crawler(&init_params).await {
            Ok(movie_nfo) => {
                succecc_nfo.push(movie_nfo);
                if succecc_nfo.len() >= app_config.maximum_fetch_count {
                    break;
                }
            }
            Err(e) => {
                log::error!("template:{} crawler error: {}", template_name, e);
                process.set_message(format!("{} 模版爬取数据失败", template_name));
                continue;
            }
        };
    }

    if succecc_nfo.is_empty() {
        return Err(anyhow::anyhow!("所有模版爬取失败"));
    }

    let nfo = MovieNfo::from(clean_crawler_nfos(succecc_nfo).await?);

    Ok(nfo)
}

async fn clean_crawler_nfos(nfos: Vec<MovieNfoCrawler>) -> anyhow::Result<MovieNfoCrawler> {
    Ok(nfos[0].clone())
}

fn get_templates(
    path: &Path,
    config: &AppConfig,
) -> anyhow::Result<Vec<(String, Template<MovieNfoCrawler>)>> {
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

            templates[index] = Some((file_name.to_string(), template));
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

static PROGRESS_STYLE: LazyLock<ProgressStyle> = LazyLock::new(|| {
    ProgressStyle::with_template("{prefix} :{spinner:.blue} {msg}")
        .unwrap()
        .tick_strings(&[
            "▹▹▹▹▹",
            "▸▹▹▹▹",
            "▹▸▹▹▹",
            "▹▹▸▹▹",
            "▹▹▹▸▹",
            "▹▹▹▹▸",
            "▪▪▪▪▪",
        ])
});

fn get_progress_bar(multi_progress: &MultiProgress, msg: &str) -> ProgressBar {
    let progress_bar = multi_progress.add(ProgressBar::new(0));
    progress_bar.set_style(PROGRESS_STYLE.clone());
    progress_bar.set_message(msg.to_string());
    progress_bar
}
