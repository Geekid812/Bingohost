use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(tag = "event")]
pub enum ClientEventVariant {
    ChangeTeam(ChangeTeamEvent),
}

#[derive(Serialize)]
#[serde(tag = "event")]
pub enum ServerEventVariant {
    RoomUpdate(u8), // TODO
}

#[derive(Deserialize)]
pub struct ChangeTeamEvent {
    pub team_id: usize,
}
