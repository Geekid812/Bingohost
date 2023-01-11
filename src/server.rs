use std::sync::{Arc, Mutex};

use generational_arena::Arena;
use rand::{distributions::Uniform, prelude::Distribution};
use tokio::join;

use crate::{
    channel::ChannelCollection,
    client::{ClientId, GameClient},
    config::{self, JOINCODE_CHARS, JOINCODE_LENGTH},
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
    ) -> (PlayerRef, String, String, Vec<GameTeam>) {
        // Generate room join code
        let mut rng = rand::thread_rng();
        let uniform = Uniform::from(0..JOINCODE_CHARS.len());
        let mut join_code: String = (0..JOINCODE_LENGTH)
            .map(|_| JOINCODE_CHARS[uniform.sample(&mut rng)])
            .collect();

        while self.find_room(&join_code).is_some() {
            join_code = (0..JOINCODE_LENGTH)
                .map(|_| JOINCODE_CHARS[uniform.sample(&mut rng)])
                .collect();
        }

        // Create room data structure
        let host_name = &host.identity().display_name;
        let mut room = GameRoom::create(
            format!(
                "{}{} Bingo game",
                host_name,
                if host_name.ends_with('s') { "'" } else { "'s" }
            ),
            join_code,
            config,
            self.channels.create_one(),
        );
        let room_name = room.name().to_owned();

        // Add the two starting teams and the host
        let team1 = room.create_team(self.channels.create_one()).clone();
        let team2 = room.create_team(self.channels.create_one()).clone();
        let player_id = room
            .player_join(&host, true)
            .expect("adding host to a new room");
        self.channels.subscribe(room.channel(), host);

        let code = room.join_code().to_owned();
        let room_id = self.rooms.lock().expect("lock poisoned").insert(room);
        ((room_id, player_id), room_name, code, vec![team1, team2])
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
    ) -> Result<(PlayerRef, String, RoomConfiguration, RoomStatus), JoinRoomError> {
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
        self.channels.subscribe(channel, client);
        Ok((
            (room_id, player_id),
            room.name().to_owned(),
            room.config().clone(),
            room.status(),
        ))
    }

    pub fn disconnect(&self, id: ClientId, player: PlayerRef) {
        self.client_removed(id, player, false);
    }

    pub fn leave(&self, id: ClientId, player: PlayerRef) {
        self.client_removed(id, player, true);
    }

    fn client_removed(&self, id: ClientId, (room_id, player): PlayerRef, _explicit: bool) {
        let mut lock = self.rooms.lock().expect("lock poisoned");
        if let Some(room) = lock.get_mut(room_id) {
            self.channels.unsubscribe(room.channel(), id);
            if let Some(team_id) = room.get_player(player).and_then(|p| p.team) {
                let team_channel = room.get_team(team_id).expect("team to exist").channel_id;
                self.channels.unsubscribe(team_channel, id);
            }

            let should_close = room.player_remove(player);
            if should_close {
                let room = lock.remove(room_id).expect("room exists");
                self.channels.remove(room.channel());
                for team in &room.teams() {
                    self.channels.remove(team.channel_id);
                }
            } else {
                self.channels.broadcast(
                    room.channel(),
                    ServerEventVariant::RoomUpdate(room.status()),
                );
            }
        }
    }
}
