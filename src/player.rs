use crate::protocol::Protocol;
use crate::util::auth::PlayerIdentity;

pub struct PlayerControl {
    protocol: Protocol,
    identity: PlayerIdentity,
}

impl PlayerControl {
    pub fn new(protocol: Protocol, identity: PlayerIdentity) -> Self {
        Self { protocol, identity }
    }

    pub async fn run(mut self) {
        loop {
            if self.protocol.recv().await.is_ok() {
                println!("received a message in runloop");
            } else {
                return;
            }
        }
    }
}
