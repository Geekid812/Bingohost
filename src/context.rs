use std::sync::{Arc, Weak};

use crate::{rest::auth::PlayerIdentity, roomlist::SharedRoom, socket::SocketWriter};

pub struct ClientContext {
    pub game: Option<GameContext>,
    pub identity: PlayerIdentity,
}

impl ClientContext {
    pub fn new(identity: PlayerIdentity, game: Option<GameContext>) -> Self {
        Self { game, identity }
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
