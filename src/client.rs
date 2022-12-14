use crate::gameroom::{GameRoom, RoomConfiguration};
use crate::protocol::Protocol;
use crate::requests::{BaseRequest, BaseResponse, CreateRoomResponse, RequestVariant, Response};
use crate::util::auth::PlayerIdentity;

pub struct GameClient {
    protocol: Protocol,
    identity: PlayerIdentity,
}

impl GameClient {
    pub fn new(protocol: Protocol, identity: PlayerIdentity) -> Self {
        Self { protocol, identity }
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
                self.protocol
                    .send(&serde_json::to_string(&response).expect("response serialization"))
                    .await
                    .unwrap(); // TODO
            } else if let Err(e) = data {
                self.protocol.error(&e.to_string()).await;
                return;
            }
        }
    }

    pub async fn handle(&mut self, msg: &RequestVariant) -> impl Response {
        let room = GameRoom::create(RoomConfiguration {});
        CreateRoomResponse {
            room_code: room.join_code().to_owned(),
            max_teams: 10,
        }
    }
}
