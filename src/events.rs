use crate::gameroom::{RoomConfiguration, RoomStatus};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(tag = "event")]
pub enum ClientEventVariant {
    ChangeTeam { team_id: usize },
    LeaveRoom,
}

#[derive(Serialize)]
#[serde(tag = "event")]
pub enum ServerEventVariant {
    RoomUpdate(RoomStatus),
    RoomConfigUpdate(RoomConfiguration),
    MapsLoadResult { loaded: bool },
}
