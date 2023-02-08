use generational_arena::Arena;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use thiserror::Error;
use tracing::warn;

use crate::{
    channel::ChannelAddress,
    client::GameClient,
    config::TEAMS,
    gamedata::{ActiveGameData, BingoLine, MapCell},
    gamemap::GameMap,
    gameteam::{GameTeam, TeamIdentifier},
    rest::auth::PlayerIdentity,
};

pub type RoomIdentifier = generational_arena::Index;
pub type PlayerIdentifier = generational_arena::Index;
pub type PlayerRef = (RoomIdentifier, PlayerIdentifier);

pub struct GameRoom {
    name: String,
    config: RoomConfiguration,
    join_code: String,
    members: Arena<PlayerData>,
    teams: Vec<GameTeam>,
    channel: ChannelAddress,
    maps: Vec<GameMap>,
    active: Option<ActiveGameData>,
}

impl GameRoom {
    pub fn create(
        name: String,
        join_code: String,
        config: RoomConfiguration,
        channel: ChannelAddress,
    ) -> Self {
        Self {
            name,
            config: config,
            join_code,
            members: Arena::new(),
            teams: Vec::new(),
            channel,
            maps: Vec::new(),
            active: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn join_code(&self) -> &str {
        &self.join_code
    }

    pub fn config(&self) -> &RoomConfiguration {
        &self.config
    }

    pub fn channel(&self) -> ChannelAddress {
        self.channel.clone()
    }

    pub fn maps(&self) -> &Vec<GameMap> {
        &self.maps
    }

    pub fn game_data(&self) -> &Option<ActiveGameData> {
        &self.active
    }

    pub fn has_started(&self) -> bool {
        self.active.is_some()
    }

    pub fn add_maps(&mut self, maps: Vec<GameMap>) {
        self.maps.extend(maps);
    }

    pub fn remove_maps(&mut self, count: usize) -> Vec<GameMap> {
        if count > self.maps.len() {
            self.remove_all_maps()
        } else {
            self.maps.split_off(self.maps.len() - count)
        }
    }

    pub fn remove_all_maps(&mut self) -> Vec<GameMap> {
        self.remove_maps(self.maps.len())
    }

    pub fn players(&self) -> Vec<NetworkPlayer> {
        self.members
            .iter()
            .map(|(_, player)| NetworkPlayer::from(player))
            .collect()
    }

    pub fn teams(&self) -> Vec<GameTeam> {
        self.teams.clone()
    }

    pub fn status(&self) -> RoomStatus {
        RoomStatus {
            members: self.players(),
            teams: self.teams(),
        }
    }

    pub fn get_player(&self, player: PlayerIdentifier) -> Option<&PlayerData> {
        self.members.get(player)
    }

    pub fn get_team(&self, player: TeamIdentifier) -> Option<&GameTeam> {
        self.teams.get(player)
    }

    pub fn create_team(&mut self, channel: ChannelAddress) -> Option<&GameTeam> {
        let team_count = self.teams.len();
        if team_count >= TEAMS.len() {
            // FIXME avoiding panic here
            // panic!("attempted to create more than {} teams", TEAMS.len());
            warn!("attempted to create more than {} teams", TEAMS.len());
            return None;
        }

        let mut rng = rand::thread_rng();
        let mut idx = rng.gen_range(0..TEAMS.len());
        while self.team_exsits_with_index(idx) {
            idx = rng.gen_range(0..TEAMS.len());
        }

        self.teams.push(GameTeam::new(team_count, idx, channel));
        self.teams.last()
    }

    fn team_exsits(&self, id: usize) -> bool {
        self.teams.iter().any(|t| t.id == id)
    }

    fn team_exsits_with_index(&self, idx: usize) -> bool {
        self.teams.iter().any(|t| t.gen_index == idx)
    }

    fn add_player(&mut self, client: &GameClient, operator: bool) -> PlayerIdentifier {
        let team = if !self.config.randomize {
            Some(0) // TODO: sort players in teams upon join
        } else {
            None
        };
        self.members.insert(PlayerData {
            identity: client.identity().clone(),
            team,
            operator,
            disconnected: false,
        })
    }

    pub fn identify_player(&self, identity: &PlayerIdentity) -> Option<PlayerIdentifier> {
        self.members
            .iter()
            .filter(|(_, p)| &p.identity == identity)
            .next()
            .map(|(id, _)| id)
    }

    pub fn player_join(
        &mut self,
        client: &GameClient,
        operator: bool,
    ) -> Result<PlayerIdentifier, JoinRoomError> {
        if self.has_started() {
            return Err(JoinRoomError::HasStarted);
        }
        if self.config.size != 0 && self.members.len() as u32 >= self.config.size {
            return Err(JoinRoomError::PlayerLimitReached);
        }
        Ok(self.add_player(client, operator))
    }

    // Returns: whether the room should be closed
    pub fn player_remove(&mut self, player: PlayerIdentifier) -> bool {
        self.members.remove(player).map_or(false, |p| p.operator)
    }

    pub fn change_team(&mut self, player: PlayerIdentifier, team: TeamIdentifier) {
        if !self.team_exsits(team) {
            return;
        }
        if let Some(data) = self.members.get_mut(player) {
            data.team = Some(team);
        }
    }

    pub fn set_config(&mut self, config: RoomConfiguration) {
        self.config = config
    }

    pub fn set_started(&mut self, started: bool) {
        if started {
            self.active = Some(ActiveGameData::new(self.maps.len()));
        } else {
            self.active = None;
        }
    }

    pub fn get_cell_record(&mut self, cell_id: usize) -> Option<&mut MapCell> {
        self.active
            .as_mut()
            .and_then(|state| state.cells.get_mut(cell_id))
    }

    pub fn get_map(&self, uid: String) -> Option<(usize, &GameMap)> {
        self.maps
            .iter()
            .enumerate()
            .filter(|m| m.1.uid == uid)
            .next()
    }

    pub fn check_for_bingos(&self) -> Vec<BingoLine> {
        self.active.as_ref().map_or(Vec::new(), |a| {
            a.check_for_bingos(self.config.grid_size.into())
        })
    }
}

#[derive(Serialize)]
pub struct RoomStatus {
    pub members: Vec<NetworkPlayer>,
    pub teams: Vec<GameTeam>,
}

#[derive(Error, Debug)]
pub enum JoinRoomError {
    #[error("The room is already full.")]
    PlayerLimitReached,
    #[error("No room was found with code {0}.")]
    DoesNotExist(String),
    #[error("The game has already started.")]
    HasStarted,
}

pub struct PlayerData {
    pub identity: PlayerIdentity,
    pub team: Option<TeamIdentifier>,
    pub operator: bool,
    pub disconnected: bool,
}

#[derive(Serialize, Clone)]
pub struct NetworkPlayer {
    pub name: String,
    pub team: Option<TeamIdentifier>,
}

impl From<&PlayerData> for NetworkPlayer {
    fn from(value: &PlayerData) -> Self {
        Self {
            name: value.identity.display_name.clone(),
            team: value.team,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RoomConfiguration {
    pub size: u32,
    pub randomize: bool,
    pub chat_enabled: bool,
    pub grid_size: u8,
    pub selection: MapMode,
    pub medal: Medal,
    pub time_limit: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mappack_id: Option<u32>,
}

#[derive(Clone, Copy, Debug, Serialize_repr, Deserialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum MapMode {
    TOTD,
    RandomTMX,
    Mappack,
}

#[derive(Clone, Copy, Debug, Serialize_repr, Deserialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum Medal {
    Author,
    Gold,
    Silver,
    Bronze,
    None,
}
