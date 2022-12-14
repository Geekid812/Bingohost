use rand::{distributions::Uniform, prelude::Distribution};
use serde_repr::Deserialize_repr;

use crate::{
    channel::Channel,
    config::{JOINCODE_CHARS, JOINCODE_LENGTH},
    util::auth::PlayerIdentity,
};

pub struct GameRoom {
    config: RoomConfiguration,
    join_code: String,
    channel: Channel,
    members: Vec<PlayerData>,
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
        }
    }

    pub fn join_code(&self) -> &str {
        return &self.join_code;
    }
}

struct PlayerData {
    identity: PlayerIdentity,
}

pub struct RoomConfiguration {}

#[derive(Deserialize_repr)]
#[repr(i32)]
pub enum MapMode {
    TOTD,
    RandomTMX,
    Mappack,
}

#[derive(Deserialize_repr)]
#[repr(i32)]
pub enum Medal {
    Author,
    Gold,
    Silver,
    Bronze,
    None,
}
