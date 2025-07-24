use std::path::{Path, PathBuf};
use std::fs;
use quick_xml::se::to_string;
use crate::nfo::MovieNfo;
use crate::config::AppConfig;

/// NFO文件生成器
/// 
/// 负责将MovieNfo结构序列化为XML格式并保存到文件系统
pub struct NfoGenerator;

impl NfoGenerator {
    /// 创建新的NFO生成器
    pub fn new() -> Self {
        Self
    }

    /// 生成并保存NFO文件
    /// 
    /// # 参数
    /// - `nfo`: 要保存的NFO数据
    /// - `original_file_path`: 原始视频文件路径
    /// - `config`: 应用配置
    /// 
    /// # 返回
    /// 成功时返回生成的NFO文件路径，失败时返回错误
    pub fn generate_and_save(
        &self,
        nfo: &MovieNfo,
        original_file_path: &Path,
        config: &AppConfig,
    ) -> anyhow::Result<PathBuf> {
        // 生成NFO文件路径
        let nfo_path = self.generate_nfo_path(original_file_path, nfo, config)?;
        
        // 生成XML内容
        let xml_content = self.generate_xml_content(nfo)?;
        
        // 确保输出目录存在
        if let Some(parent) = nfo_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // 保存文件
        fs::write(&nfo_path, xml_content)?;
        
        log::info!("NFO文件已保存到: {}", nfo_path.display());
        
        Ok(nfo_path)
    }

    /// 生成NFO文件的保存路径
    /// 
    /// 规则：输出目录/[影片ID] [标题].nfo
    fn generate_nfo_path(
        &self,
        original_file_path: &Path,
        nfo: &MovieNfo,
        config: &AppConfig,
    ) -> anyhow::Result<PathBuf> {
        let output_dir = config.get_output_dir();
        
        // 获取文件名基础部分（不包含扩展名）
        let file_stem = original_file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("无法获取文件名"))?;
        
        // 生成新的文件名
        let new_filename = if !nfo.title.is_empty() {
            // 如果有标题，使用格式：[原文件名] [标题].nfo
            format!("{} [{}].nfo", file_stem, self.sanitize_filename(&nfo.title))
        } else {
            // 如果没有标题，使用原文件名
            format!("{}.nfo", file_stem)
        };
        
        let nfo_path = output_dir.join(new_filename);
        
        Ok(nfo_path)
    }

    /// 生成XML内容
    fn generate_xml_content(&self, nfo: &MovieNfo) -> anyhow::Result<String> {
        // 添加XML头部
        let mut xml_content = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
        
        // 序列化NFO数据为XML
        let nfo_xml = to_string(nfo)
            .map_err(|e| anyhow::anyhow!("序列化NFO数据失败: {}", e))?;
        
        xml_content.push_str(&nfo_xml);
        
        Ok(xml_content)
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

    /// 预览NFO内容（不保存文件）
    #[allow(dead_code)] // 有用的预览功能，保留给未来使用
    pub fn preview_xml(&self, nfo: &MovieNfo) -> anyhow::Result<String> {
        self.generate_xml_content(nfo)
    }

    /// 验证NFO数据的完整性
    pub fn validate_nfo(&self, nfo: &MovieNfo) -> Vec<String> {
        let mut warnings = Vec::new();
        
        if nfo.title.is_empty() {
            warnings.push("标题为空".to_string());
        }
        
        if nfo.plot.is_empty() {
            warnings.push("剧情简介为空".to_string());
        }
        
        if nfo.year.is_none() {
            warnings.push("发行年份未设置".to_string());
        }
        
        if nfo.runtime.is_none() {
            warnings.push("运行时长未设置".to_string());
        }
        
        if nfo.actors.is_empty() {
            warnings.push("演员列表为空".to_string());
        }
        
        if nfo.genres.is_empty() {
            warnings.push("类型标签为空".to_string());
        }
        
        warnings
    }
}

impl Default for NfoGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nfo::{MovieNfo, Actor};
    use std::env;

    fn create_test_nfo() -> MovieNfo {
        MovieNfo {
            title: "测试电影".to_string(),
            original_title: "Test Movie".to_string(),
            year: Some(2023),
            plot: "这是一个测试电影的剧情介绍。".to_string(),
            runtime: Some(120),
            genres: vec!["动作".to_string(), "冒险".to_string()],
            actors: vec![
                Actor {
                    name: "演员1".to_string(),
                    role: "主角".to_string(),
                    ..Default::default()
                }
            ],
            ..Default::default()
        }
    }

    #[allow(dead_code)] // 测试辅助函数
    fn create_test_config() -> AppConfig {
        use std::fs;
        
        let test_config_content = r#"
migrate_files = ["mp4"]
migrate_subtitles = false
ignored_id_pattern = []
capital = false
input_dir = "./input"
output_dir = "./test_output"
thread_limit = 4
template_priority = ["javdb.yaml"]
maximum_fetch_count = 3
"#;
        
        let temp_dir = env::temp_dir();
        let config_path = temp_dir.join("test_nfo_config.toml");
        fs::write(&config_path, test_config_content).unwrap();
        
        AppConfig::new(&config_path).unwrap()
    }

    #[test]
    fn test_generate_xml_content() {
        let generator = NfoGenerator::new();
        let nfo = create_test_nfo();
        
        let xml_result = generator.generate_xml_content(&nfo);
        assert!(xml_result.is_ok());
        
        let xml = xml_result.unwrap();
        assert!(xml.contains("<?xml version=\"1.0\""));
        assert!(xml.contains("<movie>"));
        assert!(xml.contains("测试电影"));
    }

    #[test]
    fn test_sanitize_filename() {
        let generator = NfoGenerator::new();
        
        let test_cases = vec![
            ("test<file>name", "testfilename"),
            ("file/with\\slashes", "filewithslashes"),
            ("file:with|illegal*chars?", "filewithillegalchars"),
            ("  multiple   spaces  ", "multiple spaces"),
        ];
        
        for (input, expected) in test_cases {
            let result = generator.sanitize_filename(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_validate_nfo() {
        let generator = NfoGenerator::new();
        
        // 测试完整的NFO
        let complete_nfo = create_test_nfo();
        let warnings = generator.validate_nfo(&complete_nfo);
        assert!(warnings.is_empty());
        
        // 测试不完整的NFO
        let incomplete_nfo = MovieNfo::default();
        let warnings = generator.validate_nfo(&incomplete_nfo);
        assert!(!warnings.is_empty());
    }
} 