use std::time::Instant;

use serde::{Serialize, Serializer};

use crate::gameroom::{Medal, NetworkPlayer};

#[derive(Serialize, Clone)]
pub struct ActiveGameData {
    #[serde(serialize_with = "serialize_time")]
    pub start_time: Instant,
    pub cells: Vec<MapCell>,
}

impl ActiveGameData {
    pub fn new(cell_count: usize) -> Self {
        let mut cells = Vec::with_capacity(cell_count);
        for _ in 0..cell_count {
            cells.push(MapCell { claim: None });
        }
        Self {
            start_time: Instant::now(),
            cells,
        }
    }
}

fn serialize_time<S: Serializer>(time: &Instant, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_u128(time.elapsed().as_millis())
}

#[derive(Serialize, Clone)]
pub struct MapCell {
    pub claim: Option<MapClaim>,
}

#[derive(Serialize, Clone)]
pub struct MapClaim {
    pub player: NetworkPlayer,
    pub time: u64,
    pub medal: Medal,
}
