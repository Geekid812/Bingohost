use std::io::ErrorKind;
use std::sync::Arc;

use tracing::info;

use crate::config::TEAMS;
use crate::events::{ClientEventVariant, ServerEventVariant};
use crate::gameroom::PlayerRef;
use crate::protocol::Protocol;
use crate::requests::{BaseRequest, CreateRoomResponse, RequestVariant, ResponseVariant};
use crate::rest::auth::PlayerIdentity;
use crate::GlobalServer;

pub type ClientId = u32;

pub struct GameClient {
    id: ClientId,
    server: GlobalServer,
    protocol: Arc<Protocol>,
    identity: PlayerIdentity,
    player_id: Option<PlayerRef>,
}

impl GameClient {
    pub fn new(
        id: ClientId,
        server: GlobalServer,
        protocol: Protocol,
        identity: PlayerIdentity,
    ) -> Self {
        Self {
            id,
            server,
            protocol: Arc::new(protocol),
            identity,
            player_id: None,
        }
    }

    pub async fn run(mut self) {
        loop {
            let data = self.protocol.recv().await;
            match data {
                Ok(text) => {
                    info!("Received: {}", text);
                    // Match a request
                    if let Ok(request) = serde_json::from_str::<BaseRequest>(&text) {
                        let res = self.handle_request(&request.variant).await;
                        let response = request.reply(res);
                        let res_text =
                            serde_json::to_string(&response).expect("response serialization");
                        info!("Response: {}", &res_text);
                        let sent = self.protocol.send(&res_text).await;
                        if let Err(e) = sent {
                            self.protocol.error(&e.to_string()).await;
                            return;
                        }
                    } else {
                        // Match an event
                        match serde_json::from_str::<ClientEventVariant>(&text) {
                            Ok(event) => self.handle_event(&event).await,
                            Err(e) => self.protocol.error(&e.to_string()).await,
                        };
                    }
                }
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                    // Handle disconnection
                    self.handle_disconnect();
                    break;
                }
                Err(e) => self.protocol.error(&e.to_string()).await,
            }
        }
    }

    async fn handle_request(&mut self, variant: &RequestVariant) -> ResponseVariant {
        match variant {
            RequestVariant::Ping => ResponseVariant::Pong,
            RequestVariant::CreateRoom(req) => {
                let (player, name, join_code, teams) =
                    self.server.create_new_room(req.config.clone(), &self);
                self.player_id = Some(player);
                ResponseVariant::CreateRoom(CreateRoomResponse {
                    name,
                    join_code,
                    teams,
                    max_teams: TEAMS.len(),
                })
            }
            RequestVariant::JoinRoom { join_code } => {
                match self.server.join_room(&self, join_code) {
                    Ok((player, name, config, status)) => {
                        self.player_id = Some(player);
                        ResponseVariant::JoinRoom {
                            name,
                            config: config,
                            status: status,
                        }
                    }
                    Err(e) => ResponseVariant::Error {
                        error: e.to_string(),
                    },
                }
            }
            RequestVariant::EditRoomConfig { config } => {
                if let Some((room, _)) = self.player_id {
                    self.server.edit_room_config(room, config.clone());
                    return ResponseVariant::Ok;
                }
                ResponseVariant::Error {
                    error: "You are not in a room.".to_owned(),
                }
            }
            RequestVariant::CreateTeam => {
                if let Some((room, _)) = self.player_id {
                    self.server.add_team(room);
                    return ResponseVariant::Ok;
                }
                ResponseVariant::Error {
                    error: "You are not in a room.".to_owned(),
                }
            }
            RequestVariant::StartGame => {
                self.server
                    .start_game(self.player_id.expect("client is in a room"));
                ResponseVariant::Ok
            }
            RequestVariant::ClaimCell { uid, time, medal } => {
                if let Some(player) = self.player_id {
                    self.server.claim_cell(player, uid.clone(), *time, *medal);
                    return ResponseVariant::Ok;
                }
                ResponseVariant::Error {
                    error: "You are not in a room.".to_owned(),
                }
            }
        }
    }

    async fn handle_event(&mut self, variant: &ClientEventVariant) {
        match variant {
            ClientEventVariant::ChangeTeam { team_id } => {
                if let Some(player) = self.player_id {
                    self.server.change_team(player.clone(), *team_id);
                }
            }
            ClientEventVariant::LeaveRoom => {
                if let Some(player) = self.player_id {
                    self.server.leave(self.id, player);
                }
            }
        }
    }

    fn handle_disconnect(&mut self) {
        info!("Client disconnected: {}", self.identity.display_name);
        self.protocol.close();
        if let Some(player) = self.player_id {
            self.server.disconnect(self.id, player);
        }
    }

    async fn fire_event(&mut self, event: ServerEventVariant) {
        let sent = self
            .protocol
            .send(&serde_json::to_string(&event).expect("event serialization"))
            .await;
        if let Err(e) = sent {
            self.protocol.error(&e.to_string()).await;
            return;
        }
    }

    pub fn identity(&self) -> &PlayerIdentity {
        &self.identity
    }

    pub fn get_protocol(&self) -> Arc<Protocol> {
        self.protocol.clone()
    }

    pub fn get_id(&self) -> ClientId {
        self.id
    }
}
