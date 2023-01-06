use generational_arena::Arena;
use rand::{distributions::Uniform, prelude::Distribution, Rng};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{
    channel::ChannelAddress,
    client::GameClient,
    config::{JOINCODE_CHARS, JOINCODE_LENGTH, TEAMS},
    gameteam::{GameTeam, TeamIdentifier},
    rest::auth::PlayerIdentity,
};

pub type RoomIdentifier = generational_arena::Index;
pub type PlayerIdentifier = generational_arena::Index;
pub type PlayerRef = (RoomIdentifier, PlayerIdentifier);

pub struct GameRoom {
    config: RoomConfiguration,
    join_code: String,
    members: Arena<PlayerData>,
    teams: Vec<GameTeam>,
    channel: ChannelAddress,
}

impl GameRoom {
    pub fn create(config: RoomConfiguration, channel: ChannelAddress) -> Self {
        let mut rng = rand::thread_rng();
        let uniform = Uniform::from(0..JOINCODE_CHARS.len());
        let join_code: String = (0..JOINCODE_LENGTH)
            .map(|_| JOINCODE_CHARS[uniform.sample(&mut rng)])
            .collect();

        Self {
            config: config,
            join_code,
            members: Arena::new(),
            teams: Vec::new(),
            channel,
        }
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

    pub fn player_join(&mut self, client: &GameClient) -> PlayerIdentifier {
        let team = if !self.config.randomize {
            Some(0) // TODO: sort players in teams upon join
        } else {
            None
        };
        self.members.insert(PlayerData {
            identity: client.identity().clone(),
            team,
        })
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

pub struct PlayerData {
    pub identity: PlayerIdentity,
    pub team: Option<TeamIdentifier>,
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
