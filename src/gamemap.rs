use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

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
    rest::tmexchange::{get_randomtmx, get_totd},
};

pub type MapResult = Option<Vec<GameMap>>;
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
                        MapMode::TOTD => {
                            self.totd.lock().expect("lock poisoned").size()
                                >= config::MAP_QUEUE_CAPACITY
                        }
                        MapMode::RandomTMX => {
                            self.random_tmx.lock().expect("lock poisoned").size()
                                >= config::MAP_QUEUE_CAPACITY
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
                        MapMode::Mappack => continue, // TODO
                    };
                    match result {
                        Ok(maps) => {
                            tracing::info!(
                                "map queue for {:?} was extended by {} maps",
                                mode,
                                count
                            );
                            queue.lock().expect("lock poisoned").extend(maps)
                        }
                        Err(e) => tracing::error!("fetch_loop failure: {}", e),
                    };
                }
                None => return,
            };
        }
    }

    pub async fn get_maps(&self, mode: MapMode, count: usize) -> MapResult {
        self.notifier
            .send((mode, count))
            .expect("MapStock notifier failed");

        match mode {
            MapMode::TOTD => Self::get_from_queue(&self.totd, count).await,
            MapMode::RandomTMX => Self::get_from_queue(&self.random_tmx, count).await,
            MapMode::Mappack => None, // TODO
        }
    }

    pub fn extend_maps(&self, mode: MapMode, maps: Vec<GameMap>) {
        info!("Replacing {} maps for mode {:?}", maps.len(), mode);
        match mode {
            MapMode::TOTD => Self::extend_queue(&self.totd, maps),
            MapMode::RandomTMX => Self::extend_queue(&self.random_tmx, maps),
            MapMode::Mappack => (),
        }
    }

    fn extend_queue(queue: &Mutex<MapQueue>, maps: Vec<GameMap>) {
        queue.lock().expect("lock poisioned").extend(maps)
    }

    async fn get_from_queue(queue: &Mutex<MapQueue>, count: usize) -> MapResult {
        let timeout = Instant::now() + config::TMX_FETCH_TIMEOUT;
        loop {
            {
                let mut lock = queue.lock().ok()?;
                let result = lock.get(count);
                if result.is_some() {
                    return result;
                }
            }
            if Instant::now() > timeout {
                return None;
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

    pub fn get(&mut self, count: usize) -> MapResult {
        let length = self.stock.len();
        if length < count {
            return None;
        }
        Some(self.stock.split_off(length - count))
    }

    pub fn extend(&mut self, maps: Vec<GameMap>) {
        self.stock.extend(maps)
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
