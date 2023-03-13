use std::sync::{Arc, Weak};

use crate::{rest::auth::PlayerIdentity, roomlist::SharedRoom, socket::SocketWriter};

pub struct ClientContext {
    pub game: Option<GameContext>,
    pub identity: PlayerIdentity,
    pub writer: Arc<SocketWriter>,
}

impl ClientContext {
    pub fn new(
        identity: PlayerIdentity,
        game: Option<GameContext>,
        writer: Arc<SocketWriter>,
    ) -> Self {
        Self {
            game,
            identity,
            writer,
        }
    }
}

pub struct GameContext {
    room: SharedRoom,
    writer: Arc<Weak<SocketWriter>>,
}

impl GameContext {
    pub fn is_alive(&self) -> bool {
        self.room.strong_count() > 0
    }
}
