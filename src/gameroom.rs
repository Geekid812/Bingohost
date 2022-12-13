use rand::{distributions::Uniform, prelude::Distribution};

use crate::{
    channel::Channel,
    config::{JOINCODE_CHARS, JOINCODE_LENGTH},
    util::auth::PlayerIdentity,
};

pub struct Room {
    config: RoomConfiguration,
    join_code: String,
    channel: Channel,
    members: Vec<PlayerData>,
}

impl Room {
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
}

struct PlayerData {
    identity: PlayerIdentity,
}

pub struct RoomConfiguration {}
