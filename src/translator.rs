use std::collections::HashMap;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::time::{timeout, Duration};

use crate::config::AppConfig;
use crate::nfo::MovieNfoCrawler;

/// OpenAI API 兼容的请求结构
#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

/// OpenAI API 兼容的响应结构
#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage2,
}

#[derive(Debug, Deserialize)]
struct OpenAiMessage2 {
    content: String,
}

/// 翻译服务提供商
#[derive(Debug, Clone)]
pub enum TranslationProvider {
    OpenAI,
    Ollama,
    Custom(String), // 自定义 API 端点
}

impl std::str::FromStr for TranslationProvider {
    type Err = ();
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "openai" => TranslationProvider::OpenAI,
            "ollama" => TranslationProvider::Ollama,
            url if url.starts_with("http") => TranslationProvider::Custom(url.to_string()),
            _ => TranslationProvider::OpenAI, // 默认
        })
    }
}

impl TranslationProvider {
    pub fn get_base_url(&self) -> &str {
        match self {
            TranslationProvider::OpenAI => "https://api.openai.com/v1",
            TranslationProvider::Ollama => "http://localhost:11434/v1",
            TranslationProvider::Custom(url) => url,
        }
    }
}

/// 翻译配置
#[derive(Debug, Clone)]
pub struct TranslationConfig {
    pub provider: TranslationProvider,
    pub api_key: Option<String>,
    pub model: String,
    pub target_language: String,
    pub source_language: Option<String>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub timeout_seconds: u64,
    pub retry_count: u32,
}

impl Default for TranslationConfig {
    fn default() -> Self {
        Self {
            provider: TranslationProvider::OpenAI,
            api_key: None,
            model: "gpt-3.5-turbo".to_string(),
            target_language: "中文".to_string(),
            source_language: Some("日语".to_string()),
            max_tokens: 1000,
            temperature: 0.3,
            timeout_seconds: 30,
            retry_count: 3,
        }
    }
}

/// 翻译器
pub struct Translator {
    client: Client,
    config: TranslationConfig,
    tag_mapping: HashMap<String, String>, // 标签映射表
}

impl Translator {
    pub fn new(config: TranslationConfig) -> Result<Self> {
        let mut headers = reqwest::header::HeaderMap::new();
        
        // 添加认证头
        if let Some(api_key) = &config.api_key {
            if !api_key.is_empty() {
                let auth_value = format!("Bearer {}", api_key);
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    auth_value.parse().context("无效的 API key")?,
                );
            }
        }

        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(config.timeout_seconds))
            .user_agent("jav-tidy-rs/1.0")
            .build()
            .context("创建 HTTP 客户端失败")?;

        // 初始化标签映射表
        let tag_mapping = Self::build_tag_mapping();

        Ok(Self {
            client,
            config,
            tag_mapping,
        })
    }

    pub fn from_app_config(app_config: &AppConfig) -> Result<Self> {
        let translation_config = TranslationConfig {
            provider: app_config.get_translation_provider().parse().unwrap_or(TranslationProvider::OpenAI),
            api_key: app_config.get_translation_api_key().clone(),
            model: app_config.get_translation_model().to_string(),
            target_language: app_config.get_translation_target_language().to_string(),
            source_language: app_config.get_translation_source_language().clone(),
            max_tokens: app_config.get_translation_max_tokens(),
            temperature: app_config.get_translation_temperature(),
            timeout_seconds: app_config.get_translation_timeout(),
            retry_count: app_config.get_translation_retry_count(),
        };

        Self::new(translation_config)
    }

    /// 构建标签映射表，用于合并不同名称的相同标签
    fn build_tag_mapping() -> HashMap<String, String> {
        let mut mapping = HashMap::new();
        
        // 常见的标签映射规则
        let mappings = [
            // 演员类型
            ("女優", "演员"),
            ("女优", "演员"),
            ("AV女優", "演员"),
            ("AV女优", "演员"),
            ("actress", "演员"),
            
            // 题材类型
            ("巨乳", "大胸"),
            ("爆乳", "大胸"),
            ("美乳", "大胸"),
            ("巨尻", "翘臀"),
            ("美尻", "翘臀"),
            ("美少女", "少女"),
            ("美女", "少女"),
            
            // 情节类型
            ("中出", "内射"),
            ("中出し", "内射"),
            ("creampie", "内射"),
            ("口交", "口活"),
            ("フェラ", "口活"),
            ("blowjob", "口活"),
            
            // 服装类型
            ("制服", "校服"),
            ("セーラー服", "校服"),
            ("OL", "白领"),
            ("office lady", "白领"),
            ("メイド", "女仆"),
            ("maid", "女仆"),
            
            // 场景类型
            ("学校", "校园"),
            ("学園", "校园"),
            ("school", "校园"),
            ("家庭", "家里"),
            ("ホーム", "家里"),
            ("home", "家里"),
        ];

        for (from, to) in mappings {
            mapping.insert(from.to_string(), to.to_string());
        }

        mapping
    }

    /// 翻译文本
    pub async fn translate_text(&self, text: &str) -> Result<String> {
        if text.is_empty() {
            return Ok(String::new());
        }

        log::debug!("开始翻译文本: {}", text);

        let prompt = self.build_translation_prompt(text);
        
        let mut last_error = None;
        
        // 重试机制
        for attempt in 1..=self.config.retry_count {
            match self.call_api(&prompt).await {
                Ok(translated) => {
                    log::info!("翻译成功 (第{}次尝试): {} -> {}", attempt, text, translated);
                    return Ok(translated);
                }
                Err(e) => {
                    log::warn!("翻译失败 (第{}次尝试): {}", attempt, e);
                    last_error = Some(e);
                    
                    if attempt < self.config.retry_count {
                        tokio::time::sleep(Duration::from_secs(2u64.pow(attempt))).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("翻译失败")))
    }

    /// 构建翻译提示词
    fn build_translation_prompt(&self, text: &str) -> String {
        let source_lang = self.config.source_language.as_deref().unwrap_or("自动检测");
        let target_lang = &self.config.target_language;

        format!(
            "请将以下{}文本翻译为{}，保持原意的同时使其更易读懂。只返回翻译结果，不要任何解释：\n\n{}",
            source_lang, target_lang, text
        )
    }

    /// 调用 API
    async fn call_api(&self, prompt: &str) -> Result<String> {
        let request = OpenAiRequest {
            model: self.config.model.clone(),
            messages: vec![OpenAiMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: Some(self.config.max_tokens),
            temperature: Some(self.config.temperature),
            stream: false,
        };

        let url = format!("{}/chat/completions", self.config.provider.get_base_url());
        
        log::debug!("调用翻译 API: {}", url);

        let response = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            self.client.post(&url).json(&request).send(),
        )
        .await
        .context("API 请求超时")?
        .context("发送 API 请求失败")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("API 错误 {}: {}", status, body));
        }

        let api_response: OpenAiResponse = response
            .json()
            .await
            .context("解析 API 响应失败")?;

        if api_response.choices.is_empty() {
            return Err(anyhow::anyhow!("API 响应为空"));
        }

        let translated = api_response.choices[0].message.content.trim().to_string();
        
        if translated.is_empty() {
            return Err(anyhow::anyhow!("翻译结果为空"));
        }

        Ok(translated)
    }

    /// 翻译影片数据
    pub async fn translate_movie_data(&self, movie_data: &mut MovieNfoCrawler, config: &crate::config::AppConfig) -> Result<()> {
        log::info!("开始翻译影片数据: {}", movie_data.title);

        // 翻译标题
        if !movie_data.title.is_empty() {
            match self.translate_text(&movie_data.title).await {
                Ok(translated) => {
                    log::info!("标题翻译: {} -> {}", movie_data.title, translated);
                    movie_data.title = translated;
                }
                Err(e) => {
                    log::warn!("标题翻译失败: {}", e);
                }
            }
        }

        // 翻译原始标题（如果与标题不同）
        if let Some(original_title) = &movie_data.original_title {
            if !original_title.is_empty() && original_title != &movie_data.title {
                match self.translate_text(original_title).await {
                    Ok(translated) => {
                        log::info!("原始标题翻译: {} -> {}", original_title, translated);
                        movie_data.original_title = Some(translated);
                    }
                    Err(e) => {
                        log::warn!("原始标题翻译失败: {}", e);
                    }
                }
            }
        }

        // 翻译剧情简介
        if !movie_data.plot.is_empty() && movie_data.plot.len() > 10 {
            match self.translate_text(&movie_data.plot).await {
                Ok(translated) => {
                    log::info!("剧情简介翻译完成 ({} -> {} 字符)", movie_data.plot.len(), translated.len());
                    movie_data.plot = translated;
                }
                Err(e) => {
                    log::warn!("剧情简介翻译失败: {}", e);
                }
            }
        }

        // 翻译标语
        if !movie_data.tagline.is_empty() {
            match self.translate_text(&movie_data.tagline).await {
                Ok(translated) => {
                    log::info!("标语翻译: {} -> {}", movie_data.tagline, translated);
                    movie_data.tagline = translated;
                }
                Err(e) => {
                    log::warn!("标语翻译失败: {}", e);
                }
            }
        }

        // 翻译系列名称
        if !movie_data.series_name.is_empty() {
            match self.translate_text(&movie_data.series_name).await {
                Ok(translated) => {
                    log::info!("系列名称翻译: {} -> {}", movie_data.series_name, translated);
                    movie_data.series_name = translated;
                }
                Err(e) => {
                    log::warn!("系列名称翻译失败: {}", e);
                }
            }
        }

        // 翻译系列描述
        if !movie_data.series_overview.is_empty() {
            match self.translate_text(&movie_data.series_overview).await {
                Ok(translated) => {
                    log::info!("系列描述翻译完成");
                    movie_data.series_overview = translated;
                }
                Err(e) => {
                    log::warn!("系列描述翻译失败: {}", e);
                }
            }
        }

        // 翻译标签（如果启用）
        if config.should_translate_tags() {
            if !movie_data.tags.is_empty() {
                match self.translate_tags(&mut movie_data.tags).await {
                    Ok(_) => {
                        log::info!("标签翻译完成: {:?}", movie_data.tags);
                    }
                    Err(e) => {
                        log::warn!("标签翻译失败: {}，使用原始标签", e);
                    }
                }
            }

            // 翻译类型（如果启用）
            if !movie_data.genres.is_empty() {
                match self.translate_tags(&mut movie_data.genres).await {
                    Ok(_) => {
                        log::info!("类型翻译完成: {:?}", movie_data.genres);
                    }
                    Err(e) => {
                        log::warn!("类型翻译失败: {}，使用原始类型", e);
                    }
                }
            }
        }

        // 处理演员名称合并（始终开启）
        self.merge_actors(&mut movie_data.actors);

        // 处理基础标签合并（始终开启）
        self.merge_tags(&mut movie_data.tags);
        self.merge_tags(&mut movie_data.genres);

        // 处理AI辅助标签合并（如果启用）
        if config.is_tag_merging_enabled() {
            match self.ai_merge_tags(&mut movie_data.tags, config.get_ai_merge_threshold()).await {
                Ok(_) => {
                    log::info!("AI标签合并完成: {:?}", movie_data.tags);
                }
                Err(e) => {
                    log::warn!("AI标签合并失败: {}，使用基础合并结果", e);
                }
            }

            match self.ai_merge_tags(&mut movie_data.genres, config.get_ai_merge_threshold()).await {
                Ok(_) => {
                    log::info!("AI类型合并完成: {:?}", movie_data.genres);
                }
                Err(e) => {
                    log::warn!("AI类型合并失败: {}，使用基础合并结果", e);
                }
            }
        }

        log::info!("影片数据翻译完成: {}", movie_data.title);
        Ok(())
    }

    /// 合并相同演员名称（基础合并，始终开启）
    pub fn merge_actors(&self, actors: &mut Vec<crate::nfo::Actor>) {
        if actors.is_empty() {
            return;
        }

        let mut merged_actors = Vec::new();
        let mut seen_names = std::collections::HashSet::new();

        for actor in actors.iter() {
            let normalized_name = self.normalize_actor_name(&actor.name);
            
            if !seen_names.contains(&normalized_name) {
                merged_actors.push(actor.clone());
                seen_names.insert(normalized_name);
            }
        }

        if merged_actors.len() != actors.len() {
            log::info!("演员合并: {} -> {} 个演员", actors.len(), merged_actors.len());
            *actors = merged_actors;
        }
    }

    /// 标准化演员名称
    fn normalize_actor_name(&self, name: &str) -> String {
        name.trim()
            .replace(" ", "")
            .replace("　", "") // 全角空格
            .replace("・", "")
            .replace(".", "")
            .replace("-", "")
            .to_lowercase()
    }

    /// 合并相似标签（基础合并，始终开启）
    pub fn merge_tags(&self, tags: &mut Vec<String>) {
        if tags.is_empty() {
            return;
        }

        let mut merged_tags = Vec::new();
        let mut used_canonical = std::collections::HashSet::new();

        for tag in tags.iter() {
            let canonical = self.tag_mapping
                .get(tag)
                .cloned()
                .unwrap_or_else(|| tag.clone());

            if !used_canonical.contains(&canonical) {
                merged_tags.push(canonical.clone());
                used_canonical.insert(canonical);
            }
        }

        // 按长度和字母顺序排序
        merged_tags.sort_by(|a, b| {
            a.len().cmp(&b.len()).then_with(|| a.cmp(b))
        });

        if merged_tags != *tags {
            log::info!("标签合并: {:?} -> {:?}", tags, merged_tags);
            *tags = merged_tags;
        }
    }

    /// 翻译标签列表
    pub async fn translate_tags(&self, tags: &mut Vec<String>) -> Result<()> {
        if tags.is_empty() {
            return Ok(());
        }

        log::info!("开始翻译标签: {:?}", tags);

        let mut translated_tags = Vec::new();
        
        for tag in tags.iter() {
            if tag.is_empty() {
                continue;
            }

            // 如果标签很短或已经是中文，可能不需要翻译
            if tag.chars().count() <= 2 || self.is_chinese_text(tag) {
                translated_tags.push(tag.clone());
                continue;
            }

            match self.translate_text(tag).await {
                Ok(translated) => {
                    log::debug!("标签翻译: {} -> {}", tag, translated);
                    translated_tags.push(translated);
                }
                Err(e) => {
                    log::warn!("标签 '{}' 翻译失败: {}，保留原文", tag, e);
                    translated_tags.push(tag.clone());
                }
            }

            // 为了避免 API 限制，标签之间稍作延迟
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // 合并相似标签
        self.merge_tags(&mut translated_tags);

        if translated_tags != *tags {
            log::info!("标签翻译完成: {:?} -> {:?}", tags, translated_tags);
            *tags = translated_tags;
        }

        Ok(())
    }

    /// AI辅助标签合并（智能识别相似标签）
    pub async fn ai_merge_tags(&self, tags: &mut Vec<String>, threshold: f32) -> Result<()> {
        if tags.len() <= 1 {
            return Ok(());
        }

        log::debug!("开始AI标签合并，原始标签: {:?}", tags);

        // 构建AI提示词
        let prompt = self.build_tag_merge_prompt(tags, threshold);
        
        match self.call_api(&prompt).await {
            Ok(response) => {
                // 解析AI响应
                match self.parse_tag_merge_response(&response) {
                    Ok(merged_tags) => {
                        if merged_tags != *tags && !merged_tags.is_empty() {
                            log::info!("AI标签合并: {:?} -> {:?}", tags, merged_tags);
                            *tags = merged_tags;
                        }
                        Ok(())
                    }
                    Err(e) => {
                        log::warn!("AI标签合并响应解析失败: {}", e);
                        Err(e)
                    }
                }
            }
            Err(e) => {
                log::warn!("AI标签合并API调用失败: {}", e);
                Err(e)
            }
        }
    }

    /// 构建标签合并的AI提示词
    fn build_tag_merge_prompt(&self, tags: &[String], threshold: f32) -> String {
        let tags_str = tags.join("、");
        
        format!(
            "请分析以下标签并合并意思相同或相似的标签（相似度阈值: {:.1}）。\
            只保留最具代表性和常用的标签名称，删除重复或过于相似的标签。\
            请直接返回合并后的标签列表，用逗号分隔，不要任何解释：\n\n标签列表: {}",
            threshold, tags_str
        )
    }

    /// 解析AI标签合并响应
    fn parse_tag_merge_response(&self, response: &str) -> Result<Vec<String>> {
        let cleaned_response = response.trim()
            .replace("：", ":")
            .replace("，", ",");

        // 尝试不同的解析方式
        let merged_tags: Vec<String> = if cleaned_response.contains(",") {
            // 逗号分隔
            cleaned_response.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else if cleaned_response.contains("、") {
            // 中文顿号分隔
            cleaned_response.split('、')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else if cleaned_response.contains(" ") {
            // 空格分隔
            cleaned_response.split_whitespace()
                .map(|s| s.to_string())
                .collect()
        } else {
            // 单个标签或无法解析
            vec![cleaned_response]
        };

        if merged_tags.is_empty() {
            return Err(anyhow::anyhow!("AI返回的合并结果为空"));
        }

        Ok(merged_tags)
    }

    /// 检测是否为中文文本
    fn is_chinese_text(&self, text: &str) -> bool {
        let chinese_chars = text.chars()
            .filter(|c| {
                let code = *c as u32;
                // Unicode 中文字符范围
                (0x4E00..=0x9FFF).contains(&code) || // CJK 统一表意文字
                (0x3400..=0x4DBF).contains(&code) || // CJK 扩展 A
                (0x20000..=0x2A6DF).contains(&code) || // CJK 扩展 B
                (0x2A700..=0x2B73F).contains(&code) || // CJK 扩展 C
                (0x2B740..=0x2B81F).contains(&code) || // CJK 扩展 D
                (0x2B820..=0x2CEAF).contains(&code) // CJK 扩展 E
            })
            .count();

        let total_chars = text.chars().count();
        
        // 如果超过50%是中文字符，则认为是中文文本
        total_chars > 0 && (chinese_chars as f64 / total_chars as f64) > 0.5
    }

    /// 测试翻译服务连接
    pub async fn test_connection(&self) -> Result<()> {
        log::info!("测试翻译服务连接...");
        
        let test_text = "テスト";
        match self.translate_text(test_text).await {
            Ok(result) => {
                log::info!("翻译服务连接正常，测试翻译: {} -> {}", test_text, result);
                Ok(())
            }
            Err(e) => {
                log::error!("翻译服务连接失败: {}", e);
                Err(e)
            }
        }
    }
}

impl Default for Translator {
    fn default() -> Self {
        Self::new(TranslationConfig::default()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_mapping() {
        let translator = Translator::default();
        
        let mut tags = vec![
            "女優".to_string(),
            "巨乳".to_string(),
            "美乳".to_string(),
            "中出".to_string(),
        ];
        
        translator.merge_tags(&mut tags);
        
        // 验证标签被正确合并
        assert!(tags.contains(&"演员".to_string()));
        assert!(tags.contains(&"大胸".to_string()));
        assert!(tags.contains(&"内射".to_string()));
        
        // 验证重复标签被去除
        assert_eq!(tags.iter().filter(|t| *t == "大胸").count(), 1);
    }

    #[test]
    fn test_chinese_detection() {
        let translator = Translator::default();
        
        assert!(translator.is_chinese_text("这是中文"));
        assert!(translator.is_chinese_text("中文测试"));
        assert!(!translator.is_chinese_text("english text"));
        assert!(!translator.is_chinese_text("テスト"));
        assert!(translator.is_chinese_text("中英混合 mixed"));
    }

    #[test]
    fn test_translation_provider() {
        assert!(matches!(
            "openai".parse().unwrap(),
            TranslationProvider::OpenAI
        ));
        
        assert!(matches!(
            "ollama".parse().unwrap(),
            TranslationProvider::Ollama
        ));
        
        if let TranslationProvider::Custom(url) = "http://localhost:8080".parse().unwrap() {
            assert_eq!(url, "http://localhost:8080");
        } else {
            panic!("Custom provider not parsed correctly");
        }
    }
}