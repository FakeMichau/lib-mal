use crate::{model::{
    fields::AnimeFields,
    options::{Params, RankingType, Season, StatusUpdate},
    AnimeDetails, AnimeList, EpisodesList, ForumBoards, ForumTopics, ListStatus, TopicDetails, User,
}, prelude::EpisodeNode};
use async_trait::async_trait;
use reqwest::Client;
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write, path::PathBuf, str, time::SystemTime};
use tiny_http::{Response, Server};

use crate::MALError;

use aes_gcm::{aead::Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};

///Exposes all of the API functions for the [MyAnimeList API](https://myanimelist.net/apiconfig/references/api/v2)
///
///**With the exception of all the manga-related functions which haven't been implemented yet**
///
///# Example
///```no_run
/// use lib_mal::ClientBuilder;
/// # use lib_mal::MALError;
/// # async fn test() -> Result<(), MALError> {
/// let client = ClientBuilder::new().secret("[YOUR_SECRET_HERE]".to_string()).build_no_refresh();
/// //--do authorization stuff before accessing the functions--//
///
/// //Gets the details with all fields for Mobile Suit Gundam
/// let anime = client.get_anime_details(80, None).await?;
/// //You should actually handle the potential error
/// println!("Title: {} | Started airing: {} | Finished airing: {}",
///     anime.show.title,
///     anime.start_date.unwrap(),
///     anime.end_date.unwrap());
/// # Ok(())
/// # }
///```
#[allow(clippy::module_name_repetitions)]
pub struct MALClient {
    client_secret: String,
    dirs: PathBuf,
    access_token: String,
    client: reqwest::Client,
    caching: bool,
    pub need_auth: bool,
}

#[async_trait]
pub trait MALClientTrait {
    fn new(
        client_secret: String,
        dirs: PathBuf,
        access_token: String,
        client: Client,
        caching: bool,
        need_auth: bool,
    ) -> Self;
    fn with_access_token(token: &str) -> Self;
    fn set_cache_dir(&mut self, dir: PathBuf);
    fn set_caching(&mut self, caching: bool);
    fn get_auth_parts(&self) -> (String, String, String);
    async fn auth(
        &mut self,
        callback_url: &str,
        challenge: &str,
        state: &str,
    ) -> Result<(), MALError>;
    fn get_access_token(&self) -> &str;
    async fn get_anime_list(
        &self,
        query: &str,
        limit: impl Into<Option<u8>> + Send,
    ) -> Result<AnimeList, MALError>;
    async fn get_anime_details(
        &self,
        id: usize,
        fields: impl Into<Option<AnimeFields>> + Send,
    ) -> Result<AnimeDetails, MALError>;
    async fn get_anime_ranking(
        &self,
        ranking_type: RankingType,
        limit: impl Into<Option<u8>> + Send,
    ) -> Result<AnimeList, MALError>;
    async fn get_seasonal_anime(
        &self,
        season: Season,
        year: usize,
        limit: impl Into<Option<u8>> + Send,
    ) -> Result<AnimeList, MALError>;
    async fn get_suggested_anime(
        &self,
        limit: impl Into<Option<u8>> + Send,
    ) -> Result<AnimeList, MALError>;
    async fn update_user_anime_status(
        &self,
        id: usize,
        update: StatusUpdate,
    ) -> Result<ListStatus, MALError>;
    async fn get_user_anime_list(&self) -> Result<AnimeList, MALError>;
    async fn delete_anime_list_item(&self, id: usize) -> Result<(), MALError>;
    async fn get_forum_boards(&self) -> Result<ForumBoards, MALError>;
    async fn get_forum_topic_detail(
        &self,
        topic_id: usize,
        limit: impl Into<Option<u8>> + Send,
    ) -> Result<TopicDetails, MALError>;
    async fn get_forum_topics(
        &self,
        board_id: impl Into<Option<usize>> + Send,
        subboard_id: impl Into<Option<usize>> + Send,
        query: impl Into<Option<String>> + Send,
        topic_user_name: impl Into<Option<String>> + Send,
        user_name: impl Into<Option<String>> + Send,
        limit: impl Into<Option<usize>> + Send,
    ) -> Result<ForumTopics, MALError>;
    async fn get_my_user_info(&self) -> Result<User, MALError>;
    async fn get_anime_episodes(&self, id: usize, precise_score: bool) -> Result<EpisodesList, MALError>;
    fn need_auth(&self) -> bool;
}

#[async_trait]
impl MALClientTrait for MALClient {
    fn new(
        client_secret: String,
        dirs: PathBuf,
        access_token: String,
        client: Client,
        caching: bool,
        need_auth: bool,
    ) -> Self {
        Self { client_secret, dirs, access_token, client, caching, need_auth }
    }
    ///Creates a client using provided token. Caching is disable by default.
    ///
    ///A client created this way can't authenticate the user if needed because it lacks a
    ///`client_secret`
    fn with_access_token(token: &str) -> Self {
        Self {
            client_secret: String::new(),
            need_auth: false,
            dirs: PathBuf::new(),
            access_token: token.to_owned(),
            client: reqwest::Client::new(),
            caching: false,
        }
    }

    ///Sets the directory the client will use for the token cache
    fn set_cache_dir(&mut self, dir: PathBuf) {
        self.dirs = dir;
    }

    ///Sets wether the client will cache or not
    fn set_caching(&mut self, caching: bool) {
        self.caching = caching;
    }

    ///Returns the auth URL and code challenge which will be needed to authorize the user.
    ///
    ///# Example
    ///#[async_trait]
    ///```no_run
    ///     use lib_mal::ClientBuilder;
    ///     # use  lib_mal::MALError;
    ///     # async fn test() -> Result<(), MALError> {
    ///     let redirect_uri = "http://localhost:2525";//<-- example uri
    ///     let mut client =
    ///     ClientBuilder::new().secret("[YOUR_SECRET_HERE]".to_string()).build_no_refresh();
    ///     let (url, challenge, state) = client.get_auth_parts();
    ///     println!("Go here to log in: {}", url);
    ///     client.auth(&redirect_uri, &challenge, &state).await?;
    ///     # Ok(())
    ///     # }
    ///```
    fn get_auth_parts(&self) -> (String, String, String) {
        let verifier = pkce::code_verifier(128);
        let challenge = pkce::code_challenge(&verifier);
        let random = Box::into_raw(Box::new(727)) as u16;
        let state = random.to_string();
        let url = format!("https://myanimelist.net/v1/oauth2/authorize?response_type=code&client_id={}&code_challenge={}&state={}", self.client_secret, challenge, state, );
        (url, challenge, state)
    }

    ///Listens for the `OAuth2` callback from MAL on `callback_url`, which is the `redirect_uri`
    ///registered when obtaining the API token from MAL. Only HTTP URIs are supported right now.
    ///
    ///# NOTE
    ///
    ///For now only applications with a single registered URI are supported, having more than one
    ///seems to cause issues with the MAL api itself
    ///
    ///# Example
    ///
    ///```no_run
    ///     use lib_mal::ClientBuilder;
    ///     # use lib_mal::MALError;
    ///     # async fn test() -> Result<(), MALError> {
    ///     let redirect_uri = "localhost:2525";//<-- example uri,
    ///     //appears as "http://localhost:2525" in the API settings
    ///     let mut client = ClientBuilder::new().secret("[YOUR_SECRET_HERE]".to_string()).build_no_refresh();
    ///     let (url, challenge, state) = client.get_auth_parts();
    ///     println!("Go here to log in: {}", url);
    ///     client.auth(&redirect_uri, &challenge, &state).await?;
    ///     # Ok(())
    ///     # }
    ///
    ///```
    async fn auth(
        &mut self,
        callback_url: &str,
        challenge: &str,
        state: &str,
    ) -> Result<(), MALError> {
        let mut code = String::new();
        let url = if callback_url.contains("http") {
            //server won't work if the url has the protocol in it
            callback_url
                .trim_start_matches("http://")
                .trim_start_matches("https://")
        } else {
            callback_url
        };

        let server = Server::http(url).unwrap();
        for i in server.incoming_requests() {
            if !i.url().contains(&format!("state={state}")) {
                //if the state doesn't match, discard this response
                continue;
            }
            let res_raw = i.url();
            code = res_raw
                .split_once('=')
                .unwrap()
                .1
                .split_once('&')
                .unwrap()
                .0
                .to_owned();
            let response = Response::from_string("You're logged in! You can now close this window");
            i.respond(response).unwrap();
            break;
        }
        self.need_auth = false;
        self.get_tokens(&code, challenge).await
    }
    ///Returns the current access token. Intended mostly for debugging.
    ///
    ///# Example
    ///
    ///```no_run
    /// # use lib_mal::ClientBuilder;
    /// # use lib_mal::MALError;
    /// # use std::path::PathBuf;
    /// # async fn test() -> Result<(), MALError> {
    ///     let client = ClientBuilder::new().secret("[YOUR_SECRET_HERE]".to_string()).caching(true).cache_dir(Some(PathBuf::new())).build_with_refresh().await?;
    ///     let token = client.get_access_token();
    ///     Ok(())
    /// # }
    ///```
    fn get_access_token(&self) -> &str {
        &self.access_token
    }

    //Begin API functions

    //--Anime functions--//
    ///Gets a list of anime based on the query string provided
    ///`limit` defaults to 100 if `None`
    ///
    ///# Example
    ///
    ///```no_run
    /// # use lib_mal::MALClient;
    /// # use lib_mal::MALError;
    /// # async fn test() -> Result<(), MALError> {
    ///     # let client = MALClient::with_access_token("[YOUR_SECRET_HERE]");
    ///     let list = client.get_anime_list("Mobile Suit Gundam", None).await?;
    ///     # Ok(())
    /// # }
    ///```
    async fn get_anime_list(
        &self,
        query: &str,
        limit: impl Into<Option<u8>> + Send,
    ) -> Result<AnimeList, MALError> {
        let url = format!(
            "https://api.myanimelist.net/v2/anime?q={}&limit={}",
            query,
            limit.into().unwrap_or(100)
        );
        let res = self.do_request(url).await?;
        Self::parse_response(&res)
    }

    ///Gets the details for an anime by the show's ID.
    ///Only returns the fields specified in the `fields` parameter
    ///
    ///Returns all fields when supplied `None`
    ///
    ///# Example
    ///
    ///```no_run
    /// use lib_mal::model::fields::AnimeFields;
    /// # use lib_mal::{MALError, MALClient};
    /// # async fn test() -> Result<(), MALError> {
    ///     # let client = MALClient::with_access_token("[YOUR_SECRET_HERE]");
    /// //returns an AnimeDetails struct with just the Rank, Mean, and Studio data for Mobile Suit Gundam
    /// let res = client.get_anime_details(80, AnimeFields::Rank | AnimeFields::Mean | AnimeFields::Studios).await?;
    /// # Ok(())
    /// # }
    ///
    ///```
    ///
    async fn get_anime_details(
        &self,
        id: usize,
        fields: impl Into<Option<AnimeFields>> + Send,
    ) -> Result<AnimeDetails, MALError> {
        let url = fields.into().map_or_else(|| format!(
                "https://api.myanimelist.net/v2/anime/{}?fields={}",
                id,
                AnimeFields::ALL
            ), |f| format!("https://api.myanimelist.net/v2/anime/{id}?fields={f}"));
        let res = self.do_request(url).await?;
        Self::parse_response(&res)
    }

    ///Gets a list of anime ranked by `RankingType`
    ///
    ///`limit` defaults to the max of 100 when `None`
    ///
    ///# Example
    ///
    ///```no_run
    /// # use lib_mal::{MALError, MALClient};
    /// use lib_mal::model::options::RankingType;
    /// # async fn test() -> Result<(), MALError> {
    ///     # let client = MALClient::with_access_token("[YOUR_SECRET_HERE]");
    /// // Gets a list of the top 5 most popular anime
    /// let ranking_list = client.get_anime_ranking(RankingType::ByPopularity, 5).await?;
    /// # Ok(())
    /// # }
    ///
    ///```
    async fn get_anime_ranking(
        &self,
        ranking_type: RankingType,
        limit: impl Into<Option<u8>> + Send,
    ) -> Result<AnimeList, MALError> {
        let url = format!(
            "https://api.myanimelist.net/v2/anime/ranking?ranking_type={}&limit={}",
            ranking_type,
            limit.into().unwrap_or(100)
        );
        let res = self.do_request(url).await?;
        Ok(serde_json::from_str(&res).unwrap())
    }

    ///Gets the anime for a given season in a given year
    ///
    ///`limit` defaults to the max of 100 when `None`
    ///
    ///# Example
    ///
    ///```no_run
    /// # use lib_mal::{MALClient, MALError};
    /// use lib_mal::model::options::Season;
    /// # async fn test() -> Result<(), MALError> {
    ///     # let client = MALClient::with_access_token("[YOUR_SECRET_HERE]");
    ///     let summer_2019 = client.get_seasonal_anime(Season::Summer, 2019, None).await?;
    ///     # Ok(())
    /// # }
    ///```
    async fn get_seasonal_anime(
        &self,
        season: Season,
        year: usize,
        limit: impl Into<Option<u8>> + Send,
    ) -> Result<AnimeList, MALError> {
        let url = format!(
            "https://api.myanimelist.net/v2/anime/season/{}/{}?limit={}",
            year,
            season,
            limit.into().unwrap_or(100)
        );
        let res = self.do_request(url).await?;
        Self::parse_response(&res)
    }

    ///Returns the suggested anime for the current user. Can return an empty list if the user has
    ///no suggestions.
    ///
    ///# Example
    ///
    ///```no_run
    /// # use lib_mal::{MALClient, MALError};
    /// # async fn test() -> Result<(), MALError> {
    ///     # let client = MALClient::with_access_token("[YOUR_SECRET_HERE]");
    ///     let suggestions = client.get_suggested_anime(10).await?;
    ///     # Ok(())
    /// # }
    ///```
    async fn get_suggested_anime(
        &self,
        limit: impl Into<Option<u8>> + Send,
    ) -> Result<AnimeList, MALError> {
        let url = format!(
            "https://api.myanimelist.net/v2/anime/suggestions?limit={}",
            limit.into().unwrap_or(100)
        );
        let res = self.do_request(url).await?;
        Self::parse_response(&res)
    }

    //--User anime list functions--//

    ///Adds an anime to the list, or updates the element if it already exists
    ///
    ///# Example
    ///
    ///```no_run
    /// # use lib_mal::{MALClient, MALError};
    /// use lib_mal::model::StatusBuilder;
    /// use lib_mal::model::options::Status;
    /// # async fn test() -> Result<(), MALError> {
    ///     # let client = MALClient::with_access_token("[YOUR_SECRET_HERE]");
    ///     // add a new anime to the user's list
    ///     let updated_status = client.update_user_anime_status(80, StatusBuilder::new().status(Status::Watching).build()).await?;
    ///     // or update an existing one
    ///     let new_status = StatusBuilder::new().status(Status::Dropped).num_watched_episodes(2).build();
    ///     let updated_status = client.update_user_anime_status(32981, new_status).await?;
    ///
    ///     # Ok(())
    ///
    /// # }
    ///```
    async fn update_user_anime_status(
        &self,
        id: usize,
        update: StatusUpdate,
    ) -> Result<ListStatus, MALError> {
        let params = update.get_params();
        let url = format!("https://api.myanimelist.net/v2/anime/{id}/my_list_status");
        let res = self.do_request_forms(url, params).await?;
        Self::parse_response(&res)
    }

    ///Returns the user's full anime list as an `AnimeList` struct.
    ///
    ///# Example
    ///
    ///```no_run
    /// # use lib_mal::{MALClient, MALError};
    /// # async fn test() -> Result<(), MALError> {
    ///     # let client = MALClient::with_access_token("[YOUR_SECRET_HERE]");
    ///     let my_list = client.get_user_anime_list().await?;
    ///     # Ok(())
    ///
    /// # }
    ///```
    async fn get_user_anime_list(&self) -> Result<AnimeList, MALError> {
        let url = "https://api.myanimelist.net/v2/users/@me/animelist?fields=list_status&limit=4";
        let res = self.do_request(url.to_owned()).await?;

        Self::parse_response(&res)
    }

    ///Deletes the anime with `id` from the user's anime list
    ///
    ///# Note
    /// The [API docs from MAL](https://myanimelist.net/apiconfig/references/api/v2#operation/anime_anime_id_my_list_status_delete) say this method should return 404 if the anime isn't in the user's
    /// list, but in my testing this wasn't true. Without that there's no way to tell if the item
    /// was actually deleted or not.
    ///
    ///# Example
    ///
    ///```no_run
    /// # use lib_mal::{MALClient, MALError};
    /// # async fn test() -> Result<(), MALError> {
    ///     # let client = MALClient::with_access_token("[YOUR_SECRET_HERE]");
    ///     client.delete_anime_list_item(80).await?;
    ///     # Ok(())
    /// # }
    ///```
    async fn delete_anime_list_item(&self, id: usize) -> Result<(), MALError> {
        let url = format!("https://api.myanimelist.net/v2/anime/{id}/my_list_status");
        let res = self
            .client
            .delete(url)
            .bearer_auth(&self.access_token)
            .send()
            .await;
        match res {
            Ok(r) => {
                if r.status() == StatusCode::NOT_FOUND {
                    Err(MALError::new(
                        &format!("Anime {id} not found"),
                        r.status().as_str(),
                        None,
                    ))
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(MALError::new(
                "Unable to send request",
                &format!("{e}"),
                None,
            )),
        }
    }

    //--Forum functions--//

    ///Returns a vector of `HashMap`s that represent all the forum boards on MAL
    async fn get_forum_boards(&self) -> Result<ForumBoards, MALError> {
        let res = self
            .do_request("https://api.myanimelist.net/v2/forum/boards".to_owned())
            .await?;
        Self::parse_response(&res)
    }

    ///Returns details of the specified topic
    async fn get_forum_topic_detail(
        &self,
        topic_id: usize,
        limit: impl Into<Option<u8>> + Send,
    ) -> Result<TopicDetails, MALError> {
        let url = format!(
            "https://api.myanimelist.net/v2/forum/topic/{}?limit={}",
            topic_id,
            limit.into().unwrap_or(100)
        );
        let res = self.do_request(url).await?;
        Self::parse_response(&res)
    }

    ///Returns all topics for a given query
    async fn get_forum_topics(
        &self,
        board_id: impl Into<Option<usize>> + Send,
        subboard_id: impl Into<Option<usize>> + Send,
        query: impl Into<Option<String>> + Send,
        topic_user_name: impl Into<Option<String>> + Send,
        user_name: impl Into<Option<String>> + Send,
        limit: impl Into<Option<usize>> + Send,
    ) -> Result<ForumTopics, MALError> {
        let params = {
            let mut tmp = vec![];
            if let Some(bid) = board_id.into() {
                tmp.push(format!("board_id={bid}"));
            }
            if let Some(bid) = subboard_id.into() {
                tmp.push(format!("subboard_id={bid}"));
            }
            if let Some(bid) = query.into() {
                tmp.push(format!("q={bid}"));
            }
            if let Some(bid) = topic_user_name.into() {
                tmp.push(format!("topic_user_name={bid}"));
            }
            if let Some(bid) = user_name.into() {
                tmp.push(format!("user_name={bid}"));
            }
            tmp.push(format!("limit={}", limit.into().unwrap_or(100)));
            tmp.join(",")
        };
        let url = format!("https://api.myanimelist.net/v2/forum/topics?{params}");
        let res = self.do_request(url).await?;
        Self::parse_response(&res)
    }

    ///Gets the details for the current user
    ///
    ///# Example
    ///
    ///```no_run
    /// # use lib_mal::{MALClient, MALError};
    /// # async fn test() -> Result<(), MALError> {
    ///     # let client = MALClient::with_access_token("[YOUR_SECRET_HERE]");
    ///     let me = client.get_my_user_info().await?;
    ///     # Ok(())
    /// # }
    ///```
    async fn get_my_user_info(&self) -> Result<User, MALError> {
        let url = "https://api.myanimelist.net/v2/users/@me?fields=anime_statistics";
        let res = self.do_request(url.to_owned()).await?;
        Self::parse_response(&res)
    }

    /// Returns just the first page
    async fn get_anime_episodes(&self, id: usize, precise_score: bool) -> Result<EpisodesList, MALError> {
        let page: usize = 1;
        let url = format!(
            "https://api.jikan.moe/v4/anime/{id}/episodes?page={page}",
        );
        let res = self.do_request(url).await?;
        let mut api: Result<EpisodesList, MALError> = match serde_json::from_str(&res) {
            Ok(list) => Ok(list),
            Err(e) => Err(MALError::new(
                "unable to get anime episodes",
                &format!("{e}"),
                res.to_string(),
            )),
        };
        if precise_score {
            let offset = page.checked_sub(1).unwrap_or_default() * 100;
            let extra = self.get_raw_episodes_score(id, offset).await?;
            api.as_mut()
                .map(|list| {
                    list.data.iter_mut().for_each(|episode| {
                        let score = extra
                            .iter()
                            .find(|ee| ee.mal_id == episode.mal_id)
                            .map(|ee| ee.score)
                            .unwrap_or_default();
                        episode.score = score;
                    });
                })
                .map_err(|e| e.clone())?;
        }
        api
    }

    fn need_auth(&self) -> bool {
        self.need_auth
    }
}

impl MALClient {
    async fn get_tokens(&mut self, code: &str, verifier: &str) -> Result<(), MALError> {
        let params = [
            ("client_id", self.client_secret.as_str()),
            ("grant_type", "authorization_code"),
            ("code_verifier", verifier),
            ("code", code),
        ];
        let rec = self
            .client
            .request(Method::POST, "https://myanimelist.net/v1/oauth2/token")
            .form(&params)
            .build()
            .unwrap();
        let response = self.client.execute(rec).await.unwrap();
        let text = response.text().await.unwrap();
        if let Ok(tokens) = serde_json::from_str::<TokenResponse>(&text) {
            self.access_token = tokens.access_token.clone();

            let tjson = Tokens {
                access_token: tokens.access_token,
                refresh_token: tokens.refresh_token,
                expires_in: tokens.expires_in,
                today: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };
            if self.caching {
                let mut f =
                    File::create(self.dirs.join("tokens")).expect("Unable to create token file");
                f.write_all(&encrypt_token(&tjson))
                    .expect("Unable to write tokens");
            }
            Ok(())
        } else {
            Err(MALError::new("Unable to get tokens", "None", text))
        }
    }

    ///Sends a get request to the specified URL with the appropriate auth header
    async fn do_request(&self, url: String) -> Result<String, MALError> {
        match self
            .client
            .get(url)
            .bearer_auth(&self.access_token)
            .send()
            .await
        {
            Ok(res) => Ok(res.text().await.unwrap()),
            Err(e) => Err(MALError::new(
                "Unable to send request",
                &format!("{e}"),
                None,
            )),
        }
    }

    ///Sends a put request to the specified URL with the appropriate auth header and
    ///form encoded parameters
    async fn do_request_forms(
        &self,
        url: String,
        params: Vec<(&str, String)>,
    ) -> Result<String, MALError> {
        match self
            .client
            .put(url)
            .bearer_auth(&self.access_token)
            .form(&params)
            .send()
            .await
        {
            Ok(res) => Ok(res.text().await.unwrap()),
            Err(e) => Err(MALError::new(
                "Unable to send request",
                &format!("{e}"),
                None,
            )),
        }
    }

    ///Tries to parse a JSON response string into the type provided in the `::<>` turbofish
    fn parse_response<'a, T: Serialize + Deserialize<'a>>(
        res: &'a str,
    ) -> Result<T, MALError> {
        serde_json::from_str::<T>(res).map_or_else(|_| Err(match serde_json::from_str::<MALError>(res) {
                Ok(o) => o,
                Err(e) => MALError::new(
                    "unable to parse response",
                    &format!("{e}"),
                    res.to_string(),
                ),
            }), |v| Ok(v))
    }

    /// Returns just the scores from the first page
    async fn get_raw_episodes_score(&self, id: usize, offset: usize) -> Result<Vec<EpisodeNode>, MALError> {
        let url = format!("https://myanimelist.net/anime/{id}/1/episode?offset={offset}");
        let res = self.do_request(url).await?;

        let mut episodes_range_iter = res
            .lines()
            .find(|line| line.contains("Episodes") && line.contains("h2_overwrite"))
            .unwrap_or_default()
            .split(">(")
            .nth(1)
            .unwrap_or_default()
            .split(")<")
            .next()
            .unwrap_or_default()
            .split('/')
            .map(|value| {
                value.replace(',', "").parse::<usize>().unwrap_or_default()
            });

        let present_episodes = episodes_range_iter.next().unwrap_or_default();

        if present_episodes == 0 {
            return Ok(Vec::new());
        }

        let episodes_score: Vec<EpisodeNode> = res
            .lines()
            .filter(|line| line.contains("episode-poll") && line.contains("data-raw"))
            .enumerate()
            .map(|(i, line)| {
                let score = line
                    .split("data-raw=\"")
                    .nth(1)
                    .map(|v| v.split('"'))
                    .and_then(|mut v| v.next())
                    .map(str::parse::<f32>)
                    .and_then(Result::ok);
                (usize::try_from(i).unwrap_or_default() + 1 + offset, score)
            })
            .filter(|(_, score)| score.is_some())
            .map(|(k, score)| {
                EpisodeNode {
                    mal_id: Some(k),
                    score,
                    ..Default::default()
                }
            })
            .collect();
        Ok(episodes_score)
    }
}

#[derive(Deserialize)]
pub struct TokenResponse {
    pub token_type: String,
    pub expires_in: usize,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Serialize, Deserialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: usize,
    pub today: u64,
}

pub fn encrypt_token(toks: &Tokens) -> Vec<u8> {
    let key = Key::<Aes256Gcm>::from_slice(b"one two three four five six seve");
    let cypher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"but the eart");
    let plain = serde_json::to_vec(&toks).unwrap();
    let res = cypher.encrypt(nonce, plain.as_ref()).unwrap();
    res
}

pub fn decrypt_tokens(raw: &[u8]) -> Result<Tokens, MALError> {
    let key = Key::<Aes256Gcm>::from_slice(b"one two three four five six seve");
    let cypher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"but the eart");
    match cypher.decrypt(nonce, raw.as_ref()) {
        Ok(plain) => {
            let text = String::from_utf8(plain).unwrap();
            Ok(serde_json::from_str(&text).expect("couldn't parse decrypted tokens"))
        }
        Err(e) => Err(MALError::new(
            "Unable to decrypt encrypted tokens",
            &format!("{e}"),
            None,
        )),
    }
}
