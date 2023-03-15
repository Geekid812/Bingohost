use std::sync::{Arc, Weak};

use crate::{
    rest::auth::PlayerIdentity,
    roomlist::{OwnedRoom, SharedRoom},
    socket::{SocketAction, SocketWriter},
};

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

    pub fn game_room(&mut self) -> Option<OwnedRoom> {
        self.game.as_mut().and_then(|gamectx| gamectx.room())
    }

    pub fn trace<M: Into<String>>(&self, message: M) {
        if let Ok(text) = serde_json::to_string(&message.into()) {
            drop(self.writer.send(SocketAction::Message(format!(
                "{{\"event\":\"Trace\",\"value\":{}}}",
                text
            ))));
        }
    }
}

pub struct GameContext {
    room: SharedRoom,
    pub writer: Arc<Weak<SocketWriter>>,
}

impl GameContext {
    pub fn new(ctx: &ClientContext, room: &OwnedRoom) -> Self {
        Self {
            room: Arc::downgrade(room),
            writer: Arc::new(Arc::downgrade(&ctx.writer)),
        }
    }
    pub fn is_alive(&self) -> bool {
        self.room.strong_count() > 0
    }

    pub fn room<'a>(&self) -> Option<OwnedRoom> {
        self.room.upgrade()
    }
}
