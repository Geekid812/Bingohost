use crate::util::color::RgbColor;

#[derive(PartialEq, Eq)]
pub struct GameTeam(pub &'static str, pub RgbColor);
