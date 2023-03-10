use crate::{gameroom::PlayerIdentifier, roomlist::SharedRoom};

pub struct GameContext {
    room: SharedRoom,
    player_id: PlayerIdentifier,
}

impl GameContext {
    pub fn is_alive(&self) -> bool {
        self.room.strong_count() > 0
    }
}
