use std::sync::Arc;

use crate::config::TEAMS;
use crate::protocol::Protocol;
use crate::requests::{BaseRequest, CreateRoomResponse, RequestVariant, Response};
use crate::rest::auth::PlayerIdentity;
use crate::GlobalServer;

pub struct GameClient {
    server: GlobalServer,
    protocol: Arc<Protocol>,
    identity: PlayerIdentity,
}

impl GameClient {
    pub fn new(server: GlobalServer, protocol: Protocol, identity: PlayerIdentity) -> Self {
        Self {
            server,
            protocol: Arc::new(protocol),
            identity,
        }
    }

    pub async fn run(mut self) {
        loop {
            let data = self.protocol.recv().await;
            if let Ok(text) = data {
                let request: BaseRequest = match serde_json::from_str(&text) {
                    Ok(m) => m,
                    Err(e) => {
                        self.protocol.error(&e.to_string()).await;
                        continue;
                    }
                };
                let res = self.handle(&request.variant).await;
                let response = request.reply(res);
                let sent = self
                    .protocol
                    .send(&serde_json::to_string(&response).expect("response serialization"))
                    .await;
                if let Err(e) = sent {
                    self.protocol.error(&e.to_string()).await;
                    return;
                }
            } else if let Err(e) = data {
                self.protocol.error(&e.to_string()).await;
                return;
            }
        }
    }

    pub async fn handle(&mut self, msg: &RequestVariant) -> impl Response {
        match msg {
            RequestVariant::CreateRoom(req) => {
                let (join_code, teams) = self.server.create_new_room(req.config.clone(), &self);
                CreateRoomResponse {
                    join_code,
                    teams,
                    max_teams: TEAMS.len(),
                }
            }
        }
    }

    pub fn identity(&self) -> &PlayerIdentity {
        &self.identity
    }

    pub fn get_protocol(&self) -> Arc<Protocol> {
        self.protocol.clone()
    }
}
