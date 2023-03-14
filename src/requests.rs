use serde::{Deserialize, Serialize};

use crate::{
    gameroom::{Medal, RoomConfiguration, RoomStatus},
    gameteam::GameTeam,
    handlers::{Request, Response},
    sync::SyncPacket,
};

#[derive(Deserialize, Debug)]
pub struct BaseRequest {
    #[serde(rename = "seq")]
    sequence: u32,
    #[serde(flatten)]
    pub request: Box<dyn Request>,
}

impl BaseRequest {
    pub fn build_reply(&self, response: Box<dyn Response>) -> BaseResponse {
        BaseResponse {
            sequence: self.sequence,
            response: response,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct BaseResponse {
    #[serde(rename = "seq")]
    sequence: u32,
    #[serde(flatten)]
    pub response: Box<dyn Response>,
}

// #[derive(Deserialize)]
// #[serde(tag = "request")]
// pub enum Request {
//     Ping,
//     CreateRoom(CreateRoomRequest),
//     JoinRoom {
//         join_code: String,
//     },
//     EditRoomConfig {
//         config: RoomConfiguration,
//     },
//     CreateTeam,
//     StartGame,
//     ClaimCell {
//         uid: String,
//         time: u64,
//         medal: Medal,
//     },
//     Sync,
// }

// #[derive(Serialize)]
// #[serde(untagged)]
// pub enum Response {
//     Pong,
//     Ok,
//     Error {
//         error: String,
//     },
//     CreateRoom(CreateRoomResponse),
//     JoinRoom {
//         name: String,
//         config: RoomConfiguration,
//         status: RoomStatus,
//     },
//     Sync(SyncPacket),
// }

#[derive(Deserialize)]
pub struct CreateRoomRequest {
    #[serde(flatten)]
    pub config: RoomConfiguration,
}

#[derive(Serialize)]
pub struct CreateRoomResponse {
    pub name: String,
    pub join_code: String,
    pub max_teams: usize,
    pub teams: Vec<GameTeam>,
}
