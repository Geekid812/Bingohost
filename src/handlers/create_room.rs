use serde::{Deserialize, Serialize};

use crate::{
    client::GameClient, config::TEAMS, gamecommon::setup_room, gameroom::RoomConfiguration,
    gameteam::GameTeam, roomlist,
};

use super::{generic::Ok, Request, Response};

#[derive(Deserialize)]
pub struct CreateRoom(RoomConfiguration);

#[derive(Serialize)]
pub struct CreateRoomResponse {
    pub name: String,
    pub join_code: String,
    pub max_teams: usize,
    pub teams: Vec<GameTeam>,
}

#[typetag::deserialize]
impl Request for CreateRoom {
    fn handle(&self, client: &mut GameClient) -> Box<dyn Response> {
        let (roomid, room) = roomlist::create_room(self.0.clone());
        setup_room(room);
        Box::new(CreateRoomResponse {
            name: room.name().to_owned(),
            join_code: room.join_code().to_owned(),
            max_teams: TEAMS.len(),
            teams: room.teams(),
        })
    }
}

#[typetag::serialize]
impl Response for CreateRoomResponse {}
