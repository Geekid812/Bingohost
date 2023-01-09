use std::sync::{Arc, Mutex};

use generational_arena::Arena;
use tokio::join;

use crate::{
    channel::ChannelCollection,
    client::GameClient,
    config,
    events::ServerEventVariant,
    gamemap::{MapStock, Receiver, Sender},
    gameroom::{GameRoom, JoinRoomError, PlayerRef, RoomConfiguration, RoomIdentifier, RoomStatus},
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
        let player_id = room
            .player_join(&host, true)
            .expect("adding host to a new room");
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
                ServerEventVariant::RoomUpdate(room.status()),
            );
        }
    }

    fn find_room(&self, join_code: &str) -> Option<RoomIdentifier> {
        let lock = self.rooms.lock().expect("lock poisoned");
        let result = lock
            .iter()
            .filter(|(_, room)| room.join_code() == join_code)
            .next()?;
        Some(result.0)
    }

    pub fn join_room(
        &self,
        client: &GameClient,
        join_code: &str,
    ) -> Result<(PlayerRef, RoomConfiguration, RoomStatus), JoinRoomError> {
        let room_id = self
            .find_room(join_code)
            .ok_or(JoinRoomError::DoesNotExist(join_code.to_owned()))?;
        let mut lock = self.rooms.lock().expect("lock poisioned");
        let room = lock
            .get_mut(room_id)
            .ok_or(JoinRoomError::DoesNotExist(join_code.to_owned()))?;
        let player_id = room.player_join(client, false)?;
        let channel = room.channel();
        self.channels
            .broadcast(channel, ServerEventVariant::RoomUpdate(room.status()));
        self.channels.subscribe(channel, client.get_protocol());
        Ok(((room_id, player_id), room.config().clone(), room.status()))
    }
}
