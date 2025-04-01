use crawler_template::Crawler;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "movie")]
pub struct NFO {
    #[serde(default)]
    pub plot: Option<String>,

    #[serde(default)]
    pub outline: Option<String>,

    #[serde(default)]
    pub lockdata: Option<bool>,

    #[serde(default)]
    pub dateadded: Option<String>,

    #[serde(default)]
    pub title: Option<String>,

    #[serde(default, rename = "actor")]
    pub actors: Vec<Actor>,

    #[serde(default)]
    pub top250: Option<i32>,

    #[serde(default)]
    pub director: Option<String>,

    #[serde(default)]
    pub trailer: Option<String>,

    #[serde(default)]
    pub rating: Option<f32>,

    #[serde(default)]
    pub year: Option<i32>,

    #[serde(default)]
    pub sorttitle: Option<String>,

    #[serde(default)]
    pub originaltitle: Option<String>,

    #[serde(default)]
    pub mpaa: Option<String>,

    #[serde(default)]
    pub premiered: Option<String>,

    #[serde(default)]
    pub releasedate: Option<String>,

    #[serde(default)]
    pub runtime: Option<i32>,

    #[serde(default)]
    pub country: Option<String>,

    #[serde(default, rename = "tag")]
    pub tags: Vec<String>,

    #[serde(default, rename = "genre")]
    pub genres: Vec<String>,

    #[serde(default)]
    pub studio: Option<String>,

    #[serde(default)]
    pub uniqueid: Option<UniqueId>,

    #[serde(default)]
    pub numid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Actor {
    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub role: Option<String>,

    #[serde(default)]
    pub order: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UniqueId {
    #[serde(rename = "type")]
    pub id_type: String,

    #[serde(rename = "$value")]
    pub value: String,
}


#[derive(Debug, Default, Crawler)]
pub struct CrawlerNfo {
    pub plot: Option<String>,
    pub outline: Option<String>,
    pub lockdata: Option<bool>,
    pub dateadded: Option<String>,
    pub title: Option<String>,
    pub actors: Vec<String>,
    pub top250: Option<i32>,
    pub director: Option<String>,
    pub trailer: Option<String>,
    pub rating: Option<f32>,
    pub year: Option<i32>,
    pub sorttitle: Option<String>,
    pub originaltitle: Option<String>,
    pub mpaa: Option<String>,
    pub premiered: Option<String>,
    pub releasedate: Option<String>,
    pub runtime: Option<i32>,
    pub country: Option<String>,
    pub tags: Vec<String>,
    pub genres: Vec<String>,
    pub studio: Option<String>,
    pub uniqueid: Option<String>,
    pub numid: Option<String>,
}
