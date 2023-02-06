use std::time::Instant;

use serde::{Serialize, Serializer};
use serde_repr::Serialize_repr;

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

    pub fn check_for_bingos(&self, grid_size: usize) -> Vec<BingoLine> {
        let mut bingos = Vec::new();
        // Horizontal
        for i in 0..grid_size {
            let mut line = self.cells[i * grid_size..(i + 1) * grid_size]
                .iter()
                .take(grid_size);
            let first = line
                .next()
                .expect("invalid grid_size")
                .claim
                .as_ref()
                .and_then(|c| c.player.team);
            let unique_team = line.fold(first, |acc, x| {
                acc.and_then(|y| {
                    if x.claim.as_ref().and_then(|c| c.player.team) == Some(y) {
                        Some(y)
                    } else {
                        None
                    }
                })
            });

            if let Some(team) = unique_team {
                bingos.push(BingoLine {
                    direction: Direction::Horizontal,
                    index: i as u32,
                    team,
                });
            }
        }

        bingos
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

#[derive(Serialize, Clone)]
pub struct BingoLine {
    pub direction: Direction,
    pub index: u32,
    pub team: usize,
}

#[derive(Serialize_repr, Clone, Copy)]
#[repr(u32)]
pub enum Direction {
    None = 0,
    Horizontal = 1,
    Vertical = 2,
    Diagonal = 3,
}
