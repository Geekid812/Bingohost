use serde::{Deserialize, Serialize};

use crate::{gameroom::NetworkPlayer, gameteam::GameTeam};

#[derive(Deserialize)]
#[serde(tag = "event")]
pub enum ClientEventVariant {
    ChangeTeam(ChangeTeamEvent),
}

#[derive(Serialize)]
#[serde(tag = "event")]
pub enum ServerEventVariant {
    RoomUpdate {
        members: Vec<NetworkPlayer>,
        teams: Vec<GameTeam>,
    },
}

#[derive(Deserialize)]
pub struct ChangeTeamEvent {
    pub team_id: usize,
}
