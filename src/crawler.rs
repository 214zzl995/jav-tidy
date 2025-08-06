use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
    time::SystemTime,
};

use crate::{
    config::AppConfig,
    error::AppError,
    file_organizer::FileOrganizer,
    image_manager::ImageManager,
    nfo::{MediaCenterType, MovieNfo, MovieNfoCrawler, NfoFormatter},
    nfo_generator::NfoGenerator,
    parser::FileNameParser,
    translator::Translator,
};
use anyhow::Context;
use crawler_template::Template;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::sync::mpsc;

type Templates = Arc<Vec<(String, Template<MovieNfoCrawler>)>>;

/// 文件处理锁，防止文件在处理过程中被其他进程操作
pub struct FileProcessingLock {
    lock_path: PathBuf,
    _lock_file: File,
}

impl FileProcessingLock {
    pub fn acquire(file_path: &Path) -> anyhow::Result<Self> {
        let lock_path = file_path.with_extension("javtidy.lock");

        // 检查锁文件是否已存在
        if lock_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&lock_path) {
                if let Some(created_time) = content
                    .lines()
                    .nth(1)
                    .and_then(|line| line.parse::<u64>().ok())
                    .map(|timestamp| {
                        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp)
                    })
                {
                    // 如果锁文件超过5分钟，认为是僵尸锁
                    if created_time.elapsed().unwrap_or_default().as_secs() > 300 {
                        log::warn!("清理僵尸锁文件: {}", lock_path.display());
                        let _ = std::fs::remove_file(&lock_path);
                    } else {
                        return Err(anyhow::anyhow!(
                            "文件正在被其他进程处理: {}",
                            file_path.display()
                        ));
                    }
                }
            }
        }

        let lock_content = format!(
            "{}\n{}\n{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            file_path.display()
        );

        std::fs::write(&lock_path, lock_content)?;
        let lock_file = File::open(&lock_path)?;

        log::debug!("获取文件处理锁: {}", file_path.display());

        Ok(FileProcessingLock {
            lock_path,
            _lock_file: lock_file,
        })
    }
}

impl Drop for FileProcessingLock {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.lock_path) {
            log::warn!("释放文件锁失败: {}", e);
        } else {
            log::debug!("释放文件处理锁: {}", self.lock_path.display());
        }
    }
}

/// 文件完整性检查器
pub struct FileIntegrityChecker {
    path: PathBuf,
    initial_size: u64,
    initial_modified: SystemTime,
}

impl FileIntegrityChecker {
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        let metadata = std::fs::metadata(path)?;
        let initial_size = metadata.len();
        let initial_modified = metadata.modified()?;

        Ok(FileIntegrityChecker {
            path: path.to_path_buf(),
            initial_size,
            initial_modified,
        })
    }

    pub fn verify_integrity(&self) -> anyhow::Result<bool> {
        if !self.path.exists() {
            log::warn!("文件已不存在: {}", self.path.display());
            return Ok(false);
        }

        let metadata = std::fs::metadata(&self.path)?;
        let current_size = metadata.len();
        let current_modified = metadata.modified()?;

        if current_size != self.initial_size {
            log::warn!(
                "文件大小发生变化: {} ({} -> {})",
                self.path.display(),
                self.initial_size,
                current_size
            );
            return Ok(false);
        }

        if current_modified != self.initial_modified {
            log::warn!("文件修改时间发生变化: {}", self.path.display());
            return Ok(false);
        }

        Ok(true)
    }
}

/// 文件处理事务，确保操作的原子性
pub struct FileProcessingTransaction {
    original_path: PathBuf,
    operations: Vec<TransactionOperation>,
    completed: bool,
}

#[derive(Debug, Clone)]
enum TransactionOperation {
    CreateNfo { path: PathBuf, content: String },
    MoveFile { from: PathBuf, to: PathBuf },
    #[allow(dead_code)] // 预留给未来的目录创建功能
    CreateDirectory { path: PathBuf },
}

impl FileProcessingTransaction {
    pub fn new(original_path: &Path) -> Self {
        FileProcessingTransaction {
            original_path: original_path.to_path_buf(),
            operations: Vec::new(),
            completed: false,
        }
    }

    pub fn add_nfo_creation(&mut self, path: PathBuf, content: String) {
        self.operations
            .push(TransactionOperation::CreateNfo { path, content });
    }

    pub fn add_file_move(&mut self, from: PathBuf, to: PathBuf) {
        self.operations
            .push(TransactionOperation::MoveFile { from, to });
    }

    #[allow(dead_code)] // 预留给未来的目录创建功能
    pub fn add_directory_creation(&mut self, path: PathBuf) {
        self.operations
            .push(TransactionOperation::CreateDirectory { path });
    }

    pub fn commit(mut self) -> anyhow::Result<()> {
        log::info!("开始提交文件处理事务: {}", self.original_path.display());

        for (i, operation) in self.operations.iter().enumerate() {
            match operation {
                TransactionOperation::CreateNfo { path, content } => {
                    log::debug!("创建NFO文件: {}", path.display());
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(path, content)?;
                }
                TransactionOperation::MoveFile { from, to } => {
                    log::debug!("移动文件: {} -> {}", from.display(), to.display());
                    if let Some(parent) = to.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::rename(from, to)?;
                }
                TransactionOperation::CreateDirectory { path } => {
                    log::debug!("创建目录: {}", path.display());
                    std::fs::create_dir_all(path)?;
                }
            }
            log::debug!("完成事务操作 {}/{}", i + 1, self.operations.len());
        }

        self.completed = true;
        log::info!("文件处理事务提交成功: {}", self.original_path.display());
        Ok(())
    }
}

impl Drop for FileProcessingTransaction {
    fn drop(&mut self) {
        if !self.completed {
            log::warn!(
                "文件处理事务未完成，可能需要手动清理: {}",
                self.original_path.display()
            );
        }
    }
}

pub fn initial(
    template_path: &Path,
    config: &AppConfig,
    file_rx: mpsc::Receiver<PathBuf>,
    multi_progress: MultiProgress,
) -> anyhow::Result<()> {
    log::info!("初始化爬虫系统...");
    log::info!("模板目录: {}", template_path.display());
    
    let templates = Arc::new(
        get_templates(template_path, config)
            .with_context(|| format!("get template from {}", template_path.display()))?,
    );
    
    log::info!("成功加载 {} 个模板", templates.len());

    let config = Arc::new(config.clone());

    // 启动文件处理任务
    log::info!("启动文件处理队列任务...");
    tokio::spawn(process_file_queue(
        file_rx,
        templates,
        config,
        multi_progress,
    ));

    log::info!("爬虫系统初始化完成");
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

    // 创建通用 NFO 生成器
    let nfo_generator = NfoGenerator::for_media_center(MediaCenterType::Universal);
    let file_organizer = FileOrganizer::new();
    let image_manager = ImageManager::new();
    
    // 创建翻译器（如果启用）
    let translator = if config.is_translation_enabled() {
        match Translator::from_app_config(&config) {
            Ok(translator) => {
                log::info!("翻译器初始化成功，提供商: {}", config.get_translation_provider());
                
                // 测试连接
                match translator.test_connection().await {
                    Ok(_) => {
                        log::info!("翻译服务连接测试成功");
                    }
                    Err(e) => {
                        log::warn!("翻译服务连接测试失败: {}，翻译功能可能无法正常工作", e);
                    }
                }
                
                Some(translator)
            }
            Err(e) => {
                log::warn!("翻译器初始化失败: {}，将跳过翻译功能", e);
                None
            }
        }
    } else {
        log::info!("翻译功能已禁用");
        None
    };

    // 处理文件队列
    while let Some(file_path) = file_rx.recv().await {
        log::info!("接收到新文件: {}", file_path.display());

        // 创建进度条
        let progress_bar = get_progress_bar(
            &multi_progress,
            &format!(
                "处理文件: {}",
                file_path
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or("未知")
            ),
        );

        // 处理单个文件
        match process_single_file(
            &file_path,
            &parser,
            &nfo_generator,
            &file_organizer,
            &image_manager,
            translator.as_ref(),
            &templates,
            &config,
            &progress_bar,
        )
        .await
        {
            Ok(_) => {
                progress_bar.finish_with_message("处理完成");
            }
            Err(e) => {
                if let Some(app_error) = e.downcast_ref::<AppError>() {
                    if app_error.should_skip_processing() {
                        let reason = app_error.skip_reason().unwrap_or("未知原因");
                        log::info!("跳过文件 {}: {}", file_path.display(), reason);
                        progress_bar.finish_with_message("已跳过");
                    } else {
                        log::error!("处理文件 {} 失败: {}", file_path.display(), e);
                        progress_bar.finish_with_message("处理失败");
                    }
                } else {
                    log::error!("处理文件 {} 失败: {}", file_path.display(), e);
                    progress_bar.finish_with_message("处理失败");
                }
            }
        }

        multi_progress.remove(&progress_bar);
    }

    log::info!("文件处理队列已停止");
}

/// 处理单个文件（带文件保护机制）
async fn process_single_file(
    file_path: &Path,
    parser: &FileNameParser,
    nfo_generator: &NfoGenerator,
    file_organizer: &FileOrganizer,
    image_manager: &ImageManager,
    translator: Option<&Translator>,
    templates: &Templates,
    config: &AppConfig,
    progress_bar: &ProgressBar,
) -> anyhow::Result<()> {
    progress_bar.set_message("获取文件锁...");

    let _lock = FileProcessingLock::acquire(file_path)
        .with_context(|| format!("无法获取文件锁: {}", file_path.display()))?;

    let integrity_checker = FileIntegrityChecker::new(file_path)
        .with_context(|| format!("无法创建文件完整性检查器: {}", file_path.display()))?;

    if !file_path.exists() {
        return Err(anyhow::anyhow!("文件不存在: {}", file_path.display()));
    }

    log::info!("开始安全处理文件: {}", file_path.display());

    progress_bar.set_message("解析文件名...");

    let movie_id = parser
        .extract_movie_id(file_path, config)
        .ok_or_else(|| anyhow::anyhow!("无法从文件名提取影片ID"))?;

    log::info!("提取到影片ID: {}", movie_id);

    // 验证文件完整性（第一次检查）
    if !integrity_checker.verify_integrity()? {
        return Err(anyhow::anyhow!("文件在处理过程中被修改"));
    }

    progress_bar.set_message(format!("搜索影片信息: {}", movie_id));

    let crawler_data = match crawler(
        &movie_id,
        progress_bar,
        templates.clone(),
        &Arc::new(config.clone()),
    )
    .await
    {
        Ok(data) => {
            log::info!("影片 {} 数据爬取成功", movie_id);
            data
        }
        Err(e) => {
            log::warn!("影片 {} 数据爬取失败: {}，跳过处理此文件", movie_id, e);
            progress_bar.set_message("爬取失败，跳过处理");
            
            return Err(anyhow::Error::from(e));
        }
    };

    if !integrity_checker.verify_integrity()? {
        return Err(anyhow::anyhow!("文件在爬取过程中被修改"));
    }

    // 翻译影片数据（如果启用）
    let mut final_crawler_data = crawler_data.clone();
    if let Some(translator) = translator {
        progress_bar.set_message("翻译影片内容...");
        
        if let Err(e) = translator.translate_movie_data(&mut final_crawler_data, config).await {
            log::warn!("影片数据翻译失败: {}，继续使用原始数据", e);
            final_crawler_data = crawler_data.clone();
        } else {
            log::info!("影片数据翻译完成");
        }
    }

    let movie_nfo = MovieNfo::for_universal(final_crawler_data.clone());

    progress_bar.set_message("验证NFO数据...");

    let warnings = nfo_generator.validate_nfo(&movie_nfo);
    if !warnings.is_empty() {
        log::warn!("NFO数据验证警告: {:?}", warnings);
    }

    // 阶段4.5: 下载图片（如果启用）
    if config.should_download_images() {
        progress_bar.set_message("下载影片图片...");
        
        let output_dir = if file_organizer.needs_organization(file_path, config) {
            // 预览组织后的目录结构
            let (video_path, _) = file_organizer.preview_media_center_structure(file_path, &movie_nfo, config)?;
            video_path.parent().unwrap_or(config.get_output_dir()).to_path_buf()
        } else {
            file_path.parent().unwrap_or(config.get_output_dir()).to_path_buf()
        };

        match image_manager.download_movie_images(
            &final_crawler_data,
            &output_dir,
            &movie_id,
            config,
        ).await {
            Ok(downloaded_images) => {
                if !downloaded_images.is_empty() {
                    log::info!("成功下载 {} 个图片文件: {:?}", 
                        downloaded_images.len(),
                        downloaded_images.iter().map(|p| p.file_name().unwrap_or_default()).collect::<Vec<_>>()
                    );
                } else {
                    log::info!("没有可下载的图片或图片已存在");
                }
            },
            Err(e) => {
                log::warn!("图片下载失败: {}，继续处理文件", e);
            }
        }
    }

    // 阶段5: 创建处理事务
    progress_bar.set_message("准备文件操作...");

    let mut transaction = FileProcessingTransaction::new(file_path);

    let (final_video_path, final_nfo_path) = if file_organizer.needs_organization(file_path, config)
    {
        let (video_path, nfo_path) =
            file_organizer.preview_media_center_structure(file_path, &movie_nfo, config)?;

        transaction.add_file_move(file_path.to_path_buf(), video_path.clone());

        (video_path, nfo_path)
    } else {
        let nfo_path = file_path.with_extension("nfo");
        (file_path.to_path_buf(), nfo_path)
    };

    let nfo_xml_content = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<!-- Generated by jav-tidy-rs with media center compatibility -->\n{}",
        movie_nfo.format_to_xml()
    );
    transaction.add_nfo_creation(final_nfo_path.clone(), nfo_xml_content);

    if !integrity_checker.verify_integrity()? {
        return Err(anyhow::anyhow!("文件在准备操作时被修改"));
    }

    // 阶段6: 执行事务
    progress_bar.set_message("执行文件操作...");

    transaction
        .commit()
        .with_context(|| format!("文件处理事务失败: {}", file_path.display()))?;

    if config.migrate_subtitles() {
        progress_bar.set_message("处理字幕文件...");
        
        if let Some(input_dir) = file_path.parent() {
            match file_organizer.migrate_subtitle_files(
                &movie_id,
                input_dir,
                &final_video_path,
                config,
            ) {
                Ok(migrated_subtitles) => {
                    if !migrated_subtitles.is_empty() {
                        log::info!(
                            "成功迁移 {} 个字幕文件: {:?}",
                            migrated_subtitles.len(),
                            migrated_subtitles.iter().map(|p| p.file_name().unwrap_or_default()).collect::<Vec<_>>()
                        );
                    } else {
                        log::debug!("未找到匹配的字幕文件");
                    }
                }
                Err(e) => {
                    log::warn!("字幕文件迁移失败: {}", e);
                }
            }
        }
    }

    // 阶段8: 处理多演员链接策略
    if movie_nfo.actors.len() > 1 {
        progress_bar.set_message("处理多演员链接...");
        
        match file_organizer.handle_multi_actor_links(
            file_path,
            &movie_nfo,
            config,
            &final_video_path,
            &final_nfo_path,
        ) {
            Ok(additional_paths) => {
                if !additional_paths.is_empty() {
                    log::info!(
                        "成功创建 {} 个多演员链接: {:?}",
                        additional_paths.len(),
                        additional_paths
                    );
                }
            }
            Err(e) => {
                log::warn!("多演员链接处理失败: {}", e);
            }
        }
    }

    // 阶段9: 完成处理
    progress_bar.set_message("处理完成");

    log::info!(
        "影片 {} 处理完成 - 媒体中心结构已创建\n  原始文件: {}\n  视频文件: {}\n  NFO文件: {}",
        movie_id,
        file_path.display(),
        final_video_path.display(),
        final_nfo_path.display()
    );

    Ok(())
}

async fn crawler(
    crawler_name: &str,
    process: &ProgressBar,
    templates: Templates,
    app_config: &Arc<AppConfig>,
) -> Result<MovieNfoCrawler, AppError> {
    let mut succecc_nfo = vec![];
    log::info!("开始爬取影片数据: {}", crawler_name);

    for (template_name, template) in templates.iter() {
        log::info!("尝试使用模板 '{}' 爬取数据", template_name);
        process.set_message(format!("正在使用: {} 模版爬取数据", template_name));
        let mut init_params = HashMap::new();
        init_params.insert("crawl_name", crawler_name.to_string());

        match template.crawler(&init_params).await {
            Ok(movie_nfo) => {
                log::info!("模板 '{}' 爬取成功", template_name);
                log::debug!("爬取到的数据摘要: 标题='{}', 演员数={}, 导演数={}, 厂商数={}", 
                    movie_nfo.title, 
                    movie_nfo.actors.len(),
                    movie_nfo.directors.len(),
                    movie_nfo.studios.len()
                );
                
                // 检查数据质量
                let data_quality_score = calculate_data_quality(&movie_nfo);
                log::info!("数据质量评分: {}/100", data_quality_score);
                
                if data_quality_score < 20 {
                    log::warn!("模板 '{}' 返回的数据质量较差 (评分: {}), 数据可能不完整", 
                        template_name, data_quality_score);
                } else {
                    log::info!("模板 '{}' 返回的数据质量良好 (评分: {})", 
                        template_name, data_quality_score);
                }
                
                succecc_nfo.push(movie_nfo);
                if succecc_nfo.len() >= app_config.maximum_fetch_count {
                    log::info!("已达到最大爬取数量限制: {}", app_config.maximum_fetch_count);
                    break;
                }
            }
            Err(e) => {
                log::error!("模板 '{}' 爬取失败: {}", template_name, e);
                process.set_message(format!("{} 模版爬取数据失败", template_name));
                continue;
            }
        };
    }

    if succecc_nfo.is_empty() {
        log::error!("所有模板爬取失败，影片ID: {}", crawler_name);
        return Err(AppError::MovieDataNotFound(format!("所有模版爬取失败，影片ID: {}", crawler_name)));
    }

    log::info!("总共成功爬取 {} 个数据源", succecc_nfo.len());
    let crawler_nfo = clean_crawler_nfos(succecc_nfo).await?;

    Ok(crawler_nfo)
}

/// 计算数据质量评分 (0-100)
fn calculate_data_quality(nfo: &MovieNfoCrawler) -> u32 {
    let mut score = 0u32;
    
    // 基本信息权重
    if !nfo.title.is_empty() { score += 15; }
    if !nfo.plot.is_empty() { score += 10; }
    if !nfo.tagline.is_empty() { score += 5; }
    if nfo.year.is_some() { score += 10; }
    
    // 人员信息权重
    if !nfo.actors.is_empty() { score += 20; }
    if !nfo.directors.is_empty() { score += 10; }
    
    // 制作信息权重
    if !nfo.studios.is_empty() { score += 10; }
    if !nfo.genres.is_empty() { score += 10; }
    
    // 其他信息权重
    if !nfo.fanarts.is_empty() { score += 5; }
    if nfo.original_title.as_ref().map_or(false, |t| !t.is_empty()) { score += 5; }
    
    score
}

async fn clean_crawler_nfos(nfos: Vec<MovieNfoCrawler>) -> Result<MovieNfoCrawler, AppError> {
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
