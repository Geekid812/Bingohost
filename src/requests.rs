use serde::{Deserialize, Serialize};

use crate::{
    gameroom::{Medal, RoomConfiguration, RoomStatus},
    gameteam::GameTeam,
};

#[macro_use]
mod macros {
    macro_rules! impl_request {
        ($req:ident, $res:ident) => {
            impl Request for $req {
                type Response = $res;
            }

            impl Response for $res {}
        };
    }
}

pub trait Request {
    type Response: Response;
}

pub trait Response: Serialize {}

#[derive(Deserialize)]
pub struct BaseRequest {
    #[serde(rename = "seq")]
    sequence: u32,
    #[serde(flatten)]
    pub variant: RequestVariant,
}

impl BaseRequest {
    pub fn reply(&self, response: ResponseVariant) -> BaseResponse {
        BaseResponse {
            sequence: self.sequence,
            data: response,
        }
    }
}

#[derive(Serialize)]
pub struct BaseResponse {
    #[serde(rename = "seq")]
    sequence: u32,
    #[serde(flatten)]
    data: ResponseVariant,
}

#[derive(Deserialize)]
#[serde(tag = "request")]
pub enum RequestVariant {
    Ping,
    CreateRoom(CreateRoomRequest),
    JoinRoom {
        join_code: String,
    },
    EditRoomConfig {
        config: RoomConfiguration,
    },
    CreateTeam,
    StartGame,
    ClaimCell {
        uid: String,
        time: u64,
        medal: Medal,
    },
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum ResponseVariant {
    Pong,
    Ok,
    Error {
        error: String,
    },
    CreateRoom(CreateRoomResponse),
    JoinRoom {
        name: String,
        config: RoomConfiguration,
        status: RoomStatus,
    },
}

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

impl_request!(CreateRoomRequest, CreateRoomResponse);
