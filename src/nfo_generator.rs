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
pub struct NfoGeneratorConfig {
    pub media_center_type: MediaCenterType,
    pub generate_multiple_formats: bool,   // 是否生成多种格式
    pub custom_xml_header: Option<String>, // 自定义 XML 头部
}

impl NfoGenerator {
    /// 创建新的 NFO 生成器 (默认兼容所有格式)
    pub fn new() -> Self {
        Self {
            media_center_type: MediaCenterType::All,
        }
    }

    /// 创建指定媒体中心类型的 NFO 生成器
    pub fn for_media_center(media_center: MediaCenterType) -> Self {
        Self {
            media_center_type: media_center,
        }
    }

    /// 从爬虫数据生成并保存 NFO 文件
    pub fn generate_from_crawler(
        &self,
        crawler_data: MovieNfoCrawler,
        original_file_path: &Path,
        config: &AppConfig,
    ) -> anyhow::Result<Vec<PathBuf>> {
        let nfo = MovieNfo::for_media_center(crawler_data, self.media_center_type.clone());
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
    pub fn generate_and_save(
        &self,
        nfo: &MovieNfo,
        original_file_path: &Path,
        config: &AppConfig,
    ) -> anyhow::Result<Vec<PathBuf>> {
        let mut generated_files = Vec::new();

        match &self.media_center_type {
            MediaCenterType::All => {
                // 生成所有格式的 NFO 文件
                generated_files.push(self.save_single_format(
                    nfo,
                    original_file_path,
                    config,
                    MediaCenterType::Kodi,
                )?);
                generated_files.push(self.save_single_format(
                    nfo,
                    original_file_path,
                    config,
                    MediaCenterType::Emby,
                )?);
                generated_files.push(self.save_single_format(
                    nfo,
                    original_file_path,
                    config,
                    MediaCenterType::Jellyfin,
                )?);
            }
            specific_type => {
                // 生成指定格式的 NFO 文件
                generated_files.push(self.save_single_format(
                    nfo,
                    original_file_path,
                    config,
                    specific_type.clone(),
                )?);
            }
        }

        Ok(generated_files)
    }

    /// 保存单一格式的 NFO 文件
    fn save_single_format(
        &self,
        nfo: &MovieNfo,
        original_file_path: &Path,
        config: &AppConfig,
        format_type: MediaCenterType,
    ) -> anyhow::Result<PathBuf> {
        // 生成NFO文件路径
        let nfo_path = self.generate_nfo_path(original_file_path, nfo, config, &format_type)?;

        // 生成指定格式的XML内容
        let xml_content = self.generate_xml_content_for_type(nfo, &format_type)?;

        // 确保输出目录存在
        if let Some(parent) = nfo_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 保存文件
        fs::write(&nfo_path, xml_content)?;

        log::info!(
            "NFO文件已保存到: {} (格式: {:?})",
            nfo_path.display(),
            format_type
        );

        Ok(nfo_path)
    }

    /// 生成NFO文件的保存路径
    ///
    /// 根据媒体中心类型生成不同的文件名
    fn generate_nfo_path(
        &self,
        original_file_path: &Path,
        nfo: &MovieNfo,
        config: &AppConfig,
        format_type: &MediaCenterType,
    ) -> anyhow::Result<PathBuf> {
        let output_dir = config.get_output_dir();

        // 获取文件名基础部分（不包含扩展名）
        let file_stem = original_file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("无法获取文件名"))?;

        // 根据媒体中心类型生成不同的文件名
        let new_filename = match format_type {
            MediaCenterType::Kodi => {
                // Kodi: [文件名].nfo
                if !nfo.title.is_empty() {
                    format!("{} [{}].nfo", file_stem, self.sanitize_filename(&nfo.title))
                } else {
                    format!("{}.nfo", file_stem)
                }
            }
            MediaCenterType::Emby => {
                // Emby: [文件名].emby.nfo 或 movie.nfo (如果在单独目录中)
                if !nfo.title.is_empty() {
                    format!(
                        "{} [{}].emby.nfo",
                        file_stem,
                        self.sanitize_filename(&nfo.title)
                    )
                } else {
                    format!("{}.emby.nfo", file_stem)
                }
            }
            MediaCenterType::Jellyfin => {
                // Jellyfin: movie.nfo (推荐) 或 [文件名].jellyfin.nfo
                if !nfo.title.is_empty() {
                    format!(
                        "{} [{}].jellyfin.nfo",
                        file_stem,
                        self.sanitize_filename(&nfo.title)
                    )
                } else {
                    format!("{}.jellyfin.nfo", file_stem)
                }
            }
            MediaCenterType::All => {
                // 兼容格式
                if !nfo.title.is_empty() {
                    format!("{} [{}].nfo", file_stem, self.sanitize_filename(&nfo.title))
                } else {
                    format!("{}.nfo", file_stem)
                }
            }
        };

        let nfo_path = output_dir.join(new_filename);

        Ok(nfo_path)
    }

    /// 生成指定类型的XML内容
    fn generate_xml_content_for_type(
        &self,
        nfo: &MovieNfo,
        format_type: &MediaCenterType,
    ) -> anyhow::Result<String> {
        let xml_content = match format_type {
            MediaCenterType::Kodi => nfo.format_for_kodi(),
            MediaCenterType::Emby => nfo.format_for_emby(),
            MediaCenterType::Jellyfin => nfo.format_for_jellyfin(),
            MediaCenterType::All => nfo.format_for_kodi(), // 默认使用 Kodi 格式
        };

        if xml_content.is_empty() {
            return Err(anyhow::anyhow!("序列化NFO数据失败: 生成的XML内容为空"));
        }

        // 添加XML头部
        let mut full_xml =
            String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");

        // 添加格式特定的注释
        match format_type {
            MediaCenterType::Kodi => {
                full_xml.push_str("<!-- Generated for Kodi Media Center -->\n");
            }
            MediaCenterType::Emby => {
                full_xml.push_str("<!-- Generated for Emby Media Server -->\n");
            }
            MediaCenterType::Jellyfin => {
                full_xml.push_str("<!-- Generated for Jellyfin Media Server -->\n");
            }
            MediaCenterType::All => {
                full_xml.push_str(
                    "<!-- Universal NFO format compatible with multiple media centers -->\n",
                );
            }
        }

        full_xml.push_str(&xml_content);

        Ok(full_xml)
    }

    /// 生成XML内容 (向下兼容)
    fn generate_xml_content(&self, nfo: &MovieNfo) -> anyhow::Result<String> {
        self.generate_xml_content_for_type(nfo, &self.media_center_type)
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
        format_type: &MediaCenterType,
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

        // 添加格式特定的验证
        match format_type {
            MediaCenterType::Kodi => {
                if nfo.unique_ids.is_empty() && nfo.imdb_id.is_empty() {
                    warnings.push("Kodi推荐设置唯一ID (uniqueid)".to_string());
                }
            }
            MediaCenterType::Emby => {
                if nfo.provider_ids.is_none() && nfo.imdb_id.is_empty() {
                    warnings.push("Emby推荐设置提供商ID (providerids)".to_string());
                }
                if nfo.community_rating.is_none() {
                    warnings.push("Emby推荐设置社区评分".to_string());
                }
            }
            MediaCenterType::Jellyfin => {
                if nfo.provider_ids.is_none() && nfo.imdb_id.is_empty() {
                    warnings.push("Jellyfin推荐设置提供商ID (providerids)".to_string());
                }
                if !nfo.lock_data {
                    warnings.push("Jellyfin推荐锁定数据以防止被覆盖".to_string());
                }
            }
            MediaCenterType::All => {
                // 通用验证已在上面完成
            }
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
pub struct NfoGeneratorBuilder {
    media_center_type: MediaCenterType,
    generate_multiple_formats: bool,
    custom_xml_header: Option<String>,
}

impl NfoGeneratorBuilder {
    pub fn new() -> Self {
        Self {
            media_center_type: MediaCenterType::All,
            generate_multiple_formats: false,
            custom_xml_header: None,
        }
    }

    pub fn for_media_center(mut self, media_center: MediaCenterType) -> Self {
        self.media_center_type = media_center;
        self
    }

    pub fn generate_multiple_formats(mut self, enabled: bool) -> Self {
        self.generate_multiple_formats = enabled;
        self
    }

    pub fn with_custom_xml_header(mut self, header: String) -> Self {
        self.custom_xml_header = Some(header);
        self
    }

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
    fn test_generate_xml_for_different_media_centers() {
        let nfo = create_test_nfo();

        // 测试 Kodi 格式
        let kodi_generator = NfoGenerator::for_media_center(MediaCenterType::Kodi);
        let kodi_xml = kodi_generator
            .generate_xml_content_for_type(&nfo, &MediaCenterType::Kodi)
            .unwrap();
        assert!(kodi_xml.contains("Generated for Kodi"));

        // 测试 Emby 格式
        let emby_generator = NfoGenerator::for_media_center(MediaCenterType::Emby);
        let emby_xml = emby_generator
            .generate_xml_content_for_type(&nfo, &MediaCenterType::Emby)
            .unwrap();
        assert!(emby_xml.contains("Generated for Emby"));

        // 测试 Jellyfin 格式
        let jellyfin_generator = NfoGenerator::for_media_center(MediaCenterType::Jellyfin);
        let jellyfin_xml = jellyfin_generator
            .generate_xml_content_for_type(&nfo, &MediaCenterType::Jellyfin)
            .unwrap();
        assert!(jellyfin_xml.contains("Generated for Jellyfin"));
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
        // 由于缺少ID信息，可能会有警告

        // 测试不完整的NFO
        let incomplete_nfo = MovieNfo::default();
        let warnings = generator.validate_nfo(&incomplete_nfo);
        assert!(!warnings.is_empty());
    }

    #[test]
    fn test_nfo_generator_builder() {
        let generator = NfoGeneratorBuilder::new()
            .for_media_center(MediaCenterType::Kodi)
            .generate_multiple_formats(true)
            .build();

        // 验证构建器正确设置了媒体中心类型
        assert_eq!(generator.media_center_type, MediaCenterType::Kodi);
    }

    #[test]
    fn test_media_center_specific_validation() {
        let generator = NfoGenerator::new();
        let nfo = create_test_nfo();

        // 测试 Kodi 特定验证
        let kodi_warnings = generator.validate_nfo_for_type(&nfo, &MediaCenterType::Kodi);
        assert!(kodi_warnings.iter().any(|w| w.contains("uniqueid")));

        // 测试 Emby 特定验证
        let emby_warnings = generator.validate_nfo_for_type(&nfo, &MediaCenterType::Emby);
        assert!(emby_warnings.iter().any(|w| w.contains("providerids")));
    }
}
