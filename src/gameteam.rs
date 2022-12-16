use crate::{config::TEAMS, gameroom::PlayerData, util::color::RgbColor};

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
