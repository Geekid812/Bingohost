use std::sync::Arc;

use crate::{rest::auth::PlayerIdentity, roomlist::SharedRoom, socket::SocketWriter};

pub struct GameContext {
    room: SharedRoom,
    identity: PlayerIdentity,
    inbox: Arc<SocketWriter>,
}

impl GameContext {
    pub fn is_alive(&self) -> bool {
        self.room.strong_count() > 0
    }
}
