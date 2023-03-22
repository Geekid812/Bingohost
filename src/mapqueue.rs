use futures::Future;
use once_cell::sync::Lazy;
use parking_lot::{Mutex, MutexGuard};
use reqwest::Client;
use tokio::join;
use tracing::{debug, warn};

use crate::{
    config,
    gamemap::GameMap,
    gameroom::MapMode,
    rest::tmexchange::{self as tmxapi, MapError},
};

static MXRANDOM_MAP_QUEUE: Mutex<Lazy<Vec<GameMap>>> =
    Mutex::new(Lazy::new(|| Vec::with_capacity(config::MAP_QUEUE_CAPACITY)));
static TOTD_MAP_QUEUE: Mutex<Lazy<Vec<GameMap>>> =
    Mutex::new(Lazy::new(|| Vec::with_capacity(config::MAP_QUEUE_CAPACITY)));
static CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

fn get_tracks_inner<'a>(
    count: usize,
    mut lock: MutexGuard<'a, Lazy<Vec<GameMap>>>,
) -> Result<Vec<GameMap>, usize> {
    let length = lock.len();
    if length < count {
        return Err(length);
    }
    Ok(lock.split_off(length - count))
}

fn pushback_tracks_inner<'a>(maps: Vec<GameMap>, mut lock: MutexGuard<'a, Lazy<Vec<GameMap>>>) {
    lock.extend(maps)
}

pub fn get_tracks(mode: MapMode, count: usize) -> Result<Vec<GameMap>, usize> {
    match mode {
        MapMode::TOTD => get_tracks_inner(count, TOTD_MAP_QUEUE.lock()),
        MapMode::RandomTMX => get_tracks_inner(count, MXRANDOM_MAP_QUEUE.lock()),
        MapMode::Mappack => panic!("get_tracks called with Mappack mode"),
    }
}

pub fn pushback_tracks(mode: MapMode, maps: Vec<GameMap>) {
    match mode {
        MapMode::TOTD => pushback_tracks_inner(maps, TOTD_MAP_QUEUE.lock()),
        MapMode::RandomTMX => pushback_tracks_inner(maps, MXRANDOM_MAP_QUEUE.lock()),
        MapMode::Mappack => panic!("pushback_tracks called with Mappack mode"),
    }
}

pub async fn get_mappack_tracks(
    mappack_id: u32,
    count: usize,
) -> Result<Vec<GameMap>, reqwest::Error> {
    tmxapi::get_mappack_tracks(&CLIENT, mappack_id).await
}

async fn queue_loop<F, Fut>(queue: &Mutex<Lazy<Vec<GameMap>>>, fetch_callback: F)
where
    F: Fn(&'static Client) -> Fut,
    Fut: Future<Output = Result<GameMap, MapError>>,
{
    loop {
        match fetch_callback(&CLIENT).await {
            Ok(map) => queue.lock().push(map),
            Err(e) => match e {
                tmxapi::MapError::Rejected(_) => debug!("map rejected from queue"),
                tmxapi::MapError::Request(e) => warn!("TMX api fetch error: {:?}", e),
            },
        }
        tokio::time::sleep(config::FETCH_INTERVAL);
    }
}

pub async fn run_loop() {
    join!(
        queue_loop(&TOTD_MAP_QUEUE, tmxapi::get_totd),
        queue_loop(&MXRANDOM_MAP_QUEUE, tmxapi::get_randomtmx)
    );
}
