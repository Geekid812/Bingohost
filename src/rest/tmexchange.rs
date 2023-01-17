use std::str::FromStr;

use reqwest::{Client, Url};
use serde::Deserialize;

use crate::config::routes::tmexchange;
use crate::gamemap::GameMap;

pub async fn get_randomtmx(client: &Client, count: usize) -> Result<Vec<GameMap>, reqwest::Error> {
    get_maps(
        client,
        Url::from_str(&format!("{}{}", tmexchange::BASE, tmexchange::MAP_SEARCH))
            .expect("map search url to be valid"),
        &[
            ("api", "on"),
            ("random", "1"),
            ("mtype", "TM_Race"),
            ("etags", "23,37,40"),
            ("vehicles", "1"),
        ],
        count,
    )
    .await
}

pub async fn get_totd(client: &Client, count: usize) -> Result<Vec<GameMap>, reqwest::Error> {
    get_maps(
        client,
        Url::from_str(&format!("{}{}", tmexchange::BASE, tmexchange::MAP_SEARCH))
            .expect("map search url to be valid"),
        &[("api", "on"), ("random", "1"), ("mode", "25")],
        count,
    )
    .await
}

async fn get_maps(
    client: &Client,
    url: Url,
    params: &[(&str, &str)],
    mut count: usize,
) -> Result<Vec<GameMap>, reqwest::Error> {
    let mut maps = Vec::with_capacity(count);
    while count > 0 {
        let map: MapsResult = client
            .get(url.clone())
            .query(params)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        maps.push(map.results[0].clone().into());
        count -= 1;
    }
    Ok(maps)
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
struct TMExchangeMap {
    #[serde(rename = "TrackID")]
    track_id: i64,
    #[serde(rename = "TrackUID")]
    track_uid: String,
    name: String,
    username: String,
}

impl Into<GameMap> for TMExchangeMap {
    fn into(self) -> GameMap {
        GameMap {
            track_id: self.track_id,
            uid: self.track_uid,
            name: self.name,
            author_name: self.username,
        }
    }
}

#[derive(Deserialize)]
struct MapsResult {
    results: [TMExchangeMap; 1],
}
