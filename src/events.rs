use crate::{
    gamedata::{BingoLine, MapClaim},
    gamemap::GameMap,
    gameroom::{RoomConfiguration, RoomStatus},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(tag = "event")]
pub enum ClientEvent {
    ChangeTeam { team_id: usize },
    LeaveRoom,
}

#[derive(Serialize)]
#[serde(tag = "event")]
pub enum ServerEvent {
    RoomUpdate(RoomStatus),
    RoomConfigUpdate(RoomConfiguration),
    MapsLoadResult {
        error: Option<String>,
    },
    GameStart {
        maps: Vec<GameMap>,
    },
    CellClaim {
        cell_id: usize,
        claim: MapClaim,
    },
    AnnounceBingo {
        #[serde(flatten)]
        line: BingoLine,
    },
}
