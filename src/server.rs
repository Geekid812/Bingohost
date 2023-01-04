use std::sync::{Arc, Mutex};

use generational_arena::Arena;
use tokio::join;

use crate::{
    client::GameClient,
    config,
    gamemap::{MapStock, Receiver, Sender},
    gameroom::{GameRoom, RoomConfiguration},
    gameteam::NetworkTeam,
};

pub type InternalRoomIdentifier = generational_arena::Index;

pub struct GameServer {
    rooms: Mutex<Arena<GameRoom>>,
    maps: MapStock,
}

impl GameServer {
    pub fn new(maps_tx: Sender) -> Self {
        let map_stock = MapStock::new(config::MAP_QUEUE_SIZE, maps_tx);
        Self {
            rooms: Mutex::new(Arena::new()),
            maps: map_stock,
        }
    }

    pub async fn spawn(self: Arc<Self>, maps_rx: Receiver) {
        join! { self.maps.fetch_loop(maps_rx) };
    }

    pub fn create_new_room(
        &self,
        config: RoomConfiguration,
        host: &GameClient,
    ) -> (InternalRoomIdentifier, String, Vec<NetworkTeam>) {
        let mut room = GameRoom::create(config);
        room.create_team();
        room.create_team();
        room.player_join(&host);

        let teams = room
            .teams
            .iter()
            .map(|team| NetworkTeam::from(team))
            .collect();
        let code = room.join_code().to_owned();
        let ident = self.rooms.lock().expect("lock poisoned").insert(room);
        (ident, code, teams)
    }

    pub fn change_team(&self, room: InternalRoomIdentifier, player: &GameClient, team: usize) {
        if let Some(room) = self.rooms.lock().expect("lock poisoned").get(room) {
            // TODO
        }
    }
}
