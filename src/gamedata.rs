use std::time::SystemTime;

use serde::Serialize;

use crate::gameroom::{Medal, NetworkPlayer};

#[derive(Serialize)]
pub struct ActiveGameData {
    start_time: SystemTime,
    cells: Vec<MapCell>,
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

#[derive(Serialize)]
pub struct MapClaim {
    player: NetworkPlayer,
    time: u64,
    medal: Medal,
}
