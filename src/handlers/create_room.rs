use serde::{Deserialize, Serialize};

use crate::{
    config::TEAMS,
    context::{ClientContext, GameContext},
    gamecommon::setup_room,
    gameroom::RoomConfiguration,
    gameteam::GameTeam,
    roomlist,
};

use super::{Request, Response};

#[derive(Deserialize, Debug)]
pub struct CreateRoom(RoomConfiguration);

#[derive(Serialize, Debug)]
pub struct CreateRoomResponse {
    pub name: String,
    pub join_code: String,
    pub max_teams: usize,
    pub teams: Vec<GameTeam>,
}

#[typetag::deserialize]
impl Request for CreateRoom {
    fn handle(&self, ctx: &mut ClientContext) -> Box<dyn Response> {
        if let Some(room) = ctx.game.as_mut().and_then(|game_ctx| game_ctx.room()) {
            room.lock().player_remove(&ctx.identity);
            // TODO: on player removed?
        }
        let room_arc = roomlist::create_room(self.0.clone());
        let mut room = room_arc.lock();

        setup_room(&mut room);
        ctx.game = Some(GameContext::new(&ctx, &room_arc));
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
