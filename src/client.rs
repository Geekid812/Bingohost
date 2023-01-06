use std::sync::Arc;

use tracing::info;

use crate::config::TEAMS;
use crate::events::{ClientEventVariant, ServerEventVariant};
use crate::gameroom::PlayerRef;
use crate::protocol::Protocol;
use crate::requests::{BaseRequest, CreateRoomResponse, RequestVariant, ResponseVariant};
use crate::rest::auth::PlayerIdentity;
use crate::GlobalServer;

pub struct GameClient {
    server: GlobalServer,
    protocol: Arc<Protocol>,
    identity: PlayerIdentity,
    player_id: Option<PlayerRef>,
}

impl GameClient {
    pub fn new(server: GlobalServer, protocol: Protocol, identity: PlayerIdentity) -> Self {
        Self {
            server,
            protocol: Arc::new(protocol),
            identity,
            player_id: None,
        }
    }

    pub async fn run(mut self) {
        loop {
            let data = self.protocol.recv().await;
            if let Ok(text) = data {
                info!("Received: {}", text);
                // Match a request
                if let Ok(request) = serde_json::from_str::<BaseRequest>(&text) {
                    let res = self.handle_request(&request.variant).await;
                    let response = request.reply(res);
                    let sent = self
                        .protocol
                        .send(&serde_json::to_string(&response).expect("response serialization"))
                        .await;
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
            } else if let Err(e) = data {
                self.protocol.error(&e.to_string()).await;
                return;
            }
        }
    }

    async fn handle_request(&mut self, variant: &RequestVariant) -> ResponseVariant {
        match variant {
            RequestVariant::CreateRoom(req) => {
                let (player, join_code, teams) =
                    self.server.create_new_room(req.config.clone(), &self);
                self.player_id = Some(player);
                ResponseVariant::CreateRoom(CreateRoomResponse {
                    join_code,
                    teams,
                    max_teams: TEAMS.len(),
                })
            }
        }
    }

    async fn handle_event(&mut self, variant: &ClientEventVariant) {
        match variant {
            ClientEventVariant::ChangeTeam(event) => {
                if let Some(player) = self.player_id {
                    self.server.change_team(player.clone(), event.team_id);
                }
            }
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
}
