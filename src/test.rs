use std::env;

use crate::model::fields::AnimeFields;
use crate::model::options::{RankingType, Season};
use crate::model::{AnimeDetails, AnimeList};
use crate::{MALClient, MALClientTrait};

#[tokio::test]
async fn anime_list() {
    let client = setup();
    let expected =
        serde_json::from_str::<AnimeList>(include_str!("test-data/anime_list.json")).unwrap();
    let result = client.get_anime_list("one", Some(4)).await.expect("Error performing request");
    let first = expected.data[0].node.id;
    let res_first = result.data[0].node.id;
    assert_eq!(first, res_first); //Really don't want to implement partial_eq for all these structs lol
}

#[tokio::test]
async fn anime_details() {
    let client = setup();
    let expected =
        serde_json::from_str::<AnimeDetails>(include_str!("test-data/anime_details.json")).unwrap();
    let result = client.get_anime_details(30230, AnimeFields::ALL).await
        .expect("Error performing request");
    let left = expected.show.title;
    let right = result.show.title;
    assert_eq!(left, right);
}

#[tokio::test]
async fn anime_ranking() {
    let client = setup();
    let expected =
        serde_json::from_str::<AnimeList>(include_str!("test-data/anime_ranking.json")).unwrap();
    let result = client.get_anime_ranking(RankingType::All, Some(4)).await
        .expect("Error performing request");
    let left = expected.data[0].node.id;
    let right = result.data[0].node.id;
    assert_eq!(left, right);
}

#[tokio::test]
async fn seasonal_anime() {
    let client = setup();
    // let expected =
    //     serde_json::from_str::<AnimeList>(include_str!("test-data/seasonal_anime.json")).unwrap();
    let result = client.get_seasonal_anime(Season::Summer, 2017, Some(4)).await;
    //.expect("Error performing request");
    // let left = expected.data[0].node.id;
    // let right = result.data[0].node.id;
    assert!(result.is_ok());
}

fn setup() -> MALClient {
    let token = env::var("MAL_TOKEN").expect("Access token not in environment");
    MALClient::with_access_token(&token)
}
