use std::time::{Duration, Instant};

use anyhow::anyhow;
use parking_lot::Mutex;
use rand::seq::SliceRandom;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde::Serialize;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::time::sleep;
use tracing::{info, warn};

use crate::{
    config,
    gameroom::MapMode,
    rest::tmexchange::{get_mappack_tracks, get_randomtmx, get_totd},
};

pub type MapResult = anyhow::Result<Vec<GameMap>>;
pub type Sender = UnboundedSender<(MapMode, usize)>;
pub type Receiver = UnboundedReceiver<(MapMode, usize)>;

pub struct MapStock {
    totd: Mutex<MapQueue>,
    random_tmx: Mutex<MapQueue>,
    notifier: Sender,
    client: Client,
}

impl MapStock {
    pub fn new(size: usize, notifier: Sender) -> Self {
        notifier
            .send((MapMode::TOTD, size))
            .expect("MapStock notifier to be used");
        notifier
            .send((MapMode::RandomTMX, size))
            .expect("MapStock notifier to be used");

        let mut headers = HeaderMap::new();
        headers.insert(
            "user-agent",
            HeaderValue::from_static(config::TMX_USERAGENT),
        );

        Self {
            totd: Mutex::new(MapQueue::new(size)),
            random_tmx: Mutex::new(MapQueue::new(size)),
            notifier,
            client: Client::builder()
                .timeout(config::TMX_FETCH_TIMEOUT)
                .default_headers(headers)
                .build()
                .expect("Client to be built"),
        }
    }

    pub async fn fetch_loop(&self, mut rx: Receiver) {
        loop {
            match rx.recv().await {
                Some((mode, count)) => {
                    let queue_full = match mode {
                        MapMode::TOTD => self.totd.lock().size() >= config::MAP_QUEUE_CAPACITY,
                        MapMode::RandomTMX => {
                            self.random_tmx.lock().size() >= config::MAP_QUEUE_CAPACITY
                        }
                        MapMode::Mappack => false,
                    };
                    if queue_full {
                        warn!("map queue for {:?} is full, doing nothing", mode);
                        continue;
                    }
                    info!("loading {} maps for mode {:?}", count, mode);
                    let (result, queue) = match mode {
                        MapMode::TOTD => (get_totd(&self.client, count).await, &self.totd),
                        MapMode::RandomTMX => {
                            (get_randomtmx(&self.client, count).await, &self.random_tmx)
                        }
                        MapMode::Mappack => continue, // mappacks don't have queues, this should be unreachable
                    };
                    match result {
                        Ok(maps) => {
                            let added = queue.lock().extend(maps);
                            tracing::info!(
                                "map queue for {:?} was extended by {} maps",
                                mode,
                                added
                            );
                        }
                        Err(e) => tracing::error!("fetch_loop failure: {}", e),
                    };
                }
                None => return,
            };
        }
    }

    pub async fn get_maps(&self, query: &MapQuery) -> MapResult {
        self.notifier
            .send((query.mode, query.count))
            .expect("MapStock notifier failed");

        match query.mode {
            MapMode::TOTD => Self::get_from_queue(&self.totd, query.count).await,
            MapMode::RandomTMX => Self::get_from_queue(&self.random_tmx, query.count).await,
            MapMode::Mappack => {
                self.get_mappack(
                    query
                        .mappack_id
                        .expect("mode is mappack and mappack id should exist"),
                    query.count,
                )
                .await
            }
        }
    }

    pub fn extend_maps(&self, mode: MapMode, maps: Vec<GameMap>) {
        info!("Replacing {} maps for mode {:?}", maps.len(), mode);
        match mode {
            MapMode::TOTD => {
                Self::extend_queue(&self.totd, maps);
            }
            MapMode::RandomTMX => {
                Self::extend_queue(&self.random_tmx, maps);
            }
            MapMode::Mappack => (),
        }
    }

    fn extend_queue(queue: &Mutex<MapQueue>, maps: Vec<GameMap>) -> usize {
        queue.lock().extend(maps)
    }

    async fn get_mappack(&self, tmxid: u32, count: usize) -> MapResult {
        get_mappack_tracks(&self.client, tmxid)
            .await
            .map_err(|e|
                if e.is_decode() {
                    return anyhow!("Invalid reply from TMX server. The mappack may not exist, or it is hidden/unreleased.");
                } else {
                    return anyhow::Error::new(e);
                }
            )
            .and_then(|mut maps| {
                if maps.len() < count {
                    return Err(anyhow!(
                        "Insufficient maps in the mappack: needs {} tracks, but only has {}",
                        count,
                        maps.len(),
                    ));
                }
                maps.shuffle(&mut rand::thread_rng());
                Ok(maps.into_iter().take(count).collect())
            })
    }

    async fn get_from_queue(queue: &Mutex<MapQueue>, count: usize) -> MapResult {
        let timeout = Instant::now() + config::TMX_FETCH_TIMEOUT;
        loop {
            {
                let mut lock = queue.lock();
                let result = lock.get(count);
                if let Some(maps) = result {
                    return Ok(maps);
                }
            }
            if Instant::now() > timeout {
                return Err(anyhow!(
                    "Map request to the TMX servers timed out after {}s",
                    config::TMX_FETCH_TIMEOUT.as_secs()
                ));
            }
            sleep(Duration::from_millis(100)).await;
        }
    }
}

pub struct MapQueue {
    stock: Vec<GameMap>,
}

impl MapQueue {
    pub fn new(size: usize) -> Self {
        Self {
            stock: Vec::with_capacity(size),
        }
    }

    pub fn get(&mut self, count: usize) -> Option<Vec<GameMap>> {
        let length = self.stock.len();
        if length < count {
            return None;
        }
        Some(self.stock.split_off(length - count))
    }

    pub fn extend(&mut self, maps: Vec<GameMap>) -> usize {
        let filtered: Vec<GameMap> = maps
            .into_iter()
            .filter(|m| {
                self.stock
                    .iter()
                    .all(|queued| queued.track_id != m.track_id)
            })
            .collect();
        let added = filtered.len();
        self.stock.extend(filtered);
        added
    }

    pub fn size(&self) -> usize {
        self.stock.len()
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct GameMap {
    pub track_id: i64,
    pub uid: String,
    pub name: String,
    pub author_name: String,
}

pub struct MapQuery {
    pub mode: MapMode,
    pub count: usize,
    pub mappack_id: Option<u32>,
}

impl MapQuery {
    pub fn new(mode: MapMode, count: usize, mappack_id: Option<u32>) -> Self {
        Self {
            mode,
            count,
            mappack_id,
        }
    }
}
