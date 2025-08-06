use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use reqwest::Client;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::nfo::MovieNfoCrawler;
use crate::config::AppConfig;

/// 媒体中心图片类型
#[derive(Debug, Clone)]
pub enum ImageType {
    /// 主海报/封面图
    Poster,
    /// 背景图/剧照
    Fanart,
    /// 缩略图
    Thumb,
    /// 预览图集
    Preview,
    /// 演员头像
    ActorThumb(String), // 演员名称
}

/// 媒体中心图片命名规则
#[derive(Debug, Clone)]
pub struct ImageNamingRule {
    pub filename: String,
    pub description: String,
}

/// 图片管理器
pub struct ImageManager {
    client: Client,
}

impl ImageManager {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("jav-tidy-rs/1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self { client }
    }

    /// 获取 Emby/Jellyfin 图片命名规则
    /// 参考: https://emby.media/support/articles/Movie-Naming.html
    pub fn get_emby_naming_rules(movie_id: &str) -> Vec<(ImageType, ImageNamingRule)> {
        vec![
            // 主海报
            (ImageType::Poster, ImageNamingRule {
                filename: format!("{}.jpg", movie_id),
                description: "主海报 (Emby/Jellyfin 自动识别)".to_string(),
            }),
            (ImageType::Poster, ImageNamingRule {
                filename: "poster.jpg".to_string(),
                description: "主海报 (通用命名)".to_string(),
            }),
            (ImageType::Poster, ImageNamingRule {
                filename: "cover.jpg".to_string(),
                description: "封面图 (备用命名)".to_string(),
            }),
            
            // 背景图/剧照
            (ImageType::Fanart, ImageNamingRule {
                filename: "backdrop.jpg".to_string(),
                description: "背景图 (Emby/Jellyfin 推荐)".to_string(),
            }),
            (ImageType::Fanart, ImageNamingRule {
                filename: "fanart.jpg".to_string(),
                description: "背景图 (Kodi 兼容)".to_string(),
            }),
            (ImageType::Fanart, ImageNamingRule {
                filename: "background.jpg".to_string(),
                description: "背景图 (备用命名)".to_string(),
            }),
            
            // 缩略图
            (ImageType::Thumb, ImageNamingRule {
                filename: "thumb.jpg".to_string(),
                description: "缩略图".to_string(),
            }),
            (ImageType::Thumb, ImageNamingRule {
                filename: "landscape.jpg".to_string(),
                description: "横向缩略图".to_string(),
            }),
        ]
    }

    /// 获取 Kodi 图片命名规则
    /// 参考: https://kodi.wiki/view/Movie_information_folder
    pub fn get_kodi_naming_rules(movie_id: &str) -> Vec<(ImageType, ImageNamingRule)> {
        vec![
            // Kodi 海报命名
            (ImageType::Poster, ImageNamingRule {
                filename: format!("{}-poster.jpg", movie_id),
                description: "Kodi 海报命名".to_string(),
            }),
            (ImageType::Poster, ImageNamingRule {
                filename: "poster.jpg".to_string(),
                description: "Kodi 通用海报".to_string(),
            }),
            
            // Kodi 背景图命名
            (ImageType::Fanart, ImageNamingRule {
                filename: format!("{}-fanart.jpg", movie_id),
                description: "Kodi 背景图命名".to_string(),
            }),
            (ImageType::Fanart, ImageNamingRule {
                filename: "fanart.jpg".to_string(),
                description: "Kodi 通用背景图".to_string(),
            }),
            
            // Kodi 缩略图
            (ImageType::Thumb, ImageNamingRule {
                filename: format!("{}-thumb.jpg", movie_id),
                description: "Kodi 缩略图命名".to_string(),
            }),
            (ImageType::Thumb, ImageNamingRule {
                filename: "thumb.jpg".to_string(),
                description: "Kodi 通用缩略图".to_string(),
            }),
        ]
    }

    /// 获取 Plex 图片命名规则
    /// 参考: https://support.plex.tv/articles/200220677-local-media-assets-movies/
    pub fn get_plex_naming_rules(movie_id: &str) -> Vec<(ImageType, ImageNamingRule)> {
        vec![
            // Plex 海报命名
            (ImageType::Poster, ImageNamingRule {
                filename: format!("{}.jpg", movie_id),
                description: "Plex 主海报".to_string(),
            }),
            (ImageType::Poster, ImageNamingRule {
                filename: "poster.jpg".to_string(),
                description: "Plex 文件夹海报".to_string(),
            }),
            
            // Plex 背景图命名  
            (ImageType::Fanart, ImageNamingRule {
                filename: format!("{}.fanart.jpg", movie_id),
                description: "Plex 背景图".to_string(),
            }),
            (ImageType::Fanart, ImageNamingRule {
                filename: "art.jpg".to_string(),
                description: "Plex 文件夹背景图".to_string(),
            }),
        ]
    }

    /// 根据配置获取所有适用的命名规则
    pub fn get_naming_rules(movie_id: &str, config: &AppConfig) -> Vec<(ImageType, ImageNamingRule)> {
        let mut rules = Vec::new();
        
        // 默认使用 Emby/Jellyfin 规则（最通用）
        rules.extend(Self::get_emby_naming_rules(movie_id));
        
        // 如果配置了特定平台，添加对应规则
        match config.get_media_center_type() {
            "kodi" => rules.extend(Self::get_kodi_naming_rules(movie_id)),
            "plex" => rules.extend(Self::get_plex_naming_rules(movie_id)),
            _ => {}, // 默认已经包含 Emby/Jellyfin（通用格式）
        }
        
        // 去重（保留第一个匹配的规则）
        let mut seen = std::collections::HashSet::new();
        rules.retain(|(_, rule)| seen.insert(rule.filename.clone()));
        
        rules
    }

    /// 下载图片到指定路径
    pub async fn download_image(&self, url: &str, output_path: &Path) -> Result<()> {
        if url.is_empty() {
            return Err(anyhow::anyhow!("图片 URL 为空"));
        }

        log::debug!("开始下载图片: {} -> {}", url, output_path.display());

        // 创建输出目录
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).await
                .with_context(|| format!("创建目录失败: {}", parent.display()))?;
        }

        // 下载图片
        let response = self.client.get(url)
            .send()
            .await
            .with_context(|| format!("请求图片失败: {}", url))?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP 错误: {}", response.status()));
        }

        let bytes = response.bytes()
            .await
            .with_context(|| format!("读取图片数据失败: {}", url))?;

        // 写入文件
        let mut file = fs::File::create(output_path)
            .await
            .with_context(|| format!("创建文件失败: {}", output_path.display()))?;

        file.write_all(&bytes)
            .await
            .with_context(|| format!("写入文件失败: {}", output_path.display()))?;

        log::info!("图片下载成功: {} ({} bytes)", output_path.display(), bytes.len());
        Ok(())
    }

    /// 为影片下载所有图片
    pub async fn download_movie_images(
        &self,
        movie_data: &MovieNfoCrawler,
        output_dir: &Path,
        movie_id: &str,
        config: &AppConfig,
    ) -> Result<Vec<PathBuf>> {
        let mut downloaded_files = Vec::new();
        let naming_rules = Self::get_naming_rules(movie_id, config);

        log::info!("开始下载影片 {} 的图片，输出目录: {}", movie_id, output_dir.display());

        // 下载海报
        if !movie_data.posters.is_empty() {
            let poster_url = &movie_data.posters[0]; // 使用第一个海报
            for (image_type, rule) in &naming_rules {
                if matches!(image_type, ImageType::Poster) {
                    let output_path = output_dir.join(&rule.filename);
                    if let Err(e) = self.download_image(poster_url, &output_path).await {
                        log::warn!("下载海报失败 {}: {}", rule.filename, e);
                    } else {
                        downloaded_files.push(output_path);
                        break; // 只下载第一个成功的海报
                    }
                }
            }
        }

        // 下载背景图/剧照
        if !movie_data.fanarts.is_empty() {
            let fanart_url = &movie_data.fanarts[0]; // 使用第一个背景图
            for (image_type, rule) in &naming_rules {
                if matches!(image_type, ImageType::Fanart) {
                    let output_path = output_dir.join(&rule.filename);
                    if let Err(e) = self.download_image(fanart_url, &output_path).await {
                        log::warn!("下载背景图失败 {}: {}", rule.filename, e);
                    } else {
                        downloaded_files.push(output_path);
                        break; // 只下载第一个成功的背景图
                    }
                }
            }
        }

        // 下载缩略图
        if !movie_data.thumbs.is_empty() {
            let thumb_url = &movie_data.thumbs[0]; // 使用第一个缩略图
            for (image_type, rule) in &naming_rules {
                if matches!(image_type, ImageType::Thumb) {
                    let output_path = output_dir.join(&rule.filename);
                    if let Err(e) = self.download_image(thumb_url, &output_path).await {
                        log::warn!("下载缩略图失败 {}: {}", rule.filename, e);
                    } else {
                        downloaded_files.push(output_path);
                        break; // 只下载第一个成功的缩略图
                    }
                }
            }
        }

        // 下载预览图集（可选）
        if config.should_download_preview_images() && !movie_data.preview_images.is_empty() {
            for (i, preview_url) in movie_data.preview_images.iter().enumerate().take(10) {
                let filename = format!("preview_{:02}.jpg", i + 1);
                let output_path = output_dir.join(&filename);
                if let Err(e) = self.download_image(preview_url, &output_path).await {
                    log::warn!("下载预览图失败 {}: {}", filename, e);
                } else {
                    downloaded_files.push(output_path);
                }
            }
        }

        log::info!("影片 {} 图片下载完成，共下载 {} 个文件", movie_id, downloaded_files.len());
        Ok(downloaded_files)
    }

    /// 检查图片是否已存在且有效
    pub async fn is_image_valid(&self, path: &Path) -> bool {
        if !path.exists() {
            return false;
        }

        // 检查文件大小（至少1KB）
        if let Ok(metadata) = fs::metadata(path).await {
            if metadata.len() < 1024 {
                return false;
            }
        } else {
            return false;
        }

        // 简单检查文件头是否为图片格式
        if let Ok(mut file) = fs::File::open(path).await {
            let mut buffer = [0; 4];
            if let Ok(_) = tokio::io::AsyncReadExt::read_exact(&mut file, &mut buffer).await {
                // 检查常见图片格式的文件头
                return match buffer {
                    [0xFF, 0xD8, 0xFF, _] => true, // JPEG
                    [0x89, 0x50, 0x4E, 0x47] => true, // PNG
                    [0x47, 0x49, 0x46, 0x38] => true, // GIF
                    [0x42, 0x4D, _, _] => true, // BMP
                    _ => false,
                };
            }
        }

        false
    }

    /// 清理无效或损坏的图片文件
    pub async fn cleanup_invalid_images(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut removed_files = Vec::new();
        
        if !dir.exists() {
            return Ok(removed_files);
        }

        let mut entries = fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if matches!(ext.to_str(), Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("bmp")) {
                        if !self.is_image_valid(&path).await {
                            log::warn!("发现无效图片文件，准备删除: {}", path.display());
                            if let Err(e) = fs::remove_file(&path).await {
                                log::error!("删除无效图片失败: {}", e);
                            } else {
                                removed_files.push(path);
                            }
                        }
                    }
                }
            }
        }

        if !removed_files.is_empty() {
            log::info!("清理了 {} 个无效图片文件", removed_files.len());
        }

        Ok(removed_files)
    }
}

impl Default for ImageManager {
    fn default() -> Self {
        Self::new()
    }
}