use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[allow(non_upper_case_globals)]
pub mod fields;
pub mod options;

pub use options::StatusBuilder;

#[derive(Serialize, Deserialize, Debug)]
pub struct AnimeList {
    pub data: Vec<ListNode>,
    paging: HashMap<String, Value>,
    pub season: Option<HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListNode {
    pub node: Anime,
    pub list_status: Option<ListStatus>,
    pub ranking: Option<HashMap<String, u32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListStatus {
    pub status: Option<String>,
    pub num_episodes_watched: Option<u32>,
    pub score: Option<u8>,
    pub updated_at: Option<String>,
    pub is_rewatching: Option<bool>,
    pub priority: Option<u32>,
    pub rewatch_value: Option<u32>,
    pub tags: Option<Vec<String>>,
    pub comments: Option<String>,
    pub start_date: Option<String>,
    pub finish_date: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Anime {
    pub id: u32,
    pub title: String,
    pub main_picture: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnimeDetails {
    #[serde(flatten)]
    pub show: Anime,
    pub alternative_titles: Option<AlternativeTitles>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub synopsis: Option<String>,
    pub mean: Option<f32>,
    pub rank: Option<u32>,
    pub num_list_users: Option<u32>,
    pub num_scoring_users: Option<u32>,
    pub nsfw: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub media_type: Option<String>,
    pub status: Option<String>,
    pub genres: Option<Vec<HashMap<String, Value>>>,
    pub my_list_status: Option<ListStatus>,
    pub num_episodes: Option<u32>,
    pub start_season: Option<HashMap<String, Value>>,
    pub broadcast: Option<HashMap<String, String>>,
    pub source: Option<String>,
    pub average_episode_duration: Option<u32>,
    pub rating: Option<String>,
    pub pictures: Option<Vec<HashMap<String, String>>>,
    pub background: Option<String>,
    pub related_anime: Option<Vec<Related>>,
    pub related_manga: Option<Vec<HashMap<String, Value>>>,
    pub recommendations: Option<Vec<Recommnendation>>,
    pub studios: Option<Vec<HashMap<String, Value>>>,
    pub statistics: Option<Stats>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Stats {
    pub status: HashMap<String, String>,
    pub num_list_users: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AlternativeTitles {
    pub synonyms: Vec<String>,
    #[serde(flatten)]
    pub languages: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Related {
    pub node: Anime,
    pub relation_type: String,
    pub relation_type_formatted: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Recommnendation {
    pub node: Anime,
    pub num_recommendations: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub location: String,
    pub joined_at: String,
    pub anime_statistics: HashMap<String, f32>,
}

//TODO: Improve struct coverage for forum fucntions
#[derive(Serialize, Deserialize, Debug)]
pub struct ForumBoards {
    pub categories: Vec<HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TopicDetails {
    pub data: Vec<HashMap<String, Value>>,
    pub paging: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ForumTopics {
    pub data: Vec<HashMap<String, Value>>,
    pub paging: Vec<HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EpisodesList {
    pub data: Vec<EpisodeNode>,
    paging: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EpisodeNode {
    pub mal_id: Option<u32>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub title_japanese: Option<String>,
    pub title_romanji: Option<String>,
    pub duration: Option<u32>,
    pub aired: Option<String>,
    pub filler: Option<bool>,
    pub recap: Option<bool>,
    pub forum_url: Option<String>,
}
