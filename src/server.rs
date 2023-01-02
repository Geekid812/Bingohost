use std::sync::{Arc, Mutex};

use tokio::join;

use crate::{
    client::GameClient,
    config,
    gamemap::{MapStock, Receiver, Sender},
    gameroom::{GameRoom, PlayerData, RoomConfiguration},
    gameteam::{NetworkTeam, TeamId},
};

pub struct GameServer {
    // rooms never have any shared references, they are always owned by GameServer.
    // therefore, it's okay to declare them 'static
    rooms: Mutex<Vec<GameRoom<'static>>>,
    maps: MapStock,
}

impl GameServer {
    pub fn new(maps_tx: Sender) -> Self {
        let map_stock = MapStock::new(config::MAP_QUEUE_SIZE, maps_tx);
        Self {
            rooms: Mutex::new(Vec::new()),
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
    ) -> (String, Vec<NetworkTeam>) {
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
        self.rooms.lock().expect("lock poisoned").push(room);
        (code, teams)
    }
}
