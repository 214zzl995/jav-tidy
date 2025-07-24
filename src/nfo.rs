use crawler_template::Crawler;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Actor {
    #[serde(rename = "name", default, skip_serializing_if = "String::is_empty")]
    pub name: String, // 演员姓名
    #[serde(rename = "role", default, skip_serializing_if = "String::is_empty")]
    pub role: String, // 演员扮演的角色名
    #[serde(rename = "thumb", default, skip_serializing_if = "String::is_empty")]
    pub thumb: String, // 演员头像的 URL 或路径 (可选)
    #[serde(rename = "profile", default, skip_serializing_if = "String::is_empty")]
    pub profile: String, // 演员资料页面的 URL (可选, 不常用)
    #[serde(rename = "order", default, skip_serializing_if = "Option::is_none")]
    pub order: Option<u32>, // 演员排序 (Kodi 特有)
}

// 用于表示电影所属的系列/集合
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct MovieSet {
    #[serde(rename = "name", default, skip_serializing_if = "String::is_empty")]
    pub name: String, // 系列/集合的名称
    #[serde(rename = "overview", default, skip_serializing_if = "String::is_empty")]
    pub overview: String, // 系列/集合的简介 (可选)
}

// 用于表示唯一的媒体 ID (如 IMDB, TMDB 等)
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UniqueID {
    #[serde(rename = "@type", default)] // XML 属性 'type'
    pub id_type: String, // ID 类型 (例如 "imdb", "tmdb", "tvdb")
    #[serde(rename = "@default", default, skip_serializing_if = "Option::is_none")]
    // XML 属性 'default'
    pub default: Option<bool>, // 标记是否为主要 ID (可选, true/false)
    #[serde(rename = "$text")] // XML 标签的文本内容
    pub value: String, // ID 的具体值 (例如 "tt1234567")
}

// 用于表示评分信息
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Rating {
    #[serde(rename = "@name", default)] // XML 属性 'name'
    pub name: String, // 评分来源 (例如 "imdb", "tmdb", "metacritic", "default" 通常指用户评分)
    #[serde(rename = "@max", default, skip_serializing_if = "Option::is_none")] // XML 属性 'max'
    pub max: Option<u32>, // 最高评分值 (例如 10 或 100)
    #[serde(rename = "@default", default, skip_serializing_if = "Option::is_none")]
    // XML 属性 'default'
    pub default: Option<bool>, // 标记是否为主要评分 (可选)
    #[serde(rename = "value", default, skip_serializing_if = "Option::is_none")]
    pub value: Option<f32>, // 评分值
    #[serde(rename = "votes", default, skip_serializing_if = "Option::is_none")]
    pub votes: Option<u32>, // 投票数 (可选)
}

// 用于表示文件信息 (通常由媒体服务器自动生成，但也可包含在 NFO 中)
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct FileInfo {
    #[serde(rename = "streamdetails", default)] // 包裹视频、音频、字幕流的容器
    pub stream_details: StreamDetails,
    // 可选添加 path, dateadded 等字段
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
    pub codec: String, // e.g., "h264", "hevc"
    #[serde(rename = "aspect", default, skip_serializing_if = "Option::is_none")]
    pub aspect: Option<f32>, // e.g., 1.778 (for 16:9)
    #[serde(rename = "width", default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>, // e.g., 1920
    #[serde(rename = "height", default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>, // e.g., 1080
    #[serde(
        rename = "durationinseconds",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub duration_in_seconds: Option<u32>, // 视频时长 (秒)
    #[serde(
        rename = "stereomode",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub stereo_mode: String, // 3D 模式 (e.g., "sbs", "tab")
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AudioStream {
    #[serde(rename = "codec", default, skip_serializing_if = "String::is_empty")]
    pub codec: String, // e.g., "ac3", "dts", "aac"
    #[serde(rename = "language", default, skip_serializing_if = "String::is_empty")]
    pub language: String, // e.g., "eng", "jpn" (ISO 639-2 code)
    #[serde(rename = "channels", default, skip_serializing_if = "Option::is_none")]
    pub channels: Option<u32>, // e.g., 2, 6 (for 5.1), 8 (for 7.1)
    #[serde(rename = "default", default)]
    pub default: bool, // 是否为默认音轨
    #[serde(rename = "forced", default)]
    pub forced: bool, // 是否为强制音轨
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SubtitleStream {
    #[serde(rename = "language", default, skip_serializing_if = "String::is_empty")]
    pub language: String, // e.g., "eng", "chi" (ISO 639-2 code)
    #[serde(rename = "default", default)]
    pub default: bool, // 是否为默认字幕
    #[serde(rename = "forced", default)]
    pub forced: bool, // 是否为强制字幕
}

// --- 主要的电影 NFO 结构 ---
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename = "movie")] // 指定 XML 根元素为 <movie>
pub struct MovieNfo {
    // --- 基本信息 ---
    #[serde(rename = "title", default, skip_serializing_if = "String::is_empty")]
    pub title: String, // 电影标题 (本地化/显示用)

    #[serde(
        rename = "originaltitle",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub original_title: String, // 电影原始标题 (通常是其原始语言的标题)

    #[serde(
        rename = "sorttitle",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub sort_title: String, // 用于排序的标题 (例如，去除 "The ", "A " 等前缀)

    #[serde(rename = "rating", default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<f32>, // 主要评分值 (通常是用户/社区评分, 如 7.8) - 这是旧格式，推荐使用下面的 <ratings>

    #[serde(rename = "ratings", default, skip_serializing_if = "Vec::is_empty")]
    pub ratings: Vec<Rating>, // 包含多个来源评分的列表 (推荐方式)

    #[serde(
        rename = "userrating",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub user_rating: Option<u32>, // 用户个人评分 (通常是 1-10 的整数, Kodi 格式)

    #[serde(rename = "top250", default, skip_serializing_if = "Option::is_none")]
    pub top250: Option<u32>, // 在 IMDB Top 250 或类似榜单中的排名 (可选)

    #[serde(rename = "year", default, skip_serializing_if = "Option::is_none")]
    pub year: Option<u16>, // 电影上映年份 (四位数字)

    #[serde(rename = "plot", default, skip_serializing_if = "String::is_empty")]
    pub plot: String, // 电影情节摘要 (较长)

    #[serde(rename = "outline", default, skip_serializing_if = "String::is_empty")]
    pub outline: String, // 电影情节概要 (较短, 有时与 plot 相同)

    #[serde(rename = "tagline", default, skip_serializing_if = "String::is_empty")]
    pub tagline: String, // 电影宣传语

    #[serde(rename = "runtime", default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<u32>, // 电影时长 (单位：分钟)

    // --- 标识符 ---
    #[serde(rename = "imdbid", default, skip_serializing_if = "String::is_empty")]
    pub imdb_id: String, // IMDb ID (例如 "tt1234567") - 这是旧格式，推荐使用 <uniqueid>

    #[serde(rename = "tmdbid", default, skip_serializing_if = "Option::is_none")]
    pub tmdb_id: Option<u32>, // TheMovieDB ID (纯数字) - 这是旧格式，推荐使用 <uniqueid>

    #[serde(rename = "uniqueid", default, skip_serializing_if = "Vec::is_empty")]
    pub unique_ids: Vec<UniqueID>, // 唯一的媒体 ID 列表 (推荐方式)

    // --- 元数据 ---
    #[serde(rename = "genre", default, skip_serializing_if = "Vec::is_empty")]
    pub genres: Vec<String>, // 类型列表 (例如 ["Action", "Adventure", "Sci-Fi"]) - XML 中会生成多个 <genre> 标签

    #[serde(rename = "tag", default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>, // 标签列表 (用户自定义或特定主题, 例如 ["Superhero", "Based on Book"]) - XML 中会生成多个 <tag> 标签

    #[serde(rename = "country", default, skip_serializing_if = "Vec::is_empty")]
    pub countries: Vec<String>, // 制片国家/地区列表 - XML 中会生成多个 <country> 标签

    #[serde(rename = "studio", default, skip_serializing_if = "Vec::is_empty")]
    pub studios: Vec<String>, // 制片公司列表 - XML 中会生成多个 <studio> 标签

    #[serde(
        rename = "premiered",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub premiered: String, // 首映日期 (格式: YYYY-MM-DD)

    #[serde(
        rename = "releasedate",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub release_date: String, // 发行日期 (格式: YYYY-MM-DD, 通常与 premiered 相同或稍晚)

    #[serde(rename = "mpaa", default, skip_serializing_if = "String::is_empty")]
    pub mpaa: String, // MPAA 分级信息 (例如 "PG-13", "R")

    #[serde(
        rename = "certification",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub certification: String, // 其他地区的分级信息 (例如 "USA:PG-13", "UK:15")

    #[serde(rename = "set", default, skip_serializing_if = "Option::is_none")]
    pub set: Option<MovieSet>, // 所属的电影系列/集合信息 (可选)

    // --- 演职人员 ---
    #[serde(rename = "director", default, skip_serializing_if = "Vec::is_empty")]
    pub directors: Vec<String>, // 导演列表 - XML 中会生成多个 <director> 标签

    #[serde(rename = "writer", default, skip_serializing_if = "Vec::is_empty")]
    pub writers: Vec<String>, // 编剧列表 - XML 中会生成多个 <writer> 标签

    #[serde(rename = "actor", default, skip_serializing_if = "Vec::is_empty")]
    pub actors: Vec<Actor>, // 演员列表 (包含姓名、角色等信息)

    // --- 用户数据 (通常由媒体服务器管理，写入 NFO 可能被覆盖) ---
    #[serde(rename = "playcount", default, skip_serializing_if = "Option::is_none")]
    pub play_count: Option<u32>, // 播放次数

    #[serde(rename = "watched", default)] // bool 类型通常不需要 skip_serializing_if
    pub watched: bool, // 是否已观看 (true/false)

    #[serde(
        rename = "lastplayed",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub last_played: String, // 最后播放日期 (格式: YYYY-MM-DD HH:MM:SS 或 YYYY-MM-DD)

    #[serde(
        rename = "dateadded",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub date_added: String, // 添加到媒体库的日期 (格式: YYYY-MM-DD HH:MM:SS)

    #[serde(rename = "fileinfo", default, skip_serializing_if = "Option::is_none")]
    pub file_info: Option<FileInfo>, // 包含视频/音频/字幕流详细信息 (可选)
}

#[derive(Debug, Default, Clone, Crawler)]
pub struct MovieNfoCrawler {
    pub title: String,
    pub rating: Option<f32>,
    pub top250: Option<u32>,
    pub release_date: Option<String>,
    pub runtime: Option<u32>,
    pub tags: Vec<String>,
    pub studios: Vec<String>,
}

impl From<MovieNfoCrawler> for MovieNfo {
    fn from(crawler: MovieNfoCrawler) -> Self {
        MovieNfo {
            title: crawler.title,
            rating: crawler.rating,
            top250: crawler.top250,
            release_date: crawler.release_date.unwrap_or_default(),
            runtime: crawler.runtime,
            tags: crawler.tags,
            studios: crawler.studios,
            ..Default::default()
        }
    }
}
