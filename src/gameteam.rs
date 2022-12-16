use serde::Serialize;

use crate::{config::TEAMS, gameroom::PlayerData, util::color::RgbColor};

#[derive(Clone)]
pub struct GameTeam<'a> {
    pub id: usize,
    pub name: &'static str,
    pub color: RgbColor,
    pub members: Vec<&'a PlayerData<'a>>,
}

impl<'a> GameTeam<'a> {
    pub fn new(id: usize) -> Self {
        let (name, color_string) = TEAMS[id];
        let color = RgbColor::from_hex(color_string).expect("team color parsing failed");
        Self {
            id,
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
            id: value.id,
            name: value.name,
            color: value.color,
        }
    }
}
