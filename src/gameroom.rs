use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use thiserror::Error;
use tracing::warn;

use crate::{
    channel::Channel,
    client::GameClient,
    config::TEAMS,
    context::{ClientContext, GameContext},
    gamedata::{ActiveGameData, BingoLine, MapCell},
    gamemap::GameMap,
    gameteam::{GameTeam, TeamIdentifier},
    rest::auth::PlayerIdentity,
    util::color::RgbColor,
};

pub struct GameRoom {
    config: RoomConfiguration,
    join_code: String,
    members: Vec<PlayerData>,
    teams: Vec<GameTeam>,
    maps: Vec<GameMap>,
    active: Option<ActiveGameData>,
    channel: Channel,
}

impl GameRoom {
    pub fn create(config: RoomConfiguration, join_code: String) -> Self {
        Self {
            config: config,
            join_code,
            members: Vec::new(),
            teams: Vec::new(),
            maps: Vec::new(),
            active: None,
            channel: Channel::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.config.name
    }

    pub fn join_code(&self) -> &str {
        &self.join_code
    }

    pub fn config(&self) -> &RoomConfiguration {
        &self.config
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

    pub fn channel(&mut self) -> &mut Channel {
        &mut self.channel
    }

    pub fn add_maps(&mut self, maps: Vec<GameMap>) {
        self.maps.extend(maps);
    }

    pub fn set_maps(&mut self, maps: Vec<GameMap>) {
        self.maps = maps;
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
        self.members.iter().map(NetworkPlayer::from).collect()
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

    pub fn get_player(&self, identity: PlayerIdentity) -> Option<&PlayerData> {
        self.members.iter().find(|p| p.identity == identity)
    }

    pub fn get_team(&self, player: TeamIdentifier) -> Option<&GameTeam> {
        self.teams.get(player)
    }

    pub fn create_team(&mut self) -> Option<&GameTeam> {
        let team_count = self.teams.len();
        if team_count >= TEAMS.len() {
            warn!("attempted to create more than {} teams", TEAMS.len());
            return None;
        }

        let mut rng = rand::thread_rng();
        let mut idx = rng.gen_range(0..TEAMS.len());
        while self.team_exsits_with_name(TEAMS[idx].0) {
            idx = rng.gen_range(0..TEAMS.len());
        }

        let color = RgbColor::from_hex(TEAMS[idx].1).ok()?;
        self.teams
            .push(GameTeam::new(team_count, TEAMS[idx].0.to_owned(), color));
        self.teams.last()
    }

    fn team_exsits(&self, id: usize) -> bool {
        self.teams.iter().any(|t| t.id == id)
    }

    fn team_exsits_with_name(&self, name: &str) -> bool {
        self.teams.iter().any(|t| t.name == name)
    }

    fn add_player(&mut self, identity: &PlayerIdentity, operator: bool) {
        let team = if !self.config.randomize {
            Some(0) // TODO: sort players in teams upon join
        } else {
            None
        };
        self.members.push(PlayerData {
            identity: identity.clone(),
            team,
            operator,
            disconnected: false, // TODO: is this accurate?
        });
    }

    pub fn player_join(
        &mut self,
        identity: &PlayerIdentity,
        operator: bool,
    ) -> Result<(), JoinRoomError> {
        if self.has_started() {
            return Err(JoinRoomError::HasStarted);
        }
        if self.config.size != 0 && self.members.len() as u32 >= self.config.size {
            return Err(JoinRoomError::PlayerLimitReached);
        }
        Ok(self.add_player(identity, operator))
    }

    // Returns: whether the room should be closed
    pub fn player_remove(&mut self, identity: &PlayerIdentity) -> bool {
        for i in 0..self.members.len() {
            if &self.members[i].identity == identity {
                self.members.remove(i);
                break;
            }
        }
        return self.members.iter().any(|m| m.operator);
    }

    pub fn change_team(&mut self, identity: &PlayerIdentity, team: TeamIdentifier) {
        if !self.team_exsits(team) {
            return;
        }
        if let Some(data) = self
            .members
            .iter_mut()
            .filter(|m| &m.identity == identity)
            .next()
        {
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
    // Room Settings
    pub name: String,
    pub visibility: RoomVisibility,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    pub size: u32,
    pub randomize: bool,
    pub chat_enabled: bool,
    // Game Settings
    pub grid_size: u8,
    pub selection: MapMode,
    pub medal: Medal,
    pub time_limit: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mappack_id: Option<u32>,
}

#[derive(Clone, Copy, Debug, Serialize_repr, Deserialize_repr, PartialEq, Eq)]
#[repr(u8)]
pub enum RoomVisibility {
    Public,
    Private,
}

#[derive(Clone, Copy, Debug, Serialize_repr, Deserialize_repr, PartialEq, Eq)]
#[repr(u8)]
pub enum MapMode {
    TOTD,
    RandomTMX,
    Mappack,
}

#[derive(Clone, Copy, Debug, Serialize_repr, Deserialize_repr, PartialEq, Eq)]
#[repr(u8)]
pub enum Medal {
    Author,
    Gold,
    Silver,
    Bronze,
    None,
}
