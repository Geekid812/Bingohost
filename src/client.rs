use tracing::debug;

use crate::context::GameContext;
use crate::requests::BaseRequest;
use crate::rest::auth::PlayerIdentity;
use crate::socket::{SocketAction, SocketReader, SocketWriter};

pub struct GameClient {
    identity: PlayerIdentity,
    ctx: Option<GameContext>,
    reader: SocketReader,
    writer: SocketWriter,
}

pub async fn run_loop(
    identity: PlayerIdentity,
    mut ctx: Option<GameContext>,
    mut reader: SocketReader,
    writer: SocketWriter,
) -> LoopExit {
    loop {
        let data = reader.recv().await;
        if data.is_none() {
            // Client disconnected
            return if let Some(ctx) = ctx {
                LoopExit::Linger(identity, ctx)
            } else {
                LoopExit::Close
            };
        }
        let msg = data.unwrap();
        debug!("received: {}", msg);

        // Match a request
        if let Ok(incoming) = serde_json::from_str::<BaseRequest>(&msg) {
            let response = incoming.request.handle(&identity, &mut ctx);
            let outgoing = incoming.build_reply(response);
            let res_text = serde_json::to_string(&outgoing).expect("response serialization failed");
            debug!("response: {}", &res_text);
            let sent = writer.send(SocketAction::Message(res_text));
        } else {
            // Match an event
            // match serde_json::from_str::<ClientEvent>(&msg) {
            //     Ok(event) => self.handle_event(&event).await,
            //     Err(e) => self.protocol.error(&e.to_string()).await,
            // };
        }
    }
}

pub enum LoopExit {
    Linger(PlayerIdentity, GameContext),
    Close,
}

impl GameClient {
    // async fn handle_request(&mut self, variant: &Request) -> Response {
    //     match variant {
    //         Request::Ping => Response::Pong,
    //         Request::CreateRoom(req) => {
    //             let (player, name, join_code, teams) =
    //                 self.server.create_new_room(req.config.clone(), &self);
    //             self.player_id = Some(player);
    //             Response::CreateRoom(CreateRoomResponse {
    //                 name,
    //                 join_code,
    //                 teams,
    //                 max_teams: TEAMS.len(),
    //             })
    //         }
    //         Request::JoinRoom { join_code } => match self.server.join_room(&self, join_code) {
    //             Ok((player, name, config, status)) => {
    //                 self.player_id = Some(player);
    //                 Response::JoinRoom {
    //                     name,
    //                     config: config,
    //                     status: status,
    //                 }
    //             }
    //             Err(e) => Response::Error {
    //                 error: e.to_string(),
    //             },
    //         },
    //         Request::EditRoomConfig { config } => {
    //             if let Some((room, _)) = self.player_id {
    //                 self.server.edit_room_config(room, config.clone());
    //                 return Response::Ok;
    //             }
    //             Response::Error {
    //                 error: "You are not in a room.".to_owned(),
    //             }
    //         }
    //         Request::CreateTeam => {
    //             if let Some((room, _)) = self.player_id {
    //                 self.server.add_team(room);
    //                 return Response::Ok;
    //             }
    //             Response::Error {
    //                 error: "You are not in a room.".to_owned(),
    //             }
    //         }
    //         Request::StartGame => {
    //             self.server
    //                 .start_game(self.player_id.expect("client is in a room"));
    //             Response::Ok
    //         }
    //         Request::ClaimCell { uid, time, medal } => {
    //             if let Some(player) = self.player_id {
    //                 self.server.claim_cell(player, uid.clone(), *time, *medal);
    //                 return Response::Ok;
    //             }
    //             Response::Error {
    //                 error: "You are not in a room.".to_owned(),
    //             }
    //         }
    //         Request::Sync => {
    //             if self.player_id.is_none() {
    //                 return Response::Error {
    //                     error: "Sync failed, the game you joined may have ended already."
    //                         .to_string(),
    //                 };
    //             }
    //             match self.server.sync_client(self.player_id.unwrap()) {
    //                 Some(sync) => Response::Sync(sync),
    //                 None => Response::Error {
    //                     error: "Sync error".to_string(),
    //                 }, // TODO: handle results
    //             }
    //         }
    //     }
    // }
}
