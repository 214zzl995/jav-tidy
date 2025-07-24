use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

use crate::{
    config::AppConfig,
    nfo::{MovieNfo, MovieNfoCrawler},
    parser::FileNameParser,
    nfo_generator::NfoGenerator,
    file_organizer::FileOrganizer,
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

    let config = Arc::new(config.clone());
    
    // 启动文件处理任务
    tokio::spawn(process_file_queue(
        file_rx,
        templates,
        config,
        multi_progress,
    ));

    Ok(())
}

/// 文件处理队列的主循环
async fn process_file_queue(
    mut file_rx: mpsc::Receiver<PathBuf>,
    templates: Templates,
    config: Arc<AppConfig>,
    multi_progress: MultiProgress,
) {
    log::info!("文件处理队列已启动");
    
    // 创建工具实例
    let parser = match FileNameParser::new() {
        Ok(p) => p,
        Err(e) => {
            log::error!("创建文件名解析器失败: {}", e);
            return;
        }
    };
    
    let nfo_generator = NfoGenerator::new();
    let file_organizer = FileOrganizer::new();
    
    // 处理文件队列
    while let Some(file_path) = file_rx.recv().await {
        log::info!("接收到新文件: {}", file_path.display());
        
        // 创建进度条
        let progress_bar = get_progress_bar(&multi_progress, &format!("处理文件: {}", file_path.file_name().unwrap_or_default().to_str().unwrap_or("未知")));
        
        // 处理单个文件
        if let Err(e) = process_single_file(
            &file_path,
            &parser,
            &nfo_generator,
            &file_organizer,
            &templates,
            &config,
            &progress_bar,
        ).await {
            log::error!("处理文件 {} 失败: {}", file_path.display(), e);
            progress_bar.finish_with_message("处理失败");
        } else {
            progress_bar.finish_with_message("处理完成");
        }
        
        // 移除进度条
        multi_progress.remove(&progress_bar);
    }
    
    log::info!("文件处理队列已停止");
}

/// 处理单个文件
async fn process_single_file(
    file_path: &Path,
    parser: &FileNameParser,
    nfo_generator: &NfoGenerator,
    file_organizer: &FileOrganizer,
    templates: &Templates,
    config: &AppConfig,
    progress_bar: &ProgressBar,
) -> anyhow::Result<()> {
    progress_bar.set_message("解析文件名...");
    
    // 1. 从文件名提取影片ID
    let movie_id = parser
        .extract_movie_id(file_path, config)
        .ok_or_else(|| anyhow::anyhow!("无法从文件名提取影片ID"))?;
    
    log::info!("提取到影片ID: {}", movie_id);
    progress_bar.set_message(format!("搜索影片信息: {}", movie_id));
    
    // 2. 使用爬虫获取影片信息
    let movie_nfo = crawler(&movie_id, progress_bar, templates.clone(), &Arc::new(config.clone())).await?;
    
    progress_bar.set_message("验证NFO数据...");
    
    // 3. 验证NFO数据
    let warnings = nfo_generator.validate_nfo(&movie_nfo);
    if !warnings.is_empty() {
        log::warn!("NFO数据验证警告: {:?}", warnings);
    }
    
    progress_bar.set_message("生成NFO文件...");
    
    // 4. 生成并保存NFO文件
    let nfo_path = nfo_generator.generate_and_save(&movie_nfo, file_path, config)?;
    
    progress_bar.set_message("整理文件...");
    
    // 5. 整理文件（移动到输出目录并重命名）
    if file_organizer.needs_organization(file_path, config) {
        let new_file_path = file_organizer.organize_file(file_path, &movie_nfo, config)?;
        log::info!("影片 {} 处理完成，文件移动至: {}，NFO文件: {}", 
                   movie_id, new_file_path.display(), nfo_path.display());
    } else {
        log::info!("影片 {} 处理完成，文件无需移动，NFO文件: {}", 
                   movie_id, nfo_path.display());
    }
    
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
