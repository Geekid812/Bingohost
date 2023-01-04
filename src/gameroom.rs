use rand::{distributions::Uniform, prelude::Distribution, Rng};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{
    channel::Channel,
    client::GameClient,
    config::{JOINCODE_CHARS, JOINCODE_LENGTH, TEAMS},
    gameteam::{GameTeam, TeamId},
    rest::auth::PlayerIdentity,
};

pub struct GameRoom {
    config: RoomConfiguration,
    join_code: String,
    channel: Channel,
    pub members: Vec<PlayerData>,
    pub teams: Vec<GameTeam>,
}

impl GameRoom {
    pub fn create(config: RoomConfiguration) -> Self {
        let mut rng = rand::thread_rng();
        let uniform = Uniform::from(0..JOINCODE_CHARS.len());
        let join_code: String = (0..JOINCODE_LENGTH)
            .map(|_| JOINCODE_CHARS[uniform.sample(&mut rng)])
            .collect();

        Self {
            config,
            join_code,
            channel: Channel::new(),
            members: Vec::new(),
            teams: Vec::new(),
        }
    }

    pub fn join_code(&self) -> &str {
        return &self.join_code;
    }

    pub fn config(&self) -> &RoomConfiguration {
        return &self.config;
    }

    pub fn create_team(&mut self) {
        if self.teams.len() >= TEAMS.len() {
            panic!("attempted to create more than {} teams", TEAMS.len());
        }

        let mut rng = rand::thread_rng();
        let mut id = rng.gen_range(0..TEAMS.len());
        while self.team_exsits(id) {
            id = rng.gen_range(0..TEAMS.len());
        }

        self.teams.push(GameTeam::new(id));
    }

    fn team_exsits(&self, id: usize) -> bool {
        self.teams.iter().any(|t| t.id.0 == id)
    }

    pub fn player_join(&mut self, client: &GameClient) {
        self.members.push(PlayerData {
            identity: client.identity().clone(),
            team: None,
        });
        self.channel.subscribe(client.get_protocol());
    }
}

pub struct PlayerData {
    pub identity: PlayerIdentity,
    pub team: Option<TeamId>,
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
#[repr(i32)]
pub enum MapMode {
    TOTD,
    RandomTMX,
    Mappack,
}

#[derive(Clone, Copy, Debug, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum Medal {
    Author,
    Gold,
    Silver,
    Bronze,
    None,
}
