use serde::Serialize;

use crate::{config::TEAMS, util::color::RgbColor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeamId(pub usize);

#[derive(Clone)]
pub struct GameTeam {
    pub id: TeamId,
    pub name: &'static str,
    pub color: RgbColor,
}

impl GameTeam {
    pub fn new(id: usize) -> Self {
        let (name, color_string) = TEAMS[id];
        let color = RgbColor::from_hex(color_string).expect("team color parsing failed");
        Self {
            id: TeamId(id),
            name,
            color,
        }
    }
}

impl PartialEq for GameTeam {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for GameTeam {}

#[derive(Serialize)]
pub struct NetworkTeam {
    pub id: usize,
    pub name: &'static str,
    pub color: RgbColor,
}

impl From<&GameTeam> for NetworkTeam {
    fn from(value: &GameTeam) -> Self {
        Self {
            id: value.id.0,
            name: value.name,
            color: value.color,
        }
    }
}
