use crawler_template::Crawler;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// 媒体中心类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum MediaCenterType {
    Kodi,
    Emby,
    Jellyfin,
    All, // 兼容所有格式
}

/// 演员信息结构
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Actor {
    #[serde(rename = "name", default, skip_serializing_if = "String::is_empty")]
    pub name: String, // 演员姓名
    #[serde(rename = "role", default, skip_serializing_if = "String::is_empty")]
    pub role: String, // 演员扮演的角色名
    #[serde(rename = "thumb", default, skip_serializing_if = "String::is_empty")]
    pub thumb: String, // 演员头像的 URL 或路径
    #[serde(rename = "profile", default, skip_serializing_if = "String::is_empty")]
    pub profile: String, // 演员资料页面的 URL
    #[serde(rename = "order", default, skip_serializing_if = "Option::is_none")]
    pub order: Option<u32>, // 演员排序 (主要用于 Kodi)
    #[serde(rename = "type", default, skip_serializing_if = "String::is_empty")]
    pub actor_type: String, // 演员类型 (Emby/Jellyfin 支持)
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

/// 唯一的媒体 ID 结构 (支持所有媒体中心)
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UniqueID {
    #[serde(rename = "@type", default)]
    pub id_type: String, // ID 类型 (imdb, tmdb, tvdb, douban, javdb 等)
    #[serde(rename = "@default", default, skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>, // 是否为主要 ID
    #[serde(rename = "$text")]
    pub value: String, // ID 的具体值
}

/// 提供商 ID 映射 (用于 Emby/Jellyfin)
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ProviderIds {
    #[serde(rename = "imdbid", default, skip_serializing_if = "String::is_empty")]
    pub imdb_id: String,
    #[serde(rename = "tmdbid", default, skip_serializing_if = "String::is_empty")]
    pub tmdb_id: String,
    #[serde(rename = "tvdbid", default, skip_serializing_if = "String::is_empty")]
    pub tvdb_id: String,
    #[serde(rename = "doubanid", default, skip_serializing_if = "String::is_empty")]
    pub douban_id: String,
    #[serde(rename = "javdbid", default, skip_serializing_if = "String::is_empty")]
    pub javdb_id: String,
}

/// 评分信息结构 (支持多个评分来源)
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Rating {
    #[serde(rename = "@name", default)]
    pub name: String, // 评分来源 (imdb, tmdb, metacritic, douban, javdb, default)
    #[serde(rename = "@max", default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f32>, // 最高评分值
    #[serde(rename = "@default", default, skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>, // 是否为主要评分
    #[serde(rename = "value", default, skip_serializing_if = "Option::is_none")]
    pub value: Option<f32>, // 评分值
    #[serde(rename = "votes", default, skip_serializing_if = "Option::is_none")]
    pub votes: Option<u32>, // 投票数
}

/// 简化的评分信息 (用于 Emby/Jellyfin)
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SimpleRating {
    #[serde(rename = "Value")]
    pub value: f32,
    #[serde(rename = "VoteCount", skip_serializing_if = "Option::is_none")]
    pub vote_count: Option<u32>,
}

/// 艺术作品信息 (海报、背景等)
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ArtWork {
    #[serde(rename = "poster", default, skip_serializing_if = "Vec::is_empty")]
    pub posters: Vec<String>, // 海报 URL 列表
    #[serde(rename = "fanart", default, skip_serializing_if = "Vec::is_empty")]
    pub fanarts: Vec<String>, // 背景图 URL 列表
    #[serde(rename = "thumb", default, skip_serializing_if = "Vec::is_empty")]
    pub thumbs: Vec<String>, // 缩略图 URL 列表
    #[serde(rename = "banner", default, skip_serializing_if = "Vec::is_empty")]
    pub banners: Vec<String>, // 横幅图 URL 列表
    #[serde(rename = "clearart", default, skip_serializing_if = "Vec::is_empty")]
    pub cleararts: Vec<String>, // 透明艺术图 URL 列表
    #[serde(rename = "clearlogo", default, skip_serializing_if = "Vec::is_empty")]
    pub clearlogos: Vec<String>, // 透明 Logo URL 列表
    #[serde(rename = "landscape", default, skip_serializing_if = "Vec::is_empty")]
    pub landscapes: Vec<String>, // 横版图 URL 列表
}

/// 文件信息结构
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct FileInfo {
    #[serde(rename = "streamdetails", default)]
    pub stream_details: StreamDetails,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct StreamDetails {
    #[serde(rename = "video", default, skip_serializing_if = "Vec::is_empty")]
    pub video: Vec<VideoStream>,
    #[serde(rename = "audio", default, skip_serializing_if = "Vec::is_empty")]
    pub audio: Vec<AudioStream>,
    #[serde(rename = "subtitle", default, skip_serializing_if = "Vec::is_empty")]
    pub subtitle: Vec<SubtitleStream>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct VideoStream {
    #[serde(rename = "codec", default, skip_serializing_if = "String::is_empty")]
    pub codec: String,
    #[serde(rename = "aspect", default, skip_serializing_if = "Option::is_none")]
    pub aspect: Option<f32>,
    #[serde(rename = "width", default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(rename = "height", default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(
        rename = "durationinseconds",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub duration_in_seconds: Option<u32>,
    #[serde(
        rename = "stereomode",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub stereo_mode: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AudioStream {
    #[serde(rename = "codec", default, skip_serializing_if = "String::is_empty")]
    pub codec: String,
    #[serde(rename = "language", default, skip_serializing_if = "String::is_empty")]
    pub language: String,
    #[serde(rename = "channels", default, skip_serializing_if = "Option::is_none")]
    pub channels: Option<u32>,
    #[serde(rename = "default", default)]
    pub default: bool,
    #[serde(rename = "forced", default)]
    pub forced: bool,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SubtitleStream {
    #[serde(rename = "language", default, skip_serializing_if = "String::is_empty")]
    pub language: String,
    #[serde(rename = "default", default)]
    pub default: bool,
    #[serde(rename = "forced", default)]
    pub forced: bool,
}

/// 统一的电影 NFO 数据结构 (兼容 Kodi/Emby/Jellyfin)
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename = "movie")]
pub struct MovieNfo {
    // === 基本信息 ===
    #[serde(rename = "title", default, skip_serializing_if = "String::is_empty")]
    pub title: String,

    #[serde(
        rename = "originaltitle",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub original_title: String,

    #[serde(
        rename = "sorttitle",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub sort_title: String,

    #[serde(
        rename = "localtitle",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub local_title: String, // Emby/Jellyfin

    #[serde(rename = "plot", default, skip_serializing_if = "String::is_empty")]
    pub plot: String,

    #[serde(rename = "outline", default, skip_serializing_if = "String::is_empty")]
    pub outline: String,

    #[serde(rename = "tagline", default, skip_serializing_if = "String::is_empty")]
    pub tagline: String,

    // === 评分信息 ===
    #[serde(rename = "rating", default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<f32>, // 向下兼容

    #[serde(rename = "ratings", default, skip_serializing_if = "Vec::is_empty")]
    pub ratings: Vec<Rating>,

    #[serde(
        rename = "userrating",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub user_rating: Option<u32>,

    #[serde(
        rename = "communityrating",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub community_rating: Option<f32>, // Emby/Jellyfin

    #[serde(
        rename = "criticrating",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub critic_rating: Option<f32>, // Emby/Jellyfin

    #[serde(
        rename = "criticratingsummary",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub critic_rating_summary: String, // Emby/Jellyfin

    #[serde(rename = "top250", default, skip_serializing_if = "Option::is_none")]
    pub top250: Option<u32>,

    // === 时间信息 ===
    #[serde(rename = "year", default, skip_serializing_if = "Option::is_none")]
    pub year: Option<u16>,

    #[serde(
        rename = "premiered",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub premiered: String,

    #[serde(
        rename = "releasedate",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub release_date: String,

    #[serde(rename = "runtime", default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<u32>,

    // === 标识符 ===
    #[serde(rename = "imdbid", default, skip_serializing_if = "String::is_empty")]
    pub imdb_id: String, // 向下兼容

    #[serde(rename = "tmdbid", default, skip_serializing_if = "Option::is_none")]
    pub tmdb_id: Option<u32>, // 向下兼容

    #[serde(rename = "uniqueid", default, skip_serializing_if = "Vec::is_empty")]
    pub unique_ids: Vec<UniqueID>, // Kodi 标准格式

    #[serde(
        rename = "providerids",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub provider_ids: Option<ProviderIds>, // Emby/Jellyfin

    #[serde(
        rename = "tmdbcollectionid",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub tmdb_collection_id: String, // Emby 特有

    #[serde(
        rename = "tmdbsetid",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub tmdb_set_id: String, // 另一种格式

    // === 分类信息 ===
    #[serde(rename = "genre", default, skip_serializing_if = "Vec::is_empty")]
    pub genres: Vec<String>,

    #[serde(rename = "tag", default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    #[serde(rename = "country", default, skip_serializing_if = "Vec::is_empty")]
    pub countries: Vec<String>,

    #[serde(rename = "studio", default, skip_serializing_if = "Vec::is_empty")]
    pub studios: Vec<String>,

    #[serde(
        rename = "productioncompany",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub production_companies: Vec<String>, // Emby/Jellyfin

    #[serde(
        rename = "productionlocation",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub production_locations: Vec<String>, // Emby/Jellyfin

    // === 演职人员 ===
    #[serde(rename = "director", default, skip_serializing_if = "Vec::is_empty")]
    pub directors: Vec<String>,

    #[serde(rename = "writer", default, skip_serializing_if = "Vec::is_empty")]
    pub writers: Vec<String>,

    #[serde(rename = "credits", default, skip_serializing_if = "Vec::is_empty")]
    pub credits: Vec<String>, // Kodi 格式

    #[serde(rename = "actor", default, skip_serializing_if = "Vec::is_empty")]
    pub actors: Vec<Actor>,

    #[serde(rename = "producer", default, skip_serializing_if = "Vec::is_empty")]
    pub producers: Vec<String>, // Emby/Jellyfin

    #[serde(rename = "composer", default, skip_serializing_if = "Vec::is_empty")]
    pub composers: Vec<String>, // Emby/Jellyfin

    // === 分级信息 ===
    #[serde(rename = "mpaa", default, skip_serializing_if = "String::is_empty")]
    pub mpaa: String,

    #[serde(
        rename = "certification",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub certification: String,

    // === 系列信息 ===
    #[serde(rename = "set", default, skip_serializing_if = "Option::is_none")]
    pub set: Option<MovieSet>,

    // === 用户数据 ===
    #[serde(rename = "playcount", default, skip_serializing_if = "Option::is_none")]
    pub play_count: Option<u32>,

    #[serde(rename = "watched", default)]
    pub watched: bool,

    #[serde(
        rename = "lastplayed",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub last_played: String,

    #[serde(
        rename = "dateadded",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub date_added: String,

    // === 文件和艺术作品 ===
    #[serde(rename = "fileinfo", default, skip_serializing_if = "Option::is_none")]
    pub file_info: Option<FileInfo>,

    #[serde(rename = "art", default, skip_serializing_if = "Option::is_none")]
    pub art: Option<ArtWork>,

    // === 其他扩展字段 ===
    #[serde(rename = "lockdata", default)]
    pub lock_data: bool, // 锁定数据，防止被覆盖 (Emby/Jellyfin)

    #[serde(rename = "isadult", default)]
    pub is_adult: bool, // 是否为成人内容 (Emby/Jellyfin)

    #[serde(
        rename = "customrating",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub custom_rating: String, // 自定义评级 (Emby/Jellyfin)

    #[serde(rename = "enddate", default, skip_serializing_if = "String::is_empty")]
    pub end_date: String, // 结束日期 (用于连续剧等)

    #[serde(
        rename = "lockedfields",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub locked_fields: Vec<String>, // 锁定字段列表 (Emby/Jellyfin)
}

/// 统一的爬虫数据结构 (用于模板转换)
#[derive(Debug, Default, Clone, Crawler)]
pub struct MovieNfoCrawler {
    // 基本信息
    pub title: String,
    pub original_title: String,
    pub local_title: String,
    pub plot: String,
    pub outline: String,
    pub tagline: String,

    // 评分信息
    pub rating: Option<f32>,
    pub user_rating: Option<u32>,
    pub community_rating: Option<f32>,
    pub critic_rating: Option<f32>,
    pub top250: Option<u32>,

    // 时间信息
    pub year: Option<u16>,
    pub premiered: String,
    pub release_date: String,
    pub runtime: Option<u32>,

    // ID 信息
    pub imdb_id: String,
    pub tmdb_id: Option<u32>,
    pub douban_id: String,
    pub javdb_id: String,

    // 分类信息
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub countries: Vec<String>,
    pub studios: Vec<String>,

    // 人员信息
    pub directors: Vec<String>,
    pub writers: Vec<String>,
    pub actors: Vec<Actor>,

    // 分级信息
    pub mpaa: String,
    pub certification: String,

    // 艺术作品
    pub posters: Vec<String>,
    pub fanarts: Vec<String>,
    pub thumbs: Vec<String>,

    // 其他
    pub is_adult: Option<bool>,
    pub custom_rating: String,
}

impl MovieNfo {
    /// 根据媒体中心类型生成对应的 NFO 结构
    pub fn for_media_center(crawler: MovieNfoCrawler, media_center: MediaCenterType) -> Self {
        let mut nfo = MovieNfo::from(crawler.clone());

        match media_center {
            MediaCenterType::Kodi => {
                // Kodi 特有配置
                nfo.unique_ids = Self::build_kodi_unique_ids(&crawler);
                nfo.ratings = Self::build_kodi_ratings(&crawler);
            }
            MediaCenterType::Emby => {
                // Emby 特有配置
                nfo.provider_ids = Some(Self::build_emby_provider_ids(&crawler));
                nfo.community_rating = crawler.community_rating;
                nfo.critic_rating = crawler.critic_rating;
                nfo.tmdb_collection_id =
                    crawler.tmdb_id.map(|id| id.to_string()).unwrap_or_default();
            }
            MediaCenterType::Jellyfin => {
                // Jellyfin 特有配置
                nfo.provider_ids = Some(Self::build_jellyfin_provider_ids(&crawler));
                nfo.community_rating = crawler.community_rating;
                nfo.critic_rating = crawler.critic_rating;
                nfo.lock_data = true; // 默认锁定数据
            }
            MediaCenterType::All => {
                // 兼容所有格式
                nfo.unique_ids = Self::build_kodi_unique_ids(&crawler);
                nfo.provider_ids = Some(Self::build_emby_provider_ids(&crawler));
                nfo.ratings = Self::build_kodi_ratings(&crawler);
                nfo.community_rating = crawler.community_rating;
                nfo.critic_rating = crawler.critic_rating;
            }
        }

        nfo
    }

    fn build_kodi_unique_ids(crawler: &MovieNfoCrawler) -> Vec<UniqueID> {
        let mut ids = Vec::new();

        if !crawler.imdb_id.is_empty() {
            ids.push(UniqueID {
                id_type: "imdb".to_string(),
                default: Some(true),
                value: crawler.imdb_id.clone(),
            });
        }

        if let Some(tmdb_id) = crawler.tmdb_id {
            ids.push(UniqueID {
                id_type: "tmdb".to_string(),
                default: Some(false),
                value: tmdb_id.to_string(),
            });
        }

        if !crawler.douban_id.is_empty() {
            ids.push(UniqueID {
                id_type: "douban".to_string(),
                default: Some(false),
                value: crawler.douban_id.clone(),
            });
        }

        if !crawler.javdb_id.is_empty() {
            ids.push(UniqueID {
                id_type: "javdb".to_string(),
                default: Some(false),
                value: crawler.javdb_id.clone(),
            });
        }

        ids
    }

    fn build_emby_provider_ids(crawler: &MovieNfoCrawler) -> ProviderIds {
        ProviderIds {
            imdb_id: crawler.imdb_id.clone(),
            tmdb_id: crawler.tmdb_id.map(|id| id.to_string()).unwrap_or_default(),
            douban_id: crawler.douban_id.clone(),
            javdb_id: crawler.javdb_id.clone(),
            ..Default::default()
        }
    }

    fn build_jellyfin_provider_ids(crawler: &MovieNfoCrawler) -> ProviderIds {
        // Jellyfin 使用与 Emby 相同的格式
        Self::build_emby_provider_ids(crawler)
    }

    fn build_kodi_ratings(crawler: &MovieNfoCrawler) -> Vec<Rating> {
        let mut ratings = Vec::new();

        if let Some(rating) = crawler.rating {
            ratings.push(Rating {
                name: "default".to_string(),
                max: Some(10.0),
                default: Some(true),
                value: Some(rating),
                votes: None,
            });
        }

        if let Some(community_rating) = crawler.community_rating {
            ratings.push(Rating {
                name: "community".to_string(),
                max: Some(10.0),
                default: Some(false),
                value: Some(community_rating),
                votes: None,
            });
        }

        if let Some(critic_rating) = crawler.critic_rating {
            ratings.push(Rating {
                name: "critic".to_string(),
                max: Some(10.0),
                default: Some(false),
                value: Some(critic_rating),
                votes: None,
            });
        }

        ratings
    }
}

impl From<MovieNfoCrawler> for MovieNfo {
    fn from(crawler: MovieNfoCrawler) -> Self {
        MovieNfo {
            // 基本信息
            title: crawler.title,
            original_title: crawler.original_title,
            local_title: crawler.local_title,
            plot: crawler.plot,
            outline: crawler.outline,
            tagline: crawler.tagline,

            // 评分信息
            rating: crawler.rating,
            user_rating: crawler.user_rating,
            community_rating: crawler.community_rating,
            critic_rating: crawler.critic_rating,
            top250: crawler.top250,

            // 时间信息
            year: crawler.year,
            premiered: crawler.premiered,
            release_date: crawler.release_date,
            runtime: crawler.runtime,

            // ID 信息 (向下兼容)
            imdb_id: crawler.imdb_id,
            tmdb_id: crawler.tmdb_id,

            // 分类信息
            genres: crawler.genres,
            tags: crawler.tags,
            countries: crawler.countries,
            studios: crawler.studios,

            // 人员信息
            directors: crawler.directors,
            writers: crawler.writers,
            actors: crawler.actors,

            // 分级信息
            mpaa: crawler.mpaa,
            certification: crawler.certification,

            // 艺术作品
            art: if !crawler.posters.is_empty()
                || !crawler.fanarts.is_empty()
                || !crawler.thumbs.is_empty()
            {
                Some(ArtWork {
                    posters: crawler.posters,
                    fanarts: crawler.fanarts,
                    thumbs: crawler.thumbs,
                    ..Default::default()
                })
            } else {
                None
            },

            // 其他
            is_adult: crawler.is_adult.unwrap_or(false),
            custom_rating: crawler.custom_rating,

            ..Default::default()
        }
    }
}

/// NFO 格式化器特征
pub trait NfoFormatter {
    fn format_for_kodi(&self) -> String;
    fn format_for_emby(&self) -> String;
    fn format_for_jellyfin(&self) -> String;
}

impl NfoFormatter for MovieNfo {
    fn format_for_kodi(&self) -> String {
        // Kodi 使用标准的 XML 序列化
        quick_xml::se::to_string(self).unwrap_or_default()
    }

    fn format_for_emby(&self) -> String {
        // Emby 可能需要特殊的格式化
        // 这里可以添加 Emby 特有的处理逻辑
        self.format_for_kodi()
    }

    fn format_for_jellyfin(&self) -> String {
        // Jellyfin 与 Emby 类似
        self.format_for_emby()
    }
}
