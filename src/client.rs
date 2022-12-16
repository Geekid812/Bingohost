use std::sync::{Arc, Mutex};

use crate::config::TEAMS;
use crate::gameroom::{GameRoom, RoomConfiguration};
use crate::protocol::Protocol;
use crate::requests::{BaseRequest, BaseResponse, CreateRoomResponse, RequestVariant, Response};
use crate::server::GameServer;
use crate::util::auth::PlayerIdentity;

pub struct GameClient {
    server: Arc<Mutex<GameServer>>,
    protocol: Protocol,
    identity: PlayerIdentity,
}

impl GameClient {
    pub fn new(
        server: Arc<Mutex<GameServer>>,
        protocol: Protocol,
        identity: PlayerIdentity,
    ) -> Self {
        Self {
            server,
            protocol,
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
                let mut lock = self.server.lock().expect("lock poisoned");
                let (join_code, teams) = lock.create_new_room(req.config.clone());
                CreateRoomResponse {
                    join_code,
                    teams,
                    max_teams: TEAMS.len(),
                }
            }
        }
    }
}
