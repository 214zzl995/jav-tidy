use crate::config::AppConfig;
use crate::nfo::{MediaCenterType, MovieNfo, MovieNfoCrawler, NfoFormatter};
use std::fs;
use std::path::{Path, PathBuf};

/// NFO 文件生成器
///
/// 负责将 MovieNfo 结构序列化为不同媒体中心格式的 XML 并保存到文件系统
pub struct NfoGenerator {
    media_center_type: MediaCenterType,
}

/// NFO 生成配置
#[derive(Debug, Clone)]
#[allow(dead_code)] // 预留给未来的配置功能
pub struct NfoGeneratorConfig {
    pub media_center_type: MediaCenterType,
    pub generate_multiple_formats: bool,   // 是否生成多种格式
    pub custom_xml_header: Option<String>, // 自定义 XML 头部
}

impl NfoGenerator {
    /// 创建新的 NFO 生成器 (默认通用格式)
    pub fn new() -> Self {
        Self {
            media_center_type: MediaCenterType::Universal,
        }
    }

    /// 创建指定媒体中心类型的 NFO 生成器
    pub fn for_media_center(media_center: MediaCenterType) -> Self {
        Self {
            media_center_type: media_center,
        }
    }

    /// 从爬虫数据生成并保存 NFO 文件
    #[allow(dead_code)] // 预留给未来的爬虫数据生成功能
    pub fn generate_from_crawler(
        &self,
        crawler_data: MovieNfoCrawler,
        original_file_path: &Path,
        config: &AppConfig,
    ) -> anyhow::Result<Vec<PathBuf>> {
        let nfo = MovieNfo::for_universal(crawler_data);
        self.generate_and_save(&nfo, original_file_path, config)
    }

    /// 生成并保存 NFO 文件
    ///
    /// # 参数
    /// - `nfo`: 要保存的NFO数据
    /// - `original_file_path`: 原始视频文件路径
    /// - `config`: 应用配置
    ///
    /// # 返回
    /// 成功时返回生成的NFO文件路径列表，失败时返回错误
    #[allow(dead_code)] // 预留给未来的NFO保存功能
    pub fn generate_and_save(
        &self,
        nfo: &MovieNfo,
        original_file_path: &Path,
        config: &AppConfig,
    ) -> anyhow::Result<Vec<PathBuf>> {
        // 生成通用格式的 NFO 文件
        let generated_files = vec![self.save_single_format(
            nfo,
            original_file_path,
            config,
            MediaCenterType::Universal,
        )?];

        Ok(generated_files)
    }

    /// 保存单一格式的 NFO 文件
    #[allow(dead_code)] // 预留给未来的单一格式保存功能
    fn save_single_format(
        &self,
        nfo: &MovieNfo,
        original_file_path: &Path,
        config: &AppConfig,
        _format_type: MediaCenterType,
    ) -> anyhow::Result<PathBuf> {
        // 生成NFO文件路径
        let nfo_path = self.generate_nfo_path(original_file_path, nfo, config, &_format_type)?;

        // 生成指定格式的XML内容
        let xml_content = self.generate_xml_content_for_type(nfo, &_format_type)?;

        // 确保输出目录存在
        if let Some(parent) = nfo_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 保存文件
        fs::write(&nfo_path, xml_content)?;

        log::info!(
            "NFO文件已保存到: {} (格式: {:?})",
            nfo_path.display(),
            _format_type
        );

        Ok(nfo_path)
    }

    /// 生成NFO文件的保存路径
    ///
    /// 根据媒体中心类型生成不同的文件名
    #[allow(dead_code)] // 预留给未来的NFO路径生成功能
    fn generate_nfo_path(
        &self,
        original_file_path: &Path,
        nfo: &MovieNfo,
        config: &AppConfig,
        _format_type: &MediaCenterType,
    ) -> anyhow::Result<PathBuf> {
        let output_dir = config.get_output_dir();

        // 获取文件名基础部分（不包含扩展名）
        let file_stem = original_file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("无法获取文件名"))?;

        // 生成通用格式的文件名
        let new_filename = if !nfo.title.is_empty() {
            format!("{} [{}].nfo", file_stem, self.sanitize_filename(&nfo.title))
        } else {
            format!("{}.nfo", file_stem)
        };

        let nfo_path = output_dir.join(new_filename);

        Ok(nfo_path)
    }

    /// 生成指定类型的XML内容
    fn generate_xml_content_for_type(
        &self,
        nfo: &MovieNfo,
        _format_type: &MediaCenterType,
    ) -> anyhow::Result<String> {
        let xml_content = nfo.format_to_xml();

        if xml_content.is_empty() {
            return Err(anyhow::anyhow!("序列化NFO数据失败: 生成的XML内容为空"));
        }

        // 添加XML头部
        let mut full_xml =
            String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");

        // 添加通用格式注释
        full_xml.push_str("<!-- Universal NFO format compatible with Kodi/Emby/Jellyfin -->\n");

        full_xml.push_str(&xml_content);

        Ok(full_xml)
    }

    /// 生成XML内容 (向下兼容)
    fn generate_xml_content(&self, nfo: &MovieNfo) -> anyhow::Result<String> {
        self.generate_xml_content_for_type(nfo, &self.media_center_type)
    }

    /// 清理文件名中的非法字符
    #[allow(dead_code)] // 预留给未来的文件名清理功能
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

    /// 预览NFO内容（不保存文件）
    #[allow(dead_code)] // 有用的预览功能，保留给未来使用
    pub fn preview_xml(&self, nfo: &MovieNfo) -> anyhow::Result<String> {
        self.generate_xml_content(nfo)
    }

    /// 验证NFO数据的完整性
    pub fn validate_nfo(&self, nfo: &MovieNfo) -> Vec<String> {
        self.validate_nfo_for_type(nfo, &self.media_center_type)
    }

    /// 为指定媒体中心类型验证NFO数据
    pub fn validate_nfo_for_type(
        &self,
        nfo: &MovieNfo,
        _format_type: &MediaCenterType,
    ) -> Vec<String> {
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

        // 通用验证
        if nfo.imdb_id.is_empty() {
            warnings.push("推荐设置 IMDB ID".to_string());
        }

        if nfo.rating.is_none() && nfo.ratings.is_none() {
            warnings.push("推荐设置评分信息".to_string());
        }

        warnings
    }
}

impl Default for NfoGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// NFO 生成器构建器
#[allow(dead_code)] // 预留给未来的构建器功能
pub struct NfoGeneratorBuilder {
    media_center_type: MediaCenterType,
    #[allow(dead_code)] // 预留字段
    generate_multiple_formats: bool,
    #[allow(dead_code)] // 预留字段
    custom_xml_header: Option<String>,
}

impl NfoGeneratorBuilder {
    pub fn new() -> Self {
        Self {
            media_center_type: MediaCenterType::Universal,
            generate_multiple_formats: false,
            custom_xml_header: None,
        }
    }

    #[allow(dead_code)] // 预留给未来的媒体中心配置功能
    pub fn for_media_center(mut self, media_center: MediaCenterType) -> Self {
        self.media_center_type = media_center;
        self
    }

    #[allow(dead_code)] // 预留给未来的多格式生成功能
    pub fn generate_multiple_formats(mut self, enabled: bool) -> Self {
        self.generate_multiple_formats = enabled;
        self
    }

    #[allow(dead_code)] // 预留给未来的自定义XML头部功能
    pub fn with_custom_xml_header(mut self, header: String) -> Self {
        self.custom_xml_header = Some(header);
        self
    }

    #[allow(dead_code)] // 预留给未来的构建功能
    pub fn build(self) -> NfoGenerator {
        NfoGenerator {
            media_center_type: self.media_center_type,
        }
    }
}

impl Default for NfoGeneratorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nfo::{Actor, MediaCenterType, MovieNfo};
    use std::env;

    fn create_test_nfo() -> MovieNfo {
        MovieNfo {
            title: "测试电影".to_string(),
            original_title: "Test Movie".to_string(),
            year: Some(2023),
            plot: "这是一个测试电影的剧情介绍。".to_string(),
            runtime: Some(120),
            genres: vec!["动作".to_string(), "冒险".to_string()],
            actors: vec![Actor {
                name: "演员1".to_string(),
                role: "主角".to_string(),
                ..Default::default()
            }],
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
    fn test_generate_xml_for_universal_format() {
        let nfo = create_test_nfo();

        // 测试通用格式
        let universal_generator = NfoGenerator::for_media_center(MediaCenterType::Universal);
        let universal_xml = universal_generator
            .generate_xml_content_for_type(&nfo, &MediaCenterType::Universal)
            .unwrap();
        assert!(universal_xml.contains("Universal NFO format"));
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
        let _warnings = generator.validate_nfo(&complete_nfo);
        // 由于缺少ID信息，可能会有警告

        // 测试不完整的NFO
        let incomplete_nfo = MovieNfo::default();
        let warnings = generator.validate_nfo(&incomplete_nfo);
        assert!(!warnings.is_empty());
    }

    #[test]
    fn test_nfo_generator_builder() {
        let generator = NfoGeneratorBuilder::new()
            .for_media_center(MediaCenterType::Universal)
            .generate_multiple_formats(true)
            .build();

        // 验证构建器正确设置了媒体中心类型
        assert_eq!(generator.media_center_type, MediaCenterType::Universal);
    }

    #[test]
    fn test_universal_format_validation() {
        let generator = NfoGenerator::new();
        let nfo = create_test_nfo();

        // 测试通用格式验证
        let universal_warnings = generator.validate_nfo_for_type(&nfo, &MediaCenterType::Universal);
        assert!(universal_warnings.iter().any(|w| w.contains("IMDB ID")));
    }
}
