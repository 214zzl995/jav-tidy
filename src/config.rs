use std::path::{Path, PathBuf};

use config::Config;
use serde::Deserialize;

/// 图片下载配置
#[derive(Debug, Deserialize, Clone)]
pub struct ImageConfig {
    /// 是否下载图片（海报、背景图等）
    #[serde(default = "default_download_images")]
    pub download_images: bool,
    /// 是否下载预览图集
    #[serde(default = "default_download_preview_images")]
    pub download_preview_images: bool,
    /// 媒体中心类型 (emby/jellyfin/kodi/plex/universal)
    #[serde(default = "default_media_center_type")]
    pub media_center_type: String,
    /// 图片下载超时时间（秒）
    #[serde(default = "default_image_download_timeout")]
    pub timeout: u64,
}

/// 翻译服务配置
#[derive(Debug, Deserialize, Clone)]
pub struct TranslationConfig {
    /// 是否启用翻译功能
    #[serde(default = "default_enable_translation")]
    pub enabled: bool,
    /// 翻译服务提供商 (openai/ollama/自定义URL)
    #[serde(default = "default_translation_provider")]
    pub provider: String,
    /// 翻译服务 API Key
    #[serde(default = "default_translation_api_key")]
    pub api_key: Option<String>,
    /// 翻译模型名称
    #[serde(default = "default_translation_model")]
    pub model: String,
    /// 目标语言
    #[serde(default = "default_translation_target_language")]
    pub target_language: String,
    /// 源语言（可选，留空为自动检测）
    #[serde(default = "default_translation_source_language")]
    pub source_language: Option<String>,
    /// 翻译最大令牌数
    #[serde(default = "default_translation_max_tokens")]
    pub max_tokens: u32,
    /// 翻译温度参数 (0.0-2.0)
    #[serde(default = "default_translation_temperature")]
    pub temperature: f32,
    /// 翻译请求超时时间（秒）
    #[serde(default = "default_translation_timeout")]
    pub timeout: u64,
    /// 翻译重试次数
    #[serde(default = "default_translation_retry_count")]
    pub retry_count: u32,
}

/// 标签处理配置
#[derive(Debug, Deserialize, Clone)]
pub struct TagConfig {
    /// 是否翻译标签
    #[serde(default = "default_translate_tags")]
    pub translate: bool,
    /// 是否启用AI辅助标签合并
    #[serde(default = "default_enable_ai_tag_merging")]
    pub ai_merge: bool,
    /// AI标签合并的最小相似度阈值 (0.0-1.0)
    #[serde(default = "default_ai_merge_threshold")]
    pub ai_merge_threshold: f32,
}

/// 字幕文件配置
#[derive(Debug, Deserialize, Clone)]
pub struct SubtitleConfig {
    /// 是否同时处理字幕文件
    #[serde(default = "default_migrate_subtitles")]
    pub migrate: bool,
    /// 支持的字幕文件扩展名
    #[serde(default = "default_subtitle_extensions")]
    pub extensions: Vec<String>,
    /// 字幕文件语言标识（ISO 639-1 + ISO 3166-1格式）
    #[serde(default = "default_subtitle_language")]
    pub language: String,
}

/// 文件命名配置
#[derive(Debug, Deserialize, Clone)]
pub struct NamingConfig {
    /// 文件命名模板
    #[serde(default = "default_file_naming_template")]
    pub template: String,
    /// 多演员处理策略
    #[serde(default = "default_multi_actor_strategy")]
    pub multi_actor_strategy: String,
    /// 是否将文件名转为小写
    pub capital: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    // 基础配置
    pub migrate_files: Vec<String>,
    ignored_id_pattern: Vec<String>,
    pub input_dir: PathBuf,
    output_dir: PathBuf,
    #[allow(dead_code)] // 预留给未来的并发控制功能
    pub thread_limit: usize,
    pub template_priority: Vec<String>,
    #[serde(default = "default_maximum_fetch_count")]
    pub maximum_fetch_count: usize,

    // 分组配置
    /// 图片下载相关配置
    #[serde(default)]
    pub image: ImageConfig,
    /// 翻译服务相关配置
    #[serde(default)]
    pub translation: TranslationConfig,
    /// 标签处理相关配置
    #[serde(default)]
    pub tag: TagConfig,
    /// 字幕文件相关配置
    #[serde(default)]
    pub subtitle: SubtitleConfig,
    /// 文件命名相关配置
    #[serde(default)]
    pub naming: NamingConfig,

    // 兼容性字段（保持向后兼容）
    #[serde(skip_serializing_if = "Option::is_none")]
    migrate_subtitles: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    capital: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_naming_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    multi_actor_strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subtitle_extensions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subtitle_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    download_images: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    download_preview_images: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    media_center_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_download_timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_translation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    translation_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    translation_api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    translation_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    translation_target_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    translation_source_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    translation_max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    translation_temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    translation_timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    translation_retry_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    translate_tags: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_tag_merging: Option<bool>,
}

fn default_maximum_fetch_count() -> usize {
    3
}

/// 默认文件命名模板：系列名/影片标题 (年份)
fn default_file_naming_template() -> String {
    "$series$/$title$ ($year$)".to_string()
}

/// 默认多演员处理策略：创建符号链接
fn default_multi_actor_strategy() -> String {
    "symlink".to_string()
}

/// 默认字幕文件扩展名：Emby/Jellyfin/Kodi 常用格式
fn default_subtitle_extensions() -> Vec<String> {
    vec![
        "srt".to_string(),    // SubRip (最常用)
        "ass".to_string(),    // Advanced SubStation Alpha
        "ssa".to_string(),    // SubStation Alpha
        "vtt".to_string(),    // WebVTT
        "sub".to_string(),    // MicroDVD
        "idx".to_string(),    // DVD 字幕索引
        "sup".to_string(),    // BD 字幕
    ]
}

/// 默认字幕语言：中文简体
fn default_subtitle_language() -> String {
    "zh-CN".to_string()
}

/// 默认图片下载：启用
fn default_download_images() -> bool {
    true
}

/// 默认预览图下载：禁用（节省带宽）
fn default_download_preview_images() -> bool {
    false
}

/// 默认媒体中心：通用格式（兼容所有平台）
fn default_media_center_type() -> String {
    "universal".to_string()
}

/// 默认图片下载超时：30秒
fn default_image_download_timeout() -> u64 {
    30
}

/// 默认翻译功能：禁用
fn default_enable_translation() -> bool {
    false
}

/// 默认翻译服务提供商：OpenAI
fn default_translation_provider() -> String {
    "openai".to_string()
}

/// 默认翻译 API Key：无
fn default_translation_api_key() -> Option<String> {
    None
}

/// 默认翻译模型：GPT-3.5 Turbo
fn default_translation_model() -> String {
    "gpt-3.5-turbo".to_string()
}

/// 默认目标语言：中文
fn default_translation_target_language() -> String {
    "中文".to_string()
}

/// 默认源语言：自动检测
fn default_translation_source_language() -> Option<String> {
    Some("日语".to_string())
}

/// 默认翻译最大令牌数：1000
fn default_translation_max_tokens() -> u32 {
    1000
}

/// 默认翻译温度：0.3
fn default_translation_temperature() -> f32 {
    0.3
}

/// 默认翻译超时：30秒
fn default_translation_timeout() -> u64 {
    30
}

/// 默认翻译重试次数：3次
fn default_translation_retry_count() -> u32 {
    3
}

/// 默认翻译标签：启用
fn default_translate_tags() -> bool {
    true
}

/// 默认标签合并：启用
fn default_enable_tag_merging() -> bool {
    true
}

/// 默认字幕迁移：启用
fn default_migrate_subtitles() -> bool {
    true
}

/// 默认AI标签合并：禁用
fn default_enable_ai_tag_merging() -> bool {
    false
}

/// 默认AI合并阈值：0.8
fn default_ai_merge_threshold() -> f32 {
    0.8
}

// 为新的配置结构实现默认值
impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            download_images: default_download_images(),
            download_preview_images: default_download_preview_images(),
            media_center_type: default_media_center_type(),
            timeout: default_image_download_timeout(),
        }
    }
}

impl Default for TranslationConfig {
    fn default() -> Self {
        Self {
            enabled: default_enable_translation(),
            provider: default_translation_provider(),
            api_key: default_translation_api_key(),
            model: default_translation_model(),
            target_language: default_translation_target_language(),
            source_language: default_translation_source_language(),
            max_tokens: default_translation_max_tokens(),
            temperature: default_translation_temperature(),
            timeout: default_translation_timeout(),
            retry_count: default_translation_retry_count(),
        }
    }
}

impl Default for TagConfig {
    fn default() -> Self {
        Self {
            translate: default_translate_tags(),
            ai_merge: default_enable_ai_tag_merging(),
            ai_merge_threshold: default_ai_merge_threshold(),
        }
    }
}

impl Default for SubtitleConfig {
    fn default() -> Self {
        Self {
            migrate: default_migrate_subtitles(),
            extensions: default_subtitle_extensions(),
            language: default_subtitle_language(),
        }
    }
}

impl Default for NamingConfig {
    fn default() -> Self {
        Self {
            template: default_file_naming_template(),
            multi_actor_strategy: default_multi_actor_strategy(),
            capital: false, // 默认不转小写
        }
    }
}

impl AppConfig {
    pub fn new(config_file: &Path) -> anyhow::Result<Self> {
        let settings = Config::builder()
            .add_source(config::File::from(config_file))
            .add_source(config::Environment::with_prefix("JAVTIDY"))
            .build()
            .unwrap();

        let mut config: AppConfig = settings.try_deserialize()?;

        // 处理向后兼容性
        config.apply_legacy_fields();

        Ok(config)
    }

    /// 应用旧版本配置字段到新结构中（向后兼容）
    fn apply_legacy_fields(&mut self) {
        // 字幕配置兼容
        if let Some(migrate) = self.migrate_subtitles {
            self.subtitle.migrate = migrate;
        }
        if let Some(ref extensions) = self.subtitle_extensions {
            self.subtitle.extensions = extensions.clone();
        }
        if let Some(ref language) = self.subtitle_language {
            self.subtitle.language = language.clone();
        }

        // 命名配置兼容
        if let Some(capital) = self.capital {
            self.naming.capital = capital;
        }
        if let Some(ref template) = self.file_naming_template {
            self.naming.template = template.clone();
        }
        if let Some(ref strategy) = self.multi_actor_strategy {
            self.naming.multi_actor_strategy = strategy.clone();
        }

        // 图片配置兼容
        if let Some(download) = self.download_images {
            self.image.download_images = download;
        }
        if let Some(download_preview) = self.download_preview_images {
            self.image.download_preview_images = download_preview;
        }
        if let Some(ref media_type) = self.media_center_type {
            self.image.media_center_type = media_type.clone();
        }
        if let Some(timeout) = self.image_download_timeout {
            self.image.timeout = timeout;
        }

        // 翻译配置兼容
        if let Some(enabled) = self.enable_translation {
            self.translation.enabled = enabled;
        }
        if let Some(ref provider) = self.translation_provider {
            self.translation.provider = provider.clone();
        }
        if let Some(ref api_key) = self.translation_api_key {
            self.translation.api_key = Some(api_key.clone());
        }
        if let Some(ref model) = self.translation_model {
            self.translation.model = model.clone();
        }
        if let Some(ref target_lang) = self.translation_target_language {
            self.translation.target_language = target_lang.clone();
        }
        if let Some(ref source_lang) = self.translation_source_language {
            self.translation.source_language = Some(source_lang.clone());
        }
        if let Some(max_tokens) = self.translation_max_tokens {
            self.translation.max_tokens = max_tokens;
        }
        if let Some(temperature) = self.translation_temperature {
            self.translation.temperature = temperature;
        }
        if let Some(timeout) = self.translation_timeout {
            self.translation.timeout = timeout;
        }
        if let Some(retry_count) = self.translation_retry_count {
            self.translation.retry_count = retry_count;
        }

        // 标签配置兼容
        if let Some(translate_tags) = self.translate_tags {
            self.tag.translate = translate_tags;
        }
        if let Some(tag_merging) = self.enable_tag_merging {
            // 旧版的标签合并映射到基础合并，AI合并保持默认值
            if !tag_merging {
                self.tag.ai_merge = false;
            }
        }
    }

    pub fn get_migrate_files_ext(&self) -> &'static [&'static str] {
        let leaked_strs: Vec<&'static str> = self
            .migrate_files
            .clone()
            .into_iter()
            .map(|s| Box::leak(s.into_boxed_str()) as &'static str)
            .collect();

        Box::leak(leaked_strs.into_boxed_slice())
    }

    pub fn is_useing_template(&self, template: &str) -> bool {
        self.template_priority.iter().any(|t| t == template)
    }

    pub fn get_template_index(&self, template: &str) -> Option<usize> {
        self.template_priority.iter().position(|t| t == template)
    }

    /// 获取要忽略的ID模式列表
    pub fn get_ignored_id_pattern(&self) -> &[String] {
        &self.ignored_id_pattern
    }

    /// 获取输出目录
    pub fn get_output_dir(&self) -> &std::path::Path {
        &self.output_dir
    }

    /// 获取是否需要迁移字幕文件的配置
    pub fn migrate_subtitles(&self) -> bool {
        self.subtitle.migrate
    }

    /// 获取文件命名模板
    pub fn get_file_naming_template(&self) -> &str {
        &self.naming.template
    }

    /// 获取多演员处理策略
    pub fn get_multi_actor_strategy(&self) -> &str {
        &self.naming.multi_actor_strategy
    }

    /// 获取支持的字幕文件扩展名
    pub fn get_subtitle_extensions(&self) -> &[String] {
        &self.subtitle.extensions
    }

    /// 获取字幕文件语言标识
    pub fn get_subtitle_language(&self) -> &str {
        &self.subtitle.language
    }

    /// 获取是否下载图片的配置
    pub fn should_download_images(&self) -> bool {
        self.image.download_images
    }

    /// 获取是否下载预览图集的配置
    pub fn should_download_preview_images(&self) -> bool {
        self.image.download_preview_images
    }

    /// 获取媒体中心类型
    pub fn get_media_center_type(&self) -> &str {
        &self.image.media_center_type
    }

    /// 获取图片下载超时时间（秒）
    pub fn get_image_download_timeout(&self) -> u64 {
        self.image.timeout
    }

    /// 获取是否启用翻译功能
    pub fn is_translation_enabled(&self) -> bool {
        self.translation.enabled
    }

    /// 获取翻译服务提供商
    pub fn get_translation_provider(&self) -> &str {
        &self.translation.provider
    }

    /// 获取翻译 API Key
    pub fn get_translation_api_key(&self) -> &Option<String> {
        &self.translation.api_key
    }

    /// 获取翻译模型
    pub fn get_translation_model(&self) -> &str {
        &self.translation.model
    }

    /// 获取翻译目标语言
    pub fn get_translation_target_language(&self) -> &str {
        &self.translation.target_language
    }

    /// 获取翻译源语言
    pub fn get_translation_source_language(&self) -> &Option<String> {
        &self.translation.source_language
    }

    /// 获取翻译最大令牌数
    pub fn get_translation_max_tokens(&self) -> u32 {
        self.translation.max_tokens
    }

    /// 获取翻译温度参数
    pub fn get_translation_temperature(&self) -> f32 {
        self.translation.temperature
    }

    /// 获取翻译超时时间
    pub fn get_translation_timeout(&self) -> u64 {
        self.translation.timeout
    }

    /// 获取翻译重试次数
    pub fn get_translation_retry_count(&self) -> u32 {
        self.translation.retry_count
    }

    /// 获取是否翻译标签
    pub fn should_translate_tags(&self) -> bool {
        self.tag.translate
    }

    /// 获取是否启用标签合并（基础合并始终开启，这里指AI合并）
    pub fn is_tag_merging_enabled(&self) -> bool {
        self.tag.ai_merge
    }

    /// 获取是否将文件名转为小写
    pub fn is_capital(&self) -> bool {
        self.naming.capital
    }

    /// 获取AI标签合并阈值
    pub fn get_ai_merge_threshold(&self) -> f32 {
        self.tag.ai_merge_threshold
    }
}
