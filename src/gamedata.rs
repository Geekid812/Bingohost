use std::time::SystemTime;

use serde::Serialize;

use crate::gameroom::{Medal, NetworkPlayer};

#[derive(Serialize)]
pub struct ActiveGameData {
    pub start_time: SystemTime,
    pub cells: Vec<MapCell>,
}

impl ActiveGameData {
    pub fn new(cell_count: usize) -> Self {
        let mut cells = Vec::with_capacity(cell_count);
        for _ in 0..cell_count {
            cells.push(MapCell { claim: None });
        }
        Self {
            start_time: SystemTime::now(),
            cells,
        }
    }
}

#[derive(Serialize)]
pub struct MapCell {
    pub claim: Option<MapClaim>,
}

#[derive(Serialize, Clone)]
pub struct MapClaim {
    pub player: NetworkPlayer,
    pub time: u64,
    pub medal: Medal,
}
