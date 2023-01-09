use serde::Serialize;

use crate::{channel::ChannelAddress, config::TEAMS, util::color::RgbColor};

pub type TeamIdentifier = usize;

#[derive(Clone, Serialize)]
pub struct GameTeam {
    pub id: TeamIdentifier,
    pub name: &'static str,
    pub color: RgbColor,
    #[serde(skip_serializing)]
    pub gen_index: usize,
    #[serde(skip_serializing)]
    pub channel_id: ChannelAddress,
}

impl GameTeam {
    pub fn new(id: usize, index: usize, channel_id: ChannelAddress) -> Self {
        let (name, color_string) = TEAMS[index];
        let color = RgbColor::from_hex(color_string).expect("team color parsing failed");
        Self {
            id: id,
            name,
            color,
            channel_id,
            gen_index: index,
        }
    }
}

impl PartialEq for GameTeam {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for GameTeam {}
