use crate::gameroom::{GameRoom, RoomConfiguration};

pub struct GameServer {
    // rooms never have any shared references, they are always owned by GameServer.
    // therefore, it's okay to declare them 'static
    rooms: Vec<GameRoom<'static>>,
}

impl GameServer {
    pub const fn new() -> Self {
        Self { rooms: Vec::new() }
    }
    pub fn create_new_room(&mut self, config: RoomConfiguration) -> String {
        let mut room = GameRoom::create(config);
        room.create_team();
        room.create_team();
        let code = room.join_code().to_owned();
        self.rooms.push(room);
        code
    }
}
