use generational_arena::Arena;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use thiserror::Error;

use crate::{
    channel::ChannelAddress,
    client::GameClient,
    config::TEAMS,
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

    pub fn create_team(&mut self, channel: ChannelAddress) -> &GameTeam {
        let team_count = self.teams.len();
        if team_count >= TEAMS.len() {
            panic!("attempted to create more than {} teams", TEAMS.len());
        }

        let mut rng = rand::thread_rng();
        let mut idx = rng.gen_range(0..TEAMS.len());
        while self.team_exsits_with_index(idx) {
            idx = rng.gen_range(0..TEAMS.len());
        }

        self.teams.push(GameTeam::new(team_count, idx, channel));
        self.teams.last().unwrap()
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

    pub fn player_join(
        &mut self,
        client: &GameClient,
        operator: bool,
    ) -> Result<PlayerIdentifier, JoinRoomError> {
        // TODO: check that it has started
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

#[derive(Serialize)]
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
    pub mappack_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum MapMode {
    TOTD,
    RandomTMX,
    Mappack,
}

#[derive(Clone, Copy, Debug, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum Medal {
    Author,
    Gold,
    Silver,
    Bronze,
    None,
}
