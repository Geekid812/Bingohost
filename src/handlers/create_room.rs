use serde::{Deserialize, Serialize};

use crate::{
    client::GameClient,
    config::TEAMS,
    context::{ClientContext, GameContext},
    gamecommon::setup_room,
    gameroom::RoomConfiguration,
    gameteam::GameTeam,
    roomlist,
};

use super::{Request, Response};

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
    fn handle(&self, ctx: &mut ClientContext) -> Box<dyn Response> {
        let (lock, room) = roomlist::create_room(self.0.clone());

        setup_room(&mut room);
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
