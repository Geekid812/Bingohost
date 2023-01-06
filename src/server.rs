use std::sync::{Arc, Mutex};

use generational_arena::Arena;
use tokio::join;

use crate::{
    channel::ChannelCollection,
    client::GameClient,
    config,
    events::ServerEventVariant,
    gamemap::{MapStock, Receiver, Sender},
    gameroom::{GameRoom, PlayerRef, RoomConfiguration},
    gameteam::{GameTeam, TeamIdentifier},
};

pub struct GameServer {
    rooms: Mutex<Arena<GameRoom>>,
    channels: ChannelCollection,
    maps: MapStock,
}

impl GameServer {
    pub fn new(maps_tx: Sender) -> Self {
        let map_stock = MapStock::new(config::MAP_QUEUE_SIZE, maps_tx);
        Self {
            rooms: Mutex::new(Arena::new()),
            channels: ChannelCollection::new(),
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
    ) -> (PlayerRef, String, Vec<GameTeam>) {
        let mut room = GameRoom::create(config, self.channels.create_one());
        let team1 = room.create_team(self.channels.create_one()).clone();
        let team2 = room.create_team(self.channels.create_one()).clone();
        let player_id = room.player_join(&host);
        self.channels.subscribe(room.channel(), host.get_protocol());

        let code = room.join_code().to_owned();
        let room_id = self.rooms.lock().expect("lock poisoned").insert(room);
        ((room_id, player_id), code, vec![team1, team2])
    }

    pub fn change_team(&self, (room, player): PlayerRef, team: TeamIdentifier) {
        if let Some(room) = self.rooms.lock().expect("lock poisoned").get_mut(room) {
            room.change_team(player, team);
            self.channels.broadcast(
                room.channel(),
                ServerEventVariant::RoomUpdate {
                    members: room.players(),
                    teams: room.teams(),
                },
            );
        }
    }
}
