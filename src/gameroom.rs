use rand::{distributions::Uniform, prelude::Distribution, Rng};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{
    channel::Channel,
    config::{JOINCODE_CHARS, JOINCODE_LENGTH, TEAMS},
    gameteam::GameTeam,
    util::auth::PlayerIdentity,
};

pub struct GameRoom<'a> {
    config: RoomConfiguration,
    join_code: String,
    channel: Channel,
    members: Vec<PlayerData<'a>>,
    teams: Vec<GameTeam<'a>>,
}

impl<'a> GameRoom<'a> {
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
        self.teams.iter().any(|t| t.id == id)
    }
}

pub struct PlayerData<'a> {
    pub identity: PlayerIdentity,
    pub team: &'a GameTeam<'a>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RoomConfiguration {
    pub size: u32,
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
