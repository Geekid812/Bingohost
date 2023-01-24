use crate::{
    gamedata::MapClaim,
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
    MapsLoadResult { loaded: bool },
    GameStart { maps: Vec<GameMap> },
    CellClaim { cell_id: usize, claim: MapClaim },
}
