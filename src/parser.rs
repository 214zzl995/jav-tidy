use crate::config::AppConfig;
use regex::Regex;
use std::path::Path;

/// 文件名解析器
///
/// 负责从文件路径中提取影片ID，用于后续的网络搜索
pub struct FileNameParser {
    /// 影片ID提取的正则表达式列表
    movie_id_regexes: Vec<Regex>,
}

impl FileNameParser {
    /// 创建新的文件名解析器
    pub fn new() -> anyhow::Result<Self> {
        // 匹配常见的影片ID格式，按优先级排列
        let patterns = vec![
            // FC2-PPV-数字 格式
            r"(?i)\b(FC2-PPV-\d+)\b",
            // 字母-字母-数字 格式 (如 IPX-001, STAR-123)
            r"(?i)\b([A-Z]+-\d+)\b",
            // 字母数字 格式 (如 IPX001)
            r"(?i)\b([A-Z]+\d+)\b",
        ];

        let mut movie_id_regexes = Vec::new();
        for pattern in patterns {
            movie_id_regexes.push(Regex::new(pattern)?);
        }

        Ok(Self { movie_id_regexes })
    }

    /// 从文件路径中提取影片ID
    ///
    /// # 参数
    /// - `file_path`: 文件路径
    /// - `config`: 应用配置，包含清理规则
    ///
    /// # 返回
    /// 成功时返回影片ID，失败时返回None
    pub fn extract_movie_id(&self, file_path: &Path, config: &AppConfig) -> Option<String> {
        // 获取文件名（不包含扩展名）
        let file_stem = file_path.file_stem()?.to_str()?;

        // 清理文件名
        let cleaned_name = self.clean_filename(file_stem, config);

        log::debug!("原始文件名: {}", file_stem);
        log::debug!("清理后文件名: {}", cleaned_name);

        // 提取影片ID
        let movie_id = self.extract_id_from_cleaned_name(&cleaned_name)?;

        log::info!("从文件 {} 提取到影片ID: {}", file_path.display(), movie_id);

        Some(movie_id)
    }

    /// 清理文件名，移除配置中指定的模式
    fn clean_filename(&self, filename: &str, config: &AppConfig) -> String {
        let mut cleaned = filename.to_string();

        // 按配置移除不需要的模式，用空格替换以避免单词粘连
        for pattern in config.get_ignored_id_pattern() {
            cleaned = cleaned.replace(pattern, " ");
        }

        // 移除多余的空格和分隔符
        cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");

        // 根据配置决定是否转换大小写
        if config.is_capital() {
            cleaned = cleaned.to_lowercase();
        } else {
            cleaned = cleaned.to_uppercase();
        }

        cleaned
    }

    /// 从清理后的文件名中提取影片ID
    fn extract_id_from_cleaned_name(&self, cleaned_name: &str) -> Option<String> {
        // 依次尝试所有正则表达式，按优先级顺序
        for regex in &self.movie_id_regexes {
            if let Some(captures) = regex.captures(cleaned_name) {
                if let Some(movie_id) = captures.get(1) {
                    let movie_id_str = movie_id.as_str();
                    // 标准化格式：确保字母部分大写，保持分隔符
                    let normalized_id = self.normalize_movie_id(movie_id_str);
                    return Some(normalized_id);
                }
            }
        }
        None
    }

    /// 标准化影片ID格式
    fn normalize_movie_id(&self, movie_id: &str) -> String {
        // 分离字母和数字部分
        let parts: Vec<&str> = movie_id.split('-').collect();

        if parts.len() >= 2 {
            // 处理类似 "IPX-001" 的格式
            let prefix = parts[0].to_uppercase();
            let number = parts[1..].join("-");
            format!("{}-{}", prefix, number)
        } else {
            // 处理没有分隔符的格式
            movie_id.to_uppercase()
        }
    }

    /// 验证提取的影片ID是否有效
    #[allow(dead_code)] // 有用的验证功能，保留给未来使用
    pub fn is_valid_movie_id(&self, movie_id: &str) -> bool {
        // 尝试所有正则表达式，如果任何一个匹配就认为有效
        for regex in &self.movie_id_regexes {
            if regex.is_match(movie_id) {
                return true;
            }
        }
        false
    }
}

impl Default for FileNameParser {
    fn default() -> Self {
        Self::new().expect("Failed to create FileNameParser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> AppConfig {
        // 创建一个临时的配置文件进行测试
        use std::env;
        use std::fs;

        let test_config_content = r#"
migrate_files = ["mp4"]
migrate_subtitles = false
ignored_id_pattern = ["-HD", "-1080p", "_chinese", ".subtitled", "_", "."]
capital = false
input_dir = "./input"
output_dir = "./output"
thread_limit = 4
template_priority = ["javdb.yaml"]
maximum_fetch_count = 3
"#;

        let temp_dir = env::temp_dir();
        let config_path = temp_dir.join("test_config.toml");
        fs::write(&config_path, test_config_content).unwrap();

        AppConfig::new(&config_path).unwrap()
    }

    #[test]
    fn test_extract_movie_id_basic() {
        let parser = FileNameParser::new().unwrap();
        let config = create_test_config();

        let test_cases = vec![
            ("IPX-001.mp4", Some("IPX-001")),
            ("STAR-123_1080p.mkv", Some("STAR-123")),
            ("SSIS-456-HD_chinese.avi", Some("SSIS-456")),
            ("FC2-PPV-1234567.mp4", Some("FC2-PPV-1234567")),
            ("invalid_file.mp4", None),
        ];

        for (filename, expected) in test_cases {
            let path = Path::new(filename);
            let result = parser.extract_movie_id(path, &config);
            assert_eq!(
                result.as_deref(),
                expected,
                "Failed for filename: {}",
                filename
            );
        }
    }

    #[test]
    fn test_is_valid_movie_id() {
        let parser = FileNameParser::new().unwrap();

        assert!(parser.is_valid_movie_id("IPX-001"));
        assert!(parser.is_valid_movie_id("STAR-123"));
        assert!(parser.is_valid_movie_id("FC2-PPV-1234567"));
        assert!(!parser.is_valid_movie_id("invalid"));
        assert!(!parser.is_valid_movie_id("123-456"));
    }
}
