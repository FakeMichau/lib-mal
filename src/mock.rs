use std::{path::PathBuf, collections::HashMap, str::FromStr};
use async_trait::async_trait;
use reqwest::Client;
use crate::{MALClientTrait, MALError, prelude::{AnimeList, fields::AnimeFields, AnimeDetails, options::{RankingType, Season, StatusUpdate, Params}, ListStatus, ForumBoards, TopicDetails, ForumTopics, User, EpisodesList}};

pub struct MockMALClient {
    client_secret: String,
    dirs: PathBuf,
    access_token: String,
    client: reqwest::Client,
    caching: bool,
    pub need_auth: bool,
    pub give_error: bool,
}

#[async_trait]
impl MALClientTrait for MockMALClient {
    fn new(
        client_secret: String,
        dirs: PathBuf,
        access_token: String,
        client: Client,
        caching: bool,
        need_auth: bool,
    ) -> Self {
        Self { client_secret, dirs, access_token, client, caching, need_auth, give_error: false }
    }
    fn with_access_token(token: &str) -> Self {
        Self {
            client_secret: String::new(),
            need_auth: false,
            dirs: PathBuf::new(),
            access_token: token.to_owned(),
            client: reqwest::Client::new(),
            caching: false,
            give_error: false,
        }
    }
    fn set_cache_dir(&mut self, dir: PathBuf) {
        self.dirs = dir;
    }
    fn set_caching(&mut self, caching: bool) {
        self.caching = caching;
    }
    fn get_auth_parts(&self) -> (String, String, String) {
        let verifier = pkce::code_verifier(128);
        let challenge = pkce::code_challenge(&verifier);
        let state = String::new();
        let url = format!("https://example.com/&client_id={}&code_challenge={}", self.client_secret, challenge);
        (url, challenge, state)
    }
    async fn auth(
        &mut self,
        callback_url: &str,
        challenge: &str,
        state: &str,
    ) -> Result<(), MALError> {
        self.need_auth = false;
        self.access_token = String::from("Auth done");
        Ok(())
    }
    fn get_access_token(&self) -> &str {
        &self.access_token
    }
    /// answers for get_anime_list("one", Some(4))
    async fn get_anime_list(
        &self,
        query: &str,
        limit: impl Into<Option<u8>> + Send,
    ) -> Result<AnimeList, MALError> {
        let anime_list = serde_json::from_str::<AnimeList>(include_str!("test-data/anime_list.json")).unwrap();
        Ok(anime_list)
    }
    /// answers for get_anime_details(30230, AnimeFields::ALL)
    async fn get_anime_details(
        &self,
        id: u32,
        fields: impl Into<Option<AnimeFields>> + Send,
    ) -> Result<AnimeDetails, MALError> {
        match id {
            21 => Ok(serde_json::from_str::<AnimeDetails>(include_str!("test-data/one_piece_details.json")).unwrap()),
            30230 => Ok(serde_json::from_str::<AnimeDetails>(include_str!("test-data/anime_details.json")).unwrap()),
            _ => Err(MALError::new("Not found", "error", Some(String::from("info")))),
        }
    }
    /// answers for get_anime_ranking(RankingType::All, Some(4))
    async fn get_anime_ranking(
        &self,
        ranking_type: RankingType,
        limit: impl Into<Option<u8>> + Send,
    ) -> Result<AnimeList, MALError> {
        let anime_ranking = serde_json::from_str::<AnimeList>(include_str!("test-data/anime_ranking.json")).unwrap();
        Ok(anime_ranking)
    }
    /// likely answers for get_seasonal_anime(Season::Summer, 2017, Some(4))
    async fn get_seasonal_anime(
        &self,
        season: Season,
        year: u32,
        limit: impl Into<Option<u8>> + Send,
    ) -> Result<AnimeList, MALError> {
        let seasonal_anime = serde_json::from_str::<AnimeList>(include_str!("test-data/seasonal_anime.json")).unwrap();
        Ok(seasonal_anime)
    }
    /// WARNING: answers like get_anime_list("one", Some(4)) would
    async fn get_suggested_anime(
        &self,
        limit: impl Into<Option<u8>> + Send,
    ) -> Result<AnimeList, MALError> {
        let anime_list = serde_json::from_str::<AnimeList>(include_str!("test-data/anime_list.json")).unwrap();
        Ok(anime_list)
    }
    /// return back given status
    async fn update_user_anime_status(
        &self,
        id: u32,
        update: StatusUpdate,
    ) -> Result<ListStatus, MALError> {
        let update_params: HashMap<&str, String> =
            update.get_params().iter().map(|(k, v)| (*k, v.clone())).collect();
        let list_status = ListStatus {
            status: update_params.get("status").cloned(),
            num_episodes_watched: update_params
                .get("num_episodes_watched")
                .map(|v| v.parse().unwrap_or_default()),
            score: update_params
                .get("score")
                .map(|v| v.parse().unwrap_or_default()),
            updated_at: update_params.get("updated_at").cloned(),
            is_rewatching: update_params
                .get("is_rewatching")
                .map(|v| FromStr::from_str(v).unwrap_or_default()),
            num_times_rewatched: update_params
                .get("num_times_rewatched")
                .map(|v| v.parse().unwrap_or_default()),
            priority: update_params
                .get("priority")
                .map(|v| v.parse().unwrap_or_default()),
            rewatch_value: update_params
                .get("rewatch_value")
                .map(|v| v.parse().unwrap_or_default()),
            tags: update_params
                .get("tags")
                .map(|str| str.split(',').map(String::from).collect()),
            comments: update_params.get("status").cloned(),
            start_date: update_params.get("status").cloned(),
            finish_date: update_params.get("status").cloned(),
        };
        Ok(list_status)
    }
    /// WARNING: answers like get_anime_list("one", Some(4)) would
    async fn get_user_anime_list(&self) -> Result<AnimeList, MALError> {
        let anime_list = serde_json::from_str::<AnimeList>(include_str!("test-data/anime_list.json")).unwrap();
        Ok(anime_list)
    }
    async fn delete_anime_list_item(&self, id: u32) -> Result<(), MALError> {
        Ok(())
    }
    /// WARNING: returns an empty struct
    async fn get_forum_boards(&self) -> Result<ForumBoards, MALError> {
        let forum_boards = ForumBoards {
            categories: Vec::new()
        };
        Ok(forum_boards)
    }
    /// WARNING: returns an empty struct
    async fn get_forum_topic_detail(
        &self,
        topic_id: u32,
        limit: impl Into<Option<u8>> + Send,
    ) -> Result<TopicDetails, MALError> {
        let topic_details = TopicDetails {
            data: Vec::new(),
            paging: HashMap::new(),
        };
        Ok(topic_details)
    }
    /// WARNING: returns an empty struct
    async fn get_forum_topics(
        &self,
        board_id: impl Into<Option<u32>> + Send,
        subboard_id: impl Into<Option<u32>> + Send,
        query: impl Into<Option<String>> + Send,
        topic_user_name: impl Into<Option<String>> + Send,
        user_name: impl Into<Option<String>> + Send,
        limit: impl Into<Option<u32>> + Send,
    ) -> Result<ForumTopics, MALError> {
        let forum_topics = ForumTopics {
            data: Vec::new(),
            paging: Vec::new(),
        };
        Ok(forum_topics)
    }
    /// WARNING: anime_statistics are empty
    async fn get_my_user_info(&self) -> Result<User, MALError> {
        let user = User {
            id: 727,
            name: String::from("Mocked user"),
            location: String::from("Space"),
            joined_at: String::from("2016-01-02T06:03:11+00:00"),
            anime_statistics: HashMap::new(),
        };
        Ok(user)
    }
    /// WARNING: returns an empty struct
    async fn get_anime_episodes(&self, id: u32) -> Result<EpisodesList, MALError> {
        let episodes_list = EpisodesList {
            data: Vec::new(),
            pagination: HashMap::new(),
        };
        Ok(episodes_list)
    }
    fn need_auth(&self) -> bool {
        self.need_auth
    }
}