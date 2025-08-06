use crawler_template::Crawler;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// 媒体中心类型枚举 - 基于 NFO 研究，所有平台都使用 Kodi 标准
#[derive(Debug, Clone, PartialEq)]
pub enum MediaCenterType {
    Universal, // 通用格式，兼容 Kodi/Emby/Jellyfin
}

/// 演员信息结构 - 简化为通用字段
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Actor {
    #[serde(rename = "name", default, skip_serializing_if = "String::is_empty")]
    pub name: String, // 演员姓名
    #[serde(rename = "role", default, skip_serializing_if = "String::is_empty")]
    pub role: String, // 演员扮演的角色名
    #[serde(rename = "thumb", default, skip_serializing_if = "String::is_empty")]
    pub thumb: String, // 演员头像的 URL 或路径
    #[serde(rename = "order", default, skip_serializing_if = "Option::is_none")]
    pub order: Option<u32>, // 演员排序
}

impl FromStr for Actor {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Actor {
            name: s.to_string(),
            ..Default::default()
        })
    }
}

/// 电影系列/集合信息
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct MovieSet {
    #[serde(rename = "name", default, skip_serializing_if = "String::is_empty")]
    pub name: String, // 系列/集合的名称
    #[serde(rename = "overview", default, skip_serializing_if = "String::is_empty")]
    pub overview: String, // 系列/集合的简介
}

/// 唯一标识符结构 - 基于 NFO 研究，支持现代化 uniqueid 格式
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UniqueId {
    #[serde(rename = "@type")]
    pub id_type: String, // 标识符类型 (imdb, tmdb, tvdb 等)
    #[serde(rename = "@default", skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>, // 是否为默认标识符
    #[serde(rename = "$text")]
    pub value: String, // 标识符值
}

/// 评分容器结构 - 基于 NFO 研究，符合标准格式
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Ratings {
    #[serde(rename = "rating", default, skip_serializing_if = "Vec::is_empty")]
    pub ratings: Vec<Rating>, // 评分列表
}

/// 评分信息结构 - 基于 NFO 研究，使用通用的多源评分格式
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Rating {
    #[serde(rename = "@name", default)]
    pub name: String, // 评分来源 (imdb, javdb 等)
    #[serde(rename = "@max", default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f32>, // 最高评分值
    #[serde(rename = "@default", default, skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>, // 是否为主要评分
    #[serde(rename = "value", default, skip_serializing_if = "Option::is_none")]
    pub value: Option<f32>, // 评分值
    #[serde(rename = "votes", default, skip_serializing_if = "Option::is_none")]
    pub votes: Option<u32>, // 投票数
}

/// Fanart 结构 - 符合 Kodi 标准
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct FanArt {
    #[serde(rename = "thumb", default, skip_serializing_if = "Vec::is_empty")]
    pub thumbs: Vec<FanArtThumb>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FanArtThumb {
    #[serde(rename = "@preview", skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>, // 预览图 URL
    #[serde(rename = "$text")]
    pub url: String, // 高清图 URL
}

/// 艺术作品信息 - 基于 NFO 研究，支持 Kodi 完整格式
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ArtWork {
    #[serde(rename = "poster", default, skip_serializing_if = "String::is_empty")]
    pub poster: String, // 主海报 URL
    #[serde(rename = "fanart", skip_serializing_if = "Option::is_none")]
    pub fanart: Option<FanArt>, // 背景图信息
    #[serde(rename = "thumb", default, skip_serializing_if = "String::is_empty")]
    pub thumb: String, // 缩略图 URL
    #[serde(
        rename = "landscape",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub landscape: String, // 横版图 URL
}

// 移除复杂的文件信息结构，根据 NFO 研究，大多数用户不需要技术细节

/// 简化的电影 NFO 数据结构 - 基于 NFO 研究，专注核心字段和通用兼容性
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename = "movie")]
pub struct MovieNfo {
    // === 必需的基本信息标签 ===
    #[serde(rename = "title", default, skip_serializing_if = "String::is_empty")]
    pub title: String, // 必需 - 所有平台

    #[serde(
        rename = "originaltitle",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub original_title: String, // 通用支持

    #[serde(rename = "plot", default, skip_serializing_if = "String::is_empty")]
    pub plot: String, // 通用支持

    #[serde(rename = "tagline", default, skip_serializing_if = "String::is_empty")]
    pub tagline: String, // 通用支持

    // === 时间信息 ===
    #[serde(rename = "year", default, skip_serializing_if = "Option::is_none")]
    pub year: Option<u16>, // 通用支持

    #[serde(
        rename = "premiered",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub premiered: String, // 通用支持

    #[serde(
        rename = "releasedate",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub release_date: String, // 通用支持

    #[serde(rename = "runtime", default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<u32>, // 通用支持

    // === 简化的评分系统 ===
    #[serde(rename = "rating", default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<f32>, // 简单评分，向下兼容

    #[serde(rename = "ratings", skip_serializing_if = "Option::is_none")]
    pub ratings: Option<Ratings>, // 多源评分容器，支持 TOP250

    // === 现代化 ID 系统 ===
    #[serde(rename = "uniqueid", default, skip_serializing_if = "Vec::is_empty")]
    pub unique_ids: Vec<UniqueId>, // 新格式标识符，推荐使用

    #[serde(rename = "imdbid", default, skip_serializing_if = "String::is_empty")]
    pub imdb_id: String, // 传统格式，向下兼容

    // === 分类信息 ===
    #[serde(rename = "genre", default, skip_serializing_if = "Vec::is_empty")]
    pub genres: Vec<String>, // 通用支持

    #[serde(rename = "tag", default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>, // 通用支持

    #[serde(rename = "studio", default, skip_serializing_if = "Vec::is_empty")]
    pub studios: Vec<String>, // 通用支持

    // === 人员信息 ===
    #[serde(rename = "director", default, skip_serializing_if = "Vec::is_empty")]
    pub directors: Vec<String>, // 通用支持

    #[serde(rename = "actor", default, skip_serializing_if = "Vec::is_empty")]
    pub actors: Vec<Actor>, // 通用支持

    // === 分级信息 ===
    #[serde(rename = "mpaa", default, skip_serializing_if = "String::is_empty")]
    pub mpaa: String, // 通用支持

    // === 系列信息 ===
    #[serde(rename = "set", default, skip_serializing_if = "Option::is_none")]
    pub set: Option<MovieSet>, // 通用支持

    // === 艺术作品 ===
    #[serde(rename = "art", default, skip_serializing_if = "Option::is_none")]
    pub art: Option<ArtWork>, // 通用支持

    // === 成人内容标记 ===
    #[serde(rename = "isadult", default)]
    pub is_adult: bool, // Emby/Jellyfin 支持
}

/// 简化的爬虫数据结构 - 匹配简化的 NFO 结构
#[derive(Debug, Default, Clone, Crawler)]
pub struct MovieNfoCrawler {
    // 基本信息
    pub title: String,
    pub original_title: Option<String>,
    pub plot: String,
    pub tagline: String,

    // 时间信息
    pub year: Option<u16>,
    pub premiered: String,
    pub release_date: String,
    pub runtime: Option<u32>,

    // 评分信息
    pub rating: Option<f32>,

    // ID 信息
    pub imdb_id: String,
    pub tmdb_id: String,
    pub tvdb_id: String,

    // 分类信息
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub studios: Vec<String>,

    // 人员信息
    pub directors: Vec<String>,
    pub actors: Vec<Actor>,

    // 分级信息
    pub mpaa: String,

    // 艺术作品
    pub posters: Vec<String>,
    pub fanarts: Vec<String>,
    pub thumbs: Vec<String>,
    pub preview_images: Vec<String>,

    // 成人内容标记
    pub is_adult: Option<bool>,

    // TOP250 排名信息 (用于构建 Rating)
    pub ranking_numbers: Vec<String>,
    pub ranking_categories: Vec<String>,

    // 电影系列/集合信息
    pub series_name: String,
    pub series_overview: String,
}

impl MovieNfo {
    /// 生成通用的 NFO 结构，兼容 Kodi/Emby/Jellyfin
    pub fn for_universal(crawler: MovieNfoCrawler) -> Self {
        let mut nfo = MovieNfo::from(crawler.clone());
        // 构建通用的 ratings，包含 TOP250 排名
        nfo.ratings = Self::build_universal_ratings(&crawler);
        // 构建现代化的 uniqueid 标签
        nfo.unique_ids = Self::build_unique_ids(&crawler);
        nfo
    }

    /// 构建通用的评分系统，包含 TOP250 排名
    fn build_universal_ratings(crawler: &MovieNfoCrawler) -> Option<Ratings> {
        let mut ratings = Vec::new();

        // 添加基础评分
        if let Some(rating) = crawler.rating {
            ratings.push(Rating {
                name: "default".to_string(),
                max: Some(10.0),
                default: Some(true),
                value: Some(rating),
                votes: None,
            });
        }

        // 添加 TOP250 排名信息
        for (rank_str, category) in crawler
            .ranking_numbers
            .iter()
            .zip(crawler.ranking_categories.iter())
        {
            let cleaned_rank = rank_str.strip_prefix("No.").unwrap_or(rank_str);
            if let Ok(rank_num) = cleaned_rank.parse::<f32>() {
                ratings.push(Rating {
                    name: category.clone(),
                    max: Some(250.0),
                    default: Some(false),
                    value: Some(rank_num),
                    votes: None,
                });
            }
        }

        if ratings.is_empty() {
            None
        } else {
            Some(Ratings { ratings })
        }
    }

    /// 构建现代化的 uniqueid 标签
    fn build_unique_ids(crawler: &MovieNfoCrawler) -> Vec<UniqueId> {
        let mut unique_ids = Vec::new();

        // 添加 IMDB ID（如果存在，设为默认）
        if !crawler.imdb_id.is_empty() {
            unique_ids.push(UniqueId {
                id_type: "imdb".to_string(),
                default: Some(true),
                value: crawler.imdb_id.clone(),
            });
        }

        // 添加 TMDB ID
        if !crawler.tmdb_id.is_empty() {
            unique_ids.push(UniqueId {
                id_type: "tmdb".to_string(),
                default: None,
                value: crawler.tmdb_id.clone(),
            });
        }

        // 添加 TVDB ID
        if !crawler.tvdb_id.is_empty() {
            unique_ids.push(UniqueId {
                id_type: "tvdb".to_string(),
                default: None,
                value: crawler.tvdb_id.clone(),
            });
        }

        unique_ids
    }

    /// 构建艺术作品信息，符合 Kodi 标准格式
    fn build_artwork(crawler: &MovieNfoCrawler) -> Option<ArtWork> {
        let has_any_art = !crawler.posters.is_empty()
            || !crawler.fanarts.is_empty()
            || !crawler.thumbs.is_empty()
            || !crawler.preview_images.is_empty();

        if !has_any_art {
            return None;
        }

        let fanart = if !crawler.fanarts.is_empty() || !crawler.preview_images.is_empty() {
            let mut thumbs = Vec::new();

            // 添加背景图
            for fanart_url in &crawler.fanarts {
                thumbs.push(FanArtThumb {
                    preview: None,
                    url: fanart_url.clone(),
                });
            }

            // 添加预览图，使用预览图作为高清图的预览
            for (i, preview_url) in crawler.preview_images.iter().enumerate() {
                // 如果有对应的高清图，使用预览图作为预览
                if i < crawler.fanarts.len() {
                    // 已经在上面处理了
                } else {
                    // 预览图本身作为背景图
                    thumbs.push(FanArtThumb {
                        preview: Some(preview_url.clone()),
                        url: preview_url.clone(),
                    });
                }
            }

            if !thumbs.is_empty() {
                Some(FanArt { thumbs })
            } else {
                None
            }
        } else {
            None
        };

        Some(ArtWork {
            poster: crawler.posters.first().cloned().unwrap_or_default(),
            fanart,
            thumb: crawler.thumbs.first().cloned().unwrap_or_default(),
            landscape: crawler.preview_images.first().cloned().unwrap_or_default(),
        })
    }
}

impl From<MovieNfoCrawler> for MovieNfo {
    fn from(crawler: MovieNfoCrawler) -> Self {
        // 先构建艺术作品，避免借用冲突
        let art = Self::build_artwork(&crawler);

        MovieNfo {
            // 基本信息
            title: crawler.title,
            original_title: crawler.original_title.unwrap_or_default(),
            plot: crawler.plot,
            tagline: crawler.tagline,

            // 时间信息
            year: crawler.year,
            premiered: crawler.premiered,
            release_date: crawler.release_date,
            runtime: crawler.runtime,

            // 评分信息 (简单)
            rating: crawler.rating,

            // ID 信息
            imdb_id: crawler.imdb_id,

            // 分类信息
            genres: crawler.genres,
            tags: crawler.tags,
            studios: crawler.studios,

            // 人员信息
            directors: crawler.directors,
            actors: crawler.actors,

            // 分级信息
            mpaa: crawler.mpaa,

            // 电影系列/集合
            set: if !crawler.series_name.is_empty() {
                Some(MovieSet {
                    name: crawler.series_name,
                    overview: crawler.series_overview,
                })
            } else {
                None
            },

            // 艺术作品
            art,

            // 成人内容
            is_adult: crawler.is_adult.unwrap_or(false),

            ..Default::default()
        }
    }
}

/// NFO 格式化器 - 简化为通用格式
pub trait NfoFormatter {
    fn format_to_xml(&self) -> String;
}

impl NfoFormatter for MovieNfo {
    fn format_to_xml(&self) -> String {
        // 使用标准的 XML 序列化，兼容所有平台
        match quick_xml::se::to_string(self) {
            Ok(xml) => xml,
            Err(e) => {
                log::error!("NFO XML 序列化失败: {}", e);
                String::new()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nfo_xml_generation() {
        let crawler = MovieNfoCrawler {
            title: "测试电影".to_string(),
            original_title: Some("Test Movie".to_string()),
            plot: "这是一个测试电影的剧情介绍。".to_string(),
            year: Some(2023),
            imdb_id: "tt1234567".to_string(),
            tmdb_id: "12345".to_string(),
            genres: vec!["动作".to_string(), "冒险".to_string()],
            actors: vec![Actor {
                name: "演员1".to_string(),
                role: "主角".to_string(),
                order: Some(1),
                ..Default::default()
            }],
            directors: vec!["导演1".to_string()],
            rating: Some(8.5),
            ranking_numbers: vec!["No.50".to_string()],
            ranking_categories: vec!["TOP250".to_string()],
            series_name: "测试系列".to_string(),
            series_overview: "测试系列描述".to_string(),
            posters: vec!["https://example.com/poster.jpg".to_string()],
            fanarts: vec!["https://example.com/fanart.jpg".to_string()],
            ..Default::default()
        };

        let nfo = MovieNfo::for_universal(crawler);
        let xml = nfo.format_to_xml();

        println!("Generated NFO XML:\n{}", xml);

        // 验证包含基本信息
        assert!(xml.contains("<title>测试电影</title>"));
        assert!(xml.contains("<originaltitle>Test Movie</originaltitle>"));
        assert!(xml.contains("<year>2023</year>"));

        // 验证 uniqueid 格式
        assert!(xml.contains("uniqueid"));
        assert!(xml.contains("imdb"));
        assert!(xml.contains("tmdb"));

        // 验证 ratings 格式
        assert!(xml.contains("ratings"));
        assert!(xml.contains("TOP250"));

        // 验证 set 标签
        assert!(xml.contains("<set>"));
        assert!(xml.contains("<name>测试系列</name>"));

        // 验证艺术作品
        assert!(xml.contains("<art>"));
        assert!(xml.contains("<fanart>"));
    }
}
