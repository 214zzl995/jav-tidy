use crate::config::AppConfig;
use crate::nfo::MovieNfo;
use crate::template_parser::{TemplateParser, MultiActorStrategy};
use std::fs;
use std::path::{Path, PathBuf};

/// 文件整理器
///
/// 负责将处理完成的视频文件移动到输出目录并重命名
pub struct FileOrganizer;

impl FileOrganizer {
    /// 创建新的文件整理器
    pub fn new() -> Self {
        Self
    }

    /// 整理文件：为媒体中心创建标准目录结构
    ///
    /// # 参数
    /// - `original_file_path`: 原始视频文件路径
    /// - `nfo`: NFO数据，用于生成新文件名和目录结构
    /// - `config`: 应用配置
    ///
    /// # 返回
    /// 成功时返回新的文件路径和NFO文件路径，失败时返回错误
    #[allow(dead_code)] // 预留给未来的文件整理功能
    pub fn organize_file(
        &self,
        original_file_path: &Path,
        nfo: &MovieNfo,
        config: &AppConfig,
    ) -> anyhow::Result<(PathBuf, PathBuf)> {
        // 为媒体中心生成标准目录结构
        let (movie_dir, video_filename, nfo_filename) =
            self.generate_media_center_structure(original_file_path, nfo, config)?;

        // 确保影片目录存在
        fs::create_dir_all(&movie_dir)?;
        log::info!("创建影片目录: {}", movie_dir.display());

        // 生成最终文件路径
        let final_video_path = movie_dir.join(&video_filename);
        let final_nfo_path = movie_dir.join(&nfo_filename);

        // 处理文件名冲突
        let resolved_video_path = self.resolve_filename_conflict(&final_video_path)?;
        let resolved_nfo_path = if resolved_video_path != final_video_path {
            // 如果视频文件名被修改，同步修改NFO文件名
            let video_stem = resolved_video_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("movie");
            movie_dir.join(format!("{}.nfo", video_stem))
        } else {
            final_nfo_path
        };

        // 移动视频文件
        self.move_file(original_file_path, &resolved_video_path)?;
        log::info!("视频文件已移动到: {}", resolved_video_path.display());

        // 如果配置允许，同时移动字幕文件
        if config.migrate_subtitles() {
            if let Err(e) = self.move_subtitle_files(original_file_path, &resolved_video_path) {
                log::warn!("移动字幕文件失败: {}", e);
            }
        }

        log::info!(
            "文件整理完成 - 视频: {}, NFO: {}",
            resolved_video_path.display(),
            resolved_nfo_path.display()
        );

        Ok((resolved_video_path, resolved_nfo_path))
    }

    /// 为媒体中心生成标准目录结构
    ///
    /// 结构：输出目录/[系列名或影片ID (Year)]/影片名 (Year).扩展名
    fn generate_media_center_structure(
        &self,
        original_file_path: &Path,
        nfo: &MovieNfo,
        config: &AppConfig,
    ) -> anyhow::Result<(PathBuf, String, String)> {
        let output_dir = config.get_output_dir();

        // 获取原文件的扩展名
        let extension = original_file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow::anyhow!("无法获取文件扩展名"))?;

        // 创建模板解析器并填充NFO数据
        let mut parser = TemplateParser::new();
        parser.populate_from_nfo(nfo)?;

        // 从配置获取模板和策略
        let template = config.get_file_naming_template();
        let strategy = MultiActorStrategy::from_string(config.get_multi_actor_strategy());

        // 解析模板获取路径结构
        let parse_result = parser.parse_template(template, strategy)?;

        // 构建主要路径
        let movie_dir = output_dir.join(&parse_result.primary_path);
        
        // 生成文件名：从路径中提取最后一部分作为文件名基础
        let path_parts: Vec<&str> = parse_result.primary_path.split('/').collect();
        let base_filename = path_parts.last().map_or("Unknown", |v| v);
        
        let video_filename = format!("{}.{}", base_filename, extension);
        let nfo_filename = format!("{}.nfo", base_filename);

        Ok((movie_dir, video_filename, nfo_filename))
    }

    /// 处理多演员文件链接
    /// 
    /// 根据配置的多演员策略，为每个额外的演员创建链接
    pub fn handle_multi_actor_links(
        &self,
        original_file_path: &Path,
        nfo: &MovieNfo,
        config: &AppConfig,
        primary_video_path: &Path,
        primary_nfo_path: &Path,
    ) -> anyhow::Result<Vec<(PathBuf, PathBuf)>> {
        let mut additional_links = Vec::new();
        
        // 获取原文件的扩展名
        let extension = original_file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow::anyhow!("无法获取文件扩展名"))?;

        // 创建模板解析器并填充NFO数据
        let mut parser = TemplateParser::new();
        parser.populate_from_nfo(nfo)?;

        // 从配置获取模板和策略
        let template = config.get_file_naming_template();
        let strategy = MultiActorStrategy::from_string(config.get_multi_actor_strategy());

        // 只有当策略为HardLink或SymLink时才处理额外链接
        if !matches!(strategy, MultiActorStrategy::HardLink | MultiActorStrategy::SymLink) {
            return Ok(additional_links);
        }

        // 解析模板获取额外路径
        let parse_result = parser.parse_template(template, strategy.clone())?;
        
        let output_dir = config.get_output_dir();
        
        for additional_path in parse_result.additional_paths {
            // 构建额外演员的目录
            let additional_movie_dir = output_dir.join(&additional_path);
            
            // 生成文件名
            let path_parts: Vec<&str> = additional_path.split('/').collect();
            let base_filename = path_parts.last().map_or("Unknown", |v| v);
            
            let additional_video_path = additional_movie_dir.join(format!("{}.{}", base_filename, extension));
            let additional_nfo_path = additional_movie_dir.join(format!("{}.nfo", base_filename));
            
            // 创建目录
            fs::create_dir_all(&additional_movie_dir)?;
            log::info!("创建额外演员目录: {}", additional_movie_dir.display());
            
            // 创建链接
            match strategy {
                MultiActorStrategy::HardLink => {
                    // 创建硬链接
                    if let Err(e) = fs::hard_link(primary_video_path, &additional_video_path) {
                        log::warn!("创建硬链接失败，回退到符号链接: {}", e);
                        self.create_symlink(primary_video_path, &additional_video_path)?;
                    }
                    if let Err(e) = fs::hard_link(primary_nfo_path, &additional_nfo_path) {
                        log::warn!("创建NFO硬链接失败，回退到符号链接: {}", e);
                        self.create_symlink(primary_nfo_path, &additional_nfo_path)?;
                    }
                },
                MultiActorStrategy::SymLink => {
                    // 创建符号链接
                    self.create_symlink(primary_video_path, &additional_video_path)?;
                    self.create_symlink(primary_nfo_path, &additional_nfo_path)?;
                },
                _ => unreachable!(), // 前面已经过滤了其他策略
            }
            
            log::info!(
                "创建多演员链接 - 视频: {}, NFO: {}",
                additional_video_path.display(),
                additional_nfo_path.display()
            );
            
            additional_links.push((additional_video_path, additional_nfo_path));
        }
        
        Ok(additional_links)
    }

    /// 创建符号链接的跨平台实现
    #[cfg(unix)]
    fn create_symlink(&self, src: &Path, dst: &Path) -> anyhow::Result<()> {
        std::os::unix::fs::symlink(src, dst)
            .map_err(|e| anyhow::anyhow!("创建符号链接失败: {}", e))
    }

    #[cfg(windows)]
    fn create_symlink(&self, src: &Path, dst: &Path) -> anyhow::Result<()> {
        std::os::windows::fs::symlink_file(src, dst)
            .map_err(|e| anyhow::anyhow!("创建符号链接失败: {}", e))
    }

    /// 生成新的文件路径（保持向后兼容）
    ///
    /// 规则：输出目录/[影片ID] [标题].扩展名
    pub fn generate_new_file_path(
        &self,
        original_file_path: &Path,
        nfo: &MovieNfo,
        config: &AppConfig,
    ) -> anyhow::Result<PathBuf> {
        let (movie_dir, video_filename, _) =
            self.generate_media_center_structure(original_file_path, nfo, config)?;
        Ok(movie_dir.join(video_filename))
    }

    /// 解决文件名冲突
    #[allow(dead_code)] // 预留给未来的文件冲突处理功能
    fn resolve_filename_conflict(&self, file_path: &Path) -> anyhow::Result<PathBuf> {
        if !file_path.exists() {
            return Ok(file_path.to_path_buf());
        }

        let file_stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("无法获取文件名"))?;

        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let parent = file_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("无法获取父目录"))?;

        // 尝试添加序号后缀
        for i in 1..=999 {
            let new_filename = if extension.is_empty() {
                format!("{} ({})", file_stem, i)
            } else {
                format!("{} ({}).{}", file_stem, i, extension)
            };

            let new_path = parent.join(new_filename);
            if !new_path.exists() {
                log::info!("解决文件名冲突，使用: {}", new_path.display());
                return Ok(new_path);
            }
        }

        Err(anyhow::anyhow!("无法解决文件名冲突，尝试了999个后缀"))
    }

    /// 移动文件
    #[allow(dead_code)] // 预留给未来的文件移动功能
    fn move_file(&self, source: &Path, destination: &Path) -> anyhow::Result<()> {
        // 首先尝试重命名（如果在同一个文件系统上）
        if fs::rename(source, destination).is_err() {
            // 如果重命名失败，尝试复制然后删除
            fs::copy(source, destination)?;
            fs::remove_file(source)?;
        }

        log::debug!(
            "文件移动成功: {} -> {}",
            source.display(),
            destination.display()
        );

        Ok(())
    }

    /// 移动相关的字幕文件 (基于爬取后的ID匹配)
    /// 
    /// 使用爬取后的影片ID（如IPX-001）在输入目录中查找匹配的字幕文件
    pub fn migrate_subtitle_files(
        &self,
        movie_id: &str,
        input_dir: &Path,
        target_video_path: &Path,
        config: &AppConfig,
    ) -> anyhow::Result<Vec<PathBuf>> {
        if !config.migrate_subtitles() {
            return Ok(vec![]);
        }

        let mut migrated_subtitles = Vec::new();
        let subtitle_extensions = config.get_subtitle_extensions();
        
        // 标准化影片ID：移除特殊字符，转为小写
        let normalized_movie_id = self.normalize_identifier(movie_id);
        
        log::info!("开始查找字幕文件，影片ID: {} (标准化: {})", movie_id, normalized_movie_id);

        // 获取目标目录和文件名基础部分
        let target_dir = target_video_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("无法获取目标目录"))?;
        
        let target_stem = target_video_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("无法获取目标文件名"))?;

        // 遍历输入目录查找匹配的字幕文件
        let entries = fs::read_dir(input_dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if !path.is_file() {
                continue;
            }
            
            let file_name = match path.file_name().and_then(|s| s.to_str()) {
                Some(name) => name,
                None => continue,
            };
            
            // 检查是否为字幕文件
            let extension = match path.extension().and_then(|s| s.to_str()) {
                Some(ext) => ext.to_lowercase(),
                None => continue,
            };
            
            if !subtitle_extensions.iter().any(|sub_ext| sub_ext.to_lowercase() == extension) {
                continue;
            }
            
            // 从字幕文件名中提取标识符并标准化
            let subtitle_identifier = self.extract_identifier_from_filename(file_name);
            let normalized_subtitle_id = self.normalize_identifier(&subtitle_identifier);
            
            log::debug!(
                "检查字幕文件: {} -> 标识符: {} (标准化: {})", 
                file_name, subtitle_identifier, normalized_subtitle_id
            );
            
            // 匹配标准化后的标识符
            if normalized_subtitle_id == normalized_movie_id {
                // 生成目标字幕文件路径，包含语言标识
                let subtitle_language = config.get_subtitle_language();
                let target_subtitle_path = target_dir.join(format!("{}.{}.{}", target_stem, subtitle_language, extension));
                
                // 移动字幕文件
                if let Err(e) = fs::rename(&path, &target_subtitle_path) {
                    log::warn!("移动字幕文件失败，尝试复制: {}", e);
                    fs::copy(&path, &target_subtitle_path)?;
                    if let Err(e) = fs::remove_file(&path) {
                        log::warn!("删除原字幕文件失败: {}", e);
                    }
                }
                
                log::info!("字幕文件已迁移: {} -> {}", path.display(), target_subtitle_path.display());
                migrated_subtitles.push(target_subtitle_path);
            }
        }
        
        if migrated_subtitles.is_empty() {
            log::debug!("未找到匹配的字幕文件: {}", movie_id);
        } else {
            log::info!("成功迁移 {} 个字幕文件", migrated_subtitles.len());
        }
        
        Ok(migrated_subtitles)
    }

    /// 标准化标识符：移除特殊字符，转为小写
    fn normalize_identifier(&self, identifier: &str) -> String {
        identifier
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_lowercase()
    }

    /// 从文件名中提取标识符（影片ID）
    fn extract_identifier_from_filename(&self, filename: &str) -> String {
        // 移除扩展名
        let name_without_ext = match filename.rfind('.') {
            Some(pos) => &filename[..pos],
            None => filename,
        };
        
        // 尝试提取类似 "IPX-001", "PRED-123" 等格式的ID
        // 使用正则表达式匹配常见的影片ID格式
        if let Ok(re) = regex::Regex::new(r"([A-Za-z]+[-_]?\d+)") {
            if let Some(captures) = re.find(name_without_ext) {
                return captures.as_str().to_string();
            }
        }
        
        // 如果没有匹配到标准格式，返回整个文件名（不含扩展名）
        name_without_ext.to_string()
    }

    /// 旧版本的字幕文件移动方法（保持向后兼容）
    #[allow(dead_code)] // 预留给未来的字幕文件移动功能
    fn move_subtitle_files(
        &self,
        original_video_path: &Path,
        new_video_path: &Path,
    ) -> anyhow::Result<()> {
        let subtitle_extensions = ["srt", "ass", "ssa", "vtt", "sub", "idx"];

        let original_stem = original_video_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("无法获取原始文件名"))?;

        let new_stem = new_video_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("无法获取新文件名"))?;

        let original_dir = original_video_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("无法获取原始目录"))?;

        let new_dir = new_video_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("无法获取新目录"))?;

        // 查找并移动字幕文件
        for ext in &subtitle_extensions {
            let subtitle_path = original_dir.join(format!("{}.{}", original_stem, ext));
            if subtitle_path.exists() {
                let new_subtitle_path = new_dir.join(format!("{}.{}", new_stem, ext));

                // 解决字幕文件的文件名冲突
                let final_subtitle_path = self.resolve_filename_conflict(&new_subtitle_path)?;

                self.move_file(&subtitle_path, &final_subtitle_path)?;
                log::info!("字幕文件已移动: {}", final_subtitle_path.display());
            }
        }

        Ok(())
    }

    /// 清理文件名中的非法字符
    #[allow(dead_code)]
    fn sanitize_filename(&self, filename: &str) -> String {
        // 移除或替换文件名中的非法字符
        let illegal_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
        let mut sanitized = filename.to_string();

        for char in illegal_chars {
            sanitized = sanitized.replace(char, "");
        }

        // 移除多余的空格
        sanitized = sanitized.split_whitespace().collect::<Vec<_>>().join(" ");

        // 限制长度以避免路径过长
        if sanitized.len() > 100 {
            sanitized.truncate(100);
            sanitized = sanitized.trim_end().to_string();
        }

        sanitized
    }

    /// 检查文件是否需要整理（已经在输出目录中）
    pub fn needs_organization(&self, file_path: &Path, config: &AppConfig) -> bool {
        let output_dir = config.get_output_dir();

        // 检查文件是否已经在输出目录中
        match file_path.parent() {
            Some(parent) => parent != output_dir,
            None => true, // 如果无法获取父目录，假设需要整理
        }
    }

    /// 生成NFO文件路径
    #[allow(dead_code)] // 预留给未来的NFO路径生成功能
    pub fn generate_nfo_path(
        &self,
        original_file_path: &Path,
        nfo: &MovieNfo,
        config: &AppConfig,
    ) -> anyhow::Result<PathBuf> {
        let (movie_dir, _, nfo_filename) =
            self.generate_media_center_structure(original_file_path, nfo, config)?;
        Ok(movie_dir.join(nfo_filename))
    }

    /// 预览新的文件路径（不实际移动文件）
    #[allow(dead_code)] // 有用的预览功能，保留给未来使用
    pub fn preview_new_path(
        &self,
        original_file_path: &Path,
        nfo: &MovieNfo,
        config: &AppConfig,
    ) -> anyhow::Result<PathBuf> {
        self.generate_new_file_path(original_file_path, nfo, config)
    }

    /// 预览媒体中心结构
    pub fn preview_media_center_structure(
        &self,
        original_file_path: &Path,
        nfo: &MovieNfo,
        config: &AppConfig,
    ) -> anyhow::Result<(PathBuf, PathBuf)> {
        let (movie_dir, video_filename, nfo_filename) =
            self.generate_media_center_structure(original_file_path, nfo, config)?;
        let video_path = movie_dir.join(video_filename);
        let nfo_path = movie_dir.join(nfo_filename);
        Ok((video_path, nfo_path))
    }
}

impl Default for FileOrganizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nfo::MovieNfo;
    use std::env;
    use std::fs;

    fn create_test_config() -> AppConfig {
        let test_config_content = r#"
migrate_files = ["mp4"]
migrate_subtitles = true
ignored_id_pattern = []
capital = false
input_dir = "./test_input"
output_dir = "./test_output"
thread_limit = 4
template_priority = ["javdb.yaml"]
maximum_fetch_count = 3
subtitle_language = "zh-CN"
"#;

        let temp_dir = env::temp_dir();
        let config_path = temp_dir.join("test_organizer_config.toml");
        fs::write(&config_path, test_config_content).unwrap();

        AppConfig::new(&config_path).unwrap()
    }

    fn create_test_nfo() -> MovieNfo {
        MovieNfo {
            title: "测试电影".to_string(),
            original_title: "Test Movie".to_string(),
            year: Some(2023),
            ..Default::default()
        }
    }

    #[test]
    fn test_sanitize_filename() {
        let organizer = FileOrganizer::new();

        let test_cases = vec![
            ("test<file>name", "testfilename"),
            ("file/with\\slashes", "filewithslashes"),
            ("file:with|illegal*chars?", "filewithillegalchars"),
            ("  multiple   spaces  ", "multiple spaces"),
        ];

        for (input, expected) in test_cases {
            let result = organizer.sanitize_filename(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_needs_organization() {
        let organizer = FileOrganizer::new();
        let config = create_test_config();

        // 测试需要整理的文件
        let input_file = Path::new("./test_input/movie.mp4");
        assert!(organizer.needs_organization(input_file, &config));

        // 测试不需要整理的文件
        let output_file = Path::new("./test_output/movie.mp4");
        assert!(!organizer.needs_organization(output_file, &config));
    }

    #[test]
    fn test_preview_new_path() {
        let organizer = FileOrganizer::new();
        let config = create_test_config();
        let nfo = create_test_nfo();

        let original_path = Path::new("./test_input/IPX-001.mp4");
        let result = organizer.preview_media_center_structure(original_path, &nfo, &config);

        assert!(result.is_ok());
        let (video_path, nfo_path) = result.unwrap();

        // 验证媒体中心结构
        assert!(video_path.to_string_lossy().contains("测试电影 (2023)"));
        assert!(video_path.to_string_lossy().ends_with(".mp4"));

        // 验证NFO文件路径
        assert!(nfo_path.to_string_lossy().contains("测试电影 (2023)"));
        assert!(nfo_path.to_string_lossy().ends_with(".nfo"));

        // 验证它们在同一个目录中
        assert_eq!(video_path.parent(), nfo_path.parent());
    }

    #[test]
    fn test_normalize_identifier() {
        let organizer = FileOrganizer::new();
        
        let test_cases = vec![
            ("IPX-001", "ipx001"),
            ("PRED_123", "pred123"),
            ("ABC-DEF-456", "abcdef456"),
            ("Test Movie!", "testmovie"),
            ("Movie@123#", "movie123"),
            ("  spaces  ", "spaces"),
        ];

        for (input, expected) in test_cases {
            let result = organizer.normalize_identifier(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_extract_identifier_from_filename() {
        let organizer = FileOrganizer::new();
        
        let test_cases = vec![
            ("IPX-001.mp4", "IPX-001"),
            ("PRED_123.mkv", "PRED_123"),
            ("ABC-456-some-extra-text.mp4", "ABC-456"),
            ("Movie IPX001 other text.mp4", "IPX001"),
            ("no-match-here.mp4", "no-match-here"),
            ("STARS123.srt", "STARS123"),
        ];

        for (input, expected) in test_cases {
            let result = organizer.extract_identifier_from_filename(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_migrate_subtitle_files() {
        use std::fs;
        let organizer = FileOrganizer::new();
        let config = create_test_config();

        // 创建临时目录结构
        let temp_dir = env::temp_dir();
        let input_dir = temp_dir.join("test_subtitle_input");
        let output_dir = temp_dir.join("test_subtitle_output");
        
        // 创建测试目录
        let _ = fs::create_dir_all(&input_dir);
        let _ = fs::create_dir_all(&output_dir);

        // 创建测试字幕文件
        let subtitle_files = vec![
            "IPX-001.srt",           // 匹配的字幕
            "IPX-001.chs.srt",       // 匹配的中文字幕
            "PRED-123.ass",          // 不匹配的字幕
            "ipx001.vtt",            // 匹配的字幕（不同格式）
            "random-file.txt",       // 非字幕文件
        ];

        for file in &subtitle_files {
            let path = input_dir.join(file);
            fs::write(&path, "test subtitle content").unwrap();
        }

        // 目标视频文件路径
        let target_video_path = output_dir.join("Test Movie (2023).mp4");
        fs::create_dir_all(target_video_path.parent().unwrap()).unwrap();

        // 测试字幕迁移
        let result = organizer.migrate_subtitle_files(
            "IPX-001",
            &input_dir,
            &target_video_path,
            &config,
        );

        assert!(result.is_ok());
        let migrated_files = result.unwrap();
        
        // 应该迁移3个匹配的字幕文件（IPX-001.srt, IPX-001.chs.srt, ipx001.vtt）
        assert_eq!(migrated_files.len(), 3);

        // 验证文件确实被移动了，并检查文件名格式
        for migrated in &migrated_files {
            assert!(migrated.exists(), "Migrated file should exist: {:?}", migrated);
            
            // 验证文件名包含语言标识 zh-CN
            let file_name = migrated.file_name().unwrap().to_str().unwrap();
            assert!(file_name.contains(".zh-CN."), "File name should contain language code: {}", file_name);
        }
        
        // 验证生成的文件数量正确
        assert_eq!(migrated_files.len(), 3, "应该迁移3个字幕文件");

        // 清理测试文件
        let _ = fs::remove_dir_all(&input_dir);
        let _ = fs::remove_dir_all(&output_dir);
    }

    #[test]
    fn test_migrate_subtitle_files_disabled() {
        let organizer = FileOrganizer::new();
        
        // 创建禁用字幕迁移的配置
        let test_config_content = r#"
migrate_files = ["mp4"]
migrate_subtitles = false
ignored_id_pattern = []
capital = false
input_dir = "./test_input"
output_dir = "./test_output"
thread_limit = 4
template_priority = ["javdb.yaml"]
maximum_fetch_count = 3
"#;

        let temp_dir = env::temp_dir();
        let config_path = temp_dir.join("test_organizer_config_disabled.toml");
        fs::write(&config_path, test_config_content).unwrap();
        let config = AppConfig::new(&config_path).unwrap();

        let input_dir = temp_dir.join("test_subtitle_input_disabled");
        let output_dir = temp_dir.join("test_subtitle_output_disabled");
        let target_video_path = output_dir.join("Test Movie (2023).mp4");

        // 当禁用字幕迁移时，应该返回空数组
        let result = organizer.migrate_subtitle_files(
            "IPX-001",
            &input_dir,
            &target_video_path,
            &config,
        );

        assert!(result.is_ok());
        let migrated_files = result.unwrap();
        assert_eq!(migrated_files.len(), 0);

        // 清理
        let _ = fs::remove_file(&config_path);
    }
}
