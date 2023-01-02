use serde::Serialize;

use crate::{config::TEAMS, gameroom::PlayerData, util::color::RgbColor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeamId(pub usize);

#[derive(Clone)]
pub struct GameTeam<'a> {
    pub id: TeamId,
    pub name: &'static str,
    pub color: RgbColor,
    pub members: Vec<&'a PlayerData>,
}

impl<'a> GameTeam<'a> {
    pub fn new(id: usize) -> Self {
        let (name, color_string) = TEAMS[id];
        let color = RgbColor::from_hex(color_string).expect("team color parsing failed");
        Self {
            id: TeamId(id),
            name,
            color,
            members: Vec::new(),
        }
    }
}

impl<'a> PartialEq for GameTeam<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl<'a> Eq for GameTeam<'a> {}

#[derive(Serialize)]
pub struct NetworkTeam {
    pub id: usize,
    pub name: &'static str,
    pub color: RgbColor,
}

impl<'a> From<&GameTeam<'_>> for NetworkTeam {
    fn from(value: &GameTeam<'_>) -> Self {
        Self {
            id: value.id.0,
            name: value.name,
            color: value.color,
        }
    }
}
