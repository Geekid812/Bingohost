use serde::{Deserialize, Serialize};

use crate::gameroom::RoomStatus;

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
}
