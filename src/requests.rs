#![allow(unused)]
use serde::{Deserialize, Serialize};

use crate::gameroom::{MapMode, Medal};

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
    pub fn reply<R: Response>(&self, response: R) -> BaseResponse<R> {
        BaseResponse {
            sequence: self.sequence,
            data: response,
        }
    }
}

#[derive(Serialize)]
pub struct BaseResponse<R: Response> {
    #[serde(rename = "seq")]
    sequence: u32,
    #[serde(flatten)]
    data: R,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum RequestVariant {
    CreateRoom(CreateRoomRequest),
}

#[derive(Deserialize)]
pub struct CreateRoomRequest {
    pub size: u32,
    pub selection: MapMode,
    pub medal: Medal,
    pub timelimit: u32,
    pub mappack_id: Option<String>,
}

#[derive(Serialize)]
pub struct CreateRoomResponse {
    pub room_code: String,
    pub max_teams: u32,
}

impl_request!(CreateRoomRequest, CreateRoomResponse);
