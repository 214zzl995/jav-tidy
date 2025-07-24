use std::path::{Path, PathBuf};
use std::fs;
use crate::nfo::MovieNfo;
use crate::config::AppConfig;

/// 文件整理器
/// 
/// 负责将处理完成的视频文件移动到输出目录并重命名
pub struct FileOrganizer;

impl FileOrganizer {
    /// 创建新的文件整理器
    pub fn new() -> Self {
        Self
    }

    /// 整理文件：移动并重命名视频文件
    /// 
    /// # 参数
    /// - `original_file_path`: 原始视频文件路径
    /// - `nfo`: NFO数据，用于生成新文件名
    /// - `config`: 应用配置
    /// 
    /// # 返回
    /// 成功时返回新的文件路径，失败时返回错误
    pub fn organize_file(
        &self,
        original_file_path: &Path,
        nfo: &MovieNfo,
        config: &AppConfig,
    ) -> anyhow::Result<PathBuf> {
        // 生成新的文件路径
        let new_file_path = self.generate_new_file_path(original_file_path, nfo, config)?;
        
        // 确保输出目录存在
        if let Some(parent) = new_file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // 处理文件名冲突
        let final_file_path = self.resolve_filename_conflict(&new_file_path)?;
        
        // 移动文件
        self.move_file(original_file_path, &final_file_path)?;
        
        // 如果配置允许，同时移动字幕文件
        if config.migrate_subtitles() {
            if let Err(e) = self.move_subtitle_files(original_file_path, &final_file_path) {
                log::warn!("移动字幕文件失败: {}", e);
            }
        }
        
        log::info!("文件已整理到: {}", final_file_path.display());
        
        Ok(final_file_path)
    }

    /// 生成新的文件路径
    /// 
    /// 规则：输出目录/[影片ID] [标题].扩展名
    fn generate_new_file_path(
        &self,
        original_file_path: &Path,
        nfo: &MovieNfo,
        config: &AppConfig,
    ) -> anyhow::Result<PathBuf> {
        let output_dir = config.get_output_dir();
        
        // 获取原文件的扩展名
        let extension = original_file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow::anyhow!("无法获取文件扩展名"))?;
        
        // 获取文件名基础部分（用于提取影片ID）
        let file_stem = original_file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("无法获取文件名"))?;
        
        // 生成新的文件名
        let new_filename = if !nfo.title.is_empty() {
            // 如果有标题，使用格式：[原文件名] [标题].扩展名
            format!("{} [{}].{}", 
                file_stem,
                self.sanitize_filename(&nfo.title),
                extension
            )
        } else {
            // 如果没有标题，保持原文件名
            format!("{}.{}", file_stem, extension)
        };
        
        let new_file_path = output_dir.join(new_filename);
        
        Ok(new_file_path)
    }

    /// 解决文件名冲突
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
    fn move_file(&self, source: &Path, destination: &Path) -> anyhow::Result<()> {
        // 首先尝试重命名（如果在同一个文件系统上）
        if let Err(_) = fs::rename(source, destination) {
            // 如果重命名失败，尝试复制然后删除
            fs::copy(source, destination)?;
            fs::remove_file(source)?;
        }
        
        log::debug!("文件移动成功: {} -> {}", source.display(), destination.display());
        
        Ok(())
    }

    /// 移动相关的字幕文件
    fn move_subtitle_files(&self, original_video_path: &Path, new_video_path: &Path) -> anyhow::Result<()> {
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
    fn sanitize_filename(&self, filename: &str) -> String {
        // 移除或替换文件名中的非法字符
        let illegal_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
        let mut sanitized = filename.to_string();
        
        for char in illegal_chars {
            sanitized = sanitized.replace(char, "");
        }
        
        // 移除多余的空格
        sanitized = sanitized
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        
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
        let result = organizer.preview_new_path(original_path, &nfo, &config);
        
        assert!(result.is_ok());
        let new_path = result.unwrap();
        assert!(new_path.to_string_lossy().contains("IPX-001"));
        assert!(new_path.to_string_lossy().contains("测试电影"));
        assert!(new_path.to_string_lossy().ends_with(".mp4"));
    }
} 